use std::sync::Arc;

use anyhow::{Context, Result};
use egui::ViewportId;
use egui_wgpu::ScreenDescriptor;
use exhale_core::{
    settings::Settings,
    settings_manager::SettingsManager,
    types::{AnimationMode, AnimationShape, AppVisibility, ColorFillGradient, HoldRippleMode},
};
use exhale_render::GpuContext;
use egui::ThemePreference;
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Theme, Window},
};

use crate::platform;

// ─── SettingsWindow ───────────────────────────────────────────────────────────

pub struct SettingsWindow {
    pub window:            Arc<Window>,
    surface:               wgpu::Surface<'static>,
    config:                wgpu::SurfaceConfiguration,
    egui_ctx:              egui::Context,
    egui_state:            egui_winit::State,
    egui_renderer:         egui_wgpu::Renderer,
    gpu:                   Arc<GpuContext>,
    pending_reset:         bool,
    /// Tracks the OS appearance so egui visuals + the wgpu clear color stay
    /// in sync with Light/Dark mode.  `None` means the platform doesn't
    /// report a theme (some Linux desktops); we default to Dark there.
    theme: Theme,
    /// Raw pointer to the NSVisualEffectView installed on macOS.  0 when
    /// not applicable (non-macOS, or EXHALE_DISABLE_VIBRANCY set).  When
    /// the theme changes we call `platform::update_settings_vibrancy`
    /// with this pointer so the blur material/appearance follows the
    /// system's Light/Dark toggle.
    vev_ptr: usize,
    /// Lazy-loaded SF Symbol textures for the Start / Stop / Reset
    /// buttons in dark and light themes.  None on non-macOS or when the
    /// rasteriser fails — callers fall back to Unicode glyphs.
    icon_cache: IconCache,
}

/// Holds the texture handles for each control-button icon × theme.  We
/// load both themes up-front (cheap: 6 × ~32×32 RGBA = ~24 KB) so the
/// theme toggle doesn't have to re-rasterise on first paint.
#[derive(Default)]
struct IconCache {
    play_dark:    Option<egui::TextureHandle>,
    play_light:   Option<egui::TextureHandle>,
    stop_dark:    Option<egui::TextureHandle>,
    stop_light:   Option<egui::TextureHandle>,
    reset_dark:   Option<egui::TextureHandle>,
    reset_light:  Option<egui::TextureHandle>,
}

impl IconCache {
    fn load(ctx: &egui::Context) -> Self {
        Self {
            play_dark:   load_sf_icon(ctx, "play.circle.fill",                  true),
            play_light:  load_sf_icon(ctx, "play.circle.fill",                  false),
            stop_dark:   load_sf_icon(ctx, "stop.circle.fill",                  true),
            stop_light:  load_sf_icon(ctx, "stop.circle.fill",                  false),
            reset_dark:  load_sf_icon(ctx, "arrow.counterclockwise.circle.fill", true),
            reset_light: load_sf_icon(ctx, "arrow.counterclockwise.circle.fill", false),
        }
    }

    fn play(&self, dark: bool) -> Option<&egui::TextureHandle> {
        if dark { self.play_dark.as_ref() } else { self.play_light.as_ref() }
    }
    fn stop(&self, dark: bool) -> Option<&egui::TextureHandle> {
        if dark { self.stop_dark.as_ref() } else { self.stop_light.as_ref() }
    }
    fn reset(&self, dark: bool) -> Option<&egui::TextureHandle> {
        if dark { self.reset_dark.as_ref() } else { self.reset_light.as_ref() }
    }
}

/// Rasterise an SF Symbol via AppKit, upload as an egui texture.  The
/// 16-pt point size matches the medium SF Symbol scale used in
/// SwiftUI's `Image(systemName:).imageScale(.medium)`.  Returns `None`
/// off-macOS or if the symbol isn't found.
fn load_sf_icon(ctx: &egui::Context, name: &str, dark_mode: bool) -> Option<egui::TextureHandle> {
    let (bytes, w, h) = platform::render_sf_symbol(name, 16.0, dark_mode)?;
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &bytes,
    );
    let id = format!("sf:{}:{}", name, if dark_mode { "d" } else { "l" });
    Some(ctx.load_texture(id, image, egui::TextureOptions::LINEAR))
}

// Fixed logical width of the settings window.  Wider than the Swift 246 pt
// reference so the segmented-picker column (right-aligned, uniform width
// across every row) has room for "Rectangle" / "Sinusoidal" without
// truncation while still leaving a visible gap between the left-aligned
// label column and the right-aligned picker column.
const SETTINGS_WIDTH:      u32 = 360;
/// Lower bound when dragging the bottom edge.  Tuned so the window can
/// shrink down to roughly "Controls + Appearance card top", with everything
/// below the drag point scrollable — matches Swift's resize behaviour.
const SETTINGS_MIN_HEIGHT: u32 = 428;

