mod theme;
mod widgets;

use std::sync::Arc;

use anyhow::{Context, Result};

use theme::{
    clear_color_for_theme, install_system_ui_font, theme_preference,
    visuals_for_theme,
};
// Star import is intentional: `settings_ui` references ~12 widget
// helpers and half a dozen layout constants, and the widget submodule
// is tightly coupled to this file by design.  Listing each one
// explicitly would add 20+ lines of boilerplate for no readability win
use widgets::*;
use egui::ViewportId;
use egui_wgpu::ScreenDescriptor;
use exhale_core::{
    settings::Settings,
    settings_manager::SettingsManager,
    types::{
        AnimationMode, AnimationShape, AppVisibility, ColorFillGradient, HoldRippleMode,
    },
};
use exhale_render::GpuContext;
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
    /// Isolated per-window device + queue (own command queue),
    /// minted from `GpuContext::new_render_device` so settings
    /// rendering doesn't serialize behind the overlay's GPU work
    /// on a single shared queue.  This is what removes the
    /// hover-storm-induces-overlay-lag bottleneck on Windows.
    device:                Arc<wgpu::Device>,
    queue:                 Arc<wgpu::Queue>,
    pending_reset:         bool,
    /// Fired when the user clicks the Quit button in the settings
    /// Controls row.  Wired up at construction so `SettingsWindow`
    /// can dispatch directly without `main.rs` having to poll a
    /// flag after every render.  The closure is `Send + Sync` so
    /// the settings window could be moved off the main thread
    /// later — though right now it stays on main for egui_winit.
    on_quit:               Box<dyn Fn() + Send + Sync + 'static>,
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
    /// Last value passed to `set_max_inner_size` so we can no-op when
    /// the natural content height hasn't changed.  Calling
    /// `set_max_inner_size` every frame translates on macOS to
    /// `NSWindow.setContentMaxSize:`, and AppKit re-enforces the
    /// constraint on every call — at egui's natural ~500 Hz event-
    /// driven repaint rate during a live bottom-edge drag, this
    /// fights the user's pointer and the window feels stuck.
    /// Caching the last value reduces the call count to "once per
    /// content-tree change" (≈ when settings actually change).
    last_max_height: Option<u32>,
}

/// Which control-button icon a lookup is for.  `usize` index into
/// [`IconCache::handles`] — keep the variants in the same order as
/// the array, accessors index by `kind as usize`
#[derive(Clone, Copy)]
enum IconKind {
    Play  = 0,
    Stop  = 1,
    Reset = 2,
    Quit  = 3,
}

const ICON_KIND_COUNT: usize = 4;

/// Holds the texture handles for each control-button icon × theme.
/// Loads both themes up-front (cheap: 8 × ~32×32 RGBA = ~32 KB) so
/// the theme toggle doesn't have to re-rasterise on first paint.
///
/// Storage is a flat 2-D array `[IconKind; 2]` (dark first, then
/// light) with a single indexed lookup.  The SF Symbol name table
/// lives in [`IconCache::load`] so adding a new icon is a one-line
/// enum variant plus one row in the table
struct IconCache {
    /// `handles[kind as usize][dark as usize]` — `dark=true` is
    /// index 1.  `None` slot for either non-macOS (where
    /// `render_sf_symbol` returns `None`) or rasterisation failure.
    handles: [[Option<egui::TextureHandle>; 2]; ICON_KIND_COUNT],
}

impl IconCache {
    fn load(ctx: &egui::Context) -> Self {
        // SF Symbol names in the order matching `IconKind`.
        // `power.circle.fill` reads as a power-off / quit affordance
        // in SF Symbols and visually pairs with the other
        // `circle.fill` icons in the row.  Non-mac platforms fall
        // back to the Unicode glyph in the call site (U+00D7
        // MULTIPLICATION SIGN, supported by every system UI font;
        // U+23FB POWER SYMBOL renders as a tofu box on Windows
        // because Segoe UI is missing it).
        const NAMES: [&str; ICON_KIND_COUNT] = [
            "play.circle.fill",
            "stop.circle.fill",
            "arrow.counterclockwise.circle.fill",
            "power.circle.fill",
        ];
        let mut handles: [[Option<egui::TextureHandle>; 2]; ICON_KIND_COUNT] =
            Default::default();
        for (i, name) in NAMES.iter().enumerate() {
            handles[i][0] = load_sf_icon(ctx, name, false); // light
            handles[i][1] = load_sf_icon(ctx, name, true);  // dark
        }
        Self { handles }
    }

