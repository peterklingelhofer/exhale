use anyhow::Result;
use exhale_core::{KeyboardShortcuts, ShortcutAction};
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    TrayIcon, TrayIconBuilder,
};

// ─── Menu item IDs ────────────────────────────────────────────────────────────

pub struct TrayMenuIds {
    pub preferences: tray_icon::menu::MenuId,
    pub start:       tray_icon::menu::MenuId,
    pub stop:        tray_icon::menu::MenuId,
    pub reset:       tray_icon::menu::MenuId,
    pub quit:        tray_icon::menu::MenuId,
    // Handles for dynamic enable/disable.
    pub start_item:  MenuItem,
    pub stop_item:   MenuItem,
    // Top-level item handles whose labels include the current
    // keybinding — kept here so the rebind path can `set_text` them
    // when the user changes a shortcut, instead of rebuilding the
    // whole tray
    pub preferences_item: MenuItem,
    pub reset_item:       MenuItem,
    pub quit_item:        MenuItem,
    // ── "Keyboard Shortcuts ▶" submenu ────────────────────────────────────────
    //
    // Each entry both displays the action's current binding (label
    // text via `set_text` on rebind) and acts as a click target that
    // opens the settings window in capture mode for that action.
    // Storing the handles here lets us update labels in place without
    // a tray rebuild
    pub kb_start_item:       MenuItem,
    pub kb_stop_item:        MenuItem,
    pub kb_reset_item:       MenuItem,
    pub kb_quit_item:        MenuItem,
    pub kb_preferences_item: MenuItem,
    pub kb_start:       tray_icon::menu::MenuId,
    pub kb_stop:        tray_icon::menu::MenuId,
    pub kb_reset:       tray_icon::menu::MenuId,
    pub kb_quit:        tray_icon::menu::MenuId,
    pub kb_preferences: tray_icon::menu::MenuId,
}

impl TrayMenuIds {
    /// Match a clicked tray-menu item id back to the
    /// [`ShortcutAction`] whose binding the user wants to change.
    /// Returns `None` for items that aren't part of the
    /// "Keyboard Shortcuts ▶" submenu
    pub fn kb_action_for(&self, id: &tray_icon::menu::MenuId) -> Option<ShortcutAction> {
        if id == &self.kb_start       { Some(ShortcutAction::Start) }
        else if id == &self.kb_stop   { Some(ShortcutAction::Stop) }
        else if id == &self.kb_reset  { Some(ShortcutAction::Reset) }
        else if id == &self.kb_quit   { Some(ShortcutAction::Quit) }
        else if id == &self.kb_preferences { Some(ShortcutAction::Preferences) }
        else { None }
    }

    /// Refresh every label that embeds a keyboard-shortcut binding
    /// after the user reassigns one.  Called from the rebind path so
    /// the tray menu stays in sync with `settings.keyboard_shortcuts`
    /// without a full tray rebuild (which would flash the tray icon)
    pub fn refresh_labels(&self, shortcuts: &KeyboardShortcuts) {
        self.preferences_item.set_text(top_level_label("Preferences",       shortcuts.get(ShortcutAction::Preferences)));
        self.start_item.set_text(      top_level_label("Start Animation",   shortcuts.get(ShortcutAction::Start)));
        self.stop_item.set_text(       top_level_label("Stop Animation",    shortcuts.get(ShortcutAction::Stop)));
        self.reset_item.set_text(      top_level_label("Reset to Defaults", shortcuts.get(ShortcutAction::Reset)));
        self.quit_item.set_text(       top_level_label("Quit exhale",       shortcuts.get(ShortcutAction::Quit)));

        self.kb_start_item.set_text(       submenu_label(ShortcutAction::Start,       shortcuts));
        self.kb_stop_item.set_text(        submenu_label(ShortcutAction::Stop,        shortcuts));
        self.kb_reset_item.set_text(       submenu_label(ShortcutAction::Reset,       shortcuts));
        self.kb_quit_item.set_text(        submenu_label(ShortcutAction::Quit,        shortcuts));
        self.kb_preferences_item.set_text( submenu_label(ShortcutAction::Preferences, shortcuts));
    }
}

/// Format a top-level menu item's label.  Embeds the current
/// binding in parentheses so the user can read it without opening
/// the submenu; reads "Preferences" when the slot is unbound
fn top_level_label(base: &str, sc: Option<&exhale_core::KeyboardShortcut>) -> String {
    match sc {
        Some(sc) => format!("{base}  ({})", sc.display()),
        None     => base.to_string(),
    }
}

/// Format a "Keyboard Shortcuts ▶" submenu item.  Action name on
/// the left, current binding (or "(none)") on the right.  Clicking
/// the row opens settings in capture mode for the matching action
fn submenu_label(action: ShortcutAction, shortcuts: &KeyboardShortcuts) -> String {
    let binding = shortcuts
        .get(action)
        .map(|sc| sc.display())
        .unwrap_or_else(|| "(none)".to_string());
    format!("{}: {binding}", action.label())
}

