//! Linux (X11) implementation of the platform layer.
//! See the parent `platform` module for the public API surface and
//! cross-platform stubs.

use super::*;

    use std::ffi::CString;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    // x11-dl's `x11_link!` macro generates a type named `Xlib` in every
    // module it's invoked in (including `xfixes`), so the Xfixes handle is
    // `x11_dl::xfixes::Xlib`, not `x11_dl::xfixes::Xfixes`. Alias it for
    // readability.
    use x11_dl::{xfixes::Xlib as Xfixes, xlib::{Display, Xlib, XClientMessageEvent, XEvent, ClientMessage}};

    struct X11<'a> {
        xlib:    &'a Xlib,
        xfixes:  Option<&'a Xfixes>,
        display: *mut Display,
        window:  u64,
    }

    impl<'a> X11<'a> {
        fn open(xlib: &'a Xlib, xfixes: Option<&'a Xfixes>, window: u64) -> Option<Self> {
            let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
            if display.is_null() { return None; }
            Some(Self { xlib, xfixes, display, window })
        }

        fn atom(&self, name: &[u8]) -> x11_dl::xlib::Atom {
            // Every call site below passes a `b"_NET_WM_STATE…"`
            // byte-string literal; none contain interior NULs.  If a
            // future caller passes user input here, `CString::new`
            // returning `Err` would yield `Atom(0)` (the X11 sentinel
            // for "no such atom") rather than panicking — same as
            // any other lookup miss.
            let Ok(c) = CString::new(name) else { return 0; };
            unsafe { (self.xlib.XInternAtom)(self.display, c.as_ptr(), 0) }
        }

        /// ClientMessage to root for `_NET_WM_STATE_{ADD,REMOVE}`.
        fn set_wm_state(&self, atom_name: &[u8], add: bool) {
            let state_atom = self.atom(b"_NET_WM_STATE");
            let target     = self.atom(atom_name);
            if state_atom == 0 || target == 0 { return; }

            unsafe {
                let root = (self.xlib.XDefaultRootWindow)(self.display);

                let mut ev: XClientMessageEvent = std::mem::zeroed();
                ev.type_        = ClientMessage;
                ev.window       = self.window;
                ev.message_type = state_atom;
                ev.format       = 32;
                ev.data.set_long(0, if add { 1 } else { 0 });
                ev.data.set_long(1, target as i64);
                ev.data.set_long(2, 0);
                ev.data.set_long(3, 1); // source = application

                let mut xevent: XEvent = std::mem::zeroed();
                xevent.client_message = ev;

                // SubstructureRedirectMask | SubstructureNotifyMask
                let mask: i64 = (1 << 20) | (1 << 19);
                (self.xlib.XSendEvent)(self.display, root, 0, mask, &mut xevent);
                (self.xlib.XFlush)(self.display);
            }
        }

        /// Empty input-region via XFixes = click-through.
        fn set_click_through(&self) {
            let Some(xfixes) = self.xfixes else { return; };
            unsafe {
                let region = (xfixes.XFixesCreateRegion)(self.display, std::ptr::null_mut(), 0);
                // ShapeInput = 2 per XFixes spec.
                (xfixes.XFixesSetWindowShapeRegion)(
                    self.display, self.window, 2, 0, 0, region,
                );
                (xfixes.XFixesDestroyRegion)(self.display, region);
                (self.xlib.XFlush)(self.display);
            }
        }
    }

    impl<'a> Drop for X11<'a> {
        fn drop(&mut self) {
            unsafe { (self.xlib.XCloseDisplay)(self.display); }
        }
    }

    fn x11_window(window: &Window) -> Option<u64> {
        let handle = window.window_handle().ok()?;
        if let RawWindowHandle::Xlib(h) = handle.as_raw() {
            Some(h.window)
        } else {
            None
        }
    }

    pub fn setup_overlay_window(window: &Window) {
        let Some(xwin) = x11_window(window) else { return; };
        let Ok(xlib)   = Xlib::open() else { return; };
        let xfixes     = Xfixes::open().ok();
        let Some(x)    = X11::open(&xlib, xfixes.as_ref(), xwin) else { return; };

        x.set_click_through();
        // `_NET_WM_STATE_FULLSCREEN` is what makes the overlay cover
        // the panel / dock on EWMH-compliant window managers.  Without
        // it, GNOME-Shell / Mutter / Xfwm reserve the dock area and
        // force our window into the work-area rectangle, leaving a
        // visible gap where the dock sits — even when we requested
        // monitor-spanning geometry from winit.  `_NET_WM_STATE_ABOVE`
        // (kept below) is for stacking against other normal windows;
        // FULLSCREEN is for covering struts / panels.
        x.set_wm_state(b"_NET_WM_STATE_FULLSCREEN",   true);
        x.set_wm_state(b"_NET_WM_STATE_ABOVE",        true);
        x.set_wm_state(b"_NET_WM_STATE_STICKY",       true);
        x.set_wm_state(b"_NET_WM_STATE_SKIP_TASKBAR", true);
        x.set_wm_state(b"_NET_WM_STATE_SKIP_PAGER",   true);
    }

    pub fn setup_settings_window(window: &Window) {
        // Mark the settings window `_NET_WM_STATE_ABOVE` so it can rise
        // above the breathing overlay (also `ABOVE`).  X11 has no
        // explicit window levels — among `ABOVE` windows, activation
        // order determines z-stacking, so opening preferences activates
        // it and brings it forward.  Without this hint, EWMH-compliant
        // window managers permanently order the overlay (which is
        // ABOVE) on top of the settings (NORMAL), even when settings is
        // focused.  Wayland compositors that ignore EWMH will fall back
        // to their own stacking — documented limitation, same as the
        // overlay's click-through hint.
        //
        // `apply_app_visibility` still handles `SKIP_TASKBAR/SKIP_PAGER`
        // for the Top-Bar-only mode, independent of this hint.
        let Some(xwin) = x11_window(window) else { return; };
        let Ok(xlib)   = Xlib::open() else { return; };
        let Some(x)    = X11::open(&xlib, None, xwin) else { return; };
        x.set_wm_state(b"_NET_WM_STATE_ABOVE", true);
    }

    /// No-op on Linux: the settings window is OPAQUE on every Linux DE.
    /// KDE/KWin's `_KDE_NET_WM_BLUR_BEHIND_REGION` would give a frosted
    /// settings window on Plasma but produces the same compositing
    /// regressions seen on Windows DWM acrylic (overlay stacking above
    /// the controls, mouse-hover lag).  `BLUR_ACTIVE` stays `false`, so
    /// the egui clear colour + panel fill render the themed solid
    /// background.  macOS is the only platform with a translucent
    /// settings backdrop.
    pub fn install_settings_vibrancy(_window: &Window, _dark_mode: bool) -> usize {
        0
    }

    /// No theme-dependent state to update on X11 — KDE follows the
    /// system theme automatically once the blur property is set.
    pub fn update_settings_vibrancy(_vev_ptr: usize, _dark_mode: bool) {}

    /// No-op on X11 (no backdrop window to keep in sync — the blur
    /// region attaches to the settings window itself).
    pub fn sync_settings_backdrop_frame(_backdrop_ptr: usize) {}

    /// No-op on Linux — `install_settings_vibrancy` returned 0, no
    /// backdrop NSWindow to release.  KDE blur attaches to the
    /// settings window itself and is cleaned up when the window drops.
    pub fn uninstall_settings_vibrancy(_backdrop_ptr: usize) {}

    pub fn request_notification_permission() {
        // D-Bus org.freedesktop.Notifications needs no per-app permission.
    }

    pub fn register_reopen_handler() { /* no analog */ }

    pub fn apply_app_visibility(vis: AppVisibility, settings: Option<&Window>) {
        let Some(win)  = settings else { return; };
        let Some(xwin) = x11_window(win) else { return; };
        let Ok(xlib)   = Xlib::open() else { return; };
        let Some(x)    = X11::open(&xlib, None, xwin) else { return; };

        let hide = matches!(vis, AppVisibility::TopBarOnly);
        x.set_wm_state(b"_NET_WM_STATE_SKIP_TASKBAR", hide);
        x.set_wm_state(b"_NET_WM_STATE_SKIP_PAGER",   hide);
    }
