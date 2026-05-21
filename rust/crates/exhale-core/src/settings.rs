use serde::{Deserialize, Serialize};

use crate::types::{
    AnimationMode, AnimationShape, AppVisibility, ColorFillGradient, HoldRippleMode,
};

/// All user-configurable settings for the exhale app.
///
/// Matches the Swift `SettingsModel` exactly in field names and default values.
/// Stored as TOML in the platform config directory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    // ── Appearance ────────────────────────────────────────────────────────────
    /// Linear RGBA of the inhale color. Alpha is display alpha (not premultiplied).
    pub inhale_color: [f32; 4],
    /// Linear RGBA of the exhale color.
    pub exhale_color: [f32; 4],
    /// Linear RGBA of the background tint. Alpha controls background opacity.
    pub background_color: [f32; 4],

    /// Master opacity of the overlay (0.0–1.0).
    pub overlay_opacity: f32,

    pub shape: AnimationShape,
    pub color_fill_gradient: ColorFillGradient,
    pub animation_mode: AnimationMode,
    pub hold_ripple_mode: HoldRippleMode,
    pub app_visibility: AppVisibility,

    // ── Timing (seconds) ─────────────────────────────────────────────────────
    pub inhale_duration: f64,
    pub post_inhale_hold_duration: f64,
    pub exhale_duration: f64,
    pub post_exhale_hold_duration: f64,

    /// Per-cycle duration multiplier. 1.01 = each cycle 1 % longer (drift).
    pub drift: f64,

    // ── Randomisation (±seconds of jitter per phase) ─────────────────────────
    pub randomized_timing_inhale: f64,
    pub randomized_timing_post_inhale_hold: f64,
    pub randomized_timing_exhale: f64,
    pub randomized_timing_post_exhale_hold: f64,

    // ── Timers ────────────────────────────────────────────────────────────────
    /// Minutes between "Remember to breathe" notifications. 0 = off.
    pub reminder_interval_minutes: f64,
    /// Stop animation after this many minutes. 0 = off.
    pub auto_stop_minutes: f64,

    // ── State ─────────────────────────────────────────────────────────────────
    pub is_animating: bool,
    /// Pause holds the animation on the current frame without resetting position.
    pub is_paused: bool,

    // ── Window frame (persisted so each window reopens where you left it) ──
    //
    // Stored as an offset relative to a named monitor (matching Swift's
    // per-screen persistence in AppDelegate.windowDidMove).  When the saved
    // monitor is no longer connected we clamp the position to whichever
    // monitor still has visible real estate rather than restoring to
    // off-screen coordinates.  Both windows use the same shape, see
    // [`WindowPlacement`] / [`Settings::settings_window_placement`] /
    // [`Settings::animation_window_placement`].
    //
    // The settings window has a fixed width; only its height is
    // persisted (`settings_window_width` doesn't exist).  The animation
    // window persists both dimensions because the user can resize it
    // freely.
    #[serde(default)]
    pub settings_window_x: Option<i32>,
    #[serde(default)]
    pub settings_window_y: Option<i32>,
    #[serde(default)]
    pub settings_window_height: Option<u32>,
    #[serde(default)]
    pub settings_window_screen: Option<String>,

    #[serde(default)]
    pub animation_window_x: Option<i32>,
    #[serde(default)]
    pub animation_window_y: Option<i32>,
    #[serde(default)]
    pub animation_window_width: Option<u32>,
    #[serde(default)]
    pub animation_window_height: Option<u32>,
    #[serde(default)]
    pub animation_window_screen: Option<String>,

    // ── User-customisable global hotkeys ─────────────────────────────────────
    /// Per-action global keyboard shortcuts.  Users right-click the
    /// matching button in the settings window to change one;
    /// "Reset to Defaults" restores every shortcut to its
    /// [`KeyboardShortcuts::default`] value.  Stored at the bottom of
    /// the file with `#[serde(default)]` so older `settings.toml`
    /// files without this section still load and pick up the defaults
    #[serde(default)]
    pub keyboard_shortcuts: KeyboardShortcuts,
}

