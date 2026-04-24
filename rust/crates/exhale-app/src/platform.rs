/// Platform-specific overlay / settings window setup.
///
/// Public surface (identical on every target so main.rs doesn't need cfgs):
///   • `setup_overlay_window`   — make window click-through, always-on-top,
///                                spans every workspace/Space.
///   • `setup_settings_window`  — float above overlay, don't appear in taskbar.
///   • `apply_app_visibility`   — TopBarOnly/DockOnly/Both (tray vs taskbar).
///   • `request_notification_permission` — no-op off macOS.
///   • `register_reopen_handler` — no-op off macOS.
///   • `DOCK_REOPEN`            — atomic flag (always defined; only macOS sets it).
///
/// These wrappers mean main.rs can call the platform layer unconditionally.

use std::sync::atomic::AtomicBool;

use exhale_core::types::AppVisibility;
use winit::window::Window;

/// Set when the macOS Dock icon is clicked while the app is already running.
/// Defined unconditionally so callers don't need `cfg` around the read.
pub static DOCK_REOPEN: AtomicBool = AtomicBool::new(false);

// ─── macOS ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod mac {
    use super::*;

    // NSWindowCollectionBehavior bitmask values:
    //   CanJoinAllSpaces       = 1 << 0  = 1
    //   IgnoresCycle           = 1 << 6  = 64
    //   FullScreenAuxiliary    = 1 << 8  = 256
    //
    // Window level 1000 ≈ NSScreenSaverWindowLevel.
    // Settings window sits one level above (1001) so it remains usable at any opacity.

    fn get_ns_window(window: &Window) -> *mut objc::runtime::Object {
        use objc::{msg_send, runtime::Object, sel, sel_impl};
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let handle = window.window_handle().expect("window handle");
        if let RawWindowHandle::AppKit(h) = handle.as_raw() {
            let ns_view = h.ns_view.as_ptr() as *mut Object;
            unsafe { msg_send![ns_view, window] }
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn setup_overlay_window(window: &Window) {
        use objc::{msg_send, sel, sel_impl};

        let ns_win = get_ns_window(window);
        if ns_win.is_null() { return; }
        unsafe {
            let _: () = msg_send![ns_win, setIgnoresMouseEvents: true];
            // Float above fullscreen apps, join every Space, stay out of Cmd+Tab.
            let behavior: u64 = 1 | 64 | 256;
            let _: () = msg_send![ns_win, setCollectionBehavior: behavior];
            // Just below the macOS screen-saver level.
            let _: () = msg_send![ns_win, setLevel: 1000i64];
        }
    }

    pub fn setup_settings_window(window: &Window) {
        use objc::{msg_send, sel, sel_impl};

        // Window level only — the vibrancy install runs post-wgpu-surface via
        // `install_settings_vibrancy` so we don't re-parent winit's NSView
        // before its CAMetalLayer has been attached (which would crash wgpu
        // with a 0×0 initial drawable and a detached layer hierarchy).
        let ns_win = get_ns_window(window);
        if ns_win.is_null() { return; }
        unsafe { let _: () = msg_send![ns_win, setLevel: 1001i64]; }
    }

    /// Update the VEV's material + appearance when the system theme
    /// changes.  Called from the settings window's render loop when
    /// `window.theme()` reports a different value than before.  The VEV
    /// pointer must be the one installed by `install_settings_vibrancy`.
    ///
    /// Doing the update ourselves (instead of letting AppKit's
    /// `_systemAppearanceDidChange` auto-propagate) avoids the
    /// `NSCursor makeObjectsPerformSelector:` crash we hit when AppKit
    /// walked our re-parented view hierarchy.  Safe because `setAppearance:`
    /// and `setMaterial:` are simple property setters that don't trigger
    /// the recursive `_layoutSubtreeWithOldSize:` propagation.
    pub fn update_settings_vibrancy(vev_ptr: usize, dark_mode: bool) {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};
        if vev_ptr == 0 { return; }
        let vev = vev_ptr as *mut Object;
        unsafe {
            let material: i64 = if dark_mode { 6 } else { 8 };
            let _: () = msg_send![vev, setMaterial: material];

            let appearance_name_c = if dark_mode {
                b"NSAppearanceNameDarkAqua\0".as_ptr() as *const i8
            } else {
                b"NSAppearanceNameAqua\0".as_ptr() as *const i8
            };
            let ns_appearance_name: *mut Object = msg_send![
                class!(NSString), stringWithUTF8String: appearance_name_c
            ];
            let appearance: *mut Object = msg_send![
                class!(NSAppearance), appearanceNamed: ns_appearance_name
            ];
            if !appearance.is_null() {
                let _: () = msg_send![vev, setAppearance: appearance];
            }
        }
    }

    /// Swap the NSWindow's contentView for an NSVisualEffectView with a
    /// theme-appropriate material + `.behindWindow` blending, mirroring
    /// Swift's `AppDelegate.applicationDidFinishLaunching`.  Called AFTER
    /// the wgpu surface has been created so winit's NSView already has a
    /// sized CAMetalLayer — we only re-parent the view in the hierarchy,
    /// we don't touch its layer.
    ///
    /// Material selection:
    ///   Light  → hudWindow (8)  — strong blur, subtle light tint that reads
    ///                             as "translucent over the desktop" when
    ///                             the backdrop is bright.
    ///   Dark   → popover (6)    — designed for translucent popovers.  More
    ///                             transparent than underWindowBackground
    ///                             (which was too dense in Dark mode) and
    ///                             more neutral than hudWindow (which
    ///                             brightened dark backdrops).  Strong
    ///                             native blur + dark-mode tint that blends
    ///                             into dark apps/desktop without reading
    ///                             as lighter than what's behind.
    ///
    /// Constants (raw Cocoa NSInteger / NSUInteger):
    ///   NSVisualEffectBlendingMode.behindWindow = 0
    ///   NSVisualEffectState.active              = 1
    ///   NSAutoresizingMaskOptions width|height  = 2 | 16 = 18
    pub fn install_settings_vibrancy(window: &Window, dark_mode: bool) -> usize {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        // Opt-out escape hatch — setting `EXHALE_DISABLE_VIBRANCY=1` in
        // the environment skips the contentView swap entirely and leaves
        // winit's NSView as the window's direct contentView.  No vibrancy
        // blur, but no firstResponder / inputContext crashes either.
        if std::env::var_os("EXHALE_DISABLE_VIBRANCY").is_some() {
            return 0;
        }

        let ns_win = get_ns_window(window);
        if ns_win.is_null() { return 0; }

        unsafe {
            let handle = match window.window_handle() {
                Ok(h) => h,
                Err(_) => return 0,
            };
            let RawWindowHandle::AppKit(h) = handle.as_raw() else { return 0; };
            let ns_view = h.ns_view.as_ptr() as *mut Object;
            if ns_view.is_null() { return 0; }

            // Non-opaque window + clear backgroundColor so the compositor
            // honours the Metal layer's alpha over the VEV behind it.
            let _: () = msg_send![ns_win, setOpaque: false];
            let clear: *mut Object = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![ns_win, setBackgroundColor: clear];

            // Container-view approach: swap the window's contentView for a
            // plain, layer-backed NSView that holds *both* the NSVisualEffectView
            // AND winit's NSView as **siblings**.  This is the key invariant —
            // adding VEV as a subview of winit's view (which has a CAMetalLayer)
            // puts VEV's layer UNDER the Metal layer's sublayer tree, so the
            // blur ends up drawn ON TOP of egui's pixels.  Making them siblings
            // in a plain container places their layers next to each other,
            // with VEV at z=0 and the Metal layer at z=1 — exactly what Swift's
            // `settingsWindow.contentView = visualEffect; visualEffect.addSubview(hostingView)`
            // achieves, adapted to avoid the SIGSEGV we hit when VEV itself
            // became the immediate parent of the Metal-hosting NSView.
            let bounds: NSRect = msg_send![ns_view, bounds];

            let container: *mut Object = msg_send![class!(NSView), alloc];
            let container: *mut Object = msg_send![container, initWithFrame: bounds];
            if container.is_null() { return 0; }
            let _: () = msg_send![container, setWantsLayer:         true];
            let _: () = msg_send![container, setAutoresizesSubviews: true];
            let _: () = msg_send![container, setAutoresizingMask:   18u64];

            let vev: *mut Object = msg_send![class!(NSVisualEffectView), alloc];
            let vev: *mut Object = msg_send![vev, initWithFrame: bounds];
            if vev.is_null() { return 0; }
            // Per-theme material selection (see doc comment above):
            //   Light: hudWindow (8) — strong blur + subtle tint, visibly
            //                          translucent over bright desktops.
            //   Dark:  popover   (6) — translucent, blurred, neutral tint
            //                          over dark backdrops.
            //
            // `alphaValue` is not set (default is 1.0) — setting it
            // synchronously fires a key-resign notification that re-enters
            // winit's event loop and panics.
            let material: i64 = if dark_mode { 6 } else { 8 };
            let _: () = msg_send![vev, setMaterial:         material];
            let _: () = msg_send![vev, setBlendingMode:     0i64];  // behindWindow
            let _: () = msg_send![vev, setState:            1i64];  // active
            let _: () = msg_send![vev, setAutoresizingMask: 18u64];

            // Pin the NSVisualEffectView's NSAppearance explicitly so the
            // system's appearance-change notification doesn't propagate
            // through this view's _layoutSubtreeWithOldSize: — AppKit's
            // propagation walks tracking-area / cursor-rect arrays and
            // corrupts under our re-parented view hierarchy, raising
            // `NSCursor does not respond to makeObjectsPerformSelector:`
            // and aborting the app when the user toggles Light/Dark.
            // With an explicit appearance set, the VEV declines the
            // propagation and the iteration stops safely.
            let appearance_name_c = if dark_mode {
                b"NSAppearanceNameDarkAqua\0".as_ptr() as *const i8
            } else {
                b"NSAppearanceNameAqua\0".as_ptr() as *const i8
            };
            let ns_appearance_name: *mut Object = msg_send![
                class!(NSString), stringWithUTF8String: appearance_name_c
            ];
            let appearance: *mut Object = msg_send![
                class!(NSAppearance), appearanceNamed: ns_appearance_name
            ];
            if !appearance.is_null() {
                let _: () = msg_send![vev, setAppearance: appearance];
            }

            // Retain winit's NSView so NSWindow.setContentView:container —
            // which releases the outgoing contentView — doesn't drop it to
            // refcount zero before we re-insert it under the container.
            let _: () = msg_send![ns_view,   retain];
            let _: () = msg_send![ns_win,    setContentView:        container];
            let _: () = msg_send![container, addSubview:            vev];
            let _: () = msg_send![container, addSubview:            ns_view];
            let _: () = msg_send![ns_view,   setFrame:              bounds];
            let _: () = msg_send![ns_view,   setAutoresizingMask:   18u64];
            let _: () = msg_send![ns_view,   release];

            // NOTE: we deliberately do NOT call `makeFirstResponder:
            // ns_view` here.  Every variant we tried (synchronous,
            // performSelector:afterDelay:0, dispatch_async) either panics
            // winit from reentrancy or segfaults mid-frame.  Winit picks
            // its view back up as first responder on the next mouse
            // click organically, which is close enough to Swift's
            // behaviour.
            vev as usize
        }
    }

    // Minimal CGRect / NSRect encoding so we can round-trip `bounds` / `frame`
    // through objc messages without pulling in the cocoa or objc2 crates.
    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    struct NSPoint { x: f64, y: f64 }
    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    struct NSSize { width: f64, height: f64 }
    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    struct NSRect { origin: NSPoint, size: NSSize }

    unsafe impl objc::Encode for NSPoint {
        fn encode() -> objc::Encoding { unsafe { objc::Encoding::from_str("{CGPoint=dd}") } }
    }
    unsafe impl objc::Encode for NSSize {
        fn encode() -> objc::Encoding { unsafe { objc::Encoding::from_str("{CGSize=dd}") } }
    }
    unsafe impl objc::Encode for NSRect {
        fn encode() -> objc::Encoding {
            unsafe { objc::Encoding::from_str("{CGRect={CGPoint=dd}{CGSize=dd}}") }
        }
    }

    /// `.alert` + `.sound` authorization request.  Matches Swift AppDelegate
    /// `requestNotificationPermission()`.
    pub fn request_notification_permission() {
        use block::ConcreteBlock;
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};

        unsafe {
            let cls: *const objc::runtime::Class = class!(UNUserNotificationCenter);
            let center: *mut Object = msg_send![cls, currentNotificationCenter];
            if center.is_null() { return; }

            // UNAuthorizationOptionAlert = 4, UNAuthorizationOptionSound = 2
            let options: u64 = 4 | 2;
            let block = ConcreteBlock::new(|_granted: bool, _err: *mut Object| {});
            let block = block.copy();
            let _: () = msg_send![center, requestAuthorizationWithOptions: options
                                           completionHandler: &*block];
        }
    }

    /// Install `applicationShouldHandleReopen:hasVisibleWindows:` on the
    /// existing NSApplication delegate so `DOCK_REOPEN` is set when the user
    /// clicks the Dock icon while the app is already running.
    ///
    /// winit registers its own NSApplicationDelegate; calling
    /// `setDelegate:` here would replace it and trip winit's
    /// `tried to get a delegate that was not the one Winit has registered`
    /// panic.  Instead we attach the method to winit's delegate class via
    /// `class_addMethod`.  winit does not implement this selector, so the
    /// add always succeeds.
    pub fn register_reopen_handler() {
        use objc::{class, msg_send, runtime, sel, sel_impl};
        use std::ffi::CString;
        use std::sync::atomic::Ordering;

        extern "C" fn reopen(
            _this: &runtime::Object,
            _cmd: runtime::Sel,
            _app: *mut runtime::Object,
            _has_visible: runtime::BOOL,
        ) -> runtime::BOOL {
            super::DOCK_REOPEN.store(true, Ordering::Relaxed);
            runtime::NO
        }

        unsafe {
            let app: *mut runtime::Object = msg_send![class!(NSApplication), sharedApplication];
            let delegate: *mut runtime::Object = msg_send![app, delegate];
            if delegate.is_null() { return; }

            let cls = runtime::object_getClass(delegate) as *mut runtime::Class;
            if cls.is_null() { return; }

            // BOOL, id, SEL, id, BOOL — `c` works on every arch because
            // objc dispatch uses the type string for introspection only;
            // actual calls pass through registers with matching 1-byte size.
            let types = CString::new("c@:@c").expect("encoding CString");
            let sel   = sel!(applicationShouldHandleReopen:hasVisibleWindows:);
            let imp: runtime::Imp = std::mem::transmute(reopen as *const ());
            runtime::class_addMethod(cls, sel, imp, types.as_ptr());
        }
    }

    /// Toggle the macOS activation policy.
    ///   DockOnly / Both → regular (Dock icon shown; tray still works because
    ///                     NSStatusItem is independent of activation policy)
    ///   TopBarOnly      → accessory (menu-bar only, no Dock)
    pub fn apply_app_visibility(vis: AppVisibility, _settings: Option<&Window>) {
        use objc::{msg_send, runtime::Object, sel, sel_impl};

        let value: i64 = match vis {
            AppVisibility::TopBarOnly => 1, // accessory
            _                         => 0, // regular
        };
        unsafe {
            let cls = objc::runtime::Class::get("NSApplication").unwrap();
            let app: *mut Object = msg_send![cls, sharedApplication];
            let _: () = msg_send![app, setActivationPolicy: value];
        }
    }
}

