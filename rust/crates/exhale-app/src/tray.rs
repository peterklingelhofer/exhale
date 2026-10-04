use anyhow::Result;
use exhale_core::{KeyboardShortcuts, ShortcutAction};
use tray_icon::{
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};
// Only the "Keyboard Shortcuts ▶" submenu needs a `Submenu`, and that
// submenu doesn't exist without the feature
#[cfg(feature = "global-hotkeys")]
use tray_icon::menu::Submenu;
#[cfg(feature = "global-hotkeys")]
use crate::hotkeys;

// ─── Research link ────────────────────────────────────────────────────────────

/// The corpus page, opened at the top so the reader gets the reading
/// guide (verification status, access level, evidence tier) before the
/// entries and the gaps ledger the page links to from its first screen.
/// `scripts/generate-citations.py` validates that this URL still points
/// into the corpus, and that any anchor it carries still names a heading
///
/// Pinned to `main` instead of a release tag.  A binary
/// stays installed long after its tag stops being the current state
/// of the evidence, and a retraction has to reach the people running
/// old builds.  A moved anchor is a CI failure, but a stale claim isn't
///
/// Points at `docs/citations.html` instead of the file itself.  That
/// page fetches `docs/CITATIONS.md` from `main` at load, so it stays
/// as current as the blob view did, but a reader who followed a menu
/// item called "Research" arrives at a document instead of at a code
/// host's file browser
pub const RESEARCH_URL: &str = "https://peterklingelhofer.github.io/exhale/citations.html";

/// Named next to the URL because the wording and the destination are one
/// decision: a plain label pointing at the whole corpus, instead of a
/// claim-shaped label pointing at a supporting entry.  Shared with the
/// macOS app menu, which shows the same item, so the two can never
/// disagree
pub const RESEARCH_LABEL: &str = "Research";

// ─── Menu item IDs ────────────────────────────────────────────────────────────

/// Every action a shortcut can bind, in the order the "Keyboard
/// Shortcuts ▶" submenu lists them.  `hotkeys::register_hotkeys`
/// registers them in this order too
pub const ACTIONS: [ShortcutAction; 5] = [
    ShortcutAction::Start,
    ShortcutAction::Stop,
    ShortcutAction::Reset,
    ShortcutAction::Quit,
    ShortcutAction::Preferences,
];

pub struct TrayMenuIds {
    // Top-level item handles whose labels include the current
    // keybinding: kept here so the rebind path can `set_text` them
    // when the user changes a shortcut, instead of rebuilding the
    // whole tray.  The Start and Stop entries are also the handles
    // for dynamic enable/disable
    top:          [(ShortcutAction, MenuItem); 5],
    // No binding is embedded in this one's label, so `refresh_labels`
    // never touches it
    pub research: MenuItem,
    // ── "Keyboard Shortcuts ▶" submenu ────────────────────────────────────────
    //
    // Each entry both displays the action's current binding (label
    // text via `set_text` on rebind) and acts as a click target that
    // opens the settings window in capture mode for that action.
    // Storing the handles here lets us update labels in place without
    // a tray rebuild.  Doesn't exist without the feature: a source
    // build is the only one that can ever register a binding
    #[cfg(feature = "global-hotkeys")]
    kb:           [(ShortcutAction, MenuItem); 5],
}

impl TrayMenuIds {
    /// Match a clicked tray-menu item id back to the
    /// [`ShortcutAction`] whose binding the user wants to change.
    /// Returns `None` for items that aren't part of the
    /// "Keyboard Shortcuts ▶" submenu
    #[cfg(feature = "global-hotkeys")]
    pub fn kb_action_for(&self, id: &MenuId) -> Option<ShortcutAction> {
        action_for(&self.kb, id)
    }

    /// Match a clicked tray-menu item id back to the top-level item's
    /// [`ShortcutAction`].  Returns `None` for Research and for the
    /// "Keyboard Shortcuts ▶" submenu rows
    pub fn top_action_for(&self, id: &MenuId) -> Option<ShortcutAction> {
        action_for(&self.top, id)
    }

    /// The top-level item for `action`.  `top` is built from
    /// [`ACTIONS`], so every action has one
    pub fn top_item(&self, action: ShortcutAction) -> &MenuItem {
        let (_, item) = self.top.iter()
            .find(|(a, _)| *a == action)
            .expect("`top` holds an item for every action");
        item
    }

