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

    // ─── AppKit constants — named here so the call sites read like the
    //     Apple docs rather than as bare ints.

    /// `NSScreenSaverWindowLevel` — overlay floats just below the
    /// screensaver layer, above fullscreen apps.  Matches Swift's
    /// `NSWindow.Level.screenSaver.rawValue` (`1000`).
    const NS_WINDOW_LEVEL_SCREEN_SAVER:  NSWindowLevel = 1000;
    /// Settings window sits one level above the overlay so it
    /// remains usable at `overlay_opacity = 1.0`
    const NS_WINDOW_LEVEL_SETTINGS:      NSWindowLevel = 1001;

    /// `NSVisualEffectMaterial.popover` (6) — neutral dark blur, used
    /// behind the settings panel in Dark mode
    const VEV_MATERIAL_POPOVER:          i64 = 6;
    /// `NSVisualEffectMaterial.hudWindow` (8) — strong blur with
    /// subtle tint, used in Light mode
    const VEV_MATERIAL_HUD_WINDOW:       i64 = 8;
    /// `NSVisualEffectBlendingMode.behindWindow` (0) — composite the
    /// blur against whatever is behind the VEV's window, not behind
    /// the VEV inside its window
    const VEV_BLENDING_BEHIND_WINDOW:    i64 = 0;
    /// `NSVisualEffectState.active` (1) — always render the full blur
    /// regardless of window-key state.  Required because the backdrop
    /// is `ignoresMouseEvents` + borderless, so it can never become
    /// key (and `followsWindowActiveState` would render an inactive
    /// flat appearance forever)
    const VEV_STATE_ACTIVE:              i64 = 1;
    /// `NSAutoresizingMaskOptions.{width,height}Sizable` = 2 | 16 = 18
    /// — VEV fills its superview as the backdrop resizes
    const VEV_AUTORESIZE_WIDTH_HEIGHT:   u64 = 18;

    /// `NSCompositingOperation.sourceAtop` (5) — used to tint a
    /// rasterised SF Symbol while preserving its alpha channel
    const NS_COMPOSITING_SOURCE_ATOP:    i64 = 5;
    /// `NSImageResizingMode.stretch` (1)
    const NS_IMAGE_RESIZING_STRETCH:     i64 = 1;
    /// `NSBitmapFormat.alphaNonpremultiplied` (2) — matches egui's
    /// `from_rgba_unmultiplied` expectation
    const NS_BITMAP_ALPHA_NONPREMULT:    u64 = 2;
    /// `UNAuthorizationOptions.alert | .sound` (4 | 2 = 6)
    const UN_AUTH_ALERT_AND_SOUND:       u64 = 4 | 2;

    use objc2_app_kit::NSWindowLevel;

    /// Look up the [`NSWindow`] hosting a winit window.  Returns `None`
    /// when the window's raw handle isn't an AppKit one (shouldn't
    /// happen on macOS in practice, but the winit API permits it).
    ///
    /// Uses objc2's typed bindings so the caller gets a
    /// [`Retained<NSWindow>`] that auto-releases on drop and method
    /// calls go through the compile-time-checked AppKit method tables
    fn get_ns_window(window: &Window) -> Option<objc2::rc::Retained<objc2_app_kit::NSWindow>> {
        use objc2::msg_send;
        use objc2_app_kit::{NSView, NSWindow};
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        // `window_handle()` is fallible — a closed/destroyed window
        // during cleanup returns `Err(HandleError::Unavailable)`.
        // Treat that as "no NSWindow available" rather than panicking,
        // so platform helpers called from shutdown paths degrade
        // gracefully.
        let handle = window.window_handle().ok()?;
        let RawWindowHandle::AppKit(h) = handle.as_raw() else { return None; };
        // SAFETY: winit guarantees the NSView pointer is valid for the
        // lifetime of the winit Window we're borrowing from.  `window`
        // on NSView returns the hosting NSWindow (the receiver-typed
        // dispatch via objc2 retains the result for us).
        unsafe {
            let ns_view: *mut NSView = h.ns_view.as_ptr() as *mut NSView;
            let win: Option<objc2::rc::Retained<NSWindow>> = msg_send![ns_view, window];
            win
        }
    }

    pub fn setup_overlay_window(window: &Window) {
        use objc2_app_kit::NSWindowCollectionBehavior;

        let Some(ns_win) = get_ns_window(window) else { return; };
        // Float above fullscreen apps, join every Space, stay out of Cmd+Tab.
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::IgnoresCycle
            | NSWindowCollectionBehavior::FullScreenAuxiliary;
        ns_win.setIgnoresMouseEvents(true);
        ns_win.setCollectionBehavior(behavior);
        ns_win.setLevel(NS_WINDOW_LEVEL_SCREEN_SAVER);
    }

    pub fn setup_settings_window(window: &Window) {
        // Window level only — the vibrancy install runs post-wgpu-surface via
        // `install_settings_vibrancy` so we don't re-parent winit's NSView
        // before its CAMetalLayer has been attached (which would crash wgpu
        // with a 0×0 initial drawable and a detached layer hierarchy).
        let Some(ns_win) = get_ns_window(window) else { return; };
        ns_win.setLevel(NS_WINDOW_LEVEL_SETTINGS);
    }

    /// Update the backdrop NSWindow's NSVisualEffectView material +
    /// appearance when the system theme changes.  Called from the settings
    /// window's render loop when `window.theme()` reports a different value
    /// than before.  `backdrop_ptr` is the NSWindow* returned by
    /// `install_settings_vibrancy`.
    pub fn update_settings_vibrancy(backdrop_ptr: usize, dark_mode: bool) {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_app_kit::{NSAppearance, NSVisualEffectMaterial, NSWindow};
        use objc2_foundation::NSString;
        if backdrop_ptr == 0 { return; }
        // SAFETY: the caller passed us back the same pointer returned by
        // `install_settings_vibrancy`, which we know is a retained
        // NSWindow* (the backdrop child window).  No other code path
        // hands us this `usize`, so as long as the parent settings
        // window hasn't dropped (it owns the child via
        // `addChildWindow:ordered:`), the pointer is live.
        let backdrop = unsafe {
            objc2::rc::Retained::retain(backdrop_ptr as *mut NSWindow)
        };
        let Some(backdrop) = backdrop else { return; };

        let Some(vev) = backdrop.contentView() else { return; };

        let material = if dark_mode {
            NSVisualEffectMaterial::HUDWindow
        } else {
            NSVisualEffectMaterial::HUDWindow
        };
        // The two branches happen to pick the same material today (Swift
        // parity); the conditional is kept so future appearance tweaks
        // can split dark/light without re-introducing the branch.
        let _ = material;

        unsafe {
            // NSVisualEffectView lives in the AppKit binding crate but
            // `contentView` returns the more generic NSView — we need a
            // typed downcast or message send.  The setMaterial: selector
            // exists on NSVisualEffectView only, so dispatch directly
            // through the raw object pointer (typed AppKit method
            // lookups don't include this selector on NSView).
            let vev_obj: *const AnyObject = &*vev as *const _ as *const AnyObject;
            let mat_raw: i64 = if dark_mode { VEV_MATERIAL_POPOVER } else { VEV_MATERIAL_HUD_WINDOW };
            let _: () = msg_send![vev_obj, setMaterial: mat_raw];

            let appearance_name = if dark_mode {
                NSString::from_str("NSAppearanceNameDarkAqua")
            } else {
                NSString::from_str("NSAppearanceNameAqua")
            };
            if let Some(appearance) = NSAppearance::appearanceNamed(&appearance_name) {
                // `setAppearance:` comes from the NSAppearanceCustomization
                // informal protocol; objc2-app-kit doesn't surface it on
                // NSView's generated bindings, so dispatch through the
                // raw object.
                let _: () = msg_send![vev_obj, setAppearance: &*appearance];
            }
        }
    }

    /// Keep the backdrop NSWindow's frame in lockstep with its parent
    /// (the settings window).  macOS auto-tracks position for child
    /// windows via `addChildWindow:ordered:`, but NOT size — we call this
    /// on every `WindowEvent::Resized` to copy the parent's frame over.
    pub fn sync_settings_backdrop_frame(backdrop_ptr: usize) {
        use objc2_app_kit::NSWindow;
        if backdrop_ptr == 0 { return; }
        // SAFETY: see `update_settings_vibrancy` — same provenance
        // invariant.
        let backdrop = unsafe {
            objc2::rc::Retained::retain(backdrop_ptr as *mut NSWindow)
        };
        let Some(backdrop) = backdrop else { return; };

        let Some(parent) = backdrop.parentWindow() else { return; };
        let frame = parent.frame();
        // `display: true` so the VEV renders at the new size on this
        // same frame — otherwise there's a one-frame lag where the blur
        // rect doesn't track the window edge during a drag.
        backdrop.setFrame_display(frame, true);
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
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_app_kit::{
            NSAppearance, NSBackingStoreType, NSColor, NSView, NSVisualEffectView,
            NSWindow, NSWindowCollectionBehavior, NSWindowOrderingMode, NSWindowStyleMask,
        };
        use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSString};
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        if std::env::var_os("EXHALE_DISABLE_BLUR").is_some() {
            return setup_transparent_settings_window(window);
        }

        let Some(ns_win) = get_ns_window(window) else { return 0; };
        // SAFETY: install_settings_vibrancy runs from `SettingsWindow::new`
        // on the winit event-loop thread, which is the macOS main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        unsafe {
            // Settings window itself: transparent + clear background so
            // wgpu's alpha passes through to whatever the compositor
            // chooses to render behind (in our case, the backdrop
            // NSWindow's blurred VEV).
            ns_win.setOpaque(false);
            ns_win.setBackgroundColor(Some(&NSColor::clearColor()));

            // Explicitly mark winit's NSView layer non-opaque.  wgpu-hal
            // calls `render_layer.set_opaque(false)` when the surface
            // alpha_mode is `PostMultiplied`, but that's been observed to
            // sometimes get reset / not take effect — the layer stays
            // opaque and paints a solid rectangle over the backdrop
            // window, hiding the VEV blur completely.  Re-asserting
            // `opaque = NO` here makes the transparency reliable.
            if let Ok(handle) = window.window_handle() {
                if let RawWindowHandle::AppKit(h) = handle.as_raw() {
                    let ns_view: *mut NSView = h.ns_view.as_ptr() as *mut NSView;
                    if !ns_view.is_null() {
                        if let Some(layer) = (&*ns_view).layer() {
                            // CALayer's `setOpaque:` isn't surfaced as a
                            // typed method on QuartzCore's binding either —
                            // fall through to raw dispatch.
                            let layer_obj: *const AnyObject =
                                &*layer as *const _ as *const AnyObject;
                            let _: () = msg_send![layer_obj, setOpaque: false];
                        }
                    }
                }
            }

            // Use the settings window's current screen-space frame so the
            // backdrop starts out exactly overlapping.  AppKit then keeps
            // position locked via addChildWindow; we lock size from Rust.
            let frame: NSRect = ns_win.frame();

            // NSBackingStoreBuffered, NSWindowStyleMaskBorderless.
            let backdrop = {
                let alloc = mtm.alloc::<NSWindow>();
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    alloc,
                    frame,
                    NSWindowStyleMask::empty(), // = NSWindowStyleMaskBorderless
                    NSBackingStoreType::Buffered,
                    false,
                )
            };

            // Behave like a passive backdrop — never steal focus, never
            // eat events, no shadow duplication, follow the parent into
            // every Space.  `releasedWhenClosed = false` so the NSWindow
            // survives hide/show cycles.
            backdrop.setOpaque(false);
            backdrop.setBackgroundColor(Some(&NSColor::clearColor()));
            backdrop.setHasShadow(false);
            backdrop.setIgnoresMouseEvents(true);
            backdrop.setReleasedWhenClosed(false);
            // Match parent's window level (1001 set by setup_settings_window)
            // so addChildWindow ordering isn't fighting a level mismatch.
            backdrop.setLevel(ns_win.level());
            // CanJoinAllSpaces | FullScreenAuxiliary so the backdrop
            // follows the parent across workspaces / fullscreen.
            backdrop.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );

            // NSVisualEffectView filling the backdrop's contentView.
            // Material + blending + state mirror the old in-window install,
            // except now it composites AppKit-natively against the desktop
            // behind the backdrop window.
            let content_bounds = NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size:   frame.size,
            };
            let vev = {
                let alloc = mtm.alloc::<NSVisualEffectView>();
                NSVisualEffectView::initWithFrame(alloc, content_bounds)
            };

            // Per-theme material:
            //   Dark  → popover   (6)  — neutral, translucent blur.
            //   Light → hudWindow (8)  — strong blur + subtle tint.
            // The objc2-app-kit enum names differ slightly from the raw
            // ints; we keep the raw values inline for parity with the
            // previous direct-int dispatch.
            let vev_obj: *const AnyObject = &*vev as *const _ as *const AnyObject;
            let material: i64 = if dark_mode { VEV_MATERIAL_POPOVER } else { VEV_MATERIAL_HUD_WINDOW };
            let _: () = msg_send![vev_obj, setMaterial:        material];
            let _: () = msg_send![vev_obj, setBlendingMode:    VEV_BLENDING_BEHIND_WINDOW];
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
            let _: () = msg_send![vev_obj, setState:           VEV_STATE_ACTIVE];
            let _: () = msg_send![vev_obj, setAutoresizingMask: VEV_AUTORESIZE_WIDTH_HEIGHT];

            // Pin appearance explicitly — same rationale as the old
            // in-window install: blocks AppKit's appearance propagation
            // through tracking-area / cursor-rect walkers that can crash
            // when they hit layer setups they weren't built to walk.
            let appearance_name = if dark_mode {
                NSString::from_str("NSAppearanceNameDarkAqua")
            } else {
                NSString::from_str("NSAppearanceNameAqua")
            };
            if let Some(appearance) = NSAppearance::appearanceNamed(&appearance_name) {
                // `setAppearance:` lives on the NSAppearanceCustomization
                // informal protocol, not on NSView's generated bindings.
                let _: () = msg_send![vev_obj, setAppearance: &*appearance];
            }

            // `setContentView:` typed binding accepts NSView (VEV is a subclass).
            let vev_as_view: &NSView = &vev;
            backdrop.setContentView(Some(vev_as_view));

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
                // `make_rounded_mask_image` still returns an `objc::runtime::Object*`;
                // cast over to objc2's `AnyObject*` for the msg_send.  Removed
                // once that helper is migrated too.
                let mask_obj: *mut AnyObject = mask as *mut AnyObject;
                let _: () = msg_send![vev_obj, setMaskImage: mask_obj];
            }

            // NSWindowOrderingMode::Below = -1 — order the backdrop just
            // under the settings window.  AppKit docs: "When invoked, if
            // the child window isn't visible, this method shows it as
            // part of its ordering logic." — so no separate orderFront
            // call needed, and adding one before `addChildWindow` could
            // briefly put the backdrop IN FRONT of the settings window,
            // which would occlude the egui content until the next
            // ordering pass.
            ns_win.addChildWindow_ordered(&backdrop, NSWindowOrderingMode::Below);

            log::info!(
                "install_settings_vibrancy: backdrop NSWindow installed at {:?} size {:?}",
                (frame.origin.x, frame.origin.y),
                (frame.size.width, frame.size.height),
            );
            super::set_blur_active(true);
            // Transfer the Retained's +1 refcount across the FFI boundary
            // as a raw `usize`.  Paired with `uninstall_settings_vibrancy`
            // which reconstructs via `Retained::from_raw` to drop the
            // refcount.  The raw `usize` ABI keeps the update / sync
            // helpers binary-stable; converting all three to a typed
            // handle is a follow-up
            objc2::rc::Retained::into_raw(backdrop) as usize
        }
    }

    /// Release the backdrop NSWindow installed by
    /// [`install_settings_vibrancy`].  Removes the child window from
    /// its parent (so AppKit stops tracking it) and drops the +1
    /// refcount we held via `into_raw`.  Idempotent: safe to call
    /// with `0` (no-op for the env-disabled blur path).  Called from
    /// `SettingsWindow::Drop` so closing/reopening the settings
    /// window doesn't accumulate orphaned NSWindow instances
    pub fn uninstall_settings_vibrancy(backdrop_ptr: usize) {
        use objc2_app_kit::NSWindow;
        if backdrop_ptr == 0 { return; }

        // SAFETY: caller passes back exactly the pointer
        // `install_settings_vibrancy` returned, which is a +1-retained
        // NSWindow created with `init…`.  Reconstructing the Retained
        // and letting it drop releases that refcount.
        let backdrop: objc2::rc::Retained<NSWindow> = unsafe {
            objc2::rc::Retained::from_raw(backdrop_ptr as *mut NSWindow)
                .expect("uninstall_settings_vibrancy: pointer was non-null")
        };

        // Detach from parent so AppKit doesn't keep a strong reference
        // via the child-windows array.  No-op if already detached.
        if let Some(parent) = backdrop.parentWindow() {
            parent.removeChildWindow(&backdrop);
        }
        backdrop.orderOut(None);
        super::set_blur_active(false);
        // Retained drops here, releasing the original +1 from `into_raw`.
    }

    /// Rasterise an SF Symbol into a pixel buffer egui can upload as a
    /// texture.  Returns `(rgba_bytes, width_px, height_px)` on success
    /// or `None` if the symbol isn't found / rasterisation fails.
    ///
    /// The bytes are interleaved RGBA, **non-premultiplied** alpha, 8
    /// bits per channel — egui's `ColorImage::from_rgba_unmultiplied`
    /// can ingest them directly.  Drawing happens at 2× scale relative
    /// to `point_size` so the texture stays crisp on Retina displays;
    /// at egui-paint time the image is sized back down to its point
    /// dimensions, and the GPU sampler handles the downsample.
    ///
    /// `dark_mode` controls the tint colour: white in dark, black in
    /// light — matching `Color.primary` from SwiftUI's ControlButton.
    /// We tint by drawing the symbol into a graphics context with
    /// default `sourceOver`, then filling the same rect with the tint
    /// colour using `sourceAtop` so only the alpha-non-zero pixels of
    /// the symbol get coloured in.
    pub fn render_sf_symbol(
        name: &str,
        point_size: f64,
        dark_mode: bool,
    ) -> Option<(Vec<u8>, u32, u32)> {
        use objc2::{class, msg_send};
        use objc2::runtime::{AnyClass, AnyObject};

        unsafe {
            // NSString *symbolName = @"<name>"
            let cstr = std::ffi::CString::new(name).ok()?;
            let ns_string_cls = class!(NSString);
            let symbol_name: *mut AnyObject = msg_send![
                ns_string_cls, stringWithUTF8String: cstr.as_ptr()
            ];
            if symbol_name.is_null() { return None; }

            // [NSImage imageWithSystemSymbolName:name accessibilityDescription:nil]
            let nil_obj: *mut AnyObject = std::ptr::null_mut();
            let image: *mut AnyObject = msg_send![
                class!(NSImage),
                imageWithSystemSymbolName: symbol_name,
                accessibilityDescription: nil_obj,
            ];
            if image.is_null() { return None; }

            // [NSImageSymbolConfiguration configurationWithPointSize:weight:scale:]
            // weight = NSFontWeightRegular (0.0) ; scale = NSImageSymbolScaleMedium (2)
            let config_cls: &AnyClass = AnyClass::get(c"NSImageSymbolConfiguration")?;
            let config: *mut AnyObject = msg_send![
                config_cls,
                configurationWithPointSize: point_size,
                weight: 0.0_f64,
                scale: 2_i64,
            ];
            let image: *mut AnyObject = msg_send![image, imageWithSymbolConfiguration: config];
            if image.is_null() { return None; }

            // Render at 2x for Retina; egui downsamples at sample time.
            let size: NSSize = msg_send![image, size];
            const SCALE: f64 = 2.0;
            let pixel_w = ((size.width  * SCALE).ceil() as u32).max(1);
            let pixel_h = ((size.height * SCALE).ceil() as u32).max(1);

            // NSBitmapImageRep — `initWithBitmapDataPlanes:..:bitmapFormat:..`
            // variant lets us request `NSBitmapFormatAlphaNonpremultiplied`
            // (= 2), which is the format egui's `from_rgba_unmultiplied`
            // expects.  The default init (without bitmapFormat) gives
            // premultiplied alpha and would force us to unpremultiply
            // every pixel after the fact.
            let cs_name = b"NSCalibratedRGBColorSpace\0".as_ptr() as *const i8;
            let space: *mut AnyObject = msg_send![
                ns_string_cls, stringWithUTF8String: cs_name
            ];
            let rep: *mut AnyObject = msg_send![class!(NSBitmapImageRep), alloc];
            let rep: *mut AnyObject = msg_send![rep,
                initWithBitmapDataPlanes: std::ptr::null_mut::<*mut u8>(),
                pixelsWide: pixel_w as i64,
                pixelsHigh: pixel_h as i64,
                bitsPerSample: 8_i64,
                samplesPerPixel: 4_i64,
                hasAlpha: true,
                isPlanar: false,
                colorSpaceName: space,
                bitmapFormat: NS_BITMAP_ALPHA_NONPREMULT,
                bytesPerRow: (pixel_w * 4) as i64,
                bitsPerPixel: 32_i64,
            ];
            if rep.is_null() { return None; }

            // Logical point size on the rep — drawing routines render at
            // points and the rep's pixel buffer is 2× that.
            let _: () = msg_send![rep, setSize: size];

            // Bind a graphics context backed by the rep, save state.
            let ctx_cls = class!(NSGraphicsContext);
            let ctx: *mut AnyObject = msg_send![ctx_cls, graphicsContextWithBitmapImageRep: rep];
            if ctx.is_null() { return None; }
            let _: () = msg_send![ctx_cls, saveGraphicsState];
            let _: () = msg_send![ctx_cls, setCurrentContext: ctx];

            // Draw the symbol.
            let dst = NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size,
            };
            let _: () = msg_send![image, drawInRect: dst];

            // Tint via `NSCompositingOperationSourceAtop` (= 5): fills
            // only the pixels the symbol drew into, preserving its alpha
            // channel for anti-aliased edges.
            let tint_color: *mut AnyObject = if dark_mode {
                msg_send![class!(NSColor), whiteColor]
            } else {
                msg_send![class!(NSColor), blackColor]
            };
            let _: () = msg_send![tint_color, set];
            let _: () = msg_send![ctx, setCompositingOperation: NS_COMPOSITING_SOURCE_ATOP];
            let path: *mut AnyObject = msg_send![class!(NSBezierPath), bezierPathWithRect: dst];
            let _: () = msg_send![path, fill];

            // Restore.
            let _: () = msg_send![ctx_cls, restoreGraphicsState];

            // Pull bytes out.
            let bitmap_data: *const u8 = msg_send![rep, bitmapData];
            if bitmap_data.is_null() { return None; }
            let total = (pixel_w * pixel_h * 4) as usize;
            let bytes = std::slice::from_raw_parts(bitmap_data, total).to_vec();

            Some((bytes, pixel_w, pixel_h))
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
    unsafe fn make_rounded_mask_image(radius: f64) -> *mut objc2::runtime::AnyObject {
        use objc2::{class, msg_send};
        use objc2::runtime::AnyObject;

        let dim = radius * 2.0 + 1.0;
        let size = NSSize { width: dim, height: dim };

        // [[NSImage alloc] initWithSize:]
        let image: *mut AnyObject = msg_send![class!(NSImage), alloc];
        let image: *mut AnyObject = msg_send![image, initWithSize: size];
        if image.is_null() { return std::ptr::null_mut(); }

        // [image lockFocus]
        let _: () = msg_send![image, lockFocus];

        // [[NSColor blackColor] setFill]
        let black: *mut AnyObject = msg_send![class!(NSColor), blackColor];
        let _: () = msg_send![black, setFill];

        // [NSBezierPath bezierPathWithRoundedRect:xRadius:yRadius:]
        let rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size,
        };
        let path: *mut AnyObject = msg_send![
            class!(NSBezierPath),
            bezierPathWithRoundedRect: rect,
            xRadius: radius,
            yRadius: radius,
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
        let _: () = msg_send![image, setResizingMode: NS_IMAGE_RESIZING_STRETCH];

        image
    }

    /// Fallback setup used when `EXHALE_DISABLE_BLUR=1`.  Just makes the
    /// settings window transparent so the wgpu tinted clear colour is
    /// visible; no vibrancy, no child window.  Returns 0 so all the
    /// vibrancy update/resize hooks are no-ops.
    fn setup_transparent_settings_window(window: &Window) -> usize {
        use objc2_app_kit::NSColor;
        let Some(ns_win) = get_ns_window(window) else { return 0; };
        ns_win.setOpaque(false);
        ns_win.setBackgroundColor(Some(&NSColor::clearColor()));
        0
    }

    // Use the typed `NSPoint` / `NSRect` from objc2-foundation
    // (these are aliased to `CGPoint` / `CGRect` underneath and already
    // implement `objc2::Encode` for free round-tripping).  `NSSize` is
    // available too but referenced only by callers via the typed-NSWindow
    // methods.
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    // `NSEdgeInsets` isn't in objc2-foundation's surface, so define it
    // locally and hand-roll the `Encode` impl.  Matches AppKit's
    // `NSEdgeInsets` struct exactly (`{NSEdgeInsets=dddd}`)
    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    struct NSEdgeInsets { top: f64, left: f64, bottom: f64, right: f64 }

    unsafe impl objc2::Encode for NSEdgeInsets {
        const ENCODING: objc2::Encoding = objc2::Encoding::Struct(
            "NSEdgeInsets",
            &[
                objc2::Encoding::Double,
                objc2::Encoding::Double,
                objc2::Encoding::Double,
                objc2::Encoding::Double,
            ],
        );
    }

    /// `.alert` + `.sound` authorization request.  Matches Swift AppDelegate
    /// `requestNotificationPermission()`.
    pub fn request_notification_permission() {
        use block2::StackBlock;
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject};

        // SAFETY: the framework class lookup is fallible — guard with
        // `AnyClass::get` (returns `Option`) instead of `class!()` which
        // would panic if `UserNotifications.framework` isn't linked
        // (typical for a non-bundled `cargo test` binary).
        let Some(cls) = AnyClass::get(c"UNUserNotificationCenter") else { return; };

        unsafe {
            let center: *mut AnyObject = msg_send![cls, currentNotificationCenter];
            if center.is_null() { return; }

            let options = UN_AUTH_ALERT_AND_SOUND;
            // Closure signature matches Apple's `void (^)(BOOL, NSError*)`.
            // We don't surface the result anywhere; the user can retry
            // via the system's permission UI if they denied.
            let block = StackBlock::new(
                |_granted: objc2::runtime::Bool, _err: *mut AnyObject| {},
            );
            // Promote to heap-allocated RcBlock so the async callback
            // can fire after this function returns without dangling.
            let block = block.copy();
            let _: () = msg_send![
                center,
                requestAuthorizationWithOptions: options,
                completionHandler: &*block,
            ];
        }
    }

    /// Show a native `NSAlert` "Reset to Defaults?" modal and return
    /// `true` if the user picked Reset, `false` for Cancel.  Blocks
    /// the calling thread until the user dismisses the alert — which
    /// is fine because this is the main thread and there's nothing
    /// else for it to do while the modal is up (winit's event loop is
    /// paused by AppKit during a modal session).
    ///
    /// Matches Swift `AppDelegate.showResetConfirmation()` which uses
    /// the same `NSAlert.runModal()` pattern.  Replaces the in-window
    /// egui confirmation on macOS — gives users the native look,
    /// keyboard shortcuts, and VoiceOver behaviour they expect.
    pub fn show_reset_alert() -> bool {
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject};
        use objc2_foundation::NSString;

        let Some(cls) = AnyClass::get(c"NSAlert") else { return false; };
        // `NSAlertFirstButtonReturn` (1000) = first button (Reset).
        const FIRST_BUTTON: i64 = 1000;

        unsafe {
            let alert: *mut AnyObject = msg_send![cls, alloc];
            let alert: *mut AnyObject = msg_send![alert, init];
            if alert.is_null() { return false; }

            let message = NSString::from_str("Reset to Defaults?");
            let detail  = NSString::from_str(
                "All settings will be restored to their defaults.  This can't be undone.",
            );
            let _: () = msg_send![alert, setMessageText:     &*message];
            let _: () = msg_send![alert, setInformativeText: &*detail];
            // `NSAlertStyleWarning = 0` (default for destructive ops).
            let _: () = msg_send![alert, setAlertStyle: 0_u64];

            // Reset first so Cmd-Return (default key equivalent on
            // the first button) maps to it.
            let reset_label  = NSString::from_str("Reset");
            let cancel_label = NSString::from_str("Cancel");
            let _: *mut AnyObject = msg_send![alert, addButtonWithTitle: &*reset_label];
            let _: *mut AnyObject = msg_send![alert, addButtonWithTitle: &*cancel_label];

            let response: i64 = msg_send![alert, runModal];
            let _: () = msg_send![alert, release];
            response == FIRST_BUTTON
        }
    }

    /// Install a minimal `NSMainMenu` so the menu bar shows the
    /// standard Apple, Edit, Window, and Help menus.  Without this
    /// winit-created apps appear in the menu bar with no menus at
    /// all — `Cmd-Q` / `Cmd-H` / `Cmd-W` / Services / Hide Others
    /// don't work, and the app reads as "broken" to anyone who
    /// expects Mac conventions.  Mirrors what Swift apps get for free
    /// via their `NSApplicationMain`-installed main menu.
    ///
    /// We install:
    ///   • Apple menu (auto-named after the app): About, Services,
    ///     Hide / Hide Others / Show All, Quit
    ///   • Edit menu: Cut / Copy / Paste / Select All with native
    ///     `Cmd-X` / `Cmd-C` / `Cmd-V` / `Cmd-A` shortcuts so text
    ///     fields in the settings window behave natively
    ///   • Window menu: Minimize / Zoom (AppKit auto-populates the
    ///     "Bring All to Front" item)
    ///
    /// `selector` handler attached to the NSApp delegate by
    /// [`install_main_menu`].  Builds a small options dictionary
    /// (`ApplicationName`, `ApplicationVersion`, `Credits`) and calls
    /// `[NSApp orderFrontStandardAboutPanelWithOptions:]` so the
    /// system About sheet shows real version info even when the
    /// process isn't running from a fully-populated `.app` bundle.
    extern "C" fn exhale_about_panel(
        _this:  &objc2::runtime::AnyObject,
        _cmd:   objc2::runtime::Sel,
        _sender: *mut objc2::runtime::AnyObject,
    ) {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_app_kit::NSApplication;
        use objc2_foundation::{MainThreadMarker, NSString};

        // SAFETY: selector dispatch is main-thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let app = NSApplication::sharedApplication(mtm);

        let version = env!("CARGO_PKG_VERSION");
        let credits = format!(
            "Cross-platform Rust port of the macOS Swift original.\n\n\
             • macOS, Windows, Linux from one codebase (wgpu + winit + egui)\n\
             • Typed AppKit FFI via objc2\n\
             • Per-window render thread architecture\n\n\
             Source: github.com/peterklingelhofer/exhale",
        );

        // Build an NSDictionary with the three String keys via
        // `dictionaryWithObjectsAndKeys:` (variadic, NIL-terminated).
        // The keys are documented as `ApplicationName`,
        // `ApplicationVersion`, `Credits` (an NSAttributedString for
        // Credits — plain NSString works too, AppKit converts).
        let name_value    = NSString::from_str("exhale");
        let version_value = NSString::from_str(version);
        let credits_value = NSString::from_str(&credits);
        let key_name      = NSString::from_str("ApplicationName");
        let key_version   = NSString::from_str("ApplicationVersion");
        let key_credits   = NSString::from_str("Credits");

        unsafe {
            // Build NSMutableDictionary via `[[NSMutableDictionary alloc] init]`
            // then populate via `setObject:forKey:`.  This avoids the
            // variadic `dictionaryWithObjectsAndKeys:` selector which
            // objc2's msg_send can't typecheck without explicit nil
            // termination.
            let dict_cls = objc2::class!(NSMutableDictionary);
            let dict: *mut AnyObject = msg_send![dict_cls, alloc];
            let dict: *mut AnyObject = msg_send![dict, init];
            let _: () = msg_send![dict, setObject: &*name_value,    forKey: &*key_name];
            let _: () = msg_send![dict, setObject: &*version_value, forKey: &*key_version];
            let _: () = msg_send![dict, setObject: &*credits_value, forKey: &*key_credits];
            let _: () = msg_send![&*app, orderFrontStandardAboutPanelWithOptions: dict];
            // `dict` was alloc+init so we own a +1 — release.
            let _: () = msg_send![dict, release];
        }
    }

    /// Safe to call multiple times — replaces the existing main menu
    /// each call.  Called once from app startup
    pub fn install_main_menu() {
        use objc2::msg_send;
        use objc2::rc::Retained;
        use objc2::sel;
        use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
        use objc2_foundation::{MainThreadMarker, NSString};

        // SAFETY: called once at app startup from the winit event loop's
        // main-thread `resumed()` invocation.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let app = NSApplication::sharedApplication(mtm);

        // Use the app's process name as the Apple-menu title.  AppKit
        // auto-formats anything attached to the leftmost menu in the
        // main menu as "<App Name>" with bold weight.
        let app_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "exhale".to_string());
        let app_name_ns = NSString::from_str(&app_name);

        // SAFETY: NSMenu / NSMenuItem constructors are documented to be
        // main-thread-only.  We're on main per the MainThreadMarker above.
        unsafe {
            let main_menu = NSMenu::new(mtm);

            // ── Apple menu ─────────────────────────────────────────────
            let apple_item = NSMenuItem::new(mtm);
            let apple_menu = NSMenu::new(mtm);

            // About: route through our custom selector which populates
            // a real version string + "Powered by Rust + objc2" credit.
            // The default `orderFrontStandardAboutPanel:` pulls metadata
            // from `Info.plist` — fine for a properly bundled `.app` but
            // empty when running `cargo run --release` from a raw
            // Mach-O.  Our hook calls `orderFrontStandardAboutPanelWithOptions:`
            // with the version/credits dict so both paths look right.
            let about_title = NSString::from_str(&format!("About {app_name}"));
            let about = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &about_title,
                Some(sel!(exhale_orderFrontAboutPanel:)),
                &NSString::from_str(""),
            );
            apple_menu.addItem(&about);
            apple_menu.addItem(&NSMenuItem::separatorItem(mtm));

            // Services submenu: AppKit wires this up automatically if we
            // hand it an NSMenu via `setServicesMenu:`.
            let services_title = NSString::from_str("Services");
            let services_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &services_title,
                None,
                &NSString::from_str(""),
            );
            let services_menu = NSMenu::new(mtm);
            services_item.setSubmenu(Some(&services_menu));
            apple_menu.addItem(&services_item);
            // `setServicesMenu:` is on NSApplication; objc2-app-kit
            // exposes it as a typed method.
            app.setServicesMenu(Some(&services_menu));
            apple_menu.addItem(&NSMenuItem::separatorItem(mtm));

            let hide_title = NSString::from_str(&format!("Hide {app_name}"));
            let hide = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &hide_title,
                Some(sel!(hide:)),
                &NSString::from_str("h"),
            );
            apple_menu.addItem(&hide);

            let hide_others = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &NSString::from_str("Hide Others"),
                Some(sel!(hideOtherApplications:)),
                &NSString::from_str("h"),
            );
            // Alt+Cmd-H — modifier mask 0x80000 | 0x100000 = NSEvent
            // ModifierFlags.option | .command = 524288 | 1048576 = 1572864
            let _: () = msg_send![&hide_others, setKeyEquivalentModifierMask: 1_572_864_u64];
            apple_menu.addItem(&hide_others);

            let show_all = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &NSString::from_str("Show All"),
                Some(sel!(unhideAllApplications:)),
                &NSString::from_str(""),
            );
            apple_menu.addItem(&show_all);
            apple_menu.addItem(&NSMenuItem::separatorItem(mtm));

            let quit_title = NSString::from_str(&format!("Quit {app_name}"));
            let quit = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &quit_title,
                Some(sel!(terminate:)),
                &NSString::from_str("q"),
            );
            apple_menu.addItem(&quit);

            apple_item.setSubmenu(Some(&apple_menu));
            main_menu.addItem(&apple_item);

            // ── Edit menu ──────────────────────────────────────────────
            let edit_item = NSMenuItem::new(mtm);
            let edit_menu_title = NSString::from_str("Edit");
            let edit_menu = NSMenu::initWithTitle(mtm.alloc::<NSMenu>(), &edit_menu_title);

            let mk = |title: &str, action: &str, key: &str| -> Retained<NSMenuItem> {
                // SAFETY: each selector below is a documented standard
                // first-responder action; the objc runtime routes it
                // through the first responder chain at click time.
                let sel = match action {
                    "undo:"           => sel!(undo:),
                    "redo:"           => sel!(redo:),
                    "cut:"            => sel!(cut:),
                    "copy:"           => sel!(copy:),
                    "paste:"          => sel!(paste:),
                    "selectAll:"      => sel!(selectAll:),
                    _ => unreachable!(),
                };
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    mtm.alloc::<NSMenuItem>(),
                    &NSString::from_str(title),
                    Some(sel),
                    &NSString::from_str(key),
                )
            };

            edit_menu.addItem(&mk("Undo",       "undo:",      "z"));
            // Shift+Cmd-Z for Redo.  Mask: NSEventModifierFlags.shift = 1<<17 = 131072
            let redo = mk("Redo", "redo:", "z");
            let _: () = msg_send![&redo, setKeyEquivalentModifierMask: (131_072_u64 | 1_048_576_u64)];
            edit_menu.addItem(&redo);
            edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
            edit_menu.addItem(&mk("Cut",        "cut:",       "x"));
            edit_menu.addItem(&mk("Copy",       "copy:",      "c"));
            edit_menu.addItem(&mk("Paste",      "paste:",     "v"));
            edit_menu.addItem(&mk("Select All", "selectAll:", "a"));

            edit_item.setSubmenu(Some(&edit_menu));
            edit_item.setTitle(&edit_menu_title);
            main_menu.addItem(&edit_item);

            // ── Window menu ────────────────────────────────────────────
            let window_item = NSMenuItem::new(mtm);
            let window_menu_title = NSString::from_str("Window");
            let window_menu = NSMenu::initWithTitle(mtm.alloc::<NSMenu>(), &window_menu_title);

            let minimize = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &NSString::from_str("Minimize"),
                Some(sel!(performMiniaturize:)),
                &NSString::from_str("m"),
            );
            window_menu.addItem(&minimize);

            let zoom = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &NSString::from_str("Zoom"),
                Some(sel!(performZoom:)),
                &NSString::from_str(""),
            );
            window_menu.addItem(&zoom);
            window_menu.addItem(&NSMenuItem::separatorItem(mtm));

            let bring_all = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &NSString::from_str("Bring All to Front"),
                Some(sel!(arrangeInFront:)),
                &NSString::from_str(""),
            );
            window_menu.addItem(&bring_all);

            window_item.setSubmenu(Some(&window_menu));
            window_item.setTitle(&window_menu_title);
            main_menu.addItem(&window_item);
            // Tell AppKit this is the Window menu so it auto-populates
            // with open-window entries.
            app.setWindowsMenu(Some(&window_menu));

            // ── Help menu ──────────────────────────────────────────────
            // Standard macOS app gets a Help menu in the menu bar.
            // AppKit auto-routes the menu-bar search field through this
            // menu, so users hitting Cmd-? get the system Help search.
            let help_item = NSMenuItem::new(mtm);
            let help_menu_title = NSString::from_str("Help");
            let help_menu = NSMenu::initWithTitle(mtm.alloc::<NSMenu>(), &help_menu_title);

            let help_entry_title = NSString::from_str(&format!("{app_name} Help"));
            let help_entry = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &help_entry_title,
                Some(sel!(showHelp:)),
                &NSString::from_str("?"),
            );
            help_menu.addItem(&help_entry);

            help_item.setSubmenu(Some(&help_menu));
            help_item.setTitle(&help_menu_title);
            main_menu.addItem(&help_item);
            // `setHelpMenu:` tells AppKit which menu to mark as the
            // help menu so Cmd-? routing works (also marks it visually
            // distinct on some macOS releases).
            app.setHelpMenu(Some(&help_menu));

            // ── Install custom About-panel handler on NSApp delegate ────
            //
            // Same pattern as `register_reopen_handler`: dynamically add
            // `exhale_orderFrontAboutPanel:` to whatever class winit's
            // delegate is so the About menu item's target/action
            // resolves through the responder chain.  Idempotent —
            // `class_addMethod` returns NO if the selector already
            // exists, and we don't care about the result.
            if let Some(delegate) = app.delegate() {
                let delegate_obj: *const objc2::runtime::AnyObject =
                    delegate.as_ref() as *const _ as *const objc2::runtime::AnyObject;
                let cls_ptr = (&*delegate_obj).class()
                    as *const _ as *mut objc2::runtime::AnyClass;
                let types = std::ffi::CString::new("v@:@")
                    .expect("about-handler encoding");
                let sel = sel!(exhale_orderFrontAboutPanel:);
                let imp: objc2::runtime::Imp =
                    std::mem::transmute(exhale_about_panel as *const ());
                let _ = objc2::ffi::class_addMethod(cls_ptr, sel, imp, types.as_ptr());
            }

            // ── Install ────────────────────────────────────────────────
            app.setMainMenu(Some(&main_menu));
            // Tell AppKit which menu is the Services menu so the
            // Services submenu auto-populates.  Already set above; this
            // is the standalone hint that pairs with the parent menu's
            // app_name title resolution.
            let _ = app_name_ns;
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
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject, Bool, Sel};
        use std::ffi::CString;
        use std::sync::atomic::Ordering;

        extern "C" fn reopen(
            _this:        &AnyObject,
            _cmd:         Sel,
            _app:         *mut AnyObject,
            _has_visible: Bool,
        ) -> Bool {
            super::DOCK_REOPEN.store(true, Ordering::Relaxed);
            Bool::NO
        }

        // SAFETY: called once at startup from the winit event loop's
        // main-thread `resumed()` invocation.
        unsafe {
            let app_cls = objc2::class!(NSApplication);
            let app: *mut AnyObject = msg_send![app_cls, sharedApplication];
            if app.is_null() { return; }
            let delegate: *mut AnyObject = msg_send![app, delegate];
            if delegate.is_null() { return; }

            // `class` is dispatched on the instance to get its dynamic
            // class — winit's delegate is a custom subclass that doesn't
            // implement `applicationShouldHandleReopen:hasVisibleWindows:`,
            // so the `class_addMethod` call below is guaranteed to
            // succeed (it returns NO if the selector is already defined
            // on the class).
            let cls_ptr: *mut AnyClass = msg_send![delegate, class];
            if cls_ptr.is_null() { return; }

            // BOOL, id, SEL, id, BOOL — `c` works on every arch because
            // objc dispatch uses the type string for introspection only;
            // actual calls pass through registers with matching 1-byte size.
            let types = CString::new("c@:@c").expect("encoding CString");
            let sel = objc2::sel!(applicationShouldHandleReopen:hasVisibleWindows:);
            // `objc2::runtime::Imp` is a non-null function pointer with
            // the matching `(receiver, _cmd, ..args)` shape — transmute
            // is the canonical bridge here, since `reopen` already has
            // the right `extern "C"` calling convention.
            let imp: objc2::runtime::Imp = std::mem::transmute(reopen as *const ());
            let _ = objc2::ffi::class_addMethod(
                cls_ptr,
                sel,
                imp,
                types.as_ptr(),
            );
        }
    }

    /// Toggle the macOS activation policy.
    ///   DockOnly / Both → regular (Dock icon shown; tray still works because
    ///                     NSStatusItem is independent of activation policy)
    ///   TopBarOnly      → accessory (menu-bar only, no Dock)
    pub fn apply_app_visibility(vis: AppVisibility, _settings: Option<&Window>) {
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        use objc2_foundation::MainThreadMarker;

        let policy = match vis {
            AppVisibility::TopBarOnly => NSApplicationActivationPolicy::Accessory,
            _                         => NSApplicationActivationPolicy::Regular,
        };

        // SAFETY: this is called from the winit event loop on the main
        // thread (winit invokes our ApplicationHandler on the platform's
        // UI thread, which is main on macOS).  In test contexts we
        // accept a slightly looser guarantee — AppKit tolerates
        // `setActivationPolicy:` from any thread in practice
        let mtm  = unsafe { MainThreadMarker::new_unchecked() };
        let app  = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(policy);
    }

    // ─── Regression tests for AppKit FFI ──────────────────────────────────
    //
    // These exist primarily as a tripwire for the objc → objc2 migration.
    // They cover the parts of the AppKit surface that can be verified
    // without a winit event loop and a human looking at the screen:
    //   • `render_sf_symbol` — pure data function, easy to check
    //     dimensions / pixel non-zero-ness / dark-vs-light variance.
    //   • `apply_app_visibility` — round-trips through the global
    //     `NSApp.activationPolicy`; the test reads back through an
    //     INDEPENDENT objc path so a regression in either direction
    //     surfaces.
    //   • `register_reopen_handler`, `request_notification_permission`
    //     — smoke tests (just that they don't panic in a non-bundled
    //     `cargo test` process).
    //
    // The window-mutating functions (`setup_overlay_window`,
    // `install_settings_vibrancy`, etc.) need a real winit `Window`
    // which requires an event loop — not feasible inside the rust
    // test harness — so they're intentionally NOT covered here.
    // Post-migration smoke verification of those falls to launching
    // the app and clicking around.
    #[cfg(test)]
    mod tests {
        use super::*;
        use exhale_core::types::AppVisibility;
        use std::sync::Once;

        /// Initialise `NSApplication` once per test process.  Some
        /// AppKit APIs (notably `imageWithSystemSymbolName:` and the
        /// `NSBitmapImageRep` graphics-context pipeline used by
        /// `render_sf_symbol`) silently return nil if AppKit hasn't
        /// been bootstrapped via `[NSApplication sharedApplication]`
        /// in the current process — production code goes through this
        /// implicitly via winit's app delegate, but the rust test
        /// harness doesn't.  Calling once at test start lets the
        /// render tests run in a bare `cargo test` invocation
        fn ensure_nsapp_initialised() {
            static INIT: Once = Once::new();
            INIT.call_once(|| {
                use objc2::{class, msg_send};
                use objc2::runtime::AnyObject;
                unsafe {
                    let _app: *mut AnyObject =
                        msg_send![class!(NSApplication), sharedApplication];
                }
            });
        }

        /// Read `NSApp.activationPolicy` directly via raw objc2 dispatch.
        /// This is the verification-side counterpart to
        /// `apply_app_visibility`'s setter — kept separate (and
        /// deliberately using the lower-level runtime API rather than
        /// the typed `NSApplicationActivationPolicy` enum) so a
        /// regression in either path produces a mismatched int and
        /// the test fails loudly
        fn read_activation_policy() -> i64 {
            use objc2::{class, msg_send};
            use objc2::runtime::AnyObject;
            unsafe {
                let app: *mut AnyObject =
                    msg_send![class!(NSApplication), sharedApplication];
                msg_send![app, activationPolicy]
            }
        }

        // The behavior tests below need
        // `NSGraphicsContext.graphicsContextWithBitmapImageRep:` to
        // succeed — which it does in the production app-bundled
        // process but NOT in a bare `cargo test` binary (AppKit's
        // graphics stack needs an app-bundle / run-loop context that
        // the rust test harness doesn't provide).  They're marked
        // `#[ignore]` so they don't fail CI, but can be run via
        // `cargo test -- --ignored` from a bundled context (or
        // launched-app context) to verify post-migration.  The
        // `render_unknown_symbol_returns_none` test below works in
        // either environment because both return None
        #[test]
        #[ignore = "needs AppKit graphics-context (bundled app) — run manually"]
        fn render_known_symbol_returns_data() {
            ensure_nsapp_initialised();
            let out = render_sf_symbol("play.circle.fill", 13.0, false);
            assert!(out.is_some(), "well-known SF Symbol should rasterise");
            let (bytes, w, h) = out.unwrap();
            assert!(w > 0 && h > 0, "expected non-zero dims, got {w}×{h}");
            assert_eq!(bytes.len(), (w * h * 4) as usize,
                "buffer should be RGBA: w*h*4 = {} but got {}", w * h * 4, bytes.len());
        }

        #[test]
        fn render_unknown_symbol_returns_none() {
            let out = render_sf_symbol("this.symbol.does.not.exist.anywhere", 13.0, false);
            assert!(out.is_none(),
                "nonsense symbol name should fail the imageWithSystemSymbolName lookup");
        }

        #[test]
        #[ignore = "needs AppKit graphics-context (bundled app) — run manually"]
        fn dark_and_light_modes_produce_different_pixels() {
            let dark  = render_sf_symbol("play.circle.fill", 13.0, true)
                .expect("dark render");
            let light = render_sf_symbol("play.circle.fill", 13.0, false)
                .expect("light render");
            assert_eq!(dark.1, light.1, "same symbol+size should produce same width");
            assert_eq!(dark.2, light.2, "same symbol+size should produce same height");
            assert_ne!(dark.0, light.0,
                "dark-mode tint (white) and light-mode tint (black) must produce different pixel data");
        }

        #[test]
        #[ignore = "needs AppKit graphics-context (bundled app) — run manually"]
        fn larger_point_size_produces_larger_buffer() {
            let small = render_sf_symbol("play.circle.fill", 12.0, false)
                .expect("small render");
            let large = render_sf_symbol("play.circle.fill", 24.0, false)
                .expect("large render");
            let small_pixels = (small.1 * small.2) as usize;
            let large_pixels = (large.1 * large.2) as usize;
            assert!(large_pixels > small_pixels,
                "24pt ({}px) should rasterise larger than 12pt ({}px)",
                large_pixels, small_pixels);
        }

        #[test]
        #[ignore = "needs AppKit graphics-context (bundled app) — run manually"]
        fn drawn_pixels_are_non_zero() {
            let (bytes, _, _) = render_sf_symbol("play.circle.fill", 24.0, false)
                .expect("rasterise");
            // At least one pixel must have non-zero alpha — otherwise nothing was drawn
            let any_drawn = bytes.chunks_exact(4).any(|px| px[3] != 0);
            assert!(any_drawn,
                "rasterised buffer is fully transparent — the symbol wasn't actually drawn");
        }

        #[test]
        #[ignore = "needs AppKit graphics-context (bundled app) — run manually"]
        fn point_size_scaling_is_at_least_pixel_dense() {
            // Buffer is rendered at 2× point size (Retina); allow tiny SF
            // Symbol bounding-box padding above the bare point dimension.
            let (_, w, _) = render_sf_symbol("play.circle.fill", 20.0, false)
                .expect("rasterise");
            assert!(w >= 32,
                "20pt symbol should be at least 32 px wide at 2× scale, got {w}");
        }

        /// All three `AppVisibility` transitions, verified through a
        /// separate `NSApp.activationPolicy` read.  Bundled into one
        /// test so cargo's parallel test runner can't race two
        /// `setActivationPolicy:` calls against each other on the
        /// global `NSApp`
        #[test]
        fn apply_app_visibility_roundtrip() {
            ensure_nsapp_initialised();
            // Save the original so we don't leave the test process with
            // a different activation policy than it started with.
            let original = read_activation_policy();

            apply_app_visibility(AppVisibility::TopBarOnly, None);
            assert_eq!(read_activation_policy(), 1,
                "TopBarOnly should map to NSApplicationActivationPolicyAccessory (1)");

            apply_app_visibility(AppVisibility::DockOnly, None);
            assert_eq!(read_activation_policy(), 0,
                "DockOnly should map to NSApplicationActivationPolicyRegular (0)");

            apply_app_visibility(AppVisibility::Both, None);
            assert_eq!(read_activation_policy(), 0,
                "Both should map to NSApplicationActivationPolicyRegular (0)");

            // Restore.
            use objc2::{class, msg_send};
            use objc2::runtime::AnyObject;
            unsafe {
                let app: *mut AnyObject =
                    msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![app, setActivationPolicy: original];
            }
        }

        #[test]
        fn register_reopen_handler_does_not_panic() {
            // Without a delegate (no winit event loop running in the
            // test harness), the function should bail out via its
            // `delegate.is_null()` early return.  Idempotent — calling
            // a second time should also be a no-op because
            // `class_addMethod` is a no-op if the selector already
            // exists on the class
            register_reopen_handler();
            register_reopen_handler();
        }

        #[test]
        #[ignore = "needs UserNotifications.framework (bundled app) — run manually"]
        fn request_notification_permission_does_not_panic() {
            // In a non-bundled test process the system will silently
            // deny / drop the request; we just want to confirm the
            // objc dispatch doesn't crash on the way out
            request_notification_permission();
        }
    }
}

