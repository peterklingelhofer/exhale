use std::sync::{Arc, Mutex, RwLock, mpsc::{self, Sender}};

use exhale_core::poison::{MutexPoisonExt, RwLockPoisonExt};
use std::thread::{self, JoinHandle};

use anyhow::Result;
use exhale_core::{
    controller::BreathingState,
    settings::Settings,
};
use exhale_render::{GpuContext, OverlayRenderer};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::ActiveEventLoop,
    monitor::MonitorHandle,
    window::Window,
};

/// Geometry-based identifier for a connected monitor.  Used to diff the
/// set of currently-connected monitors against the set we already have
/// overlays for, so hot-plug events (connect / disconnect) can be
/// detected by polling `available_monitors()` without relying on the
/// `MonitorHandle`'s own identity (which is not guaranteed stable
/// across calls on every platform).
pub type MonitorKey = (i32, i32, u32, u32);

/// Derive a stable key from a monitor's physical position + size.
/// Two monitors can't share the same rectangle, so the tuple is unique
/// per session and survives DPI changes (it's all physical pixels)
pub fn monitor_key(m: &MonitorHandle) -> MonitorKey {
    let p = m.position();
    let s = m.size();
    (p.x, p.y, s.width, s.height)
}

use crate::platform;

// ─── Render-thread message protocol ───────────────────────────────────────────

/// Messages the main thread (or controller thread) sends to a per-overlay
/// render thread.  The render thread owns the wgpu surface + renderer and
/// processes one message at a time, coalescing duplicate `Frame` requests
/// so it never falls behind under controller bursts.
enum RenderMsg {
    /// Wake up and render the latest controller state.  The controller
    /// writes its state slot BEFORE sending this, so a `Frame` always
    /// observes the most recent breathing snapshot via the Mutex barrier
    Frame,
    /// Window resized — re-configure the swap chain.  Coalesced: if
    /// multiple resizes are pending, only the latest dimensions take
    /// effect
    Resize(PhysicalSize<u32>),
    /// Tear down the render thread and drop the renderer.  Sent on
    /// app shutdown so the per-window wgpu device + queue + surface
    /// release cleanly
    Shutdown,
}

/// Wake handle for an overlay's render thread.  Clone-friendly so the
/// controller can hold one per overlay and fan out frame signals
/// without owning the [`OverlayHandle`].  Bypasses the main event
/// loop's message queue entirely
#[derive(Clone)]
pub struct FrameSender(Sender<RenderMsg>);

impl FrameSender {
    /// Wake the render thread.  Coalesced on the receiving side, so
    /// calling this faster than the renderer can keep up just folds
    /// into a single render pass against the latest controller state.
    pub fn send_frame(&self) {
        let _ = self.0.send(RenderMsg::Frame);
    }
}

// ─── Public handle (main-thread side) ─────────────────────────────────────────

/// One transparent overlay covering a single monitor.
///
/// The main thread keeps only the [`Arc<Window>`] and a channel to the
/// per-overlay render thread.  The render thread owns the
/// [`OverlayRenderer`] (and through it the per-window wgpu device, queue,
/// surface) and renders directly from the controller's shared state
/// snapshot.  This decouples overlay frame delivery from the Windows
/// main-thread message pump, WM_PAINT being the lowest-priority message
/// in the queue, so when WM_MOUSEMOVE floods the queue (hover storm over
/// the settings window), a main-thread render loop would have its
/// WM_PAINT slots starved and the breath animation would stutter.
/// With render threads, the controller signals the thread via an mpsc
/// channel that bypasses the Windows message queue entirely
pub struct OverlayHandle {
    pub window: Arc<Window>,
    /// Sender to the render thread.  Cloned for the controller's
    /// `request_draw` callback so frame signals go straight to the
    /// renderer without touching the main thread's message queue
    msg_tx:     Sender<RenderMsg>,
    /// Join handle for the render thread.  Taken on shutdown so we
    /// can wait for the thread to drop the renderer / wgpu resources
    /// before the app exits
    thread:     Option<JoinHandle<()>>,
    /// Monitor this overlay covers.  `None` for the fallback overlay
    /// created when `available_monitors()` returned empty at startup
    monitor_key: Option<MonitorKey>,
    /// `true` when the swap chain supports per-pixel alpha and the
    /// window is acting as a true click-through overlay.  `false`
    /// when the compositor / GPU only exposed Opaque alpha and we
    /// reconfigured the window as a regular windowed app — used by
    /// `create_all` to avoid spawning duplicate fallback windows on
    /// every monitor
    pub alpha_capable: bool,
}