impl SettingsWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
        settings:   &exhale_core::settings::Settings,
    ) -> Result<Self> {
        // Width is fixed; only height is user-resizable.  Max height is set
        // later (once egui has measured the natural content size) so the
        // window can never extend past the last visible setting.
        //
        // Default height shows Controls + Appearance + Timing + Randomization
        // comfortably; the Timers section lives just below the fold so the
        // user has to scroll to reach it, giving a compact settings surface
        // without hiding the commonly-tweaked rows.  Saved height (when
        // present) wins over the default — the user's own resize is always
        // respected on relaunch.
        const INITIAL_PREFERRED_H: u32 = 796;
        let initial_h = settings.settings_window_height
            .unwrap_or(INITIAL_PREFERRED_H)
            .max(SETTINGS_MIN_HEIGHT);
        // Request a transparent window everywhere EXCEPT Windows.  On
        // macOS / Linux X11, winit's `with_transparent(true)` selects an
        // alpha-capable visual / clearColor background so the OS-level
        // blur effect (NSVisualEffectView child window, KDE
        // blur-behind) can show through where egui doesn't paint.
        //
        // Windows is special: winit's transparent flag adds
        // `WS_EX_LAYERED` to the window, which (a) is incompatible with
        // `DWMWA_SYSTEMBACKDROP_TYPE` (DWM silently ignores acrylic on
        // layered windows), and (b) breaks DXGI flip-model swap chains
        // that wgpu uses — the result is an opaque black window even
        // though every other transparency knob is set.  On Windows we
        // get a normal HWND, then DWMWA_SYSTEMBACKDROP_TYPE composes
        // acrylic behind the entire client area, and our wgpu surface's
        // `PostMultiplied` alpha mode lets DWM blend our transparent
        // pixels over that acrylic.  No layered window needed.
        let want_transparent = !cfg!(target_os = "windows");
        let attrs = Window::default_attributes()
            .with_title("exhale")
            .with_inner_size(winit::dpi::LogicalSize::new(SETTINGS_WIDTH, initial_h))
            .with_min_inner_size(winit::dpi::LogicalSize::new(SETTINGS_WIDTH, SETTINGS_MIN_HEIGHT))
            .with_resizable(true)
            .with_transparent(want_transparent)
            .with_decorations(true);

        let window = Arc::new(event_loop.create_window(attrs)?);

        // Restore saved position if available.  Saved x/y are stored as an
        // offset relative to `settings_window_screen`; when that monitor is
        // still connected we anchor against its current origin (so the window
        // follows a rearranged display), matching Swift's
        // `NSScreen.screens.first(where: { $0.localizedName == screenName })`
        // lookup in AppDelegate.toggleSettings.  With no saved screen or a
        // disconnected one, fall back to treating the offset as absolute.
        if let (Some(x), Some(y)) = (settings.settings_window_x, settings.settings_window_y) {
            let (abs_x, abs_y) = match &settings.settings_window_screen {
                Some(name) => {
                    let matching = event_loop.available_monitors()
                        .find(|m| m.name().as_deref() == Some(name.as_str()));
                    match matching {
                        Some(m) => {
                            let o = m.position();
                            (o.x + x, o.y + y)
                        }
                        None => (x, y),
                    }
                }
                None => (x, y),
            };
            window.set_outer_position(winit::dpi::PhysicalPosition::new(abs_x, abs_y));
        }

        platform::setup_settings_window(&window);

        let surface: wgpu::Surface<'static> =
            gpu.instance.create_surface(Arc::clone(&window))?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(
            &pollster::block_on(gpu.instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                }
            )).context("settings adapter")?
        );
        let format = caps.formats.iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);

        // Pick `PostMultiplied` whenever the platform supports it so the
        // OS-level blur (macOS VEV, Windows DWM acrylic, KDE blur-behind)
        // can composite through transparent pixels in our render.
        // `PreMultiplied` is the next-best alpha-respecting mode (some
        // Linux/Wayland adapters expose it instead of PostMultiplied).
        // Fall back to `Auto` (typically Opaque) if neither is available
        // — `clear_color_for_theme` handles that case by rendering the
        // window opaquely.
        let alpha_mode =
            if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PostMultiplied) {
                wgpu::CompositeAlphaMode::PostMultiplied
            } else if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            };

        let config = wgpu::SurfaceConfiguration {
            usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width:                         size.width.max(1),
            height:                        size.height.max(1),
            present_mode:                  wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats:                  vec![],
        };
        surface.configure(&gpu.device, &config);

        // Install the NSVisualEffectView with a theme-appropriate material
        // so the Dark-mode vibrancy uses a neutral blend (underWindowBackground)
        // that doesn't lighten dark backdrops, while Light mode uses hudWindow
        // for a visibly translucent blur over bright desktops.
        let initial_theme = window.theme().unwrap_or(Theme::Dark);
        let vev_ptr = platform::install_settings_vibrancy(
            &window, matches!(initial_theme, Theme::Dark),
        );

        let egui_ctx = egui::Context::default();

        // Swap in the OS-native UI font (SF Pro on macOS, Segoe UI on Windows,
        // Ubuntu/Cantarell/Noto on Linux) so text in our settings window
        // matches the rest of the OS's system preferences.  Silently falls
        // back to egui's default Ubuntu font if the platform's system font
        // isn't locatable — nothing critical breaks.
        install_system_ui_font(&egui_ctx);

        // Pre-populate both style buckets with our custom visuals.  egui owns
        // separate dark_style / light_style slots and picks one per-frame
        // based on `ThemePreference` + `system_theme`.  Populating both up
        // front means whichever bucket egui selects already contains the
        // correct visuals — no rewrite-on-switch, no risk of writing to the
        // wrong bucket under a rapid toggle race.
        egui_ctx.set_visuals_of(egui::Theme::Dark,  visuals_for_theme(Theme::Dark));
        egui_ctx.set_visuals_of(egui::Theme::Light, visuals_for_theme(Theme::Light));

        // Pin the theme preference explicitly (not `System`) so egui never
        // flips styles on our behalf based on a stale egui_winit `system_theme`
        // — we remain the single authoritative source, driven by the
        // render-time `window.theme()` poll below.
        let theme = initial_theme;
        egui_ctx.set_theme(theme_preference(theme));

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            ViewportId::ROOT,
            &*window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(&*gpu.device, format, None, 1, false);

        // Pre-rasterise SF Symbol icons for both themes — cheap one-shot
        // cost (~6 small RGBA blobs uploaded as textures) so the theme
        // toggle doesn't have to lock-focus into AppKit on the hot path.
        let icon_cache = IconCache::load(&egui_ctx);

        Ok(Self {
            window, surface, config, egui_ctx, egui_state, egui_renderer, gpu,
            pending_reset: false,
            theme,
            vev_ptr,
            icon_cache,
        })
    }

    /// Forward a window event to egui.
    /// Returns (consumed, wants_repaint) — the caller uses `wants_repaint`
    /// to drive redraws instead of polling every idle tick.
    pub fn on_window_event(&mut self, event: &WindowEvent) -> (bool, bool) {
        let response = self.egui_state.on_window_event(&*self.window, event);
        match event {
            WindowEvent::Resized(size) => self.resize(*size),
            // We intentionally do NOT use the event's Theme payload: during
            // rapid System Settings toggles the queue can hold an in-flight
            // event whose value is already stale by the time we dequeue it,
            // leaving the window inverted for one frame.  Instead, just
            // schedule a redraw — the render-time poll of `window.theme()`
            // is the single authoritative source.
            WindowEvent::ThemeChanged(_) => self.window.request_redraw(),
            _ => {}
        }
        (response.consumed, response.repaint)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 { return; }
        self.config.width  = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.gpu.device, &self.config);
        // Keep the vibrancy backdrop NSWindow the same size as the
        // settings window — AppKit auto-tracks child-window position but
        // NOT size, so we copy the parent's frame onto the backdrop here.
        // No-op on non-macOS or when `EXHALE_DISABLE_BLUR` is set.
        platform::sync_settings_backdrop_frame(self.vev_ptr);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Raise the Reset confirmation dialog on the next frame.  Used by the
    /// Ctrl+Shift+F global hotkey, mirroring Swift's `showResetConfirmation`.
    #[cfg(feature = "global-hotkeys")]
    pub fn request_reset_confirmation(&mut self) {
        self.pending_reset = true;
    }

    /// Render one egui frame onto the settings surface.
    /// Returns the `repaint_delay` egui requests for its next frame — used by
    /// the caller to schedule the next repaint via a deadline instead of
    /// polling every idle tick.  `Duration::MAX` means no scheduled repaint.
    pub fn render(
        &mut self,
        settings:         &mut Settings,
        settings_manager: &Arc<SettingsManager>,
    ) -> Result<std::time::Duration> {
        // Reconcile against the authoritative OS theme every frame.
        // `WindowEvent::ThemeChanged` can coalesce or drop during rapid
        // System Settings toggles on macOS/Windows, which leaves the egui
        // visuals inverted from the real appearance.  Re-querying here
        // guarantees the window can never stay out of sync for more than
        // one frame regardless of how events were delivered.
        //
        // We flip egui's `ThemePreference` (not `set_visuals`) because both
        // style buckets were pre-populated in `new()`.  `set_visuals` would
        // write into whichever bucket `ctx.theme()` currently resolves to —
        // and under a rapid toggle that can be the wrong bucket (egui_winit
        // may not have fed the latest `system_theme` yet), leaving the
        // wrong-colour visuals stuck in the bucket egui later selects.
        if let Some(current) = self.window.theme() {
            if current != self.theme {
                self.theme = current;
                self.egui_ctx.set_theme(theme_preference(current));
                // Sync the NSVisualEffectView's material + appearance so
                // the vibrancy blur tint follows the Light/Dark toggle.
                // We do this ourselves because `install_settings_vibrancy`
                // pinned the VEV's appearance explicitly (to avoid the
                // AppKit auto-propagation crash) — without this call the
                // blur would stay frozen at its install-time theme.
                platform::update_settings_vibrancy(
                    self.vev_ptr,
                    matches!(current, Theme::Dark),
                );
            }
        }

        let output = match self.surface.get_current_texture() {
            Ok(t)  => t,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.gpu.device, &self.config);
                return Ok(std::time::Duration::MAX);
            }
            Err(e) => return Err(e).context("settings get_current_texture"),
        };

        let raw_input = self.egui_state.take_egui_input(&*self.window);
        let pixels_per_point = self.window.scale_factor() as f32;

        let mut content_height: f32 = 0.0;
        let mut full_output = self.egui_ctx.run(raw_input, |ctx| {
            content_height = settings_ui(
                ctx, settings, settings_manager,
                &mut self.pending_reset,
                &self.icon_cache,
            );
        });

        // egui populates a repaint_delay per viewport — respect it so we can
        // stop blindly repainting every idle tick.  Short delays keep tooltip
        // fade-ins and button-press animations working; `Duration::MAX` means
        // nothing is animating and the window can sit idle until the next
        // user/external event.
        let repaint_delay = full_output
            .viewport_output
            .get(&ViewportId::ROOT)
            .map(|v| v.repaint_delay)
            .unwrap_or(std::time::Duration::MAX);

        // Cap the resizable window to the exact amount of content egui just
        // laid out.  The `+ 24.0` covers the CentralPanel's inner padding that
        // `content_size` doesn't itself include.  This runs every frame so new
        // settings/added rows grow the max automatically, and stays cheap
        // because winit no-ops redundant set_*_inner_size calls.
        if content_height > 0.0 {
            // Cap the user-drag upper bound at the exact content height so
            // they can't drag past the last control (empty space below the
            // Timers card looks wrong).  Window starts at the compact
            // `INITIAL_PREFERRED_H`; overflow is handled by the ScrollArea.
            let natural_h = (content_height + 24.0).ceil().max(SETTINGS_MIN_HEIGHT as f32) as u32;
            self.window.set_max_inner_size(Some(
                winit::dpi::LogicalSize::new(SETTINGS_WIDTH, natural_h),
            ));
        }

        // On macOS: bypass `handle_platform_output` entirely and pipe ONLY
        // the clipboard update through.  `handle_platform_output` internally
        // calls `window.set_cursor_visible`, `window.set_cursor`, and
        // `window.set_ime_*`, each of which takes a `borrow_mut()` on the
        // `cursor_state` RefCell living inside winit's custom NSView
        // subclass.  AppKit's `resetCursorRects` callback borrows the same
        // RefCell (shared) during scroll-wheel event dispatch, so any
        // overlap produces the classic "RefCell already borrowed" panic on
        // a two-finger scroll.  Separately, the same pipeline has produced
        // `objc_retain` segfaults mid-session when the ivar backing the
        // NSCursor reference becomes stale (likely a consequence of our
        // vibrancy install reparenting winit's NSView under a sibling
        // container, which winit's cursor tracking doesn't expect).
        //
        // Neither hazard is reachable if we never let egui_winit hand the
        // platform output back to winit.  We do still need clipboard copy
        // (egui TextEdits write `copied_text` when the user presses ⌘C),
        // and `set_clipboard_text` is a pure arboard call that never
        // touches winit state — safe to invoke from here.
        //
        // Cost on macOS: no cursor-icon changes (buttons / TextEdits keep
        // the default arrow), no IME cursor positioning (acceptable for a
        // Latin-text settings window), and no open_url handling (we don't
        // generate URL output anywhere).  All other platforms follow the
        // normal path.
        #[cfg(target_os = "macos")]
        {
            let copied = std::mem::take(&mut full_output.platform_output.copied_text);
            if !copied.is_empty() {
                self.egui_state.set_clipboard_text(copied);
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.egui_state.handle_platform_output(
                &*self.window,
                full_output.platform_output,
            );
        }

        let primitives = self.egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen_desc = ScreenDescriptor {
            size_in_pixels:  [self.config.width, self.config.height],
            pixels_per_point,
        };

        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("egui-frame") }
        );

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&*self.gpu.device, &*self.gpu.queue, *id, delta);
        }
        self.egui_renderer.update_buffers(
            &*self.gpu.device, &*self.gpu.queue, &mut encoder, &primitives, &screen_desc,
        );

        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label:                    Some("egui-pass"),
                color_attachments:        &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    ops:            wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(clear_color_for_theme(self.theme)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes:         None,
                occlusion_query_set:      None,
            });
            let mut pass = pass.forget_lifetime();
            self.egui_renderer.render(&mut pass, &primitives, &screen_desc);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(repaint_delay)
    }
}

// ─── Settings UI ─────────────────────────────────────────────────────────────
//
// Layout mirrors the Swift SettingsView exactly:
//   • Controls  — Start / Stop / Reset
//   • Appearance — colors, opacity, shape, gradient, animation, ripple, visibility
//   • Timing    — 4 phase durations
//   • Randomization — 4 jitter sliders + drift
//   • Timers    — reminder + auto-stop