    fn get(&self, kind: IconKind, dark: bool) -> Option<&egui::TextureHandle> {
        self.handles[kind as usize][dark as usize].as_ref()
    }

    fn play (&self, dark: bool) -> Option<&egui::TextureHandle> { self.get(IconKind::Play,  dark) }
    fn stop (&self, dark: bool) -> Option<&egui::TextureHandle> { self.get(IconKind::Stop,  dark) }
    fn reset(&self, dark: bool) -> Option<&egui::TextureHandle> { self.get(IconKind::Reset, dark) }
    fn quit (&self, dark: bool) -> Option<&egui::TextureHandle> { self.get(IconKind::Quit,  dark) }
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
pub(super) const SETTINGS_WIDTH: u32 = 360;
/// Lower bound when dragging the bottom edge.  100 pt lets the user
/// collapse the settings window down to just the titlebar + a sliver
/// of the Controls row (Start / Stop / Reset / Quit buttons) for a
/// "compact" mode — the ScrollArea handles everything below the
/// drag.  Matches Swift's "resize this far if you want" behaviour
/// while leaving a tighter floor than the previous 428 pt cap.
const SETTINGS_MIN_HEIGHT: u32 = 100;

impl SettingsWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        gpu:        Arc<GpuContext>,
        settings:   &exhale_core::settings::Settings,
        on_quit:    Box<dyn Fn() + Send + Sync + 'static>,
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
        // Sanity ceiling on the saved height: no real monitor is taller
        // than this in logical points, so anything above is corrupt
        // (typically from older builds that mistakenly persisted
        // PHYSICAL pixels and then re-multiplied them by the scale
        // factor every launch).  Falling back to the default lets a
        // corrupted settings file self-heal on the next save.
        const SETTINGS_MAX_LOGICAL_H: u32 = 4096;
        let saved_h = settings.settings_window_height
            .filter(|&h| h <= SETTINGS_MAX_LOGICAL_H);
        if settings.settings_window_height
            .map(|h| h > SETTINGS_MAX_LOGICAL_H)
            .unwrap_or(false)
        {
            log::warn!(
                "settings_window_height = {:?} pt is larger than {} — \
                 ignoring (likely corrupted by physical-vs-logical bug \
                 in older builds); using default",
                settings.settings_window_height,
                SETTINGS_MAX_LOGICAL_H,
            );
        }
        let initial_h = saved_h
            .unwrap_or(INITIAL_PREFERRED_H)
            .max(SETTINGS_MIN_HEIGHT);
        // Transparent settings window is macOS-only.
        //
        // On macOS we install an NSVisualEffectView child window behind
        // the settings NSWindow to get the AppKit vibrancy / blur
        // backdrop; the wgpu surface clears at alpha 0 and the egui
        // panel fill is `TRANSPARENT`, so the VEV shows through.
        //
        // On Windows + Linux we deliberately render the settings
        // window OPAQUE.  Previous attempts to wire up DWM acrylic on
        // Windows and KDE blur-behind on Linux introduced two visible
        // regressions:
        //   1. The overlay layer composited above the settings
        //      window's translucent client area in the z-stack,
        //      making the breath animation render in FRONT of the
        //      settings window (so opacity=1.0 trivially hid the
        //      controls — no way to edit them back).
        //   2. Mouse hover over the (DWM-composed alpha) settings
        //      window forced DWM to recomposite the whole acrylic
        //      stack per cursor move, producing very visible animation
        //      lag on Windows.
        // Both go away when the settings window is a plain opaque
        // surface — `clear_color_for_theme` paints the themed panel
        // colour directly and `panel_fill` is opaque too, so the
        // window is just a normal Windows / Linux app window.
        // macOS retains its vibrancy because Cocoa composes the VEV
        // child window outside the wgpu pipeline entirely.
        let want_transparent = cfg!(target_os = "macos");
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

