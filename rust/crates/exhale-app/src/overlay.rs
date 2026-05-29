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
pub enum RenderMsg {
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

/// Wake handle for an overlay.  Clone-friendly so the controller can
/// hold one per overlay and fan out frame signals without owning the
/// [`OverlayHandle`].
///
/// Has two variants because overlays use two different rendering
/// architectures depending on the platform:
///
///   - [`FrameSender::Channel`] — fires a `Frame` message into the
///     per-window render thread's mpsc channel, bypassing the main
///     event loop's message queue entirely.  Used on macOS / Windows
///     / Linux X11 where wgpu can acquire a swap-chain texture from
///     any thread.
///   - [`FrameSender::Redraw`] — calls `window.request_redraw()`
///     which causes winit to emit a `RedrawRequested` event on the
///     main thread.  Used on Wayland where the compositor's
///     frame-callback protocol requires `surface.get_current_texture()`
///     to run on the main thread; a background-thread render leaves
///     the xdg_toplevel in a "configured but unmapped" state
#[derive(Clone)]
pub enum FrameSender {
    Channel(Sender<RenderMsg>),
    Redraw(Arc<Window>),
}

impl FrameSender {
    /// Signal that the overlay should render the latest controller
    /// state.  Coalesced on the receiving side (channel: thread loop;
    /// redraw: winit's per-frame `request_redraw` dedup), so calling
    /// this faster than the renderer can keep up just folds into a
    /// single render pass against the latest controller state.
    pub fn send_frame(&self) {
        match self {
            Self::Channel(tx) => { let _ = tx.send(RenderMsg::Frame); }
            Self::Redraw(window) => window.request_redraw(),
        }
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
    /// Render dispatch: per-window thread (macOS / Windows / Linux
    /// X11) or main-thread synchronous rendering (Wayland)
    mode:        HandleMode,
}

/// How this overlay's renderer is driven.
///
/// `Threaded` keeps a per-window background thread that owns the
/// renderer outright and renders in response to `Frame` channel
/// messages — the original architecture, used everywhere wgpu can
/// safely acquire swap-chain textures from a non-main thread.
///
/// `MainThread` keeps the renderer in a [`Mutex`] on the handle so
/// it can be locked and called from `main.rs`'s `RedrawRequested`
/// event handler.  Required on Wayland: `surface.get_current_texture()`
/// must run synchronized with the compositor's `frame_callback`
/// protocol, which winit only delivers via `RedrawRequested` on the
/// main thread.  A background-thread render on Wayland returns
/// `Outdated` indefinitely and never commits a buffer, leaving the
/// xdg_toplevel in a "configured but unmapped" state — visible in
/// the dock but with nothing to display
enum HandleMode {
    Threaded {
        /// Sender to the render thread.  Cloned for the controller's
        /// `request_draw` callback so frame signals go straight to
        /// the renderer without touching the main thread's message
        /// queue.
        msg_tx: Sender<RenderMsg>,
        /// Join handle for the render thread.  Taken on shutdown so
        /// we can wait for the thread to drop the renderer / wgpu
        /// resources before the app exits.
        thread: Option<JoinHandle<()>>,
    },
    /// Boxed because the `Mutex<OverlayRenderer>` payload is much
    /// larger than the `Threaded` variant's two pointer-sized fields;
    /// boxing keeps `HandleMode` itself compact
    MainThread(Box<MainThreadState>),
}

struct MainThreadState {
    renderer:         Mutex<OverlayRenderer>,
    state:            Arc<Mutex<Option<BreathingState>>>,
    settings:         Arc<RwLock<Settings>>,
    max_circle_scale: f32,
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

        // ── Wayland: always take the windowed-app + main-thread path ──
        //
        // Wayland's security model doesn't expose a portable
        // always-on-top or click-through protocol to winit, so the
        // fullscreen-overlay model fundamentally doesn't work the
        // way it does on macOS / Windows / X11.  Additionally,
        // Mutter / GNOME (Ubuntu's default compositor) only exposes
        // Opaque alpha to wgpu, so even an AlwaysOnBottom overlay
        // would render as a solid-black rectangle covering the
        // whole screen.  The right answer here is a regular
        // windowed app the user can move / resize like anything
        // else, rendered on the main thread so wgpu's surface
        // acquisition stays synchronized with the compositor's
        // frame-callback protocol (see `HandleMode` doc).
        #[cfg(all(unix, not(target_os = "macos")))]
        let is_wayland = std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);
        #[cfg(not(all(unix, not(target_os = "macos"))))]
        let is_wayland = false;

