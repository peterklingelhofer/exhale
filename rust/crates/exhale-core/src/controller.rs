use std::{
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    easing::EasingTable,
    poison::{MutexPoisonExt, RwLockPoisonExt},
    settings::Settings,
    types::{AnimationMode, BreathingPhase},
};

// ─── Public state snapshot ────────────────────────────────────────────────────

/// Snapshot of the breathing animation at a point in time. Read by
/// the renderer every time it draws a frame
#[derive(Clone, Copy, Debug)]
pub struct BreathingState {
    pub phase:     BreathingPhase,
    /// 0.0 = fully collapsed, 1.0 = fully expanded
    pub progress:  f32,
    /// 0.0–1.0 elapsed fraction within the current hold phase (for ripple)
    pub hold_time: f32,
}

// ─── Cadence ─────────────────────────────────────────────────────────────────
//
// 24 fps matches Swift's `MetalBreathingController.swift`
// `maximumDrawIntervalFast`.  We don't run a slower "near-hold"
// cadence anymore: bench measurements showed the slow-cadence path
// fired for <10 % of clock time and saved well under 0.1 % CPU, which
// is below the noise floor of any user-facing measurement and not
// worth the extra hysteresis state
const INTERVAL_FAST: Duration = Duration::from_nanos(41_666_667);  // 1/24 s

const MIN_PROGRESS_DELTA: f32 = 0.003;

// ─── Internal thread state ────────────────────────────────────────────────────

struct Inner {
    phase:         BreathingPhase,
    phase_start:   Instant,
    phase_duration: Duration,

    cycle_count:    u64,
    /// Running drift multiplier: starts at 1.0, multiplied by `drift` each cycle
    current_drift:  f64,

    did_render_hold:    bool,
    last_draw_time:     Instant,
    last_drawn_phase:   BreathingPhase,
    last_drawn_progress: f32,
}

// ─── Controller ──────────────────────────────────────────────────────────────

/// Drives the breathing animation timing on a dedicated background thread
///
/// Direct port of `MetalBreathingController.swift`.  The thread sleeps
/// between ticks, never spins, so CPU overhead between frames is near zero
pub struct BreathingController {
    state:        Arc<Mutex<Option<BreathingState>>>,
    thread:       Option<JoinHandle<()>>,
    stop_flag:    Arc<std::sync::atomic::AtomicBool>,
    /// Set to restart the animation from inhale phase 0 on the next tick
    reset_flag:   Arc<std::sync::atomic::AtomicBool>,
}

