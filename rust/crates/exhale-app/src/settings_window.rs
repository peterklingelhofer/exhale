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
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::Window,
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
}

// Fixed logical width of the settings window — wider than the Swift 246 pt
// reference because egui reserves a few pixels for a scrollbar track even
// when no bar is visible, and our right-aligned DragValues would otherwise
// clip at 260 pt.
const SETTINGS_WIDTH:      u32 = 300;
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

        // Apply a style close to the native macOS HUD look.
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = 10.0.into();
        visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(30, 30, 30, 245);
        egui_ctx.set_visuals(visuals);

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
        })
    }

    /// Forward a window event to egui.
    /// Returns (consumed, wants_repaint) — the caller uses `wants_repaint`
    /// to drive redraws instead of polling every idle tick.
    pub fn on_window_event(&mut self, event: &WindowEvent) -> (bool, bool) {
        let response = self.egui_state.on_window_event(&*self.window, event);
        if let WindowEvent::Resized(size) = event {
            self.resize(*size);
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
                        load:  wgpu::LoadOp::Clear(wgpu::Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 }),
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
                    true,
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
                    settings.shape != AnimationShape::Fullscreen,
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
                    true,
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
                    true,
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
                    true,
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

fn labeled_row(ui: &mut egui::Ui, label: &str, add_widget: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    ui.horizontal(|ui| {
        // Force the label to sit at the left edge of a fixed-width cell so
        // every row in the window aligns on the same column.  `add_sized`
        // centers by default — use a left-to-right layout instead.
        ui.allocate_ui_with_layout(
            egui::vec2(130.0, 20.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| { ui.label(label); },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_widget);
    }).response
}

/// Segmented picker row.  All segmented pickers in this window occupy the
/// full remaining column width (after the label cell) and divide it equally
/// between their options, so 2-option and 3-option rows line up on the same
/// left AND right bounds — matching Swift's `.pickerStyle(.segmented)`.
fn segmented_row<T: Copy + PartialEq>(
    ui:      &mut egui::Ui,
    label:   &str,
    help:    &str,
    enabled: bool,
    current: &mut T,
    options: &[(&str, T)],
) -> bool {
    let mut changed = false;
    labeled_row(ui, label, |ui| {
        ui.add_enabled_ui(enabled, |ui| {
            let total_w = ui.available_width();
            let n       = options.len() as f32;
            let spacing = ui.spacing().item_spacing.x;
            let per_w   = ((total_w - spacing * (n - 1.0)) / n).max(1.0);
            ui.allocate_ui_with_layout(
                egui::vec2(total_w, 20.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    for (text, variant) in options {
                        let is_selected = *current == *variant;
                        if ui.add_sized(
                            [per_w, 20.0],
                            egui::SelectableLabel::new(is_selected, *text),
                        ).clicked() {
                            *current = *variant;
                            changed  = true;
                        }
                    }
                },
            );
        });
    }).on_hover_text(help);
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