// ── KeyboardShortcut bitmask constants ────────────────────────────────────────
//
// Used as a packed `u8` instead of a `bitflags` type so the on-disk
// TOML representation is a simple integer the user can sanity-check
// without consulting documentation.  Bits chosen to match the order
// global-hotkey's `Modifiers` flags use internally
/// Control / `^`
pub const KBD_MOD_CTRL:  u8 = 1 << 0;
/// Shift / `⇧`
pub const KBD_MOD_SHIFT: u8 = 1 << 1;
/// Alt / Option / `⌥`
pub const KBD_MOD_ALT:   u8 = 1 << 2;
/// Meta / Command / Windows key / `⌘`
pub const KBD_MOD_META:  u8 = 1 << 3;

/// A single global-hotkey binding.  `code` is the string form of
/// [`keyboard_types::Code`] (e.g. `"KeyA"`, `"Comma"`, `"Digit1"`) so
/// the serialised representation stays stable across crate-version
/// bumps that might re-number the underlying enum
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    /// Bitmask of `KBD_MOD_*`.
    pub modifiers: u8,
    /// `keyboard_types::Code` variant name.
    pub code:      String,
}

impl KeyboardShortcut {
    pub fn new(modifiers: u8, code: impl Into<String>) -> Self {
        Self { modifiers, code: code.into() }
    }

    pub fn ctrl_shift(code: impl Into<String>) -> Self {
        Self::new(KBD_MOD_CTRL | KBD_MOD_SHIFT, code)
    }

    pub fn has_ctrl(&self)  -> bool { self.modifiers & KBD_MOD_CTRL  != 0 }
    pub fn has_shift(&self) -> bool { self.modifiers & KBD_MOD_SHIFT != 0 }
    pub fn has_alt(&self)   -> bool { self.modifiers & KBD_MOD_ALT   != 0 }
    pub fn has_meta(&self)  -> bool { self.modifiers & KBD_MOD_META  != 0 }

    /// Human-readable rendering for tooltips and capture prompts.
    /// macOS users see the standard glyph triad (`⌃⇧⌥⌘`); other
    /// platforms get textual `Ctrl+Shift+...` so the string remains
    /// legible in any font.  The trailing key strips the `Key` /
    /// `Digit` prefixes that come from `keyboard_types::Code`'s
    /// variant names
    pub fn display(&self) -> String {
        #[cfg(target_os = "macos")]
        let (ctrl, shift, alt, meta) = ("\u{2303}", "\u{21E7}", "\u{2325}", "\u{2318}");
        #[cfg(not(target_os = "macos"))]
        let (ctrl, shift, alt, meta) = ("Ctrl", "Shift", "Alt", "Meta");

        let mut parts: Vec<String> = Vec::new();
        if self.has_ctrl()  { parts.push(ctrl.into()); }
        if self.has_alt()   { parts.push(alt.into()); }
        if self.has_shift() { parts.push(shift.into()); }
        if self.has_meta()  { parts.push(meta.into()); }
        parts.push(human_key(&self.code));
        #[cfg(target_os = "macos")]
        { parts.join("") }
        #[cfg(not(target_os = "macos"))]
        { parts.join("+") }
    }
}

fn human_key(code: &str) -> String {
    if let Some(rest) = code.strip_prefix("Key")   { return rest.to_string(); }
    if let Some(rest) = code.strip_prefix("Digit") { return rest.to_string(); }
    match code {
        "Comma"        => ",".into(),
        "Period"       => ".".into(),
        "Slash"        => "/".into(),
        "Backslash"    => "\\".into(),
        "Semicolon"    => ";".into(),
        "Quote"        => "'".into(),
        "Backquote"    => "`".into(),
        "Minus"        => "-".into(),
        "Equal"        => "=".into(),
        "BracketLeft"  => "[".into(),
        "BracketRight" => "]".into(),
        "Space"        => "Space".into(),
        "Enter"        => "Enter".into(),
        "Tab"          => "Tab".into(),
        "Escape"       => "Esc".into(),
        "Backspace"    => "Backspace".into(),
        "ArrowLeft"    => "←".into(),
        "ArrowRight"   => "→".into(),
        "ArrowUp"      => "↑".into(),
        "ArrowDown"    => "↓".into(),
        _              => code.into(),
    }
}