#[cfg(target_os = "macos")]
pub use mac::{
    apply_app_visibility, install_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

// ─── Windows ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
            GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_EX_TOPMOST, WS_EX_TRANSPARENT,
        },
    };

    fn hwnd(window: &Window) -> HWND {
        let handle = window.window_handle().expect("window handle");
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
            // Layered + transparent = click-through.  Tool window + NoActivate so the
            // overlay never steals focus or shows on Alt+Tab / in the taskbar.
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
        }
    }

    pub fn setup_settings_window(_window: &Window) {
        // Keep default styles: titled, focusable, shows in taskbar by default.
        // AppVisibility controls taskbar presence via apply_app_visibility.
    }

    /// Windows has no first-class vibrancy primitive; DWM's blur-behind is a
    /// follow-up.  No-op for now.
    pub fn install_settings_vibrancy(_window: &Window, _dark_mode: bool) -> usize { 0 }

    /// No-op on Windows (no VEV to update).
    pub fn update_settings_vibrancy(_vev_ptr: usize, _dark_mode: bool) {}

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
}

#[cfg(target_os = "windows")]
pub use win::{
    apply_app_visibility, install_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

// ─── Linux / BSD (X11) ───────────────────────────────────────────────────────
//
// Click-through uses XFixes to set an empty input-region.  Wayland compositors
// honour the same concept via `wl_surface::set_input_region(empty)`; winit does
// not expose that yet, so Wayland users get a transparent overlay that still
// intercepts clicks — documented limitation.

#[cfg(all(unix, not(target_os = "macos")))]
mod nix {
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
            let c = CString::new(name).unwrap();
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
        x.set_wm_state(b"_NET_WM_STATE_ABOVE",        true);
        x.set_wm_state(b"_NET_WM_STATE_STICKY",       true);
        x.set_wm_state(b"_NET_WM_STATE_SKIP_TASKBAR", true);
        x.set_wm_state(b"_NET_WM_STATE_SKIP_PAGER",   true);
    }

    pub fn setup_settings_window(_window: &Window) {
        // Default X11 settings window is fine; apply_app_visibility handles
        // taskbar presence for the Top-Bar-only mode.
    }

    /// X11 has no portable blur; KWin honours `_KDE_NET_WM_BLUR_BEHIND_REGION`
    /// but it's KWin-specific and GNOME/Mutter ignore it.  Leave as no-op.
    pub fn install_settings_vibrancy(_window: &Window, _dark_mode: bool) -> usize { 0 }

    /// No-op on X11 (no VEV to update).
    pub fn update_settings_vibrancy(_vev_ptr: usize, _dark_mode: bool) {}

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
}

#[cfg(all(unix, not(target_os = "macos")))]
pub use nix::{
    apply_app_visibility, install_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};