/// Returns the natural (fully-expanded) content height in logical points so
/// the caller can clamp the window's max size.
fn settings_ui(
    ctx:              &egui::Context,
    settings:         &mut Settings,
    settings_manager: &Arc<SettingsManager>,
    pending_reset:    &mut bool,
    icons:            &IconCache,
) -> f32 {
    let mut dirty = false;
    let mut content_height = 0.0f32;

    // Swift's settings window has 14 pt horizontal padding + 14 pt top/bottom,
    // with a ScrollView wrapping everything below the pinned Controls card.
    // Mirror that with a frame-less CentralPanel whose inner_margin supplies
    // the outer breathing room, and a vertical ScrollArea inside.  The panel
    // fill is kept transparent so the macOS NSVisualEffectView (installed at
    // window level) shows through between cards.
    // Panel fill: on macOS we lay a semi-opaque tint over the NSVisualEffectView
    // so the vibrancy doesn't let detail from windows behind exhale bleed
    // through between the SectionCards — text like code/terminal content
    // reading as "e (%)" / "ally" ghost characters at the card's edges was
    // what read as "sections are clipped by the window edge" in the earlier
    // builds.  Alpha ~160 is dense enough to mute backdrop content while
    // still showing the blurred desktop colour (so the vibrancy effect
    // reads as a subtle tint, not a plain solid background).
    // On macOS the gutters between/around the cards are painted entirely by
    // the NSVisualEffectView vibrancy (see `platform::install_settings_vibrancy`).
    //   - In Dark mode, `.hudWindow` renders as a darkish blur — composites
    //     nicely with the translucent cards without any panel overlay.
    //   - In Light mode, `.hudWindow` is near-white with desktop colour tint
    //     coming through the blur; earlier attempts at a near-white panel
    //     overlay composited right back into "solid white" and hid the
    //     vibrancy entirely.
    // Fully-transparent panel fill lets the vibrancy be the sole gutter
    // material in both themes, matching Swift's look exactly.
    //
    // The NSVisualEffectView's own blur masks backdrop content (terminal
    // text, etc.) enough that nothing legible leaks through the 14-px
    // gutters — we verified this after switching the material to `.hudWindow`
    // (same as Swift).
    //
    // Other platforms: opaque fallback (they have no vibrancy backend).
    let panel_fill = if platform::is_blur_active() {
        egui::Color32::TRANSPARENT
    } else if ctx.style().visuals.dark_mode {
        egui::Color32::from_rgb(24, 24, 28)
    } else {
        egui::Color32::from_rgb(240, 240, 242)
    };
    egui::CentralPanel::default()
        .frame(egui::Frame::none()
            .fill(panel_fill)
            .inner_margin(egui::Margin::symmetric(OUTER_PAD, OUTER_PAD)))
        .show(ctx, |ui| {
        // Cap the ScrollArea's max width to the panel content area so the
        // sections it contains can never be wider than the window's 14-px
        // horizontal gutters.  Using `max_width` here (rather than
        // `ui.set_width` on each card) propagates down through the nested
        // `available_width` chain — so rows inside the cards also know the
        // true right edge and right-aligned widgets (color swatches,
        // stepper fields, segmented pickers) end up flush with the card
        // boundary instead of extending past it.
        let scroll_max_w = SETTINGS_WIDTH as f32 - 2.0 * OUTER_PAD;
        let scroll_out = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_width(scroll_max_w)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
            // `sectionSpacing: 10` — vertical gap between SectionCard instances.
            ui.spacing_mut().item_spacing.y = SECTION_GAP;

            // Measure every segmented picker in the Appearance section once,
            // pick the widest natural width, and use it as the uniform column
            // width for all of them.  This guarantees the leftmost option of
            // every picker lands on the same X, regardless of option count
            // or text length — and the column only extends as far left as
            // the widest picker requires.
            let picker_column_w = uniform_picker_column_width(ui, &[
                &["Rectangle", "Circle", "Full"],
                &["Inner", "Off", "On"],
                &["Linear", "Sinusoidal"],
                &["Gradient", "Stark", "Off"],
                &["Top Bar", "Dock", "Both"],
            ]);

            // ── Controls (no header — matches Swift's top SectionCard) ───────
            // Swift has only THREE buttons: Start, Stop, Reset.  Pause is
            // implemented in `SettingsModel` but isn't exposed in the
            // SwiftUI `SettingsView`, so we omit it here too for 1:1
            // parity.  `Ctrl+Shift+S` (stop) handles "I want it to halt"
            // and the breathing controller treats `is_animating=false`
            // the same as paused for the purposes of stopping renders.
            section(ui, "", |ui| {
                ui.horizontal(|ui| {
                    const BUTTON_SPACING: f32 = 8.0;
                    ui.spacing_mut().item_spacing.x = BUTTON_SPACING;
                    let n_buttons = 3.0_f32;
                    let avail = ui.available_width();
                    let btn_w = ((avail - BUTTON_SPACING * (n_buttons - 1.0))
                                 / n_buttons)
                                .floor()
                                .max(1.0);

                    let dark = ui.visuals().dark_mode;
                    if control_button(
                        ui, btn_w,
                        "\u{25B6}", icons.play(dark),
                        "Start",
                        "Start the app and re-initialize animation.",
                    ).clicked()
                    {
                        settings.is_animating = true;
                        settings.is_paused    = false;
                        dirty = true;
                    }
                    if control_button(
                        ui, btn_w,
                        "\u{25A0}", icons.stop(dark),
                        "Stop",
                        "Stop the animation and remove all screen tints.",
                    ).clicked()
                    {
                        settings.is_animating = false;
                        settings.is_paused    = false;
                        dirty = true;
                    }
                    if control_button(
                        ui, btn_w,
                        "\u{21BA}", icons.reset(dark),
                        "Reset",
                        "Reset all settings to their default values.",
                    ).clicked()
                    {
                        let was_animating   = settings.is_animating;
                        let was_paused      = settings.is_paused;
                        let win_x           = settings.settings_window_x;
                        let win_y           = settings.settings_window_y;
                        let win_h           = settings.settings_window_height;
                        let win_screen      = settings.settings_window_screen.clone();
                        *settings = Settings::default();
                        settings.is_animating            = was_animating;
                        settings.is_paused               = was_paused;
                        settings.settings_window_x       = win_x;
                        settings.settings_window_y       = win_y;
                        settings.settings_window_height  = win_h;
                        settings.settings_window_screen  = win_screen;
                        dirty = true;
                    }
                });
            });

            // ── Appearance ───────────────────────────────────────────────────
            section(ui, "Appearance", |ui| {
                // Inhale color — no alpha (Swift: supportsOpacity: false)
                labeled_row(ui, "Inhale Color", |ui| {
                    let mut c = to_color32(settings.inhale_color);
                    if egui::color_picker::color_edit_button_srgba(
                        ui, &mut c, egui::color_picker::Alpha::Opaque,
                    ).changed() {
                        settings.inhale_color = from_color32_opaque(c);
                        dirty = true;
                    }
                }).on_hover_text("Choose the color for the inhale phase.");

                // Exhale color — no alpha (Swift: supportsOpacity: false)
                labeled_row(ui, "Exhale Color", |ui| {
                    let mut c = to_color32(settings.exhale_color);
                    if egui::color_picker::color_edit_button_srgba(
                        ui, &mut c, egui::color_picker::Alpha::Opaque,
                    ).changed() {
                        settings.exhale_color = from_color32_opaque(c);
                        dirty = true;
                    }
                }).on_hover_text("Choose the color for the exhale phase.");

                // Background color (with alpha) — disabled for Fullscreen (matches Swift)
                labeled_row(ui, "Background Color", |ui| {
                    ui.add_enabled_ui(settings.shape != AnimationShape::Fullscreen, |ui| {
                        let mut c = to_color32(settings.background_color);
                        if egui::color_picker::color_edit_button_srgba(
                            ui, &mut c, egui::color_picker::Alpha::OnlyBlend,
                        ).changed() {
                            settings.background_color = from_color32(c);
                            dirty = true;
                        }
                    });
                }).on_hover_text("Choose the background color. No effect when Shape is Fullscreen.");

                // Overlay opacity — Swift stores 0.0..1.0, displays 0..100 %.
                // Wrap with an f64 shim because ValueScale::Percent operates
                // on `*value / 100.0` and settings.overlay_opacity is f32.
                let mut opacity_pct = settings.overlay_opacity as f64;
                if stepper_row(
                    ui, "Overlay Opacity (%)",
                    "Transparency of the overlay. Lower = more transparent.",
                    None, &mut opacity_pct, 1.0, 0.0, Some(100.0),
                    ValueScale::Percent,
                ) {
                    settings.overlay_opacity = opacity_pct as f32;
                    dirty = true;
                }

                // Shape
                if segmented_row(
                    ui, "Shape",
                    "Shape of the animation: Fullscreen, Rectangle, or Circle.",
                    true, picker_column_w,
                    &mut settings.shape,
                    &[
                        ("Rectangle", AnimationShape::Rectangle),
                        ("Circle",    AnimationShape::Circle),
                        ("Full",      AnimationShape::Fullscreen),
                    ],
                ) { dirty = true; }

                // Gradient — order matches Swift's enum declaration (Inner, Off, On)
                // so segmented-picker placement is identical to the macOS app.
                if segmented_row(
                    ui, "Gradient",
                    "Gradient color effect. No effect when Shape is Fullscreen.",
                    settings.shape != AnimationShape::Fullscreen, picker_column_w,
                    &mut settings.color_fill_gradient,
                    &[
                        ("Inner", ColorFillGradient::Inner),
                        ("Off",   ColorFillGradient::Off),
                        ("On",    ColorFillGradient::On),
                    ],
                ) { dirty = true; }

                // Animation mode — labels use Swift's enum raw values.
                if segmented_row(
                    ui, "Animation",
                    "Sinusoidal eases in/out naturally. Linear is constant speed.",
                    true, picker_column_w,
                    &mut settings.animation_mode,
                    &[
                        ("Linear",     AnimationMode::Linear),
                        ("Sinusoidal", AnimationMode::Sinusoidal),
                    ],
                ) { dirty = true; }

                // Hold ripple — order matches Swift's enum declaration
                // (Gradient, Stark, Off) so the default (Gradient) sits first.
                if segmented_row(
                    ui, "Hold Ripple",
                    "Hold phase ripple: Gradient (smooth glow), Stark (solid edge), or Off.",
                    true, picker_column_w,
                    &mut settings.hold_ripple_mode,
                    &[
                        ("Gradient", HoldRippleMode::Gradient),
                        ("Stark",    HoldRippleMode::Stark),
                        ("Off",      HoldRippleMode::Off),
                    ],
                ) { dirty = true; }

                // App visibility (macOS concept; show on all platforms for settings parity)
                if segmented_row(
                    ui, "Show In",
                    "Where exhale appears: Top Bar, Dock, or Both.",
                    true, picker_column_w,
                    &mut settings.app_visibility,
                    &[
                        ("Top Bar", AppVisibility::TopBarOnly),
                        ("Dock",    AppVisibility::DockOnly),
                        ("Both",    AppVisibility::Both),
                    ],
                ) { dirty = true; }
            });

            // ── Timing ───────────────────────────────────────────────────────
            section(ui, "Timing", |ui| {
                if duration_row(ui, "Inhale Duration (s)",  "Duration of the inhale phase, in seconds.",             &mut settings.inhale_duration) { dirty = true; }
                if duration_row(ui, "Post-Inhale Hold (s)", "Hold/pause duration at the end of inhale, in seconds.", &mut settings.post_inhale_hold_duration) { dirty = true; }
                if duration_row(ui, "Exhale Duration (s)",  "Duration of the exhale phase, in seconds.",             &mut settings.exhale_duration) { dirty = true; }
                if duration_row(ui, "Post-Exhale Hold (s)", "Hold/pause duration at the end of exhale, in seconds.", &mut settings.post_exhale_hold_duration) { dirty = true; }
            });

            // ── Randomization ────────────────────────────────────────────────
            section(ui, "Randomization", |ui| {
                if pct_row(ui, "Inhale (%)",           "Randomize inhale duration by this percentage.",            &mut settings.randomized_timing_inhale) { dirty = true; }
                if pct_row(ui, "Post-Inhale Hold (%)", "Randomize post-inhale hold duration by this percentage.",  &mut settings.randomized_timing_post_inhale_hold) { dirty = true; }
                if pct_row(ui, "Exhale (%)",           "Randomize exhale duration by this percentage.",            &mut settings.randomized_timing_exhale) { dirty = true; }
                if pct_row(ui, "Post-Exhale Hold (%)", "Randomize post-exhale hold duration by this percentage.",  &mut settings.randomized_timing_post_exhale_hold) { dirty = true; }

                // Drift — stored as a per-cycle multiplier (1.01 = +1 %), displayed
                // as a percentage above 1.0 so "1" reads as "+1 % per cycle".
                // Swift's stepper has no upper limit (`max: nil`).  Minimum
                // stays at 0 % (drift = 1.0) per the `defaultMin` clamp.
                if stepper_row(
                    ui, "Drift (%)",
                    "Multiplicative drift per cycle. 1-5% recommended for gradually lengthening breath.",
                    None, &mut settings.drift, 1.0, 0.0, None,
                    ValueScale::DriftPercent,
                ) { dirty = true; }
            });

            // ── Timers ───────────────────────────────────────────────────────
            section(ui, "Timers", |ui| {
                if stepper_row(
                    ui, "Reminder (mins)",
                    "Notification reminder every N minutes. 0 to disable.",
                    Some("0 = off"),
                    &mut settings.reminder_interval_minutes,
                    1.0, 0.0, None,
                    ValueScale::Identity,
                ) { dirty = true; }
                if stepper_row(
                    ui, "End After (mins)",
                    "Auto-stop after N minutes. 0 to disable.",
                    Some("0 = off"),
                    &mut settings.auto_stop_minutes,
                    1.0, 0.0, None,
                    ValueScale::Identity,
                ) { dirty = true; }
            });
        });
        content_height = scroll_out.content_size.y;
    });

    if dirty {
        settings_manager.mark_dirty();
    }

    // ── Reset confirmation dialog ─────────────────────────────────────────────
    if *pending_reset {
        egui::Window::new("Reset to Defaults")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_width(210.0);
                ui.label("Reset all settings to their default values? This cannot be undone.");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        // Preserve runtime/window state that Swift's resetToDefaults
                        // doesn't touch either (isAnimating is kept; window position
                        // is a Rust-only persistence that we don't want to lose).
                        let was_animating   = settings.is_animating;
                        let was_paused      = settings.is_paused;
                        let win_x           = settings.settings_window_x;
                        let win_y           = settings.settings_window_y;
                        let win_h           = settings.settings_window_height;
                        let win_screen      = settings.settings_window_screen.clone();
                        *settings = Settings::default();
                        settings.is_animating            = was_animating;
                        settings.is_paused               = was_paused;
                        settings.settings_window_x       = win_x;
                        settings.settings_window_y       = win_y;
                        settings.settings_window_height  = win_h;
                        settings.settings_window_screen  = win_screen;
                        settings_manager.mark_dirty();
                        *pending_reset = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *pending_reset = false;
                    }
                });
            });
    }

    content_height
}

