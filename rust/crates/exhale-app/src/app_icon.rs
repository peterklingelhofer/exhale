//! App-icon embedding for window managers / dock / Alt-Tab / taskbar.
//!
//! The build script (`build.rs`) pre-decodes the shared
//! `exhaleColorGradient512.png` to a 256×256 RGBA buffer at compile
//! time and writes it to `$OUT_DIR/icon.rgba`.  We `include_bytes!`
//! that here so no PNG decoder is needed at run time and the icon
//! travels inside the binary itself.  No external `.desktop` file is
//! required for the icon to appear on Linux Wayland / X11 — the
//! compositor uses the icon attached to the window directly.
//!
//! Windows has a separate path (the `.exe` resource set by `build.rs`)
//! for the file-explorer / Start-menu / Alt-Tab icon, but we set the
//! window icon here too so per-window UI (Alt-Tab tooltip on Win11,
//! the title-bar app icon if shown) matches.
//!
//! macOS ignores window icons (`NSWindow` doesn't have a per-window
//! icon concept) — the Dock / Cmd-Tab icon comes from the .app
//! bundle's `Info.plist` + `.icns` instead.  Calling
//! `WindowAttributes::with_window_icon` on macOS is a no-op, not an
//! error.

const ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon.rgba"));
const ICON_W: u32 = 256;
const ICON_H: u32 = 256;

/// Build a winit `Icon` from the embedded RGBA bytes.  Returns `None`
/// only if the build script failed to produce a valid 256×256 buffer
/// (e.g. the source PNG was missing or unreadable); the binary still
/// runs in that case, the windows just fall back to the platform's
/// default icon.
pub(crate) fn window_icon() -> Option<winit::window::Icon> {
    if ICON_RGBA.len() != (ICON_W * ICON_H * 4) as usize {
        return None;
    }
    winit::window::Icon::from_rgba(ICON_RGBA.to_vec(), ICON_W, ICON_H).ok()
}