impl OverlayHandle {
    /// Geometry-key of the monitor this overlay covers, or `None`
    /// for the no-monitor fallback overlay.  Used by the hot-plug
    /// rescan path to diff against the current monitor list
    pub fn monitor_key(&self) -> Option<MonitorKey> {
        self.monitor_key
    }

    /// Create one overlay per connected monitor, all sharing `gpu`.
    pub fn create_all(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
        settings:   Arc<RwLock<Settings>>,
        state:      Arc<Mutex<Option<BreathingState>>>,
        max_circle_scale: f32,
    ) -> Vec<Self> {
        let monitors: Vec<_> = event_loop.available_monitors().collect();
        let mut handles = Vec::with_capacity(monitors.len());

        for monitor in monitors {
            match Self::create_one(
                event_loop, Arc::clone(&gpu), Some(monitor),
                Arc::clone(&settings), Arc::clone(&state), max_circle_scale,
            ) {
                Ok(h)  => {
                    // If the system's swap chain doesn't support
                    // per-pixel alpha, the first overlay reconfigured
                    // itself as a regular windowed app (see
                    // `create_one`).  Spawning more "overlays" on
                    // additional monitors in this mode would just
                    // produce duplicate windowed copies of the
                    // animation, none of which act as overlays.
                    // Bail out after the first
                    let opaque_only = !h.alpha_capable;
                    handles.push(h);
                    if opaque_only { break; }
                },
                Err(e) => log::error!("overlay window error: {e}"),
            }
        }

        // Fallback: at least one window on the primary monitor.
        if handles.is_empty() {
            match Self::create_one(
                event_loop, Arc::clone(&gpu), None,
                Arc::clone(&settings), Arc::clone(&state), max_circle_scale,
            ) {
                Ok(h)  => handles.push(h),
                Err(e) => log::error!("fallback overlay error: {e}"),
            }
        }

        handles
    }

