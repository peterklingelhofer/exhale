use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

pub struct HotkeyIds {
    pub start:        u32,
    pub stop:         u32,
    pub reset:        u32,
    pub preferences:  u32,
    pub preferences2: u32,
}

/// Register global hotkeys matching the Swift app:
///   Ctrl+Shift+A  — Start animation
///   Ctrl+Shift+S  — Stop animation
///   Ctrl+Shift+F  — Reset to defaults
///   Ctrl+Shift+,  — Show preferences (Swift primary)
///   Ctrl+Shift+W  — Show preferences (Swift secondary)
pub fn register_hotkeys(manager: &GlobalHotKeyManager) -> Result<HotkeyIds> {
    let mods = Modifiers::CONTROL | Modifiers::SHIFT;

    let start  = HotKey::new(Some(mods), Code::KeyA);
    let stop   = HotKey::new(Some(mods), Code::KeyS);
    let reset  = HotKey::new(Some(mods), Code::KeyF);
    let prefs  = HotKey::new(Some(mods), Code::Comma);
    let prefs2 = HotKey::new(Some(mods), Code::KeyW);

    let ids = HotkeyIds {
        start:        start.id(),
        stop:         stop.id(),
        reset:        reset.id(),
        preferences:  prefs.id(),
        preferences2: prefs2.id(),
    };

    manager.register(start)?;
    manager.register(stop)?;
    manager.register(reset)?;
    manager.register(prefs)?;
    manager.register(prefs2)?;

    Ok(ids)
}
