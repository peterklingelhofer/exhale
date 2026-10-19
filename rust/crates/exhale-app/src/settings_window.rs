mod theme;
mod widgets;

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};

use theme::{
    clear_color_for_theme, install_system_ui_font, theme_preference,
    visuals_for_theme, PANEL_FILL_DARK, PANEL_FILL_LIGHT,
};
// Star import is intentional: `settings_ui` references ~12 widget
// helpers and half a dozen layout constants, and the widget submodule
// is tightly coupled to this file by design.  Listing each one
// explicitly would add 20+ lines of boilerplate for no readability win
use widgets::*;
use egui::ViewportId;
use egui_wgpu::ScreenDescriptor;
use exhale_core::{
    session_log::{export_dirs, SessionLog},
    settings::{KeyboardShortcut, Settings, ShortcutAction},
    settings_manager::SettingsManager,
    study::{StudyError, StudyStatus},
    types::{
        AnimationMode, AnimationShape, AppVisibility, ColorFillGradient, DisplayMode,
        HoldRippleMode,
    },
};
use crate::keymap::{egui_key_to_code, egui_modifiers_to_mask};
use exhale_render::GpuContext;
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Theme, Window},
};

use crate::platform;
#[cfg(feature = "global-hotkeys")]
use crate::hotkeys;

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
    /// hover-storm-induces-overlay-lag bottleneck on Windows
    device:                Arc<wgpu::Device>,
    queue:                 Arc<wgpu::Queue>,
    pending_reset:         bool,
    /// `Some(action)` when the user has chosen Change Shortcut from a
    /// right-click context menu and the next valid key combination
    /// should be captured and bound to `action`.  The capture overlay
    /// rendered in [`settings_ui`] watches `ctx.input` for any key
    /// pressed while this is `Some` and writes through to
    /// `settings.keyboard_shortcuts` on the first match
    capturing_shortcut_for: Option<ShortcutAction>,
    /// Fired when the user clicks the Quit button in the settings
    /// Controls row.  Wired up at construction so `SettingsWindow`
    /// can dispatch directly without `main.rs` having to poll a
    /// flag after every render.  The closure is `Send + Sync` so
    /// the settings window could be moved off the main thread
    /// later, though right now it stays on main for egui_winit
    on_quit:               Box<dyn Fn() + Send + Sync + 'static>,
    /// Fired right after the user captures a new keyboard shortcut
    /// (or resets one to its default).  The main loop receives an
    /// `AppEvent::RebindHotkeys` and reconciles the global-hotkey
    /// registrations with the updated `settings.keyboard_shortcuts`
    on_rebind_hotkeys:     Box<dyn Fn() + Send + Sync + 'static>,
    /// Tracks the OS appearance so egui visuals + the wgpu clear color stay
    /// in sync with Light/Dark mode.  Stays Dark when the platform doesn't
    /// report a theme (some Linux desktops)
    theme: Theme,
    /// Raw pointer to the NSVisualEffectView installed on macOS.  0 when
    /// not applicable (non-macOS, or EXHALE_DISABLE_VIBRANCY set).  When
    /// the theme changes we call `platform::update_settings_vibrancy`
    /// with this pointer so the blur material/appearance follows the
    /// system's Light/Dark toggle
    vev_ptr: usize,
    /// SF Symbol textures for the Start / Reset / Quit buttons in
    /// dark and light themes.  None on non-macOS or when the
    /// rasteriser fails: callers fall back to Unicode glyphs
    icon_cache: IconCache,
    /// Study builds only: the session log the Export and Clear buttons
    /// in the Study card act on.  `None` everywhere else, and the card
    /// then shows the participant id with no buttons
    session_log: Option<Arc<SessionLog>>,
    /// Last thing the Study card's buttons did, shown under them
    study_note: Option<StudyNote>,
    /// Last value passed to `set_max_inner_size` so we can no-op when
    /// the natural content height hasn't changed.  Calling
    /// `set_max_inner_size` every frame translates on macOS to
    /// `NSWindow.setContentMaxSize:`, and AppKit re-enforces the
    /// constraint on every call.  At egui's natural ~500 Hz event-
    /// driven repaint rate during a live bottom-edge drag, this
    /// fights the user's pointer and the window feels stuck.
    /// Caching the last value reduces the call count to "once per
    /// content-tree change" (≈ when settings change)
    last_max_height: Option<u32>,
}

/// Which control-button icon a lookup is for.  `usize` index into
/// [`IconCache::handles`]: keep the variants in the same order as
/// the array, [`IconCache::get`] indexes by `kind as usize`
#[derive(Clone, Copy)]
enum IconKind {
    Play  = 0,
    Reset = 1,
    Quit  = 2,
}

const ICON_KIND_COUNT: usize = 3;

/// Holds the texture handles for each control-button icon × theme.
/// Loads both themes up-front (cheap: 6 × ~32×32 RGBA = ~24 KB) so
/// the theme toggle doesn't have to re-rasterise on first paint
///
/// Storage is a flat 2-D array `[IconKind; 2]` (light first, then
/// dark) with a single indexed lookup.  The SF Symbol name table
/// lives in [`IconCache::load`] so adding a new icon is a one-line
/// enum variant plus one row in the table
struct IconCache {
    /// `handles[kind as usize][dark as usize]`: `dark=true` is
    /// index 1.  `None` slot for either non-macOS (where
    /// `render_sf_symbol` returns `None`) or rasterisation failure
    handles: [[Option<egui::TextureHandle>; 2]; ICON_KIND_COUNT],
}