/// Build the system-tray icon + menu.
/// Returns the `TrayIcon` handle (must stay alive) and the menu item IDs
/// so the caller can match incoming `MenuEvent`s.
///
/// `shortcuts` is the current snapshot of user keybindings; labels
/// embed each action's binding so the user can see at a glance what's
/// bound to what without leaving the menu.  Pass the same struct back
/// to [`TrayMenuIds::refresh_labels`] when bindings change to keep
/// the menu in sync
pub fn build_tray(shortcuts: &KeyboardShortcuts) -> Result<(TrayIcon, TrayMenuIds)> {
    let icon = make_icon();

    // No `Accelerator::new(...)` on any item.  Reasons:
    //   1. Bindings are user-customisable now; a static accelerator
    //      label would lie when the user reassigns a shortcut.
    //   2. On macOS, an `Accelerator` becomes the NSMenuItem's
    //      `keyEquivalent`, which fires WHILE the menu is open.  The
    //      same key press also queues in the global-hotkey channel,
    //      so closing the menu plays the action a second time —
    //      double-trigger bug.
    // Embed the binding in the label text instead so it stays correct
    // and avoids the dual-dispatch hazard
    let prefs_item = MenuItem::new(top_level_label("Preferences",       shortcuts.get(ShortcutAction::Preferences)), true, None);
    let start_item = MenuItem::new(top_level_label("Start Animation",   shortcuts.get(ShortcutAction::Start)),       true, None);
    let stop_item  = MenuItem::new(top_level_label("Stop Animation",    shortcuts.get(ShortcutAction::Stop)),        true, None);
    let reset_item = MenuItem::new(top_level_label("Reset to Defaults", shortcuts.get(ShortcutAction::Reset)),       true, None);
    let quit_item  = MenuItem::new(top_level_label("Quit exhale",       shortcuts.get(ShortcutAction::Quit)),        true, None);

    // ── Keyboard Shortcuts submenu ────────────────────────────────────────────
    let kb_start       = MenuItem::new(submenu_label(ShortcutAction::Start,       shortcuts), true, None);
    let kb_stop        = MenuItem::new(submenu_label(ShortcutAction::Stop,        shortcuts), true, None);
    let kb_reset       = MenuItem::new(submenu_label(ShortcutAction::Reset,       shortcuts), true, None);
    let kb_quit        = MenuItem::new(submenu_label(ShortcutAction::Quit,        shortcuts), true, None);
    let kb_preferences = MenuItem::new(submenu_label(ShortcutAction::Preferences, shortcuts), true, None);

    let kb_submenu = Submenu::new("Keyboard Shortcuts", true);
    kb_submenu.append(&kb_start)?;
    kb_submenu.append(&kb_stop)?;
    kb_submenu.append(&kb_reset)?;
    kb_submenu.append(&kb_quit)?;
    kb_submenu.append(&PredefinedMenuItem::separator())?;
    kb_submenu.append(&kb_preferences)?;

    let ids = TrayMenuIds {
        preferences: prefs_item.id().clone(),
        start:       start_item.id().clone(),
        stop:        stop_item.id().clone(),
        reset:       reset_item.id().clone(),
        quit:        quit_item.id().clone(),
        kb_start:       kb_start.id().clone(),
        kb_stop:        kb_stop.id().clone(),
        kb_reset:       kb_reset.id().clone(),
        kb_quit:        kb_quit.id().clone(),
        kb_preferences: kb_preferences.id().clone(),
        start_item,
        stop_item,
        preferences_item:    prefs_item,
        reset_item,
        quit_item,
        kb_start_item:       kb_start,
        kb_stop_item:        kb_stop,
        kb_reset_item:       kb_reset,
        kb_quit_item:        kb_quit,
        kb_preferences_item: kb_preferences,
    };

    let menu = Menu::new();
    menu.append(&ids.preferences_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&ids.start_item)?;
    menu.append(&ids.stop_item)?;
    menu.append(&ids.reset_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&kb_submenu)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&ids.quit_item)?;

    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        // Treat the ring glyph as a template image on macOS: AppKit re-tints
        // template NSImages (white + alpha) based on menu-bar appearance, so
        // the icon reads correctly in both light and dark mode. No-op on
        // Windows/Linux.
        .with_icon_as_template(true)
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
            // White RGB so template-image tinting on macOS and plain display
            // on Windows/Linux both come out legible; alpha carries the shape.
            [0xFF, 0xFF, 0xFF, alpha]
        }))
        .collect();

    tray_icon::Icon::from_rgba(rgba, w, h).expect("tray icon")
}