// ─── UI helpers ───────────────────────────────────────────────────────────────

/// Swift's SectionCard: 10 px rounded rect, 1 px stroke at `Color.primary.opacity(0.06)`,
/// fill at `Color(NSColor.controlBackgroundColor).opacity(0.55)`, 12 px internal padding.
/// Header (when present) is 10 pt uppercase `.secondary` with 0.8 pt letter-spacing.
fn section(ui: &mut egui::Ui, header: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let dark_mode = ui.visuals().dark_mode;

    // Swift's SectionCard fill is `Color(NSColor.controlBackgroundColor)
    // .opacity(0.55)` — over an NSVisualEffectView .hudWindow backdrop that
    // renders at ~80% (dark) or ~85% (light) luminance, this produces cards
    // that are *barely* distinguishable from the vibrancy.  Matching that
    // with a hand-tuned premul-unaware fill:
    //   dark  — controlBackgroundColor ≈ #1E1E1E, .55 alpha ≈ 86 out of 255
    //           but vibrancy already tints toward dark, so the visible delta
    // EXACT match for Swift's `SectionCard.fill`:
    //   Color(NSColor.controlBackgroundColor).opacity(0.55)
    // `controlBackgroundColor`:
    //   dark  → (0.118, 0.118, 0.118, 1.0) ≈ #1E1E1E (RGB 30, 30, 30)
    //   light → #FFFFFF (RGB 255, 255, 255)
    // `.opacity(0.55)` → alpha = 140 / 255.  Composited over the
    // NSVisualEffectView's popover/hudWindow material, this gives
    // Swift's "dark dark gray" card in dark mode and a translucent
    // white card in light mode.
    let fill = if dark_mode {
        egui::Color32::from_rgba_unmultiplied(30, 30, 30, 140)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 140)
    };
    // Constrain the card to exactly the scroll area's viewport width so every
    // section (Controls, Appearance, Timing, Randomization, Timers) aligns at
    // the same left and right gutters.
    let target_w = (SETTINGS_WIDTH as f32 - 2.0 * OUTER_PAD).min(ui.available_width());
    ui.allocate_ui_with_layout(
        egui::vec2(target_w, 0.0),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_max_width(target_w);
            // No stroke on the section Frame — the translucent `fill` alone
            // separates the card from the vibrancy gutters.  The outlined
            // look is reserved for the control buttons inside the top card,
            // matching Swift's `ControlButton.strokeBorder` treatment.
            egui::Frame::none()
                .inner_margin(CARD_PAD)
                .rounding(CARD_RADIUS)
                .fill(fill)
                .show(ui, |ui| {
                    ui.set_max_width(target_w - 2.0 * CARD_PAD);
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = ROW_GAP;

                    if !header.is_empty() {
                        section_header(ui, header);
                    }
                    add_contents(ui);
                });
        },
    );
}

/// Uppercase, letter-spaced, `.secondary` header mimicking SwiftUI's
/// `.font(.system(size: 10, weight: .semibold)).foregroundColor(.secondary).tracking(0.8)`.
fn section_header(ui: &mut egui::Ui, text: &str) {
    use egui::text::{LayoutJob, TextFormat};

    let dark_mode = ui.visuals().dark_mode;
    // `.secondary` ≈ 60% of primary.  Picked by eye to match SwiftUI's
    // secondaryLabelColor (#EBEBF599 on dark, #3C3C4399 on light).
    let color = if dark_mode {
        egui::Color32::from_rgb(160, 160, 166)
    } else {
        egui::Color32::from_rgb(99, 99, 106)
    };

    let mut job = LayoutJob::default();
    job.append(
        &text.to_uppercase(),
        0.0,
        TextFormat {
            font_id: egui::FontId::proportional(10.0),
            color,
            // SwiftUI tracking(0.8) → 0.8 pt extra between glyphs.
            extra_letter_spacing: 0.8,
            ..Default::default()
        },
    );
    ui.label(job);
    ui.add_space(2.0);
}

/// SwiftUI `ControlButton`: an icon + 12 pt medium label on a 7 px rounded rect
/// with theme-aware translucent fill + stroke, and a hover/press tint that
/// brightens by ~5% / 8% respectively.  Buttons expand equally across the row
/// (`.frame(maxWidth: .infinity)` in Swift) — we achieve that by dividing the
/// ui's available width by the number of siblings, which egui's horizontal
/// layout with equal allocations gives us for free via `allocate_exact_size`.
/// `icon` is the Unicode fallback glyph used when `icon_texture` is
/// `None` (non-macOS, or symbol rasterisation failed).  When a texture
/// is supplied we render the real SF Symbol bitmap instead.
fn control_button(
    ui:           &mut egui::Ui,
    width:        f32,
    icon:         &str,
    icon_texture: Option<&egui::TextureHandle>,
    text:         &str,
    help:         &str,
) -> egui::Response {
    // Match Swift's `.padding(.vertical, 6)` around a 16-pt SF Symbol +
    // 12-pt label.  ROW_H + 6 ≈ 28 lands on the same physical button
    // height as `ControlButton.swift`.  We were at +10 (= 32) before,
    // which read as too tall next to AppKit's standard pushbutton.
    let button_h = ROW_H + 6.0;
    let size     = egui::vec2(width, button_h);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let enabled   = ui.is_enabled();
    let hovered   = response.hovered() && enabled;
    let pressed   = response.is_pointer_button_down_on() && enabled;
    let dark_mode = ui.visuals().dark_mode;

    // Button styling: solid dark button surface with light text in dark mode,
    // solid light button surface with dark text in light mode — mirroring
    // Swift ControlButton's "lighter wash on darker card" look but with
    // enough contrast to remain legible against the opaque cards we now use.
    //
    // Dark mode: card is near-solid #1C1C20; button surface slightly lighter
    //            (#333338) so the button reads as a distinct tile; border
    //            and text pure white.
    // Light mode: card is #F0F0F2; button surface slightly darker (#D8D8DC)
    //             for the same "distinct tile" effect; border and text pure
    //             black.
    // EXACT match for Swift's `ControlButton`:
    //   .background(RoundedRectangle(7).fill(Color.primary.opacity(rest:0.05/hover:0.10)))
    //   .overlay(RoundedRectangle(7).strokeBorder(Color.primary.opacity(rest:0.12/hover:0.20), lineWidth: 1))
    // `Color.primary` is white in dark mode and black in light mode, so the
    // button is a translucent wash + outline of the *foreground* colour
    // over whatever's behind (the card fill).
    let primary = if dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK };
    // Light mode keeps Swift's exact `Color.primary.opacity(...)` —
    // black wash over the light card / vibrancy reads as a faint
    // depressed tile, matching ControlButton.swift.
    //
    // Dark mode deliberately deviates: instead of Swift's white wash
    // (which our compositing path makes brighter than the AppKit
    // version), we use a BLACK wash so the buttons land slightly
    // DARKER than the card behind.  That matches the user's "match the
    // Swift button" goal while keeping the buttons readable as
    // distinct tiles.  Stroke stays as `primary` (white in dark) at a
    // muted alpha for a subtle outline.
    let (fill_color, stroke_a): (egui::Color32, u8) = if dark_mode {
        let (fa, sa) = match (hovered, pressed) {
            (_, true)      => (70, 38),
            (true,  false) => (45, 26),
            (false, false) => (28, 16),
        };
        (egui::Color32::from_rgba_unmultiplied(0, 0, 0, fa), sa)
    } else {
        let (fa, sa) = match (hovered, pressed) {
            (_, true)      => (38, 64),
            (true,  false) => (26, 51),
            (false, false) => (13, 31),
        };
        (egui::Color32::from_rgba_unmultiplied(0, 0, 0, fa), sa)
    };
    let with_alpha = |base: egui::Color32, a: u8| {
        egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), a)
    };

    let painter = ui.painter().clone();
    painter.rect(
        rect,
        BUTTON_RADIUS,
        fill_color,
        egui::Stroke::new(1.0, with_alpha(primary, stroke_a)),
    );

    // Pressed state: Swift uses `.opacity(0.7)` + `.scaleEffect(0.97)`.  Scale
    // is awkward in immediate-mode; drop opacity instead — the user still gets
    // a clear "pressed" read.
    let content_alpha: u8 = if pressed { 178 } else if enabled { 255 } else { 110 };
    let content_color = with_alpha(primary, content_alpha);

    // Icon + label, centered inside the button rect.  When a real SF
    // Symbol texture is available (macOS only) we paint that; otherwise
    // we fall back to the Unicode glyph (▶ ■ ↺).
    let font_label = egui::FontId::proportional(12.0);
    // Match Swift's `.imageScale(.medium)` — 16 pt SF Symbol next to a
    // 12-pt label.
    let icon_w     = if icon_texture.is_some() { 16.0 } else {
        ui.fonts(|f| f.layout_no_wrap(icon.to_string(), egui::FontId::proportional(14.0), content_color).size()).x
    };
    let label_size = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font_label.clone(), content_color).size());
    let gap        = 6.0_f32;
    let total_w    = icon_w + gap + label_size.x;
    let start_x    = rect.center().x - total_w * 0.5;
    let baseline_y = rect.center().y;

    if let Some(tex) = icon_texture {
        // Render the SF Symbol texture at 16×16 pt centered on its
        // half-width slot.  `tint(content_color)` re-tints the texture
        // alpha-channel to match `pressed` / `enabled` state — the
        // texture itself is a white silhouette in dark mode and a
        // black silhouette in light mode (see `render_sf_symbol`'s
        // tint pass), and `tint` modulates that further so the icon
        // dims when pressed or disabled the same way the label does.
        let icon_size = 16.0_f32;
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(start_x, baseline_y - icon_size / 2.0),
            egui::vec2(icon_size, icon_size),
        );
        painter.image(
            tex.id(),
            icon_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            content_color,
        );
    } else {
        painter.text(
            egui::pos2(start_x + icon_w * 0.5, baseline_y),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(14.0),
            content_color,
        );
    }

    painter.text(
        egui::pos2(start_x + icon_w + gap + label_size.x * 0.5, baseline_y),
        egui::Align2::CENTER_CENTER,
        text, font_label, content_color,
    );

    response.on_hover_text(help)
}

