use std::{
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    easing::EasingTable,
    settings::Settings,
    types::{AnimationMode, BreathingPhase},
};

// ─── Public state snapshot ────────────────────────────────────────────────────

/// Snapshot of the breathing animation at a point in time.
/// Read by the renderer every time it draws a frame.
#[derive(Clone, Copy, Debug)]
pub struct BreathingState {
    pub phase:     BreathingPhase,
    /// 0.0 = fully collapsed, 1.0 = fully expanded.
    pub progress:  f32,
    /// 0.0–1.0 elapsed fraction within the current hold phase (for ripple).
    pub hold_time: f32,
}

// ─── Cadence constants (exact match with MetalBreathingController.swift) ─────

const INTERVAL_FAST: Duration = Duration::from_nanos(41_666_667); // 1/24 s
const INTERVAL_SLOW: Duration = Duration::from_nanos(83_333_333); // 1/12 s

const ENTER_FAST_THRESHOLD: f32 = 0.0075;
const EXIT_FAST_THRESHOLD:  f32 = 0.0045;
const MIN_PROGRESS_DELTA:   f32 = 0.003;

// ─── Internal thread state ────────────────────────────────────────────────────

struct Inner {
    phase:         BreathingPhase,
    phase_start:   Instant,
    phase_duration: Duration,

    cycle_count:    u64,
    /// Running drift multiplier: starts at 1.0, multiplied by `drift` each cycle.
    current_drift:  f64,

    did_render_hold:    bool,
    is_fast_cadence:    bool,
    last_draw_time:     Instant,
    last_drawn_phase:   BreathingPhase,
    last_drawn_progress: f32,
}

// ─── Controller ──────────────────────────────────────────────────────────────

/// Drives the breathing animation timing on a dedicated background thread.
///
/// Direct port of `MetalBreathingController.swift`.  The thread sleeps between
/// ticks — it never spins — so CPU overhead between frames is near zero.
pub struct BreathingController {
    state:        Arc<Mutex<Option<BreathingState>>>,
    thread:       Option<JoinHandle<()>>,
    stop_flag:    Arc<std::sync::atomic::AtomicBool>,
    /// Set to restart the animation from inhale phase 0 on the next tick.
    reset_flag:   Arc<std::sync::atomic::AtomicBool>,
}

