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

    // ── Window frame (persisted so the settings panel reopens where you left it) ─
    //
    // Stored as an offset relative to a named monitor (matching Swift's
    // per-screen persistence in AppDelegate.windowDidMove).  When the monitor
    // listed in `settings_window_screen` is no longer connected we fall back
    // to the OS default position rather than restoring to absolute coordinates
    // that might now be off-screen.
    #[serde(default)]
    pub settings_window_x: Option<i32>,
    #[serde(default)]
    pub settings_window_y: Option<i32>,
    #[serde(default)]
    pub settings_window_height: Option<u32>,
    #[serde(default)]
    pub settings_window_screen: Option<String>,
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
            app_visibility:      AppVisibility::TopBarOnly,

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
        }
    }
}

impl Settings {
    /// Returns true when inhale and exhale colors are perceptually identical.
    /// Used to skip unnecessary redraws in the fullscreen shape.
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
}