        // Mint an isolated (Device, Queue) pair for this window so its
        // GPU submits run on a separate command queue from the overlay's.
        // On Windows/DX12 a shared device meant every hover-driven
        // settings repaint serialised the overlay's next present on the
        // same ID3D12CommandQueue, producing very visible breath-
        // animation lag.  Per-window devices remove that contention
        let (device, queue) = gpu.new_render_device()
            .context("settings per-window device")?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&gpu.adapter);
        // Prefer a non-sRGB format so wgpu doesn't gamma-encode the egui
        // output (mid-tone blends render brighter than intended under
        // sRGB).  Driver-bug guard: wgpu specifies `formats` as
        // non-empty, but a misbehaving driver could return an empty
        // list — fall back to a hard-coded `Bgra8Unorm` rather than
        // panic on `formats[0]`.
        let format = caps.formats.iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);

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
        surface.configure(&device, &config);

        // Install the NSVisualEffectView with a theme-appropriate material
        // so the Dark-mode vibrancy uses a neutral blend (underWindowBackground)
        // that doesn't lighten dark backdrops, while Light mode uses hudWindow
        // for a visibly translucent blur over bright desktops.
        let initial_theme = window.theme().unwrap_or(Theme::Dark);
        // RAII guard so the backdrop NSWindow is released even if some
        // future code between here and `Self { … }` adds a fallible
        // operation.  `install_settings_vibrancy` hands us a +1 retain
        // count as a raw `usize`; if we don't `take()` the guard into
        // `Self.vev_ptr`, the guard's `Drop` calls `uninstall_…` and
        // balances the retain.  Without this guard, a future `?` after
        // the vibrancy install would silently leak one NSWindow per
        // SettingsWindow creation failure.
        let vev_guard = BackdropGuard(platform::install_settings_vibrancy(
            &window, matches!(initial_theme, Theme::Dark),
        ));

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

        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

        // Pre-rasterise SF Symbol icons for both themes — cheap one-shot
        // cost (~6 small RGBA blobs uploaded as textures) so the theme
        // toggle doesn't have to lock-focus into AppKit on the hot path.
        let icon_cache = IconCache::load(&egui_ctx);

        // Take ownership of the backdrop pointer from the RAII guard.
        // If we reach this line, `Self` is being constructed and the
        // guard's drop will be skipped — `vev_ptr` lives on with the
        // window and is balanced by `Drop for SettingsWindow`.
        let vev_ptr = vev_guard.take();

        Ok(Self {
            window, surface, config, egui_ctx, egui_state, egui_renderer,
            device, queue,
            pending_reset: false,
            on_quit,
            theme,
            vev_ptr,
            icon_cache,
            last_max_height: None,
        })
    }

    /// Forward a window event to egui.
    /// Returns (consumed, wants_repaint) — the caller uses `wants_repaint`
    /// to drive redraws instead of polling every idle tick.
    pub fn on_window_event(&mut self, event: &WindowEvent) -> (bool, bool) {
        let response = self.egui_state.on_window_event(&self.window, event);
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
        self.surface.configure(&self.device, &self.config);
        // Keep the vibrancy backdrop NSWindow the same size as the
        // settings window — AppKit auto-tracks child-window position but
        // NOT size, so we copy the parent's frame onto the backdrop here.
        // No-op on non-macOS or when `EXHALE_DISABLE_BLUR` is set.
        platform::sync_settings_backdrop_frame(self.vev_ptr);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Raise the in-window Reset confirmation dialog on the next frame.
    /// Used by the Ctrl+Shift+F global hotkey on Windows / Linux only —
    /// macOS routes the same hotkey through a native `NSAlert.runModal()`
    /// in `do_reset_with_confirm` and never sets this flag.
    #[cfg(all(feature = "global-hotkeys", not(target_os = "macos")))]
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
                self.surface.configure(&self.device, &self.config);
                return Ok(std::time::Duration::MAX);
            }
            Err(e) => return Err(e).context("settings get_current_texture"),
        };

        let raw_input = self.egui_state.take_egui_input(&self.window);
        let pixels_per_point = self.window.scale_factor() as f32;

        let mut content_height: f32 = 0.0;
        // `full_output.platform_output.copied_text` is taken on
        // macOS only (clipboard hand-off); on other platforms the
        // binding is read-only, hence the cross-cfg `unused_mut`.
        #[allow(unused_mut)]
        let mut full_output = self.egui_ctx.run(raw_input, |ctx| {
            content_height = settings_ui(
                ctx, settings, settings_manager,
                &mut self.pending_reset,
                &*self.on_quit,
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

        // Cap the resizable window to the exact amount of content egui
        // just laid out.  `content_size.y` is the laid-out height of
        // the ScrollArea's inner content alone; the surrounding
        // CentralPanel adds `OUTER_PAD` of inner_margin on top AND
        // bottom, so the window's client area needs
        // `content + 2 * OUTER_PAD` to fit without scrolling.
        // Anything less than `2 * OUTER_PAD = 28.0` total leaves the
        // ScrollArea thinking it's under-tall and showing a scrollbar
        // even when every control fits, AND clamps the bottom-edge
        // resize handle short of fitting the content.
        //
        // Only forward the value to `set_max_inner_size` when the
        // computed max differs from what we last sent — calling
        // `setContentMaxSize:` on macOS is NOT a no-op on equal
        // input; AppKit re-enforces the constraint on every call,
        // and at egui's event-driven ~500 Hz repaint rate during a
        // bottom-edge live drag, the constant re-enforcement fights
        // the user's pointer and the window feels stuck.  Caching
        // reduces the call rate to "once per layout change" (≈ when
        // a setting changes or a section folds open/closed).
        if content_height > 0.0 {
            let natural_h = (content_height + 2.0 * OUTER_PAD)
                .ceil()
                .max(SETTINGS_MIN_HEIGHT as f32) as u32;
            if self.last_max_height != Some(natural_h) {
                self.window.set_max_inner_size(Some(
                    winit::dpi::LogicalSize::new(SETTINGS_WIDTH, natural_h),
                ));
                self.last_max_height = Some(natural_h);
            }
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
                &self.window,
                full_output.platform_output,
            );
        }

        let primitives = self.egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen_desc = ScreenDescriptor {
            size_in_pixels:  [self.config.width, self.config.height],
            pixels_per_point,
        };

        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("egui-frame") }
        );

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
        }
        self.egui_renderer.update_buffers(
            &self.device, &self.queue, &mut encoder, &primitives, &screen_desc,
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

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(repaint_delay)
    }
}

