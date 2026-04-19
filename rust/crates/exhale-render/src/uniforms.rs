use bytemuck::{Pod, Zeroable};
use exhale_core::{
    controller::BreathingState,
    settings::Settings,
    types::HoldRippleMode,
};

/// GPU-side uniform buffer matching the WGSL `OverlayUniforms` struct.
///
/// Memory layout (112 bytes, repr(C)):
///
/// | offset | size | field                |
/// |--------|------|----------------------|
/// |      0 |    8 | viewport_size        |
/// |      8 |    4 | overlay_opacity      |
/// |     12 |    4 | background_opacity   |
/// |     16 |    4 | max_circle_scale     |
/// |     20 |    4 | shape                |
/// |     24 |    4 | gradient_mode        |
/// |     28 |    4 | phase                |
/// |     32 |    4 | progress             |
/// |     36 |    4 | hold_time            |
/// |     40 |    4 | rectangle_scale      |
/// |     44 |    4 | circle_gradient_scale|
/// |     48 |    4 | ripple_enabled       |
/// |     52 |    4 | display_mode         | 0=normal 1=paused 2=stopped
/// |  56–63 |    8 | explicit padding     | ← aligns vec4 at offset 64
/// |     64 |   16 | background_color     |
/// |     80 |   16 | inhale_color         |
/// |     96 |   16 | exhale_color         |
///
/// WGSL naturally pads from offset 52→64 (vec4<f32> alignment = 16).
/// Rust's repr(C) does not, so two explicit `u32` padding fields are needed
/// after display_mode.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct OverlayUniforms {
    pub viewport_size:         [f32; 2], // offset  0
    pub overlay_opacity:       f32,      // offset  8
    pub background_opacity:    f32,      // offset 12
    pub max_circle_scale:      f32,      // offset 16
    pub shape:                 u32,      // offset 20
    pub gradient_mode:         u32,      // offset 24
    pub phase:                 u32,      // offset 28
    pub progress:              f32,      // offset 32
    pub hold_time:             f32,      // offset 36
    pub rectangle_scale:       f32,      // offset 40
    pub circle_gradient_scale: f32,      // offset 44
    pub ripple_enabled:        u32,      // offset 48
    pub display_mode:          u32,      // offset 52  0=normal 1=paused 2=stopped
    _pad1:                     u32,      // offset 56  ┐ explicit padding
    _pad2:                     u32,      // offset 60  ┘
    pub background_color:      [f32; 4], // offset 64
    pub inhale_color:          [f32; 4], // offset 80
    pub exhale_color:          [f32; 4], // offset 96
}
// Total: 112 bytes

/// `display_mode` values for the shader.
pub mod display_mode {
    pub const NORMAL:  u32 = 0;
    pub const PAUSED:  u32 = 1;
    pub const STOPPED: u32 = 2;
}