    pub(crate) fn create_one(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
        monitor:    Option<MonitorHandle>,
        settings:   Arc<RwLock<Settings>>,
        state:      Arc<Mutex<Option<BreathingState>>>,
        max_circle_scale: f32,
    ) -> Result<Self> {
        let monitor_key = monitor.as_ref().map(monitor_key);
        // Borderless window sized to the monitor — NOT a macOS-fullscreen
        // window.  `Fullscreen::Borderless` on macOS puts the window into
        // its own fullscreen Space, which triggers the swipe animation and
        // cancels the click-through / always-on-top overlay behavior.
        // The Swift reference app builds a plain NSWindow with styleMask
        // `[.borderless, .fullSizeContentView]` covering `screen.frame` at
        // window level `NSScreenSaverWindowLevel`; we mirror that here by
        // supplying explicit position + size and letting
        // `platform::setup_overlay_window` apply the level and collection
        // behavior on macOS (and equivalent flags on Windows / X11).
        // Cross-platform transparency: `with_transparent(true)` selects
        // an alpha-capable visual on macOS / Linux and routes through
        // `WS_EX_LAYERED` + `DwmEnableBlurBehindWindow` on Windows.
        // `WS_EX_NOREDIRECTIONBITMAP` is NOT a usable alternative on
        // Windows because that path requires a DirectComposition visual
        // tree (`CreateSwapChainForComposition` + bound DComp visual)
        // for the swap chain to appear, and wgpu's stock DX12 backend
        // uses `CreateSwapChainForHwnd` which doesn't wire that up:
        // NRB-without-DComp produces solid-black output.  Layered +
        // DwmEnableBlurBehindWindow is the supported wgpu-friendly
        // alpha pipeline on Windows.
        let mut attrs = Window::default_attributes()
            .with_title("exhale-overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_resizable(false)
            .with_window_icon(crate::app_icon::window_icon());

        // Wayland-specific demotion: place the overlay at
        // `AlwaysOnBottom` so other windows naturally cover it.  Wayland
        // (Mutter/GNOME on Ubuntu's default session) doesn't expose any
        // protocol winit can use for click-through — `wp_input_region`
        // isn't surfaced through the winit API — so a topmost overlay
        // would block every click that should go to the desktop or
        // another app.  Bottom-stacking lets the user Alt-Tab past the
        // overlay (other apps come in front of it), and Alt-Tab back to
        // see the breath animation when they want it.  On X11 sessions
        // we keep our existing `setup_overlay_window` behavior with
        // `_NET_WM_STATE_ABOVE` + XFixes click-through, where this isn't
        // a problem.
        // AlwaysOnBottom on Wayland is applied ONLY to the probe
        // window — it will become the live overlay window only if
        // the alpha probe below succeeds.  On the fallback path
        // (compositor exposes Opaque alpha only) we drop the probe
        // and create a fresh windowed-app window with no window-level
        // overrides, so the user gets a normal movable / resizable
        // app window that participates in z-order like anything else.
        #[cfg(all(unix, not(target_os = "macos")))]
        let is_wayland = std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);
        #[cfg(all(unix, not(target_os = "macos")))]
        if is_wayland {
            attrs = attrs.with_window_level(
                winit::window::WindowLevel::AlwaysOnBottom,
            );
        }

        if let Some(m) = monitor.as_ref() {
            let pos  = m.position();
            let size = m.size();
            // On Windows, shrink the overlay by 1 px on the bottom edge.
            // Windows treats a topmost window that exactly matches the
            // monitor's geometry as an "exclusive fullscreen
            // application" and suspends the auto-hide taskbar reveal
            // logic — moving the mouse to the bottom edge produces no
            // animation.  Coming up 1 pixel short prevents the
            // fullscreen-app classification, the taskbar trigger zone
            // reappears, and the visible breath animation loses only
            // the bottom physical pixel (imperceptible against the
            // soft animated gradient).  Other platforms keep the
            // exact monitor size — macOS doesn't have this issue;
            // X11 EWMH FULLSCREEN we apply elsewhere is the intended
            // signal there.
            let win_h = if cfg!(target_os = "windows") {
                size.height.saturating_sub(1).max(1)
            } else {
                size.height.max(1)
            };
            attrs = attrs
                .with_position(PhysicalPosition::new(pos.x, pos.y))
                .with_inner_size(PhysicalSize::new(size.width.max(1), win_h));
        }

        // Probe the swap chain alpha modes BEFORE deciding what kind
        // of window to expose to the user.  Some compositors (Wayland
        // on typical Linux hardware running Mutter / GNOME, WARP under
        // Windows VMs, remote-desktop sessions) only expose `Opaque`
        // alpha to wgpu, which means a click-through topmost overlay
        // would paint a blanket-black rectangle covering every pixel
        // of every monitor.  In that case we throw away the probe
        // window + surface entirely and recreate as a regular
        // movable / resizable / decorated app window — patching the
        // overlay window's attrs after the fact doesn't reliably
        // work on Wayland (decoration / position / level changes
        // post-mapping are no-ops on Mutter).
        //
        // The probe window is created `with_visible(false)` so the
        // user never sees the fullscreen-transparent probe surface
        // flash on screen before the decision is made
        attrs = attrs.with_visible(false);
        let probe_window = Arc::new(event_loop.create_window(attrs)?);
        let probe_size   = probe_window.inner_size();
        let probe_surface = gpu.instance.create_surface(Arc::clone(&probe_window))?;
        let probe_renderer = OverlayRenderer::new(
            Arc::clone(&gpu), probe_surface, probe_size.width, probe_size.height,
        )?;
        let alpha_capable = probe_renderer.alpha_capable();

        let (window, mut renderer) = if alpha_capable {
            // Keep the probe window — it IS the overlay window.
            // Apply platform overlay flags (click-through, topmost,
            // all-Spaces) and then make it visible.
            platform::setup_overlay_window(&probe_window);
            probe_window.set_visible(true);
            #[cfg(all(unix, not(target_os = "macos")))]
            if is_wayland {
                log::info!(
                    "Wayland session with alpha-capable swap chain: \
                     overlay running at AlwaysOnBottom (no portable \
                     click-through API on Wayland — for full topmost \
                     + click-through behaviour log out and pick an \
                     X11 session at the login screen)."
                );
            }
            (probe_window, probe_renderer)
        } else {
            log::warn!(
                "overlay swap chain only supports Opaque alpha; recreating \
                 as a regular windowed app.  Common on real Wayland hardware \
                 (compositor doesn't expose alpha-capable swap chains to \
                 wgpu) and under VMs running WARP.  The breath animation \
                 will render in a normal movable / resizable window — you \
                 can watch it directly, or send it behind your other apps \
                 in Rectangle mode and narrow them so the edges show \
                 through (Python bars-mode style)."
            );
            // Drop the probe window + renderer + surface.  Order
            // matters: renderer holds the wgpu surface which holds a
            // reference to the window — drop renderer first so the
            // surface releases before the window does
            drop(probe_renderer);
            drop(probe_window);

            // Build the windowed-app from scratch.  Fresh attrs (no
            // AlwaysOnBottom, no fullscreen sizing, no transparent
            // visual) — these all stick across Wayland's map cycle
            // because we set them at creation time
            let mut win_attrs = Window::default_attributes()
                .with_title("exhale")
                .with_inner_size(PhysicalSize::new(480u32, 360u32))
                .with_decorations(true)
                .with_resizable(true)
                .with_window_icon(crate::app_icon::window_icon());
            if let Some(m) = monitor.as_ref() {
                let mp = m.position();
                let ms = m.size();
                let x  = mp.x + (ms.width  as i32 - 480) / 2;
                let y  = mp.y + (ms.height as i32 - 360) / 2;
                win_attrs = win_attrs
                    .with_position(PhysicalPosition::new(x.max(mp.x), y.max(mp.y)));
            }
            let window = Arc::new(event_loop.create_window(win_attrs)?);
            let size = window.inner_size();
            let surface = gpu.instance.create_surface(Arc::clone(&window))?;
            let renderer = OverlayRenderer::new(
                Arc::clone(&gpu), surface, size.width, size.height,
            )?;
            (window, renderer)
        };

        // Spawn the per-overlay render thread.  It owns the renderer
        // for its entire lifetime; the main thread only sends Frame /
        // Resize / Shutdown messages over the channel.  Frame messages
        // bypass the Windows main-thread message pump entirely, so a
        // WM_MOUSEMOVE storm over the settings window can't starve the
        // overlay's render slots.
        let (msg_tx, msg_rx) = mpsc::channel::<RenderMsg>();
        let thread = thread::Builder::new()
            .name(format!("exhale-overlay-{:?}", window.id()))
            .spawn(move || {
                render_thread_loop(
                    msg_rx, &mut renderer, state, settings, max_circle_scale,
                    alpha_capable,
                );
            })
            .map_err(|e| anyhow::anyhow!(
                "spawn overlay render thread for {:?}: {} \
                 (system thread limit hit — skipping this monitor's overlay)",
                window.id(), e,
            ))?;

        Ok(Self { window, msg_tx, thread: Some(thread), monitor_key, alpha_capable })
    }