        if is_wayland {
            log::info!(
                "Wayland session: running exhale as a regular windowed \
                 app (no fullscreen overlay).  Rendering on the main \
                 thread via RedrawRequested so wgpu surface acquisition \
                 stays in sync with the compositor's frame_callback \
                 protocol — move / resize / Alt-Tab the window like \
                 any other app; the breath animation renders as its \
                 content."
            );
            let (window, renderer, alpha_capable) =
                create_windowed_app(event_loop, &gpu, monitor.as_ref(), &settings)?;
            // Kick off the first frame.  Wayland needs an initial
            // RedrawRequested → render → buffer commit cycle for the
            // xdg_toplevel to actually map; without this the window
            // sits in the dock but never appears on screen.  This
            // request lands on the main thread's event queue and
            // fires as soon as resumed() returns
            window.request_redraw();
            return Ok(Self {
                window,
                monitor_key,
                alpha_capable,
                mode: HandleMode::MainThread(Box::new(MainThreadState {
                    renderer: Mutex::new(renderer),
                    state,
                    settings,
                    max_circle_scale,
                })),
            });
        }

        // ── Non-Wayland: per-window render thread ──────────────────
        let (window, mut renderer, alpha_capable) =
            create_overlay_or_fallback(event_loop, &gpu, monitor.as_ref(), &settings)?;

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

        Ok(Self {
            window,
            monitor_key,
            alpha_capable,
            mode: HandleMode::Threaded { msg_tx, thread: Some(thread) },
        })
    }

    /// Build a [`FrameSender`] tuned to this overlay's render mode —
    /// channel send for threaded overlays, `request_redraw` for
    /// main-thread (Wayland) overlays.  See [`FrameSender`] doc.
    pub fn frame_sender(&self) -> FrameSender {
        match &self.mode {
            HandleMode::Threaded { msg_tx, .. } =>
                FrameSender::Channel(msg_tx.clone()),
            HandleMode::MainThread(_) =>
                FrameSender::Redraw(Arc::clone(&self.window)),
        }
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        match &self.mode {
            HandleMode::Threaded { msg_tx, .. } => {
                let _ = msg_tx.send(RenderMsg::Resize(size));
            }
            HandleMode::MainThread(state) => {
                state.renderer.lock_or_recover().resize(size.width, size.height);
            }
        }
    }

    /// Wake the renderer to draw one frame against the latest shared
    /// controller state.  Used by main-thread code paths that mutate
    /// settings outside of a controller tick (Start/Stop, theme
    /// change, etc.) and want the overlay to reflect the change
    /// immediately.
    pub fn wake_render(&self) {
        self.frame_sender().send_frame();
    }

    /// Show or hide the underlying window.  Only meaningful for
    /// windowed-mode (MainThread) overlays: when the user hits Stop
    /// the window should close, and Start should bring it back.
    /// On threaded fullscreen-overlay windows (macOS / Windows /
    /// Linux X11) hiding would just remove the always-on-top
    /// breath animation while the app keeps running — not what
    /// Stop means on those platforms, where the render thread
    /// instead paints a "stopped" clear frame.  So we gate this on
    /// `alpha_capable`: threaded overlays ignore the toggle, the
    /// windowed-app shows / hides
    pub fn set_animation_visible(&self, visible: bool) {
        if self.alpha_capable {
            return;
        }
        self.window.set_visible(visible);
        if visible {
            // Wayland needs an explicit redraw request after a
            // hide → show cycle to re-commit a buffer; without it
            // the xdg_toplevel comes back to the dock but the
            // surface stays unattached.
            self.window.request_redraw();
        }
    }

    /// Render synchronously on the calling thread.  No-op for
    /// threaded overlays (their render thread owns the renderer).
    /// Called from `main.rs`'s `RedrawRequested` handler for
    /// `MainThread` overlays — the only path that drives Wayland
    /// rendering, since the compositor's frame_callback arrives
    /// through the event loop and not via the controller's
    /// background-thread `Frame` channel.
    pub fn render_on_main(&self) {
        if let HandleMode::MainThread(mt) = &self.mode {
            let mut r = mt.renderer.lock_or_recover();
            let snap = *mt.state.lock_or_recover();
            if let Some(snap) = snap {
                let s = mt.settings.read_or_recover();
                if let Err(e) = r.render(&snap, &s, mt.max_circle_scale) {
                    log::warn!("overlay main-thread render: {e}");
                }
            }
        }
    }
}