impl OverlayUniforms {
    /// Build a uniform buffer from the current breathing state and user settings.
    ///
    /// `max_circle_scale` is passed in (rather than derived from the viewport)
    /// so all monitors can share the primary-monitor scale, matching Swift's
    /// `getMaxCircleScale()` which snapshots `NSScreen.main` once at onAppear.
    pub fn from_state(
        state: &BreathingState,
        settings: &Settings,
        viewport_width: u32,
        viewport_height: u32,
        max_circle_scale: f32,
    ) -> Self {
        let w = viewport_width as f32;
        let h = viewport_height as f32;

        // Match Swift: ripple is active during a hold phase with duration > 0,
        // and also stays active during the first 10% of the following inhale/
        // exhale so the shader can fade rippleOpacity 1 → 0 the way Swift does
        // (`withAnimation(.linear(duration: duration * 0.1)) { rippleOpacity = 0 }`).
        use exhale_core::types::BreathingPhase;
        let ripple_active = match state.phase {
            BreathingPhase::HoldAfterInhale => settings.post_inhale_hold_duration > 0.0,
            BreathingPhase::HoldAfterExhale => settings.post_exhale_hold_duration > 0.0,
            // Cross-phase fade windows (preceding hold must have had duration > 0).
            BreathingPhase::Exhale => settings.post_inhale_hold_duration > 0.0,
            BreathingPhase::Inhale => settings.post_exhale_hold_duration > 0.0,
        };
        let ripple_enabled: u32 = if !ripple_active {
            0
        } else {
            match settings.hold_ripple_mode {
                HoldRippleMode::Off      => 0,
                HoldRippleMode::Stark    => 1,
                HoldRippleMode::Gradient => 2,
            }
        };

        let display_mode = if !settings.is_animating && !settings.is_paused {
            display_mode::STOPPED
        } else if settings.is_paused {
            display_mode::PAUSED
        } else {
            display_mode::NORMAL
        };

        Self {
            viewport_size:         [w, h],
            overlay_opacity:       settings.overlay_opacity,
            background_opacity:    settings.background_opacity(),
            max_circle_scale,
            shape:                 settings.shape.shader_value(),
            gradient_mode:         settings.color_fill_gradient.shader_value(),
            phase:                 state.phase.shader_value(),
            progress:              state.progress,
            hold_time:             state.hold_time,
            rectangle_scale:       settings.rectangle_scale(),
            circle_gradient_scale: settings.circle_gradient_scale(),
            ripple_enabled,
            display_mode,
            _pad1:                 0,
            _pad2:                 0,
            background_color:      settings.background_color,
            inhale_color:          settings.inhale_color,
            exhale_color:          settings.exhale_color,
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn struct_size_is_112() {
        assert_eq!(size_of::<OverlayUniforms>(), 112);
    }

    #[test]
    fn viewport_size_at_offset_0() {
        assert_eq!(offset_of!(OverlayUniforms, viewport_size), 0);
    }

    #[test]
    fn overlay_opacity_at_offset_8() {
        assert_eq!(offset_of!(OverlayUniforms, overlay_opacity), 8);
    }

    #[test]
    fn shape_at_offset_20() {
        assert_eq!(offset_of!(OverlayUniforms, shape), 20);
    }

    #[test]
    fn phase_at_offset_28() {
        assert_eq!(offset_of!(OverlayUniforms, phase), 28);
    }

    #[test]
    fn progress_at_offset_32() {
        assert_eq!(offset_of!(OverlayUniforms, progress), 32);
    }

    #[test]
    fn ripple_enabled_at_offset_48() {
        assert_eq!(offset_of!(OverlayUniforms, ripple_enabled), 48);
    }

    #[test]
    fn display_mode_at_offset_52() {
        assert_eq!(offset_of!(OverlayUniforms, display_mode), 52);
    }

    #[test]
    fn display_mode_normal_when_animating() {
        use exhale_core::types::BreathingPhase;
        let state = BreathingState { phase: BreathingPhase::Inhale, progress: 0.5, hold_time: 0.0 };
        let mut s = Settings::default();
        s.is_animating = true;
        s.is_paused = false;
        let u = OverlayUniforms::from_state(&state, &s, 1920, 1080, 1920.0_f32 / 1080.0_f32);
        assert_eq!(u.display_mode, display_mode::NORMAL);
    }

    #[test]
    fn display_mode_paused_when_paused() {
        use exhale_core::types::BreathingPhase;
        let state = BreathingState { phase: BreathingPhase::Inhale, progress: 0.5, hold_time: 0.0 };
        let mut s = Settings::default();
        s.is_animating = true;
        s.is_paused = true;
        let u = OverlayUniforms::from_state(&state, &s, 1920, 1080, 1920.0_f32 / 1080.0_f32);
        assert_eq!(u.display_mode, display_mode::PAUSED);
    }

    #[test]
    fn display_mode_stopped_when_not_animating() {
        use exhale_core::types::BreathingPhase;
        let state = BreathingState { phase: BreathingPhase::Inhale, progress: 0.5, hold_time: 0.0 };
        let mut s = Settings::default();
        s.is_animating = false;
        s.is_paused = false;
        let u = OverlayUniforms::from_state(&state, &s, 1920, 1080, 1920.0_f32 / 1080.0_f32);
        assert_eq!(u.display_mode, display_mode::STOPPED);
    }

    #[test]
    fn background_color_at_offset_64() {
        // This is the critical alignment check: vec4<f32> must be at offset 64.
        assert_eq!(offset_of!(OverlayUniforms, background_color), 64);
    }

    #[test]
    fn inhale_color_at_offset_80() {
        assert_eq!(offset_of!(OverlayUniforms, inhale_color), 80);
    }

    #[test]
    fn exhale_color_at_offset_96() {
        assert_eq!(offset_of!(OverlayUniforms, exhale_color), 96);
    }

    #[test]
    fn from_state_default_settings() {
        use exhale_core::types::BreathingPhase;

        let state = BreathingState { phase: BreathingPhase::Inhale, progress: 0.5, hold_time: 0.0 };
        let settings = Settings::default();
        let expected_scale = 1920.0_f32 / 1080.0_f32;
        let u = OverlayUniforms::from_state(&state, &settings, 1920, 1080, expected_scale);

        assert_eq!(u.viewport_size, [1920.0, 1080.0]);
        assert_eq!(u.phase, BreathingPhase::Inhale.shader_value());
        assert!((u.progress - 0.5).abs() < 1e-6);
        // Default shape is Rectangle → shader value 1
        assert_eq!(u.shape, 1);
        // Default hold durations are 0 → ripple disabled regardless of phase
        assert_eq!(u.ripple_enabled, 0);
        // Padding must be zero
        assert_eq!(u._pad1, 0);
        assert_eq!(u._pad2, 0);
        // max_circle_scale is whatever the caller provided.
        assert!((u.max_circle_scale - expected_scale).abs() < 1e-4,
            "max_circle_scale={} expected {expected_scale}", u.max_circle_scale);
    }

    #[test]
    fn max_circle_scale_passthrough() {
        use exhale_core::types::BreathingPhase;
        let state = BreathingState { phase: BreathingPhase::Inhale, progress: 0.0, hold_time: 0.0 };
        // Caller supplies the scale — verify it rides through to the uniform.
        let u = OverlayUniforms::from_state(&state, &Settings::default(), 1000, 1000, 2.5);
        assert!((u.max_circle_scale - 2.5).abs() < 1e-6);
    }

    #[test]
    fn ripple_disabled_when_hold_duration_zero() {
        use exhale_core::types::BreathingPhase;
        let state = BreathingState { phase: BreathingPhase::HoldAfterInhale, progress: 1.0, hold_time: 0.5 };
        let mut s = Settings::default();
        s.post_inhale_hold_duration = 0.0; // hold disabled
        let u = OverlayUniforms::from_state(&state, &s, 1920, 1080, 1920.0_f32 / 1080.0_f32);
        assert_eq!(u.ripple_enabled, 0, "ripple must be off when hold duration is 0");
    }

    #[test]
    fn ripple_enabled_when_hold_duration_nonzero() {
        use exhale_core::types::BreathingPhase;
        let state = BreathingState { phase: BreathingPhase::HoldAfterInhale, progress: 1.0, hold_time: 0.5 };
        let mut s = Settings::default();
        s.post_inhale_hold_duration = 2.0;
        let u = OverlayUniforms::from_state(&state, &s, 1920, 1080, 1920.0_f32 / 1080.0_f32);
        assert_eq!(u.ripple_enabled, 2, "Gradient mode = 2 when hold active");
    }

    #[test]
    fn pod_roundtrip() {
        use exhale_core::types::BreathingPhase;

        let state = BreathingState { phase: BreathingPhase::Exhale, progress: 0.75, hold_time: 0.0 };
        let settings = Settings::default();
        let original = OverlayUniforms::from_state(&state, &settings, 800, 600, 4.0 / 3.0);

        // Cast to bytes and back — bytemuck roundtrip.
        let bytes: &[u8] = bytemuck::bytes_of(&original);
        assert_eq!(bytes.len(), 112);
        let restored: OverlayUniforms = *bytemuck::from_bytes(bytes);
        assert_eq!(restored.phase, original.phase);
        assert!((restored.progress - original.progress).abs() < 1e-6);
    }
}