impl IconCache {
    fn load(ctx: &egui::Context) -> Self {
        // SF Symbol names in the order matching `IconKind`.  We use
        // the `.circle.fill` variants (whole-icon designs with the
        // ring and inner glyph baked in by Apple) so the macOS
        // rendering matches Swift's `Image(systemName:)` output
        // pixel-for-pixel: Apple has already done the optical
        // centring of the inner glyph against the surrounding ring,
        // so we don't have to fiddle with sub-pixel offsets.
        // Non-mac platforms paint their own ring + Unicode glyph
        // (U+25B6, U+25A0, U+21BA, U+00D7) at the call site
        const NAMES: [&str; ICON_KIND_COUNT] = [
            "play.circle.fill",
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
}

// Window-placement helpers (clamp, apply, capture) live in
// `crate::placement`: shared between the settings window here and
// the windowed-mode animation window (`overlay::create_windowed_app`)
// so both use the exact same persisted-position logic and survive
// the same monitor-rearrangement edge cases

/// Rasterise an SF Symbol via AppKit, upload as an egui texture.
/// Rasterised at 13 pt, the ring diameter [`widgets::control_button`]
/// paints (`icon_w = 13.0`), so egui never has to downsample the
/// texture into its slot.  Returns `None` off-macOS or if the symbol
/// isn't found
fn load_sf_icon(ctx: &egui::Context, name: &str, dark_mode: bool) -> Option<egui::TextureHandle> {
    let (bytes, w, h) = platform::render_sf_symbol(name, 13.0, dark_mode)?;
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
// label column and the right-aligned picker column
pub(super) const SETTINGS_WIDTH: u32 = 360;
/// Lower bound when dragging the bottom edge.  100 pt lets the user
/// collapse the settings window down to the titlebar + a sliver
/// of the Controls row (Start / Stop / Reset / Quit buttons) for a
/// "compact" mode: the ScrollArea handles everything below the
/// drag.  Matches Swift's "resize this far if you want" behaviour
/// while leaving a tighter floor than the previous 428 pt cap
const SETTINGS_MIN_HEIGHT: u32 = 100;

impl SettingsWindow {
    pub fn new(
        event_loop:        &ActiveEventLoop,
        gpu:               Arc<GpuContext>,
        settings:          &exhale_core::settings::Settings,
        on_quit:           Box<dyn Fn() + Send + Sync + 'static>,
        on_rebind_hotkeys: Box<dyn Fn() + Send + Sync + 'static>,
        session_log:       Option<Arc<SessionLog>>,
    ) -> Result<Self> {
        // Width is fixed.  Only height is user-resizable.  Max height is set
        // later (once egui has measured the natural content size) so the
        // window can never extend past the last visible setting
        //
        // Default height shows Controls + Appearance + Timing whole, with
        // the top of Randomization visible so it's obvious there's more
        // below.  Saved height (when present) wins over the default.  The
        // user's own resize is always respected on relaunch
        //
        // 796 is unchanged, but the content it has to fit has grown.
        // Measured at the shipped 308 pt card width, the Timing card went 160 pt ->
        // 202 pt with the pacing readout -> 324 pt with the preset chips.
        // Preserving the old fold, which sat below Randomization, would
        // mean 960 pt: taller than the usable height of a 13-inch display,
        // so the window would open with its own bottom edge off-screen.
        // Between "Randomization stays above the fold" and "the window
        // fits on the machine it opens on", the second wins, and Timing is
        // now the card worth keeping whole anyway.  It's where the
        // presets, the steppers and the coverage line all live
        const INITIAL_PREFERRED_H: u32 = 796;
        // Sanity ceiling on the saved height: no real monitor is taller
        // than this in logical points, so anything above is corrupt
        // (typically from older builds that mistakenly persisted
        // physical pixels and then re-multiplied them by the scale
        // factor every launch).  Falling back to the default lets a
        // corrupted settings file self-heal on the next save
        const SETTINGS_MAX_LOGICAL_H: u32 = 4096;
        let saved_h = settings.settings_window_height
            .filter(|&h| h <= SETTINGS_MAX_LOGICAL_H);
        if settings.settings_window_height
            .map(|h| h > SETTINGS_MAX_LOGICAL_H)
            .unwrap_or(false)
        {
            log::warn!(
                "settings_window_height = {:?} pt is larger than {}, \
                 ignoring (likely corrupted by physical-vs-logical bug \
                 in older builds), using default",
                settings.settings_window_height,
                SETTINGS_MAX_LOGICAL_H,
            );
        }
        let initial_h = saved_h
            .unwrap_or(INITIAL_PREFERRED_H)
            .max(SETTINGS_MIN_HEIGHT);
        // Transparent settings window is macOS-only
        //
        // On macOS we install an NSVisualEffectView child window behind
        // the settings NSWindow to get the AppKit vibrancy / blur
        // backdrop.  The wgpu surface clears at alpha 0 and the egui
        // panel fill is `TRANSPARENT`, so the VEV shows through
        //
        // On Windows + Linux we render the settings
        // window opaque.  Previous attempts to wire up DWM acrylic on
        // Windows and KDE blur-behind on Linux introduced two visible
        // regressions:
        //   1. The overlay layer composited above the settings
        //      window's translucent client area in the z-stack,
        //      making the breath animation render in front of the
        //      settings window (so opacity=1.0 trivially hid the
        //      controls: no way to edit them back).
        //   2. Mouse hover over the (DWM-composed alpha) settings
        //      window forced DWM to recomposite the whole acrylic
        //      stack per cursor move, producing visible animation
        //      lag on Windows.
        // Both go away when the settings window is a plain opaque
        // surface: `clear_color_for_theme` paints the themed panel
        // colour directly and `panel_fill` is opaque too, so the
        // window is a normal Windows / Linux app window.
        // macOS retains its vibrancy because Cocoa composes the VEV
        // child window outside the wgpu pipeline entirely
        let want_transparent = cfg!(target_os = "macos");
        let attrs = Window::default_attributes()
            .with_title("exhale")
            .with_inner_size(winit::dpi::LogicalSize::new(SETTINGS_WIDTH, initial_h))
            .with_min_inner_size(winit::dpi::LogicalSize::new(SETTINGS_WIDTH, SETTINGS_MIN_HEIGHT))
            .with_resizable(true)
            .with_transparent(want_transparent)
            .with_decorations(true)
            .with_window_icon(crate::app_icon::window_icon());

        let window = Arc::new(event_loop.create_window(attrs)?);

        // Restore saved position via the shared placement helper.
        // Height was already set above via
        // `with_inner_size(LogicalSize::new(SETTINGS_WIDTH, initial_h))`
        // using the persisted logical-points value, so this only
        // needs to deal with the cross-platform position restore
        // (monitor-name lookup + clamp)
        crate::placement::apply_placement(
            event_loop,
            &window,
            &settings.settings_window_placement(),
        );

        platform::setup_settings_window(&window);

        let surface: wgpu::Surface<'static> =
            gpu.instance.create_surface(Arc::clone(&window))?;

        // Mint an isolated (Device, Queue) pair for this window so its
        // GPU submits run on a separate command queue from the overlay's.
        // On Windows/DX12 a shared device meant every hover-driven
        // settings repaint serialised the overlay's next present on the
        // same ID3D12CommandQueue, producing visible breath-
        // animation lag.  Per-window devices remove that contention
        let (device, queue) = gpu.new_render_device()
            .context("settings per-window device")?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&gpu.adapter);
        // Prefer a non-sRGB format so wgpu doesn't gamma-encode the egui
        // output (mid-tone blends render brighter than intended under
        // sRGB).  Driver-bug guard: wgpu specifies `formats` as
        // non-empty, but a misbehaving driver could return an empty
        // list: fall back to a hard-coded `Bgra8Unorm` instead of
        // panicking on `formats[0]`
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
        // Fall back to `Auto` (typically Opaque) if neither is available.
        // `clear_color_for_theme` handles that case by rendering the
        // window opaquely
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

        let initial_theme = window.theme().unwrap_or(Theme::Dark);

        let egui_ctx = egui::Context::default();

        // Swap in the OS-native UI font (SF Pro on macOS, Segoe UI on Windows,
        // Ubuntu/Cantarell/Noto on Linux) so text in our settings window
        // matches the rest of the OS's system preferences.  Silently falls
        // back to egui's default Ubuntu font if the platform's system font
        // isn't locatable: nothing critical breaks
        install_system_ui_font(&egui_ctx);

        // Pre-populate both style buckets with our custom visuals.  egui owns
        // separate dark_style / light_style slots and picks one per-frame
        // based on `ThemePreference` + `system_theme`.  Populating both up
        // front means whichever bucket egui selects already contains the
        // correct visuals: no rewrite-on-switch, no risk of writing to the
        // wrong bucket under a rapid toggle race
        egui_ctx.set_visuals_of(egui::Theme::Dark,  visuals_for_theme(Theme::Dark));
        egui_ctx.set_visuals_of(egui::Theme::Light, visuals_for_theme(Theme::Light));

        // Pin the theme preference explicitly (not `System`) so egui never
        // flips styles on our behalf based on a stale egui_winit `system_theme`.
        // We remain the single authoritative source, driven by the
        // render-time `window.theme()` poll below
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

        // Pre-rasterise SF Symbol icons for both themes: cheap one-shot
        // cost (6 small RGBA blobs uploaded as textures) so the theme
        // toggle doesn't have to lock-focus into AppKit on the hot path
        let icon_cache = IconCache::load(&egui_ctx);

        // Install the NSVisualEffectView with a theme-appropriate material
        // so the Dark-mode vibrancy uses a neutral blend (underWindowBackground)
        // that doesn't lighten dark backdrops, while Light mode uses hudWindow
        // for a visibly translucent blur over bright desktops.  Last on
        // purpose: the returned +1-retained backdrop goes straight into
        // `Self`, whose `Drop` releases it, with no fallible step between
        let vev_ptr = platform::install_settings_vibrancy(
            &window, matches!(initial_theme, Theme::Dark),
        );

        Ok(Self {
            window, surface, config, egui_ctx, egui_state, egui_renderer,
            device, queue,
            pending_reset: false,
            capturing_shortcut_for: None,
            on_quit,
            on_rebind_hotkeys,
            theme,
            vev_ptr,
            icon_cache,
            session_log,
            study_note: None,
            last_max_height: None,
        })
    }

    /// Forward a window event to egui.
    /// Returns (consumed, wants_repaint): the caller uses `wants_repaint`
    /// to drive redraws instead of polling every idle tick
    pub fn on_window_event(&mut self, event: &WindowEvent) -> (bool, bool) {
        let response = self.egui_state.on_window_event(&self.window, event);
        match event {
            WindowEvent::Resized(size) => self.resize(*size),
            // We intentionally don't use the event's Theme payload: during
            // rapid System Settings toggles the queue can hold an in-flight
            // event whose value is already stale by the time we dequeue it,
            // leaving the window inverted for one frame.  Instead,
            // schedule a redraw: the render-time poll of `window.theme()`
            // is the single authoritative source
            WindowEvent::ThemeChanged(_) => self.window.request_redraw(),
            _ => {}
        }
        (response.consumed, response.repaint)
    }

    /// Whether the window is on screen.  Wayland reports no visibility
    /// and winit can't hide a window there, so an unknown state means shown
    pub fn is_shown(&self) -> bool {
        self.window.is_visible().unwrap_or(true)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 { return; }
        self.config.width  = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        // Keep the vibrancy backdrop NSWindow the same size as the
        // settings window: AppKit auto-tracks child-window position but
        // not size, so we copy the parent's frame onto the backdrop here.
        // No-op on non-macOS or when `EXHALE_DISABLE_BLUR` is set
        platform::sync_settings_backdrop_frame(self.vev_ptr);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Whether the right-click -> Change Shortcut overlay is currently
    /// open and waiting for the user's next key combination.  The
    /// main-loop dispatcher checks this before forwarding any
    /// `GlobalHotKeyEvent`s so a previously-bound hotkey doesn't
    /// execute its action at the same time the capture overlay reads
    /// the keystroke as a new binding
    #[cfg(feature = "global-hotkeys")]
    pub fn is_capturing_shortcut(&self) -> bool {
        self.capturing_shortcut_for.is_some()
    }

    /// Externally arm shortcut-capture mode for `action`.  Used by
    /// the tray menu's "Keyboard Shortcuts ▶" submenu so the
    /// capture overlay shows the next time we render: same flow as
    /// the right-click -> Change Shortcut path inside the settings
    /// window, initiated from outside
    #[cfg(feature = "global-hotkeys")]
    pub fn begin_capturing(&mut self, action: ShortcutAction) {
        self.capturing_shortcut_for = Some(action);
    }

    /// Raise the in-window Reset confirmation card on the next frame.
    /// Used by the Reset global hotkey (default Ctrl+Shift+D) on
    /// every OS: macOS originally routed this through a native
    /// `NSAlert.runModal()` for the system look, but consolidated to
    /// the inline egui card so the confirmation chrome matches the
    /// button-click path the user already sees in the settings panel
    #[cfg(feature = "global-hotkeys")]
    pub fn request_reset_confirmation(&mut self) {
        self.pending_reset = true;
    }

    /// Render one egui frame onto the settings surface.
    /// Returns the `repaint_delay` egui requests for its next frame, used by
    /// the caller to schedule the next repaint via a deadline instead of
    /// polling every idle tick.  `Duration::MAX` means no scheduled repaint
    ///
    /// `overlay_available` is false when this session can only show a
    /// window, which locks the Display row to Window
    pub fn render(
        &mut self,
        settings:          &mut Settings,
        settings_manager:  &Arc<SettingsManager>,
        overlay_available: bool,
    ) -> Result<std::time::Duration> {
        // Reconcile against the authoritative OS theme every frame.
        // `WindowEvent::ThemeChanged` can coalesce or drop during rapid
        // System Settings toggles on macOS/Windows, which leaves the egui
        // visuals inverted from the real appearance.  Re-querying here
        // guarantees the window can never stay out of sync for more than
        // one frame regardless of how events were delivered
        //
        // We flip egui's `ThemePreference` (not `set_visuals`) because both
        // style buckets were pre-populated in `new()`.  `set_visuals` would
        // write into whichever bucket `ctx.theme()` currently resolves to.
        // And under a rapid toggle that can be the wrong bucket (egui_winit
        // may not have fed the latest `system_theme` yet), leaving the
        // wrong-colour visuals stuck in the bucket egui later selects
        if let Some(current) = self.window.theme() {
            if current != self.theme {
                self.theme = current;
                self.egui_ctx.set_theme(theme_preference(current));
                // Sync the NSVisualEffectView's material + appearance so
                // the vibrancy blur tint follows the Light/Dark toggle.
                // We do this ourselves because `install_settings_vibrancy`
                // pinned the VEV's appearance explicitly (to avoid the
                // AppKit auto-propagation crash): without this call the
                // blur would stay frozen at its install-time theme
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
            Err(wgpu::SurfaceError::Timeout) => {
                // `Timeout` is transient and needs no reconfigure.  Skip
                // this frame and retry on the next one
                log::debug!("settings get_current_texture: timeout, retrying next frame");
                return Ok(std::time::Duration::from_millis(16));
            }
            Err(e) => return Err(e).context("settings get_current_texture"),
        };

        let raw_input = self.egui_state.take_egui_input(&self.window);
        let pixels_per_point = self.window.scale_factor() as f32;

        let mut content_height: f32 = 0.0;
        // `full_output.platform_output.copied_text` is taken on
        // macOS only (clipboard hand-off).  On other platforms the
        // binding is read-only, hence the cross-cfg `unused_mut`
        #[allow(unused_mut)]
        let mut full_output = self.egui_ctx.run(raw_input, |ctx| {
            content_height = settings_ui(
                ctx, settings, settings_manager,
                &mut self.pending_reset,
                &mut self.capturing_shortcut_for,
                &*self.on_quit,
                &*self.on_rebind_hotkeys,
                &self.icon_cache,
                &self.session_log,
                &mut self.study_note,
                overlay_available,
            );
        });

        // egui populates a repaint_delay per viewport.  Respect it so we can
        // stop blindly repainting every idle tick.  Short delays keep tooltip
        // fade-ins and button-press animations working.  `Duration::MAX` means
        // nothing is animating and the window can sit idle until the next
        // user/external event
        let repaint_delay = full_output
            .viewport_output
            .get(&ViewportId::ROOT)
            .map(|v| v.repaint_delay)
            .unwrap_or(std::time::Duration::MAX);

        // Cap the resizable window to the exact amount of content egui
        // just laid out.  `content_size.y` is the laid-out height of
        // the ScrollArea's inner content alone.  The surrounding
        // CentralPanel adds `OUTER_PAD` of inner_margin on top and
        // bottom, so the window's client area needs
        // `content + 2 * OUTER_PAD` to fit without scrolling.
        // Anything less than `2 * OUTER_PAD = 28.0` total leaves the
        // ScrollArea thinking it's under-tall and showing a scrollbar
        // even when every control fits, and clamps the bottom-edge
        // resize handle short of fitting the content
        //
        // Only forward the value to `set_max_inner_size` when the
        // computed max differs from what we last sent, calling
        // `setContentMaxSize:` on macOS isn't a no-op on equal
        // input.  AppKit re-enforces the constraint on every call,
        // and at egui's event-driven ~500 Hz repaint rate during a
        // bottom-edge live drag, the constant re-enforcement fights
        // the user's pointer and the window feels stuck.  Caching
        // reduces the call rate to "once per layout change" (≈ when
        // a setting changes or a section folds open/closed)
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

        // On macOS: bypass `handle_platform_output` entirely and pipe only
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
        // container, which winit's cursor tracking doesn't expect)
        //
        // Neither hazard is reachable if we never let egui_winit hand the
        // platform output back to winit.  We do still need clipboard copy
        // (egui TextEdits write `copied_text` when the user presses ⌘C),
        // and `set_clipboard_text` is a pure arboard call that never
        // touches winit state: safe to invoke from here
        //
        // Cost on macOS: no cursor-icon changes (buttons / TextEdits keep
        // the default arrow), no IME cursor positioning (acceptable for a
        // Latin-text settings window), and no open_url handling (we don't
        // generate URL output anywhere).  All other platforms follow the
        // normal path
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
    /// lifecycle: visible in `leaks` reports and accumulating over
    /// open/close cycles.  No-op on Windows / Linux (those platforms'
    /// `install_settings_vibrancy` returns 0 and the `uninstall`
    /// matches with an early return)
    fn drop(&mut self) {
        platform::uninstall_settings_vibrancy(self.vev_ptr);
    }
}

// ─── Settings UI ─────────────────────────────────────────────────────────────
//
// Layout mirrors the Swift SettingsView:
//   • Controls: Start / Stop / Reset
//   • Appearance: colors, opacity, shape, gradient, animation, ripple, visibility
//   • Timing: 4 phase durations
//   • Randomization: 4 jitter sliders + drift
//   • Timers: reminder + auto-stop
//
// In a study the Appearance card keeps visibility alone, and the Study card
// stands in for the last three

/// Attach the right-click "Change Shortcut…" / "Reset to Default"
/// menu to a control-button `Response`.  Lives next to the buttons
/// instead of inside [`control_button`] because tooltip help text is
/// already passed in: the context-menu hook is independent of the
/// glyph rendering and easier to reason about as a separate concern
#[cfg(feature = "global-hotkeys")]
fn shortcut_context_menu(
    resp:                   &egui::Response,
    action:                 ShortcutAction,
    settings:               &mut Settings,
    capturing_shortcut_for: &mut Option<ShortcutAction>,
    dirty:                  &mut bool,
    rebind_hotkeys:         &mut bool,
) {
    // No binding can ever fire on a Wayland session, so don't offer a
    // menu that promises one
    if !hotkeys::shortcuts_available() {
        return;
    }
    resp.context_menu(|ui| {
        let current = settings.keyboard_shortcuts.get(action)
            .map(|sc| sc.display())
            .unwrap_or_else(|| "(none)".to_string());
        ui.label(format!("Current: {current}"));
        ui.separator();
        if ui.button("Change Shortcut…").clicked() {
            *capturing_shortcut_for = Some(action);
            ui.close_menu();
        }
        if ui.button("Reset Shortcut to Default").clicked() {
            settings.keyboard_shortcuts.reset_to_default(action);
            *dirty = true;
            *rebind_hotkeys = true;
            ui.close_menu();
        }
        if settings.keyboard_shortcuts.get(action).is_some()
            && ui.button("Unbind Shortcut").clicked()
        {
            settings.keyboard_shortcuts.set(action, None);
            *dirty = true;
            *rebind_hotkeys = true;
            ui.close_menu();
        }
    });
}

/// Shared one-line helper for the button tooltip text, embeds the
/// current binding when present, or "Right-click to set." when the
/// slot is unbound (the default for Start / Stop / Reset / Quit
/// after the "no opt-in defaults" simplification).  Starts with the
/// line break, so the version without the feature can return nothing
#[cfg(feature = "global-hotkeys")]
fn shortcut_tooltip_line(
    settings: &Settings,
    action:   ShortcutAction,
) -> String {
    if !hotkeys::shortcuts_available() {
        return String::new();
    }
    match settings.keyboard_shortcuts.get(action) {
        Some(sc) => format!("\nShortcut: {}\nRight-click to change.", sc.display()),
        None     => "\nShortcut: (none)\nRight-click to set.".to_string(),
    }
}

/// Without the feature there's no binding to show and no right-click
/// menu to point at, so the tooltip is the button's description alone
#[cfg(not(feature = "global-hotkeys"))]
fn shortcut_tooltip_line(
    _settings: &Settings,
    _action:   ShortcutAction,
) -> String {
    String::new()
}

// Segmented-picker options, one table per picker.  Each table feeds
// both its `segmented_row` and `uniform_picker_column_width`, so the
// measured column always matches the labels drawn

const DISPLAY_OPTIONS: &[(&str, DisplayMode)] = &[
    ("Overlay", DisplayMode::Overlay),
    ("Window",  DisplayMode::Window),
];
const SHAPE_OPTIONS: &[(&str, AnimationShape)] = &[
    ("Rectangle", AnimationShape::Rectangle),
    ("Circle",    AnimationShape::Circle),
    ("Full",      AnimationShape::Fullscreen),
];
/// Order matches Swift's enum declaration (Inner, Off, On) so
/// segmented-picker placement is identical to the macOS app
const GRADIENT_OPTIONS: &[(&str, ColorFillGradient)] = &[
    ("Inner", ColorFillGradient::Inner),
    ("Off",   ColorFillGradient::Off),
    ("On",    ColorFillGradient::On),
];
/// Labels use Swift's enum raw values
const ANIMATION_OPTIONS: &[(&str, AnimationMode)] = &[
    ("Linear",     AnimationMode::Linear),
    ("Sinusoidal", AnimationMode::Sinusoidal),
];
/// Order matches Swift's enum declaration (Gradient, Stark, Off) so the
/// default (Gradient) sits first
const RIPPLE_OPTIONS: &[(&str, HoldRippleMode)] = &[
    ("Gradient", HoldRippleMode::Gradient),
    ("Stark",    HoldRippleMode::Stark),
    ("Off",      HoldRippleMode::Off),
];
const VISIBILITY_OPTIONS: &[(&str, AppVisibility)] = &[
    ("Top Bar", AppVisibility::TopBarOnly),
    ("Dock",    AppVisibility::DockOnly),
    ("Both",    AppVisibility::Both),
];

/// The one column width every picker in the Appearance card shares,
/// measured over all of them.  Split out of [`settings_ui`] so a layout
/// test measures the same column the window does
fn picker_column_width(ui: &egui::Ui) -> f32 {
    uniform_picker_column_width(ui, &[
        option_labels(DISPLAY_OPTIONS),
        option_labels(SHAPE_OPTIONS),
        option_labels(GRADIENT_OPTIONS),
        option_labels(ANIMATION_OPTIONS),
        option_labels(RIPPLE_OPTIONS),
        option_labels(VISIBILITY_OPTIONS),
    ])
}

const DISPLAY_HELP: &str = "Overlay floats above your other windows and lets clicks \
                            through. Window is a regular window you can move, resize \
                            or make full screen.";
/// Shown in its place when the session can't show an overlay
const DISPLAY_LOCKED_HELP: &str = "This session always shows a window, because Wayland and \
                                   some graphics drivers can't show an overlay.";

/// The Display row: Overlay or Window
///
/// A session that can't show an overlay (Wayland, or a swap chain with
/// no per-pixel alpha) always shows a window.  The row is then drawn
/// disabled with Window selected, and nothing it does reaches
/// `settings`, so the stored choice stays whatever the user last made.
/// Disabling it with `add_enabled_ui` is safe because `segmented_row`
/// senses hover only in a disabled `Ui`, which keeps Tab traversal
/// intact (see the Background Color comment in `settings_ui`)
///
/// Returns true when the setting changed
fn display_row(
    ui:                &mut egui::Ui,
    settings:          &mut Settings,
    overlay_available: bool,
    column_w:          f32,
) -> bool {
    if overlay_available {
        return segmented_row(
            ui, "Display", DISPLAY_HELP, column_w,
            &mut settings.display_mode, DISPLAY_OPTIONS,
        );
    }
    // The picker needs something to write to, and it never will
    let mut shown = DisplayMode::Window;
    ui.add_enabled_ui(false, |ui| {
        segmented_row(
            ui, "Display", DISPLAY_LOCKED_HELP, column_w,
            &mut shown, DISPLAY_OPTIONS,
        );
    });
    false
}

const OPACITY_HELP: &str = "Transparency of the overlay. Lower = more transparent.";
/// Shown in its place while a window is showing
const OPACITY_WINDOW_HELP: &str =
    "A window draws at full strength. Overlay Opacity applies to the overlay.";

/// Whether the animation is a window right now: Window mode, or a session
/// that can't show an overlay and so shows one whatever is stored
fn window_showing(settings: &Settings, overlay_available: bool) -> bool {
    !overlay_available || settings.display_mode == DisplayMode::Window
}

/// The Overlay Opacity row
///
/// A window draws at full strength, so while one is showing the row is
/// drawn disabled and nothing it does reaches `settings`: the stored
/// opacity waits for the next time an overlay is showing.  It's disabled
/// the way the Display row is, with the same Tab-safe widget
///
/// Returns true when the setting changed
fn opacity_row(
    ui:             &mut egui::Ui,
    settings:       &mut Settings,
    window_showing: bool,
) -> bool {
    // Swift stores 0.0..1.0, displays 0..100 %.  Wrap with an f64 shim
    // because ValueScale::Percent operates on `*value / 100.0` and
    // settings.overlay_opacity is f32
    let mut opacity_pct = settings.overlay_opacity as f64;
    if window_showing {
        ui.add_enabled_ui(false, |ui| {
            stepper_row(
                ui, "Overlay Opacity (%)", OPACITY_WINDOW_HELP,
                None, &mut opacity_pct, 1.0, 0.0, Some(100.0),
                ValueScale::Percent,
            );
        });
        return false;
    }
    if stepper_row(
        ui, "Overlay Opacity (%)", OPACITY_HELP,
        None, &mut opacity_pct, 1.0, 0.0, Some(100.0),
        ValueScale::Percent,
    ) {
        settings.overlay_opacity = opacity_pct as f32;
        return true;
    }
    false
}

/// Gap between the Controls-row buttons, and between the two reset
/// confirmation buttons under them
const CONTROL_GAP: f32 = 8.0;

/// Width of one of four equal slots across `avail` with `CONTROL_GAP`
/// between them.  The Controls row and the reset confirmation both use
/// it, so each confirmation button lines up with a button above it
fn control_slot_width(avail: f32) -> f32 {
    const SLOTS: f32 = 4.0;
    ((avail - CONTROL_GAP * (SLOTS - 1.0)) / SLOTS).floor().max(1.0)
}

/// The color the panel says something has gone wrong in
///
/// One definition for the reset confirmation and the study-file
/// warning, so the two can't drift apart. Pastel red on dark has enough
/// contrast to read as a warning without screaming, and the light
/// variant is dark enough to stay legible on the near-white card
fn warning_color(ui: &egui::Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(255, 130, 130)
    } else {
        egui::Color32::from_rgb(180, 0, 0)
    }
}

/// What Reset says it does, in the button's tooltip and in its
/// confirmation
struct ResetText {
    help:   &'static str,
    title:  &'static str,
    detail: &'static str,
}

/// What Reset has always said, true wherever no study lock is in force
const RESET_TEXT: ResetText = ResetText {
    help:   "Reset all settings to their default values.",
    title:  "Reset all settings to their defaults?",
    detail: "This restores every appearance, timing, and keyboard-shortcut \
             setting. Can't be undone.",
};

/// In a study the file locks the pace and the appearance, and Reset puts
/// them back to the file's values, so all it restores is what the file
/// leaves to the participant
const RESET_TEXT_STUDY: ResetText = ResetText {
    help:   "Reset the keyboard shortcuts and Show In to their defaults. \
             The study locks everything else.",
    title:  "Reset the keyboard shortcuts and Show In?",
    detail: "This restores the keyboard shortcuts and Show In. The study \
             locks the pace and the appearance. Can't be undone.",
};

/// The words for a Reset, true for the settings it will find
///
/// `study` is true when `settings.study` is set, which is when Reset keeps
/// the lock
fn reset_text(study: bool) -> &'static ResetText {
    if study { &RESET_TEXT_STUDY } else { &RESET_TEXT }
}

/// Said under the Export button when new lines have arrived since the
/// last export, which is when Clear isn't offered
///
/// A running animation writes a line at every phase, so an export taken
/// while it runs is already behind by the time the card redraws. The hint
/// says Stop first, which is what makes the next export the last one
const CLEAR_WITHHELD_HINT: &str =
    "New lines arrived since the export. Stop, then export again to clear the log.";

/// Start's tooltip while the study can't run
const STUDY_START_HELP: &str = "The study can't start. Tell whoever runs the study.";

/// The last thing the Study card's buttons did, shown under them
#[derive(Clone, Debug, PartialEq)]
enum StudyNote {
    /// The log was copied to this path
    Exported(PathBuf),
    /// The live log is empty again
    Cleared,
    /// An export or a clear failed, with the sentence that says why
    Failed(String),
}

/// The Study card of a study that can't run
///
/// `summary` names the file, and the line and the field when it knows
/// them. It never holds a configured value, so a participant who reads
/// it learns nothing about the pace. The full detail is in the debug log
fn study_error_card(ui: &mut egui::Ui, err: &StudyError) {
    section(ui, "Study", |ui| {
        let warning = warning_color(ui);
        ui.label(
            egui::RichText::new(format!("Study can't start: {}", err.summary()))
                .color(warning),
        );
        ui.label(egui::RichText::new("Tell whoever runs the study.").color(warning));
    });
}

/// The Appearance card
///
/// A study file locks every row here but Show In, so while the pace is
/// kept from the participant (any status but `Off`, as for the pacing
/// cards) the card shows that row alone. The locked rows are hidden and
/// not disabled: a control the file overrides would only invite presses
/// that do nothing, and a disabled widget breaks Tab for the controls after
/// it (see the Background Color comment below)
///
/// Split out of [`settings_ui`] so a test can lay the card out on its own
///
/// Returns true when something in it changed the settings
fn appearance_card(
    ui:                &mut egui::Ui,
    settings:          &mut Settings,
    status:            &StudyStatus,
    overlay_available: bool,
    picker_column_w:   f32,
) -> bool {
    let mut dirty = false;
    section(ui, "Appearance", |ui| {
        if !crate::hides_pacing(status) {
            // Inhale color: no alpha (Swift: supportsOpacity: false)
            if color_row(
                ui, "Inhale Color", "Choose the color for the inhale phase.",
                &mut settings.inhale_color, egui::color_picker::Alpha::Opaque,
            ) { dirty = true; }

            // Exhale color: no alpha (Swift: supportsOpacity: false)
            if color_row(
                ui, "Exhale Color", "Choose the color for the exhale phase.",
                &mut settings.exhale_color, egui::color_picker::Alpha::Opaque,
            ) { dirty = true; }

            // Background color (with alpha).  Only visually meaningful
            // when `shape != Fullscreen`, but we render the picker
            // enabled regardless.  Wrapping it in
            // `add_enabled_ui(false, ...)` made egui call
            // `surrender_focus` on the disabled widget every
            // time Tab landed there.  Focus was lost mid-cycle
            // and the next Tab wrapped back to the first
            // focusable widget (Start button), so users with
            // `shape = Fullscreen` saw Tab go button-button-
            // button-button-Inhale-Exhale-Background-Start
            // instead of continuing through Overlay Opacity
            // and the rest of the panel.  The tooltip tells the
            // user when the setting has no effect
            if color_row(
                ui, "Background Color",
                "Choose the background color. No effect when Shape is Fullscreen.",
                &mut settings.background_color, egui::color_picker::Alpha::OnlyBlend,
            ) { dirty = true; }

            // Overlay opacity: disabled while a window is showing
            let window = window_showing(settings, overlay_available);
            if opacity_row(ui, settings, window) {
                dirty = true;
            }

            if display_row(ui, settings, overlay_available, picker_column_w) {
                dirty = true;
            }

            // Shape
            if segmented_row(
                ui, "Shape",
                "Shape of the animation: Fullscreen, Rectangle, or Circle.",
                picker_column_w,
                &mut settings.shape,
                SHAPE_OPTIONS,
            ) { dirty = true; }

            if segmented_row(
                ui, "Gradient",
                "Gradient color effect. No effect when Shape is Fullscreen.",
                picker_column_w,
                &mut settings.color_fill_gradient,
                GRADIENT_OPTIONS,
            ) { dirty = true; }

            if segmented_row(
                ui, "Animation",
                "Sinusoidal eases in/out naturally. Linear is constant speed.",
                picker_column_w,
                &mut settings.animation_mode,
                ANIMATION_OPTIONS,
            ) { dirty = true; }

            if segmented_row(
                ui, "Hold Ripple",
                "Hold phase ripple: Gradient (smooth glow), Stark (solid edge), or Off.",
                picker_column_w,
                &mut settings.hold_ripple_mode,
                RIPPLE_OPTIONS,
            ) { dirty = true; }
        }

        // App visibility (macOS concept, show on all platforms for settings parity)
        if segmented_row(
            ui, "Show In",
            "Where exhale appears: Top Bar, Dock, or Both.",
            picker_column_w,
            &mut settings.app_visibility,
            VISIBILITY_OPTIONS,
        ) { dirty = true; }
    });
    dirty
}

/// The cards under Appearance: Timing, Randomization and Timers when no
/// study is involved, the Study card when a study file locked the
/// pacing, and nothing for a study that can't run
///
/// Any status but `Off` shows none of the three pacing cards, and no
/// preset chips and no pacing readout with them: a participant who can
/// read the rate, or change it, isn't blinded any more. The cards are
/// hidden because a read-only stepper showing 5 and 5 names the rate as
/// well as an editable one does. A study that can't run hides them too,
/// since the pace it would show is whatever `settings.toml` held. Its
/// error card sits at the top of the window
///
/// Split out of [`settings_ui`] so a test can prove that without
/// standing up a `SettingsManager` and its writer thread
///
/// Returns true when something in it changed the settings
fn body_cards(
    ui:          &mut egui::Ui,
    settings:    &mut Settings,
    status:      &StudyStatus,
    session_log: &Option<Arc<SessionLog>>,
    note:        &mut Option<StudyNote>,
) -> bool {
    if !crate::hides_pacing(status) {
        let mut dirty = false;
        // ── Timing ─────────────────────────────────────────────────────
        section(ui, "Timing", |ui| {
            // Above the steppers: the whole point of a
            // chip is that clicking it visibly moves all four rows
            // under it, which isn't something the user has to be
            // told if they can watch it happen
            if preset_chips(ui, settings) { dirty = true; }
            if duration_row(ui, "Inhale Duration (s)",  "Duration of the inhale phase, in seconds.",             &mut settings.inhale_duration) { dirty = true; }
            if duration_row(ui, "Post-Inhale Hold (s)", "Hold/pause duration at the end of inhale, in seconds.", &mut settings.post_inhale_hold_duration) { dirty = true; }
            if duration_row(ui, "Exhale Duration (s)",  "Duration of the exhale phase, in seconds.",             &mut settings.exhale_duration) { dirty = true; }
            if duration_row(ui, "Post-Exhale Hold (s)", "Hold/pause duration at the end of exhale, in seconds.", &mut settings.post_exhale_hold_duration) { dirty = true; }
            // Computed from the four rows above, every frame.  A
            // rate stored anywhere else would be wrong the moment
            // one stepper moves, and wrong from the first cycle
            // whenever Drift is on.  Drift lives in the
            // Randomization card below, so its projection is
            // reported here where the cycle it lengthens is
            // defined, instead of duplicated under the stepper
            pacing_readout(ui, settings);
        });

        // ── Randomization ──────────────────────────────────────────────
        section(ui, "Randomization", |ui| {
            if pct_row(ui, "Inhale (%)",           "Randomize inhale duration by this percentage.",            &mut settings.randomized_timing_inhale) { dirty = true; }
            if pct_row(ui, "Post-Inhale Hold (%)", "Randomize post-inhale hold duration by this percentage.",  &mut settings.randomized_timing_post_inhale_hold) { dirty = true; }
            if pct_row(ui, "Exhale (%)",           "Randomize exhale duration by this percentage.",            &mut settings.randomized_timing_exhale) { dirty = true; }
            if pct_row(ui, "Post-Exhale Hold (%)", "Randomize post-exhale hold duration by this percentage.",  &mut settings.randomized_timing_post_exhale_hold) { dirty = true; }

            // Drift: stored as a per-cycle multiplier (1.01 = +1 %), displayed
            // as a percentage above 1.0 so "1" reads as "+1 % per cycle".
            // Swift's stepper has no upper limit (`max: nil`).  Minimum
            // stays at 0 % (drift = 1.0) per the `defaultMin` clamp
            //
            // Step is 0.1 percentage points: compounding makes whole
            // percents enormous. 1 % doubles the breath in 70 breaths, which
            // runs away inside one sitting.  0.1 % takes 693, about 2.8 hours
            // from the 10 s default. Counted in breaths instead of minutes because
            // `d^k = 2` has no cycle length in it, so the figure holds
            // whatever the four steppers above are set to. The useful range
            // sat entirely below the old minimum step, so whole percents
            // offered no gentle setting at all. Finer values can still be
            // typed: `format_num` caps display at three decimals, so 0.001 %
            // is the finest value the field round-trips
            //
            // Display is `(stored - 1) * 100`, so the user reads 0 for "off"
            // and never sees the multiplier. One step up from off is 0.1 %,
            // stored as 1.001. See `ValueScale::DriftPercent`
            if stepper_row(
                ui, "Drift (%)",
                "Lengthens every cycle by this much, compounding, for pranayama-style graded extension. Off by default. Type a value for finer control than the arrows give.",
                None, &mut settings.drift, 0.1, 0.0, None,
                ValueScale::DriftPercent,
            ) { dirty = true; }
        });

        // ── Timers ─────────────────────────────────────────────────────
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
        return dirty;
    }

    // A study that can't run shows its error card at the top of the
    // window and nothing here
    if *status != StudyStatus::Locked {
        return false;
    }
    // Cloned so the card can read the lock while it also reads the rest of
    // `settings` for the log header: two short strings and a few floats,
    // on a window that repaints on input
    let Some(study) = settings.study.clone() else {
        return false;
    };

    // ── Study ───────────────────────────────────────────────────────
    section(ui, "Study", |ui| {
        study_line(ui, "Participant", &study.participant_id);

        let Some(log) = session_log else { return; };

        ui.add_space(ROW_GAP);
        // Set once the Export press, if there is one, has landed
        let mut clear_offered = false;
        ui.horizontal(|ui| {
            if ui.button("Export log").clicked() {
                *note = Some(match log.export_to(&export_dirs(), &study.participant_id) {
                    Ok(path) => StudyNote::Exported(path),
                    Err(e)   => StudyNote::Failed(format!("Export failed: {e}")),
                });
            }
            // Offered only while the log holds nothing an export didn't
            // copy, so a press can't wipe lines nobody has. Left out
            // when it can't act, because a disabled egui widget
            // surrenders focus mid-Tab and breaks the traversal for
            // every control after it (see the Background Color comment)
            clear_offered = log.clear_allowed();
            if clear_offered && ui.button("Clear log").clicked() {
                *note = Some(match log.clear(&crate::header_fields(settings)) {
                    Ok(())  => StudyNote::Cleared,
                    Err(e)  => StudyNote::Failed(format!("Clear failed: {e}")),
                });
            }
        });
        study_note_lines(ui, note.as_ref(), clear_offered);
    });
    false
}

/// What the Study card says under its buttons
///
/// Each line wraps inside the card, because an export path or an OS error
/// can be longer than the card is wide. After an export, `clear_offered`
/// false means the log has grown since, and the card says so
fn study_note_lines(ui: &mut egui::Ui, note: Option<&StudyNote>, clear_offered: bool) {
    fn small(ui: &mut egui::Ui, text: String) {
        ui.add(
            egui::Label::new(
                egui::RichText::new(text)
                    .small()
                    .color(ui.visuals().weak_text_color()),
            )
            .wrap(),
        );
    }
    match note {
        Some(StudyNote::Exported(path)) => {
            small(ui, format!("Exported to {}", path.display()));
            if !clear_offered {
                small(ui, CLEAR_WITHHELD_HINT.to_string());
            }
        }
        Some(StudyNote::Cleared)     => small(ui, "Log cleared.".to_string()),
        Some(StudyNote::Failed(why)) => small(ui, why.clone()),
        None                         => {}
    }
}

/// One "label: value" line in the Study card
///
/// Wraps long values, because a lab can hand out participant ids of
/// any length and a clipped one is worse than a two-line one
fn study_line(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        paint_label(ui, label);
        ui.add(egui::Label::new(egui::RichText::new(value).strong()).wrap());
    });
}

/// Returns the natural (fully-expanded) content height in logical points so
/// the caller can clamp the window's max size
#[allow(clippy::too_many_arguments)]
fn settings_ui(
    ctx:                    &egui::Context,
    settings:               &mut Settings,
    settings_manager:       &Arc<SettingsManager>,
    pending_reset:          &mut bool,
    capturing_shortcut_for: &mut Option<ShortcutAction>,
    on_quit:                &dyn Fn(),
    on_rebind_hotkeys:      &dyn Fn(),
    icons:                  &IconCache,
    session_log:            &Option<Arc<SessionLog>>,
    study_note:             &mut Option<StudyNote>,
    overlay_available:      bool,
) -> f32 {
    let mut dirty = false;
    let mut content_height = 0.0f32;
    // What the study file amounted to at launch, read once a frame. Every
    // surface that shows or changes the pace goes by it, and a study that
    // can't run also keeps Start from doing anything
    let study_status = settings_manager.study_status();
    // Set on any action that updates `settings.keyboard_shortcuts`, both
    // direct rebinds via the capture overlay and the per-action "Reset to
    // Default" context-menu entry need to fire `on_rebind_hotkeys` so
    // `main.rs` re-registers with the global-hotkey manager.  Set inside
    // the panel closure and consumed once at the end so we don't fire
    // the callback in the middle of an `egui::CentralPanel::show`
    let mut rebind_hotkeys = false;

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
    // Fully-transparent panel fill matches Swift's look
    //
    // The NSVisualEffectView's `.hudWindow` blur masks backdrop content
    // (terminal text, etc.) enough that nothing legible leaks through
    // the 14-px gutters
    //
    // Other platforms: opaque fallback (they have no vibrancy backend)
    let panel_fill = if platform::is_blur_active() {
        egui::Color32::TRANSPARENT
    } else if ctx.style().visuals.dark_mode {
        PANEL_FILL_DARK
    } else {
        PANEL_FILL_LIGHT
    };
    egui::CentralPanel::default()
        .frame(egui::Frame::none()
            .fill(panel_fill)
            .inner_margin(egui::Margin::symmetric(OUTER_PAD, OUTER_PAD)))
        .show(ctx, |ui| {
        // Cap the ScrollArea's max width to the panel content area so the
        // sections it contains can never be wider than the window's 14-px
        // horizontal gutters.  Using `max_width` here (instead of
        // `ui.set_width` on each card) propagates down through the nested
        // `available_width` chain: so rows inside the cards also know the
        // true right edge and right-aligned widgets (color swatches,
        // stepper fields, segmented pickers) end up flush with the card
        // boundary instead of extending past it
        let scroll_max_w = SETTINGS_WIDTH as f32 - 2.0 * OUTER_PAD;
        let scroll_out = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_width(scroll_max_w)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
            // `sectionSpacing: 10`: vertical gap between SectionCard instances
            ui.spacing_mut().item_spacing.y = SECTION_GAP;

            // Measure every segmented picker in the Appearance section once,
            // pick the widest natural width, and use it as the uniform column
            // width for all of them.  This guarantees the leftmost option of
            // every picker lands on the same X, regardless of option count
            // or text length, and the column only extends as far left as
            // the widest picker requires
            let picker_column_w = picker_column_width(ui);

            // ── A study that can't run ────────────────────────────────
            //
            // First card in the window, above Controls.  A lab that
            // installs a file and gets no lock has to find that out
            // before the participant runs a session at the shipped
            // default pace believing it's an assigned arm.  The same card
            // reports a session log that won't open.  The overlay stays
            // still and every pacing surface stays hidden until the file
            // is fixed and the app relaunched
            if let Some(err) = study_status.error() {
                study_error_card(ui, err);
            }

            // ── Controls (no header, matches Swift's top SectionCard) ───────
            // Swift has only three buttons: Start, Stop, Reset.  Pause is
            // implemented in `SettingsModel` but isn't exposed in the
            // SwiftUI `SettingsView`, so we omit it here too for 1:1
            // parity.  `Ctrl+Shift+S` (stop) handles "I want it to halt"
            // and the breathing controller treats `is_animating=false`
            // the same as paused for the purposes of stopping renders
            section(ui, "", |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = CONTROL_GAP;
                    let btn_w = control_slot_width(ui.available_width());

                    let dark = ui.visuals().dark_mode;

                    // Build per-button hover text that includes the current
                    // (possibly user-customised) shortcut binding.  Re-read
                    // from `settings.keyboard_shortcuts` every frame so the
                    // tooltip stays in sync with whatever the user just
                    // captured in the right-click -> Change Shortcut overlay
                    //
                    // Start can't do anything in a study that can't run, so
                    // its tooltip says why in place of what it would do
                    let start_enabled = study_status.error().is_none();
                    let start_help = if start_enabled {
                        format!(
                            "Start the app and re-initialize animation.{}",
                            shortcut_tooltip_line(settings, ShortcutAction::Start),
                        )
                    } else {
                        STUDY_START_HELP.to_string()
                    };
                    let stop_help = format!(
                        "Stop the animation and remove all screen tints.{}",
                        shortcut_tooltip_line(settings, ShortcutAction::Stop),
                    );
                    let reset_help = format!(
                        "{}{}",
                        reset_text(settings.study.is_some()).help,
                        shortcut_tooltip_line(settings, ShortcutAction::Reset),
                    );
                    let quit_help = format!(
                        "Quit exhale (full shutdown).{}",
                        shortcut_tooltip_line(settings, ShortcutAction::Quit),
                    );

                    let start_resp = ui.add_enabled_ui(start_enabled, |ui| control_button(
                        ui, btn_w,
                        // `icon` is the Unicode fallback when no SF
                        // Symbol texture is available and
                        // `draw_inner_triangle: false`.  Here we pass
                        // `true` so the Win / Linux path paints a
                        // primitive triangle instead, sidestepping
                        // Segoe UI's low-left U+25B6 placement.
                        // macOS keeps using Apple's `play.circle.fill`
                        // because the triangle flag yields to the
                        // texture path when one is available
                        "\u{25B6}", icons.get(IconKind::Play, dark),
                        None, 0.0, false, true,
                        "Start",
                        &start_help,
                    )).inner;
                    // No menu on a disabled Start: `context_menu` gives the
                    // widget click sense, which makes it focusable again and
                    // strands Tab on it (see `click_sense`).  A shortcut for a
                    // Start that can't act isn't worth binding either
                    #[cfg(feature = "global-hotkeys")]
                    if start_enabled {
                        shortcut_context_menu(
                            &start_resp, ShortcutAction::Start, settings,
                            capturing_shortcut_for, &mut dirty, &mut rebind_hotkeys,
                        );
                    }
                    if start_resp.clicked() {
                        settings.is_animating = true;
                        settings.is_paused    = false;
                        dirty = true;
                    }

                    let stop_resp = control_button(
                        ui, btn_w,
                        // `draw_inner_square: true` paints a primitive
                        // square, so `icon` is only a placeholder and
                        // there's no SF Symbol texture to pass
                        "\u{25A0}", None,
                        None, 0.0, true, false,
                        "Stop",
                        &stop_help,
                    );
                    #[cfg(feature = "global-hotkeys")]
                    shortcut_context_menu(
                        &stop_resp, ShortcutAction::Stop, settings,
                        capturing_shortcut_for, &mut dirty, &mut rebind_hotkeys,
                    );
                    if stop_resp.clicked() {
                        settings.is_animating = false;
                        settings.is_paused    = false;
                        dirty = true;
                    }

                    let reset_resp = control_button(
                        ui, btn_w,
                        "\u{21BA}", icons.get(IconKind::Reset, dark),
                        // U+21BA ANTICLOCKWISE OPEN CIRCLE ARROW lives
                        // in the Arrows block, and Segoe UI draws it
                        // taller than the Geometric Shapes glyphs
                        // (`▶ ■`): arrows traditionally reach into
                        // the ascender region.  At the 8 pt default
                        // the arrow is already pixel-tight.  Nudge to
                        // 9 pt for visual parity with the other
                        // icons in the row.  Applies only on the
                        // Unicode fallback path (Win / Linux).  macOS
                        // uses the SF Symbol texture which is sized
                        // uniformly.
                        // Scaled with the 13 pt ring (was 9.0 alongside
                        // the 16 pt ring: 7.3 ≈ 9.0 × 13 / 16 -> 7.5
                        // rounded keeps the arrow visually centred
                        // without poking past the ring's rim on the
                        // Unicode-fallback path)
                        Some(7.5), 0.0, false, false,
                        "Reset",
                        &reset_help,
                    );
                    #[cfg(feature = "global-hotkeys")]
                    shortcut_context_menu(
                        &reset_resp, ShortcutAction::Reset, settings,
                        capturing_shortcut_for, &mut dirty, &mut rebind_hotkeys,
                    );
                    if reset_resp.clicked() {
                        // Defer the actual reset to an inline
                        // confirmation card below the button row.  An
                        // unconfirmed Reset wipes every setting
                        // including custom keyboard shortcuts, and
                        // a single misclick (or focused-Reset + Space
                        // bar from the keyboard nav path) shouldn't
                        // be unrecoverable.  Setting `pending_reset`
                        // makes the confirmation card render this
                        // frame.  The user picks Cancel or Reset to
                        // resolve it.  Idempotent: clicking Reset
                        // again while already pending is a no-op
                        *pending_reset = true;
                    }
                    // Quit: full shutdown.  Dispatches directly via the
                    // injected `on_quit` callback (set up at
                    // `SettingsWindow::new`) so we don't need `main.rs`
                    // to poll a `pending_quit` flag after every render.
                    // Matches the tray-menu Quit path so all teardown
                    // (settings flush, controller stop, tray destroy)
                    // runs in the canonical order
                    let quit_resp = control_button(
                        ui, btn_w,
                        // U+00D7 MULTIPLICATION SIGN: Latin-1 Supplement
                        // block, part of basic Western font coverage so
                        // every TTF / OTF system UI font carries it by
                        // default.  Previous attempts (U+23FB POWER
                        // SYMBOL, U+2715 HEAVY MULTIPLICATION X) lived
                        // in Misc Technical / Dingbats blocks, and
                        // Segoe UI's regular face on Windows skips
                        // those: egui's font-fallback chain then
                        // rendered the tofu / missing-glyph box.
                        // `×` reads unambiguously as "close / quit" at
                        // glyph-icon scale and stays monochrome so it
                        // pairs with the other Geometric-Shapes icons
                        // (▶ / ■ / ↺) in the row
                        //
                        // `×` is intrinsically sized to lowercase
                        // x-height (Latin-1 lives alongside `é` `ñ`
                        // etc.), so at the default 8 pt it renders
                        // visibly shorter than the full-em-box
                        // Geometric Shapes glyphs.  Bump it ~50% so
                        // `×` lands at the same visible height as the
                        // other three icons inside the 16 pt circle
                        "\u{00D7}", icons.get(IconKind::Quit, dark),
                        // `×` lives at the math-axis (below the
                        // em-centre) instead of the em-centre where
                        // Geometric Shapes glyphs sit, so even when
                        // CENTER_CENTER-aligned the visible glyph
                        // lands a pixel low on the Unicode-fallback
                        // path.  Lift it 1 px so the four icons read
                        // as a single horizontal row.  macOS uses
                        // the `power` SF Symbol texture which is
                        // em-centred by Apple's design and doesn't
                        // need the offset, but the unified inner
                        // offset is small enough at 1 px that the
                        // texture path is still acceptably aligned
                        // Scaled with the 13 pt ring (was 12.0 alongside
                        // the 16 pt ring: 9.75 ≈ 12.0 × 13 / 16 -> 10.0
                        // rounded so the `×` keeps its visible weight
                        // against the smaller ring on the Unicode-
                        // fallback path)
                        Some(10.0), -1.0, false, false,
                        "Quit",
                        &quit_help,
                    );
                    #[cfg(feature = "global-hotkeys")]
                    shortcut_context_menu(
                        &quit_resp, ShortcutAction::Quit, settings,
                        capturing_shortcut_for, &mut dirty, &mut rebind_hotkeys,
                    );
                    if quit_resp.clicked() {
                        on_quit();
                    }
                });

                // ── Inline reset-confirmation card ─────────────────────────
                //
                // Rendered just below the Start/Stop/Reset/Quit row
                // when the user has clicked Reset (or fired the
                // Ctrl+Shift+D global hotkey on non-macOS).  Lives
                // inside the same `section` as the buttons so the
                // prompt sits in the document flow instead of a
                // floating `egui::Window` popup: the user asked
                // for it integrated into the settings UI directly.
                // Reset is destructive (wipes every setting and the
                // user's custom keyboard shortcuts), so the
                // confirmation lays out the consequence in plain
                // language and offers Cancel as the more prominent
                // default action.  The destructive `Reset` button is
                // a `small`-styled red text label to lower its
                // visual weight relative to Cancel and reduce
                // misclick risk
                if *pending_reset {
                    ui.add_space(ROW_GAP);
                    let warning_color = warning_color(ui);
                    let reset = reset_text(settings.study.is_some());
                    ui.label(egui::RichText::new(reset.title).strong());
                    ui.label(
                        egui::RichText::new(reset.detail)
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(4.0);
                    // Cancel + Reset are paired, equal-weight
                    // actions: easier to read when they share the
                    // exact same footprint and sit symmetrically
                    // under the warning text instead of left-
                    // aligned and unevenly sized (Cancel's longer
                    // glyph string would otherwise auto-grow it
                    // wider than Reset).  Per-button width is
                    // `control_slot_width`, the same four-slot split
                    // the top row uses, so each confirmation button
                    // lines up with one of the Start / Stop / Reset /
                    // Quit slots above it.  The pair sits centred in
                    // the row via explicit left padding
                    const BTN_H: f32 = ROW_H;
                    let avail    = ui.available_width();
                    let btn_w    = control_slot_width(avail);
                    let pair_w   = btn_w * 2.0 + CONTROL_GAP;
                    let left_pad = ((avail - pair_w) * 0.5).max(0.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = CONTROL_GAP;
                        if left_pad > 0.0 {
                            ui.add_space(left_pad);
                        }
                        // Cancel: muted default styling.  Esc also
                        // dismisses via the input handling below so
                        // a user who hit Reset by accident can fall
                        // back to the keyboard
                        if ui.add_sized(
                            egui::vec2(btn_w, BTN_H),
                            egui::Button::new("Cancel"),
                        ).clicked() {
                            *pending_reset = false;
                        }
                        // Reset: destructive, red label.  Same
                        // dimensions as Cancel so the visual weight
                        // of the choice is equal.  The colour is the
                        // only thing that flags it as the dangerous
                        // option.  Stays focusable via Tab so
                        // keyboard-nav (Tab onto Reset, Enter /
                        // Space to confirm) still works
                        if ui.add_sized(
                            egui::vec2(btn_w, BTN_H),
                            egui::Button::new(
                                egui::RichText::new("Reset")
                                    .color(warning_color)
                                    .strong(),
                            ),
                        ).clicked() {
                            // Writes the study log's `reset` line too, as
                            // the tray's Reset does.  The hotkey comes
                            // through this card as well
                            crate::reset_settings(settings, session_log.as_deref());
                            dirty = true;
                            // Defaults include the keyboard-shortcut
                            // block, so the rebind path must fire to
                            // re-register globals from the new
                            // (default) shortcut set.  Otherwise the
                            // user's pre-reset bindings would keep
                            // firing despite the settings file
                            // claiming defaults
                            rebind_hotkeys = true;
                            *pending_reset = false;
                        }
                    });
                    // Esc dismisses the confirmation without
                    // resetting: matches the platform convention
                    // for any prompt with a Cancel button.  Only
                    // consume the key while the prompt is open so
                    // it stays available to text fields / other
                    // egui widgets otherwise
                    if ui.input_mut(|i| i.consume_key(
                        egui::Modifiers::NONE, egui::Key::Escape,
                    )) {
                        *pending_reset = false;
                    }
                }
            });