    /// Clone the channel sender so the controller's `request_draw`
    /// closure can deliver Frame signals directly to this render
    /// thread (bypassing the main event loop).
    pub fn frame_sender(&self) -> FrameSender {
        FrameSender(self.msg_tx.clone())
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        let _ = self.msg_tx.send(RenderMsg::Resize(size));
    }

    /// Wake the render thread to draw one frame against the latest
    /// shared controller state.  Used by main-thread code paths that
    /// mutate settings outside of a controller tick (Start/Stop, theme
    /// change, etc.) and want the overlay to reflect the change
    /// immediately.
    pub fn wake_render(&self) {
        let _ = self.msg_tx.send(RenderMsg::Frame);
    }
}

impl Drop for OverlayHandle {
    fn drop(&mut self) {
        // Best-effort: tell the render thread to exit, then wait for
        // it.  If the channel is already closed (thread panicked) the
        // send fails silently and the join still works.  Joining
        // before drop completes prevents the wgpu device from being
        // released while the thread is mid-frame.
        let _ = self.msg_tx.send(RenderMsg::Shutdown);
        if let Some(h) = self.thread.take() {
            let thread_name = h.thread().name().unwrap_or("exhale-overlay-?").to_string();
            // Surface a panic in the render thread rather than silently
            // swallowing it.  Previous code did `let _ = h.join();`
            // which meant a wgpu device crash mid-frame produced only a
            // missing log line; now we log loudly with the captured
            // panic payload so post-mortem debugging has something to
            // grep for.
            match h.join() {
                Ok(()) => {}
                Err(payload) => {
                    let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
                        (*s).to_string()
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "<non-string panic payload>".to_string()
                    };
                    log::error!("render thread `{thread_name}` panicked: {msg}");
                }
            }
        }
    }
}

