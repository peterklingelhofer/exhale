//! Platform-specific overlay / settings window setup
//!
//! Public surface (identical on every target so main.rs doesn't need cfgs):
//!   - `setup_overlay_window`: make window click-through, always-on-top,
//!     spans every workspace/Space
//!   - `setup_settings_window`: float above overlay, don't appear in taskbar
//!   - `apply_app_visibility`: TopBarOnly/DockOnly/Both (tray vs taskbar)
//!   - `request_notification_permission`: no-op off macOS
//!   - `register_reopen_handler`: no-op off macOS
//!   - `DOCK_REOPEN`: atomic flag (always defined, only macOS sets it)
//!
//! The per-OS implementations live in submodules
//! (`platform/{mac,win,linux}.rs`).  This file is the API layer: it
//! owns the cross-platform globals, declares the submodules
//! conditionally, re-exports the right submodule's symbols, and
//! supplies no-op stubs for symbols that only exist on a subset of
//! platforms.  That keeps `main.rs` and `settings_window.rs` cfg-free

use std::sync::atomic::AtomicBool;

#[allow(unused_imports)] // brought into scope for the submodules via `use super::*;`
use exhale_core::types::AppVisibility;
#[allow(unused_imports)]
use winit::window::Window;

/// Set when the macOS Dock icon is clicked while the app is already
/// running.  Defined unconditionally so callers don't need `cfg` around
/// the read
pub static DOCK_REOPEN: AtomicBool = AtomicBool::new(false);

/// True after `install_settings_vibrancy` has successfully attached a blur
/// effect to the settings window on the current platform (macOS VEV
/// child-window, Windows DWM acrylic, Linux KDE blur-behind).  Read from
/// the egui render path so we know whether to clear at alpha 0 + paint
/// transparent panels (blur active) or fall back to opaque rendering
static BLUR_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Public read-side accessor for [`BLUR_ACTIVE`]
pub fn is_blur_active() -> bool {
    BLUR_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}

#[allow(dead_code)] // used only on platforms where install_settings_vibrancy succeeds
fn set_blur_active(active: bool) {
    BLUR_ACTIVE.store(active, std::sync::atomic::Ordering::Relaxed);
}

// ─── Wayland session detection ────────────────────────────────────────────────

/// Whether the current session is Wayland.  Only Linux/BSD have a
/// Wayland/X11 split to detect, so every other OS reports `false`.
/// Single source of truth for the overlay's X11-only click-through
/// check and, behind `global-hotkeys`, for `hotkeys::shortcuts_available`
pub fn is_wayland_session() -> bool {
    #[cfg(all(unix, not(target_os = "macos")))]
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    #[cfg(not(all(unix, not(target_os = "macos"))))]
    let session_type: Option<String> = None;

    is_wayland_session_type(session_type.as_deref())
}

/// Pure decision behind [`is_wayland_session`]: does this
/// `XDG_SESSION_TYPE` value (`None` when unset) mean Wayland?  Split
/// out so the comparison is unit-testable without touching the
/// environment
fn is_wayland_session_type(session_type: Option<&str>) -> bool {
    session_type.is_some_and(|s| s.eq_ignore_ascii_case("wayland"))
}