impl Drop for SettingsWindow {
    /// Release the macOS NSVisualEffectView backdrop NSWindow we
    /// installed via `platform::install_settings_vibrancy`.  Without
    /// this we leaked one retained NSWindow per settings-window
    /// lifecycle — visible in `leaks` reports and accumulating over
    /// open/close cycles.  No-op on Windows / Linux (those platforms'
    /// `install_settings_vibrancy` returns 0 and the `uninstall`
    /// matches with an early return).
    fn drop(&mut self) {
        platform::uninstall_settings_vibrancy(self.vev_ptr);
    }
}

/// RAII guard for a backdrop NSWindow pointer returned by
/// [`platform::install_settings_vibrancy`].  Releases the +1 retain
/// count via `uninstall_settings_vibrancy` if dropped without being
/// `take()`-en.  Used inside [`SettingsWindow::new`] to make
/// construction failure exception-safe — once `Self` is assembled,
/// the long-lived `Drop for SettingsWindow` impl takes over and this
/// guard is consumed.
struct BackdropGuard(usize);

impl BackdropGuard {
    /// Surrender ownership.  Caller is responsible for the eventual
    /// `uninstall_settings_vibrancy` call.
    fn take(mut self) -> usize {
        let ptr = self.0;
        self.0 = 0;       // Defuse so `Drop` no-ops.
        std::mem::forget(self); // Skip Drop entirely; no double-release.
        ptr
    }
}