    /// Refresh every label that embeds a keyboard-shortcut binding
    /// after the user reassigns one.  Called from the rebind path so
    /// the tray menu stays in sync with `settings.keyboard_shortcuts`
    /// without a full tray rebuild (which would flash the tray icon)
    #[cfg(feature = "global-hotkeys")]
    pub fn refresh_labels(&self, shortcuts: &KeyboardShortcuts) {
        for (action, item) in &self.top { item.set_text(top_level_label(*action, shortcuts)); }
        for (action, item) in &self.kb  { item.set_text(submenu_label(*action, shortcuts)); }
    }
}

/// The action whose item in `items` carries `id`
fn action_for(items: &[(ShortcutAction, MenuItem)], id: &MenuId) -> Option<ShortcutAction> {
    items.iter().find(|(_, item)| item.id() == id).map(|&(action, _)| action)
}

/// Format a top-level menu item's label.  Embeds the current
/// binding in parentheses so the user can read it without opening
/// the submenu.  Reads "Preferences" when the slot is unbound
#[cfg(feature = "global-hotkeys")]
fn top_level_label(action: ShortcutAction, shortcuts: &KeyboardShortcuts) -> String {
    let base = top_label(action);
    if !hotkeys::shortcuts_available() {
        return base.to_string();
    }
    match shortcuts.get(action) {
        Some(sc) => format!("{base}  ({})", sc.display()),
        None     => base.to_string(),
    }
}

/// No shortcut can ever fire without the feature, so the label stays
/// plain instead of claiming a binding that would never work
#[cfg(not(feature = "global-hotkeys"))]
fn top_level_label(action: ShortcutAction, _shortcuts: &KeyboardShortcuts) -> String {
    top_label(action).to_string()
}

/// A top-level item's text, before `top_level_label` appends the binding
fn top_label(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::Start       => "Start Animation",
        ShortcutAction::Stop        => "Stop Animation",
        ShortcutAction::Reset       => "Reset to Defaults",
        ShortcutAction::Quit        => "Quit exhale",
        ShortcutAction::Preferences => "Preferences",
    }
}

/// Format a "Keyboard Shortcuts ▶" submenu item.  Action name on
/// the left, current binding (or "(none)") on the right.  Clicking
/// the row opens settings in capture mode for the matching action
#[cfg(feature = "global-hotkeys")]
fn submenu_label(action: ShortcutAction, shortcuts: &KeyboardShortcuts) -> String {
    let binding = shortcuts
        .get(action)
        .map(|sc| sc.display())
        .unwrap_or_else(|| "(none)".to_string());
    format!("{}: {binding}", action.label())
}

