use serde::{Deserialize, Serialize};

/// Breathing animation shape shown on the overlay
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationShape {
    /// Solid fill covering the entire screen
    Fullscreen,
    /// Rectangle that scales up from the bottom of the screen
    Rectangle,
    /// Circle that scales from the center outward
    Circle,
}

/// Color fill / gradient style applied to the animated shape
///
/// Shader encoding: Off=0, Inner=1, On=2
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorFillGradient {
    /// Solid color, no gradient
    Off,
    /// Gradient from background at the base to shape color at the top/edge
    Inner,
    /// Gradient that peaks at the midpoint and fades back to background (full cycle)
    On,
}

/// Style of the perimeter glow shown during hold phases
///
/// Shader encoding: Off=0, Stark=1, Gradient=2
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldRippleMode {
    /// No ripple effect during holds
    Off,
    /// Hard-edged band sweeping the screen perimeter
    Stark,
    /// Soft Gaussian glow sweeping the screen perimeter
    Gradient,
}

/// Easing curve applied to each inhale/exhale transition
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationMode {
    /// No easing: progress advances at a constant rate
    Linear,
    /// CSS cubic-bezier(0.42, 0, 0.58, 1): ease-in-out
    Sinusoidal,
}

/// Controls how the app appears in the macOS UI chrome
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppVisibility {
    TopBarOnly,
    DockOnly,
    Both,
}

/// How the breathing animation is put on screen
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    /// One translucent, click-through window per monitor, floating above
    /// the other windows
    Overlay,
    /// One regular window to move, resize or make full screen
    Window,
}


/// The four phases of a single breath cycle
///
/// Shader phase encoding: Inhale=0, HoldAfterInhale=1, Exhale=2, HoldAfterExhale=3
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreathingPhase {
    Inhale,
    HoldAfterInhale,
    Exhale,
    HoldAfterExhale,
}

impl BreathingPhase {
    /// Integer encoding sent to the fragment shader
    pub fn shader_value(self) -> u32 {
        self as u32
    }

    /// Returns true for the two hold phases
    pub(crate) fn is_hold(self) -> bool {
        matches!(self, Self::HoldAfterInhale | Self::HoldAfterExhale)
    }

    /// Name written into the session log's `phase_end` lines
    ///
    /// Snake case and stable: a lab's analysis script matches on these
    /// strings, so they're part of the log format
    pub fn name(self) -> &'static str {
        match self {
            Self::Inhale          => "inhale",
            Self::HoldAfterInhale => "hold_after_inhale",
            Self::Exhale          => "exhale",
            Self::HoldAfterExhale => "hold_after_exhale",
        }
    }
}

// The `name` methods below give the session log's header the same
// snake_case strings settings.toml and study.toml spell each setting
// with. `names_match_what_settings_toml_writes` holds the two together

impl AnimationShape {
    pub fn shader_value(self) -> u32 {
        self as u32
    }

    /// Name written into the session log's `app_start` and `log_cleared` lines
    pub fn name(self) -> &'static str {
        match self {
            Self::Fullscreen => "fullscreen",
            Self::Rectangle  => "rectangle",
            Self::Circle     => "circle",
        }
    }
}

impl ColorFillGradient {
    pub fn shader_value(self) -> u32 {
        self as u32
    }

    /// Name written into the session log's `app_start` and `log_cleared` lines
    pub fn name(self) -> &'static str {
        match self {
            Self::Off   => "off",
            Self::Inner => "inner",
            Self::On    => "on",
        }
    }
}

impl HoldRippleMode {
    pub fn shader_value(self) -> u32 {
        self as u32
    }

    /// Name written into the session log's `app_start` and `log_cleared` lines
    pub fn name(self) -> &'static str {
        match self {
            Self::Off      => "off",
            Self::Stark    => "stark",
            Self::Gradient => "gradient",
        }
    }
}

impl AnimationMode {
    /// Name written into the session log's `app_start` and `log_cleared` lines
    pub fn name(self) -> &'static str {
        match self {
            Self::Linear     => "linear",
            Self::Sinusoidal => "sinusoidal",
        }
    }
}