// Layout constants mirror SwiftUI SettingsView.swift:
//   label column width (115) → SettingsView.settingLabelWidth
//   outer padding    (14)    → .padding(.horizontal, 14) + .padding(.top/.bottom, 14)
//   card padding     (12)    → SectionCard.padding(12)
//   section gap      (10)    → sectionSpacing
//   row gap          (8)     → rowSpacing
//   card radius      (10)    → RoundedRectangle(cornerRadius: 10, style: .continuous)
//   button radius    (7)     → ControlButton's RoundedRectangle(cornerRadius: 7)
//   stepper field    (56)    → CombinedStepperTextField TextField .frame(width: 56)
// Label column width.  Trade-off: shorter keeps the segmented pickers
// (Rectangle/Circle/Full, Gradient/Stark/Off, etc.) from wrapping their
// widest option text, at the cost of ellipsising a few long labels like
// "Overlay Opacity (%)" and "Background Color".  Swift's SettingsView
// uses 115 pt with `.lineLimit(1)` — same behaviour.  The pickers at this
// width get ~191 px to share across 3 segments (≈63 px each), which fits
// "Rectangle" (≈55 px natural) with room to spare when we render buttons
// with `button_padding = 0` inside the segmented row.
const LABEL_W:          f32 = 115.0;
const ROW_H:            f32 = 22.0;
const OUTER_PAD:        f32 = 14.0;
const CARD_PAD:         f32 = 12.0;
const SECTION_GAP:      f32 = 10.0;
const ROW_GAP:          f32 = 8.0;
const CARD_RADIUS:      f32 = 10.0;
const BUTTON_RADIUS:    f32 = 7.0;
const TEXT_EDIT_RADIUS: f32 = 5.0;
const STEPPER_FIELD_W:  f32 = 56.0;

/// Load the OS-native UI font and register it as the default proportional
/// font on the egui context.  Each platform's system-preferences app uses a
/// specific typeface (SF Pro on macOS, Segoe UI on Windows, Ubuntu/Cantarell
/// on common Linux desktops); matching that here makes our settings window
/// read as a native part of the OS instead of egui's default Ubuntu fallback.
///
/// System fonts are NOT redistributed — we read the font file the OS ships
/// with, exactly like every native app does (NSFont on macOS, GDI on
/// Windows, fontconfig on Linux).  No licensing concern.
///
/// If no candidate path exists on the current machine, we silently keep
/// egui's default font — the window still works, it just doesn't blend in
/// quite as well.
fn install_system_ui_font(ctx: &egui::Context) {
    // Ordered list of candidate paths per platform — first-readable wins.
    // Higher-quality faces go first (e.g. SF Pro over Helvetica fallback).
    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &[
        // Big Sur+ variable SF Pro.  SFNS is a TTC collection; egui 0.29
        // can load it via `FontData::from_owned` provided we index into
        // the first face (which we do by default).
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/SFNSDisplay.ttf",
        "/System/Library/Fonts/SFNSText.ttf",
        "/Library/Fonts/SF-Pro.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];
    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\SegoeUI.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
    ];
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let candidates: &[&str] = &[
        // Ubuntu desktop default.
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        // GNOME default on many distros (Fedora, Debian w/ GNOME).
        "/usr/share/fonts/cantarell/Cantarell-VF.otf",
        "/usr/share/fonts/cantarell/Cantarell-Regular.otf",
        // Noto Sans — very broad fallback across distros.
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
        // Last-ditch DejaVu — shipped almost everywhere.
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];

    let Some((path, data)) = candidates.iter().find_map(|p| {
        std::fs::read(p).ok().map(|d| (*p, d))
    }) else {
        log::info!("install_system_ui_font: no candidate font readable; keeping egui default");
        return;
    };
    log::info!("install_system_ui_font: using {path}");

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "system_ui".to_owned(),
        egui::FontData::from_owned(data),
    );
    // Prepend to both families so proportional AND monospace text pick up the
    // system face (egui uses monospace for the debug inspector; we don't want
    // two wildly different typefaces inside the settings window).  The
    // built-in Ubuntu / Emoji entries remain further down the list as glyph
    // fallbacks for anything SF Pro / Segoe UI doesn't cover.
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "system_ui".to_owned());
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "system_ui".to_owned());
    ctx.set_fonts(fonts);
}

/// Resolve the OS-appearance-aware egui visuals used by the settings window.
fn visuals_for_theme(theme: Theme) -> egui::Visuals {
    let mut v = match theme {
        Theme::Dark  => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    v.window_rounding = 10.0.into();

    // Force full-contrast text that reads over the vibrancy-tinted cards in
    // both modes.  egui's defaults (from_gray(140) dark / from_gray(60) light)
    // look washed-out against the translucent SectionCards — especially light
    // mode over hudWindow vibrancy, which is already near-white, so a dark
    // gray label reads as if someone turned the opacity down on the text.
    // Match SwiftUI `.primary` (#FFFFFF on dark, #000000 on light).
    let (fg_text, fg_subtle) = if matches!(theme, Theme::Dark) {
        (egui::Color32::from_rgb(235, 235, 240), egui::Color32::from_rgb(235, 235, 240))
    } else {
        (egui::Color32::from_rgb(20, 20, 22),    egui::Color32::from_rgb(20, 20, 22))
    };
    v.override_text_color = Some(fg_text);
    // Keep widget foregrounds in sync so unselected segmented-picker labels
    // and button text read with full contrast too.
    v.widgets.noninteractive.fg_stroke.color = fg_subtle;
    v.widgets.inactive.fg_stroke.color       = fg_text;
    v.widgets.hovered.fg_stroke.color        = fg_text;
    v.widgets.active.fg_stroke.color         = fg_text;

    // Light mode's default inactive widget stroke is near-invisible, so
    // segmented-picker segments bleed into the panel background.  Bump the
    // inactive stroke to a faint mid-gray so each segment's edge reads at
    // rest — matching the legibility we already get in dark mode and the
    // Swift `NSSegmentedControl` look.
    if matches!(theme, Theme::Light) {
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(180));
    }

    // Round egui widget chrome (TextEdit, checkboxes, comboboxes) to match
    // the macOS-native ~5-6 px corner.  Our hand-painted control buttons,
    // stepper buttons, and segmented picker draw their own chrome via
    // `painter.rect_*` and pass their own rounding constants — they aren't
    // affected by these widget rounding values.
    let r = egui::Rounding::same(TEXT_EDIT_RADIUS);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.rounding       = r;
    v.widgets.hovered.rounding        = r;
    v.widgets.active.rounding         = r;
    v.widgets.open.rounding           = r;

    // TextEdit fills + stepper chrome reads against the card-tinted
    // backdrop.  egui's stock dark mode picks near-black for both
    // (`extreme_bg_color` ≈ rgb(10,10,10), `widgets.inactive.weak_bg_fill`
    // ≈ rgb(60,60,60)) which sits darker than the card behind and
    // disappears against it.  AppKit's `NSTextField` and `NSStepper`
    // are noticeably LIGHTER than the surrounding controlBackground in
    // dark mode — they read as raised input affordances.  Match that
    // by lifting both fills several steps in dark mode; light mode's
    // defaults are already correct.
    if matches!(theme, Theme::Dark) {
        // TextEdit background: rgb(58,58,60) ≈ AppKit's
        // `controlBackgroundColor` for input fields in dark appearance.
        v.extreme_bg_color = egui::Color32::from_rgb(58, 58, 60);

        // NSStepper chrome: lighter gray with a subtle outline.  These
        // widget-state colours flow through `paint_stepper_chrome`'s
        // `widgets.inactive` / `hovered` / `active` lookup.
        let stepper_rest    = egui::Color32::from_rgb(78, 78, 80);
        let stepper_hover   = egui::Color32::from_rgb(96, 96, 98);
        let stepper_press   = egui::Color32::from_rgb(120, 120, 122);
        let stepper_stroke  = egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 110, 112));
        v.widgets.inactive.weak_bg_fill = stepper_rest;
        v.widgets.inactive.bg_stroke    = stepper_stroke;
        v.widgets.hovered.weak_bg_fill  = stepper_hover;
        v.widgets.hovered.bg_stroke     = stepper_stroke;
        v.widgets.active.weak_bg_fill   = stepper_press;
        v.widgets.active.bg_stroke      = stepper_stroke;
    }

    v
}

/// Map winit's `Theme` onto egui's `ThemePreference` so the context can be
/// pinned to the exact OS appearance we just polled (bypassing egui's own
/// `System` auto-detect, which runs one frame behind).
fn theme_preference(theme: Theme) -> ThemePreference {
    match theme {
        Theme::Dark  => ThemePreference::Dark,
        Theme::Light => ThemePreference::Light,
    }
}