/// Build the system-tray icon + menu, returning the `TrayIcon` handle
/// (must stay alive) and the menu item IDs so the caller can match
/// incoming `MenuEvent`s
///
/// `shortcuts` is the current snapshot of user keybindings.  Labels
/// embed each action's binding so the user can see at a glance what's
/// bound to what without leaving the menu.  Pass the same struct back
/// to [`TrayMenuIds::refresh_labels`] when bindings change to keep
/// the menu in sync
pub fn build_tray(shortcuts: &KeyboardShortcuts) -> Result<(TrayIcon, TrayMenuIds)> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if !appindicator_available() {
            anyhow::bail!(
                "neither libayatana-appindicator3 nor libappindicator3 could be loaded, \
                 so exhale runs without a tray icon"
            );
        }
    }

    // Propagate icon-construction failures via `?` instead of
    // panicking: callers (`App::sync_tray_to_visibility`) already
    // log + continue when `build_tray` returns `Err`, so a bad
    // RGBA buffer or platform limitation degrades gracefully to
    // "no tray icon" instead of aborting the whole process at
    // launch.  In practice the buffer is hardcoded RGBA we
    // generate ourselves, so this branch is never expected to
    // fire, but a `.expect()` here was the difference between
    // "app didn't open" and "log line + app keeps running"
    let icon = make_icon()?;

    // No `Accelerator::new(...)` on any item.  Reasons:
    //   1. Bindings are user-customisable now.  A static accelerator
    //      label would lie when the user reassigns a shortcut
    //   2. On macOS, an `Accelerator` becomes the NSMenuItem's
    //      `keyEquivalent`, which fires while the menu is open.  The
    //      same key press also queues in the global-hotkey channel,
    //      so closing the menu plays the action a second time: a
    //      double-trigger bug
    // Embed the binding in the label text instead so it stays correct
    // and avoids the dual-dispatch hazard
    let top = ACTIONS.map(|action| (action, MenuItem::new(top_level_label(action, shortcuts), true, None)));
    let research = MenuItem::new(RESEARCH_LABEL, true, None);

    // ── Keyboard Shortcuts submenu ────────────────────────────────────────────
    // Only a `global-hotkeys` build can register a binding, so a
    // `--no-default-features` build never builds this submenu at all
    #[cfg(feature = "global-hotkeys")]
    let kb = ACTIONS.map(|action| (action, MenuItem::new(submenu_label(action, shortcuts), true, None)));

    #[cfg(feature = "global-hotkeys")]
    let kb_submenu = Submenu::new("Keyboard Shortcuts", true);
    #[cfg(feature = "global-hotkeys")]
    for (action, item) in &kb {
        // Preferences sits below a separator, apart from the other four
        if *action == ShortcutAction::Preferences {
            kb_submenu.append(&PredefinedMenuItem::separator())?;
        }
        kb_submenu.append(item)?;
    }

    #[cfg(feature = "global-hotkeys")]
    let ids = TrayMenuIds { top, research, kb };
    #[cfg(not(feature = "global-hotkeys"))]
    let ids = TrayMenuIds { top, research };

    let menu = Menu::new();
    menu.append(ids.top_item(ShortcutAction::Preferences))?;
    menu.append(&ids.research)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(ids.top_item(ShortcutAction::Start))?;
    menu.append(ids.top_item(ShortcutAction::Stop))?;
    menu.append(ids.top_item(ShortcutAction::Reset))?;
    // No submenu on a Wayland session either: nothing can ever bind,
    // so don't show a menu that promises otherwise
    #[cfg(feature = "global-hotkeys")]
    if hotkeys::shortcuts_available() {
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&kb_submenu)?;
    }
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(ids.top_item(ShortcutAction::Quit))?;

    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        // Treat the ring glyph as a template image on macOS: AppKit re-tints
        // template NSImages (white + alpha) based on menu-bar appearance, so
        // the icon reads correctly in both light and dark mode. No-op on
        // Windows/Linux
        .with_icon_as_template(true)
        .with_menu(Box::new(menu))
        .with_tooltip("exhale")
        .build()?;

    Ok((tray, ids))
}

/// Whether the library `tray-icon` needs on Linux can be loaded
///
/// `libappindicator-sys` dlopens it the first time a tray icon is built
/// and panics when none of its candidates load, so the `Result` from
/// `build_tray` never sees the failure and the whole app exits.  Probing
/// the same names first turns a system without the library (minimal
/// desktops, AppImageHub's test machine) into "no tray icon" instead.
/// The probe runs once. A handle that loads is left open, and the
/// crate's own dlopen reuses it
#[cfg(all(unix, not(target_os = "macos")))]
fn appindicator_available() -> bool {
    use std::ffi::CStr;
    use std::sync::OnceLock;

    // Same names, same order as `libappindicator-sys` 0.9
    const CANDIDATES: [&CStr; 4] = [
        c"libayatana-appindicator3.so.1",
        c"libappindicator3.so.1",
        c"libayatana-appindicator3.so",
        c"libappindicator3.so",
    ];
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        CANDIDATES.iter().any(|name| {
            // SAFETY: `name` is a NUL-terminated C string literal, and
            // dlopen has no other preconditions
            !unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL) }.is_null()
        })
    })
}

/// Outlined-ring tray icon generated at runtime, matching the Swift
/// `StatusBarIcon` asset (15×17 ring shape).  Drawn with anti-aliased edges
/// in near-black so it reads well on both light and dark menu bars,
/// returning `Result` so a failed `Icon::from_rgba` (corrupt buffer,
/// platform limitation) bubbles up through `build_tray` and the
/// caller can log + run without a tray instead of panicking at
/// startup
fn make_icon() -> Result<tray_icon::Icon> {
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
            // of the ring.  Alpha peaks at the band between `inner` and `outer`
            let aa_outer = (outer - d).clamp(0.0, 1.0);
            let aa_inner = (d - inner).clamp(0.0, 1.0);
            let alpha = (aa_outer.min(aa_inner) * 255.0) as u8;
            // White RGB so template-image tinting on macOS and plain display
            // on Windows/Linux both come out legible.  Alpha carries the shape
            [0xFF, 0xFF, 0xFF, alpha]
        }))
        .collect();

    tray_icon::Icon::from_rgba(rgba, w, h)
        .map_err(|e| anyhow::anyhow!("tray icon from_rgba: {e}"))
}