// ─── Render-thread body ───────────────────────────────────────────────────────

/// Per-overlay render thread.  Owns the [`OverlayRenderer`] and renders
/// frames in response to channel messages.  Coalesces multiple pending
/// `Frame` messages into a single render so the thread can't fall
/// behind under controller bursts
fn render_thread_loop(
    msg_rx:           mpsc::Receiver<RenderMsg>,
    renderer:         &mut OverlayRenderer,
    state:            Arc<Mutex<Option<BreathingState>>>,
    settings:         Arc<RwLock<Settings>>,
    max_circle_scale: f32,
    alpha_capable:    bool,
) {
    #[allow(clippy::while_let_loop)]
    loop {
        // Block until at least one message arrives.  `recv` returns
        // `Err` only when every sender has been dropped (overlay handle
        // gone), which means the app is shutting down regardless.
        let first = match msg_rx.recv() {
            Ok(m)  => m,
            Err(_) => break,
        };

        let CoalescedBatch { should_render, latest_resize, should_quit } =
            coalesce_messages(first, &msg_rx);

        if should_quit {
            break;
        }

        if let Some(s) = latest_resize {
            renderer.resize(s.width, s.height);
        }

        // Skip the render entirely on a swap chain that can only do
        // Opaque alpha (VM/WARP) — the window is hidden so painting
        // wastes GPU work that nobody will see.  Still drain the
        // channel above so resize / shutdown messages flow.
        if should_render && alpha_capable {
            // Read the latest snapshot and settings.  The controller
            // writes its state BEFORE sending us the Frame, and the
            // Mutex acquire here provides the matching release/acquire
            // barrier so we observe the most recent values.
            let state_snap = state.lock_or_recover().unwrap_or_else(|| {
                BreathingState {
                    phase:     exhale_core::types::BreathingPhase::Inhale,
                    progress:  0.0,
                    hold_time: 0.0,
                }
            });
            let settings_snap = settings.read_or_recover().clone();
            if let Err(e) = renderer.render(&state_snap, &settings_snap, max_circle_scale) {
                log::error!("overlay render: {e}");
            }
        }
    }
}

