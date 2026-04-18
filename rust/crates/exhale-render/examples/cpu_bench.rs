//! Headless CPU benchmark — mirrors the Swift `measureCPU` suite.
//!
//! Runs the real `BreathingController` driving a `HeadlessRenderer` that
//! draws into an offscreen 1920×1080 Bgra8Unorm texture.  This is not a
//! like-for-like replacement for the live-process sampler — there is no
//! window, no compositor, no swapchain present — but it isolates the
//! controller + render cost of the Rust port and gives a stable number
//! that is directly comparable between local runs.
//!
//! Methodology (matches `exhale/swift/exhaleTests.swift::measureCPU`):
//!   1. Warm up 0.5 s.
//!   2. Baseline phase: `is_animating = false`, 5 × 1 s samples via
//!      `getrusage(RUSAGE_SELF)`, averaged.
//!   3. Animation phase: `is_animating = true`, another 5 × 1 s samples.
//!   4. Delta = max(0, animation − baselineAvg) for each sample.
//!   5. Print per-variant peak + avg delta.
//!
//! Usage:
//!   cargo run --release --example cpu_bench -p exhale-render
//!   cargo run --release --example cpu_bench -p exhale-render -- only=circle_ripple

use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use exhale_core::{
    AnimationShape, BreathingController, ColorFillGradient, HoldRippleMode, Settings,
};
use exhale_render::{GpuContext, HeadlessRenderer};

const WIDTH:          u32     = 1920;
const HEIGHT:         u32     = 1080;
const SAMPLE_COUNT:   usize   = 5;
const SAMPLE_SECONDS: f64     = 1.0;
const WARMUP_SECONDS: f64     = 0.5;

fn get_cpu_seconds() -> f64 {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage); }
    let u = usage.ru_utime.tv_sec as f64 + usage.ru_utime.tv_usec as f64 / 1_000_000.0;
    let s = usage.ru_stime.tv_sec as f64 + usage.ru_stime.tv_usec as f64 / 1_000_000.0;
    u + s
}

/// Sleep for `dur` and return CPU% used by *this whole process* during the sleep.
fn sample_cpu(dur: Duration) -> f64 {
    let cpu_before  = get_cpu_seconds();
    let wall_before = Instant::now();
    thread::sleep(dur);
    let wall_elapsed = wall_before.elapsed().as_secs_f64();
    let cpu_used     = get_cpu_seconds() - cpu_before;
    (cpu_used / wall_elapsed) * 100.0
}

struct Variant {
    label:    &'static str,
    tag:      &'static str,
    shape:    AnimationShape,
    gradient: ColorFillGradient,
    ripple:   HoldRippleMode,
    hold:     f64,
}

fn variants() -> Vec<Variant> {
    vec![
        Variant { label: "rect    + gradient",             tag: "rect_grad",       shape: AnimationShape::Rectangle,  gradient: ColorFillGradient::On,  ripple: HoldRippleMode::Off,      hold: 0.0 },
        Variant { label: "circle  + gradient",             tag: "circle_grad",     shape: AnimationShape::Circle,     gradient: ColorFillGradient::On,  ripple: HoldRippleMode::Off,      hold: 0.0 },
        Variant { label: "fullscr + solid",                tag: "fullscreen",      shape: AnimationShape::Fullscreen, gradient: ColorFillGradient::Off, ripple: HoldRippleMode::Off,      hold: 0.0 },
        Variant { label: "rect    + hold ripple gradient", tag: "rect_ripple",     shape: AnimationShape::Rectangle,  gradient: ColorFillGradient::On,  ripple: HoldRippleMode::Gradient, hold: 4.0 },
        Variant { label: "rect    + hold ripple stark",    tag: "rect_ripple_stark", shape: AnimationShape::Rectangle, gradient: ColorFillGradient::Off, ripple: HoldRippleMode::Stark,    hold: 4.0 },
        Variant { label: "circle  + hold ripple gradient", tag: "circle_ripple",   shape: AnimationShape::Circle,     gradient: ColorFillGradient::On,  ripple: HoldRippleMode::Gradient, hold: 4.0 },
    ]
}