/// Per-action global-hotkey defaults.  Bound to S/D/F for the main
/// triad (Start / Stop / Reset), Q for Quit, and W for Preferences.
/// The triad sits on the home row to favour muscle memory; A is
/// avoided because Ctrl+Shift+A is captured by enough other apps
/// (browsers, Teams, etc.) that the global registration looked
/// silent on at least one tested macOS environment.  `W` over
/// Cmd-style `,` because comma is also reported unreliable on
/// macOS.  `Q` over Cmd+Q because Cmd+Q is the system-wide "quit
/// current app" shortcut on macOS — registering it globally would
/// steal it from every other app.  `Ctrl+Shift+Q` is rarely used
/// globally on macOS / Windows / Linux
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardShortcuts {
    #[serde(default = "KeyboardShortcuts::default_start")]
    pub start:       KeyboardShortcut,
    #[serde(default = "KeyboardShortcuts::default_stop")]
    pub stop:        KeyboardShortcut,
    #[serde(default = "KeyboardShortcuts::default_reset")]
    pub reset:       KeyboardShortcut,
    #[serde(default = "KeyboardShortcuts::default_quit")]
    pub quit:        KeyboardShortcut,
    #[serde(default = "KeyboardShortcuts::default_preferences")]
    pub preferences: KeyboardShortcut,
}

impl KeyboardShortcuts {
    pub fn default_start()       -> KeyboardShortcut { KeyboardShortcut::ctrl_shift("KeyS") }
    pub fn default_stop()        -> KeyboardShortcut { KeyboardShortcut::ctrl_shift("KeyD") }
    pub fn default_reset()       -> KeyboardShortcut { KeyboardShortcut::ctrl_shift("KeyF") }
    pub fn default_quit()        -> KeyboardShortcut { KeyboardShortcut::ctrl_shift("KeyQ") }
    pub fn default_preferences() -> KeyboardShortcut { KeyboardShortcut::ctrl_shift("KeyW") }
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            start:       Self::default_start(),
            stop:        Self::default_stop(),
            reset:       Self::default_reset(),
            quit:        Self::default_quit(),
            preferences: Self::default_preferences(),
        }
    }
}

/// Names every per-action slot in [`KeyboardShortcuts`] so callers in
/// `exhale-app` (UI capture state, dispatcher) can refer to a slot
/// without owning the matching `KeyboardShortcut`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    Start,
    Stop,
    Reset,
    Quit,
    Preferences,
}

impl ShortcutAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start       => "Start",
            Self::Stop        => "Stop",
            Self::Reset       => "Reset",
            Self::Quit        => "Quit",
            Self::Preferences => "Preferences",
        }
    }
}

impl KeyboardShortcuts {
    pub fn get(&self, action: ShortcutAction) -> &KeyboardShortcut {
        match action {
            ShortcutAction::Start       => &self.start,
            ShortcutAction::Stop        => &self.stop,
            ShortcutAction::Reset       => &self.reset,
            ShortcutAction::Quit        => &self.quit,
            ShortcutAction::Preferences => &self.preferences,
        }
    }

    pub fn set(&mut self, action: ShortcutAction, sc: KeyboardShortcut) {
        match action {
            ShortcutAction::Start       => self.start       = sc,
            ShortcutAction::Stop        => self.stop        = sc,
            ShortcutAction::Reset       => self.reset       = sc,
            ShortcutAction::Quit        => self.quit        = sc,
            ShortcutAction::Preferences => self.preferences = sc,
        }
    }

    pub fn reset_to_default(&mut self, action: ShortcutAction) {
        let default = match action {
            ShortcutAction::Start       => Self::default_start(),
            ShortcutAction::Stop        => Self::default_stop(),
            ShortcutAction::Reset       => Self::default_reset(),
            ShortcutAction::Quit        => Self::default_quit(),
            ShortcutAction::Preferences => Self::default_preferences(),
        };
        self.set(action, default);
    }
}