/// wgpu clear colour for the settings surface.
///
/// When `platform::is_blur_active()` is true, the OS is providing a blur
/// behind the window (macOS VEV child-window, Windows DWM acrylic, KDE
/// blur-behind region) — we clear at alpha 0 so wgpu doesn't paint
/// anything where egui hasn't drawn, letting the OS blur show through.
///
/// When blur isn't active (older Windows, GNOME, opt-out via
/// `EXHALE_DISABLE_BLUR=1`), the window is rendered opaquely — clear to
/// egui's panel fill so there's no flash between surface reconfiguration
/// and the first paint, and the cards sit on a solid theme-coloured
/// panel rather than a transparent void.
fn clear_color_for_theme(theme: Theme) -> wgpu::Color {
    if platform::is_blur_active() {
        wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    } else {
        match theme {
            Theme::Dark  => wgpu::Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 },
            Theme::Light => wgpu::Color { r: 0.96, g: 0.96, b: 0.96, a: 1.0 },
        }
    }
}

/// Measure every segmented picker in a single frame and return the largest
/// natural column width across them.  Buttons within a single picker get
/// equal width (so all options in that picker fit their widest text); the
/// column width is then max-of-natural-widths so that every picker in the
/// settings window shares the same left AND right bounds.
///
/// `SEGMENT_SLACK_PX` adds a small per-segment breathing room so the
/// measurement is always wide enough for the actual rendered text — the
/// `layout_no_wrap` measure and the on-render glyph layout can disagree by
/// a couple of pixels due to font hinting and sub-pixel positioning, which
/// was enough to let "Rectangle" wrap onto a second line inside a segment
/// that measured as "just big enough".
fn uniform_picker_column_width(ui: &egui::Ui, pickers: &[&[&str]]) -> f32 {
    const SEGMENT_SLACK_PX: f32 = 10.0;
    let pad_x   = ui.spacing().button_padding.x * 2.0;
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let measure = |s: &str| ui.fonts(|f|
        f.layout_no_wrap(s.to_string(), font_id.clone(), egui::Color32::WHITE).size().x
    );

    let mut max_col: f32 = 0.0;
    for opts in pickers {
        if opts.is_empty() { continue; }
        let max_text = opts.iter().map(|&s| measure(s)).fold(0.0_f32, f32::max);
        let btn_w    = (max_text + pad_x + SEGMENT_SLACK_PX).ceil();
        let col_w    = btn_w * opts.len() as f32;
        if col_w > max_col { max_col = col_w; }
    }
    max_col.ceil()
}

/// Two-cell row layout for non-picker rows: fixed-width label on the left,
/// DragValue / ColorPicker / etc. right-aligned against the row's trailing
/// edge.  Everything to the right of the label cell sits in a `right_to_left`
/// layout so the widget hugs the right edge exactly like Swift's Form.
/// Two-column row: a fixed-width label painted directly via the painter on
/// the left, and a `right_to_left` widget area on the right.
///
/// The painter-direct approach exists because `allocate_ui_with_layout` with
/// a fixed min_size collapses to the label's natural width inside a
/// horizontal layout — which left the remaining widget area wider than it
/// should be, and caused stepper TextEdits to draw over labels that were
/// still in their natural rect.  Reserving an exact-size rect and drawing
/// into it with the painter API guarantees the widget area to the right
/// starts at `LABEL_W + item_spacing`.
fn labeled_row(ui: &mut egui::Ui, label: &str, add_widget: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        paint_label(ui, label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_widget);
    }).response
}

/// Reserve a LABEL_W × ROW_H rect and paint `label` into it flush against the
/// rect's left edge with the current theme's text colour.  Using the painter
/// directly (rather than `ui.put(rect, Label::truncate())`) pins the text
/// exactly at `rect.left()` — `Label` was adding implicit horizontal padding
/// that read as "the labels aren't left-aligned" against Swift's reference.
fn paint_label(ui: &mut egui::Ui, label: &str) {
    paint_label_with_width(ui, label, LABEL_W);
}

/// Like `paint_label` but with a caller-specified column width.  Used by
/// `segmented_row` so the picker can extend leftward into the label column
/// when its natural width would otherwise overflow the card on the right.
fn paint_label_with_width(ui: &mut egui::Ui, label: &str, width: f32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width, ROW_H),
        egui::Sense::hover(),
    );
    let color = ui.visuals().text_color();
    let font  = egui::TextStyle::Body.resolve(ui.style());
    let mut job = egui::text::LayoutJob::simple_singleline(
        label.to_string(), font, color,
    );
    job.wrap = egui::text::TextWrapping {
        max_width:          rect.width(),
        max_rows:            1,
        break_anywhere:      true,
        overflow_character:  Some('…'),
    };
    let galley = ui.painter().layout_job(job);
    let text_pos = egui::pos2(
        rect.left(),
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, color);
}

/// Segmented picker row.  Label on the left; a right-aligned picker cell
/// of `column_w` wide on the right.  `column_w` is measured once per frame
/// (see `uniform_picker_column_width`) and passed identically to every
/// picker in the Appearance section, so the leftmost option button lands
/// on the same X coordinate regardless of the picker's option count or
/// text length.
fn segmented_row<T: Copy + PartialEq>(
    ui:       &mut egui::Ui,
    label:    &str,
    help:     &str,
    enabled:  bool,
    column_w: f32,
    current:  &mut T,
    options:  &[(&str, T)],
) -> bool {
    let mut changed = false;
    let response = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        // If the picker's natural column_w wouldn't fit at the standard
        // LABEL_W, shrink the label column so the picker can claim its
        // natural width — making every segmented picker share the same
        // left and right edges regardless of the widest option's text.
        // `MIN_LABEL_W` is the lower bound at which even short labels
        // ("Shape", "Gradient") stay readable after truncation.
        const MIN_LABEL_W: f32 = 70.0;
        const SAFETY_PX:   f32 = 2.0;
        let row_avail = ui.available_width();
        let label_w   = (row_avail - column_w - SAFETY_PX)
                         .clamp(MIN_LABEL_W, LABEL_W);
        paint_label_with_width(ui, label, label_w);
        // `SAFETY_PX` slack on the right — the outer rect stroke I paint
        // under the picker is 1 px centered-on-edge (so 0.5 px bleed outside),
        // and sub-pixel rounding of `per_w = (picker_w / n).floor()` plus the
        // last_w remainder can occasionally push the child min_rect by
        // another fraction of a pixel.
        let remaining = (ui.available_width() - SAFETY_PX).max(0.0);
        let picker_w  = column_w.min(remaining).max(1.0);
        let gap       = (remaining - picker_w).max(0.0);
        if gap > 0.0 { ui.add_space(gap); }

        ui.add_enabled_ui(enabled, |ui| {
            let n = options.len();
            // Sub-pixel remainder is absorbed by the last segment so the
            // rightmost edge lands exactly on picker_w.
            let per_w  = (picker_w / n as f32).floor().max(1.0);
            let last_w = per_w + (picker_w - per_w * n as f32).max(0.0);

            // Pre-compute the outer rect ourselves and use `ui.put(rect, btn)`
            // for each segment.  `ui.add_sized(size, btn)` does NOT actually
            // constrain the Button to `size` — Button's `allocate_at_least`
            // grows the frame to the natural text+padding width, which was
            // the real source of the Appearance-section right-overflow (debug
            // showed picker_w=156 requested but actual=167 delivered).
            // `ui.put` positions the widget into a fixed rect without
            // letting it grow the parent's min_rect.
            let outer_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(picker_w, ROW_H),
            );

            let dark_mode = ui.visuals().dark_mode;

            // Disable egui's default Button hover/press fills — we'll paint
            // a rounded inset pill ourselves for hover/press/selected so all
            // three states share the same macOS-native pill look.
            {
                let widgets = &mut ui.visuals_mut().widgets;
                widgets.inactive.bg_stroke    = egui::Stroke::NONE;
                widgets.hovered.bg_stroke     = egui::Stroke::NONE;
                widgets.active.bg_stroke      = egui::Stroke::NONE;
                widgets.hovered.expansion     = 0.0;
                widgets.active.expansion      = 0.0;
                widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                widgets.hovered.weak_bg_fill  = egui::Color32::TRANSPARENT;
                widgets.active.weak_bg_fill   = egui::Color32::TRANSPARENT;
            }
            ui.spacing_mut().button_padding = egui::vec2(0.0, 0.0);

            // macOS-native segmented picker selection: a slightly inset
            // rounded rect filled in gray (lighter than the picker container
            // in dark mode, near-white in light mode) — matches AppKit's
            // NSSegmentedControl `.selectedContentBackground`.
            let selected_fill = if dark_mode {
                // ~rgb(110,110,110) at 90% — reads as a clear lighter gray
                // over the dark vibrancy without going washed-out.
                egui::Color32::from_rgba_unmultiplied(110, 110, 110, 230)
            } else {
                // Near-white selection on a light translucent backdrop —
                // matches the macOS native picker "selected" pill.
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 235)
            };
            const SELECTED_INSET:    f32 = 2.0;
            const SELECTED_ROUNDING: f32 = 5.0;

            // Pre-compute every segment's rect so we can interact + paint
            // pill chrome BEFORE rendering each label (the pill must sit
            // under the text, not over it).
            let mut seg_rects: Vec<egui::Rect> = Vec::with_capacity(n);
            let mut seg_x = outer_rect.min.x;
            for i in 0..n {
                let w = if i + 1 == n { last_w } else { per_w };
                seg_rects.push(egui::Rect::from_min_size(
                    egui::pos2(seg_x, outer_rect.min.y),
                    egui::vec2(w, ROW_H),
                ));
                seg_x += w;
            }

            let font_id = egui::TextStyle::Button.resolve(ui.style());

            for (i, (text, variant)) in options.iter().enumerate() {
                let is_selected = *current == *variant;
                let seg_rect    = seg_rects[i];

                // Interact first — gives us hover/press/click without drawing.
                let seg_id = ui.id().with("seg").with(i).with(*text);
                let resp = ui.interact(seg_rect, seg_id, egui::Sense::click());

                // Pill chrome (selected > pressed > hovered) drawn under text.
                let pill_fill: Option<egui::Color32> = if is_selected {
                    Some(selected_fill)
                } else if resp.is_pointer_button_down_on() {
                    Some(if dark_mode {
                        egui::Color32::from_white_alpha(36)
                    } else {
                        egui::Color32::from_black_alpha(30)
                    })
                } else if resp.hovered() {
                    Some(if dark_mode {
                        egui::Color32::from_white_alpha(22)
                    } else {
                        egui::Color32::from_black_alpha(18)
                    })
                } else {
                    None
                };
                if let Some(fill) = pill_fill {
                    let pill = seg_rect.shrink(SELECTED_INSET);
                    ui.painter().rect_filled(pill, SELECTED_ROUNDING, fill);
                }

                // Selected text flips to primary; unselected uses default text color.
                let label_color = if is_selected {
                    if dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK }
                } else {
                    ui.visuals().text_color()
                };

                // Paint the label centered in the segment via the painter,
                // matching the segment width we measured for the picker
                // column — `ui.put(rect, Button)` would re-allocate and
                // grow `min_rect`, which we deliberately avoid in this row.
                let galley = ui.painter().layout_no_wrap(
                    text.to_string(),
                    font_id.clone(),
                    label_color,
                );
                let text_pos = egui::pos2(
                    seg_rect.center().x - galley.size().x * 0.5,
                    seg_rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, label_color);

                if resp.clicked() {
                    *current = *variant;
                    changed  = true;
                }
            }

            // Explicitly allocate the outer rect so the parent's cursor
            // advances past picker_w exactly — otherwise nothing has
            // reserved the horizontal space and the scope's min_rect
            // wouldn't include the pickers (ui.put doesn't advance cursor).
            let _ = ui.allocate_rect(outer_rect, egui::Sense::hover());

            // Single outline around the picker's outer bounds.
            let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
            ui.painter().rect_stroke(
                outer_rect,
                0.0,
                egui::Stroke::new(1.0, stroke_color),
            );
        });
    }).response;
    response.on_hover_text(help);
    changed
}

