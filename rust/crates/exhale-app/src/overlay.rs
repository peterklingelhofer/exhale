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
        // Window-creation attributes differ on Windows because the
        // alpha-compositing path is different from macOS / Linux:
        //
        //   - macOS / Linux:   `with_transparent(true)` selects an
        //     alpha-capable visual / clearColor backing and the OS
        //     compositor blends our wgpu output naturally.
        //
        //   - Windows:         `with_transparent(true)` adds
        //     `WS_EX_LAYERED`, which sounds right but actually breaks
        //     us — DXGI flip-model swap chains (what wgpu uses) do
        //     not composite alpha through layered-window bitmaps on
        //     Win11, and the overlay renders solid black instead of
        //     transparent.  Instead we keep the window NON-layered and
        //     request `WS_EX_NOREDIRECTIONBITMAP` so our wgpu output is
        //     delivered straight to DWM via DirectComposition, which
        //     IS alpha-aware and pairs cleanly with `PreMultiplied`
        //     surface alpha + the shader's premultiplied output.
        //     Critically, `with_transparent(true)` and
        //     `with_no_redirection_bitmap(true)` together are silently
        //     contradictory (LAYERED vs NRB), so we mutually-exclude
        //     them per platform.
        let want_transparent = !cfg!(target_os = "windows");
        let mut attrs = Window::default_attributes()
            .with_title("exhale-overlay")
            .with_transparent(want_transparent)
            .with_decorations(false)
            .with_resizable(false);

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            attrs = attrs.with_no_redirection_bitmap(true);
        }

        if let Some(m) = monitor.as_ref() {
            let pos  = m.position();
            let size = m.size();
            attrs = attrs
                .with_position(PhysicalPosition::new(pos.x, pos.y))
                .with_inner_size(PhysicalSize::new(size.width.max(1), size.height.max(1)));
        }

        let window = Arc::new(event_loop.create_window(attrs)?);

        // Platform-specific: click-through, always-on-top, all-spaces.
        platform::setup_overlay_window(&window);

        let size   = window.inner_size();
        let surface = gpu.instance.create_surface(Arc::clone(&window))?;
        let renderer = OverlayRenderer::new(Arc::clone(&gpu), surface, size.width, size.height)?;

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
