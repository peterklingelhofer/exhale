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
    /// True once we've clamped the window to the natural (fully-visible)
    /// content height.  We do this on the first frame after egui can
    /// actually measure the content — before that the max is unknown.
    natural_height_applied: bool,
    /// Tracks the OS appearance so egui visuals + the wgpu clear color stay
    /// in sync with Light/Dark mode.  `None` means the platform doesn't
    /// report a theme (some Linux desktops); we default to Dark there.
    theme: Theme,
}

// Fixed logical width of the settings window.  Wider than the Swift 246 pt
// reference so the segmented-picker column (right-aligned, uniform width
// across every row) has room for "Rectangle" / "Sinusoidal" without
// truncation while still leaving a visible gap between the left-aligned
// label column and the right-aligned picker column.
const SETTINGS_WIDTH:      u32 = 360;
/// User-imposed lower bound when dragging the bottom edge; matches the
/// request that "min height is maybe 400px".
const SETTINGS_MIN_HEIGHT: u32 = 400;

impl SettingsWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
        settings:   &exhale_core::settings::Settings,
    ) -> Result<Self> {
        // Width is fixed; only height is user-resizable.  Max height is set
        // later (once egui has measured the natural content size) so the
        // window can never extend past the last visible setting.
        let saved_h = settings.settings_window_height.unwrap_or(640);
        let initial_h = saved_h.max(SETTINGS_MIN_HEIGHT);
        let attrs = Window::default_attributes()
            .with_title("exhale")
            .with_inner_size(winit::dpi::LogicalSize::new(SETTINGS_WIDTH, initial_h))
            .with_min_inner_size(winit::dpi::LogicalSize::new(SETTINGS_WIDTH, SETTINGS_MIN_HEIGHT))
            .with_resizable(true)
            .with_transparent(false)
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

        let config = wgpu::SurfaceConfiguration {
            usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width:                         size.width.max(1),
            height:                        size.height.max(1),
            present_mode:                  wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode:                    wgpu::CompositeAlphaMode::Auto,
            view_formats:                  vec![],
        };
        surface.configure(&gpu.device, &config);

        let egui_ctx = egui::Context::default();

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
        let theme = window.theme().unwrap_or(Theme::Dark);
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

        Ok(Self {
            window, surface, config, egui_ctx, egui_state, egui_renderer, gpu,
            pending_reset: false,
            natural_height_applied: false,
            theme,
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
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            content_height = settings_ui(ctx, settings, settings_manager, &mut self.pending_reset);
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
            let natural_h = (content_height + 24.0).ceil().max(SETTINGS_MIN_HEIGHT as f32) as u32;
            self.window.set_max_inner_size(Some(
                winit::dpi::LogicalSize::new(SETTINGS_WIDTH, natural_h),
            ));
            // First successful measurement — shrink the window to the natural
            // height (or the user's saved override if smaller) so users don't
            // see a huge empty pane below the last control.
            if !self.natural_height_applied {
                self.natural_height_applied = true;
                let current_logical_h = (self.config.height as f32 / pixels_per_point).round() as u32;
                let target = settings.settings_window_height
                    .map(|h| h.clamp(SETTINGS_MIN_HEIGHT, natural_h))
                    .unwrap_or(natural_h)
                    .min(natural_h)
                    .max(SETTINGS_MIN_HEIGHT);
                if target != current_logical_h {
                    let _ = self.window.request_inner_size(
                        winit::dpi::LogicalSize::new(SETTINGS_WIDTH, target),
                    );
                }
            }
        }

        self.egui_state.handle_platform_output(
            &*self.window,
            full_output.platform_output,
        );

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
) -> f32 {
    let mut dirty = false;
    let mut content_height = 0.0f32;

    egui::CentralPanel::default().show(ctx, |ui| {
        let scroll_out = egui::ScrollArea::vertical().show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

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
            section(ui, "", |ui| {
                ui.horizontal(|ui| {
                    if ui.button("▶  Start").clicked() {
                        settings.is_animating = true;
                        settings.is_paused    = false;
                        dirty = true;
                    }
                    if ui.button("■  Stop").clicked() {
                        settings.is_animating = false;
                        settings.is_paused    = false;
                        dirty = true;
                    }
                    let pause_label = if settings.is_paused { "▶  Resume" } else { "⏸  Pause" };
                    if ui.add_enabled(settings.is_animating, egui::Button::new(pause_label)).clicked() {
                        settings.is_paused = !settings.is_paused;
                        dirty = true;
                    }
                    // In-window Reset button — direct reset (no confirm),
                    // matching Swift's SettingsView ControlButton.  The
                    // confirmation dialog is reserved for the Ctrl+Shift+F
                    // hotkey path (see request_reset_confirmation).
                    if ui.button("↺  Reset").clicked() {
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

            ui.add_space(4.0);

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

                // Overlay opacity
                labeled_row(ui, "Overlay Opacity (%)", |ui| {
                    let mut pct = settings.overlay_opacity * 100.0;
                    if ui.add(egui::DragValue::new(&mut pct)
                        .range(0.0..=100.0)
                        .speed(0.5)
                        .suffix("%"))
                        .changed()
                    {
                        settings.overlay_opacity = (pct / 100.0) as f32;
                        dirty = true;
                    }
                }).on_hover_text("Transparency of the overlay. Lower = more transparent.");

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

            ui.add_space(4.0);

            // ── Timing ───────────────────────────────────────────────────────
            section(ui, "Timing", |ui| {
                if duration_row(ui, "Inhale Duration (s)",  "Duration of the inhale phase, in seconds.",             &mut settings.inhale_duration) { dirty = true; }
                if duration_row(ui, "Post-Inhale Hold (s)", "Hold/pause duration at the end of inhale, in seconds.", &mut settings.post_inhale_hold_duration) { dirty = true; }
                if duration_row(ui, "Exhale Duration (s)",  "Duration of the exhale phase, in seconds.",             &mut settings.exhale_duration) { dirty = true; }
                if duration_row(ui, "Post-Exhale Hold (s)", "Hold/pause duration at the end of exhale, in seconds.", &mut settings.post_exhale_hold_duration) { dirty = true; }
            });

            ui.add_space(4.0);

            // ── Randomization ────────────────────────────────────────────────
            section(ui, "Randomization", |ui| {
                if pct_row(ui, "Inhale (%)",           "Randomize inhale duration by this percentage.",            &mut settings.randomized_timing_inhale) { dirty = true; }
                if pct_row(ui, "Post-Inhale Hold (%)", "Randomize post-inhale hold duration by this percentage.",  &mut settings.randomized_timing_post_inhale_hold) { dirty = true; }
                if pct_row(ui, "Exhale (%)",           "Randomize exhale duration by this percentage.",            &mut settings.randomized_timing_exhale) { dirty = true; }
                if pct_row(ui, "Post-Exhale Hold (%)", "Randomize post-exhale hold duration by this percentage.",  &mut settings.randomized_timing_post_exhale_hold) { dirty = true; }

                // Drift displayed as (drift - 1) * 100 %.  Swift's stepper has
                // no upper limit (`max: nil`) — match that so power users can
                // drive long cycles.  Minimum stays at 0 % (drift = 1.0) since
                // Swift's validator clamps negative input via `defaultMin = 0`.
                labeled_row(ui, "Drift (%)", |ui| {
                    let mut pct = (settings.drift - 1.0) * 100.0;
                    if ui.add(egui::DragValue::new(&mut pct)
                        .range(0.0..=f32::MAX)
                        .speed(0.1)
                        .suffix("%"))
                        .changed()
                    {
                        settings.drift = 1.0 + pct / 100.0;
                        dirty = true;
                    }
                }).on_hover_text("Multiplicative drift per cycle. 1-5% recommended for gradually lengthening breath.");
            });

            ui.add_space(4.0);

            // ── Timers ───────────────────────────────────────────────────────
            section(ui, "Timers", |ui| {
                labeled_row(ui, "Reminder (mins)", |ui| {
                    if ui.add(egui::DragValue::new(&mut settings.reminder_interval_minutes)
                        .range(0.0..=f64::MAX)
                        .speed(0.5))
                        .changed()
                    { dirty = true; }
                    ui.add(egui::Label::new(
                        egui::RichText::new("0 = off").small().weak()
                    ));
                }).on_hover_text("Notification reminder every N minutes. 0 to disable.");
                labeled_row(ui, "End After (mins)", |ui| {
                    if ui.add(egui::DragValue::new(&mut settings.auto_stop_minutes)
                        .range(0.0..=f64::MAX)
                        .speed(0.5))
                        .changed()
                    { dirty = true; }
                    ui.add(egui::Label::new(
                        egui::RichText::new("0 = off").small().weak()
                    ));
                }).on_hover_text("Auto-stop after N minutes. 0 to disable.");
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

fn section(ui: &mut egui::Ui, header: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            // Swift renders the top Controls section with no header — only the
            // labelled Appearance/Timing/etc sections get a header row.
            if !header.is_empty() {
                ui.label(egui::RichText::new(header.to_uppercase())
                    .small()
                    .color(egui::Color32::GRAY));
                ui.add_space(4.0);
            }
            add_contents(ui);
        });
    });
}

const LABEL_W:  f32 = 120.0;
const ROW_H:    f32 = 20.0;

/// Resolve the OS-appearance-aware egui visuals used by the settings window.
fn visuals_for_theme(theme: Theme) -> egui::Visuals {
    let mut v = match theme {
        Theme::Dark  => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    v.window_rounding = 10.0.into();
    // Light mode's default inactive widget stroke is near-invisible, so
    // segmented-picker segments bleed into the panel background.  Bump the
    // inactive stroke to a faint mid-gray so each segment's edge reads at
    // rest — matching the legibility we already get in dark mode and the
    // Swift `NSSegmentedControl` look.
    if matches!(theme, Theme::Light) {
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(180));
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

/// wgpu clear colour that matches egui's panel fill for the chosen theme, so
/// there is no visible flash behind the CentralPanel before egui paints.
fn clear_color_for_theme(theme: Theme) -> wgpu::Color {
    match theme {
        Theme::Dark  => wgpu::Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 },
        Theme::Light => wgpu::Color { r: 0.96, g: 0.96, b: 0.96, a: 1.0 },
    }
}

/// Measure every segmented picker in a single frame and return the largest
/// natural column width across them.  Buttons within a single picker get
/// equal width (so all options in that picker fit their widest text); the
/// column width is then max-of-natural-widths so that every picker in the
/// settings window shares the same left AND right bounds.
fn uniform_picker_column_width(ui: &egui::Ui, pickers: &[&[&str]]) -> f32 {
    let pad_x   = ui.spacing().button_padding.x * 2.0;
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let measure = |s: &str| ui.fonts(|f|
        f.layout_no_wrap(s.to_string(), font_id.clone(), egui::Color32::WHITE).size().x
    );

    // Segments sit flush with no inter-segment spacing, so the column only
    // needs to fit `btn_w * n` — the widest text in each picker, plus its
    // button padding, times the segment count.
    let mut max_col: f32 = 0.0;
    for opts in pickers {
        if opts.is_empty() { continue; }
        let max_text = opts.iter().map(|&s| measure(s)).fold(0.0_f32, f32::max);
        let btn_w    = (max_text + pad_x).ceil();
        let col_w    = btn_w * opts.len() as f32;
        if col_w > max_col { max_col = col_w; }
    }
    max_col.ceil()
}

/// Two-cell row layout for non-picker rows: fixed-width label on the left,
/// DragValue / ColorPicker / etc. right-aligned against the row's trailing
/// edge.  Everything to the right of the label cell sits in a `right_to_left`
/// layout so the widget hugs the right edge exactly like Swift's Form.
fn labeled_row(ui: &mut egui::Ui, label: &str, add_widget: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(LABEL_W, ROW_H),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| { ui.label(label); },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_widget);
    }).response
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
        ui.allocate_ui_with_layout(
            egui::vec2(LABEL_W, ROW_H),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| { ui.label(label); },
        );
        // Right-align the picker cell.  Clamp to whatever width is left if
        // the user has narrowed the window below the natural column size.
        let remaining = ui.available_width();
        let picker_w  = column_w.min(remaining).max(1.0);
        let gap       = (remaining - picker_w).max(0.0);
        if gap > 0.0 { ui.add_space(gap); }

        ui.add_enabled_ui(enabled, |ui| {
            let n = options.len();
            // Flush segments: buttons sit edge-to-edge so the whole picker
            // reads as one cohesive control, with a single outline around
            // the outer bounds (drawn below) marking the shared left/right
            // column edges.  Sub-pixel remainder gets absorbed into the
            // last segment so the rightmost edge lands exactly on picker_w.
            let per_w  = (picker_w / n as f32).floor().max(1.0);
            let last_w = per_w + (picker_w - per_w * n as f32).max(0.0);

            let outer = ui.allocate_ui_with_layout(
                egui::vec2(picker_w, ROW_H),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    // Zero inter-segment spacing + zero per-segment stroke
                    // and rounding: adjacent buttons meet flush so there
                    // are no interior dividers between segments.  The only
                    // border is the outer frame stroke painted after this
                    // closure returns.
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let dark_mode = ui.visuals().dark_mode;
                    // Subtle hover/press overlays so the user can see which
                    // segment will be selected on click.  Theme-aware: a
                    // faint white wash on dark panels, faint black on
                    // light panels — mirroring NSSegmentedControl's own
                    // rollover feedback.
                    let (hover_fill, press_fill) = if dark_mode {
                        (
                            egui::Color32::from_white_alpha(22),
                            egui::Color32::from_white_alpha(36),
                        )
                    } else {
                        (
                            egui::Color32::from_black_alpha(18),
                            egui::Color32::from_black_alpha(30),
                        )
                    };
                    let widgets = &mut ui.visuals_mut().widgets;
                    widgets.inactive.bg_stroke    = egui::Stroke::NONE;
                    widgets.hovered.bg_stroke     = egui::Stroke::NONE;
                    widgets.active.bg_stroke      = egui::Stroke::NONE;
                    widgets.inactive.rounding     = egui::Rounding::ZERO;
                    widgets.hovered.rounding      = egui::Rounding::ZERO;
                    widgets.active.rounding       = egui::Rounding::ZERO;
                    // Kill the default 1 px hover/press expansion so the
                    // hovered segment doesn't bulge past its neighbours.
                    widgets.hovered.expansion     = 0.0;
                    widgets.active.expansion      = 0.0;
                    // Transparent at rest so unselected segments blend
                    // into the panel — only hover / press add colour.
                    widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                    widgets.hovered.weak_bg_fill  = hover_fill;
                    widgets.active.weak_bg_fill   = press_fill;

                    for (i, (text, variant)) in options.iter().enumerate() {
                        let is_selected = *current == *variant;
                        let w = if i + 1 == n { last_w } else { per_w };
                        let v = ui.visuals();
                        let label = if is_selected {
                            egui::RichText::new(*text).color(v.selection.stroke.color)
                        } else {
                            egui::RichText::new(*text).color(v.text_color())
                        };
                        let mut btn = egui::Button::new(label).min_size(egui::vec2(w, ROW_H));
                        if is_selected { btn = btn.fill(v.selection.bg_fill); }
                        if ui.add_sized([w, ROW_H], btn).clicked() {
                            *current = *variant;
                            changed  = true;
                        }
                    }
                },
            );

            // Single outline around the outer bounds of the picker so the
            // shared left/right column edges are explicit — without
            // drawing dividers between individual segments.
            let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
            ui.painter().rect_stroke(
                outer.response.rect,
                0.0,
                egui::Stroke::new(1.0, stroke_color),
            );
        });
    }).response;
    response.on_hover_text(help);
    changed
}

/// Duration drag-value row. Returns true if value changed.
fn duration_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut f64) -> bool {
    let mut changed = false;
    labeled_row(ui, label, |ui| {
        changed = ui.add(
            egui::DragValue::new(value)
                .range(0.0..=f64::MAX)
                .speed(0.1)
                .suffix("s"),
        ).changed();
    }).on_hover_text(help);
    changed
}

/// Percentage drag-value for randomized timing (stored as 0.0–1.0). Returns true if changed.
/// Swift's CombinedStepperTextField passes `max: nil` here, so the upper bound
/// is unlimited — power users can dial in very large jitter.
fn pct_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut f64) -> bool {
    let mut changed = false;
    labeled_row(ui, label, |ui| {
        let mut pct = *value * 100.0;
        if ui.add(egui::DragValue::new(&mut pct)
            .range(0.0..=f64::MAX)
            .speed(0.5)
            .suffix("%"))
            .changed()
        {
            *value  = pct / 100.0;
            changed = true;
        }
    }).on_hover_text(help);
    changed
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
