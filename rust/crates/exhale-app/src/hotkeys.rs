use anyhow::Result;
use exhale_core::{
    KeyboardShortcut, KeyboardShortcuts, ShortcutAction,
    KBD_MOD_ALT, KBD_MOD_CTRL, KBD_MOD_SHIFT,
};
#[cfg(target_os = "macos")]
use exhale_core::KBD_MOD_META;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

/// Per-action ids returned by [`register_hotkeys`].  The dispatcher in
/// `main.rs` matches incoming [`GlobalHotKeyEvent`]s by `id`, so
/// missing actions (registration failed, or no key code matched) stay
/// `None` and silently no-op rather than dispatching the wrong event.
///
/// `registered` keeps the original `HotKey` objects so the caller can
/// call [`GlobalHotKeyManager::unregister`] when the user reassigns a
/// shortcut from the settings window
pub struct HotkeyIds {
    pub start:        Option<u32>,
    pub stop:         Option<u32>,
    pub reset:        Option<u32>,
    pub quit:         Option<u32>,
    pub preferences:  Option<u32>,
    pub registered:   Vec<HotKey>,
}

/// Register every action in `shortcuts` as a global hotkey via the
/// `global-hotkey` crate.
///
/// Each hotkey is registered individually with its own error handling
/// — if one fails (typically because the combination conflicts with a
/// system or other-app global hotkey), the remaining hotkeys are
/// still registered.  An earlier version used `?` propagation which
/// meant a single failed registration silently disabled every later
/// one in the sequence; the user would see only "Reset works" with no
/// log entry pointing at the root cause (e.g. another app holding
/// Ctrl+Shift+A).
///
/// Returns the ids regardless of partial failure so the dispatcher
/// can match any that did register; failures are logged with enough
/// context for the user to recognise the conflict
pub fn register_hotkeys(
    manager:   &GlobalHotKeyManager,
    shortcuts: &KeyboardShortcuts,
) -> Result<HotkeyIds> {
    let mut state = HotkeyIds {
        start: None, stop: None, reset: None, quit: None, preferences: None,
        registered: Vec::new(),
    };

    for (action, sc_opt) in [
        (ShortcutAction::Start,       shortcuts.start.as_ref()),
        (ShortcutAction::Stop,        shortcuts.stop.as_ref()),
        (ShortcutAction::Reset,       shortcuts.reset.as_ref()),
        (ShortcutAction::Quit,        shortcuts.quit.as_ref()),
        (ShortcutAction::Preferences, shortcuts.preferences.as_ref()),
    ] {
        let Some(sc) = sc_opt else {
            log::info!("hotkey {} is unbound; skipping registration", action.label());
            continue;
        };
        let label = format!("{} ({})", sc.display(), action.label());
        let Some(hk) = shortcut_to_hotkey(sc) else {
            log::warn!("hotkey {label} unrecognised key code '{}'; skipping registration", sc.code);
            continue;
        };
        let id = hk.id();
        match manager.register(hk) {
            Ok(()) => {
                log::info!("global hotkey registered: {label} (id={id})");
                state.registered.push(hk);
                match action {
                    ShortcutAction::Start       => state.start       = Some(id),
                    ShortcutAction::Stop        => state.stop        = Some(id),
                    ShortcutAction::Reset       => state.reset       = Some(id),
                    ShortcutAction::Quit        => state.quit        = Some(id),
                    ShortcutAction::Preferences => state.preferences = Some(id),
                }
            }
            Err(e) => {
                log::warn!(
                    "global hotkey {label} (id={id}) failed to register: {e} \
                     — likely conflicts with a system or other-app shortcut; \
                     the rest of exhale's hotkeys will still work.  Right-click the \
                     matching button in the settings window to assign a different key"
                );
            }
        }
    }

    Ok(state)
}