/// Build a plain decorated / resizable / movable app window that
/// renders the breath animation as its content.  Used unconditionally
/// on Wayland (no portable always-on-top there) and as the fallback
/// when the overlay probe finds the swap chain is Opaque-only
/// (WARP / remote-desktop / certain VM GPU paths).  Always returns
/// `alpha_capable = false` so the renderer paints onto the opaque
/// surface without compositing artefacts
fn create_windowed_app(
    event_loop: &ActiveEventLoop,
    gpu:        &Arc<GpuContext>,
    monitor:    Option<&MonitorHandle>,
    settings:   &Arc<RwLock<Settings>>,
) -> Result<(Arc<Window>, OverlayRenderer, bool)> {
    const DEFAULT_W: u32 = 480;
    const DEFAULT_H: u32 = 360;

    let placement = settings.read_or_recover().animation_window_placement();
    let win_attrs = Window::default_attributes()
        .with_title("exhale")
        .with_inner_size(PhysicalSize::new(
            placement.width.unwrap_or(DEFAULT_W),
            placement.height.unwrap_or(DEFAULT_H),
        ))
        .with_decorations(true)
        .with_resizable(true)
        .with_window_icon(crate::app_icon::window_icon());

    let window  = Arc::new(event_loop.create_window(win_attrs)?);

    // Apply the saved POSITION BEFORE creating the wgpu surface so
    // any compositor that re-runs configure on outer-position
    // changes settles into the final geometry before we configure
    // the surface.  Size was already set via `with_inner_size` in
    // the attrs above using the persisted (or default) physical
    // pixel dimensions
    crate::placement::apply_placement(event_loop, &window, &placement);
    if placement.x.is_none() {
        // First launch / no saved position: centre on the assigned
        // monitor explicitly so X11 / Windows / macOS don't fall back
        // to (0, 0).  Wayland still ignores this.
        if let Some(m) = monitor {
            let mp = m.position();
            let ms = m.size();
            let x  = mp.x + (ms.width  as i32 - DEFAULT_W as i32) / 2;
            let y  = mp.y + (ms.height as i32 - DEFAULT_H as i32) / 2;
            window.set_outer_position(PhysicalPosition::new(
                x.max(mp.x), y.max(mp.y),
            ));
        }
    }

    let size     = window.inner_size();
    let surface  = gpu.instance.create_surface(Arc::clone(&window))?;
    let renderer = OverlayRenderer::new(
        Arc::clone(gpu), surface, size.width, size.height,
    )?;
    Ok((window, renderer, false))
}

