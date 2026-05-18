//! Platform-specific overlay / settings window setup.
//!
//! Public surface (identical on every target so main.rs doesn't need cfgs):
//!   - `setup_overlay_window`: make window click-through, always-on-top,
//!     spans every workspace/Space.
//!   - `setup_settings_window`: float above overlay, don't appear in taskbar.
//!   - `apply_app_visibility`: TopBarOnly/DockOnly/Both (tray vs taskbar).
//!   - `request_notification_permission`: no-op off macOS.
//!   - `register_reopen_handler`: no-op off macOS.
//!   - `DOCK_REOPEN`: atomic flag (always defined; only macOS sets it).
//!
//! The per-OS implementations live in submodules
//! (`platform/{mac,win,linux}.rs`).  This file is the API layer:
//! it owns the cross-platform globals, declares the submodules
//! conditionally, re-exports the right submodule's symbols, and
//! supplies no-op stubs for symbols that only exist on a subset of
//! platforms.  That keeps `main.rs` and `settings_window.rs` cfg-free.

use std::sync::atomic::AtomicBool;

#[allow(unused_imports)] // brought into scope for the submodules via `use super::*;`
use exhale_core::types::AppVisibility;
#[allow(unused_imports)]
use winit::window::Window;

/// Set when the macOS Dock icon is clicked while the app is already running.
/// Defined unconditionally so callers don't need `cfg` around the read.
pub static DOCK_REOPEN: AtomicBool = AtomicBool::new(false);

/// True after `install_settings_vibrancy` has successfully attached a blur
/// effect to the settings window on the current platform (macOS VEV
/// child-window, Windows DWM acrylic, Linux KDE blur-behind).  Read from
/// the egui render path so we know whether to clear at alpha 0 + paint
/// transparent panels (blur active) or fall back to opaque rendering.
static BLUR_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Public read-side accessor for [`BLUR_ACTIVE`].
pub fn is_blur_active() -> bool {
    BLUR_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}

#[allow(dead_code)] // used only on platforms where install_settings_vibrancy succeeds
fn set_blur_active(active: bool) {
    BLUR_ACTIVE.store(active, std::sync::atomic::Ordering::Relaxed);
}

// ─── Per-OS implementation modules ───────────────────────────────────────────

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::{
    activate_running_exhale, apply_app_visibility, install_main_menu,
    install_settings_vibrancy, render_sf_symbol, show_reset_alert,
    sync_settings_backdrop_frame,
    uninstall_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::{
    apply_app_visibility, install_settings_vibrancy, is_topmost_top,
    reassert_overlay_topmost, sync_settings_backdrop_frame, uninstall_settings_vibrancy,
    update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(all(unix, not(target_os = "macos")))]
pub use linux::{
    apply_app_visibility, install_settings_vibrancy, sync_settings_backdrop_frame,
    uninstall_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

// ─── Cross-platform stubs for symbols that only exist on a subset of OSes ────

/// Non-macOS stub for `install_main_menu`.  Windows and Linux apps
/// don't have a unified menu-bar concept — the settings window's own
/// in-window controls are the UI surface there.
#[cfg(not(target_os = "macos"))]
pub fn install_main_menu() {}

/// Non-macOS stub for `show_reset_alert`.  Returns `false` so callers
/// fall back to the in-window egui reset confirmation, which is the
/// only path on Windows / Linux.  The actual `do_reset_with_confirm`
/// branch in main.rs cfg-out the macOS path on non-macOS, so this
/// stub is never called — `allow(dead_code)` to silence the linter
/// while keeping the API surface consistent across platforms.
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn show_reset_alert() -> bool { false }

/// Non-macOS stub for `render_sf_symbol` — SF Symbols are AppKit-only.
/// Callers fall back to Unicode glyphs when this returns `None`.
#[cfg(not(target_os = "macos"))]
pub fn render_sf_symbol(_name: &str, _point_size: f64, _dark_mode: bool) -> Option<(Vec<u8>, u32, u32)> {
    None
}

/// Non-Windows no-op for `reassert_overlay_topmost`.  Only Windows
/// orders topmost-windows by activation in a way that lets a newly-
/// opened app rise above ours — macOS pins by window level, Linux X11
/// pins by EWMH state, neither needs periodic re-assertion.  Callers
/// are themselves cfg-gated to Windows (see `App::maybe_reassert_topmost`
/// and the `topmost_deadline` wake schedule in `about_to_wait`), so
/// this stub is only used in the rare cross-platform code path that
/// shouldn't ever fire.  `allow(dead_code)` because the call sites
/// are walled off by cfg and the linter can't see they don't exist
/// on this target.
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn reassert_overlay_topmost(_window: &winit::window::Window) {}