impl Drop for BackdropGuard {
    fn drop(&mut self) {
        if self.0 != 0 {
            platform::uninstall_settings_vibrancy(self.0);
        }
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
    on_quit:          &dyn Fn(),
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
    // Panel fill: on macOS the gutters between/around the cards are
    // painted entirely by the NSVisualEffectView vibrancy (see
    // `platform::install_settings_vibrancy`).
    //   - In Dark mode, `.hudWindow` renders as a darkish blur,
    //     composites nicely with the translucent cards without any
    //     panel overlay.
    //   - In Light mode, `.hudWindow` is near-white with desktop
    //     colour tint coming through the blur.  A near-white panel
    //     overlay would composite right back into "solid white" and
    //     hide the vibrancy entirely, so we leave the panel fully
    //     transparent and let vibrancy be the sole gutter material.
    // Fully-transparent panel fill matches Swift's look exactly.
    //
    // The NSVisualEffectView's `.hudWindow` blur masks backdrop content
    // (terminal text, etc.) enough that nothing legible leaks through
    // the 14-px gutters.
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
                    let n_buttons = 4.0_f32;
                    let avail = ui.available_width();
                    let btn_w = ((avail - BUTTON_SPACING * (n_buttons - 1.0))
                                 / n_buttons)
                                .floor()
                                .max(1.0);

                    let dark = ui.visuals().dark_mode;
                    if control_button(
                        ui, btn_w,
                        "\u{25B6}", icons.play(dark),
                        None, 0.0,
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
                        // Segoe UI (Windows) draws U+25A0 BLACK SQUARE
                        // baseline-to-cap-height instead of em-centered
                        // — same family of issue as Quit's `×` but
                        // milder.  A 1 px lift brings the square back
                        // into visual alignment with the "Stop" label.
                        // This offset is only applied on the Unicode
                        // fallback path (Windows + Linux); macOS uses
                        // the SF Symbol texture which is em-centered
                        // by design and bypasses `painter.text` entirely.
                        None, -1.0,
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
                        // U+21BA ANTICLOCKWISE OPEN CIRCLE ARROW lives
                        // in the Arrows block, and Segoe UI draws it
                        // taller than the Geometric Shapes glyphs
                        // (`▶ ■`) — arrows traditionally reach into
                        // the ascender region.  Drop the font size to
                        // 12 pt so the rendered glyph is the same
                        // visible height as the other icons in the
                        // row.  Applies only on the Unicode fallback
                        // path (Windows + Linux); macOS uses the SF
                        // Symbol texture which is sized uniformly.
                        Some(12.0), 0.0,
                        "Reset",
                        "Reset all settings to their default values.",
                    ).clicked()
                    {
                        settings.reset_preserving_runtime_state();
                        dirty = true;
                    }
                    // Quit — full shutdown.  Dispatches directly via the
                    // injected `on_quit` callback (set up at
                    // `SettingsWindow::new`) so we don't need `main.rs`
                    // to poll a `pending_quit` flag after every render.
                    // Matches the tray-menu Quit path so all teardown
                    // (settings flush, controller stop, tray destroy)
                    // runs in the canonical order.
                    if control_button(
                        ui, btn_w,
                        // U+00D7 MULTIPLICATION SIGN — Latin-1 Supplement
                        // block, part of basic Western font coverage so
                        // every TTF / OTF system UI font carries it by
                        // default.  Previous attempts (U+23FB POWER
                        // SYMBOL, U+2715 HEAVY MULTIPLICATION X) lived
                        // in Misc Technical / Dingbats blocks, and
                        // Segoe UI's regular face on Windows skips
                        // those — egui's font-fallback chain then
                        // rendered the tofu / missing-glyph box.
                        // `×` reads unambiguously as "close / quit" at
                        // glyph-icon scale and stays monochrome so it
                        // pairs with the other Geometric-Shapes icons
                        // (▶ / ■ / ↺) in the row.
                        //
                        // `×` is intrinsically sized to lowercase
                        // x-height (Latin-1 lives alongside `é` `ñ`
                        // etc.), so at the default 14 pt it renders
                        // visibly shorter than the full-em-box
                        // Geometric Shapes glyphs.  Override to 20 pt
                        // here so the `×` lands at the same visible
                        // height as the other three icons.
                        "\u{00D7}", icons.quit(dark),
                        // `×` lives at the math-axis (below the
                        // em-center) instead of the em-center where
                        // Geometric Shapes glyphs sit, so even when
                        // both galleys are CENTER_CENTER-aligned at
                        // the same baseline the visible `×` lands a
                        // couple of pixels low.  Lift it ~2 px so the
                        // four icons read as a single horizontal row.
                        Some(20.0), -2.0,
                        "Quit",
                        "Quit exhale (full shutdown).",
                    ).clicked()
                    {
                        on_quit();
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
                        settings.reset_preserving_runtime_state();
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

// Swift's SectionCard: 10 px rounded rect, 1 px stroke at `Color.primary.opacity(0.06)`,
// fill at `Color(NSColor.controlBackgroundColor).opacity(0.55)`, 12 px internal padding.
// Header (when present) is 10 pt uppercase `.secondary` with 0.8 pt letter-spacing.

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
        // egui::Context::run returns `FullOutput`; we don't need it
        // here because the test only inspects the side effect on
        // `changed`.
        let _ = ctx.run(raw_in, |ctx| {
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
        let (top_rect, bot_rect) = super::widgets::test_hooks::take_stepper_rects()
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
