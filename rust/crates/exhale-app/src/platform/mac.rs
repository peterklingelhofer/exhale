//! macOS implementation of the platform layer.
//! See the parent `platform` module for the public API surface and
//! cross-platform stubs.

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

    /// Reconstruct a `Retained<NSWindow>` from a backdrop pointer the
    /// caller stashed via [`install_settings_vibrancy`] as a `usize`.
    /// Returns `None` for the zero sentinel (no backdrop was installed
    /// — env-disabled blur path) or if the runtime says the pointer
    /// can't be retained (very rare; would mean the object was
    /// somehow already released).
    ///
    /// SAFETY: relies on the invariant that the only producer of these
    /// `usize` values is `install_settings_vibrancy` (which writes a
    /// +1-retained NSWindow*), and that the parent settings window
    /// hasn't been dropped (it owns the child via
    /// `addChildWindow:ordered:`, so as long as `SettingsWindow` is
    /// alive, the backdrop is too).
    fn backdrop_from_ptr(backdrop_ptr: usize) -> Option<objc2::rc::Retained<objc2_app_kit::NSWindow>> {
        if backdrop_ptr == 0 { return None; }
        unsafe { objc2::rc::Retained::retain(backdrop_ptr as *mut objc2_app_kit::NSWindow) }
    }

    /// Update the backdrop NSWindow's NSVisualEffectView material +
    /// appearance when the system theme changes.  Called from the settings
    /// window's render loop when `window.theme()` reports a different value
    /// than before.  `backdrop_ptr` is the NSWindow* returned by
    /// `install_settings_vibrancy`.
    pub fn update_settings_vibrancy(backdrop_ptr: usize, dark_mode: bool) {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_app_kit::NSAppearance;
        use objc2_foundation::NSString;
        let Some(backdrop) = backdrop_from_ptr(backdrop_ptr) else { return; };
        let Some(vev)      = backdrop.contentView() else { return; };

        // Material choice is identical in both themes today (Swift
        // parity uses `.hudWindow` for both dark and light).  The
        // appearance + theme-aware material lookup happens via the
        // raw `setMaterial:` dispatch below using the int-coded
        // `VEV_MATERIAL_*` constants — split dark/light there if
        // future tweaks need divergent materials.

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
        let Some(backdrop) = backdrop_from_ptr(backdrop_ptr) else { return; };
        let Some(parent)   = backdrop.parentWindow() else { return; };
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
            let vev_obj: *const AnyObject = &*vev as *const _ as *const AnyObject;
            let material: i64 = if dark_mode { VEV_MATERIAL_POPOVER } else { VEV_MATERIAL_HUD_WINDOW };
            let _: () = msg_send![vev_obj, setMaterial:        material];
            let _: () = msg_send![vev_obj, setBlendingMode:    VEV_BLENDING_BEHIND_WINDOW];
            // State must be `active` (1), not `followsWindowActiveState`
            // (0).  The backdrop is an `ignoresMouseEvents` + borderless
            // child window, so it can never become key; under
            // `followsWindowActiveState` the VEV would render its
            // INACTIVE (flat desaturated) appearance permanently,
            // painting a solid grey under the transparent settings
            // window above and looking identical to an opaque window.
            // CPU cost of always-active is bounded because the VEV
            // only covers the settings window's ~360x880 pt area
            let _: () = msg_send![vev_obj, setState:           VEV_STATE_ACTIVE];
            let _: () = msg_send![vev_obj, setAutoresizingMask: VEV_AUTORESIZE_WIDTH_HEIGHT];

            // Pin appearance explicitly: blocks AppKit's appearance
            // propagation through tracking-area / cursor-rect walkers
            // that can crash when they hit layer setups they weren't
            // built to walk.
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
                let _: () = msg_send![vev_obj, setMaskImage: mask];
                // `make_rounded_mask_image` returned a +1-retained
                // NSImage.  `setMaskImage:` is a retaining setter (the
                // VEV now owns its own +1), so the caller's +1 is the
                // leak — release it here.  Each settings-window
                // install leaked one NSImage without this.
                let _: () = msg_send![mask, release];
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

            // Force template rendering so `.circle.fill` and other
            // multi-layer SF Symbols render as a clean monochrome
            // silhouette with their natural transparent cutouts
            // preserved.  Without this, recent macOS versions may use
            // hierarchical rendering by default (primary layer at full
            // opacity + secondary layer at ~45% opacity drawn on top),
            // which composites into a near-solid disk after our
            // `SourceAtop` tint pass and hides the inner glyph cutout.
            // `setTemplate:` is macOS 10.6+ so no runtime check is
            // needed; the resulting NSImage matches what SwiftUI's
            // `Image(systemName:)` with no explicit rendering mode
            // produces for the same symbol
            let _: () = msg_send![image, setTemplate: true];

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
            let cs_name = c"NSCalibratedRGBColorSpace".as_ptr();
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
            if ctx.is_null() {
                // Release the +1 retain from alloc/init before bailing.
                let _: () = msg_send![rep, release];
                return None;
            }
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
            if bitmap_data.is_null() {
                let _: () = msg_send![rep, release];
                return None;
            }
            let total = (pixel_w * pixel_h * 4) as usize;
            let bytes = std::slice::from_raw_parts(bitmap_data, total).to_vec();

            // Release the +1 retain we acquired via `alloc/init…`.
            // `graphicsContextWithBitmapImageRep:` only borrowed `rep`
            // for the duration of `saveGraphicsState`…`restoreGraphicsState`;
            // by the time we copy the bitmap bytes the GC is done with
            // it and the rep can drop.  Without this `release` each
            // call leaked one `NSBitmapImageRep` (~16 KB at 24 pt @ 2×),
            // accumulating as the user opened the settings window.
            let _: () = msg_send![rep, release];

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
    /// the same `NSAlert.runModal()` pattern, giving users the native
    /// look, keyboard shortcuts, and VoiceOver behaviour they expect
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

            // Raise the alert above the settings window.  By default
            // an NSAlert's panel lives at `NSModalPanelWindowLevel`
            // (=8), but `setup_settings_window` pins our settings
            // NSWindow at `NS_WINDOW_LEVEL_SETTINGS` (=1001) so it
            // can sit above the breath overlay (`NS_WINDOW_LEVEL_
            // SCREEN_SAVER` = 1000).  Without explicit re-ordering
            // the alert spawns BEHIND the settings window — the user
            // hears the modal-sheet-blocked beep on every keystroke
            // but can't see what's blocking input.
            //
            // Push the alert's NSPanel one level above the settings
            // window AND force-activate the app so the alert grabs
            // both window-z-order and key-focus
            let app_cls = AnyClass::get(c"NSApplication");
            if let Some(app_cls) = app_cls {
                let shared: *mut AnyObject = msg_send![app_cls, sharedApplication];
                if !shared.is_null() {
                    let _: () = msg_send![shared, activateIgnoringOtherApps: true];
                }
            }
            let alert_window: *mut AnyObject = msg_send![alert, window];
            if !alert_window.is_null() {
                // 1002 = one above NS_WINDOW_LEVEL_SETTINGS so the alert
                // unambiguously wins over the settings window's level
                let _: () = msg_send![alert_window, setLevel: 1002_i64];
                let _: () = msg_send![alert_window, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
            }

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
    ///   - Apple menu (auto-named after the app): About, Services,
    ///     Hide / Hide Others / Show All, Quit
    ///   - Edit menu: Cut / Copy / Paste / Select All with native
    ///     `Cmd-X` / `Cmd-C` / `Cmd-V` / `Cmd-A` shortcuts so text
    ///     fields in the settings window behave natively
    ///   - Window menu: Minimize / Zoom (AppKit auto-populates the
    ///     "Bring All to Front" item)
    ///
    /// About-panel content is sourced from the bundle's `Info.plist`
    /// via the standard `orderFrontStandardAboutPanel:` selector:
    /// `CFBundleShortVersionString` / `CFBundleVersion` /
    /// `NSHumanReadableCopyright` populate the panel automatically.
    /// We deliberately avoid attaching a custom
    /// `exhale_orderFrontAboutPanel:` via `class_addMethod` on the
    /// NSApp delegate because adding methods to system classes can
    /// trip Mac App Store static-analysis flags during review.
    ///
    /// Safe to call multiple times; replaces the existing main menu
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

            // About: route through a custom handler that calls
            // `orderFrontStandardAboutPanelWithOptions:` with an
            // explicit version dictionary.  AppKit's default
            // `orderFrontStandardAboutPanel:` reads from
            // `Info.plist`'s `CFBundleShortVersionString` /
            // `NSHumanReadableCopyright`, but unbundled / dev builds
            // (`cargo run`) have no Info.plist so the panel comes up
            // empty.  Passing the version from
            // `env!("CARGO_PKG_VERSION")` at compile time keeps the
            // panel populated in both bundled and unbundled builds
            ensure_about_handler_registered();
            let about_title = NSString::from_str(&format!("About {app_name}"));
            let about = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc::<NSMenuItem>(),
                &about_title,
                Some(sel!(exhaleShowAbout:)),
                &NSString::from_str(""),
            );
            if let Some(handler) = about_handler_instance() {
                let _: () = msg_send![&*about, setTarget: handler];
            }
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

            // ── Install ────────────────────────────────────────────────
            //
            // No custom delegate-method swizzles here: the About menu
            // item uses AppKit's documented `orderFrontStandardAboutPanel:`
            // selector which routes through the first responder chain
            // and is auto-handled by NSApplication.  No `class_addMethod`
            // calls on a system-class — App Store static analysis is
            // happiest when we don't touch the runtime metaclass for
            // built-in classes.
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
    /// Bring an already-running `exhale` instance to the foreground.
    /// Called from `single_instance_guard` when the file-lock acquire
    /// fails (i.e. another instance owns the lock).
    ///
    /// Uses `NSRunningApplication.runningApplicationsWithBundleIdentifier:`
    /// to locate the existing process, then `activateWithOptions:` to
    /// raise it.  That fires `applicationShouldHandleReopen:` on the
    /// running instance's NSApp delegate — our existing handler sets
    /// `DOCK_REOPEN`, which `App::about_to_wait` drains and dispatches
    /// as `AppEvent::ShowSettings`.
    ///
    /// Sandbox-safe (no entitlements required — NSWorkspace /
    /// NSRunningApplication are public, unentitled APIs).  Idempotent
    /// — calling on a non-existent instance is a no-op
    pub fn activate_running_exhale() {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_foundation::NSString;

        // Match the CFBundleIdentifier used in `scripts/bundle-mas.sh`
        // and the Swift app's Info.plist.  When running as a raw
        // binary (no .app bundle) `NSRunningApplication` won't find a
        // match; that's fine, the function silently no-ops.
        let bundle_id = NSString::from_str("peterklingelhofer.exhale");
        unsafe {
            let cls = objc2::class!(NSRunningApplication);
            // `runningApplicationsWithBundleIdentifier:` returns an
            // `NSArray<NSRunningApplication *>` of matching processes
            // (usually 0 or 1).  Iterate and activate any.
            let apps: *mut AnyObject = msg_send![
                cls,
                runningApplicationsWithBundleIdentifier: &*bundle_id,
            ];
            if apps.is_null() { return; }
            let count: usize = msg_send![apps, count];
            // `NSApplicationActivationOptions.activateAllWindows = 1`
            // — bring every window of the running app forward, not
            // just the most-recent.
            const ACTIVATE_ALL_WINDOWS: u64 = 1;
            for i in 0..count {
                let app: *mut AnyObject = msg_send![apps, objectAtIndex: i];
                if !app.is_null() {
                    let _: bool = msg_send![app, activateWithOptions: ACTIVATE_ALL_WINDOWS];
                }
            }
        }
    }

    // ── About-panel handler ───────────────────────────────────────────────────
    //
    // AppKit's stock `orderFrontStandardAboutPanel:` reads version /
    // copyright keys from the bundle's `Info.plist`.  Unbundled dev
    // builds (`cargo run`) and any bundle whose `Info.plist` is
    // missing `CFBundleShortVersionString` end up with a completely
    // empty about panel.  We work around that by using the variant
    // selector `orderFrontStandardAboutPanelWithOptions:` and
    // passing an explicit options dictionary keyed by the
    // documented `NSAboutPanelOption*` constants (their underlying
    // NSString values are literally the suffix after the prefix —
    // `"ApplicationName"`, `"ApplicationVersion"`, …).  The version
    // string comes from `env!("CARGO_PKG_VERSION")` so it tracks
    // `Cargo.toml` at compile time without any runtime plist lookup
    static ABOUT_HANDLER: std::sync::atomic::AtomicPtr<objc2::runtime::AnyObject> =
        std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

    /// Return the leaked `ExhaleAboutHandler` instance set up by
    /// [`ensure_about_handler_registered`].  Returns `None` only when
    /// the class registration step failed (extremely unlikely; would
    /// imply objc runtime allocation failure)
    pub(super) fn about_handler_instance() -> Option<&'static objc2::runtime::AnyObject> {
        let p = ABOUT_HANDLER.load(std::sync::atomic::Ordering::Acquire);
        if p.is_null() {
            None
        } else {
            // SAFETY: pointer was registered exactly once via
            // `objc_registerClassPair` + `alloc/init`, leaked, and is
            // only ever read here.  AppKit retains menu-item targets
            // weakly but our leak keeps the object alive for the
            // process lifetime
            Some(unsafe { &*p })
        }
    }

    /// Idempotently define the `ExhaleAboutHandler` NSObject
    /// subclass, allocate one instance, and stash it in
    /// `ABOUT_HANDLER`.  Safe to call multiple times — the inner
    /// `Once` guards against double-registration
    pub(super) fn ensure_about_handler_registered() {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_foundation::NSString;
        use std::sync::Once;

        // The actual menu-item action — assembles an options
        // dictionary and forwards to AppKit's standard about panel.
        // Signature follows objc method dispatch: (self, _cmd, sender).
        extern "C" fn show_about(
            _this:   &AnyObject,
            _cmd:    objc2::runtime::Sel,
            _sender: *mut AnyObject,
        ) {
            unsafe {
                let app_cls = match objc2::runtime::AnyClass::get(c"NSApplication") {
                    Some(c) => c,
                    None    => return,
                };
                let app: *mut AnyObject = msg_send![app_cls, sharedApplication];
                if app.is_null() { return; }

                // Build NSMutableDictionary with the version / name /
                // copyright keys.  Each key's literal NSString value
                // is the documented public name of the
                // `NSAboutPanelOption*` constant
                let dict_cls = match objc2::runtime::AnyClass::get(c"NSMutableDictionary") {
                    Some(c) => c,
                    None    => return,
                };
                let dict: *mut AnyObject = msg_send![dict_cls, alloc];
                let dict: *mut AnyObject = msg_send![dict, init];
                if dict.is_null() { return; }

                let put = |key_text: &str, val: &NSString| {
                    let key = NSString::from_str(key_text);
                    let _: () = msg_send![dict, setObject: val, forKey: &*key];
                };
                let app_name_val = NSString::from_str("exhale");
                let app_ver_val  = NSString::from_str(env!("CARGO_PKG_VERSION"));
                let copyright    = NSString::from_str("Copyright \u{00A9} Peter Klingelhofer");
                put("ApplicationName",    &app_name_val);
                put("ApplicationVersion", &app_ver_val);
                put("Copyright",          &copyright);

                // Surface the panel above the settings window.  Same
                // problem as the reset alert: settings window sits at
                // level 1001 and the about panel defaults to a
                // normal modal level, so without explicit re-ordering
                // it spawns behind and the user sees nothing
                let _: () = msg_send![app, activateIgnoringOtherApps: true];
                let _: () = msg_send![app, orderFrontStandardAboutPanelWithOptions: dict];
                let _: () = msg_send![dict, release];
                // After AppKit shows the panel, hoist its window to
                // settings-window-plus-one so it can't get covered.
                // The about panel is owned by AppKit; we look it up
                // via its shared key window because the app just
                // activated and brought it to the foreground
                let win: *mut AnyObject = msg_send![app, keyWindow];
                if !win.is_null() {
                    let _: () = msg_send![win, setLevel: 1002_i64];
                    let _: () = msg_send![win, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
                }
            }
        }

        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            // 1. Allocate a new NSObject subclass.  Naming matches
            //    the existing AppleEvent handler pattern in
            //    `register_reopen_handler` for consistency
            let super_cls = objc2::class!(NSObject);
            let name      = c"ExhaleAboutHandler";
            let new_cls   = objc2::ffi::objc_allocateClassPair(
                (super_cls as *const _) as *const _,
                name.as_ptr(),
                0,
            );
            if new_cls.is_null() {
                log::warn!("about handler: objc_allocateClassPair returned null");
                return;
            }

            // 2. Add `exhaleShowAbout:` — type encoding `v@:@` = void
            //    return, (self, _cmd, NSObject* sender).  AppKit
            //    passes the menu item as `sender` per the standard
            //    target/action protocol
            let sel    = objc2::sel!(exhaleShowAbout:);
            let imp: objc2::runtime::Imp = std::mem::transmute(show_about as *const ());
            let added  = objc2::ffi::class_addMethod(
                new_cls as *mut _, sel, imp, c"v@:@".as_ptr(),
            );
            if !added.as_bool() {
                log::warn!("about handler: class_addMethod returned NO");
            }
            objc2::ffi::objc_registerClassPair(new_cls as *mut _);

            // 3. Allocate one instance; leak it for the app's
            //    lifetime (menu-item target+action stores the target
            //    as a weak reference, so this leak is load-bearing —
            //    a dropped instance would mean a dead "About"
            //    selector and the menu item would silently no-op)
            let instance: *mut AnyObject = msg_send![new_cls, alloc];
            let instance: *mut AnyObject = msg_send![instance, init];
            if instance.is_null() {
                log::warn!("about handler: failed to alloc ExhaleAboutHandler");
                return;
            }
            ABOUT_HANDLER.store(instance, std::sync::atomic::Ordering::Release);
        });
    }

    /// Register a Dock-reopen handler **without swizzling winit's
    /// NSApplicationDelegate**.  Previous rounds attached
    /// `applicationShouldHandleReopen:hasVisibleWindows:` to whatever
    /// class winit's delegate happened to be via `class_addMethod`,
    /// which is technically public Obj-C runtime API but is the kind
    /// of pattern Mac App Store static analysis sometimes flags
    /// during review.
    ///
    /// New design — equivalent at the AppleEvent layer (which is
    /// what AppKit's `applicationShouldHandleReopen:` is itself built
    /// on top of):
    ///   1. Define a brand-new Obj-C class `ExhaleAEHandler` at
    ///      runtime (`objc_allocateClassPair` — adds to our own
    ///      namespace, doesn't touch any system class).
    ///   2. Add an `aevtReopen:withReplyEvent:` method to it.
    ///   3. Allocate one instance; leak it so it lives forever.
    ///   4. Register that instance with `NSAppleEventManager` for
    ///      the `kCoreEventClass / kAEReopenApplication` event.
    ///
    /// When the user clicks the Dock icon for an already-running
    /// instance, the system fires `kAEReopenApplication` (which
    /// AppKit normally translates to the delegate selector before
    /// the delegate sees it).  By hooking at the AppleEvent layer
    /// directly we sidestep winit's delegate entirely
    pub fn register_reopen_handler() {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        
        use std::sync::atomic::Ordering;
        use std::sync::Once;

        extern "C" fn handle_reopen(
            _this:  &AnyObject,
            _cmd:   objc2::runtime::Sel,
            _event: *mut AnyObject,
            _reply: *mut AnyObject,
        ) {
            super::DOCK_REOPEN.store(true, Ordering::Relaxed);
        }

        // SAFETY: this entire `Once`-guarded block uses raw objc2
        // runtime FFI to build, register, and instantiate a brand-new
        // class `ExhaleAEHandler`.  Each unsafe operation is justified
        // inline by the surrounding comment.  Aggregate invariants:
        //   * `INIT: Once` guarantees the block runs exactly once per
        //     process — no double-allocation of the class pair (which
        //     would `objc_registerClassPair` a duplicate name).
        //   * `super_cls` is `objc2::class!(NSObject)`, a statically-
        //     valid Class pointer — `objc_allocateClassPair` accepts
        //     any valid Class as the superclass.
        //   * `handle_reopen` matches the AppKit-documented signature
        //     `void (^)(NSAppleEventDescriptor*, NSAppleEventDescriptor*)`
        //     when prepended with the implicit `(self, _cmd, …)`
        //     receiver/selector args that every Obj-C method takes.
        //     `transmute(fn ptr → Imp)` is layout-compatible because
        //     both are `unsafe extern "C" fn` pointers.
        //   * The `Retained::into_raw`-equivalent leak at the end is
        //     intentional and documented — NSAppleEventManager's
        //     handler table needs the instance to outlive the
        //     registration.
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            // 1. Create our own NSObject subclass.  Adding methods to
            //    a class WE created — not a system class — is the
            //    canonical, App-Store-safe pattern for runtime-defined
            //    objc classes.
            let super_cls = objc2::class!(NSObject);
            let name = c"ExhaleAEHandler";
            let new_cls = objc2::ffi::objc_allocateClassPair(
                (super_cls as *const _) as *const _,
                name.as_ptr(),
                0,
            );
            if new_cls.is_null() {
                // Class with this name already exists — re-entrant
                // call (only happens if `register_reopen_handler` is
                // called twice, which `Once` already guards against,
                // but we're defensive).
                log::warn!("register_reopen_handler: objc_allocateClassPair returned null");
                return;
            }

            // 2. Add the AppleEvent handler method.  Type encoding
            //    `v@:@@` = void return, (self, _cmd, NSAppleEventDescriptor*, NSAppleEventDescriptor*).
            //    `c"…"` is a compile-time-nul-terminated C string so
            //    there's no runtime `CString::new` allocation or
            //    interior-NUL panic risk.
            let sel    = objc2::sel!(aevtReopen:withReplyEvent:);
            let imp: objc2::runtime::Imp = std::mem::transmute(handle_reopen as *const ());
            let added  = objc2::ffi::class_addMethod(
                new_cls as *mut _, sel, imp, c"v@:@@".as_ptr(),
            );
            if !added.as_bool() {
                log::warn!("register_reopen_handler: class_addMethod returned NO");
            }
            objc2::ffi::objc_registerClassPair(new_cls as *mut _);

            // 3. Allocate one instance; leak it for the app's lifetime
            //    (Box::leak would be wrong here — this is an Obj-C
            //    object, not a Rust heap allocation).
            let instance: *mut AnyObject = msg_send![new_cls, alloc];
            let instance: *mut AnyObject = msg_send![instance, init];
            if instance.is_null() {
                log::warn!("register_reopen_handler: failed to alloc ExhaleAEHandler");
                return;
            }

            // 4. Register with NSAppleEventManager.  The four-char-code
            //    constants are `'aevt'` (kCoreEventClass) and `'rapp'`
            //    (kAEReopenApplication) packed big-endian.  Use u32
            //    bit-shift to spell them out without
            //    target-endianness ambiguity.
            const K_CORE_EVENT_CLASS:      u32 =
                  (b'a' as u32) << 24 | (b'e' as u32) << 16
                | (b'v' as u32) <<  8 |  b't' as u32;
            const K_AE_REOPEN_APPLICATION: u32 =
                  (b'r' as u32) << 24 | (b'a' as u32) << 16
                | (b'p' as u32) <<  8 |  b'p' as u32;

            let aem_cls = objc2::class!(NSAppleEventManager);
            let aem: *mut AnyObject = msg_send![aem_cls, sharedAppleEventManager];
            if aem.is_null() {
                log::warn!("register_reopen_handler: sharedAppleEventManager returned nil");
                return;
            }
            let _: () = msg_send![
                aem,
                setEventHandler: instance,
                andSelector:     sel,
                forEventClass:   K_CORE_EVENT_CLASS,
                andEventID:      K_AE_REOPEN_APPLICATION,
            ];

            // `instance` is leaked intentionally — it lives for the
            // app's lifetime and is referenced internally by
            // NSAppleEventManager's handler table.
            let _ = instance;
        });
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
    // Cover the parts of the AppKit surface that can be verified without
    // a winit event loop and a human looking at the screen:
    //   - `render_sf_symbol`: pure data function, easy to check
    //     dimensions / pixel non-zero-ness / dark-vs-light variance.
    //   - `apply_app_visibility`: round-trips through the global
    //     `NSApp.activationPolicy`; the test reads back through an
    //     INDEPENDENT objc path so a regression in either direction
    //     surfaces.
    //   - `register_reopen_handler`, `request_notification_permission`:
    //     smoke tests (just that they don't panic in a non-bundled
    //     `cargo test` process).
    //
    // The window-mutating functions (`setup_overlay_window`,
    // `install_settings_vibrancy`, etc.) need a real winit `Window`
    // which requires an event loop, not feasible inside the rust test
    // harness, so they're intentionally NOT covered here.  Smoke
    // verification of those falls to launching the app and clicking
    // around.
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
        // succeed, which it does in the production app-bundled process
        // but NOT in a bare `cargo test` binary (AppKit's graphics
        // stack needs an app-bundle / run-loop context that the rust
        // test harness doesn't provide).  They're marked `#[ignore]`
        // so they don't fail CI, but can be run via `cargo test --
        // --ignored` from a bundled / launched-app context.  The
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
