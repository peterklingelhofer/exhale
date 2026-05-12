use std::sync::Arc;

use anyhow::Result;
use exhale_core::{controller::BreathingState, settings::Settings};
use exhale_render::{GpuContext, OverlayRenderer};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::platform;

/// One transparent overlay covering a single monitor.
pub struct OverlayHandle {
    pub window:   Arc<Window>,
    pub renderer: OverlayRenderer,
}

impl OverlayHandle {
    /// Create one overlay per connected monitor, all sharing `gpu`.
    pub fn create_all(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
    ) -> Vec<Self> {
        let monitors: Vec<_> = event_loop.available_monitors().collect();
        let mut handles = Vec::with_capacity(monitors.len());

        for monitor in monitors {
            match Self::create_one(event_loop, Arc::clone(&gpu), Some(monitor)) {
                Ok(h)  => handles.push(h),
                Err(e) => log::error!("overlay window error: {e}"),
            }
        }

        // Fallback: at least one window on the primary monitor.
        if handles.is_empty() {
            match Self::create_one(event_loop, Arc::clone(&gpu), None) {
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

        let size   = window.inner_size();
        let surface = gpu.instance.create_surface(Arc::clone(&window))?;
        let renderer = OverlayRenderer::new(Arc::clone(&gpu), surface, size.width, size.height)?;

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
        if !renderer.alpha_capable() {
            log::warn!(
                "overlay swap chain only supports Opaque alpha; hiding \
                 overlay window to avoid blanket-black-screen.  This is \
                 typical under VMs running WARP — test on real GPU \
                 hardware to see the breath animation."
            );
            window.set_visible(false);
        }

        Ok(Self { window, renderer })
    }

    pub fn render(
        &mut self,
        state:            &BreathingState,
        settings:         &Settings,
        max_circle_scale: f32,
    ) -> Result<()> {
        self.renderer.render(state, settings, max_circle_scale)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.renderer.resize(size.width, size.height);
    }
}