            // ── Appearance ───────────────────────────────────────────────────
            if appearance_card(ui, settings, &study_status, overlay_available, picker_column_w) {
                dirty = true;
            }

            if body_cards(ui, settings, &study_status, session_log, study_note) {
                dirty = true;
            }

            // Version footer, on every platform and in every build.
            // Windows and Linux have no About panel, so without this
            // there's no way to read back which build is running, and a
            // study that can't name the version its participants ran
            // can't be replicated
            ui.label(
                egui::RichText::new(format!("exhale {}", env!("CARGO_PKG_VERSION")))
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        });
        content_height = scroll_out.content_size.y;
    });

    if dirty {
        settings_manager.mark_dirty();
    }

    // Tab-wrap repaint nudge.  egui's focus traversal surrenders the
    // current widget's focus this frame and gives focus to the first
    // widget that registers interest on the next frame (via
    // `give_to_next`).  Tab itself is a keyboard event that
    // egui-winit reports with `repaint: true`, so a follow-up frame
    // normally runs and the wrap completes.  But if anything in the
    // caller skips that second frame, the user sees focus
    // disappear instead of wrapping to the top.  Explicit
    // `request_repaint` guarantees the second frame runs no matter
    // what.  egui de-dupes repeat requests so this is a no-op when
    // a frame was already going to happen
    let tab_pressed = ctx.input(|i| i.events.iter().any(|e| matches!(
        e,
        egui::Event::Key { key: egui::Key::Tab, pressed: true, .. },
    )));
    if tab_pressed {
        ctx.request_repaint();
    }
    // ── Shortcut capture overlay ──────────────────────────────────────────────
    // Painted at the top z-order via egui::Window so it sits above the
    // settings panel and intercepts the next keystroke.  Watching
    // `ctx.input` for `Event::Key { pressed: true }` ignores hover /
    // focus / scroll events so the user only has to press the new
    // combination once.  Esc cancels.  The close-button on the window
    // also cancels by clearing `capturing_shortcut_for`
    if let Some(action) = *capturing_shortcut_for {
        let mut still_capturing = true;
        let mut closed_via_button = false;
        egui::Window::new(format!("Set Shortcut: {}", action.label()))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut still_capturing)
            .show(ctx, |ui| {
                ui.set_width(280.0);
                ui.label("Press the key combination you want to bind…");
                ui.label("Include at least one modifier (Ctrl / Shift / Alt / Cmd).");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Current binding:");
                    let current = settings.keyboard_shortcuts.get(action)
                        .map(|sc| sc.display())
                        .unwrap_or_else(|| "(none)".to_string());
                    ui.monospace(current);
                });
                ui.add_space(8.0);
                if ui.button("Cancel").clicked() {
                    closed_via_button = true;
                }
                ui.add_space(4.0);
                ui.small("Press Esc to cancel.");
            });

        // egui::Window's `.open()` flips `still_capturing` to false when
        // the user clicks the title-bar close button.  Either signal
        // cancels capture and leaves the binding unchanged
        if !still_capturing || closed_via_button {
            *capturing_shortcut_for = None;
        } else {
            // Inspect this frame's input events for a key press matching
            // the "valid combo" criteria (at least one modifier)
            let captured = ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, modifiers, pressed: true, repeat: false, .. } = event {
                        if *key == egui::Key::Escape && !modifiers.any() {
                            return Some(None);
                        }
                        // Reject plain printable keys with no modifier so
                        // the user doesn't accidentally bind `A` to Start
                        // and then lose access to every text field
                        let has_modifier = modifiers.ctrl
                            || modifiers.shift
                            || modifiers.alt
                            || modifiers.mac_cmd
                            || modifiers.command;
                        if !has_modifier { continue; }
                        if let Some(code) = egui_key_to_code(*key) {
                            return Some(Some((code, *modifiers)));
                        }
                    }
                }
                None
            });
            match captured {
                Some(None) => { *capturing_shortcut_for = None; }
                Some(Some((code, mods))) => {
                    let mask = egui_modifiers_to_mask(mods);
                    settings.keyboard_shortcuts.set(
                        action,
                        Some(KeyboardShortcut::new(mask, code)),
                    );
                    *capturing_shortcut_for = None;
                    rebind_hotkeys = true;
                    settings_manager.mark_dirty();
                }
                None => {}
            }
        }
    }
    if rebind_hotkeys {
        on_rebind_hotkeys();
    }

    content_height
}