impl BreathingController {
    /// Create and immediately start the controller
    ///
    /// `state` is the shared snapshot slot the controller writes to each
    /// tick, pre-constructed so per-overlay render threads can hold the
    /// same `Arc` and read directly without round-tripping through the
    /// main event loop.  `request_draw` is called from the background
    /// thread whenever a new frame should be rendered; wire it to the
    /// overlays' render-thread channels so frame signals bypass the
    /// main thread's message pump entirely
    pub fn start(
        settings:     Arc<RwLock<Settings>>,
        state:        Arc<Mutex<Option<BreathingState>>>,
        request_draw: Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Self {
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
            .expect(
                "exhale-controller: thread::spawn failed (system thread limit / OOM): \
                 can't continue without the breathing controller; restart the process \
                 once memory is available",
            );

        Self { state, thread: Some(handle), stop_flag, reset_flag }
    }

    /// Get a snapshot of the current breathing state for rendering. Returns
    /// `None` only before the first tick
    pub fn get_state(&self) -> Option<BreathingState> {
        *self.state.lock_or_recover()
    }

    /// Shared handle to the controller's state slot.  Cheap to clone;
    /// the per-overlay render thread reads from this directly each
    /// frame instead of round-tripping through the main event loop. The
    /// controller writes to this BEFORE invoking `request_draw`,
    /// so any thread woken by `request_draw` is guaranteed to observe
    /// the latest state via the Mutex barrier
    pub fn state_handle(&self) -> Arc<Mutex<Option<BreathingState>>> {
        Arc::clone(&self.state)
    }

    /// Restart the animation from inhale phase 0 on the next tick. Matches
    /// Swift `MetalBreathingController.start()` which always resets
    /// `cycleCount = 0` and `currentPhase = .inhale`
    ///
    /// `unpark` wakes the controller out of its current
    /// `park_timeout` sleep so the reset takes effect immediately,
    /// not on the controller's next natural wakeup.  This matters
    /// most when `restart()` is called from the Stop -> Start
    /// sequence: while `is_animating == false`, the tick function
    /// returns a 10 s sleep interval (no work to do), and without
    /// the unpark the user would see the animation start mid-cycle
    /// in whatever state the renderer had cached and stay frozen
    /// there for up to 10 s, a bug that read as "Start is
    /// broken / animation is paused"
    pub fn restart(&self) {
        self.reset_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = &self.thread {
            h.thread().unpark();
        }
    }

    /// Signal the controller to stop and join the thread.  Same
    /// `unpark` rationale as [`Self::restart`]; without it,
    /// shutdown could wait up to 10 s for the controller's
    /// not-animating sleep to elapse before the join returns
    pub fn stop(&mut self) {
        self.stop_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.thread.take() {
            h.thread().unpark();
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

    let inhale_dur = settings.read_or_recover().inhale_duration;
    let mut inner  = fresh_inner(Instant::now(), inhale_dur);

    // Deadline the next iteration should fire at.  We advance this by the
    // tick's requested `next_interval` after every iteration, then sleep
    // until that deadline rather than sleeping `next_interval` from "now". Without
    // this, every `thread::sleep` overshoot (typically 0-10 ms on macOS)
    // accumulated into perceived choppiness even at the same fps: frames
    // landed at irregular wall-clock times.  Deadline-based scheduling
    // means a 5-ms-late wake-up is followed by a 5-ms-shorter sleep, so
    // the long-run cadence stays exact and the eye reads the motion as
    // smooth at the same render rate / CPU cost
    let mut next_wakeup = Instant::now();

    loop {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        // restart() was called (e.g. user pressed Start): reset to inhale phase 0,
        // matching Swift MetalBreathingController.start() which resets cycleCount=0
        if reset_flag.swap(false, std::sync::atomic::Ordering::Relaxed) {
            let inhale_dur = settings.read_or_recover().inhale_duration;
            inner = fresh_inner(Instant::now(), inhale_dur);
            next_wakeup = Instant::now();
        }

        let (should_draw, next_interval) = tick(
            &mut inner,
            &settings,
            &easing,
        );

        if should_draw {
            // Write state snapshot before requesting draw so renderer sees it
            let snap = compute_state(&inner);
            *state_out.lock_or_recover() = Some(snap);
            (request_draw)();
        }

        // Advance the target deadline by exactly `next_interval`.  Catch-up
        // clamp: if a long pause (laptop sleep, app backgrounded) put us
        // more than 1 second past the target, snap forward instead of
        // burst-rendering frames to "make up" the missed time
        next_wakeup += next_interval;
        let now = Instant::now();
        if next_wakeup + Duration::from_secs(1) < now {
            next_wakeup = now + next_interval;
        }
        let sleep_for = next_wakeup
            .saturating_duration_since(now)
            .max(Duration::from_millis(1));
        // `park_timeout` instead of `thread::sleep` so `restart()` /
        // `stop()` can wake us via `unpark()` mid-sleep.  When
        // `is_animating == false` the tick function returns a 10 s
        // sleep interval, and without interruptible sleep a Stop -> 
        // Start press would wait up to 10 s before the controller
        // noticed the reset_flag and started ticking again.  If
        // unpark was called BEFORE we reached the park, the token
        // is already pending and park_timeout returns immediately,
        // which is exactly what we want (don't lose a wakeup
        // signal that arrived during the previous tick's work)
        thread::park_timeout(sleep_for);
    }
}

fn fresh_inner(now: Instant, inhale_dur: f64) -> Inner {
    // A zero-length inhale takes zero time, so the first tick finds the
    // phase already over and advances to whatever the user did configure
    let phase_duration = if inhale_dur <= 0.0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(inhale_dur.max(0.1))
    };
    Inner {
        phase:               BreathingPhase::Inhale,
        phase_start:         now,
        phase_duration,
        cycle_count:         0,
        current_drift:       1.0,
        did_render_hold:     false,
        // Park `last_draw_time` 200 ms in the past, comfortably older
        // than the 1/24 s cadence so the first tick is immediately due
        last_draw_time:      now.checked_sub(Duration::from_millis(200)).unwrap_or(now),
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

    // Snapshot the fields we need; avoid holding the lock across sleeps
    let (
        is_animating, is_paused, hold_ripple_enabled,
        shape_is_fullscreen, colors_match,
        inhale_dur, post_inhale_dur, exhale_dur, post_exhale_dur,
        drift, anim_mode,
        rand_inhale, rand_post_inhale, rand_exhale, rand_post_exhale,
    ) = {
        let s = settings.read_or_recover();
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

    // ── Paused / static fullscreen / all-zero-duration tint ──────────────────
    //
    // `cycle_is_static` catches the degenerate input where the user
    // has zeroed every duration field.  Without this short-circuit
    // every phase would be zero length, so `advance_phase` would walk
    // all four and hand back the same zero-length phase it started on,
    // and the next tick would do it again: a whole cycle per frame,
    // spent on an animation that never moves.  Treating it as static
    // instead: the existing one-frame-per-second cadence renders
    // whatever the last `BreathingState` was, the shader keeps drawing
    // that state, and CPU stays as low as the matching-colour
    // fullscreen tint path.  Threshold is 0.05 s (50 ms), below the
    // 0.1 s floor a phase the user did configure still gets and below
    // human flicker-fusion frequency, so anything the user could
    // meaningfully type as "a real animation" stays above it
    let cycle_is_static = inhale_dur < 0.05
        && post_inhale_dur < 0.05
        && exhale_dur      < 0.05
        && post_exhale_dur < 0.05;

    if is_paused || (shape_is_fullscreen && colors_match) || cycle_is_static {
        let elapsed = now.duration_since(inner.last_draw_time);
        if elapsed >= Duration::from_secs(1) {
            inner.last_draw_time = now;
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
            inner.last_draw_time  = now;
            return (true, INTERVAL_FAST);
        }

        if hold_ripple_enabled {
            if !inner.did_render_hold
                || now.duration_since(inner.last_draw_time) >= INTERVAL_FAST
            {
                inner.did_render_hold = true;
                inner.last_draw_time  = now;
                return (true, INTERVAL_FAST.min(remaining));
            }
            return (false, INTERVAL_FAST.min(remaining));
        } else {
            // No ripple: render exactly once per hold, then sleep until it ends
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
    }

    let current = compute_state_with_easing(inner, easing, anim_mode, now);

    let phase_changed = current.phase != inner.last_drawn_phase;
    let never_drawn   = inner.last_drawn_progress < 0.0;
    let delta = (current.progress - inner.last_drawn_progress).abs();

    let should_draw = phase_changed || never_drawn || delta >= MIN_PROGRESS_DELTA;

    let cadence = INTERVAL_FAST;
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
        // Not yet time; come back when cadence expires
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

    // hold_time carries the linear phase-progress for all phases, inhale
    // and exhale included. The shader uses it to drive the cross-phase
    // ripple fade during the first 10% of inhale/exhale, matching Swift's
    // `withAnimation(.linear(duration: duration * 0.1)) { rippleOpacity = 0 }`
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

/// Move to the next phase the user actually configured
///
/// A phase set to 0 is passed over rather than rendered for a floored
/// 0.1 s, so 5 / 0 / 5 / 0 is a ten-second cycle instead of a 10.2-second
/// one.  At most four steps: an all-zero pattern comes back round to the
/// phase it started on instead of spinning forever.  `tick` never gets
/// one this far because `cycle_is_static` short-circuits first, but a
/// function that can spin is a function that will
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
    for _ in 0..4 {
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

        if inner.phase_duration > Duration::ZERO {
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    let (base, fraction) = match phase {
        BreathingPhase::Inhale          => (inhale_dur,      rand_inhale),
        BreathingPhase::HoldAfterInhale => (post_inhale_dur, rand_post_inhale),
        BreathingPhase::Exhale          => (exhale_dur,      rand_exhale),
        BreathingPhase::HoldAfterExhale => (post_exhale_dur, rand_post_exhale),
    };
    // A phase set to 0 takes no time.  The 0.1 s floor below is the
    // anti-strobe guarantee, and it's only for a phase the user asked for
    if base <= 0.0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64((jitter(base, fraction) * current_drift).max(0.1))
}

/// Perturb `base` by up to ±`fraction` of itself
///
/// The stored slider value is 0.0 to 1.0 and the settings window shows it
/// as a percent.  Scaling the phase rather than adding seconds to it means
/// the slider means the same thing on a 2 s hold as on a 10 s exhale, and
/// a phase set to 0 stays 0
fn jitter(base: f64, fraction: f64) -> f64 {
    if fraction <= 0.0 || base <= 0.0 {
        return base;
    }
    let fraction = fraction.min(1.0);
    let mut rng = rand::thread_rng();
    base * (1.0 + rng.gen_range(-fraction..=fraction))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    use crate::settings::Settings;

    // Helper: advance N phases manually through the inner state machine
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
        // Every phase non-zero, so each advance is one step. What the
        // default 5 / 0 / 5 / 0 does with its zero holds is covered by
        // `zero_length_holds_are_skipped` below
        let mut settings = Settings::default();
        settings.inhale_duration           = 4.0;
        settings.post_inhale_hold_duration = 4.0;
        settings.exhale_duration           = 4.0;
        settings.post_exhale_hold_duration = 4.0;
        let now = Instant::now();
        let mut inner = Inner {
            phase:           BreathingPhase::Inhale,
            phase_start:     now,
            phase_duration:  Duration::from_secs_f64(settings.inhale_duration),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
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
    fn the_default_cycle_is_exactly_ten_seconds() {
        // 5 / 0 / 5 / 0 is 6.0 breaths a minute, the number the readout
        // and the README quote. Flooring each hold to 0.1 s used to make
        // it a 10.2 s cycle and 5.88 a minute
        let settings = Settings::default();
        let mut inner = fresh_inner(Instant::now(), settings.inhale_duration);

        let mut total   = Duration::ZERO;
        let mut visited = Vec::new();
        for _ in 0..8 {
            if inner.cycle_count > 0 {
                break;
            }
            total += inner.phase_duration;
            visited.push(inner.phase);
            advance_n_phases(&mut inner, 1, &settings);
        }

        assert_eq!(inner.cycle_count, 1, "never completed a cycle: {visited:?}");
        assert_eq!(total, Duration::from_secs(10), "visited {visited:?}");
        assert_eq!(
            visited,
            vec![BreathingPhase::Inhale, BreathingPhase::Exhale],
            "a zero-length hold should never be a phase we sit in"
        );
    }

    #[test]
    fn zero_length_holds_are_skipped() {
        let settings = Settings::default();
        let mut inner = fresh_inner(Instant::now(), settings.inhale_duration);

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::Exhale, "the 0 s hold is passed over");
        assert_eq!(inner.phase_duration, Duration::from_secs(5));
        assert_eq!(inner.cycle_count, 0);

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::Inhale);
        assert_eq!(inner.cycle_count, 1, "the skipped hold still closes the cycle");
    }

    #[test]
    fn an_all_zero_pattern_advances_once_round_and_stops() {
        // `tick` short-circuits on `cycle_is_static` long before this,
        // but `advance_phase` has to terminate on its own
        let mut settings = Settings::default();
        settings.inhale_duration           = 0.0;
        settings.post_inhale_hold_duration = 0.0;
        settings.exhale_duration           = 0.0;
        settings.post_exhale_hold_duration = 0.0;
        let mut inner = fresh_inner_at(Instant::now(), Duration::ZERO);

        advance_n_phases(&mut inner, 1, &settings);
        assert_eq!(inner.phase, BreathingPhase::Inhale);
        assert_eq!(inner.cycle_count, 1);
        assert_eq!(inner.phase_duration, Duration::ZERO);
    }

    #[test]
    fn drift_accumulates_correctly() {
        // Drift is off by default now, so set it explicitly: this test
        // covers compounding, independent of what ships as the default
        let mut settings = Settings::default();
        settings.drift = 1.01;
        // Non-zero holds so that four advances are still exactly one
        // cycle: a hold left at 0 is skipped rather than stepped through
        settings.post_inhale_hold_duration = 1.0;
        settings.post_exhale_hold_duration = 1.0;
        let now = Instant::now();
        let mut inner = Inner {
            phase:           BreathingPhase::HoldAfterExhale,
            phase_start:     now,
            phase_duration:  Duration::from_millis(100),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        };

        // One full cycle advance (HoldAfterExhale -> Inhale)
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
        // current_drift should equal drift^cycle_count at every cycle boundary,
        // for as long as the ceiling has not been reached
        let mut settings = Settings::default();
        settings.drift = 1.01;
        // Non-zero holds, same reason as `drift_accumulates_correctly`
        settings.post_inhale_hold_duration = 1.0;
        settings.post_exhale_hold_duration = 1.0;
        let now = Instant::now();
        let mut inner = Inner {
            phase:           BreathingPhase::HoldAfterExhale,
            phase_start:     now,
            phase_duration:  Duration::from_millis(10),
            cycle_count:     0,
            current_drift:   1.0,
            did_render_hold: false,
            last_draw_time:  now,
            last_drawn_phase: BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        };

        // Deliberately far more cycles than any plausible session: drift is
        // unbounded by design, so this must hold arbitrarily far out. An
        // earlier revision capped the compounding; that cap was removed
        // because advanced pranayama practice legitimately reaches breaths
        // far longer than the research literature happens to have studied. See
        // docs/CITATIONS.md gaps ledger item 6
        for cycle in 0..2_000_u64 {
            // advance one full cycle (4 phases)
            advance_n_phases(&mut inner, 4, &settings);
            let expected = settings.drift.powi((cycle + 1) as i32);
            // A relative tolerance: repeated multiplication and `powi` agree
            // to ~13 significant figures, but drift is unbounded, so by cycle
            // ~1200 the values are in the hundreds of thousands and a fixed
            // 1e-9 epsilon compares the wrong thing entirely
            assert!(
                (inner.current_drift - expected).abs() <= expected * 1e-12,
                "cycle={cycle}: drift={} pow={expected}",
                inner.current_drift
            );
        }
    }

    #[test]
    fn drift_still_compounds_across_skipped_holds() {
        // Skipping a zero-length hold must not skip the cycle bookkeeping
        let mut settings = Settings::default();
        settings.drift = 1.01;
        let mut inner = fresh_inner(Instant::now(), settings.inhale_duration);

        // Default 5 / 0 / 5 / 0: two advances per cycle, so four is two
        advance_n_phases(&mut inner, 4, &settings);
        assert_eq!(inner.cycle_count, 2);
        assert_eq!(inner.phase, BreathingPhase::Inhale);

        let expected = 1.01_f64 * 1.01_f64;
        assert!(
            (inner.current_drift - expected).abs() < 1e-9,
            "after 2 cycles drift={} expected {expected}",
            inner.current_drift
        );
        let inhale_secs = inner.phase_duration.as_secs_f64();
        assert!(
            (inhale_secs - 5.0 * expected).abs() < 1e-6,
            "inhale is {inhale_secs} s, expected {} s",
            5.0 * expected
        );
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
            let v = jitter(5.0, 0.2);
            assert!((4.0..=6.0).contains(&v), "jitter out of range: {v}");
        }
        for _ in 0..1000 {
            let v = jitter(5.0, 1.0);
            assert!((0.0..=10.0).contains(&v), "jitter out of range: {v}");
        }
    }

    #[test]
    fn jitter_is_a_fraction_of_the_phase() {
        // The slider stores 0.0 to 1.0 and the window shows it as a
        // percent, so 50 % of a 2 s phase is a second either way
        for _ in 0..1000 {
            let v = jitter(2.0, 0.5);
            assert!((1.0..=3.0).contains(&v), "jitter out of range: {v}");
        }
        assert_eq!(jitter(0.0, 0.5), 0.0, "a zero-length phase stays zero");
        for _ in 0..1000 {
            let v = jitter(5.0, 1.5);
            assert!((0.0..=10.0).contains(&v), "fraction should clamp at 1.0: {v}");
        }
    }

    #[test]
    fn jitter_zero_range_is_exact() {
        for _ in 0..100 {
            assert_eq!(jitter(5.0, 0.0), 5.0);
        }
    }

    #[test]
    fn a_non_zero_phase_is_floored_and_a_zero_phase_is_zero() {
        // A phase the user asked for never renders shorter than 0.1 s
        let tiny = phase_duration_for(
            BreathingPhase::Inhale,
            1.0,
            0.01, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        );
        assert_eq!(tiny, Duration::from_millis(100));

        let clamped = phase_duration_for(
            BreathingPhase::Inhale,
            0.0, // impossible drift, tests the clamp
            5.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        );
        assert_eq!(clamped, Duration::from_millis(100));

        // A phase set to 0 gets no floor: it takes no time at all
        let zero = phase_duration_for(
            BreathingPhase::HoldAfterInhale,
            1.0,
            5.0, 0.0, 5.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        );
        assert_eq!(zero, Duration::ZERO);
    }

    // ── tick() cadence / hysteresis tests ─────────────────────────────────
    //
    // These exercise the per-tick scheduler logic that decides whether
    // to render *this* tick and how long to sleep before the next. Prior
    // to these tests `tick()` had zero coverage despite being the
    // hottest function in the app

    fn fresh_inner_at(now: Instant, dur: Duration) -> Inner {
        Inner {
            phase:               BreathingPhase::Inhale,
            phase_start:         now,
            phase_duration:      dur,
            cycle_count:         0,
            current_drift:       1.0,
            did_render_hold:     false,
            last_draw_time:      now - Duration::from_secs(1),
            last_drawn_phase:    BreathingPhase::Inhale,
            last_drawn_progress: -1.0,
        }
    }

    #[test]
    fn tick_returns_no_draw_when_not_animating_or_paused() {
        let mut s = Settings::default();
        s.is_animating = false;
        s.is_paused = false;
        let settings = Arc::new(RwLock::new(s));
        let easing   = EasingTable::default_ease_in_out();
        let mut inner = fresh_inner_at(Instant::now(), Duration::from_secs(5));
        let (should_draw, next) = tick(&mut inner, &settings, &easing);
        assert!(!should_draw, "a stopped controller shouldn't request a draw");
        assert!(next >= Duration::from_secs(1), "stopped controller should sleep >=1s, got {next:?}");
    }

    #[test]
    fn tick_paused_renders_once_per_second() {
        let mut s = Settings::default();
        s.is_paused = true;
        let settings = Arc::new(RwLock::new(s));
        let easing   = EasingTable::default_ease_in_out();
        let mut inner = fresh_inner_at(Instant::now(), Duration::from_secs(5));
        // First call: last_draw_time was 1s ago, so this should draw
        // (the paused branch redraws once a second to keep the static
        // frame current against settings changes)
        let (should_draw_1, _) = tick(&mut inner, &settings, &easing);
        assert!(should_draw_1, "paused controller draws once per second");
        // Second call immediately after: not yet a second elapsed, so
        // it should NOT draw and the sleep should be < 1s
        let (should_draw_2, next_2) = tick(&mut inner, &settings, &easing);
        assert!(!should_draw_2, "paused controller doesn't draw twice in a row");
        assert!(next_2 < Duration::from_secs(1));
    }

    #[test]
    fn tick_hold_with_ripple_redraws_periodically() {
        // Hold-with-ripple: should draw at `interval_fast` cadence
        // continuously through the hold, boundaries included
        let mut s = Settings::default();
        s.hold_ripple_mode = crate::types::HoldRippleMode::Gradient;
        s.post_inhale_hold_duration = 2.0; // long hold for the test
        let settings = Arc::new(RwLock::new(s));
        let easing   = EasingTable::default_ease_in_out();
        let now = Instant::now();
        let mut inner = fresh_inner_at(now, Duration::from_secs_f64(2.0));
        inner.phase = BreathingPhase::HoldAfterInhale;
        inner.last_draw_time = now - Duration::from_millis(200); // not yet due

        let (should_draw, _) = tick(&mut inner, &settings, &easing);
        // First tick of a fresh hold draws (did_render_hold was false)
        assert!(should_draw);
    }

    #[test]
    fn tick_hold_without_ripple_renders_once_then_sleeps() {
        let mut s = Settings::default();
        s.hold_ripple_mode = crate::types::HoldRippleMode::Off;
        s.post_inhale_hold_duration = 2.0;
        let settings = Arc::new(RwLock::new(s));
        let easing   = EasingTable::default_ease_in_out();
        let now = Instant::now();
        let mut inner = fresh_inner_at(now, Duration::from_secs_f64(2.0));
        inner.phase = BreathingPhase::HoldAfterInhale;

        let (should_draw_1, _) = tick(&mut inner, &settings, &easing);
        assert!(should_draw_1, "first tick of no-ripple hold draws");
        let (should_draw_2, _) = tick(&mut inner, &settings, &easing);
        assert!(!should_draw_2, "second tick of no-ripple hold sleeps");
    }

    #[test]
    fn a_zero_inhale_starts_on_the_next_phase() {
        // 0 / 2 / 2 / 0: there's no inhale to draw, so the very first
        // tick moves on to the hold instead of sitting out 100 ms
        let mut s = Settings::default();
        s.inhale_duration           = 0.0;
        s.post_inhale_hold_duration = 2.0;
        s.exhale_duration           = 2.0;
        s.post_exhale_hold_duration = 0.0;
        s.is_animating = true;
        s.is_paused    = false;
        let settings = Arc::new(RwLock::new(s));
        let easing   = EasingTable::default_ease_in_out();

        let mut inner = fresh_inner(Instant::now(), 0.0);
        assert_eq!(inner.phase_duration, Duration::ZERO);

        tick(&mut inner, &settings, &easing);
        assert_eq!(inner.phase, BreathingPhase::HoldAfterInhale);
    }

    #[test]
    fn cadence_matches_swift_reference() {
        // 24 fps = 1/24 s = ~41.67 ms.  Matches `MetalBreathingController.swift`'s
        // `maximumDrawIntervalFast`
        let ms = INTERVAL_FAST.as_nanos() as f64 / 1_000_000.0;
        assert!((ms - 1000.0 / 24.0).abs() < 0.01,
            "INTERVAL_FAST should be 1/24 s = 41.67 ms, got {ms}");
    }
}