/// Duration row (seconds). Swift's CombinedStepperTextField with `limits: (0, nil)`
/// and step 1.0 — so the ±-button step matches the Stepper control on macOS.
fn duration_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut f64) -> bool {
    stepper_row(ui, label, help, None, value, 1.0, 0.0, None, ValueScale::Identity)
}

/// Randomised-timing percentage row.  Stored in Settings as 0.0–1.0; Swift
/// displays it multiplied by 100 with a stepper step of 1 % (== 0.01 in
/// storage).  `ValueScale::Percent` handles the ×100 / ÷100 conversion on
/// both read and write so the displayed/entered value is always a percent.
fn pct_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut f64) -> bool {
    stepper_row(ui, label, help, None, value, 1.0, 0.0, None, ValueScale::Percent)
}

/// How a stored value is mapped to the displayed/entered number.
///
/// Swift's CombinedStepperTextField is parameterised by a Binding that
/// transforms the stored value before it reaches the TextField.  We
/// accomplish the same thing with a scale enum so callers don't have to
/// open-code `*100` / `÷100` / `(x-1)*100` conversions everywhere, and the
/// stepper's `step` field still describes the *displayed* step (e.g. 1 %).
#[derive(Clone, Copy)]
enum ValueScale {
    /// `display = stored`
    Identity,
    /// `display = stored * 100` — randomised-timing sliders stored as fractions.
    Percent,
    /// `display = (stored - 1) * 100` — drift multiplier stored as e.g. 1.01.
    DriftPercent,
}

impl ValueScale {
    fn to_display(self, stored: f64) -> f64 {
        match self {
            Self::Identity     => stored,
            Self::Percent      => stored * 100.0,
            Self::DriftPercent => (stored - 1.0) * 100.0,
        }
    }
    fn from_display(self, display: f64) -> f64 {
        match self {
            Self::Identity     => display,
            Self::Percent      => display / 100.0,
            Self::DriftPercent => 1.0 + display / 100.0,
        }
    }
}

/// SwiftUI's `CombinedStepperTextField`: a fixed-width numeric TextField with
/// a two-button vertical Stepper to its right and an optional left-hand hint
/// ("0 = off").  `step`, `min`, and `max` are in the *displayed* unit; the
/// `scale` enum maps that display value to/from the stored `value`.
///
/// The buffer is persisted in egui's temp data keyed by `label` so typing a
/// partial number ("1." on the way to "1.25") doesn't get clobbered by a
/// redraw.  When the field loses focus (or the stepper nudges the value) we
/// canonicalise the buffer via `format_num` so extraneous zeros/decimals get
/// cleaned up — matching Swift's NumberFormatter with `maximumFractionDigits: 3`.
fn stepper_row(
    ui:     &mut egui::Ui,
    label:  &str,
    help:   &str,
    hint:   Option<&str>,
    value:  &mut f64,
    step:   f64,
    min:    f64,
    max:    Option<f64>,
    scale:  ValueScale,
) -> bool {
    let mut changed = false;
    let resp = ui.horizontal(|ui| {
        // Zero item_spacing.x at the row level; we'll insert explicit
        // `add_space` between components so the math for right-aligning is
        // exact.  (Prior code used `item_spacing.x = 2` but `widgets_w`
        // only accounted for ONE spacing even though two were actually
        // placed, so the column overflowed by 2-4 px on the right.)
        ui.spacing_mut().item_spacing.x = 0.0;

        // Label column — painter-direct, fixed LABEL_W wide.
        paint_label(ui, label);

        let stepper_btn_w = 14.0_f32;
        let field_gap:  f32 = 2.0;   // between field and ± buttons
        let hint_gap:   f32 = 4.0;   // between hint text and field
        let hint_w: f32 = if let Some(h) = hint {
            let font = egui::TextStyle::Small.resolve(ui.style());
            ui.fonts(|f| f.layout_no_wrap(h.to_string(), font, egui::Color32::WHITE).size().x).ceil()
        } else { 0.0 };
        // Exact total width of the trailing column: hint + hint_gap + field
        // + field_gap + stepper buttons.  Every component lines up with an
        // explicit add_space so this equals the actual placed width.
        let trailing_gap = if hint.is_some() { hint_gap } else { 0.0 };
        let widgets_w = hint_w + trailing_gap + STEPPER_FIELD_W + field_gap + stepper_btn_w;

        let remaining = ui.available_width();
        let gap = (remaining - widgets_w).max(0.0);
        if gap > 0.0 { ui.add_space(gap); }

        // Hint (left of field)
        if let Some(h) = hint {
            ui.label(egui::RichText::new(h).color(egui::Color32::GRAY).small());
            ui.add_space(hint_gap);
        }

        // Numeric text field
        let displayed = scale.to_display(*value);
        let max_disp  = max;
        let edit_id   = egui::Id::new(("stepper_buf", label));
        let focused   = ui.ctx().memory(|m| m.focused() == Some(edit_id));
        let mut buf: String = ui.data_mut(|d| {
            d.get_temp::<String>(edit_id).unwrap_or_else(|| format_num(displayed))
        });

        let field_resp = ui.add_sized(
            egui::vec2(STEPPER_FIELD_W, ROW_H),
            egui::TextEdit::singleline(&mut buf)
                .id(edit_id)
                .margin(egui::vec2(4.0, 2.0)),
        );
        if field_resp.changed() {
            if let Ok(parsed) = buf.trim().parse::<f64>() {
                let mut disp = parsed.max(min);
                if let Some(m) = max_disp { disp = disp.min(m); }
                let v = scale.from_display(disp);
                if (v - *value).abs() > f64::EPSILON {
                    *value  = v;
                    changed = true;
                }
            }
        }

        // Gap between field and ± buttons (now explicit since item_spacing = 0).
        ui.add_space(field_gap);

        // Stepper buttons (right of field) — pass the TextEdit's actual
        // rendered rect so the stepper's vertical bounds match the field's
        // visible bounds exactly (otherwise `ROW_H`-sized stepper overhangs
        // the TextEdit's slightly-shorter visible rectangle).
        let field_rect = field_resp.rect;
        let stepper_changed = stepper_buttons(
            ui,
            field_rect,
            label,  // row_salt — makes interact IDs unique per stepper row
            &scale.to_display(*value),
            step, min, max_disp,
        );
        if let Some(new_disp) = stepper_changed {
            let v = scale.from_display(new_disp);
            if (v - *value).abs() > f64::EPSILON {
                *value  = v;
                changed = true;
            }
        }

        // Canonicalise the buffer when the field isn't focused, or when the
        // stepper just nudged the value — prevents stale text hanging around
        // after external state changes (reset, cross-row effects).
        if !focused || stepper_changed.is_some() {
            buf = format_num(scale.to_display(*value));
        }
        ui.data_mut(|d| d.insert_temp(edit_id, buf));
    }).response;
    resp.on_hover_text(help);
    changed
}

/// Vertically stacked ▲/▼ Stepper buttons sized to match the adjacent
/// TextEdit's physical rect exactly.  `field_rect` is the TextEdit's
/// response rect — we use its `top()` and `bottom()` directly rather than
/// the parent UI's `ROW_H` so the stepper's top and bottom edges align with
/// the field's visible frame, never overhanging top or bottom.
///
/// Button widgets handle clicks and draw the chrome (fill + stroke +
/// hover/press states); triangles are drawn geometrically with the painter
/// because egui's default font (Ubuntu) doesn't include the ▲ U+25B2 /
/// ▼ U+25BC glyphs — they rendered as missing-glyph tofu boxes.
fn stepper_buttons(
    ui:         &mut egui::Ui,
    field_rect: egui::Rect,
    row_salt:   &str,
    value:      &f64,
    step:       f64,
    min:        f64,
    max:        Option<f64>,
) -> Option<f64> {
    let max_v = max.unwrap_or(f64::MAX);
    let btn_w: f32 = 13.0;
    let total_h = field_rect.height();

    // Reserve horizontal space WITHOUT creating a widget response at the
    // full rect — `allocate_exact_size(Sense::hover())` was registering an
    // interaction zone at the whole column that could absorb pointer
    // events ahead of the per-half `ui.interact` calls below, resulting
    // in clicks never registering for the stepper halves.  `allocate_space`
    // only advances the cursor; the actual hit-testing is done exclusively
    // by the two `ui.interact` calls, whose IDs are unique to each half.
    let (_, alloc_rect) = ui.allocate_space(egui::vec2(btn_w, total_h));
    let rect = egui::Rect::from_min_size(
        egui::pos2(alloc_rect.left(), field_rect.top()),
        egui::vec2(btn_w, total_h),
    );
    let half_h  = (total_h * 0.5).floor();
    let top_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(btn_w, half_h),
    );
    let bot_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top() + half_h),
        egui::vec2(btn_w, total_h - half_h),
    );

    // Hit-testing via `ui.interact` — this is the ONLY way to get pixel-
    // perfect sub_rects.  Debug logs proved `egui::Button` ignores
    // ui.put's max_rect and draws at its own desired_size (empty-text
    // galley line-height ≈ 15 px), overhanging the 9 px sub_rect by 6 px
    // below — exactly the "gray below the input" artifact.  With raw
    // interact + painter chrome, the rect we pass IS the rect drawn.
    // Scope the interact IDs by `row_salt` (the stepper_row's label) so
    // every stepper in the window has a unique ID pair.  Using `ui.id()`
    // alone gave every stepper the SAME id because egui 0.29's default
    // UiBuilder has no id_salt, so sibling `ui.horizontal()` children of
    // a given parent all share the parent's id.  That caused egui's
    // click-tracking to silently drop every click because it couldn't
    // disambiguate which stepper was hit.
    let row_id  = ui.id().with(row_salt);
    let up_resp = ui.interact(top_rect, row_id.with("stepper_up"), egui::Sense::click());
    let dn_resp = ui.interact(bot_rect, row_id.with("stepper_dn"), egui::Sense::click());
    #[cfg(test)]
    test_hooks::record_stepper_rects(top_rect, bot_rect);

    paint_stepper_chrome(ui, top_rect, StepperDir::Up,   up_resp.hovered(), up_resp.is_pointer_button_down_on());
    paint_stepper_chrome(ui, bot_rect, StepperDir::Down, dn_resp.hovered(), dn_resp.is_pointer_button_down_on());

    // Triangles as small filled polygons (font-independent).
    let tri_color = ui.visuals().text_color();
    paint_triangle(ui, top_rect, StepperDir::Up,   tri_color);
    paint_triangle(ui, bot_rect, StepperDir::Down, tri_color);

    let mut new_val = None;
    if up_resp.clicked() { new_val = Some((*value + step).clamp(min, max_v)); }
    if dn_resp.clicked() { new_val = Some((*value - step).clamp(min, max_v)); }
    new_val
}