/// Persisted-position view of either application window.  Acts as a
/// shared shape between [`Settings::settings_window_placement`] /
/// [`Settings::animation_window_placement`] so the apply-on-create
/// and capture-on-move logic in `exhale-app::placement` is a single
/// helper used by both windows.  Not directly serialized — the flat
/// `*_window_*` fields above are the on-disk format, kept flat for
/// backward compatibility with existing settings.toml files
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowPlacement {
    pub x:      Option<i32>,
    pub y:      Option<i32>,
    pub width:  Option<u32>,
    pub height: Option<u32>,
    pub screen: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            inhale_color:    [1.0, 0.0, 0.0, 1.0], // red
            exhale_color:    [0.0, 0.0, 1.0, 1.0], // blue
            background_color:[0.0, 0.0, 0.0, 0.0], // clear

            overlay_opacity: 0.25,

            shape:               AnimationShape::Rectangle,
            color_fill_gradient: ColorFillGradient::On,
            animation_mode:      AnimationMode::Sinusoidal,
            hold_ripple_mode:    HoldRippleMode::Gradient,
            // Show both the menu-bar / tray icon AND the Dock / taskbar
            // entry by default — users new to the app are more likely to
            // notice it in the Dock, and discovering the tray-only mode
            // via Preferences once is easy.
            app_visibility:      AppVisibility::Both,

            inhale_duration:           5.0,
            post_inhale_hold_duration: 0.0,
            exhale_duration:           10.0,
            post_exhale_hold_duration: 0.0,
            drift:                     1.01,

            randomized_timing_inhale:             0.0,
            randomized_timing_post_inhale_hold:   0.0,
            randomized_timing_exhale:             0.0,
            randomized_timing_post_exhale_hold:   0.0,

            reminder_interval_minutes: 0.0,
            auto_stop_minutes:         0.0,

            is_animating: true,
            is_paused:    false,

            settings_window_x:      None,
            settings_window_y:      None,
            settings_window_height: None,
            settings_window_screen: None,

            animation_window_x:      None,
            animation_window_y:      None,
            animation_window_width:  None,
            animation_window_height: None,
            animation_window_screen: None,

            keyboard_shortcuts: KeyboardShortcuts::default(),
        }
    }
}

impl Settings {
    /// Read the persisted placement of the settings window.  Width is
    /// always `None` because the settings window is fixed-width
    pub fn settings_window_placement(&self) -> WindowPlacement {
        WindowPlacement {
            x:      self.settings_window_x,
            y:      self.settings_window_y,
            width:  None,
            height: self.settings_window_height,
            screen: self.settings_window_screen.clone(),
        }
    }

    /// Write the persisted placement of the settings window.  Width
    /// is ignored because the settings window is fixed-width.
    pub fn set_settings_window_placement(&mut self, p: WindowPlacement) {
        self.settings_window_x      = p.x;
        self.settings_window_y      = p.y;
        self.settings_window_height = p.height;
        self.settings_window_screen = p.screen;
    }

    /// Read the persisted placement of the animation (windowed-mode
    /// fallback) window.  Used only on platforms where the breath
    /// animation runs as a regular app window instead of as a
    /// fullscreen click-through overlay (currently Wayland and any
    /// GPU path that exposes only Opaque alpha)
    pub fn animation_window_placement(&self) -> WindowPlacement {
        WindowPlacement {
            x:      self.animation_window_x,
            y:      self.animation_window_y,
            width:  self.animation_window_width,
            height: self.animation_window_height,
            screen: self.animation_window_screen.clone(),
        }
    }

    /// Write the persisted placement of the animation window.
    pub fn set_animation_window_placement(&mut self, p: WindowPlacement) {
        self.animation_window_x      = p.x;
        self.animation_window_y      = p.y;
        self.animation_window_width  = p.width;
        self.animation_window_height = p.height;
        self.animation_window_screen = p.screen;
    }
}

/// Categorised diff between two `Settings` snapshots.  Adding a new
/// setting is a one-line edit to the relevant `*_changed` computation
/// here rather than coordinated edits across `main.rs`.
///
/// All `*_changed` fields are inclusive: they're `true` whenever ANY
/// field in their category differs.  Categories match the downstream
/// actions:
///
///   - `animating_changed`: drives tray-state refresh + auto-stop reschedule
///   - `paused_changed`: drives overlay redraw
///   - `visibility_changed` / `new_visibility`: drives platform Dock/menu-bar toggle
///   - `visual_changed`: drives Swift-parity `triggerAnimationReset()`
///   - `timing_changed`: same as visual_changed for timing-related fields
///   - `auto_stop_changed`: reschedules the auto-stop deadline
///   - `reminder_changed`: reschedules the reminder timer + may prompt for permission
#[derive(Clone, Copy, Debug)]
pub struct SettingsDiff {
    pub animating_started:  bool,
    pub animating_changed:  bool,
    pub paused_changed:     bool,
    pub visibility_changed: bool,
    pub new_visibility:     crate::types::AppVisibility,
    pub visual_changed:     bool,
    pub timing_changed:     bool,
    pub auto_stop_changed:  bool,
    pub reminder_changed:   bool,
}

