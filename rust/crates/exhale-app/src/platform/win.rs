//! Windows implementation of the platform layer.
//! Extracted from `platform.rs` for navigability; see the parent
//! module for the public API surface + cross-platform stubs.

use super::*;

    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::{
        Foundation::HWND,
        Graphics::Dwm::{
            DwmSetWindowAttribute,
            // DWMWA_SYSTEMBACKDROP_TYPE + DWMSBT_TRANSIENTWINDOW were
            // tried for the settings-window acrylic look but produced
            // worse results than the WS_EX_LAYERED path we now use
            // for the overlay.  Kept in the import comment to flag
            // they exist if someone wants to revisit.
            DWMWA_USE_IMMERSIVE_DARK_MODE,
        },
        UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
            GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_EX_TOPMOST, WS_EX_TRANSPARENT,
        },
    };

    fn hwnd(window: &Window) -> HWND {
        // Fallible on a closing/destroyed window — return a null HWND
        // so callers that null-check (most of this module) become
        // no-ops instead of panicking during shutdown.
        let Ok(handle) = window.window_handle() else { return std::ptr::null_mut(); };
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
            h.hwnd.get() as HWND
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn setup_overlay_window(window: &Window) {
        let h = hwnd(window);
        if h.is_null() { return; }
        unsafe {
            // `WS_EX_LAYERED + WS_EX_TRANSPARENT` is the legacy
            // wgpu-compatible click-through transparency pattern on
            // Windows.  winit's `with_transparent(true)` already adds
            // `WS_EX_LAYERED` and calls `DwmEnableBlurBehindWindow` for
            // us, but we re-assert `WS_EX_LAYERED` defensively here in
            // case some other path stripped it.  Tool window +
            // NoActivate keep the overlay out of Alt-Tab / taskbar and
            // prevent focus theft; `WS_EX_TOPMOST` and the trailing
            // `SetWindowPos(HWND_TOPMOST, …)` are belt-and-suspenders
            // for the always-on-top requirement.
            let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as isize;
            let new_ex = ex
                | WS_EX_LAYERED     as isize
                | WS_EX_TRANSPARENT as isize
                | WS_EX_TOPMOST     as isize
                | WS_EX_TOOLWINDOW  as isize
                | WS_EX_NOACTIVATE  as isize;
            SetWindowLongPtrW(h, GWL_EXSTYLE, new_ex);
            SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);

            // Diagnostic — log the final extended style so we can verify
            // in the running app's log file which transparency path is
            // in effect.  `0x80000` = LAYERED, `0x200000` = NRB.  We
            // expect LAYERED set and NRB clear with this approach.
            let final_ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
            log::info!(
                "overlay extended-style after setup: 0x{final_ex:08x} \
                 (LAYERED={}, NRB={}, TRANSPARENT={}, TOPMOST={})",
                (final_ex & 0x0008_0000) != 0,
                (final_ex & 0x0020_0000) != 0,
                (final_ex & WS_EX_TRANSPARENT) != 0,
                (final_ex & WS_EX_TOPMOST)     != 0,
            );
        }
    }

    /// Re-bump the overlay HWND to the front of the topmost z-band.
    /// Windows orders topmost windows by activation, so a newly-opened
    /// app — even one without `WS_EX_TOPMOST` — can land above our
    /// overlay if the user activates it (the OS treats activation as a
    /// foreground promotion).  Calling `SetWindowPos(HWND_TOPMOST, …)`
    /// with `SWP_NOACTIVATE` re-asserts overlay topmost without
    /// stealing focus from whatever the user is currently working in.
    ///
    /// We don't reset window styles or geometry — just the z-order
    /// position — so the call is cheap (a few microseconds) and safe
    /// to invoke on a regular cadence from the overlay render loop.
    pub fn reassert_overlay_topmost(window: &Window) {
        let h = hwnd(window);
        if h.is_null() { return; }
        unsafe {
            SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }

    pub fn setup_settings_window(window: &Window) {
        // Mark the settings window topmost so it can rise ABOVE the
        // breathing overlay (which is also `WS_EX_TOPMOST`).  Windows
        // doesn't expose explicit z-bands like macOS's window levels, so
        // both windows share the topmost band and the most-recently-
        // activated one wins — when the user opens preferences, the
        // settings window comes to front; when the overlay later starts
        // animating, settings stays interactable until the user clicks
        // away from it.  Without `WS_EX_TOPMOST`, the settings window
        // would render permanently behind the topmost overlay's
        // (translucent) layer — invisible to the user despite still
        // technically being focused.
        let h = hwnd(window);
        if h.is_null() { return; }
        unsafe {
            let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as isize;
            let new_ex = ex | WS_EX_TOPMOST as isize;
            if new_ex != ex {
                SetWindowLongPtrW(h, GWL_EXSTYLE, new_ex);
                SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }

    /// Set the settings-window title bar to dark mode when the OS is in
    /// dark appearance.  We deliberately do NOT install the DWM acrylic
    /// backdrop (`DWMWA_SYSTEMBACKDROP_TYPE`) here even though it would
    /// produce a frosted-glass settings window — every attempt at that
    /// path on Windows produced two visible regressions:
    ///
    ///   1. The breath overlay composited above the settings window's
    ///      DWM-translucent client area in the z-stack, so the
    ///      animation rendered IN FRONT of the controls (and at
    ///      opacity = 1 hid them entirely with no way to recover).
    ///   2. Mouse hover over the acrylic settings window triggered
    ///      a per-cursor-move recomposition of the whole DWM acrylic
    ///      stack, producing severe overlay-animation lag.
    ///
    /// The dark-titlebar attribute is independent of compositing — it
    /// just changes the non-client area's tint — so we keep that.  The
    /// `BLUR_ACTIVE` flag stays `false`, which makes the egui render
    /// path paint the settings window OPAQUELY (clear colour =
    /// themed panel, panel fill = themed panel), avoiding both
    /// regressions above.
    pub fn install_settings_vibrancy(window: &Window, dark_mode: bool) -> usize {
        let h = hwnd(window);
        if h.is_null() { return 0; }
        apply_dark_titlebar(h, dark_mode);
        // Return the HWND so the theme-change path can re-apply the
        // dark-titlebar attribute via `update_settings_vibrancy`.
        // `BLUR_ACTIVE` stays false → opaque rendering everywhere.
        h as usize
    }

    /// On Windows the only theme-dependent property is the title bar
    /// dark-mode flag — re-apply it when the OS appearance changes.
    /// `handle` is the HWND returned by `install_settings_vibrancy`.
    pub fn update_settings_vibrancy(handle: usize, dark_mode: bool) {
        if handle == 0 { return; }
        apply_dark_titlebar(handle as HWND, dark_mode);
    }

    /// Set the dark-mode title-bar attribute on `h` via DWM.
    /// `DWMWA_USE_IMMERSIVE_DARK_MODE` is silently ignored ("feature
    /// not present" error) on Win10 builds older than 1809, so this
    /// is no-op-safe on earlier OSes
    fn apply_dark_titlebar(h: HWND, dark_mode: bool) {
        // Pass a Win32 BOOL — `i32` 1 / 0 — by pointer.
        let dark_bool: i32 = if dark_mode { 1 } else { 0 };
        // SAFETY: `h` is a valid HWND (caller null-checked or
        // unwrapped from the live winit window via `hwnd()`).  The
        // pointer + size pair describe a stack-local i32, in scope
        // for the duration of the call.  `DwmSetWindowAttribute` has
        // no Rust-level safety requirements beyond a valid HWND.
        unsafe {
            let _ = DwmSetWindowAttribute(
                h,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                &dark_bool as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }

    /// No-op on Windows — DWM tracks window size via the HWND, no
    /// separate backdrop window to resize.
    pub fn sync_settings_backdrop_frame(_backdrop_ptr: usize) {}

    /// No-op on Windows — `install_settings_vibrancy` returned 0, no
    /// backdrop NSWindow to release.
    pub fn uninstall_settings_vibrancy(_backdrop_ptr: usize) {}

    pub fn request_notification_permission() {
        // notify-rust on Windows uses WinRT ToastNotifications which don't
        // require explicit permission from the app.
    }

    pub fn register_reopen_handler() { /* no analog */ }

    /// On Windows "Dock" == taskbar entry on the settings window.
    ///   DockOnly → show in taskbar (remove WS_EX_TOOLWINDOW, add WS_EX_APPWINDOW)
    ///   others   → hide from taskbar (add WS_EX_TOOLWINDOW, drop WS_EX_APPWINDOW)
    pub fn apply_app_visibility(vis: AppVisibility, settings: Option<&Window>) {
        let Some(win) = settings else { return; };
        let h = hwnd(win);
        if h.is_null() { return; }
        unsafe {
            let mut ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as isize;
            match vis {
                AppVisibility::DockOnly | AppVisibility::Both => {
                    ex &= !(WS_EX_TOOLWINDOW as isize);
                    ex |=   WS_EX_APPWINDOW  as isize;
                }
                AppVisibility::TopBarOnly => {
                    ex |=   WS_EX_TOOLWINDOW as isize;
                    ex &= !(WS_EX_APPWINDOW  as isize);
                }
            }
            SetWindowLongPtrW(h, GWL_EXSTYLE, ex);
        }
    }