/// Output of `coalesce_messages`: a flattened view of what the render
/// thread should do for this iteration of the loop.  Extracted so the
/// coalescing logic can be unit-tested without a real `OverlayRenderer`
/// (which needs a GPU surface).
#[derive(Debug, Default, PartialEq, Eq)]
struct CoalescedBatch {
    should_render: bool,
    latest_resize: Option<PhysicalSize<u32>>,
    should_quit:   bool,
}

/// Fold a first message plus any further messages already queued in
/// the receiver into a single decision tuple.  Resizes coalesce to
/// the last one seen; multiple Frames become a single render; a
/// Shutdown in the batch wins
fn coalesce_messages(
    first:  RenderMsg,
    msg_rx: &mpsc::Receiver<RenderMsg>,
) -> CoalescedBatch {
    let mut batch = CoalescedBatch::default();
    apply_msg(first, &mut batch);
    while let Ok(m) = msg_rx.try_recv() {
        apply_msg(m, &mut batch);
    }
    batch
}

fn apply_msg(m: RenderMsg, batch: &mut CoalescedBatch) {
    match m {
        RenderMsg::Frame      => batch.should_render = true,
        RenderMsg::Resize(s)  => batch.latest_resize = Some(s),
        RenderMsg::Shutdown   => batch.should_quit   = true,
    }
}

#[cfg(test)]
mod coalesce_tests {
    use super::*;
    use winit::dpi::PhysicalSize;

    #[test]
    fn many_frames_become_one_render() {
        let (tx, rx) = mpsc::channel();
        for _ in 0..50 {
            tx.send(RenderMsg::Frame).unwrap();
        }
        let first = rx.recv().unwrap();
        let batch = coalesce_messages(first, &rx);
        assert!(batch.should_render, "should still want to render");
        assert!(batch.latest_resize.is_none());
        assert!(!batch.should_quit);
    }

    #[test]
    fn resize_keeps_latest_only() {
        let (tx, rx) = mpsc::channel();
        tx.send(RenderMsg::Resize(PhysicalSize::new(100, 100))).unwrap();
        tx.send(RenderMsg::Resize(PhysicalSize::new(200, 200))).unwrap();
        tx.send(RenderMsg::Resize(PhysicalSize::new(300, 400))).unwrap();
        let first = rx.recv().unwrap();
        let batch = coalesce_messages(first, &rx);
        assert_eq!(batch.latest_resize, Some(PhysicalSize::new(300, 400)));
    }

    #[test]
    fn shutdown_wins_over_pending_frames() {
        let (tx, rx) = mpsc::channel();
        tx.send(RenderMsg::Frame).unwrap();
        tx.send(RenderMsg::Frame).unwrap();
        tx.send(RenderMsg::Shutdown).unwrap();
        tx.send(RenderMsg::Frame).unwrap(); // ignored, never read separately
        let first = rx.recv().unwrap();
        let batch = coalesce_messages(first, &rx);
        assert!(batch.should_quit, "shutdown must surface");
    }

    #[test]
    fn frame_then_resize_does_both() {
        let (tx, rx) = mpsc::channel();
        tx.send(RenderMsg::Frame).unwrap();
        tx.send(RenderMsg::Resize(PhysicalSize::new(50, 50))).unwrap();
        let first = rx.recv().unwrap();
        let batch = coalesce_messages(first, &rx);
        assert!(batch.should_render);
        assert_eq!(batch.latest_resize, Some(PhysicalSize::new(50, 50)));
    }

    #[test]
    fn lone_frame_with_no_extras() {
        let (tx, rx) = mpsc::channel();
        tx.send(RenderMsg::Frame).unwrap();
        let first = rx.recv().unwrap();
        let batch = coalesce_messages(first, &rx);
        assert_eq!(batch, CoalescedBatch {
            should_render: true,
            latest_resize: None,
            should_quit:   false,
        });
    }
}
