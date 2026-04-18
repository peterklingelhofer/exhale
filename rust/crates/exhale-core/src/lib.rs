pub mod controller;
pub mod easing;
pub mod settings;
pub mod settings_manager;
pub mod types;

pub use controller::{BreathingController, BreathingState};
pub use easing::EasingTable;
pub use settings::Settings;
pub use settings_manager::SettingsManager;
pub use types::{
    AnimationMode, AnimationShape, AppVisibility, BreathingPhase, ColorFillGradient,
    HoldRippleMode,
};
