use serde::{Deserialize, Serialize};

/// Breathing animation shape shown on the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationShape {
    /// Solid fill covering the entire screen.
    Fullscreen,
    /// Rectangle that scales up from the bottom of the screen.
    Rectangle,
    /// Circle that scales from the center outward.
    Circle,
}

/// Color fill / gradient style applied to the animated shape.
///
/// Shader encoding: Off=0, Inner=1, On=2
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorFillGradient {
    /// Solid color, no gradient.
    Off,
    /// Gradient from background at the base to shape color at the top/edge.
    Inner,
    /// Gradient that peaks at the midpoint and fades back to background (full cycle).
    On,
}

/// Style of the perimeter glow shown during hold phases.
///
/// Shader encoding: Off=0, Stark=1, Gradient=2
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldRippleMode {
    /// No ripple effect during holds.
    Off,
    /// Hard-edged band sweeping the screen perimeter.
    Stark,
    /// Soft Gaussian glow sweeping the screen perimeter.
    Gradient,
}

/// Easing curve applied to each inhale/exhale transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationMode {
    /// No easing — progress advances at a constant rate.
    Linear,
    /// CSS cubic-bezier(0.42, 0, 0.58, 1) — ease-in-out.
    Sinusoidal,
}

/// Controls how the app appears in the macOS UI chrome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppVisibility {
    TopBarOnly,
    DockOnly,
    Both,
}

/// The four phases of a single breath cycle.
///
/// Shader phase encoding: Inhale=0, HoldAfterInhale=1, Exhale=2, HoldAfterExhale=3
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreathingPhase {
    Inhale,
    HoldAfterInhale,
    Exhale,
    HoldAfterExhale,
}

impl BreathingPhase {
    /// Integer encoding sent to the fragment shader.
    pub fn shader_value(self) -> u32 {
        match self {
            Self::Inhale         => 0,
            Self::HoldAfterInhale => 1,
            Self::Exhale         => 2,
            Self::HoldAfterExhale => 3,
        }
    }

    /// Returns true for the two hold phases.
    pub fn is_hold(self) -> bool {
        matches!(self, Self::HoldAfterInhale | Self::HoldAfterExhale)
    }
}

impl AnimationShape {
    pub fn shader_value(self) -> u32 {
        match self {
            Self::Fullscreen => 0,
            Self::Rectangle  => 1,
            Self::Circle     => 2,
        }
    }
}

impl ColorFillGradient {
    pub fn shader_value(self) -> u32 {
        match self {
            Self::Off   => 0,
            Self::Inner => 1,
            Self::On    => 2,
        }
    }
}

impl HoldRippleMode {
    pub fn shader_value(self) -> u32 {
        match self {
            Self::Off      => 0,
            Self::Stark    => 1,
            Self::Gradient => 2,
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
        // TOML requires a table at the top level, so wrap in a struct.
        #[derive(serde::Serialize, serde::Deserialize)]
        struct W { shape: AnimationShape, ripple: HoldRippleMode }
        let w = W { shape: AnimationShape::Circle, ripple: HoldRippleMode::Gradient };
        let s = toml::to_string(&w).expect("serialise");
        let back: W = toml::from_str(&s).expect("deserialise");
        assert_eq!(back.shape, AnimationShape::Circle);
        assert_eq!(back.ripple, HoldRippleMode::Gradient);
    }
}