impl DisplayMode {
    /// Name written into the session log's `app_start` and `log_cleared` lines
    pub fn name(self) -> &'static str {
        match self {
            Self::Overlay => "overlay",
            Self::Window  => "window",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_shader_values_are_unique_and_sequential() {
        assert_eq!(BreathingPhase::Inhale.shader_value(), 0);
        assert_eq!(BreathingPhase::HoldAfterInhale.shader_value(), 1);
        assert_eq!(BreathingPhase::Exhale.shader_value(), 2);
        assert_eq!(BreathingPhase::HoldAfterExhale.shader_value(), 3);
    }

    #[test]
    fn phase_names_are_the_logged_snake_case_strings() {
        assert_eq!(BreathingPhase::Inhale.name(),          "inhale");
        assert_eq!(BreathingPhase::HoldAfterInhale.name(), "hold_after_inhale");
        assert_eq!(BreathingPhase::Exhale.name(),          "exhale");
        assert_eq!(BreathingPhase::HoldAfterExhale.name(), "hold_after_exhale");
    }

    #[test]
    fn hold_detection() {
        assert!(!BreathingPhase::Inhale.is_hold());
        assert!(BreathingPhase::HoldAfterInhale.is_hold());
        assert!(!BreathingPhase::Exhale.is_hold());
        assert!(BreathingPhase::HoldAfterExhale.is_hold());
    }

    #[test]
    fn shape_shader_values() {
        assert_eq!(AnimationShape::Fullscreen.shader_value(), 0);
        assert_eq!(AnimationShape::Rectangle.shader_value(), 1);
        assert_eq!(AnimationShape::Circle.shader_value(), 2);
    }

    #[test]
    fn gradient_shader_values() {
        assert_eq!(ColorFillGradient::Off.shader_value(), 0);
        assert_eq!(ColorFillGradient::Inner.shader_value(), 1);
        assert_eq!(ColorFillGradient::On.shader_value(), 2);
    }

    #[test]
    fn ripple_shader_values() {
        assert_eq!(HoldRippleMode::Off.shader_value(), 0);
        assert_eq!(HoldRippleMode::Stark.shader_value(), 1);
        assert_eq!(HoldRippleMode::Gradient.shader_value(), 2);
    }

    #[test]
    fn serde_round_trip() {
        // TOML requires a table at the top level, so wrap in a struct
        #[derive(serde::Serialize, serde::Deserialize)]
        struct W { shape: AnimationShape, ripple: HoldRippleMode }
        let w = W { shape: AnimationShape::Circle, ripple: HoldRippleMode::Gradient };
        let s = toml::to_string(&w).expect("serialise");
        let back: W = toml::from_str(&s).expect("deserialise");
        assert_eq!(back.shape, AnimationShape::Circle);
        assert_eq!(back.ripple, HoldRippleMode::Gradient);
    }

    #[test]
    fn names_match_what_settings_toml_writes() {
        // One wrapper per enum, so each variant is serialised as the
        // value of a key, the way settings.toml and study.toml hold it
        fn written<T: serde::Serialize>(value: T) -> String {
            #[derive(serde::Serialize)]
            struct W<T> { v: T }
            let text = toml::to_string(&W { v: value }).expect("serialise");
            text.trim_end().strip_prefix("v = \"").and_then(|t| t.strip_suffix('"')).expect("a string").to_string()
        }
        for shape in [AnimationShape::Fullscreen, AnimationShape::Rectangle, AnimationShape::Circle] {
            assert_eq!(shape.name(), written(shape));
        }
        for gradient in [ColorFillGradient::Off, ColorFillGradient::Inner, ColorFillGradient::On] {
            assert_eq!(gradient.name(), written(gradient));
        }
        for ripple in [HoldRippleMode::Off, HoldRippleMode::Stark, HoldRippleMode::Gradient] {
            assert_eq!(ripple.name(), written(ripple));
        }
        for mode in [AnimationMode::Linear, AnimationMode::Sinusoidal] {
            assert_eq!(mode.name(), written(mode));
        }
        for mode in [DisplayMode::Overlay, DisplayMode::Window] {
            assert_eq!(mode.name(), written(mode));
        }
    }

    #[test]
    fn display_mode_is_written_in_snake_case_like_its_siblings() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct W { display_mode: DisplayMode }
        for (mode, text) in [
            (DisplayMode::Overlay, "display_mode = \"overlay\"\n"),
            (DisplayMode::Window,  "display_mode = \"window\"\n"),
        ] {
            let s = toml::to_string(&W { display_mode: mode }).expect("serialise");
            assert_eq!(s, text);
            let back: W = toml::from_str(&s).expect("deserialise");
            assert_eq!(back.display_mode, mode);
        }
    }
}
