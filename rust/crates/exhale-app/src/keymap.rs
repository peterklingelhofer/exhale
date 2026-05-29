// egui → internal key/modifier translators used by the shortcut-capture
// overlay in `settings_window.rs`. Pulled out of `hotkeys.rs` so the MAS
// build (`--no-default-features`, no global-hotkey crate) can still let
// the user capture and persist a binding even when no system-level
// registration will happen

use exhale_core::{KBD_MOD_ALT, KBD_MOD_CTRL, KBD_MOD_SHIFT};
#[cfg(target_os = "macos")]
use exhale_core::KBD_MOD_META;

/// Convert an egui [`egui::Key`] back to our string-form key code so
/// the capture overlay can persist whatever the user just pressed.
/// `None` for keys we don't have a Carbon / Win32 / X11 binding for
/// (e.g. `Plus`, `Minus` aren't unambiguous physical keys)
pub fn egui_key_to_code(key: egui::Key) -> Option<&'static str> {
    use egui::Key::*;
    Some(match key {
        A => "KeyA", B => "KeyB", C => "KeyC", D => "KeyD", E => "KeyE",
        F => "KeyF", G => "KeyG", H => "KeyH", I => "KeyI", J => "KeyJ",
        K => "KeyK", L => "KeyL", M => "KeyM", N => "KeyN", O => "KeyO",
        P => "KeyP", Q => "KeyQ", R => "KeyR", S => "KeyS", T => "KeyT",
        U => "KeyU", V => "KeyV", W => "KeyW", X => "KeyX", Y => "KeyY",
        Z => "KeyZ",
        Num0 => "Digit0", Num1 => "Digit1", Num2 => "Digit2", Num3 => "Digit3",
        Num4 => "Digit4", Num5 => "Digit5", Num6 => "Digit6", Num7 => "Digit7",
        Num8 => "Digit8", Num9 => "Digit9",
        F1 => "F1", F2 => "F2", F3 => "F3", F4 => "F4", F5 => "F5",
        F6 => "F6", F7 => "F7", F8 => "F8", F9 => "F9", F10 => "F10",
        F11 => "F11", F12 => "F12",
        Comma        => "Comma",
        Period       => "Period",
        Slash        => "Slash",
        Backslash    => "Backslash",
        Semicolon    => "Semicolon",
        Quote        => "Quote",
        Backtick     => "Backquote",
        Minus        => "Minus",
        Equals       => "Equal",
        OpenBracket  => "BracketLeft",
        CloseBracket => "BracketRight",
        Space        => "Space",
        Enter        => "Enter",
        Tab          => "Tab",
        Escape       => "Escape",
        Backspace    => "Backspace",
        ArrowLeft    => "ArrowLeft",
        ArrowRight   => "ArrowRight",
        ArrowUp      => "ArrowUp",
        ArrowDown    => "ArrowDown",
        Home         => "Home",
        End          => "End",
        PageUp       => "PageUp",
        PageDown     => "PageDown",
        Insert       => "Insert",
        Delete       => "Delete",
        _ => return None,
    })
}

/// Convert egui [`egui::Modifiers`] to our bitmask form.  egui's
/// `command` field is the Mac Command key on macOS and Ctrl
/// elsewhere; we always pack Mac Command into the META bit so the
/// serialised shortcut means the same thing across OSes
pub fn egui_modifiers_to_mask(m: egui::Modifiers) -> u8 {
    let mut mask = 0;
    if m.ctrl  { mask |= KBD_MOD_CTRL; }
    if m.shift { mask |= KBD_MOD_SHIFT; }
    if m.alt   { mask |= KBD_MOD_ALT; }
    // On macOS, egui's `mac_cmd` is Command and `command` is also Command;
    // on other OSes `command` aliases to Ctrl which we've already captured.
    // Only count Meta explicitly via mac_cmd so non-macOS double-counts don't happen.
    #[cfg(target_os = "macos")]
    if m.mac_cmd { mask |= KBD_MOD_META; }
    mask
}