fn paint_stepper_chrome(
    ui:      &egui::Ui,
    rect:    egui::Rect,
    dir:     StepperDir,
    hovered: bool,
    pressed: bool,
) {
    let v = ui.visuals();
    let style = if pressed {
        &v.widgets.active
    } else if hovered {
        &v.widgets.hovered
    } else {
        &v.widgets.inactive
    };
    // Round only the outer corners so the up + down halves merge into a
    // single rounded-rect column with a flush mid-edge.  A 3-px radius
    // matches the subtle macOS-native NSStepper look.
    const STEPPER_RADIUS: f32 = 3.0;
    let rounding = match dir {
        StepperDir::Up   => egui::Rounding { nw: STEPPER_RADIUS, ne: STEPPER_RADIUS, sw: 0.0, se: 0.0 },
        StepperDir::Down => egui::Rounding { nw: 0.0, ne: 0.0, sw: STEPPER_RADIUS, se: STEPPER_RADIUS },
    };
    ui.painter().rect(
        rect,
        rounding,
        style.weak_bg_fill,
        style.bg_stroke,
    );
}

#[derive(Copy, Clone)]
enum StepperDir { Up, Down }

fn paint_triangle(ui: &egui::Ui, rect: egui::Rect, dir: StepperDir, color: egui::Color32) {
    let c = rect.center();
    let half_w: f32 = 3.0;
    let half_h: f32 = 2.0;
    let points = match dir {
        StepperDir::Up => vec![
            egui::pos2(c.x - half_w, c.y + half_h),
            egui::pos2(c.x + half_w, c.y + half_h),
            egui::pos2(c.x,          c.y - half_h),
        ],
        StepperDir::Down => vec![
            egui::pos2(c.x - half_w, c.y - half_h),
            egui::pos2(c.x + half_w, c.y - half_h),
            egui::pos2(c.x,          c.y + half_h),
        ],
    };
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

/// Swift NumberFormatter equivalent: decimal with `maximumFractionDigits: 3`,
/// `usesGroupingSeparator = false`, trailing zeros stripped so `5.0` shows as
/// "5" and `25.50` shows as "25.5".
fn format_num(v: f64) -> String {
    if v.fract().abs() < 1e-9 {
        // Whole number path — avoid the "5.000" → "5" string thrash for the
        // common case where every setting starts as an integer default.
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{:.3}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

// ─── Color conversion ─────────────────────────────────────────────────────────
// Settings stores sRGB [f32;4] in 0..1 (not linear), matching SwiftUI's Color
// values (NSColor/CGColor in the deviceRGB space). The shader treats channel
// values as sRGB and writes them to an 8-bit UNORM framebuffer as-is, which
// the OS compositor displays as sRGB — identical to Swift's MTKView
// (`colorPixelFormat = .bgra8Unorm`) pipeline. Storing sRGB also makes
// gradient lerps interpolate in gamma space, matching SwiftUI's
// LinearGradient/RadialGradient default behaviour.

fn to_color32(c: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (c[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn from_color32(c: egui::Color32) -> [f32; 4] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
        c.a() as f32 / 255.0,
    ]
}

/// Like from_color32 but forces alpha=1.0 (for inhale/exhale colors).
fn from_color32_opaque(c: egui::Color32) -> [f32; 4] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
        1.0,
    ]
}

// ─── Test hooks ─────────────────────────────────────────────────────────
//
// A handful of test-only atomics and helpers so unit tests can observe
// where stepper_buttons actually placed its interact rects during the
// previous frame.  Used only under `#[cfg(test)]`.
#[cfg(test)]
mod test_hooks {
    use std::cell::RefCell;
    thread_local! {
        static LAST: RefCell<Option<(egui::Rect, egui::Rect)>> = RefCell::new(None);
    }

    pub fn record_stepper_rects(top: egui::Rect, bot: egui::Rect) {
        LAST.with(|c| *c.borrow_mut() = Some((top, bot)));
    }

    pub fn take_stepper_rects() -> Option<(egui::Rect, egui::Rect)> {
        LAST.with(|c| c.borrow_mut().take())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────
//
// These tests drive a minimal egui::Context directly (no winit, no wgpu),
// feeding synthetic pointer events to verify the stepper_row widgets
// respond to clicks without panicking.  That exercises the exact code
// path the user complained about: "click the up and down stepper buttons
// they do nothing".
#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, Pos2, Rect, RawInput, Vec2};

    /// Build a single RawInput frame with a pointer move to `pos` followed
    /// by a down+up primary click at that position.  egui requires BOTH
    /// the down and up within the same frame to register as a `clicked()`.
    fn click_input(pos: Pos2) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 800.0))),
            events: vec![
                Event::PointerMoved(pos),
                Event::PointerButton { pos, button: PointerButton::Primary, pressed: true,  modifiers: Default::default() },
                Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Default::default() },
            ],
            ..Default::default()
        }
    }

    /// Run a single frame of the stepper_row helper, returning the
    /// TextEdit's rect so tests can target the stepper rect relative to
    /// it.  `click` is fed as the RawInput for this frame.
    fn run_stepper_frame(
        ctx:     &Context,
        raw_in:  RawInput,
        value:   &mut f64,
        label:   &str,
    ) -> bool {
        let mut changed = false;
        ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Force the ui wide enough that stepper_row's layout is
                // the same as in the real app.
                ui.set_width(332.0);
                changed = stepper_row(
                    ui,
                    label,
                    "help",
                    None,
                    value,
                    1.0,
                    0.0,
                    None,
                    ValueScale::Identity,
                );
            });
        });
        changed
    }

    /// Locate the stepper's click rect by finding the TextEdit's response
    /// in the previous frame's widget list and offsetting by the known
    /// geometry: stepper is placed `field_gap=2` px right of the field and
    /// matches its vertical bounds.  For a test we fudge by targeting the
    /// right ~7 px of the row above the field's vertical midline (UP) or
    /// below (DOWN).  In practice the rect will be to the right of the
    /// field, centered on the field's row_y.
    fn stepper_click_positions(field_rect: Rect) -> (Pos2, Pos2) {
        let btn_w: f32 = 13.0;
        let field_gap: f32 = 2.0;
        // The stepper column starts at field_rect.right() + field_gap.
        let x = field_rect.right() + field_gap + btn_w * 0.5;
        let half_h = field_rect.height() * 0.5;
        let up_y = field_rect.top() + half_h * 0.5;
        let dn_y = field_rect.bottom() - half_h * 0.5;
        (Pos2::new(x, up_y), Pos2::new(x, dn_y))
    }

    /// Warm-up frame registers the stepper's interact rects in egui's
    /// memory AND records the exact top/bot rects via the `test_hooks`
    /// side channel.  Then the click frame targets the center of the
    /// recorded up/down half.
    fn simulate_click_on_stepper(ctx: &Context, value: &mut f64, click_up: bool) -> bool {
        // Frame A (warmup): no input, just run stepper_row to register
        // widgets and capture sub-rects.
        let mut value_probe = *value;
        let _ = ctx.run(RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 800.0))),
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.set_width(332.0);
                let _ = stepper_row(
                    ui, "Test", "help", None,
                    &mut value_probe,
                    1.0, 0.0, None,
                    ValueScale::Identity,
                );
            });
        });
        let (top_rect, bot_rect) = super::test_hooks::take_stepper_rects()
            .expect("stepper_buttons should have recorded its rects during warmup frame");

        // Frame B: click the center of the desired half.
        let target = if click_up { top_rect } else { bot_rect };
        let click_pos = target.center();
        run_stepper_frame(ctx, click_input(click_pos), value, "Test")
    }

    #[test]
    fn stepper_up_increments() {
        let ctx = Context::default();
        let mut value = 5.0_f64;
        // simulate_click_on_stepper runs a warmup frame (registers the
        // stepper's interact rects in egui memory) followed by a click
        // frame.  Exactly one click per invocation.
        let changed = simulate_click_on_stepper(&ctx, &mut value, true);
        assert!(changed, "UP click should change the value");
        assert_eq!(value, 6.0, "UP click should increment by step=1.0 (5 → 6)");
    }

    #[test]
    fn stepper_down_decrements() {
        let ctx = Context::default();
        let mut value = 5.0_f64;
        let changed = simulate_click_on_stepper(&ctx, &mut value, false);
        assert!(changed, "DOWN click should change the value");
        assert_eq!(value, 4.0, "DOWN click should decrement by step=1.0 (5 → 4)");
    }

    #[test]
    fn stepper_down_clamps_at_min() {
        let ctx = Context::default();
        let mut value = 0.0_f64;
        let _ = simulate_click_on_stepper(&ctx, &mut value, false);
        assert_eq!(value, 0.0, "DOWN click at min should not go below 0.0");
    }

    #[test]
    fn stepper_many_clicks_no_crash() {
        // Regression test for the user-reported "after a few clicks it
        // crashes": drive ~50 alternating UP/DOWN clicks and make sure
        // nothing panics and the value stays finite.
        let ctx = Context::default();
        let mut value = 10.0_f64;
        for i in 0..50 {
            let _ = simulate_click_on_stepper(&ctx, &mut value, i % 2 == 0);
        }
        assert!(value.is_finite(), "value should remain finite across 50 alternating clicks");
    }

    #[test]
    fn stepper_repeated_ups_accumulate() {
        // Each call to simulate_click_on_stepper performs ONE click,
        // so N invocations should give N increments (5 + 3 = 8).
        let ctx = Context::default();
        let mut value = 5.0_f64;
        for _ in 0..3 {
            let _ = simulate_click_on_stepper(&ctx, &mut value, true);
        }
        assert_eq!(value, 8.0, "three UP clicks should give 5 + 3*1 = 8");
    }
}
