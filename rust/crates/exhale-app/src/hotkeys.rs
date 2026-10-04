use std::sync::OnceLock;

use anyhow::Result;
use exhale_core::{KeyboardShortcut, KeyboardShortcuts, ShortcutAction};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

/// Whether global shortcuts can register in this session.  False only
/// on a Wayland session: `global-hotkey`'s Linux backend is
/// `XGrabKey`, an X11-only API that never fires under Wayland.  Cached
/// since the session type can't change while exhale is running, and
/// tooltip text rebuilds this on every settings-window frame
pub fn shortcuts_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| !crate::platform::is_wayland_session())
}

/// Per-action ids returned by [`register_hotkeys`].  The dispatcher in
/// `main.rs` matches incoming `GlobalHotKeyEvent`s by `id` through
/// [`HotkeyIds::action_for`].  Missing actions (registration failed,
/// or no key code matched) have no entry and silently no-op, so the
/// dispatcher never fires the wrong event
///
/// `registered` keeps the original `HotKey` objects so the caller can
/// call [`GlobalHotKeyManager::unregister`] when the user reassigns a
/// shortcut from the settings window
pub struct HotkeyIds {
    by_id:          Vec<(u32, ShortcutAction)>,
    pub registered: Vec<HotKey>,
}

impl HotkeyIds {
    /// The action registered under `id`, or `None` for an id this
    /// registration pass didn't produce
    pub fn action_for(&self, id: u32) -> Option<ShortcutAction> {
        self.by_id.iter().find(|&&(i, _)| i == id).map(|&(_, action)| action)
    }
}

/// Register every action in `shortcuts` as a global hotkey via the
/// `global-hotkey` crate
///
/// Each hotkey is registered individually with its own error handling: if
/// one fails (typically because the combination conflicts with a
/// system or other-app global hotkey), the remaining hotkeys are
/// still registered.  An earlier version used `?` propagation which
/// meant a single failed registration silently disabled every later
/// one in the sequence.  The user would see only "Reset works" with no
/// log entry pointing at the root cause (e.g. another app holding
/// Ctrl+Shift+A)
///
/// Returns the ids regardless of partial failure so the dispatcher
/// can match any that did register.  Failures are logged with enough
/// context for the user to recognise the conflict
pub fn register_hotkeys(
    manager:   &GlobalHotKeyManager,
    shortcuts: &KeyboardShortcuts,
) -> Result<HotkeyIds> {
    let mut state = HotkeyIds { by_id: Vec::new(), registered: Vec::new() };

    for action in crate::tray::ACTIONS {
        let Some(sc) = shortcuts.get(action) else {
            log::info!("hotkey {} is unbound, skipping registration", action.label());
            continue;
        };
        let label = format!("{} ({})", sc.display(), action.label());
        let Some(hk) = shortcut_to_hotkey(sc) else {
            log::warn!("hotkey {label} unrecognised key code '{}', skipping registration", sc.code);
            continue;
        };
        let id = hk.id();
        match manager.register(hk) {
            Ok(()) => {
                log::info!("global hotkey registered: {label} (id={id})");
                state.registered.push(hk);
                state.by_id.push((id, action));
            }
            Err(e) => {
                log::warn!(
                    "global hotkey {label} (id={id}) failed to register: {e}.  \
                     Likely conflicts with a system or other-app shortcut.  \
                     The rest of exhale's hotkeys will still work.  Right-click the \
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

/// Map our string-form key code (a `keyboard_types::Code` variant
/// name like `"KeyA"`, `"Comma"`, `"Digit1"`, `"F5"`) to the actual
/// `Code` value.  The string form is what we persist to disk, chosen
/// over the underlying `u32` because the enum's discriminant ordering
/// isn't part of `keyboard_types`' stable API
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

// egui_key_to_code + egui_modifiers_to_mask live in `crate::keymap` so a
// build without the global-hotkey crate still compiles the capture-
// overlay code in settings_window.rs