/// Unregister every hotkey in `ids.registered`.  Errors are logged
/// but not returned: the caller (settings-window rebind path) wants
/// to proceed to re-registration regardless, and a stale-registration
/// leak is preferable to leaving the dispatcher partially bound
pub fn unregister_all(manager: &GlobalHotKeyManager, ids: &HotkeyIds) {
    for hk in &ids.registered {
        if let Err(e) = manager.unregister(*hk) {
            log::warn!("failed to unregister hotkey id={}: {e}", hk.id());
        }
    }
}

/// Build a `global_hotkey::HotKey` from our serialisable
/// [`KeyboardShortcut`].  Returns `None` when `sc.code` doesn't map
/// to a known [`Code`] variant (e.g. the user pasted a malformed
/// settings file).  Callers log + skip
pub fn shortcut_to_hotkey(sc: &KeyboardShortcut) -> Option<HotKey> {
    let mut mods = Modifiers::empty();
    if sc.has_ctrl()  { mods |= Modifiers::CONTROL; }
    if sc.has_shift() { mods |= Modifiers::SHIFT; }
    if sc.has_alt()   { mods |= Modifiers::ALT; }
    if sc.has_meta()  { mods |= Modifiers::META; }
    let code = code_from_str(&sc.code)?;
    Some(HotKey::new(Some(mods), code))
}

/// Map our string-form key code (a [`keyboard_types::Code`] variant
/// name like `"KeyA"`, `"Comma"`, `"Digit1"`, `"F5"`) to the actual
/// `Code` value.  The string form is what we persist to disk, chosen
/// over the underlying `u32` because the enum's discriminant ordering
/// is not part of `keyboard_types`' stable API
fn code_from_str(s: &str) -> Option<Code> {
    use Code::*;
    Some(match s {
        "KeyA" => KeyA, "KeyB" => KeyB, "KeyC" => KeyC, "KeyD" => KeyD,
        "KeyE" => KeyE, "KeyF" => KeyF, "KeyG" => KeyG, "KeyH" => KeyH,
        "KeyI" => KeyI, "KeyJ" => KeyJ, "KeyK" => KeyK, "KeyL" => KeyL,
        "KeyM" => KeyM, "KeyN" => KeyN, "KeyO" => KeyO, "KeyP" => KeyP,
        "KeyQ" => KeyQ, "KeyR" => KeyR, "KeyS" => KeyS, "KeyT" => KeyT,
        "KeyU" => KeyU, "KeyV" => KeyV, "KeyW" => KeyW, "KeyX" => KeyX,
        "KeyY" => KeyY, "KeyZ" => KeyZ,
        "Digit0" => Digit0, "Digit1" => Digit1, "Digit2" => Digit2,
        "Digit3" => Digit3, "Digit4" => Digit4, "Digit5" => Digit5,
        "Digit6" => Digit6, "Digit7" => Digit7, "Digit8" => Digit8,
        "Digit9" => Digit9,
        "F1"  => F1,  "F2"  => F2,  "F3"  => F3,  "F4"  => F4,
        "F5"  => F5,  "F6"  => F6,  "F7"  => F7,  "F8"  => F8,
        "F9"  => F9,  "F10" => F10, "F11" => F11, "F12" => F12,
        "Comma"        => Comma,
        "Period"       => Period,
        "Slash"        => Slash,
        "Backslash"    => Backslash,
        "Semicolon"    => Semicolon,
        "Quote"        => Quote,
        "Backquote"    => Backquote,
        "Minus"        => Minus,
        "Equal"        => Equal,
        "BracketLeft"  => BracketLeft,
        "BracketRight" => BracketRight,
        "Space"        => Space,
        "Enter"        => Enter,
        "Tab"          => Tab,
        "Escape"       => Escape,
        "Backspace"    => Backspace,
        "ArrowLeft"    => ArrowLeft,
        "ArrowRight"   => ArrowRight,
        "ArrowUp"      => ArrowUp,
        "ArrowDown"    => ArrowDown,
        "Home"         => Home,
        "End"          => End,
        "PageUp"       => PageUp,
        "PageDown"     => PageDown,
        "Insert"       => Insert,
        "Delete"       => Delete,
        _ => return None,
    })
}

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