impl SettingsDiff {
    /// Compute the diff between `before` (snapshot taken pre-render)
    /// and `after` (settings as mutated by the egui frame).  All
    /// float comparisons use an epsilon — 1e-9 for timing values
    /// (seconds) and 1e-4 for the [0,1] overlay opacity, both well
    /// below user-perceptible differences
    pub fn from(before: &Settings, after: &Settings) -> Self {
        let animating_changed = after.is_animating != before.is_animating;
        let animating_started = !before.is_animating && after.is_animating;
        let visual_changed = after.shape               != before.shape
            || after.color_fill_gradient != before.color_fill_gradient
            || after.animation_mode      != before.animation_mode
            || after.inhale_color        != before.inhale_color
            || after.exhale_color        != before.exhale_color
            || after.background_color    != before.background_color
            || (after.overlay_opacity - before.overlay_opacity).abs() > 1e-4;
        let timing_changed =
            (after.inhale_duration                       - before.inhale_duration).abs()           > 1e-9
         || (after.post_inhale_hold_duration             - before.post_inhale_hold_duration).abs() > 1e-9
         || (after.exhale_duration                       - before.exhale_duration).abs()           > 1e-9
         || (after.post_exhale_hold_duration             - before.post_exhale_hold_duration).abs() > 1e-9
         || (after.drift                                 - before.drift).abs()                     > 1e-9
         || (after.randomized_timing_inhale              - before.randomized_timing_inhale).abs()              > 1e-9
         || (after.randomized_timing_post_inhale_hold    - before.randomized_timing_post_inhale_hold).abs()    > 1e-9
         || (after.randomized_timing_exhale              - before.randomized_timing_exhale).abs()              > 1e-9
         || (after.randomized_timing_post_exhale_hold    - before.randomized_timing_post_exhale_hold).abs()    > 1e-9
         || after.hold_ripple_mode  != before.hold_ripple_mode;

        Self {
            animating_started,
            animating_changed,
            paused_changed:     after.is_paused      != before.is_paused,
            visibility_changed: after.app_visibility != before.app_visibility,
            new_visibility:     after.app_visibility,
            visual_changed,
            timing_changed,
            auto_stop_changed:  (after.auto_stop_minutes        - before.auto_stop_minutes).abs()        > 1e-9,
            reminder_changed:   (after.reminder_interval_minutes - before.reminder_interval_minutes).abs() > 1e-9,
        }
    }

    /// Any setting that should trigger an overlay redraw.
    pub fn should_redraw_overlay(&self) -> bool {
        self.paused_changed || self.animating_changed || self.visual_changed || self.timing_changed
    }

    /// Any setting that, per Swift's `triggerAnimationReset()`,
    /// should restart the animation from inhale phase 0 (only when
    /// the animation is actively running and not paused).
    pub fn should_restart_animation(&self, current: &Settings) -> bool {
        self.animating_started
            || ((self.visual_changed || self.timing_changed)
                && current.is_animating
                && !current.is_paused)
    }
}

impl Settings {
    /// Reset every user-visible setting to its default while preserving
    /// runtime state (animating / paused) and the persisted settings-
    /// window placement.  Single source of truth for the Reset button,
    /// the Reset confirmation dialog, and the global-hotkey reset path
    pub fn reset_preserving_runtime_state(&mut self) {
        let was_animating  = self.is_animating;
        let was_paused     = self.is_paused;
        let win_x          = self.settings_window_x;
        let win_y          = self.settings_window_y;
        let win_h          = self.settings_window_height;
        let win_screen     = self.settings_window_screen.take();
        *self = Settings::default();
        self.is_animating            = was_animating;
        self.is_paused               = was_paused;
        self.settings_window_x       = win_x;
        self.settings_window_y       = win_y;
        self.settings_window_height  = win_h;
        self.settings_window_screen  = win_screen;
    }

    /// Returns true when inhale and exhale colors are perceptually
    /// identical, used to skip unnecessary redraws in the fullscreen shape
    pub fn inhale_exhale_colors_match(&self) -> bool {
        self.inhale_color
            .iter()
            .zip(self.exhale_color.iter())
            .all(|(a, b)| (a - b).abs() < 0.001)
    }