impl BreathingController {
    /// Create and immediately start the controller.
    ///
    /// `request_draw` is called from the background thread whenever a new frame
    /// should be rendered.  Wire it to your window's redraw mechanism (e.g.
    /// `EventLoopProxy::send_event`).
    pub fn start(
        settings: Arc<RwLock<Settings>>,
        request_draw: Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Self {
        let state      = Arc::new(Mutex::new(None));
        let stop_flag  = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let reset_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let state_clone    = Arc::clone(&state);
        let stop_clone     = Arc::clone(&stop_flag);
        let reset_clone    = Arc::clone(&reset_flag);
        let settings_clone = Arc::clone(&settings);

        let handle = thread::Builder::new()
            .name("exhale-controller".to_string())
            .spawn(move || {
                run_controller(settings_clone, state_clone, request_draw, stop_clone, reset_clone);
            })
            .expect("spawn controller thread");

        Self { state, thread: Some(handle), stop_flag, reset_flag }
    }

    /// Get a snapshot of the current breathing state for rendering.
    /// Returns `None` only before the first tick.
    pub fn get_state(&self) -> Option<BreathingState> {
        *self.state.lock().unwrap()
    }

    /// Restart the animation from inhale phase 0 on the next tick.
    /// Matches Swift `MetalBreathingController.start()` which always resets
    /// `cycleCount = 0` and `currentPhase = .inhale`.
    pub fn restart(&self) {
        self.reset_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Signal the controller to stop and join the thread.
    pub fn stop(&mut self) {
        self.stop_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.thread.take() {
            let _ = h.join();
        }
    }
}

impl Drop for BreathingController {
    fn drop(&mut self) {
        self.stop();
    }
}

// ─── Thread body ──────────────────────────────────────────────────────────────

fn run_controller(
    settings:     Arc<RwLock<Settings>>,
    state_out:    Arc<Mutex<Option<BreathingState>>>,
    request_draw: Arc<dyn Fn() + Send + Sync>,
    stop_flag:    Arc<std::sync::atomic::AtomicBool>,
    reset_flag:   Arc<std::sync::atomic::AtomicBool>,
) {
    let easing = EasingTable::default_ease_in_out();

    let inhale_dur = settings.read().unwrap().inhale_duration;
    let mut inner  = fresh_inner(Instant::now(), inhale_dur);

    loop {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        // restart() was called (e.g. user pressed Start): reset to inhale phase 0,
        // matching Swift MetalBreathingController.start() which resets cycleCount=0.
        if reset_flag.swap(false, std::sync::atomic::Ordering::Relaxed) {
            let inhale_dur = settings.read().unwrap().inhale_duration;
            inner = fresh_inner(Instant::now(), inhale_dur);
        }

        let (should_draw, next_interval) = tick(
            &mut inner,
            &settings,
            &easing,
        );

        if should_draw {
            // Write state snapshot before requesting draw so renderer sees it.
            let snap = compute_state(&inner);
            *state_out.lock().unwrap() = Some(snap);
            (request_draw)();
        }

        let sleep_for = next_interval.max(Duration::from_millis(1));
        thread::sleep(sleep_for);
    }
}

fn fresh_inner(now: Instant, inhale_dur: f64) -> Inner {
    Inner {
        phase:               BreathingPhase::Inhale,
        phase_start:         now,
        phase_duration:      Duration::from_secs_f64(inhale_dur.max(0.1)),
        cycle_count:         0,
        current_drift:       1.0,
        did_render_hold:     false,
        is_fast_cadence:     false,
        last_draw_time:      now - INTERVAL_FAST * 2,
        last_drawn_phase:    BreathingPhase::Inhale,
        last_drawn_progress: -1.0,
    }
}

// ─── Per-tick logic (exact port of `MetalBreathingController.tick`) ───────────

fn tick(
    inner:    &mut Inner,
    settings: &Arc<RwLock<Settings>>,
    easing:   &EasingTable,
) -> (bool, Duration) {
    let now = Instant::now();

    // Snapshot the fields we need; avoid holding the lock across sleeps.
    let (
        is_animating, is_paused, hold_ripple_enabled,
        shape_is_fullscreen, colors_match,
        inhale_dur, post_inhale_dur, exhale_dur, post_exhale_dur,
        drift, anim_mode,
        rand_inhale, rand_post_inhale, rand_exhale, rand_post_exhale,
    ) = {
        let s = settings.read().unwrap();
        (
            s.is_animating,
            s.is_paused,
            s.hold_ripple_mode != crate::types::HoldRippleMode::Off,
            s.shape == crate::types::AnimationShape::Fullscreen,
            s.inhale_exhale_colors_match(),
            s.inhale_duration,
            s.post_inhale_hold_duration,
            s.exhale_duration,
            s.post_exhale_hold_duration,
            s.drift,
            s.animation_mode,
            s.randomized_timing_inhale,
            s.randomized_timing_post_inhale_hold,
            s.randomized_timing_exhale,
            s.randomized_timing_post_exhale_hold,
        )
    };

    // ── Not animating ─────────────────────────────────────────────────────────
    if !is_animating && !is_paused {
        return (false, Duration::from_secs(10));
    }

    // ── Paused / static fullscreen ────────────────────────────────────────────
    if is_paused || (shape_is_fullscreen && colors_match) {
        let elapsed = now.duration_since(inner.last_draw_time);
        if elapsed >= Duration::from_secs(1) {
            inner.last_draw_time = now;
            inner.is_fast_cadence = false;
            return (true, Duration::from_secs(1));
        }
        let remaining = Duration::from_secs(1).saturating_sub(elapsed);
        return (false, remaining);
    }

    // ── Hold phase ────────────────────────────────────────────────────────────
    if inner.phase.is_hold() {
        let elapsed  = now.duration_since(inner.phase_start);
        let remaining = inner.phase_duration.saturating_sub(elapsed);

        if elapsed >= inner.phase_duration {
            advance_phase(
                inner, drift, now,
                inhale_dur, post_inhale_dur, exhale_dur, post_exhale_dur,
                rand_inhale, rand_post_inhale, rand_exhale, rand_post_exhale,
            );
            inner.did_render_hold = false;
            inner.is_fast_cadence = false;
            inner.last_draw_time  = now;
            return (true, INTERVAL_FAST);
        }

        if hold_ripple_enabled {
            let cadence = INTERVAL_FAST;
            if !inner.did_render_hold
                || now.duration_since(inner.last_draw_time) >= cadence
            {
                inner.did_render_hold = true;
                inner.last_draw_time  = now;
                return (true, cadence.min(remaining));
            }
            return (false, cadence.min(remaining));
        } else {
            // No ripple: render exactly once per hold, then sleep until it ends.
            if !inner.did_render_hold {
                inner.did_render_hold = true;
                inner.last_draw_time  = now;
                return (true, remaining);
            }
            return (false, remaining);
        }
    }

    // ── Inhale / Exhale ───────────────────────────────────────────────────────
    let elapsed = now.duration_since(inner.phase_start);
    if elapsed >= inner.phase_duration {
        advance_phase(
            inner, drift, now,
            inhale_dur, post_inhale_dur, exhale_dur, post_exhale_dur,
            rand_inhale, rand_post_inhale, rand_exhale, rand_post_exhale,
        );
        inner.did_render_hold = false;
        inner.is_fast_cadence = false;
    }

    let current = compute_state_with_easing(inner, easing, anim_mode, now);

    let phase_changed = current.phase != inner.last_drawn_phase;
    let never_drawn   = inner.last_drawn_progress < 0.0;
    let delta = (current.progress - inner.last_drawn_progress).abs();

    let should_draw = phase_changed || never_drawn || delta >= MIN_PROGRESS_DELTA;

    // Hysteresis: switch between fast/slow cadence based on delta.
    if inner.is_fast_cadence {
        if delta < EXIT_FAST_THRESHOLD {
            inner.is_fast_cadence = false;
        }
    } else if delta > ENTER_FAST_THRESHOLD {
        inner.is_fast_cadence = true;
    }

    let cadence = if inner.is_fast_cadence { INTERVAL_FAST } else { INTERVAL_SLOW };
    let phase_end = inner.phase_start + inner.phase_duration;
    let time_to_phase_end = phase_end.saturating_duration_since(now).max(Duration::from_millis(1));

    if should_draw {
        let elapsed_since_last = now.duration_since(inner.last_draw_time);
        if elapsed_since_last >= cadence {
            inner.last_draw_time        = now;
            inner.last_drawn_phase      = current.phase;
            inner.last_drawn_progress   = current.progress;
            return (true, cadence.min(time_to_phase_end));
        }
        // Not yet time; come back when cadence expires.
        let wait = cadence.saturating_sub(elapsed_since_last);
        return (false, wait.min(time_to_phase_end));
    }

    (false, cadence.min(time_to_phase_end))
}

// ─── State computation ────────────────────────────────────────────────────────

fn compute_state(inner: &Inner) -> BreathingState {
    compute_state_with_easing(inner, &EasingTable::default_ease_in_out(), AnimationMode::Sinusoidal, Instant::now())
}

fn compute_state_with_easing(
    inner:   &Inner,
    easing:  &EasingTable,
    mode:    AnimationMode,
    now:     Instant,
) -> BreathingState {
    let elapsed_secs = now
        .duration_since(inner.phase_start)
        .as_secs_f64();
    let dur_secs = inner.phase_duration.as_secs_f64().max(1e-6);
    let raw_t = (elapsed_secs / dur_secs).clamp(0.0, 1.0);

    let eased_t = if mode == AnimationMode::Linear {
        raw_t
    } else {
        easing.sample(raw_t)
    };

    let hold_time = raw_t as f32;

    // hold_time carries the linear phase-progress for all phases (not only
    // holds). The shader uses it to drive the cross-phase ripple fade during
    // the first 10% of inhale/exhale, matching Swift's
    // `withAnimation(.linear(duration: duration * 0.1)) { rippleOpacity = 0 }`.
    match inner.phase {
        BreathingPhase::Inhale => BreathingState {
            phase:    BreathingPhase::Inhale,
            progress: eased_t as f32,
            hold_time,
        },
        BreathingPhase::HoldAfterInhale => BreathingState {
            phase:    BreathingPhase::HoldAfterInhale,
            progress: 1.0,
            hold_time,
        },
        BreathingPhase::Exhale => BreathingState {
            phase:    BreathingPhase::Exhale,
            progress: (1.0 - eased_t) as f32,
            hold_time,
        },
        BreathingPhase::HoldAfterExhale => BreathingState {
            phase:    BreathingPhase::HoldAfterExhale,
            progress: 0.0,
            hold_time,
        },
    }
}

// ─── Phase advancement ────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn advance_phase(
    inner:              &mut Inner,
    drift:              f64,
    now:                Instant,
    inhale_dur:         f64,
    post_inhale_dur:    f64,
    exhale_dur:         f64,
    post_exhale_dur:    f64,
    rand_inhale:        f64,
    rand_post_inhale:   f64,
    rand_exhale:        f64,
    rand_post_exhale:   f64,
) {
    inner.phase = match inner.phase {
        BreathingPhase::Inhale          => BreathingPhase::HoldAfterInhale,
        BreathingPhase::HoldAfterInhale => BreathingPhase::Exhale,
        BreathingPhase::Exhale          => BreathingPhase::HoldAfterExhale,
        BreathingPhase::HoldAfterExhale => {
            inner.cycle_count  += 1;
            inner.current_drift *= drift;
            BreathingPhase::Inhale
        }
    };

    inner.phase_start    = now;
    inner.phase_duration = phase_duration_for(
        inner.phase,
        inner.current_drift,
        inhale_dur, post_inhale_dur, exhale_dur, post_exhale_dur,
        rand_inhale, rand_post_inhale, rand_exhale, rand_post_exhale,
    );
}

fn phase_duration_for(
    phase:            BreathingPhase,
    current_drift:    f64,
    inhale_dur:       f64,
    post_inhale_dur:  f64,
    exhale_dur:       f64,
    post_exhale_dur:  f64,
    rand_inhale:      f64,
    rand_post_inhale: f64,
    rand_exhale:      f64,
    rand_post_exhale: f64,
) -> Duration {
    let base = match phase {
        BreathingPhase::Inhale          => jitter(inhale_dur,      rand_inhale),
        BreathingPhase::HoldAfterInhale => jitter(post_inhale_dur, rand_post_inhale),
        BreathingPhase::Exhale          => jitter(exhale_dur,      rand_exhale),
        BreathingPhase::HoldAfterExhale => jitter(post_exhale_dur, rand_post_exhale),
    };
    Duration::from_secs_f64((base * current_drift).max(0.1))
}

fn jitter(base: f64, range: f64) -> f64 {
    if range <= 0.0 {
        return base;
    }
    let mut rng = rand::thread_rng();
    base + rng.gen_range(-range..=range)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    use crate::settings::Settings;

    fn make_settings() -> Arc<RwLock<Settings>> {
        Arc::new(RwLock::new(Settings::default()))
    }

    // Helper: advance N phases manually through the inner state machine.
    fn advance_n_phases(inner: &mut Inner, n: usize, settings: &Settings) {
        for _ in 0..n {
            let now = Instant::now();
            advance_phase(
                inner,
                settings.drift,
                now,
                settings.inhale_duration,
                settings.post_inhale_hold_duration,
                settings.exhale_duration,
                settings.post_exhale_hold_duration,
                0.0, 0.0, 0.0, 0.0,
            );
        }
    }

    #[test]
    fn phase_sequence_is_correct() {
        let settings = Settings::default();
        let now = Instant::now();
        let mut inner = Inner {
            phase:           BreathingPhase::Inhale,
            phase_start:     now,
            phase_duration:  Duration::from_secs_f64(settings.inhale_duration),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            is_fast_cadence: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        };

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::HoldAfterInhale);

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::Exhale);

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::HoldAfterExhale);

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::Inhale);
        assert_eq!(inner.cycle_count, 1);
    }

    #[test]
    fn drift_accumulates_correctly() {
        let settings = Settings::default(); // drift = 1.01
        let now = Instant::now();
        let mut inner = Inner {
            phase:           BreathingPhase::HoldAfterExhale,
            phase_start:     now,
            phase_duration:  Duration::from_millis(100),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            is_fast_cadence: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        };

        // One full cycle advance (HoldAfterExhale → Inhale)
        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.cycle_count, 1);
        let expected_drift = 1.01_f64;
        assert!(
            (inner.current_drift - expected_drift).abs() < 1e-9,
            "after 1 cycle drift={} expected {expected_drift}",
            inner.current_drift
        );

        // Two more cycles
        advance_n_phases(&mut inner, 4, &settings); // 4 phases = 1 more cycle
        assert_eq!(inner.cycle_count, 2);
        let expected = 1.01_f64 * 1.01_f64;
        assert!(
            (inner.current_drift - expected).abs() < 1e-9,
            "after 2 cycles drift={} expected {expected}",
            inner.current_drift
        );
    }

    #[test]
    fn drift_matches_pow() {
        // current_drift should equal drift^cycle_count at every cycle boundary.
        let settings = Settings::default();
        let now = Instant::now();
        let mut inner = Inner {
            phase:           BreathingPhase::HoldAfterExhale,
            phase_start:     now,
            phase_duration:  Duration::from_millis(10),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            is_fast_cadence: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        };

        for cycle in 0..10_u64 {
            // advance one full cycle (4 phases)
            advance_n_phases(&mut inner, 4, &settings);
            let expected = settings.drift.powi((cycle + 1) as i32);
            assert!(
                (inner.current_drift - expected).abs() < 1e-9,
                "cycle={cycle}: drift={} pow={expected}",
                inner.current_drift
            );
        }
    }

    #[test]
    fn progress_range_inhale() {
        let easing = EasingTable::default_ease_in_out();
        let now = Instant::now();
        let inner = Inner {
            phase:           BreathingPhase::Inhale,
            phase_start:     now,
            phase_duration:  Duration::from_secs(5),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            is_fast_cadence: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        };
        let state = compute_state_with_easing(&inner, &easing, AnimationMode::Sinusoidal, now);
        assert!((state.progress - 0.0).abs() < 0.01, "inhale starts at 0");
    }

    #[test]
    fn progress_range_exhale_starts_at_one() {
        let easing = EasingTable::default_ease_in_out();
        let now = Instant::now();
        let inner = Inner {
            phase:           BreathingPhase::Exhale,
            phase_start:     now,
            phase_duration:  Duration::from_secs(10),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            is_fast_cadence: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Exhale,
            last_drawn_progress: -1.0,
        };
        let state = compute_state_with_easing(&inner, &easing, AnimationMode::Sinusoidal, now);
        assert!((state.progress - 1.0).abs() < 0.01, "exhale starts at 1");
    }

    #[test]
    fn hold_after_inhale_progress_is_one() {
        let easing = EasingTable::default_ease_in_out();
        let now = Instant::now();
        let inner = Inner {
            phase:           BreathingPhase::HoldAfterInhale,
            phase_start:     now,
            phase_duration:  Duration::from_secs(4),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            is_fast_cadence: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::HoldAfterInhale,
            last_drawn_progress: -1.0,
        };
        let state = compute_state_with_easing(&inner, &easing, AnimationMode::Sinusoidal, now);
        assert_eq!(state.progress, 1.0);
    }

    #[test]
    fn jitter_stays_in_range() {
        for _ in 0..1000 {
            let v = jitter(5.0, 1.0);
            assert!(v >= 4.0 && v <= 6.0, "jitter out of range: {v}");
        }
    }

    #[test]
    fn jitter_zero_range_is_exact() {
        for _ in 0..100 {
            assert_eq!(jitter(5.0, 0.0), 5.0);
        }
    }

    #[test]
    fn phase_duration_minimum_is_100ms() {
        // Even with drift=0 or base=0, duration must be ≥ 0.1s.
        let dur = phase_duration_for(
            BreathingPhase::Inhale,
            0.0, // impossible drift, tests the clamp
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        );
        assert!(dur >= Duration::from_millis(100));
    }
}