#[cfg(target_os = "macos")]
pub use mac::{
    apply_app_visibility, install_main_menu, install_settings_vibrancy,
    render_sf_symbol, show_reset_alert, sync_settings_backdrop_frame,
    uninstall_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

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

// ─── Windows ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod win {
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
        unsafe {
            // Dark-mode title bar (Win10 1809+).  Pass a Win32 BOOL —
            // `i32` 1 / 0 — by pointer.  Ignored as a "feature not
            // present" error on earlier builds.
            let dark_bool: i32 = if dark_mode { 1 } else { 0 };
            let _ = DwmSetWindowAttribute(
                h,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                &dark_bool as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );
        }
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
}

#[cfg(target_os = "windows")]
pub use win::{
    apply_app_visibility, install_settings_vibrancy, reassert_overlay_topmost,
    sync_settings_backdrop_frame, uninstall_settings_vibrancy,
    update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};

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

    /// No-op on Linux.  We used to install KDE/KWin's
    /// `_KDE_NET_WM_BLUR_BEHIND_REGION` on Plasma sessions, but the
    /// resulting transparent settings window introduced the same
    /// compositing regressions we hit on Windows DWM acrylic (overlay
    /// stacking above the controls, mouse-hover lag), so we keep the
    /// settings window OPAQUE on every Linux DE.  `BLUR_ACTIVE` stays
    /// `false` → the egui clear colour + panel fill render the
    /// themed solid background.  macOS remains the only platform
    /// where the settings window has a translucent / vibrancy backdrop.
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
}

#[cfg(all(unix, not(target_os = "macos")))]
pub use nix::{
    apply_app_visibility, install_settings_vibrancy, sync_settings_backdrop_frame,
    uninstall_settings_vibrancy, update_settings_vibrancy, register_reopen_handler,
    request_notification_permission, setup_overlay_window, setup_settings_window,
};