fn run_variant(v: &Variant) {
    // Build settings with animation ON — matches the real-app default.
    // Baseline phase intentionally does NOT spawn the controller, so the
    // baseline measures pure process idle (no controller thread, no GPU
    // work).  Delta = animation + render cost above that floor.
    let mut s = Settings::default();
    s.shape                = v.shape;
    s.color_fill_gradient  = v.gradient;
    s.hold_ripple_mode     = v.ripple;
    s.overlay_opacity      = 0.25;
    s.is_animating         = true;
    s.is_paused            = false;
    if v.hold > 0.0 {
        s.post_inhale_hold_duration = v.hold;
        s.post_exhale_hold_duration = v.hold;
        s.inhale_duration           = 2.0;
        s.exhale_duration           = 2.0;
    }
    let settings = Arc::new(RwLock::new(s));

    let gpu      = GpuContext::new_headless().expect("gpu context");
    let renderer = Arc::new(Mutex::new(
        HeadlessRenderer::new(Arc::clone(&gpu), WIDTH, HEIGHT).expect("headless renderer"),
    ));
    let frames   = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let max_scale = WIDTH as f32 / HEIGHT as f32;

    // ── Phase 1: baseline (no controller running) ────────────────────────────
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    thread::sleep(Duration::from_secs_f64(WARMUP_SECONDS));
    for _ in 0..SAMPLE_COUNT {
        baseline.push(sample_cpu(Duration::from_secs_f64(SAMPLE_SECONDS)));
    }
    let baseline_avg = baseline.iter().sum::<f64>() / baseline.len() as f64;

    // ── Phase 2: start controller + measure animation ────────────────────────
    // Weak-ref slot breaks the controller-callback reference cycle.
    let ctrl_slot: Arc<RwLock<Option<std::sync::Weak<BreathingController>>>> =
        Arc::new(RwLock::new(None));

    let renderer_cb = Arc::clone(&renderer);
    let settings_cb = Arc::clone(&settings);
    let ctrl_cb     = Arc::clone(&ctrl_slot);
    let frames_cb   = Arc::clone(&frames);

    let request_draw: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(move || {
        let weak = {
            let g = ctrl_cb.read().unwrap();
            match g.as_ref() { Some(w) => w.clone(), None => return }
        };
        let Some(ctrl)  = weak.upgrade()   else { return; };
        let Some(state) = ctrl.get_state() else { return; };
        let snap = settings_cb.read().unwrap().clone();
        let _ = renderer_cb.lock().unwrap().render(&state, &snap, max_scale);
        frames_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    });

    let ctrl = Arc::new(BreathingController::start(Arc::clone(&settings), request_draw));
    *ctrl_slot.write().unwrap() = Some(Arc::downgrade(&ctrl));

    thread::sleep(Duration::from_secs_f64(WARMUP_SECONDS));
    frames.store(0, std::sync::atomic::Ordering::Relaxed);
    let anim_start  = Instant::now();

    let mut animating = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        animating.push(sample_cpu(Duration::from_secs_f64(SAMPLE_SECONDS)));
    }

    let frame_end  = frames.load(std::sync::atomic::Ordering::Relaxed);
    let anim_elap  = anim_start.elapsed().as_secs_f64();
    let fps_effect = frame_end as f64 / anim_elap;

    // Drop the last strong Arc → controller's Drop runs stop() + joins thread.
    // Clear the slot so the Weak inside the callback is released as the thread
    // shuts down.
    *ctrl_slot.write().unwrap() = None;
    drop(ctrl);

    // ── Derive delta ─────────────────────────────────────────────────────────
    let deltas: Vec<f64> = animating.iter().map(|&x| (x - baseline_avg).max(0.0)).collect();
    let peak_delta = deltas.iter().cloned().fold(0.0_f64, f64::max);
    let avg_delta  = deltas.iter().sum::<f64>() / deltas.len() as f64;

    let base_str  = baseline.iter().map(|x| format!("{x:>4.1}%")).collect::<Vec<_>>().join(", ");
    let anim_str  = animating.iter().map(|x| format!("{x:>4.1}%")).collect::<Vec<_>>().join(", ");
    let delta_str = deltas.iter().map(|x| format!("{x:>4.1}%")).collect::<Vec<_>>().join(", ");

    println!("[{tag}] {label}", tag = v.tag, label = v.label);
    println!("  baseline : [{base_str}]  avg {baseline_avg:>4.1}%");
    println!("  animating: [{anim_str}]  ({fps_effect:>5.1} fps effective)");
    println!("  delta    : [{delta_str}]  peak {peak_delta:>4.1}%  avg {avg_delta:>4.1}%");
    println!();
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Optional filter: `-- only=circle_ripple`
    let args: Vec<String> = std::env::args().collect();
    let filter: Option<String> = args.iter()
        .find_map(|a| a.strip_prefix("only=").map(|s| s.to_string()));

    println!(
        "exhale-render headless CPU bench — {w}×{h}, {n}×{s:.1}s samples, warmup {wu:.1}s",
        w = WIDTH, h = HEIGHT, n = SAMPLE_COUNT, s = SAMPLE_SECONDS, wu = WARMUP_SECONDS,
    );
    println!("{}\n", "─".repeat(72));

    for v in variants() {
        if let Some(f) = &filter {
            if !v.tag.contains(f.as_str()) { continue; }
        }
        run_variant(&v);
    }

    println!(
        "Done.  Baseline measures idle process overhead (controller parked, no \
         render).  Delta approximates animation + render cost."
    );
}
