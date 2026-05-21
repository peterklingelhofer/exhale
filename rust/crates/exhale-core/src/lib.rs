pub mod controller;
pub mod easing;
pub mod poison;
pub mod settings;
pub mod settings_manager;
pub mod types;

pub use controller::{BreathingController, BreathingState};
pub use easing::EasingTable;
pub use settings::{
    KeyboardShortcut, KeyboardShortcuts, Settings, ShortcutAction, WindowPlacement,
    KBD_MOD_ALT, KBD_MOD_CTRL, KBD_MOD_META, KBD_MOD_SHIFT,
};
pub use settings_manager::SettingsManager;
pub use types::{
    AnimationMode, AnimationShape, AppVisibility, BreathingPhase, ColorFillGradient,
    HoldRippleMode,
};