// ─── Per-OS implementation modules ───────────────────────────────────────────

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::{
    activate_running_exhale, apply_app_visibility, install_main_menu,
    install_settings_vibrancy, render_sf_symbol,
    sync_settings_backdrop_frame,
    uninstall_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, running_in_app_bundle, setup_overlay_window,
    setup_settings_window,
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
/// don't have a unified menu-bar concept: the settings window's own
/// in-window controls are the UI surface there
#[cfg(not(target_os = "macos"))]
pub fn install_main_menu() {}

/// Non-macOS stub for `render_sf_symbol`: SF Symbols are AppKit-only.
/// Callers fall back to Unicode glyphs when this returns `None`
#[cfg(not(target_os = "macos"))]
pub fn render_sf_symbol(_name: &str, _point_size: f64, _dark_mode: bool) -> Option<(Vec<u8>, u32, u32)> {
    None
}

/// Non-Windows no-op for `reassert_overlay_topmost`.  Only Windows
/// orders topmost-windows by activation in a way that lets a newly-
/// opened app rise above ours.  macOS pins by window level, Linux X11
/// pins by EWMH state, neither needs periodic re-assertion.  Callers
/// are themselves cfg-gated to Windows (see `App::maybe_reassert_topmost`
/// and the `topmost_deadline` wake schedule in `about_to_wait`), so
/// this stub is only used in the rare cross-platform code path that
/// shouldn't ever fire.  `allow(dead_code)` because the call sites
/// are walled off by cfg and the linter can't see they don't exist
/// on this target
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn reassert_overlay_topmost(_window: &winit::window::Window) {}

// ─── Opening a URL in the user's browser ─────────────────────────────────────

/// Hand `url` to the user's default browser
///
/// This is the only outbound-link path in the app, so the scheme
/// check lives here instead of at each call site: anything that
/// isn't a plain `https://` URL is dropped with a log line and never
/// reaches the platform API.  `ShellExecuteW` in particular will
/// happily launch a local executable for a `file:` URL, and
/// `xdg-open` will hand an arbitrary scheme to whatever handler
/// claims it, so the allowlist is load-bearing instead of
/// decorative, even though every current caller passes a
/// compile-time constant
///
/// A machine with no browser, no `xdg-open`, or a refused
/// `NSWorkspace` open is still a fully working exhale by design.  The
/// documentation is on the web either way.  Failures log and return
pub fn open_url(url: &str) {
    if !is_openable(url) {
        log::warn!("open_url: refusing non-https or malformed URL: {url:?}");
        return;
    }

    #[cfg(target_os = "macos")]
    mac::open_url_impl(url);
    #[cfg(target_os = "windows")]
    win::open_url_impl(url);
    #[cfg(all(unix, not(target_os = "macos")))]
    linux::open_url_impl(url);
}

/// The allowlist [`open_url`] enforces, split out so it can be tested
/// without launching a browser
///
/// Rejecting on the whole `https://` prefix (instead of "starts with
/// http") also rules out `https:/evil` and scheme-relative junk, and
/// the control-character check keeps anything unprintable out of a
/// command argument on the Linux path.  There's no `http://` escape
/// hatch: every destination this app links to is a GitHub URL that
/// redirects to TLS anyway
fn is_openable(url: &str) -> bool {
    url.starts_with("https://")
        && url.len() > "https://".len()
        && !url.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_session_type_is_case_insensitive() {
        assert!(is_wayland_session_type(Some("wayland")));
        assert!(is_wayland_session_type(Some("Wayland")));
        assert!(!is_wayland_session_type(Some("x11")));
        assert!(!is_wayland_session_type(None));
    }

    #[test]
    fn only_plain_https_urls_are_openable() {
        assert!(is_openable("https://github.com/peterklingelhofer/exhale#a"));

        // `ShellExecuteW` would resolve this through the shell
        // association table and launch a local program
        assert!(!is_openable("file:///etc/passwd"));
        assert!(!is_openable("http://example.com"));
        // Substring-of-scheme and scheme-relative near-misses
        assert!(!is_openable("https:/example.com"));
        assert!(!is_openable("//example.com"));
        assert!(!is_openable(" https://example.com"));
        // A bare scheme resolves to nothing.  Reject instead of
        // handing an empty host to three different platform APIs
        assert!(!is_openable("https://"));
        assert!(!is_openable(""));
        // Newline injection into the argument of a spawned process
        assert!(!is_openable("https://example.com\nrm -rf /"));
        assert!(!is_openable("https://example.com\u{0}"));
    }

    /// The tray constant is the only URL that ships, so the
    /// allowlist it has to pass is asserted here instead of left to
    /// the code review that introduced it
    #[test]
    fn shipped_research_url_passes_the_allowlist() {
        assert!(is_openable(crate::tray::RESEARCH_URL));
        assert!(
            crate::tray::RESEARCH_URL.ends_with("/exhale/citations.html"),
            "the menu item opens the corpus page at the top; a fragment \
             would need a heading that scripts/generate-citations.py checks"
        );
    }
}