/// Try the platform-specific fullscreen overlay path (click-through,
/// always-on-top, all-Spaces).  Builds the borderless transparent
/// overlay window, probes the swap chain alpha modes, and either
/// keeps the window (alpha capable) or drops it and falls back to a
/// plain windowed app via [`create_windowed_app`].  Used on every
/// platform EXCEPT Wayland, which always goes through the windowed
/// path directly because none of these overlay flags are honoured by
/// the Wayland security model anyway
fn create_overlay_or_fallback(
    event_loop: &ActiveEventLoop,
    gpu:        &Arc<GpuContext>,
    monitor:    Option<&MonitorHandle>,
    settings:   &Arc<RwLock<Settings>>,
) -> Result<(Arc<Window>, OverlayRenderer, bool)> {
    // Borderless, transparent, fullscreen on the target monitor.
    // The `with_visible(false)` keeps the probe window off-screen
    // until we've decided which path to take, so the user never sees
    // a brief flash of a fullscreen transparent shell.
    let mut attrs = Window::default_attributes()
        .with_title("exhale-overlay")
        .with_transparent(true)
        .with_decorations(false)
        .with_resizable(false)
        .with_visible(false)
        .with_window_icon(crate::app_icon::window_icon());
    if let Some(m) = monitor {
        let pos  = m.position();
        let size = m.size();
        // On Windows, shrink the overlay by 1 px on the bottom edge
        // so Windows doesn't classify the topmost window as an
        // "exclusive fullscreen application" and suspend the
        // auto-hide taskbar reveal logic.
        let win_h = if cfg!(target_os = "windows") {
            size.height.saturating_sub(1).max(1)
        } else {
            size.height.max(1)
        };
        attrs = attrs
            .with_position(PhysicalPosition::new(pos.x, pos.y))
            .with_inner_size(PhysicalSize::new(size.width.max(1), win_h));
    }

    let probe_window   = Arc::new(event_loop.create_window(attrs)?);
    let probe_size     = probe_window.inner_size();
    let probe_surface  = gpu.instance.create_surface(Arc::clone(&probe_window))?;
    let probe_renderer = OverlayRenderer::new(
        Arc::clone(gpu), probe_surface, probe_size.width, probe_size.height,
    )?;
    let alpha_capable = probe_renderer.alpha_capable();

    if alpha_capable {
        // Order matters on Windows: `set_visible(true)` triggers
        // winit's internal `apply_diff` which OVERWRITES the entire
        // `GWL_EXSTYLE` word from winit's tracked `WindowFlags`
        // bitset.  That bitset does NOT include `WS_EX_TRANSPARENT`
        // (it only sets it when `IGNORE_CURSOR_EVENT` is on, which
        // we don't toggle via winit's API), so any
        // `WS_EX_TRANSPARENT` we OR-in BEFORE the visibility toggle
        // gets silently stripped on the way to the screen — visible
        // overlay, no click-through (the originally-reported
        // regression).  Calling `setup_overlay_window` AFTER
        // `set_visible` makes our raw `SetWindowLongPtrW` the LAST
        // write to the EX-style word, so the flag survives.  No-op
        // on non-Windows platforms — both calls are pure on macOS
        // and Linux
        probe_window.set_visible(true);
        platform::setup_overlay_window(&probe_window);
        Ok((probe_window, probe_renderer, true))
    } else {
        log::warn!(
            "overlay swap chain only supports Opaque alpha; falling back \
             to a regular windowed app.  Typical under VMs running WARP / \
             Microsoft Basic Render Driver or remote-desktop sessions, \
             AND on bare-metal Windows 10 installs where Vulkan reports \
             only Opaque alpha modes.  The breath animation will render \
             in a normal movable / resizable window instead of as a \
             click-through overlay."
        );
        // Belt-and-suspenders teardown of the fullscreen probe before
        // dropping: on a fresh Windows 10 install with NVIDIA Vulkan
        // (RTX-class, driver 5xx), a probe window built with
        // `.with_visible(false)` was reportedly leaving a
        // monitor-sized opaque black compositor surface on screen
        // even though the HWND should have been hidden.  Forcing
        // `set_visible(false)` and moving it far off-screen before
        // dropping the Arc<Window> ensures both WM_HIDE and a
        // WindowPosChanged out of any visible region flush through
        // the OS compositor before DestroyWindow runs.  No-op on
        // Windows 11 / macOS / Linux where the bug doesn't reproduce
        probe_window.set_visible(false);
        let _ = probe_window.request_inner_size(PhysicalSize::new(1, 1));
        probe_window.set_outer_position(PhysicalPosition::new(-32000, -32000));
        // Order matters: drop renderer (releases the surface) before
        // the window so the surface releases its reference cleanly.
        drop(probe_renderer);
        drop(probe_window);
        create_windowed_app(event_loop, gpu, monitor, settings)
    }
}

impl Drop for OverlayHandle {
    fn drop(&mut self) {
        // Threaded mode: tell the render thread to exit and wait for
        // it so the wgpu device + queue + surface release cleanly
        // before the app exits.  MainThread mode has no thread —
        // the renderer drops with the handle automatically.
        if let HandleMode::Threaded { msg_tx, thread } = &mut self.mode {
            let _ = msg_tx.send(RenderMsg::Shutdown);
            if let Some(h) = thread.take() {
                let thread_name = h.thread().name().unwrap_or("exhale-overlay-?").to_string();
                // Surface a panic in the render thread rather than
                // silently swallowing it
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

        // Render on every Frame message regardless of `alpha_capable`.
        //
        // Earlier this branch skipped rendering when
        // `alpha_capable == false` under the (now-stale) assumption
        // that an Opaque-only swap chain meant "the overlay window is
        // hidden, painting wastes GPU work".  Since the
        // windowed-fallback path was added, `alpha_capable == false`
        // ALSO means "we're running as a visible 480×360 windowed
        // app".  Skipping the render in that case left the fallback
        // window painted with whatever the GPU cleared at startup
        // (typically white) and pressing Start did nothing user-
        // visible — exactly the symptom reported on Windows 10
        // systems where Vulkan reports only the `Opaque` alpha mode.
        // Rendering is cheap (we already coalesce Frame bursts in
        // `coalesce_messages`) and the Opaque-only swap chain accepts
        // whatever colour we present, so just paint unconditionally
        if should_render {
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
        // `alpha_capable` is still passed in because callers further
        // down may grow uses for it; mark it `unused` to keep the
        // compiler quiet without breaking the public signature
        let _ = alpha_capable;
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
