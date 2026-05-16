use std::sync::{Arc, Mutex, RwLock, mpsc::{self, Sender}};
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
    window::Window,
};

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
/// [`OverlayRenderer`] (and through it the per-window wgpu device + queue
/// + surface) and renders directly from the controller's shared state
/// snapshot.  This decouples overlay frame delivery from the Windows
/// main-thread message pump — WM_PAINT is the lowest-priority message in
/// the queue, so when WM_MOUSEMOVE floods the queue (hover storm over
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
}

impl OverlayHandle {
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
                Ok(h)  => handles.push(h),
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

    fn create_one(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
        monitor:    Option<winit::monitor::MonitorHandle>,
        settings:   Arc<RwLock<Settings>>,
        state:      Arc<Mutex<Option<BreathingState>>>,
        max_circle_scale: f32,
    ) -> Result<Self> {
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
        // `WS_EX_LAYERED` + `DwmEnableBlurBehindWindow` on Windows.  We
        // tried `WS_EX_NOREDIRECTIONBITMAP` instead on Windows but that
        // path requires manually creating a DirectComposition visual
        // tree (`CreateSwapChainForComposition` + bound DComp visual)
        // for the swap chain to actually appear — wgpu's stock DX12
        // backend uses `CreateSwapChainForHwnd` which doesn't wire that
        // up, so NRB-without-DComp produced solid-black output.  The
        // legacy `WS_EX_LAYERED` + `DwmEnableBlurBehindWindow` route
        // is the supported wgpu-friendly alpha pipeline on Windows.
        let mut attrs = Window::default_attributes()
            .with_title("exhale-overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_resizable(false);

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
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let is_wayland = std::env::var("XDG_SESSION_TYPE")
                .map(|s| s.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false);
            if is_wayland {
                attrs = attrs.with_window_level(
                    winit::window::WindowLevel::AlwaysOnBottom,
                );
                log::info!(
                    "Wayland session detected: placing overlay at \
                     AlwaysOnBottom so other windows can cover it.  \
                     Wayland has no portable click-through API — for \
                     full overlay behavior log out and pick \
                     'Ubuntu on Xorg' (or any X11 session) at the \
                     login screen."
                );
            }
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

        let window = Arc::new(event_loop.create_window(attrs)?);

        // Platform-specific: click-through, always-on-top, all-spaces.
        platform::setup_overlay_window(&window);

        let size    = window.inner_size();
        let surface = gpu.instance.create_surface(Arc::clone(&window))?;
        let mut renderer =
            OverlayRenderer::new(Arc::clone(&gpu), surface, size.width, size.height)?;

        // If the swap chain doesn't advertise any per-pixel alpha mode
        // (typical for VM environments running WARP / Microsoft Basic
        // Render Driver), hide the overlay window — otherwise it
        // renders solid black across the entire screen with no way to
        // see anything else, which makes the VM unusable.  The
        // breathing animation will be invisible on this machine, but
        // the rest of the app (settings window, tray, hotkeys) stays
        // testable.  Real Windows hardware with an actual GPU exposes
        // alpha-capable DXGI surfaces and renders correctly without
        // hitting this branch.
        let alpha_capable = renderer.alpha_capable();
        if !alpha_capable {
            log::warn!(
                "overlay swap chain only supports Opaque alpha; hiding \
                 overlay window to avoid blanket-black-screen.  This is \
                 typical under VMs running WARP — test on real GPU \
                 hardware to see the breath animation."
            );
            window.set_visible(false);
        }

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
            .expect("spawn overlay render thread");

        Ok(Self { window, msg_tx, thread: Some(thread) })
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
            let _ = h.join();
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
    loop {
        // Block until at least one message arrives.  `recv` returns
        // `Err` only when every sender has been dropped (overlay handle
        // gone), which means the app is shutting down regardless.
        let first = match msg_rx.recv() {
            Ok(m)  => m,
            Err(_) => break,
        };

        let mut should_render          = false;
        let mut latest_resize          = None::<PhysicalSize<u32>>;
        let mut should_quit            = false;

        let handle = |m: RenderMsg, should_render: &mut bool,
                      latest_resize: &mut Option<PhysicalSize<u32>>,
                      should_quit: &mut bool| {
            match m {
                RenderMsg::Frame        => *should_render  = true,
                RenderMsg::Resize(s)    => *latest_resize  = Some(s),
                RenderMsg::Shutdown     => *should_quit    = true,
            }
        };
        handle(first, &mut should_render, &mut latest_resize, &mut should_quit);

        // Drain any other queued messages so a burst of Frame requests
        // (e.g. controller waking us multiple times before we got the
        // CPU) becomes a single render pass against the latest state.
        // This is the core mechanism that prevents backlog buildup
        // when present blocks longer than the controller's tick.
        while let Ok(m) = msg_rx.try_recv() {
            handle(m, &mut should_render, &mut latest_resize, &mut should_quit);
        }

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
            let state_snap = state.lock().unwrap().unwrap_or_else(|| {
                BreathingState {
                    phase:     exhale_core::types::BreathingPhase::Inhale,
                    progress:  0.0,
                    hold_time: 0.0,
                }
            });
            let settings_snap = settings.read().unwrap().clone();
            if let Err(e) = renderer.render(&state_snap, &settings_snap, max_circle_scale) {
                log::error!("overlay render: {e}");
            }
        }
    }
}
