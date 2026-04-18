use anyhow::Result;
use tray_icon::{
    menu::{
        accelerator::{Accelerator, Code, Modifiers},
        Menu, MenuItem, PredefinedMenuItem,
    },
    TrayIcon, TrayIconBuilder,
};

// ─── Menu item IDs ────────────────────────────────────────────────────────────

pub struct TrayMenuIds {
    pub preferences: tray_icon::menu::MenuId,
    pub start:       tray_icon::menu::MenuId,
    pub stop:        tray_icon::menu::MenuId,
    pub pause:       tray_icon::menu::MenuId,
    pub reset:       tray_icon::menu::MenuId,
    pub quit:        tray_icon::menu::MenuId,
    // Handles for dynamic enable/disable.
    pub start_item:  MenuItem,
    pub stop_item:   MenuItem,
    pub pause_item:  MenuItem,
}

/// Build the system-tray icon + menu.
/// Returns the `TrayIcon` handle (must stay alive) and the menu item IDs
/// so the caller can match incoming `MenuEvent`s.
pub fn build_tray() -> Result<(TrayIcon, TrayMenuIds)> {
    let icon = make_icon();

    // Accelerators shown in the tray menu; matches Swift AppDelegate shortcuts.
    // Actual key bindings are handled via `global-hotkey` so these serve as
    // visible labels next to menu items.
    let ctrl_shift = Modifiers::CONTROL | Modifiers::SHIFT;
    let cmd_only   = Modifiers::META;   // "Command" on macOS, "Windows" elsewhere

    let prefs_item = MenuItem::new("Preferences",       true, Some(Accelerator::new(Some(cmd_only),   Code::KeyW)));
    let start_item = MenuItem::new("Start Animation",   true, Some(Accelerator::new(Some(ctrl_shift), Code::KeyA)));
    let stop_item  = MenuItem::new("Stop Animation",    true, Some(Accelerator::new(Some(ctrl_shift), Code::KeyS)));
    let pause_item = MenuItem::new("Pause",             true, None);
    let reset_item = MenuItem::new("Reset to Defaults", true, Some(Accelerator::new(Some(ctrl_shift), Code::KeyF)));
    let quit_item  = MenuItem::new("Quit exhale",       true, Some(Accelerator::new(Some(cmd_only),   Code::KeyQ)));

    let ids = TrayMenuIds {
        preferences: prefs_item.id().clone(),
        start:       start_item.id().clone(),
        stop:        stop_item.id().clone(),
        pause:       pause_item.id().clone(),
        reset:       reset_item.id().clone(),
        quit:        quit_item.id().clone(),
        start_item:  start_item,
        stop_item:   stop_item,
        pause_item:  pause_item,
    };

    let menu = Menu::new();
    menu.append(&prefs_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&ids.start_item)?;
    menu.append(&ids.stop_item)?;
    menu.append(&ids.pause_item)?;
    menu.append(&reset_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_tooltip("exhale")
        .build()?;

    Ok((tray, ids))
}

/// Outlined-ring tray icon generated at runtime, matching the Swift
/// `StatusBarIcon` asset (15×17 ring shape).  Drawn with anti-aliased edges
/// in near-black so it reads well on both light and dark menu bars.
fn make_icon() -> tray_icon::Icon {
    let (w, h) = (15u32, 17u32);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let outer = (w.min(h) as f32 / 2.0) - 0.5;
    let inner = outer - 1.5;

    let rgba: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).flat_map(move |x| {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let d  = (dx * dx + dy * dy).sqrt();
            // Smooth 1-pixel antialiasing at both the outer and inner edges
            // of the ring.  Alpha peaks at the band between `inner` and `outer`.
            let aa_outer = (outer - d).clamp(0.0, 1.0);
            let aa_inner = (d - inner).clamp(0.0, 1.0);
            let alpha = (aa_outer.min(aa_inner) * 255.0) as u8;
            [0x20, 0x20, 0x20, alpha]
        }))
        .collect();

    tray_icon::Icon::from_rgba(rgba, w, h).expect("tray icon")
}
