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

    /// Update the backdrop NSWindow's NSVisualEffectView material +
    /// appearance when the system theme changes.  Called from the settings
    /// window's render loop when `window.theme()` reports a different value
    /// than before.  `backdrop_ptr` is the NSWindow* returned by
    /// `install_settings_vibrancy`.
    pub fn update_settings_vibrancy(backdrop_ptr: usize, dark_mode: bool) {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};
        if backdrop_ptr == 0 { return; }
        let backdrop = backdrop_ptr as *mut Object;
        unsafe {
            let vev: *mut Object = msg_send![backdrop, contentView];
            if vev.is_null() { return; }

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

    /// Keep the backdrop NSWindow's frame in lockstep with its parent
    /// (the settings window).  macOS auto-tracks position for child
    /// windows via `addChildWindow:ordered:`, but NOT size — we call this
    /// on every `WindowEvent::Resized` to copy the parent's frame over.
    pub fn sync_settings_backdrop_frame(backdrop_ptr: usize) {
        use objc::{msg_send, runtime::Object, sel, sel_impl};
        if backdrop_ptr == 0 { return; }
        let backdrop = backdrop_ptr as *mut Object;
        unsafe {
            let parent: *mut Object = msg_send![backdrop, parentWindow];
            if parent.is_null() { return; }
            let frame: NSRect = msg_send![parent, frame];
            // `display:true` so the VEV renders at the new size on this
            // same frame — otherwise there's a one-frame lag where the
            // blur rect doesn't track the window edge during a drag.
            let _: () = msg_send![backdrop, setFrame: frame display: YES_BOOL];
        }
    }

    /// Install a vibrancy effect behind the settings window by creating a
    /// second borderless NSWindow (the "backdrop"), anchoring it as a
    /// child of the settings window via `addChildWindow:ordered:NSWindowBelow`,
    /// and using an NSVisualEffectView as the backdrop's contentView.
    ///
    /// This gives us the same `.behindWindow` blur the Swift app has, but
    /// the settings NSWindow itself is untouched — winit's NSView stays
    /// exactly where winit put it, so the `objc_loadWeakRetained` /
    /// `cursor_state.borrow_mut` crashes we saw with in-window reparenting
    /// can't trigger.
    ///
    /// Returns the backdrop NSWindow pointer (or 0 on failure) so callers
    /// can:
    ///   • call `update_settings_vibrancy(ptr, dark)` on theme change
    ///   • call `sync_settings_backdrop_frame(ptr)` on resize (position
    ///     auto-tracks via child-window, but size does not).
    ///
    /// Opt-out: set `EXHALE_DISABLE_BLUR=1` to skip the backdrop window.
    /// The window then uses the wgpu tinted-translucent look from
    /// `clear_color_for_theme` alone.
    pub fn install_settings_vibrancy(window: &Window, dark_mode: bool) -> usize {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        if std::env::var_os("EXHALE_DISABLE_BLUR").is_some() {
            return setup_transparent_settings_window(window);
        }

        let ns_win = get_ns_window(window);
        if ns_win.is_null() { return 0; }

        unsafe {
            // Settings window itself: transparent + clear background so
            // wgpu's alpha passes through to whatever the compositor
            // chooses to render behind (in our case, the backdrop
            // NSWindow's blurred VEV).
            let _: () = msg_send![ns_win, setOpaque: false];
            let clear: *mut Object = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![ns_win, setBackgroundColor: clear];

            // Explicitly mark winit's NSView layer non-opaque.  wgpu-hal
            // calls `render_layer.set_opaque(false)` when the surface
            // alpha_mode is `PostMultiplied`, but that's been observed to
            // sometimes get reset / not take effect — the layer stays
            // opaque and paints a solid rectangle over the backdrop
            // window, hiding the VEV blur completely.  Re-asserting
            // `opaque = NO` here makes the transparency reliable.
            if let Ok(handle) = window.window_handle() {
                if let RawWindowHandle::AppKit(h) = handle.as_raw() {
                    let ns_view = h.ns_view.as_ptr() as *mut Object;
                    if !ns_view.is_null() {
                        let layer: *mut Object = msg_send![ns_view, layer];
                        if !layer.is_null() {
                            let _: () = msg_send![layer, setOpaque: false];
                        }
                    }
                }
            }

            // Use the settings window's current screen-space frame so the
            // backdrop starts out exactly overlapping.  AppKit then keeps
            // position locked via addChildWindow; we lock size from Rust.
            let frame: NSRect = msg_send![ns_win, frame];

            // NSBackingStoreBuffered = 2, NSWindowStyleMaskBorderless = 0.
            let backdrop: *mut Object = msg_send![class!(NSWindow), alloc];
            let backdrop: *mut Object = msg_send![backdrop,
                initWithContentRect: frame
                styleMask: 0u64
                backing: 2u64
                defer: NO_BOOL
            ];
            if backdrop.is_null() { return 0; }

            // Behave like a passive backdrop — never steal focus, never
            // eat events, no shadow duplication, follow the parent into
            // every Space.  `releasedWhenClosed = false` so the NSWindow
            // survives hide/show cycles.
            let _: () = msg_send![backdrop, setOpaque:              false];
            let _: () = msg_send![backdrop, setBackgroundColor:     clear];
            let _: () = msg_send![backdrop, setHasShadow:           false];
            let _: () = msg_send![backdrop, setIgnoresMouseEvents:  true];
            let _: () = msg_send![backdrop, setReleasedWhenClosed:  false];
            // Match parent's window level (1001 set by setup_settings_window)
            // so addChildWindow ordering isn't fighting a level mismatch.
            let parent_level: i64 = msg_send![ns_win, level];
            let _: () = msg_send![backdrop, setLevel: parent_level];
            // CanJoinAllSpaces (1) | FullScreenAuxiliary (256) so the
            // backdrop follows the parent across workspaces / fullscreen.
            let _: () = msg_send![backdrop, setCollectionBehavior: 1u64 | 256u64];

            // NSVisualEffectView filling the backdrop's contentView.
            // Material + blending + state mirror the old in-window install,
            // except now it composites AppKit-natively against the desktop
            // behind the backdrop window.
            let content_bounds = NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size:   frame.size,
            };
            let vev: *mut Object = msg_send![class!(NSVisualEffectView), alloc];
            let vev: *mut Object = msg_send![vev, initWithFrame: content_bounds];
            if vev.is_null() { return 0; }

            // Per-theme material:
            //   Dark  → popover   (6)  — neutral, translucent blur.
            //   Light → hudWindow (8)  — strong blur + subtle tint.
            let material: i64 = if dark_mode { 6 } else { 8 };
            let _: () = msg_send![vev, setMaterial:         material];
            let _: () = msg_send![vev, setBlendingMode:     0i64];   // behindWindow
            // State 1 = NSVisualEffectStateActive — always render the
            // full blur regardless of window key state.  We used to use
            // `followsWindowActiveState` (0) but that renders an INACTIVE
            // (flat desaturated) appearance when the VEV's window isn't
            // key — and since the backdrop is an `ignoresMouseEvents` +
            // borderless child window, it can NEVER become key.  That
            // left us with a permanently-inactive VEV painting a solid
            // grey over the transparent settings window on top, which
            // looked identical to an opaque window.  `active` always
            // renders the vibrant blur; CPU cost is bounded because the
            // VEV only covers the settings window's ~360×880 pt area.
            let _: () = msg_send![vev, setState:            1i64];
            let _: () = msg_send![vev, setAutoresizingMask: 18u64];

            // Pin appearance explicitly — same rationale as the old
            // in-window install: blocks AppKit's appearance propagation
            // through tracking-area / cursor-rect walkers that can crash
            // when they hit layer setups they weren't built to walk.
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

            let _: () = msg_send![backdrop, setContentView: vev];

            // Apply a rounded-rect mask to the NSVisualEffectView so the
            // backdrop's blur clips to the same ~10 pt corner radius as
            // the settings NSWindow above it — without this, the
            // backdrop's borderless square corners poke past the
            // settings window's rounded bottom and the user sees a
            // pointy-cornered blur rectangle behind the cards.
            //
            // NSVisualEffectView's documented hook for this is `maskImage`
            // (a 9-part stretchable NSImage whose alpha channel becomes
            // the clip mask).  We use this instead of `layer.cornerRadius`
            // because NSVisualEffectView rebuilds its internal layer
            // hierarchy on resize and clobbers any cornerRadius we set —
            // `maskImage` survives those rebuilds because it's a
            // first-class VEV property the framework owns.
            let mask = make_rounded_mask_image(10.0);
            if !mask.is_null() {
                let _: () = msg_send![vev, setMaskImage: mask];
            }

            // NSWindowBelow = -1 — order the backdrop just under the
            // settings window.  AppKit docs: "When invoked, if the child
            // window isn't visible, this method shows it as part of its
            // ordering logic." — so no separate orderFront call needed,
            // and adding one before `addChildWindow` could briefly put
            // the backdrop IN FRONT of the settings window, which would
            // occlude the egui content until the next ordering pass.
            let _: () = msg_send![ns_win, addChildWindow: backdrop ordered: (-1_i64)];

            log::info!(
                "install_settings_vibrancy: backdrop NSWindow installed at {:?} size {:?}",
                (frame.origin.x, frame.origin.y),
                (frame.size.width, frame.size.height),
            );
            super::set_blur_active(true);
            backdrop as usize
        }
    }

    /// Build a stretchable rounded-rect NSImage suitable for
    /// `NSVisualEffectView.maskImage`.  The image is `2*radius + 1`
    /// square, drawn as a single rounded rect filled black so the alpha
    /// channel encodes the mask shape (NSVisualEffectView ignores the
    /// colour).  `capInsets = radius` on every side + `resizingMode =
    /// stretch` causes AppKit to nine-slice the image when the VEV's
    /// bounds change: the four corners stay rounded at exactly `radius`
    /// pt while the four edges and the center stretch flat.  This is
    /// the same pattern Apple's own apps use for vibrancy with rounded
    /// edges — survives VEV resize because the image is stored on the
    /// VEV (not its layer) and AppKit re-applies the mask on every
    /// re-layout.
    unsafe fn make_rounded_mask_image(radius: f64) -> *mut objc::runtime::Object {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};

        let dim = radius * 2.0 + 1.0;
        let size = NSSize { width: dim, height: dim };

        // [[NSImage alloc] initWithSize:]
        let image: *mut Object = msg_send![class!(NSImage), alloc];
        let image: *mut Object = msg_send![image, initWithSize: size];
        if image.is_null() { return std::ptr::null_mut(); }

        // [image lockFocus]
        let _: () = msg_send![image, lockFocus];

        // [[NSColor blackColor] setFill]
        let black: *mut Object = msg_send![class!(NSColor), blackColor];
        let _: () = msg_send![black, setFill];

        // [NSBezierPath bezierPathWithRoundedRect:xRadius:yRadius:]
        let rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size,
        };
        let path: *mut Object = msg_send![
            class!(NSBezierPath),
            bezierPathWithRoundedRect: rect
            xRadius: radius
            yRadius: radius
        ];
        let _: () = msg_send![path, fill];

        // [image unlockFocus]
        let _: () = msg_send![image, unlockFocus];

        // image.capInsets = (radius, radius, radius, radius)
        // image.resizingMode = NSImageResizingModeStretch (1)
        let insets = NSEdgeInsets {
            top: radius, left: radius, bottom: radius, right: radius,
        };
        let _: () = msg_send![image, setCapInsets: insets];
        let _: () = msg_send![image, setResizingMode: 1i64];

        image
    }

    /// Fallback setup used when `EXHALE_DISABLE_BLUR=1`.  Just makes the
    /// settings window transparent so the wgpu tinted clear colour is
    /// visible; no vibrancy, no child window.  Returns 0 so all the
    /// vibrancy update/resize hooks are no-ops.
    fn setup_transparent_settings_window(window: &Window) -> usize {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};
        let ns_win = get_ns_window(window);
        if ns_win.is_null() { return 0; }
        unsafe {
            let _: () = msg_send![ns_win, setOpaque: false];
            let clear: *mut Object = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![ns_win, setBackgroundColor: clear];
        }
        0
    }

    // BOOL encodings for objc messages — `setFrame:display:` and
    // `initWithContentRect:styleMask:backing:defer:` want a C BOOL (i8),
    // not a Rust `bool`.
    const YES_BOOL: i8 = 1;
    const NO_BOOL:  i8 = 0;

    // Minimal CGRect / NSRect encoding so we can round-trip `frame`
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
    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    struct NSEdgeInsets { top: f64, left: f64, bottom: f64, right: f64 }

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
    unsafe impl objc::Encode for NSEdgeInsets {
        fn encode() -> objc::Encoding {
            unsafe { objc::Encoding::from_str("{NSEdgeInsets=dddd}") }
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
    apply_app_visibility, install_settings_vibrancy, sync_settings_backdrop_frame,
    update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

// ─── Windows ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::{
        Foundation::HWND,
        Graphics::Dwm::{
            DwmSetWindowAttribute,
            DWMSBT_TRANSIENTWINDOW,
            DWMWA_SYSTEMBACKDROP_TYPE,
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

    /// Enable Windows 11's DWM acrylic backdrop on the settings window
    /// and flip the title bar to dark mode if the OS is in dark
    /// appearance.  `DwmSetWindowAttribute` with
    /// `DWMWA_SYSTEMBACKDROP_TYPE = DWMSBT_TRANSIENTWINDOW` paints a
    /// frosted-glass acrylic blur of the desktop behind the entire
    /// window — closest equivalent to macOS's `NSVisualEffectView`
    /// `.popover` / `.hudWindow` materials.
    ///
    /// Both calls return harmless errors on older Windows (the
    /// attributes were added in Win10 1809 / Win11 22000 respectively),
    /// so the function degrades gracefully: if the backdrop call fails,
    /// `BLUR_ACTIVE` stays `false` and the egui render path falls back
    /// to opaque drawing.
    ///
    /// Opt-out: set `EXHALE_DISABLE_BLUR=1` to skip the DWM calls.
    pub fn install_settings_vibrancy(window: &Window, dark_mode: bool) -> usize {
        if std::env::var_os("EXHALE_DISABLE_BLUR").is_some() {
            return 0;
        }
        let h = hwnd(window);
        if h.is_null() { return 0; }

        unsafe {
            // Dark-mode title bar (Win10 1809+).  Pass a Win32 BOOL — `i32`
            // 1 / 0 — by pointer.  Ignored as a "feature not present"
            // error on earlier builds.
            let dark_bool: i32 = if dark_mode { 1 } else { 0 };
            let _ = DwmSetWindowAttribute(
                h,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                &dark_bool as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );

            // Acrylic backdrop (Win11 build 22000+).  HRESULT == 0
            // (S_OK) means the OS accepted the attribute.  Anything
            // else (e.g. DWM_E_ATTRIBUTENOTSUPPORTED on Win10) → no
            // blur, render opaquely.
            let backdrop: i32 = DWMSBT_TRANSIENTWINDOW;
            let hr = DwmSetWindowAttribute(
                h,
                DWMWA_SYSTEMBACKDROP_TYPE as u32,
                &backdrop as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );
            if hr == 0 {
                log::info!("install_settings_vibrancy: DWM acrylic enabled");
                super::set_blur_active(true);
                return h as usize;
            } else {
                log::info!("install_settings_vibrancy: DWM backdrop unsupported (hr=0x{:08x})", hr);
            }
        }
        0
    }

    /// On Windows the only theme-dependent property is the title bar
    /// dark-mode flag — re-apply it when the OS appearance changes.
    /// `handle` is the HWND returned by `install_settings_vibrancy`.
    pub fn update_settings_vibrancy(handle: usize, dark_mode: bool) {
        if handle == 0 { return; }
        unsafe {
            let h = handle as HWND;
            let dark_bool: i32 = if dark_mode { 1 } else { 0 };
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
    apply_app_visibility, install_settings_vibrancy, sync_settings_backdrop_frame,
    update_settings_vibrancy, register_reopen_handler,
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

    /// Enable KDE/KWin's `_KDE_NET_WM_BLUR_BEHIND_REGION` on the
    /// settings window.  Setting the property with empty data tells
    /// KWin to blur the desktop behind every transparent pixel of the
    /// window — closest equivalent we have to a portable Wayland or
    /// X11 blur.  GNOME/Mutter ignore the property, so we gate on
    /// `XDG_CURRENT_DESKTOP` (KDE / Plasma) up front to avoid leaving a
    /// window opaque-but-half-rendered on desktops that won't honour
    /// the hint.
    ///
    /// Opt-out: `EXHALE_DISABLE_BLUR=1` skips the X property set.
    pub fn install_settings_vibrancy(window: &Window, _dark_mode: bool) -> usize {
        if std::env::var_os("EXHALE_DISABLE_BLUR").is_some() {
            return 0;
        }
        let is_kde = std::env::var("XDG_CURRENT_DESKTOP")
            .map(|s| s.split(':').any(|tok| tok.eq_ignore_ascii_case("KDE") || tok.eq_ignore_ascii_case("Plasma")))
            .unwrap_or(false);
        if !is_kde {
            log::info!("install_settings_vibrancy: not KDE, skipping blur-behind");
            return 0;
        }
        let Some(xwin) = x11_window(window) else { return 0; };
        let Ok(xlib) = Xlib::open() else { return 0; };

        unsafe {
            let display = (xlib.XOpenDisplay)(std::ptr::null());
            if display.is_null() { return 0; }

            // _KDE_NET_WM_BLUR_BEHIND_REGION: empty CARDINAL array =
            // "blur the entire window behind every transparent pixel".
            let name = std::ffi::CString::new("_KDE_NET_WM_BLUR_BEHIND_REGION").unwrap();
            let blur_atom = (xlib.XInternAtom)(display, name.as_ptr(), 0);
            if blur_atom == 0 {
                (xlib.XCloseDisplay)(display);
                return 0;
            }
            (xlib.XChangeProperty)(
                display,
                xwin,
                blur_atom,
                x11_dl::xlib::XA_CARDINAL,
                32,
                x11_dl::xlib::PropModeReplace,
                std::ptr::null(),
                0,
            );
            (xlib.XFlush)(display);
            (xlib.XCloseDisplay)(display);

            log::info!("install_settings_vibrancy: KDE blur-behind enabled");
            super::set_blur_active(true);
            xwin as usize
        }
    }

    /// No theme-dependent state to update on X11 — KDE follows the
    /// system theme automatically once the blur property is set.
    pub fn update_settings_vibrancy(_vev_ptr: usize, _dark_mode: bool) {}

    /// No-op on X11 (no backdrop window to keep in sync — the blur
    /// region attaches to the settings window itself).
    pub fn sync_settings_backdrop_frame(_backdrop_ptr: usize) {}

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
    apply_app_visibility, install_settings_vibrancy, sync_settings_backdrop_frame,
    update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};