    /// Background colour stripped of its own alpha — used as the shape background.
    pub fn background_color_rgb(&self) -> [f32; 4] {
        let [r, g, b, _a] = self.background_color;
        [r, g, b, 1.0]
    }

    /// The alpha component of background_color, clamped to overlay_opacity.
    pub fn background_opacity(&self) -> f32 {
        self.background_color[3].min(self.overlay_opacity)
    }

    /// Rectangle scale factor: 2× when gradient is `On` (extends the fill above 100 %).
    pub fn rectangle_scale(&self) -> f32 {
        if self.color_fill_gradient == ColorFillGradient::On { 2.0 } else { 1.0 }
    }

    /// Circle gradient scale: same factor as rectangle.
    pub fn circle_gradient_scale(&self) -> f32 {
        if self.color_fill_gradient == ColorFillGradient::On { 2.0 } else { 1.0 }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_swift_app() {
        let s = Settings::default();
        assert_eq!(s.inhale_duration, 5.0);
        assert_eq!(s.exhale_duration, 10.0);
        assert_eq!(s.post_inhale_hold_duration, 0.0);
        assert_eq!(s.post_exhale_hold_duration, 0.0);
        assert!((s.drift - 1.01).abs() < 1e-9);
        assert!((s.overlay_opacity - 0.25).abs() < 1e-6);
        assert_eq!(s.shape, AnimationShape::Rectangle);
        assert_eq!(s.color_fill_gradient, ColorFillGradient::On);
        assert_eq!(s.animation_mode, AnimationMode::Sinusoidal);
        assert_eq!(s.hold_ripple_mode, HoldRippleMode::Gradient);
        assert!(s.is_animating);
    }

    #[test]
    fn toml_round_trip() {
        let original = Settings::default();
        let serialized = toml::to_string(&original).expect("serialise");
        let deserialized: Settings = toml::from_str(&serialized).expect("deserialise");
        assert_eq!(deserialized.inhale_duration, original.inhale_duration);
        assert_eq!(deserialized.shape, original.shape);
        assert_eq!(deserialized.hold_ripple_mode, original.hold_ripple_mode);
    }

    #[test]
    fn inhale_exhale_colors_match_identical() {
        let mut s = Settings::default();
        s.inhale_color = [0.5, 0.5, 0.5, 1.0];
        s.exhale_color = [0.5, 0.5, 0.5, 1.0];
        assert!(s.inhale_exhale_colors_match());
    }

    #[test]
    fn inhale_exhale_colors_do_not_match_different() {
        let s = Settings::default();
        assert!(!s.inhale_exhale_colors_match());
    }

    #[test]
    fn rectangle_scale_on_gradient() {
        let mut s = Settings::default();
        s.color_fill_gradient = ColorFillGradient::On;
        assert_eq!(s.rectangle_scale(), 2.0);
        s.color_fill_gradient = ColorFillGradient::Off;
        assert_eq!(s.rectangle_scale(), 1.0);
    }

    #[test]
    fn background_opacity_clamped_to_overlay_opacity() {
        let mut s = Settings::default();
        s.background_color = [0.0, 0.0, 0.0, 0.8];
        s.overlay_opacity = 0.25;
        assert!((s.background_opacity() - 0.25).abs() < 1e-6);
    }

    // ── SettingsDiff ────────────────────────────────────────────────

    #[test]
    fn diff_identical_settings_has_no_changes() {
        let s = Settings::default();
        let d = SettingsDiff::from(&s, &s);
        assert!(!d.animating_started);
        assert!(!d.animating_changed);
        assert!(!d.paused_changed);
        assert!(!d.visibility_changed);
        assert!(!d.visual_changed);
        assert!(!d.timing_changed);
        assert!(!d.auto_stop_changed);
        assert!(!d.reminder_changed);
    }

    #[test]
    fn diff_detects_animating_transition() {
        let mut before = Settings::default();
        before.is_animating = false;
        let mut after = before.clone();
        after.is_animating = true;
        let d = SettingsDiff::from(&before, &after);
        assert!(d.animating_started);
        assert!(d.animating_changed);
        // Stopping should not register as `started`.
        let d_rev = SettingsDiff::from(&after, &before);
        assert!(!d_rev.animating_started);
        assert!(d_rev.animating_changed);
    }

    #[test]
    fn diff_visual_changed_covers_all_visual_fields() {
        let base = Settings::default();

        for (label, mutate) in [
            ("shape",           Box::new(|s: &mut Settings| s.shape = AnimationShape::Circle) as Box<dyn Fn(&mut Settings)>),
            ("gradient",        Box::new(|s| s.color_fill_gradient = ColorFillGradient::Off)),
            ("animation_mode",  Box::new(|s| s.animation_mode = AnimationMode::Linear)),
            ("inhale_color",    Box::new(|s| s.inhale_color = [0.0; 4])),
            ("exhale_color",    Box::new(|s| s.exhale_color = [1.0; 4])),
            ("background_color", Box::new(|s| s.background_color = [0.5, 0.5, 0.5, 0.5])),
            ("overlay_opacity", Box::new(|s| s.overlay_opacity = 0.99)),
        ] {
            let mut after = base.clone();
            mutate(&mut after);
            let d = SettingsDiff::from(&base, &after);
            assert!(d.visual_changed, "visual_changed should fire for {label} change");
        }
    }

    #[test]
    fn diff_timing_changed_covers_all_timing_fields() {
        let base = Settings::default();

        for (label, mutate) in [
            ("inhale_duration",                  Box::new(|s: &mut Settings| s.inhale_duration = 7.0) as Box<dyn Fn(&mut Settings)>),
            ("post_inhale_hold_duration",        Box::new(|s| s.post_inhale_hold_duration = 1.0)),
            ("exhale_duration",                  Box::new(|s| s.exhale_duration = 7.0)),
            ("post_exhale_hold_duration",        Box::new(|s| s.post_exhale_hold_duration = 1.0)),
            ("drift",                            Box::new(|s| s.drift = 1.05)),
            ("randomized_timing_inhale",         Box::new(|s| s.randomized_timing_inhale = 0.5)),
            ("randomized_timing_post_inhale",    Box::new(|s| s.randomized_timing_post_inhale_hold = 0.5)),
            ("randomized_timing_exhale",         Box::new(|s| s.randomized_timing_exhale = 0.5)),
            ("randomized_timing_post_exhale",    Box::new(|s| s.randomized_timing_post_exhale_hold = 0.5)),
            ("hold_ripple_mode",                 Box::new(|s| s.hold_ripple_mode = HoldRippleMode::Stark)),
        ] {
            let mut after = base.clone();
            mutate(&mut after);
            let d = SettingsDiff::from(&base, &after);
            assert!(d.timing_changed, "timing_changed should fire for {label} change");
        }
    }

    #[test]
    fn diff_should_restart_animation_only_when_running() {
        let mut before = Settings::default();
        before.is_animating = true;
        before.is_paused    = false;
        let mut after = before.clone();
        after.shape = AnimationShape::Circle;
        let d = SettingsDiff::from(&before, &after);
        assert!(d.should_restart_animation(&after));

        // Paused: no restart even though visual changed.
        let mut after_paused = after.clone();
        after_paused.is_paused = true;
        let d2 = SettingsDiff::from(&before, &after_paused);
        assert!(!d2.should_restart_animation(&after_paused));

        // Not animating: no restart.
        let mut after_stopped = after.clone();
        after_stopped.is_animating = false;
        let d3 = SettingsDiff::from(&before, &after_stopped);
        assert!(!d3.should_restart_animation(&after_stopped));
    }

    #[test]
    fn diff_should_restart_on_first_animating_transition() {
        let mut before = Settings::default();
        before.is_animating = false;
        let mut after = before.clone();
        after.is_animating = true;
        // Even with no visual or timing change, `started` triggers a restart.
        let d = SettingsDiff::from(&before, &after);
        assert!(d.should_restart_animation(&after));
    }

    #[test]
    fn diff_visibility_carries_new_value() {
        let mut before = Settings::default();
        before.app_visibility = AppVisibility::Both;
        let mut after = before.clone();
        after.app_visibility = AppVisibility::TopBarOnly;
        let d = SettingsDiff::from(&before, &after);
        assert!(d.visibility_changed);
        assert_eq!(d.new_visibility, AppVisibility::TopBarOnly);
    }
}