// ─── Tests ───────────────────────────────────────────────────────────────
//
// These tests drive a minimal egui::Context directly (no winit, no wgpu),
// feeding synthetic pointer events to verify the stepper_row widgets
// respond to clicks without panicking.  That exercises the exact code
// path the user complained about: "click the up and down stepper buttons
// they do nothing"
#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, Pos2, Rect, RawInput, Vec2};

    // Clamp-helper tests for the position-restore path live in
    // `crate::placement::tests` now that the helpers are shared
    // between this window and the windowed-mode animation window

    /// Build a single RawInput frame with a pointer move to `pos` followed
    /// by a down+up primary click at that position.  egui requires both
    /// the down and up within the same frame to register as a `clicked()`
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
    /// it.  `click` is fed as the RawInput for this frame
    fn run_stepper_frame(
        ctx:     &Context,
        raw_in:  RawInput,
        value:   &mut f64,
        label:   &str,
    ) -> bool {
        let mut changed = false;
        // egui::Context::run returns `FullOutput`.  We don't need it
        // here because the test only inspects the side effect on
        // `changed`
        let _ = ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Force the ui wide enough that stepper_row's layout is
                // the same as in the real app
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
    /// memory and records the exact top/bot rects via the `test_hooks`
    /// side channel.  Then the click frame targets the center of the
    /// recorded up/down half
    fn simulate_click_on_stepper(ctx: &Context, value: &mut f64, click_up: bool) -> bool {
        // Frame A (warmup): no input, run stepper_row to register
        // widgets and capture sub-rects
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

        // Frame B: click the center of the desired half
        let target = if click_up { top_rect } else { bot_rect };
        let click_pos = target.center();
        run_stepper_frame(ctx, click_input(click_pos), value, "Test")
    }

    /// Content width inside a section card: `SETTINGS_WIDTH` minus the
    /// outer gutters and the card padding. The chips have to wrap
    /// against the real number instead of a convenient one
    const CARD_CONTENT_W: f32 = SETTINGS_WIDTH as f32 - 2.0 * OUTER_PAD - 2.0 * CARD_PAD;

    /// Run one frame of `preset_chips` inside a real `section` card
    ///
    /// Goes through the card because a bare `ui.set_width` on a
    /// `CentralPanel`'s own `Ui` is silently undone: `Placer::set_max_width`
    /// unions the result back with `min_rect`, which for a panel is already
    /// the full panel. The first version of this harness therefore laid the
    /// chips out at 384 pt and would have passed while the shipped card is
    /// 308. Going through `section` measures what the app renders
    fn run_chips_frame(
        ctx:      &Context,
        raw_in:   RawInput,
        settings: &mut exhale_core::settings::Settings,
    ) -> bool {
        let mut changed = false;
        let _ = ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                section(ui, "Timing", |ui| {
                    changed = preset_chips(ui, settings);
                });
            });
        });
        changed
    }

    fn blank_input() -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 800.0))),
            ..Default::default()
        }
    }

    fn key_input(key: egui::Key) -> RawInput {
        RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            ..blank_input()
        }
    }

    /// Warm-up frame to register the chip rects, then hand them back
    fn chip_rects(ctx: &Context, settings: &mut exhale_core::settings::Settings) -> Vec<Rect> {
        let mut probe = settings.clone();
        let _ = run_chips_frame(ctx, blank_input(), &mut probe);
        super::widgets::test_hooks::take_chip_rects()
            .expect("preset_chips should record its rects every frame")
    }

    #[test]
    fn clicking_a_chip_moves_all_four_durations() {
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        // Picks a target other than chip 0: that's the shipped default, so
        // clicking it would leave every field at the value it already
        // had and the test would pass without proving anything moved.
        // Chip 3 is box breathing, which differs in all four
        const TARGET: usize = 3;
        let rects = chip_rects(&ctx, &mut settings);
        assert_eq!(
            exhale_core::presets::selected(&settings), Some(0),
            "this test assumes the default is chip 0 and the target is not"
        );

        let changed = run_chips_frame(&ctx, click_input(rects[TARGET].center()), &mut settings);

        assert!(changed, "clicking a chip should mark settings dirty");
        let want = exhale_core::presets::PRESETS[TARGET];
        assert_eq!(settings.inhale_duration, want.inhale);
        assert_eq!(settings.post_inhale_hold_duration, want.post_inhale_hold);
        assert_eq!(settings.exhale_duration, want.exhale);
        assert_eq!(settings.post_exhale_hold_duration, want.post_exhale_hold);
        assert_eq!(exhale_core::presets::selected(&settings), Some(TARGET));
    }

    #[test]
    fn space_activates_a_focused_chip() {
        // The handoff flagged this as unverified: it wasn't known
        // whether egui 0.29 synthesises a click from Space on a focused
        // `ui.interact` response the way it does for `Button`. It does
        // (`Context::create_widget` sets `fake_primary_click`), and
        // without it the whole feature would be mouse-only, so the
        // behaviour is pinned here instead of trusted to survive an
        // egui upgrade
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();

        // Frame 1 registers the widgets so Tab has somewhere to go
        let _ = run_chips_frame(&ctx, blank_input(), &mut settings);
        // Frame 2: Tab focuses the first chip
        let _ = run_chips_frame(&ctx, key_input(egui::Key::Tab), &mut settings);
        // Frame 3: Space on the focused chip
        let changed = run_chips_frame(&ctx, key_input(egui::Key::Space), &mut settings);

        assert!(changed, "Space on a focused chip should activate it");
        assert_eq!(exhale_core::presets::selected(&settings), Some(0));
    }

    #[test]
    fn enter_also_activates_a_focused_chip() {
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        let _ = run_chips_frame(&ctx, blank_input(), &mut settings);
        let _ = run_chips_frame(&ctx, key_input(egui::Key::Tab), &mut settings);
        let changed = run_chips_frame(&ctx, key_input(egui::Key::Enter), &mut settings);
        assert!(changed);
        assert_eq!(exhale_core::presets::selected(&settings), Some(0));
    }

    #[test]
    fn every_chip_is_reachable_by_tab_in_display_order() {
        // Focus order follows creation order, so tabbing N times lands
        // on chip N. A chip that could be clicked but not reached is
        // half a control
        for want in 0..exhale_core::presets::PRESETS.len() {
            let ctx = Context::default();
            let mut settings = exhale_core::settings::Settings::default();
            let _ = run_chips_frame(&ctx, blank_input(), &mut settings);
            for _ in 0..=want {
                let _ = run_chips_frame(&ctx, key_input(egui::Key::Tab), &mut settings);
            }
            let changed = run_chips_frame(&ctx, key_input(egui::Key::Space), &mut settings);
            assert!(changed, "chip {want} was not reachable in {} tabs", want + 1);
            assert_eq!(
                exhale_core::presets::selected(&settings), Some(want),
                "tabbing {} times activated the wrong chip", want + 1
            );
        }
    }

    #[test]
    fn chips_wrap_inside_the_card_and_never_overlap() {
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        let rects = chip_rects(&ctx, &mut settings);
        assert_eq!(rects.len(), exhale_core::presets::PRESETS.len());

        let left = rects[0].min.x;
        for (i, r) in rects.iter().enumerate() {
            // Chips wrap instead of clipping. A chip wider than the card would
            // be a label that no longer describes its own pattern
            assert!(
                r.width() <= CARD_CONTENT_W + 0.5,
                "chip {i} is {} wide in a {CARD_CONTENT_W} card", r.width()
            );
            assert!(
                r.max.x <= left + CARD_CONTENT_W + 0.5,
                "chip {i} overflows the card right edge by {}",
                r.max.x - (left + CARD_CONTENT_W)
            );
        }
        for (i, a) in rects.iter().enumerate() {
            for (j, b) in rects.iter().enumerate().skip(i + 1) {
                assert!(!a.intersects(*b), "chips {i} and {j} overlap");
            }
        }

        // The five shipped labels fit one row on macOS with 11 pt to
        // spare, but text metrics differ per platform, so the assertion
        // is the property worth keeping instead of the exact result: a
        // sixth preset, or a label long enough to need three rows, turns
        // the block back into the four-line stack this was tightened to
        // avoid. Wrapping to a second row is fine.  A third row should
        // come from a layout change someone chose
        let rows = rects.iter().map(|r| r.min.y as i32).collect::<std::collections::BTreeSet<_>>();
        assert!(
            rows.len() <= 2,
            "preset chips need {} rows; the Timing card was tuned for one or two",
            rows.len()
        );
    }

    #[test]
    fn nudging_one_stepper_unselects_every_chip() {
        // Derived selection, stated as a behaviour. There's no
        // "Custom" chip to light up instead: custom is the absence of
        // a lit pill
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        assert!(exhale_core::presets::selected(&settings).is_some());

        settings.exhale_duration += 1.0;
        assert_eq!(exhale_core::presets::selected(&settings), None);

        // And the frame still renders with nothing selected
        let _ = run_chips_frame(&ctx, blank_input(), &mut settings);
    }

    #[test]
    fn clicking_a_chip_leaves_drift_and_jitter_where_the_user_put_them() {
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings {
            drift: 1.001,
            randomized_timing_exhale: 0.25,
            ..Default::default()
        };

        let rects = chip_rects(&ctx, &mut settings);
        let _ = run_chips_frame(&ctx, click_input(rects[3].center()), &mut settings);

        assert_eq!(exhale_core::presets::selected(&settings), Some(3));
        assert_eq!(settings.drift, 1.001);
        assert_eq!(settings.randomized_timing_exhale, 0.25);
    }

    #[test]
    fn clicking_a_segment_selects_it() {
        const OPTIONS: &[(&str, u8)] = &[("One", 1), ("Two", 2), ("Three", 3)];
        let ctx = Context::default();
        let mut value = 1_u8;
        let frame = |raw_in: RawInput, value: &mut u8| {
            let mut changed = false;
            let _ = ctx.run(raw_in, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    changed = segmented_row(ui, "Pick", "help", 180.0, value, OPTIONS);
                });
            });
            changed
        };

        // Warm-up frame registers the segment rects, then click the second
        let _ = frame(blank_input(), &mut value);
        let rects = super::widgets::test_hooks::take_seg_rects()
            .expect("segmented_row should record its rects every frame");
        let changed = frame(click_input(rects[1].center()), &mut value);

        assert!(changed, "clicking a segment should report a change");
        assert_eq!(value, 2);
    }

    // ── Display row ───────────────────────────────────────────────

    /// One frame of the Display row inside a real Appearance card, at the
    /// column width the window measures.  Goes through `section` for the
    /// reason `run_chips_frame` does: a bare panel `Ui` lays out wider
    /// than the shipped card
    fn run_display_frame(
        ctx:               &Context,
        raw_in:            RawInput,
        settings:          &mut exhale_core::settings::Settings,
        overlay_available: bool,
    ) -> bool {
        let mut changed = false;
        let _ = ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                section(ui, "Appearance", |ui| {
                    let column_w = picker_column_width(ui);
                    changed = display_row(ui, settings, overlay_available, column_w);
                });
            });
        });
        changed
    }

    #[test]
    fn clicking_window_in_the_display_row_changes_the_setting() {
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        assert_eq!(settings.display_mode, DisplayMode::Overlay);

        // Warm-up frame registers the two segments
        let _ = run_display_frame(&ctx, blank_input(), &mut settings, true);
        let rects = super::widgets::test_hooks::take_seg_rects()
            .expect("the Display row records its segments");
        assert_eq!(rects.len(), 2, "Overlay | Window");
        assert_eq!(super::widgets::test_hooks::take_seg_selected(), Some(0));

        let changed = run_display_frame(
            &ctx, click_input(rects[1].center()), &mut settings, true,
        );
        assert!(changed, "clicking Window should mark settings dirty");
        assert_eq!(settings.display_mode, DisplayMode::Window);

        let _ = run_display_frame(&ctx, blank_input(), &mut settings, true);
        assert_eq!(super::widgets::test_hooks::take_seg_selected(), Some(1));

        // And back
        let changed = run_display_frame(
            &ctx, click_input(rects[0].center()), &mut settings, true,
        );
        assert!(changed);
        assert_eq!(settings.display_mode, DisplayMode::Overlay);
    }

    #[test]
    fn a_session_with_no_overlay_shows_window_selected_and_ignores_every_click() {
        let ctx = Context::default();
        // What a Wayland machine has stored: the default, since the row
        // never lets anyone change it there
        let mut settings = exhale_core::settings::Settings::default();
        assert_eq!(settings.display_mode, DisplayMode::Overlay);

        let _ = run_display_frame(&ctx, blank_input(), &mut settings, false);
        let rects = super::widgets::test_hooks::take_seg_rects()
            .expect("a disabled row still lays out its segments");
        assert_eq!(
            super::widgets::test_hooks::take_seg_selected(), Some(1),
            "Window is the lit segment whatever is stored",
        );

        for (i, rect) in rects.iter().enumerate() {
            let changed = run_display_frame(
                &ctx, click_input(rect.center()), &mut settings, false,
            );
            assert!(!changed, "segment {i} of a disabled row reported a change");
        }
        assert_eq!(
            settings.display_mode, DisplayMode::Overlay,
            "the stored choice isn't touched",
        );

        let _ = run_display_frame(&ctx, blank_input(), &mut settings, false);
        assert_eq!(super::widgets::test_hooks::take_seg_selected(), Some(1));
    }

    #[test]
    fn tab_skips_a_disabled_display_row_and_walks_an_enabled_one() {
        // The display row followed by the next control in Tab order
        let frame = |ctx: &Context, raw_in: RawInput, overlay_available: bool| {
            let mut settings = exhale_core::settings::Settings::default();
            let mut next = egui::Id::NULL;
            let _ = ctx.run(raw_in, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    section(ui, "Appearance", |ui| {
                        let column_w = picker_column_width(ui);
                        display_row(ui, &mut settings, overlay_available, column_w);
                        next = ui.button("Next").id;
                    });
                });
            });
            next
        };
        let tab_presses = |overlay_available: bool| {
            let ctx = Context::default();
            let next = frame(&ctx, blank_input(), overlay_available);
            for presses in 1..=3 {
                let _ = frame(&ctx, key_input(egui::Key::Tab), overlay_available);
                if ctx.memory(|m| m.focused()) == Some(next) {
                    return presses;
                }
            }
            usize::MAX
        };

        // An enabled row is two stops, Overlay then Window, before the next control
        assert_eq!(tab_presses(true), 3);
        // A disabled row is none. egui hands focus to a disabled widget and
        // takes it back the same frame, which strands the traversal for
        // every control after it, so the row has to offer no stop at all
        assert_eq!(tab_presses(false), 1);
    }

    #[test]
    fn the_display_row_lines_up_with_shape_and_stays_in_the_card_in_both_states() {
        // Display and Shape laid out in one card, as the window does
        let frame = |ctx: &Context, overlay_available: bool| {
            let mut settings = exhale_core::settings::Settings::default();
            let (mut display, mut shape, mut right) = (vec![], vec![], 0.0_f32);
            let _ = ctx.run(blank_input(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    section(ui, "Appearance", |ui| {
                        let column_w = picker_column_width(ui);
                        display_row(ui, &mut settings, overlay_available, column_w);
                        display = super::widgets::test_hooks::take_seg_rects().unwrap_or_default();
                        let mut shape_value = settings.shape;
                        segmented_row(ui, "Shape", "help", column_w, &mut shape_value, SHAPE_OPTIONS);
                        shape = super::widgets::test_hooks::take_seg_rects().unwrap_or_default();
                        right = ui.max_rect().right();
                    });
                });
            });
            (display, shape, right)
        };

        let ctx = Context::default();
        let _ = frame(&ctx, true);
        let (enabled, shape, card_right) = frame(&ctx, true);
        let ctx = Context::default();
        let _ = frame(&ctx, false);
        let (disabled, shape_locked, _) = frame(&ctx, false);

        assert_eq!(enabled.len(), 2);
        assert_eq!(shape.len(), 3);
        assert_eq!(enabled, disabled, "locking the row moves nothing");
        assert_eq!(shape, shape_locked, "and nothing below it either");

        // Same left and right edge as the Shape picker above or below it
        let (d_left, d_right) = (enabled[0].min.x, enabled[1].max.x);
        assert!((d_left  - shape[0].min.x).abs() < 0.5, "left edge {d_left} vs Shape {}", shape[0].min.x);
        assert!((d_right - shape[2].max.x).abs() < 0.5, "right edge {d_right} vs Shape {}", shape[2].max.x);

        // Inside the card, and the segments in order without overlap
        // (contiguous segments share an edge, so `intersects` would say yes)
        assert!(d_right <= card_right + 0.5, "the picker overflows the card by {}", d_right - card_right);
        assert!(
            enabled[0].max.x <= enabled[1].min.x + 0.5,
            "Overlay ends at {} but Window starts at {}", enabled[0].max.x, enabled[1].min.x,
        );
    }

    // ── Overlay Opacity row ───────────────────────────────────────

    /// One frame of the Overlay Opacity row inside a real Appearance card,
    /// followed by the next control in Tab order.  Returns whether the row
    /// changed the setting and where that next control sits
    fn run_opacity_frame(
        ctx:            &Context,
        raw_in:         RawInput,
        settings:       &mut exhale_core::settings::Settings,
        window_showing: bool,
    ) -> (bool, egui::Response) {
        let mut changed = false;
        let mut next = None;
        let _ = ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                section(ui, "Appearance", |ui| {
                    changed = opacity_row(ui, settings, window_showing);
                    next = Some(ui.button("Next"));
                });
            });
        });
        (changed, next.expect("the frame ran"))
    }

    /// Where the row's number field sits, from the stepper's two halves
    fn opacity_field_center(up: egui::Rect, down: egui::Rect) -> egui::Pos2 {
        egui::pos2(up.left() - 2.0 - STEPPER_FIELD_W / 2.0, (up.top() + down.bottom()) / 2.0)
    }

    #[test]
    fn the_opacity_row_follows_what_is_showing() {
        let settings = |mode| exhale_core::settings::Settings {
            display_mode: mode,
            ..Default::default()
        };
        // An overlay uses its opacity
        assert!(!window_showing(&settings(DisplayMode::Overlay), true));
        // Window mode is a window, and so is a session with no overlay to show,
        // whatever the stored mode says
        assert!(window_showing(&settings(DisplayMode::Window),  true));
        assert!(window_showing(&settings(DisplayMode::Overlay), false));
        assert!(window_showing(&settings(DisplayMode::Window),  false));
    }

    #[test]
    fn the_stepper_edits_the_opacity_only_while_an_overlay_is_showing() {
        // Overlay: the up half takes 25 % to 26 %
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        assert_eq!(settings.overlay_opacity, 0.25);
        let _ = run_opacity_frame(&ctx, blank_input(), &mut settings, false);
        let (up, _) = super::widgets::test_hooks::take_stepper_rects()
            .expect("the row records its stepper halves");
        let (changed, _) = run_opacity_frame(&ctx, click_input(up.center()), &mut settings, false);
        assert!(changed, "the up half should mark settings dirty");
        assert!((settings.overlay_opacity - 0.26).abs() < 1e-6);

        // Window: neither half does anything, and the stored value stays
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        let _ = run_opacity_frame(&ctx, blank_input(), &mut settings, true);
        let (up, down) = super::widgets::test_hooks::take_stepper_rects()
            .expect("a disabled row still lays out its stepper");
        for (name, target) in [("up", up), ("down", down)] {
            let (changed, _) = run_opacity_frame(&ctx, click_input(target.center()), &mut settings, true);
            assert!(!changed, "the {name} half of a disabled stepper reported a change");
        }
        assert_eq!(settings.overlay_opacity, 0.25, "the stored opacity is untouched");
    }

    #[test]
    fn the_number_field_takes_focus_and_typing_only_while_an_overlay_is_showing() {
        let typed = |text: &str| RawInput {
            events: vec![egui::Event::Text(text.to_string())],
            ..blank_input()
        };

        // Overlay: a click puts the cursor in the field, which is how the
        // click below is known to land on it
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        let _ = run_opacity_frame(&ctx, blank_input(), &mut settings, false);
        let (up, down) = super::widgets::test_hooks::take_stepper_rects().expect("stepper rects");
        let field = opacity_field_center(up, down);
        let _ = run_opacity_frame(&ctx, click_input(field), &mut settings, false);
        assert!(ctx.memory(|m| m.focused()).is_some(), "the click should have focused the field");

        // Window: the same click and keystrokes reach nothing
        let ctx = Context::default();
        let mut settings = exhale_core::settings::Settings::default();
        let _ = run_opacity_frame(&ctx, blank_input(), &mut settings, true);
        let (up, down) = super::widgets::test_hooks::take_stepper_rects().expect("stepper rects");
        let field = opacity_field_center(up, down);
        let _ = run_opacity_frame(&ctx, click_input(field), &mut settings, true);
        assert!(ctx.memory(|m| m.focused()).is_none(), "a disabled field took focus");
        let (changed, _) = run_opacity_frame(&ctx, typed("9"), &mut settings, true);
        assert!(!changed);
        assert_eq!(settings.overlay_opacity, 0.25);
    }

    #[test]
    fn tab_skips_a_disabled_opacity_row_and_walks_an_enabled_one() {
        let tab_presses = |window_showing: bool| {
            let ctx = Context::default();
            let mut settings = exhale_core::settings::Settings::default();
            let (_, next) = run_opacity_frame(&ctx, blank_input(), &mut settings, window_showing);
            for presses in 1..=5 {
                let _ = run_opacity_frame(&ctx, key_input(egui::Key::Tab), &mut settings, window_showing);
                if ctx.memory(|m| m.focused()) == Some(next.id) {
                    return presses;
                }
            }
            usize::MAX
        };

        // An enabled row is three stops, the field then both halves, before the next control
        assert_eq!(tab_presses(false), 4);
        // A disabled one is none: see `tab_skips_a_disabled_display_row_and_walks_an_enabled_one`
        assert_eq!(tab_presses(true), 1);
    }

    #[test]
    fn locking_the_opacity_row_moves_nothing() {
        let layout = |window_showing: bool| {
            let ctx = Context::default();
            let mut settings = exhale_core::settings::Settings::default();
            let _ = run_opacity_frame(&ctx, blank_input(), &mut settings, window_showing);
            let (_, next) = run_opacity_frame(&ctx, blank_input(), &mut settings, window_showing);
            let halves = super::widgets::test_hooks::take_stepper_rects().expect("stepper rects");
            (halves, next.rect)
        };

        let (enabled_halves, enabled_next) = layout(false);
        let (locked_halves,  locked_next)  = layout(true);
        assert_eq!(enabled_halves, locked_halves, "the stepper stays where it was");
        assert_eq!(enabled_next, locked_next, "and so does the control below it");
    }

    #[test]
    fn the_tooltips_say_what_each_row_does_in_each_state() {
        assert_eq!(
            DISPLAY_HELP,
            "Overlay floats above your other windows and lets clicks through. \
             Window is a regular window you can move, resize or make full screen.",
        );
        assert!(DISPLAY_LOCKED_HELP.starts_with("This session always shows a window"));
        assert_eq!(
            OPACITY_WINDOW_HELP,
            "A window draws at full strength. Overlay Opacity applies to the overlay.",
        );
        // The overlay's own wording is still true there, so it stays
        assert_eq!(OPACITY_HELP, "Transparency of the overlay. Lower = more transparent.");
    }

    // ── Study mode ────────────────────────────────────────────────

    fn locked_settings() -> exhale_core::settings::Settings {
        let study = exhale_core::StudyConfig {
            participant_id: "P07".to_string(),
            condition:      "arm-blue-7".to_string(),
            inhale_duration:           5.0,
            post_inhale_hold_duration: 0.0,
            exhale_duration:           5.0,
            post_exhale_hold_duration: 0.0,
            drift:                     1.0,
            randomized_timing_inhale:           0.0,
            randomized_timing_post_inhale_hold: 0.0,
            randomized_timing_exhale:           0.0,
            randomized_timing_post_exhale_hold: 0.0,
            auto_stop_minutes:         0.0,
            reminder_interval_minutes: 0.0,
            inhale_color:        None,
            exhale_color:        None,
            background_color:    None,
            overlay_opacity:     None,
            shape:               None,
            color_fill_gradient: None,
            animation_mode:      None,
            hold_ripple_mode:    None,
            display_mode:        None,
        };
        let mut s = exhale_core::settings::Settings::default();
        study.apply(&mut s);
        s
    }

    /// A frame as tall as the whole window can get, at the window's own
    /// width. egui paints no label that lies wholly outside the clip
    /// rectangle, so a shorter one would hide the lower cards from every
    /// test that looks for their text
    fn tall_input() -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(SETTINGS_WIDTH as f32, 2400.0),
            )),
            ..Default::default()
        }
    }

    fn tall_click(pos: Pos2) -> RawInput {
        RawInput { events: click_input(pos).events, ..tall_input() }
    }

    fn tall_key(key: egui::Key) -> RawInput {
        RawInput { events: key_input(key).events, ..tall_input() }
    }

    /// What one frame drew: each piece of text, with the rectangle it
    /// landed in
    ///
    /// The tests read the screen through this, so "the card shows no
    /// condition" and "the hint wraps inside the card" are claims about
    /// what a participant sees
    struct Drawn(Vec<(String, Rect)>);

    impl Drawn {
        fn of(output: &egui::FullOutput) -> Self {
            fn visit(shape: &egui::epaint::Shape, found: &mut Vec<(String, Rect)>) {
                match shape {
                    egui::epaint::Shape::Text(t) => found.push((
                        t.galley.text().to_string(),
                        t.galley.rect.translate(t.pos.to_vec2()),
                    )),
                    egui::epaint::Shape::Vec(inner) => {
                        inner.iter().for_each(|s| visit(s, found));
                    }
                    _ => {}
                }
            }
            let mut found = Vec::new();
            for clipped in &output.shapes {
                visit(&clipped.shape, &mut found);
            }
            Self(found)
        }

        /// Whether any piece of text contains `needle`
        fn says(&self, needle: &str) -> bool {
            self.0.iter().any(|(text, _)| text.contains(needle))
        }

        /// How many pieces of text are exactly `text`
        fn count(&self, text: &str) -> usize {
            self.0.iter().filter(|(t, _)| t == text).count()
        }

        /// Where the one piece of text that is exactly `text` landed
        fn rect(&self, text: &str) -> Rect {
            let hits: Vec<_> = self.0.iter().filter(|(t, _)| t == text).collect();
            assert_eq!(hits.len(), 1, "expected one {text:?} on screen, saw: {}", self.dump());
            hits[0].1
        }

        /// The piece of text that is exactly `text` and lowest on screen
        fn lowest(&self, text: &str) -> Rect {
            self.0.iter()
                .filter(|(t, _)| t == text)
                .map(|&(_, r)| r)
                .max_by(|a, b| a.min.y.total_cmp(&b.min.y))
                .unwrap_or_else(|| panic!("no {text:?} on screen, saw: {}", self.dump()))
        }

        /// Everything drawn, for a failure message
        fn dump(&self) -> String {
            self.0.iter().map(|(t, _)| format!("{t:?}")).collect::<Vec<_>>().join(", ")
        }
    }

    /// One frame of the cards under Appearance. No `SettingsManager`, so
    /// no writer thread and nothing touches a real config directory
    fn run_body_cards_frame(
        ctx:      &Context,
        raw_in:   RawInput,
        settings: &mut exhale_core::settings::Settings,
        status:   &StudyStatus,
        log:      &Option<Arc<SessionLog>>,
        note:     &mut Option<StudyNote>,
    ) -> Drawn {
        let out = ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = body_cards(ui, settings, status, log, note);
            });
        });
        Drawn::of(&out)
    }

    /// One frame of the whole settings window, as the app draws it. The
    /// manager decides what a study file amounted to, and the icons are
    /// the empty set the Unicode fallback runs on
    fn run_window_frame(
        ctx:           &Context,
        raw_in:        RawInput,
        settings:      &mut exhale_core::settings::Settings,
        manager:       &Arc<SettingsManager>,
        pending_reset: &mut bool,
        log:           &Option<Arc<SessionLog>>,
        note:          &mut Option<StudyNote>,
    ) -> Drawn {
        let icons = IconCache { handles: Default::default() };
        let mut capturing = None;
        let out = ctx.run(raw_in, |ctx| {
            settings_ui(
                ctx, settings, manager, pending_reset, &mut capturing,
                &|| {}, &|| {}, &icons, log, note, true,
            );
        });
        Drawn::of(&out)
    }

    /// A study file's status the way the app reads it: a folder with the
    /// file in it, and the settings the manager loaded
    fn study_folder(
        tag:  &str,
        file: &str,
    ) -> (crate::test_support::Folder, exhale_core::settings::Settings) {
        let folder   = crate::test_support::Folder::new(tag, Some(file));
        let settings = folder.manager.settings.read().expect("settings lock").clone();
        (folder, settings)
    }

    fn broken_at_load() -> StudyStatus {
        StudyStatus::Broken(StudyError::log_unavailable(&std::io::Error::other("x")))
    }

    // ── The Appearance card ───────────────────────────────────────

    /// One frame of the Appearance card on its own, with a button after it
    /// to count Tab stops against. Returns what it drew, whether it changed
    /// the settings, and the button's id
    fn run_appearance_frame(
        ctx:      &Context,
        raw_in:   RawInput,
        settings: &mut exhale_core::settings::Settings,
        status:   &StudyStatus,
    ) -> (Drawn, bool, egui::Id) {
        let (mut changed, mut next) = (false, egui::Id::NULL);
        let out = ctx.run(raw_in, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let column_w = picker_column_width(ui);
                changed = appearance_card(ui, settings, status, true, column_w);
                next = ui.button("Next").id;
            });
        });
        (Drawn::of(&out), changed, next)
    }

    /// For each row the study file locks, one text that only that row draws
    const LOCKED_ROWS: [(&str, &str); 9] = [
        ("Inhale Color",     "Inhale Color"),
        ("Exhale Color",     "Exhale Color"),
        ("Background Color", "Background Color"),
        ("Overlay Opacity",  "Overlay Opacity (%)"),
        ("Display",          "Display"),
        ("Shape",            "Shape"),
        // The row's own label is also an option of Hold Ripple
        ("Gradient",         "Inner"),
        ("Animation",        "Animation"),
        ("Hold Ripple",      "Hold Ripple"),
    ];

    /// The three options of the Show In row
    const SHOW_IN_OPTIONS: [&str; 3] = ["Top Bar", "Dock", "Both"];

    #[test]
    fn without_a_study_the_appearance_card_shows_every_row() {
        let ctx = Context::default();
        let mut open = exhale_core::settings::Settings::default();
        let (drawn, _, _) = run_appearance_frame(&ctx, tall_input(), &mut open, &StudyStatus::Off);

        assert_eq!(drawn.count("APPEARANCE"), 1, "{}", drawn.dump());
        for (row, text) in LOCKED_ROWS {
            assert_eq!(drawn.count(text), 1, "the {row} row is missing: {}", drawn.dump());
        }
        assert_eq!(drawn.count("Show In"), 1, "{}", drawn.dump());
        for option in SHOW_IN_OPTIONS {
            assert_eq!(drawn.count(option), 1, "{option}: {}", drawn.dump());
        }
    }

    #[test]
    fn while_the_pace_is_hidden_the_appearance_card_shows_show_in_alone() {
        // A locked study, and a broken one both ways: a file that failed
        // at load locks nothing, a log that failed after launch keeps the lock
        for (how, mut settings, status) in [
            ("locked",              locked_settings(),                          StudyStatus::Locked),
            ("broken at load",      exhale_core::settings::Settings::default(), broken_at_load()),
            ("broken after launch", locked_settings(),                          broken_at_load()),
        ] {
            let ctx = Context::default();
            let (drawn, _, _) = run_appearance_frame(&ctx, tall_input(), &mut settings, &status);

            assert_eq!(drawn.count("APPEARANCE"), 1, "{how}: the card is still there: {}", drawn.dump());
            for (row, text) in LOCKED_ROWS {
                assert_eq!(drawn.count(text), 0, "{how}: the {row} row is showing: {}", drawn.dump());
            }
            // The options of the hidden pickers, which labels alone could miss
            for option in ["Overlay", "Window", "Rectangle", "Circle", "Full", "Linear", "Sinusoidal", "Stark"] {
                assert_eq!(drawn.count(option), 0, "{how}: {option} is showing: {}", drawn.dump());
            }
            assert_eq!(drawn.count("Show In"), 1, "{how}: {}", drawn.dump());
            for option in SHOW_IN_OPTIONS {
                assert_eq!(drawn.count(option), 1, "{how}: {option}: {}", drawn.dump());
            }
        }
    }

    #[test]
    fn the_window_gives_the_appearance_card_the_study_status_it_read() {
        // The whole window, with the status coming from a manager as it does
        // at launch: a locked study and one that can't run both leave Show In
        for (tag, file) in [
            ("appearance-locked", crate::test_support::STUDY_FILE),
            ("appearance-broken", crate::test_support::BROKEN_STUDY_FILE),
        ] {
            let (folder, mut settings) = study_folder(tag, file);
            let ctx = Context::default();

            let drawn = run_window_frame(
                &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
            );

            assert_eq!(drawn.count("APPEARANCE"), 1, "{tag}: {}", drawn.dump());
            for (row, text) in LOCKED_ROWS {
                assert_eq!(drawn.count(text), 0, "{tag}: the {row} row is showing: {}", drawn.dump());
            }
            assert_eq!(drawn.count("Show In"), 1, "{tag}: {}", drawn.dump());
        }
    }

    #[test]
    fn show_in_still_takes_a_click_in_a_study_and_nothing_else_moves() {
        let ctx = Context::default();
        let mut settings = locked_settings();
        let mut want = settings.clone();
        want.app_visibility = AppVisibility::DockOnly;

        let (drawn, _, _) = run_appearance_frame(&ctx, tall_input(), &mut settings, &StudyStatus::Locked);
        let dock = drawn.rect("Dock");
        let (_, changed, _) = run_appearance_frame(
            &ctx, tall_click(dock.center()), &mut settings, &StudyStatus::Locked,
        );

        assert!(changed, "the one row a study leaves is still editable");
        assert_ne!(AppVisibility::DockOnly, locked_settings().app_visibility, "so the click had something to change");
        assert_eq!(format!("{settings:?}"), format!("{want:?}"), "and the lock is where the file put it");
    }

    #[test]
    fn tab_still_walks_the_controls_that_remain() {
        // Presses of Tab it takes to get from the top of the card to the
        // button after it
        let presses = |status: &StudyStatus| {
            let ctx = Context::default();
            let mut settings = locked_settings();
            let (_, _, next) = run_appearance_frame(&ctx, tall_input(), &mut settings, status);
            for presses in 1..=40 {
                let _ = run_appearance_frame(&ctx, tall_key(egui::Key::Tab), &mut settings, status);
                if ctx.memory(|m| m.focused()) == Some(next) {
                    return presses;
                }
            }
            usize::MAX
        };

        // Show In is three stops, one for each of its options, and the
        // button is the one after
        assert_eq!(presses(&StudyStatus::Locked), 4);
        assert_eq!(presses(&broken_at_load()), 4);
        // Each of the other rows is a stop too, which is what the study hides
        assert!(presses(&StudyStatus::Off) > 4);
    }

    // ── The cards under Appearance ────────────────────────────────

    #[test]
    fn a_locked_build_lays_out_no_preset_chips() {
        let ctx = Context::default();
        let mut locked = locked_settings();
        let drawn = run_body_cards_frame(
            &ctx, tall_input(), &mut locked, &StudyStatus::Locked, &None, &mut None,
        );
        assert!(
            super::widgets::test_hooks::take_chip_rects().is_none(),
            "study mode renders no Timing card, so no chips are laid out",
        );
        assert_eq!(drawn.count("STUDY"), 1, "the Study card still occupies the window");
        for hidden in ["TIMING", "RANDOMIZATION", "TIMERS"] {
            assert_eq!(drawn.count(hidden), 0, "{hidden} is hidden in a study");
        }
    }

    #[test]
    fn an_unlocked_build_lays_out_every_chip() {
        let ctx = Context::default();
        let mut open = exhale_core::settings::Settings::default();
        let drawn = run_body_cards_frame(
            &ctx, tall_input(), &mut open, &StudyStatus::Off, &None, &mut None,
        );
        let rects = super::widgets::test_hooks::take_chip_rects()
            .expect("a build without a study file lays out the Timing card");
        assert_eq!(rects.len(), exhale_core::presets::PRESETS.len());
        for shown in ["TIMING", "RANDOMIZATION", "TIMERS"] {
            assert_eq!(drawn.count(shown), 1, "{shown} shows when no study is involved");
        }
        assert_eq!(drawn.count("STUDY"), 0);
    }

    #[test]
    fn a_study_that_cannot_run_lays_out_none_of_the_pacing() {
        // Both ways a study breaks. A file that failed at load locks
        // nothing, so `settings.study` is empty and the pace is whatever
        // settings.toml held. A log that failed after launch keeps the lock
        for (how, mut settings) in [
            ("at load",       exhale_core::settings::Settings::default()),
            ("after launch",  locked_settings()),
        ] {
            let ctx = Context::default();
            let drawn = run_body_cards_frame(
                &ctx, tall_input(), &mut settings, &broken_at_load(), &None, &mut None,
            );
            assert!(
                super::widgets::test_hooks::take_chip_rects().is_none(),
                "{how}: no preset chips",
            );
            assert!(drawn.0.is_empty(), "{how}: nothing here at all, saw {}", drawn.dump());
        }
    }

    #[test]
    fn a_locked_build_keeps_the_pace_it_was_given() {
        // The card selection is the only blinding mechanism: prove the
        // durations the study file wrote are still there after a frame
        // that had no way to change them
        let ctx = Context::default();
        let mut locked = locked_settings();
        let before = (
            locked.inhale_duration,
            locked.post_inhale_hold_duration,
            locked.exhale_duration,
            locked.post_exhale_hold_duration,
            locked.drift,
        );
        let _ = run_body_cards_frame(
            &ctx, tall_input(), &mut locked, &StudyStatus::Locked, &None, &mut None,
        );
        assert_eq!(
            (
                locked.inhale_duration,
                locked.post_inhale_hold_duration,
                locked.exhale_duration,
                locked.post_exhale_hold_duration,
                locked.drift,
            ),
            before,
        );
        assert!(locked.study.is_some(), "and the lock survives a frame");
    }

    #[test]
    fn the_study_card_shows_the_participant_and_never_the_arm() {
        let ctx = Context::default();
        let mut locked = locked_settings();
        let drawn = run_body_cards_frame(
            &ctx, tall_input(), &mut locked, &StudyStatus::Locked, &None, &mut None,
        );
        assert_eq!(drawn.count("Participant"), 1, "{}", drawn.dump());
        assert!(drawn.says("P07"));
        assert!(!drawn.says("Condition"), "no condition row: {}", drawn.dump());
        assert!(!drawn.says("arm-blue-7"), "no condition code: {}", drawn.dump());
    }

    #[test]
    fn a_participant_id_as_long_as_the_study_file_allows_wraps_inside_the_card() {
        let ctx = Context::default();
        let mut locked = locked_settings();
        let id = "P".repeat(exhale_core::session_log::MAX_PARTICIPANT_ID_CHARS);
        locked.study.as_mut().expect("locked").participant_id = id.clone();
        let drawn = run_body_cards_frame(
            &ctx, tall_input(), &mut locked, &StudyStatus::Locked, &None, &mut None,
        );
        let label = drawn.rect("Participant");
        let value = drawn.rect(&id);
        assert!(
            value.max.x <= label.min.x + CARD_CONTENT_W + 0.5,
            "the id runs {} past the card's right edge",
            value.max.x - (label.min.x + CARD_CONTENT_W),
        );
        assert!(value.height() >= 2.0 * label.height(), "and takes several lines: {value:?}");
    }

    // ── Export and Clear ──────────────────────────────────────────

    /// A study card with a live log, in a folder of its own
    struct StudyCard {
        folder:   crate::test_support::Folder,
        log:      Arc<SessionLog>,
        settings: exhale_core::settings::Settings,
        note:     Option<StudyNote>,
        ctx:      Context,
    }

    impl StudyCard {
        fn new(tag: &str) -> Self {
            let (folder, settings) = study_folder(tag, crate::test_support::STUDY_FILE);
            let log = Arc::new(SessionLog::open(&folder.dir, "P07").expect("open the log"));
            log.record("app_start", &[]);
            Self { folder, log, settings, note: None, ctx: Context::default() }
        }

        fn frame(&mut self, raw_in: RawInput) -> Drawn {
            run_body_cards_frame(
                &self.ctx, raw_in, &mut self.settings, &StudyStatus::Locked,
                &Some(Arc::clone(&self.log)), &mut self.note,
            )
        }

        /// What an Export press leaves: the copy, and the card's note
        fn export(&mut self) {
            let path = self.log.export_to(std::slice::from_ref(&self.folder.dir), "P07").expect("export");
            self.note = Some(StudyNote::Exported(path));
        }

        fn live_lines(&self) -> usize {
            std::fs::read_to_string(self.log.path()).expect("read").lines().count()
        }
    }

    #[test]
    fn clear_is_offered_while_the_log_is_what_the_last_export_copied() {
        let mut card = StudyCard::new("clear-offered");

        // Before any export there's nothing to clear and nothing to say
        let drawn = card.frame(tall_input());
        assert_eq!(drawn.count("Export log"), 1);
        assert_eq!(drawn.count("Clear log"), 0);
        assert!(!drawn.says(CLEAR_WITHHELD_HINT));

        // Right after one the log is exactly what was copied
        card.export();
        let drawn = card.frame(tall_input());
        assert!(drawn.says("Exported to"), "{}", drawn.dump());
        assert_eq!(drawn.count("Clear log"), 1, "{}", drawn.dump());
        assert!(!drawn.says(CLEAR_WITHHELD_HINT));
    }

    #[test]
    fn clear_is_withheld_once_lines_arrive_and_the_card_says_to_export_again() {
        let mut card = StudyCard::new("clear-withheld");
        card.export();
        // A phase boundary: a line no export holds
        card.log.record("phase_end", &[]);

        let drawn = card.frame(tall_input());
        assert_eq!(drawn.count("Clear log"), 0, "{}", drawn.dump());
        assert!(drawn.says("Exported to"), "the card still says where the export went");
        assert_eq!(
            CLEAR_WITHHELD_HINT,
            "New lines arrived since the export. Stop, then export again to clear the log.",
            "the guide quotes these words",
        );
        // One line, at the width the card has
        card.note = Some(StudyNote::Cleared);
        let one_line = card.frame(tall_input()).rect("Log cleared.").height();
        card.note = Some(StudyNote::Exported(std::path::PathBuf::from("export.jsonl")));
        let hint = card.frame(tall_input()).rect(CLEAR_WITHHELD_HINT);
        assert!(hint.height() < 1.5 * one_line, "the hint takes more than a line: {hint:?}");

        // Exporting again covers the new line, and the card goes back
        card.export();
        let drawn = card.frame(tall_input());
        assert_eq!(drawn.count("Clear log"), 1, "{}", drawn.dump());
        assert!(!drawn.says(CLEAR_WITHHELD_HINT));
    }

    #[test]
    fn clearing_empties_the_log_and_says_nothing_about_new_lines() {
        let mut card = StudyCard::new("clear-click");
        card.export();
        let drawn = card.frame(tall_input());
        let button = drawn.rect("Clear log");

        let _ = card.frame(tall_click(button.center()));

        assert_eq!(card.note, Some(StudyNote::Cleared));
        assert_eq!(card.live_lines(), 1, "only the log_cleared header is left");
        let drawn = card.frame(tall_input());
        assert!(drawn.says("Log cleared."));
        assert_eq!(drawn.count("Clear log"), 0, "the next Clear needs a new export");
        assert!(!drawn.says(CLEAR_WITHHELD_HINT), "{}", drawn.dump());
    }

    #[test]
    fn a_long_export_path_and_the_hint_wrap_inside_the_card() {
        let mut card = StudyCard::new("wrap");
        // One line of the note's own type, for scale
        card.note = Some(StudyNote::Cleared);
        let one_line = card.frame(tall_input()).rect("Log cleared.").height();

        // A path with no spaces to break at, and the hint under it
        let long = std::path::PathBuf::from(format!(
            "/Users/{}/Desktop/exhale-log-P07-1700000000-12.jsonl",
            "a-very-long-account-name".repeat(6),
        ));
        card.note = Some(StudyNote::Exported(long));
        card.log.record("phase_end", &[]);
        let drawn = card.frame(tall_input());

        let left  = drawn.rect("Participant").min.x;
        let limit = left + CARD_CONTENT_W + 0.5;
        for (text, rect) in &drawn.0 {
            assert!(rect.max.x <= limit, "{text:?} runs to {} in a card that ends at {limit}", rect.max.x);
        }
        let path = drawn.0.iter().find(|(t, _)| t.starts_with("Exported to")).expect("the path line");
        assert!(path.1.height() >= 2.0 * one_line, "a path that long takes several lines: {:?}", path.1);
        assert!(drawn.says(CLEAR_WITHHELD_HINT), "{}", drawn.dump());

        // A failure's reason can be long as well, and has no spaces to break at
        let reason = format!("Export failed: {}", "x".repeat(200));
        card.note = Some(StudyNote::Failed(reason.clone()));
        let drawn = card.frame(tall_input());
        let failed = drawn.rect(&reason);
        assert!(failed.max.x <= limit, "the reason runs to {} in a card that ends at {limit}", failed.max.x);
        assert!(failed.height() >= 2.0 * one_line, "and takes several lines: {failed:?}");
    }

    #[test]
    fn a_failed_export_shows_its_reason_and_no_hint() {
        let mut card = StudyCard::new("failed");
        card.note = Some(StudyNote::Failed("Export failed: no folder to export to".to_string()));
        let drawn = card.frame(tall_input());
        assert!(drawn.says("Export failed: no folder to export to"));
        assert!(!drawn.says(CLEAR_WITHHELD_HINT));
    }

    // ── A study that can't run ────────────────────────────────────

    #[test]
    fn a_broken_study_file_shows_the_card_and_hides_every_pacing_surface() {
        let (folder, mut settings) = study_folder("broken-card", crate::test_support::BROKEN_STUDY_FILE);
        assert!(folder.manager.study_status().error().is_some());
        let ctx = Context::default();

        let drawn = run_window_frame(
            &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
        );

        // The card names the file, the line and the field
        assert!(
            drawn.says("Study can't start: study.toml, line 3, field inhale_duration:"),
            "{}", drawn.dump(),
        );
        assert!(drawn.says("Tell whoever runs the study."));
        // and never the value the lab typed, or the old second line
        assert!(!drawn.says("0.0731"), "{}", drawn.dump());
        assert!(!drawn.says("Pacing isn't locked"));
        // The one Study card is the error card: no participant, no arm
        assert_eq!(drawn.count("STUDY"), 1);
        assert!(!drawn.says("Participant"));
        assert!(!drawn.says("arm-blue-7"));
        for hidden in ["TIMING", "RANDOMIZATION", "TIMERS"] {
            assert_eq!(drawn.count(hidden), 0, "{hidden}");
        }
        assert!(super::widgets::test_hooks::take_chip_rects().is_none(), "no preset chips");
        let readout = exhale_core::pacing::readout_lines(&settings);
        assert!(!readout.is_empty(), "the pace in these settings has a readout to hide");
        for line in readout {
            assert!(!drawn.says(&line), "the rate readout {line:?} is on screen");
        }
    }

    #[test]
    fn a_misnamed_study_file_is_named_on_the_card_and_the_card_wraps() {
        // What Notepad leaves when the extension is hidden, with a name long
        // enough that the card's sentence has to wrap
        let misnamed = format!("study ({}).toml.txt", "copy of the lab's file ".repeat(4));
        let folder = crate::test_support::Folder::with_files("misnamed", &[(&misnamed, "x")]);
        assert!(folder.manager.study_status().error().is_some());
        let mut settings = folder.manager.settings.read().expect("lock").clone();
        let ctx = Context::default();

        let drawn = run_window_frame(
            &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
        );

        assert!(drawn.says(&misnamed), "the card names the file it found: {}", drawn.dump());
        assert!(drawn.says("Rename it to study.toml"), "{}", drawn.dump());
        assert!(drawn.says("Tell whoever runs the study."));
        let card_right = SETTINGS_WIDTH as f32 - OUTER_PAD - CARD_PAD;
        let line = drawn.0.iter().find(|(t, _)| t.starts_with("Study can't start")).expect("the card's sentence");
        assert!(line.1.max.x <= card_right + 0.5, "it runs to {} in a card that ends at {card_right}", line.1.max.x);
        let tell = drawn.rect("Tell whoever runs the study.");
        assert!(line.1.height() >= 2.0 * tell.height(), "and it takes several lines: {:?}", line.1);
    }

    #[test]
    fn a_log_that_will_not_open_shows_the_same_card_with_the_lock_kept() {
        let (folder, _) = study_folder("log-card", crate::test_support::STUDY_FILE);
        std::fs::create_dir(folder.dir.join("exhale-log-P07.jsonl")).expect("block the log");
        assert!(crate::open_session_log(&folder.manager).is_none());
        let mut settings = folder.manager.settings.read().expect("lock").clone();
        assert!(settings.study.is_some(), "the lock is in force");
        let ctx = Context::default();

        let drawn = run_window_frame(
            &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
        );

        assert!(drawn.says("Study can't start: study.toml: the session log can't be written"), "{}", drawn.dump());
        assert!(drawn.says("Tell whoever runs the study."));
        assert_eq!(drawn.count("STUDY"), 1);
        assert!(!drawn.says("Participant"), "{}", drawn.dump());
        for hidden in ["TIMING", "RANDOMIZATION", "TIMERS"] {
            assert_eq!(drawn.count(hidden), 0, "{hidden}");
        }
        assert!(!drawn.says("Export log"), "there's no log to export");
    }

    #[test]
    fn start_does_nothing_while_the_study_cannot_run_and_works_when_it_can() {
        // Broken: Start's click lands on a button that takes none. Reset
        // is clicked first, which proves the click position is a live one
        let (folder, mut settings) = study_folder("start-broken", crate::test_support::BROKEN_STUDY_FILE);
        let ctx = Context::default();
        let mut pending = false;
        let drawn = run_window_frame(
            &ctx, tall_input(), &mut settings, &folder.manager, &mut pending, &None, &mut None,
        );
        let (start, reset) = (drawn.rect("Start"), drawn.rect("Reset"));

        let _ = run_window_frame(
            &ctx, tall_click(reset.center()), &mut settings, &folder.manager, &mut pending, &None, &mut None,
        );
        assert!(pending, "the Reset button beside it takes the click");
        let _ = run_window_frame(
            &ctx, tall_click(start.center()), &mut settings, &folder.manager, &mut pending, &None, &mut None,
        );
        assert!(!settings.is_animating, "Start does nothing");

        // Locked and stopped: the same click starts it
        let (folder, mut settings) = study_folder("start-locked", crate::test_support::STUDY_FILE);
        settings.is_animating = false;
        let ctx = Context::default();
        let drawn = run_window_frame(
            &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
        );
        let _ = run_window_frame(
            &ctx, tall_click(drawn.rect("Start").center()), &mut settings, &folder.manager, &mut false, &None, &mut None,
        );
        assert!(settings.is_animating, "Start works in a study that can run");
    }

    #[test]
    fn tab_goes_past_a_disabled_start_to_stop() {
        // Tab then Enter presses the first button Tab reaches. Running
        // settings make Stop's press visible, and stopped ones make Start's
        for (tag, file, running_at_first) in [
            ("tab-broken", crate::test_support::BROKEN_STUDY_FILE, true),
            ("tab-locked", crate::test_support::STUDY_FILE,        false),
        ] {
            let (folder, mut settings) = study_folder(tag, file);
            settings.is_animating = running_at_first;
            let ctx = Context::default();
            let mut frame = |raw_in: RawInput| {
                run_window_frame(&ctx, raw_in, &mut settings, &folder.manager, &mut false, &None, &mut None)
            };
            let _ = frame(tall_input());
            let _ = frame(tall_key(egui::Key::Tab));
            let _ = frame(tall_key(egui::Key::Enter));
            // Stop's press stops a running animation, and Start's starts a
            // stopped one, so the result says which button took the press
            if running_at_first {
                assert!(!settings.is_animating, "{tag}: Tab reached Stop, since Start takes no focus");
            } else {
                assert!(settings.is_animating, "{tag}: with Start enabled it's the first stop");
            }
        }
    }

    #[test]
    fn disabling_start_moves_no_button() {
        let columns = |file: &str, tag: &str| {
            let (folder, mut settings) = study_folder(tag, file);
            let ctx = Context::default();
            let _ = run_window_frame(
                &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
            );
            let drawn = run_window_frame(
                &ctx, tall_input(), &mut settings, &folder.manager, &mut false, &None, &mut None,
            );
            ["Start", "Stop", "Reset", "Quit"].map(|b| {
                let r = drawn.rect(b);
                (r.min.x, r.width())
            })
        };
        let enabled  = columns(crate::test_support::STUDY_FILE, "cols-enabled");
        let disabled = columns(crate::test_support::BROKEN_STUDY_FILE, "cols-disabled");
        for (i, (a, b)) in enabled.iter().zip(&disabled).enumerate() {
            assert!((a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01, "button {i}: {a:?} vs {b:?}");
        }
    }

    // ── Reset ─────────────────────────────────────────────────────

    #[test]
    fn outside_a_study_reset_says_what_it_always_said() {
        let said = reset_text(false);
        assert_eq!(said.help, "Reset all settings to their default values.");
        assert_eq!(said.title, "Reset all settings to their defaults?");
        assert_eq!(
            said.detail,
            "This restores every appearance, timing, and keyboard-shortcut setting. Can't be undone.",
        );
    }

    #[test]
    fn in_a_study_reset_says_it_restores_the_shortcuts_and_show_in_only() {
        let said = reset_text(true);
        for text in [said.help, said.title, said.detail] {
            assert!(text.contains("keyboard shortcuts") && text.contains("Show In"), "{text}");
            for untrue in ["every appearance", "timing", "all settings"] {
                assert!(!text.contains(untrue), "{text:?} says {untrue:?}");
            }
        }
        assert!(said.detail.contains("locks the pace and the appearance"), "{}", said.detail);
    }

    #[test]
    fn the_confirmation_card_uses_the_words_that_are_true_for_the_settings() {
        for (tag, file, study) in [
            ("reset-text-study", Some(crate::test_support::STUDY_FILE), true),
            ("reset-text-open",  None,                                  false),
        ] {
            let folder   = crate::test_support::Folder::new(tag, file);
            let mut settings = folder.manager.settings.read().expect("lock").clone();
            assert_eq!(settings.study.is_some(), study);
            let ctx = Context::default();

            let drawn = run_window_frame(
                &ctx, tall_input(), &mut settings, &folder.manager, &mut true, &None, &mut None,
            );

            let (said, other) = (reset_text(study), reset_text(!study));
            assert!(drawn.says(said.title), "{}", drawn.dump());
            assert!(drawn.says(said.detail), "{}", drawn.dump());
            assert!(!drawn.says(other.title) && !drawn.says(other.detail), "{}", drawn.dump());
        }
    }

    #[test]
    fn confirming_reset_in_a_study_logs_it_keeps_the_lock_and_restores_show_in() {
        let folder = crate::test_support::Folder::new("reset-click", Some(crate::test_support::STUDY_FILE));
        let log = crate::open_session_log(&folder.manager).expect("the log opens");
        let mut settings = folder.manager.settings.read().expect("lock").clone();
        let locked = format!("{settings:?}");
        // The one thing a study leaves to the participant that the panel
        // shows here
        settings.app_visibility = AppVisibility::DockOnly;
        let some_log = Some(Arc::clone(&log));
        let ctx = Context::default();
        let mut pending = true;

        let drawn = run_window_frame(
            &ctx, tall_input(), &mut settings, &folder.manager, &mut pending, &some_log, &mut None,
        );
        // Two "Reset" labels: the control button above and the card's own
        // confirm button below it
        assert_eq!(drawn.count("Reset"), 2, "{}", drawn.dump());
        let confirm = drawn.lowest("Reset");
        let _ = run_window_frame(
            &ctx, tall_click(confirm.center()), &mut settings, &folder.manager, &mut pending, &some_log, &mut None,
        );

        assert!(!pending, "the card is closed");
        assert_eq!(crate::test_support::events(&log), ["app_start", "reset"]);
        let written = std::fs::read_to_string(log.path()).expect("read");
        assert!(written.lines().last().expect("a line").ends_with("\"event\":\"reset\"}"), "no fields: {written}");
        assert_eq!(
            format!("{settings:?}"), locked,
            "Show In is back, and the pace and the look are the file's",
        );
    }

    #[test]
    fn stepper_up_increments() {
        let ctx = Context::default();
        let mut value = 5.0_f64;
        // simulate_click_on_stepper runs a warmup frame (registers the
        // stepper's interact rects in egui memory) followed by a click
        // frame.  Exactly one click per invocation
        let changed = simulate_click_on_stepper(&ctx, &mut value, true);
        assert!(changed, "UP click should change the value");
        assert_eq!(value, 6.0, "UP click should increment by step=1.0 (5 -> 6)");
    }

    #[test]
    fn stepper_down_decrements() {
        let ctx = Context::default();
        let mut value = 5.0_f64;
        let changed = simulate_click_on_stepper(&ctx, &mut value, false);
        assert!(changed, "DOWN click should change the value");
        assert_eq!(value, 4.0, "DOWN click should decrement by step=1.0 (5 -> 4)");
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
        // crashes": drive ~50 alternating up/down clicks and make sure
        // nothing panics and the value stays finite
        let ctx = Context::default();
        let mut value = 10.0_f64;
        for i in 0..50 {
            let _ = simulate_click_on_stepper(&ctx, &mut value, i % 2 == 0);
        }
        assert!(value.is_finite(), "value should remain finite across 50 alternating clicks");
    }

    #[test]
    fn stepper_repeated_ups_accumulate() {
        // Each call to simulate_click_on_stepper performs one click,
        // so N invocations should give N increments (5 + 3 = 8)
        let ctx = Context::default();
        let mut value = 5.0_f64;
        for _ in 0..3 {
            let _ = simulate_click_on_stepper(&ctx, &mut value, true);
        }
        assert_eq!(value, 8.0, "three UP clicks should give 5 + 3*1 = 8");
    }
}
