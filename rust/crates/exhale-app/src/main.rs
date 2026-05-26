// On Windows, suppress the console window that pops up alongside the
// app when launched from Explorer / Start.  `release` builds get the
// `windows` subsystem (no console), `debug` builds keep the console so
// `cargo run` and `RUST_LOG` output remain visible while developing.
//
// Linux + macOS aren't affected — they don't have the implicit-console
// behavior Windows does for entry-point executables.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod app_icon;
mod bootstrap;
mod placement;
#[cfg(feature = "global-hotkeys")]
mod hotkeys;
mod overlay;
mod platform;
mod settings_window;
mod timers;
mod tray;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use anyhow::Result;
use exhale_core::{
    controller::BreathingController,
    poison::RwLockPoisonExt,
    settings::Settings,
    settings_manager::SettingsManager,
};
use exhale_render::GpuContext;
#[cfg(feature = "global-hotkeys")]
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use log::{error, info};
use overlay::{FrameSender, OverlayHandle};
use settings_window::SettingsWindow;
use timers::Timers;
use tray::TrayMenuIds;
use tray_icon::menu::MenuEvent;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::WindowId,
};
#[cfg(target_os = "windows")]
use winit::window::Window;

// ─── User event ───────────────────────────────────────────────────────────────

#[derive(Debug)]
enum AppEvent {
    ShowSettings,
    StartAnimation,
    StopAnimation,
    ResetDefaults,
    #[cfg(feature = "global-hotkeys")]
    ResetDefaultsWithConfirm,
    /// Re-read `settings.keyboard_shortcuts` and re-register every
    /// global hotkey.  Fired by the settings window after the user
    /// completes a capture (right-click → Change Shortcut…) or
    /// resets a per-action shortcut to its default
    #[cfg(feature = "global-hotkeys")]
    RebindHotkeys,
    /// Open the settings window (creating it if necessary) and put
    /// it into shortcut-capture mode for the given action.  Fired
    /// from the tray menu's "Keyboard Shortcuts ▶" submenu so the
    /// user can rebind any action — including Preferences, which
    /// has no settings-window button to right-click
    BeginCapturingShortcut(exhale_core::settings::ShortcutAction),
    Quit,
}

// ─── App state ────────────────────────────────────────────────────────────────

struct App {
    proxy:            EventLoopProxy<AppEvent>,
    settings:         Arc<RwLock<Settings>>,
    settings_manager: Arc<SettingsManager>,

    // GPU context — shared across all renderers.
    gpu: Option<Arc<GpuContext>>,

    // One overlay per monitor.
    overlays:            HashMap<WindowId, OverlayHandle>,

    // Shared frame-sender list the controller's `request_draw` closure
    // fans out to.  Wrapped in `Arc<RwLock<...>>` (not a static `Vec`
    // capture) so [`Self::rescan_monitors`] can mutate it when a
    // monitor is hot-plugged or unplugged without restarting the
    // controller thread.  `None` until the controller is started in
    // `resumed()`.
    frame_senders:       Option<Arc<RwLock<Vec<FrameSender>>>>,

    // Shared breathing-state slot the controller writes each tick and
    // every overlay's render thread reads.  Stored on the App so the
    // hot-plug rescan path can hand a clone to newly-created overlays
    // without needing to extract it from the controller.
    breathing_state:     Option<Arc<std::sync::Mutex<Option<exhale_core::controller::BreathingState>>>>,

    // Earliest instant we'll next call `available_monitors()` to
    // detect hot-plug events.  We poll on a ~2 s cadence rather than
    // subscribing to platform-specific notifications (NSApplicationDid
    // ChangeScreenParameters / WM_DISPLAYCHANGE / XRandR) because the
    // poll cost is negligible (~microseconds), the worst-case 2 s
    // delay before a new overlay appears is imperceptible to a user
    // who just plugged in a monitor, and a single polling path keeps
    // hot-plug behaviour identical on macOS, Windows, and Linux.
    next_monitor_scan:   Option<Instant>,

    // Snapshot of max(w,h)/min(w,h) for the primary monitor at startup.
    // Shared across all overlay renderers so a circle on any display covers
    // the same fraction it would on the primary — matching Swift's
    // `getMaxCircleScale()` which snapshots `NSScreen.main` once at onAppear
    // and never recomputes.
    primary_max_circle_scale: f32,

    // Settings panel.
    settings_win:        Option<SettingsWindow>,
    settings_win_id:     Option<WindowId>,

    // Deadline at which the settings window wants its next repaint, as
    // returned by egui's `repaint_delay`.  `None` means no scheduled
    // repaint: the window sits idle until an input event or external
    // mutation, instead of spinning the event loop at refresh rate
    next_settings_repaint: Option<Instant>,


    // Throttle for `platform::reassert_overlay_topmost` on Windows —
    // Windows orders topmost windows by activation, so a newly-opened
    // app can land above our overlay until we re-bump it to the front
    // of the topmost band.  We re-assert at most once per second
    // (negligible CPU vs every-frame, still imperceptible latency
    // before the overlay reclaims the top).  None = never re-asserted
    // yet; gets set after the first call.
    #[cfg(target_os = "windows")]
    next_topmost_reassert: Option<Instant>,

    // Breathing controller.
    controller: Option<BreathingController>,

    // System tray.
    _tray:    Option<tray_icon::TrayIcon>,
    tray_ids: Option<TrayMenuIds>,

    // Global hotkeys.
    #[cfg(feature = "global-hotkeys")]
    hotkey_manager: Option<GlobalHotKeyManager>,
    #[cfg(feature = "global-hotkeys")]
    hotkey_ids:     Option<hotkeys::HotkeyIds>,

    // Timers.
    timers: Timers,
}

impl App {
    fn new(proxy: EventLoopProxy<AppEvent>, settings_manager: Arc<SettingsManager>) -> Self {
        let settings = Arc::clone(&settings_manager.settings);
        Self {
            proxy,
            settings,
            settings_manager,
            gpu:              None,
            overlays:         HashMap::new(),
            frame_senders:    None,
            breathing_state:  None,
            next_monitor_scan: None,
            primary_max_circle_scale: 1.0,
            settings_win:     None,
            settings_win_id:  None,
            next_settings_repaint: None,
            #[cfg(target_os = "windows")]
            next_topmost_reassert: None,
            controller:       None,
            _tray:            None,
            tray_ids:         None,
            #[cfg(feature = "global-hotkeys")]
            hotkey_manager:   None,
            #[cfg(feature = "global-hotkeys")]
            hotkey_ids:       None,
            timers:           Timers::new(),
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn toggle_settings(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(sw) = &self.settings_win {
            if sw.window.is_visible().unwrap_or(true) {
                sw.window.set_visible(false);
                return;
            }
            sw.window.set_visible(true);
            sw.window.focus_window();
            return;
        }
        // First open: create the window.
        if let Some(gpu) = &self.gpu {
            let settings_snap = self.settings.read_or_recover().clone();
            // Quit callback: SettingsWindow fires this when the user
            // clicks the Quit button in the Controls row.  Sending
            // through the event-loop proxy routes the click through the
            // same `AppEvent::Quit` path the tray-menu Quit uses, so
            // all teardown (settings flush, controller stop, tray
            // destroy) runs in the canonical order.
            let proxy = self.proxy.clone();
            let on_quit = Box::new({
                let proxy = proxy.clone();
                move || { let _ = proxy.send_event(AppEvent::Quit); }
            });
            // Rebind callback: SettingsWindow fires this after the user
            // captures a new shortcut (or resets one to default) so we
            // can re-register every global hotkey from the updated
            // `settings.keyboard_shortcuts`.  Gated behind the
            // `global-hotkeys` feature so the Mac App Store build (which
            // ships without Carbon hotkey integration) doesn't end up
            // sending an event variant whose dispatcher is also cfg-d out.
            #[cfg(feature = "global-hotkeys")]
            let on_rebind_hotkeys = Box::new(move || {
                let _ = proxy.send_event(AppEvent::RebindHotkeys);
            });
            #[cfg(not(feature = "global-hotkeys"))]
            let on_rebind_hotkeys = Box::new(|| {});
            match SettingsWindow::new(
                event_loop, Arc::clone(gpu), &settings_snap, on_quit, on_rebind_hotkeys,
            ) {
                Ok(sw) => {
                    self.settings_win_id = Some(sw.window.id());
                    self.settings_win    = Some(sw);
                }
                Err(e) => error!("settings window: {e}"),
            }
        }
    }

    fn do_start(&mut self) {
        let mut s = self.settings.write_or_recover();
        s.is_animating = true;
        s.is_paused    = false;
        self.timers.reschedule_auto_stop(&s);
        self.settings_manager.mark_dirty();
        self.update_tray_state(&s);
        drop(s);
        // Reset to inhale phase 0, matching Swift start() which always resets
        // cycleCount=0 and currentPhase=.inhale before restarting the timer.
        if let Some(c) = &self.controller {
            c.restart();
        }
        // Windowed-mode (Wayland fallback): bring the animation
        // window back if Stop had hidden it.  No-op on threaded
        // fullscreen-overlay windows (they were never hidden).
        for h in self.overlays.values() {
            h.set_animation_visible(true);
        }
        self.request_settings_redraw();
    }

    fn do_stop(&mut self) {
        let mut s = self.settings.write_or_recover();
        s.is_animating = false;
        s.is_paused    = false;
        self.timers.reschedule_auto_stop(&s);
        self.settings_manager.mark_dirty();
        self.update_tray_state(&s);
        drop(s);
        // Windowed-mode (Wayland fallback): hide the animation
        // window so Stop actually closes it from the user's
        // perspective.  Threaded fullscreen overlays stay mapped
        // and instead render the "stopped" clear frame below
        for h in self.overlays.values() {
            h.set_animation_visible(false);
        }
        // Force one final render so the shader sees display_mode=STOPPED and
        // clears to transparent — matches Swift `window.backgroundColor = .clear`.
        for h in self.overlays.values() { h.wake_render(); }
        self.request_settings_redraw();
    }

    fn update_tray_state(&self, s: &exhale_core::settings::Settings) {
        if let Some(ids) = &self.tray_ids {
            // Start disabled while animating (matches Swift AppDelegate).
            ids.start_item.set_enabled(!s.is_animating);
            // Stop enabled when animating OR paused (matches Swift AppDelegate).
            ids.stop_item.set_enabled(s.is_animating || s.is_paused);
        }
    }

    /// Matches Swift AppDelegate.applyAppVisibility: DockOnly removes the
    /// status-bar item; TopBarOnly / Both show it.
    fn sync_tray_to_visibility(&mut self, vis: exhale_core::types::AppVisibility) {
        use exhale_core::types::AppVisibility;
        let needs_tray = vis != AppVisibility::DockOnly;
        let has_tray   = self._tray.is_some();
        if needs_tray && !has_tray {
            let shortcuts = self.settings.read_or_recover().keyboard_shortcuts.clone();
            match tray::build_tray(&shortcuts) {
                Ok((t, ids)) => {
                    let s = self.settings.read_or_recover();
                    self._tray    = Some(t);
                    self.tray_ids = Some(ids);
                    self.update_tray_state(&s);
                }
                Err(e) => error!("tray rebuild: {e}"),
            }
        } else if !needs_tray && has_tray {
            self._tray    = None;
            self.tray_ids = None;
        }
    }

    fn do_reset(&mut self) {
        let mut s = self.settings.write_or_recover();
        s.reset_preserving_runtime_state();
        self.timers.reschedule_auto_stop(&s);
        self.timers.reschedule_reminder(&s);
        self.settings_manager.mark_dirty();
        self.update_tray_state(&s);
        drop(s);
        // `reset_preserving_runtime_state` also clears the
        // `keyboard_shortcuts` block back to its per-action default
        // (None for everything except Preferences → ⌃⇧,).  Without
        // re-running the rebind path the global-hotkey manager
        // keeps the OLD bindings active and the tray menu keeps
        // displaying them — both diverge from what the settings
        // panel and `settings.toml` claim are in effect.  Cover the
        // macOS NSAlert reset path (which feeds back into do_reset)
        // and the tray-menu Reset click; the egui in-window
        // confirmation flow already calls `on_rebind_hotkeys` for
        // its own reasons but invoking the same routine twice is a
        // cheap no-op
        #[cfg(feature = "global-hotkeys")]
        self.do_rebind_hotkeys();
        self.request_settings_redraw();
    }

    /// Open the settings window (if hidden) and raise the inline
    /// Reset-confirmation card.  Used by the Reset global hotkey
    /// (default Ctrl+Shift+D) on every OS.
    ///
    /// Originally the macOS path here ran a native
    /// `NSAlert.runModal()` for the system look + Cmd-period / Cmd-
    /// Return shortcuts, but that diverged from the button-click
    /// path the user sees in the settings panel — pressing the Reset
    /// button shows the inline card while pressing the hotkey
    /// showed a system alert, even though both meant the same
    /// thing.  Consolidated to the inline card on every OS so the
    /// confirmation chrome is consistent regardless of how the
    /// reset was initiated.  The card lives directly under the
    /// button row inside the Controls section — Esc cancels, Tab +
    /// Enter / Space on either Cancel or Reset resolves it
    #[cfg(feature = "global-hotkeys")]
    fn do_reset_with_confirm(&mut self, event_loop: &ActiveEventLoop) {
        let visible = self.settings_win
            .as_ref()
            .and_then(|sw| sw.window.is_visible())
            .unwrap_or(false);
        if self.settings_win.is_none() || !visible {
            self.toggle_settings(event_loop);
        }
        if let Some(sw) = &mut self.settings_win {
            sw.request_reset_confirmation();
            sw.window.focus_window();
            sw.request_redraw();
        }
    }

    /// Unregister every currently-bound global hotkey and re-register
    /// from `settings.keyboard_shortcuts`.  Triggered when the
    /// settings window captures a new shortcut for any action.
    /// Errors are logged but never propagated — a failed rebind
    /// leaves the dispatcher with whatever ids did register, which
    /// is preferable to a panic on a user action
    #[cfg(feature = "global-hotkeys")]
    fn do_rebind_hotkeys(&mut self) {
        let Some(manager) = &self.hotkey_manager else {
            log::warn!("rebind requested but hotkey manager not initialised");
            return;
        };
        if let Some(old) = self.hotkey_ids.take() {
            hotkeys::unregister_all(manager, &old);
        }
        let shortcuts = self.settings.read_or_recover().keyboard_shortcuts.clone();
        match hotkeys::register_hotkeys(manager, &shortcuts) {
            Ok(ids) => {
                log::info!("hotkeys rebound from updated settings");
                self.hotkey_ids = Some(ids);
            }
            Err(e) => log::error!("hotkey rebind: {e}"),
        }
        // Sync the tray menu's "Keyboard Shortcuts ▶" submenu and
        // top-level item labels in place so the user immediately
        // sees the new binding without having to close and reopen
        // the tray menu.  Avoids a full tray rebuild — `set_text`
        // on each MenuItem is cheap and doesn't flash the icon
        if let Some(tray_ids) = &self.tray_ids {
            tray_ids.refresh_labels(&shortcuts);
        }
    }

    /// Open the settings window (creating it if necessary) and put
    /// it into shortcut-capture mode for `action`.  Used by the
    /// tray menu's "Keyboard Shortcuts ▶" submenu so the user can
    /// rebind any action without first navigating to its button —
    /// critical for Preferences, which has no button to right-click
    fn do_begin_capturing_shortcut(
        &mut self,
        event_loop: &ActiveEventLoop,
        action:     exhale_core::settings::ShortcutAction,
    ) {
        // Ensure the settings window is open and frontmost so the
        // capture overlay it's about to draw is visible.
        let visible = self.settings_win
            .as_ref()
            .and_then(|sw| sw.window.is_visible())
            .unwrap_or(false);
        if self.settings_win.is_none() || !visible {
            self.toggle_settings(event_loop);
        }
        if let Some(sw) = &mut self.settings_win {
            sw.begin_capturing(action);
            sw.window.focus_window();
            sw.request_redraw();
        }
    }

    /// Re-assert topmost ordering at most once per second on Windows.
    ///
    /// Windows orders topmost windows by activation, so a newly-opened
    /// app can land above our overlay until we re-bump it to the front
    /// of the topmost band.  After bumping the overlay forward, we
    /// also bump the settings window (when visible) so the settings
    /// panel ends up ABOVE the overlay; otherwise `overlay_opacity =
    /// 1.0` would lock the user out by covering the controls.  No-op
    /// on non-Windows.  Driven from `about_to_wait` because overlay
    /// rendering lives on dedicated threads.
    ///
    /// Short-circuits via `platform::is_topmost_top()` when the
    /// "expected-top" window of our pair (settings if visible, else
    /// any overlay) is already at the top of z-order: skipping the
    /// SetWindowPos round-trip avoids the per-second `WM_WINDOWPOSCHANGED`
    /// that DWM otherwise composites as a brief frame-edge flicker
    #[cfg(target_os = "windows")]
    fn maybe_reassert_topmost(&mut self) {
        let now = Instant::now();
        let due = self.next_topmost_reassert.is_none_or(|t| now >= t);
        if !due { return; }
        self.next_topmost_reassert = Some(now + Duration::from_secs(1));

        // Determine which of our windows should be at the very top.
        // If settings is visible, settings should be above the overlay
        // (so the user can interact with controls); otherwise the
        // overlay holds the top.  If nothing foreign sits above that
        // window, the entire reassert is a no-op and we can skip the
        // SetWindowPos calls that would otherwise flicker the frame.
        let expected_top: Option<&Window> = self.settings_win.as_ref()
            .filter(|sw| sw.window.is_visible().unwrap_or(false))
            .map(|sw| sw.window.as_ref())
            .or_else(|| self.overlays.values().next().map(|h| h.window.as_ref()));
        if let Some(top) = expected_top {
            if platform::is_topmost_top(top) {
                return;
            }
        }

        for handle in self.overlays.values() {
            platform::reassert_overlay_topmost(&handle.window);
        }
        if let Some(sw) = &self.settings_win {
            if sw.window.is_visible().unwrap_or(false) {
                platform::reassert_overlay_topmost(&sw.window);
            }
        }
    }

    /// Request a settings-window redraw if the window exists and is visible.
    /// Used after external state mutations (hotkeys, tray) so the panel
    /// reflects the new `is_animating` / `is_paused` state without the old
    /// per-tick redraw loop.
    fn request_settings_redraw(&mut self) {
        if let Some(sw) = &self.settings_win {
            if sw.window.is_visible().unwrap_or(false) {
                sw.request_redraw();
                // The next RedrawRequested will overwrite this with egui's
                // fresh repaint_delay, but zero the deadline so about_to_wait
                // doesn't also fire a redundant redraw in the meantime.
                self.next_settings_repaint = None;
            }
        }
    }

    /// Diff the current connected-monitor list against the overlays we
    /// already have and reconcile.  Creates an overlay for every newly
    /// connected monitor and drops the overlay for any monitor that's
    /// no longer present.  Updates the controller's shared
    /// `frame_senders` so newly-created overlays start receiving Frame
    /// signals on the very next controller tick.
    ///
    /// Cheap to call repeatedly: `available_monitors()` is an OS query
    /// that resolves in microseconds, and when the monitor set is
    /// unchanged this function does a single HashSet diff and exits
    /// without touching any locks
    fn rescan_monitors(&mut self, event_loop: &ActiveEventLoop) {
        use std::collections::HashSet;
        let Some(gpu) = self.gpu.as_ref().map(Arc::clone) else { return; };
        let Some(senders) = self.frame_senders.as_ref().map(Arc::clone) else { return; };
        let Some(state) = self.breathing_state.as_ref().map(Arc::clone) else { return; };

        let current: Vec<_> = event_loop.available_monitors().collect();
        let current_keys: HashSet<_> = current.iter().map(overlay::monitor_key).collect();
        let existing_keys: HashSet<_> =
            self.overlays.values().filter_map(|o| o.monitor_key()).collect();

        // ── Drop overlays whose monitor is gone ──────────────────────
        let to_remove: Vec<WindowId> = self.overlays.iter()
            .filter_map(|(wid, h)| {
                let key = h.monitor_key()?;
                if current_keys.contains(&key) { None } else { Some(*wid) }
            })
            .collect();
        for wid in &to_remove {
            if let Some(h) = self.overlays.remove(wid) {
                info!("monitor disconnected; dropping overlay {:?}", h.window.id());
                // h drops here; Drop joins the render thread
            }
        }

        // ── Create overlays for newly connected monitors ─────────────
        let mut added_any = false;
        for m in current.into_iter() {
            let key = overlay::monitor_key(&m);
            if existing_keys.contains(&key) { continue; }
            match OverlayHandle::create_one(
                event_loop, Arc::clone(&gpu), Some(m),
                Arc::clone(&self.settings), Arc::clone(&state),
                self.primary_max_circle_scale,
            ) {
                Ok(h) => {
                    info!("monitor connected; created overlay {:?}", h.window.id());
                    added_any = true;
                    self.overlays.insert(h.window.id(), h);
                }
                Err(e) => log::error!("rescan_monitors: create overlay failed: {e}"),
            }
        }

        // ── Rebuild the controller's send list when anything changed ─
        if !to_remove.is_empty() || added_any {
            let mut w = senders.write_or_recover();
            *w = self.overlays.values().map(|h| h.frame_sender()).collect();
        }
    }
}

// ─── ApplicationHandler ───────────────────────────────────────────────────────

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() { return; } // already initialised

        // ── Bootstrap GPU ─────────────────────────────────────────────────────
        // Create a throw-away surface on an invisible window to pick an adapter,
        // then build the real overlay surfaces below.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends:             wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags:                wgpu::InstanceFlags::default(),
            gles_minor_version:   wgpu::Gles3MinorVersion::Automatic,
        });

        // Bootstrap: use a temporary window to negotiate device selection.
        let bootstrap_attrs = winit::window::Window::default_attributes()
            .with_visible(false)
            .with_inner_size(winit::dpi::PhysicalSize::new(1u32, 1u32));
        let bootstrap_win = match event_loop.create_window(bootstrap_attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => { error!("bootstrap window: {e}"); event_loop.exit(); return; }
        };
        let bootstrap_surface = match instance.create_surface(Arc::clone(&bootstrap_win)) {
            Ok(s) => s,
            Err(e) => { error!("bootstrap surface: {e}"); event_loop.exit(); return; }
        };
        let gpu = match GpuContext::new_for_surface(instance, &bootstrap_surface) {
            Ok(g) => g,
            Err(e) => { error!("GPU init: {e}"); event_loop.exit(); return; }
        };
        drop(bootstrap_surface);
        drop(bootstrap_win);

        self.gpu = Some(Arc::clone(&gpu));

        // ── Primary-monitor max-circle-scale snapshot ─────────────────────────
        // Swift: getMaxCircleScale() = max(w,h) / min(w,h) of NSScreen.main,
        // taken once at ContentView onAppear. Mirror that: snapshot the primary
        // monitor's aspect ratio at startup and broadcast to every overlay so a
        // 16:10 laptop display and a 16:9 external use the same scale constant.
        let primary = event_loop
            .primary_monitor()
            .or_else(|| event_loop.available_monitors().next());
        self.primary_max_circle_scale = primary
            .map(|m| {
                let s = m.size();
                let w = s.width.max(1) as f32;
                let h = s.height.max(1) as f32;
                w.max(h) / w.min(h)
            })
            .unwrap_or(1.0);

        // ── Create overlay windows (one per monitor) ──────────────────────────
        //
        // Each overlay handle spawns a dedicated render thread that owns
        // its renderer.  The controller writes its state snapshot to a
        // shared Arc and signals the render threads via channels — that
        // bypass entirely circumvents the Windows main-thread message
        // queue, where WM_MOUSEMOVE storms over the settings window
        // would otherwise starve WM_PAINT for the overlay and cause
        // animation stutter.
        let breathing_state =
            Arc::new(std::sync::Mutex::new(None::<exhale_core::controller::BreathingState>));
        self.breathing_state = Some(Arc::clone(&breathing_state));
        let handles = OverlayHandle::create_all(
            event_loop,
            Arc::clone(&gpu),
            Arc::clone(&self.settings),
            Arc::clone(&breathing_state),
            self.primary_max_circle_scale,
        );
        if handles.is_empty() {
            error!("no overlay windows created");
            event_loop.exit();
            return;
        }
        let initial_senders: Vec<FrameSender> =
            handles.iter().map(|h| h.frame_sender()).collect();
        // Honour the persisted `is_animating` flag on startup:
        // windowed-mode overlays (Wayland fallback) should start
        // hidden if the user quit the app while stopped, so they
        // come back to a clean "Stop"-state instead of a window
        // showing the stopped clear frame.  No-op for threaded
        // fullscreen overlays.
        let initial_animating = self.settings.read_or_recover().is_animating;
        for h in &handles {
            h.set_animation_visible(initial_animating);
        }
        for h in handles {
            self.overlays.insert(h.window.id(), h);
        }
        info!("{} overlay window(s) created", self.overlays.len());

        // ── Start breathing controller ────────────────────────────────────────
        //
        // `request_draw` fans out a Frame message to every overlay's
        // render thread, bypassing the main event loop.  No WM_PAINT
        // dance and no risk of WM_MOUSEMOVE starvation.
        //
        // The sender list is shared via `Arc<RwLock<...>>` rather than
        // captured by-value so [`Self::rescan_monitors`] can extend it
        // when a new monitor is hot-plugged without restarting the
        // controller thread.
        let frame_senders = Arc::new(RwLock::new(initial_senders));
        let senders_for_cb = Arc::clone(&frame_senders);
        self.controller = Some(BreathingController::start(
            Arc::clone(&self.settings),
            breathing_state,
            Arc::new(move || {
                for tx in senders_for_cb.read_or_recover().iter() {
                    tx.send_frame();
                }
            }),
        ));
        self.frame_senders = Some(frame_senders);
        // Schedule the first hot-plug scan ~2 s out so startup isn't
        // doing redundant work; subsequent scans are paced inside
        // `about_to_wait`.
        self.next_monitor_scan = Some(Instant::now() + Duration::from_secs(2));

        // ── System tray ───────────────────────────────────────────────────────
        {
            let vis = self.settings.read_or_recover().app_visibility;
            self.sync_tray_to_visibility(vis);
        }

        // ── Global hotkeys ────────────────────────────────────────────────────
        // Gated behind the `global-hotkeys` feature so the Mac App Store
        // build (`--no-default-features`) ships without the Carbon-based
        // hotkey registration as a hedge against App Review flagging it.
        #[cfg(feature = "global-hotkeys")]
        match GlobalHotKeyManager::new() {
            Ok(mgr) => {
                let shortcuts = self.settings.read_or_recover().keyboard_shortcuts.clone();
                match hotkeys::register_hotkeys(&mgr, &shortcuts) {
                    Ok(ids) => { self.hotkey_ids = Some(ids); }
                    Err(e)  => error!("hotkey registration: {e}"),
                }
                self.hotkey_manager = Some(mgr);
            }
            Err(e) => error!("hotkey manager: {e}"),
        }

        // ── Timer init ────────────────────────────────────────────────────────
        {
            let s = self.settings.read_or_recover();
            self.timers.reschedule_auto_stop(&s);
            self.timers.reschedule_reminder(&s);
        }

        // ── Standard macOS menu bar (Apple/Edit/Window/Help) ──────────────────
        // Without this winit's NSApplication has no `mainMenu` and the
        // menu bar shows only the app name with no menus — Quit/Hide/
        // Cmd-X/Cmd-C/etc all stop working.  Closes a Swift-parity gap
        // that's invisible until you try to copy text from a settings
        // field.  No-op on Windows / Linux.
        platform::install_main_menu();

        // ── Dock-icon reopen handler (macOS-only internally; no-op elsewhere) ─
        platform::register_reopen_handler();

        // ── Notification permission (macOS-only internally; no-op elsewhere) ──
        if self.settings.read_or_recover().reminder_interval_minutes > 0.0 {
            platform::request_notification_permission();
        }

        // Show the settings window on first launch (creates it as a side effect).
        self.toggle_settings(event_loop);

        // ── Activation policy / taskbar presence ──────────────────────────────
        // Cross-platform: macOS toggles NSApp activation policy; Windows toggles
        // the settings window's taskbar entry; Linux toggles SKIP_TASKBAR/PAGER.
        // Applied AFTER the settings window exists so Windows/Linux can see it.
        {
            let vis = self.settings.read_or_recover().app_visibility;
            let settings_win = self.settings_win.as_ref().map(|sw| sw.window.as_ref());
            platform::apply_app_visibility(vis, settings_win);
        }

        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::ShowSettings   => self.toggle_settings(event_loop),
            AppEvent::StartAnimation => self.do_start(),
            AppEvent::StopAnimation  => self.do_stop(),
            AppEvent::ResetDefaults  => self.do_reset(),
            #[cfg(feature = "global-hotkeys")]
            AppEvent::ResetDefaultsWithConfirm => self.do_reset_with_confirm(event_loop),
            #[cfg(feature = "global-hotkeys")]
            AppEvent::RebindHotkeys  => self.do_rebind_hotkeys(),
            AppEvent::BeginCapturingShortcut(action) => self.do_begin_capturing_shortcut(event_loop, action),
            AppEvent::Quit => {
                self.shutdown(event_loop);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id:  WindowId,
        event:      WindowEvent,
    ) {
        // ── Settings window events ─────────────────────────────────────────
        if Some(window_id) == self.settings_win_id {
            if let Some(sw) = &mut self.settings_win {
                let (consumed, wants_repaint) = sw.on_window_event(&event);
                // egui signals via `repaint` when it needs a fresh paint
                // (mouse move, click, focus change, tooltip, etc).
                //
                // BUT egui_winit returns `repaint=true` for `RedrawRequested`
                // itself: if we honour that, every paint schedules another
                // paint and we spin at refresh rate.  Skip it: we're already
                // about to render below, no second request needed.
                if wants_repaint && !matches!(event, WindowEvent::RedrawRequested) {
                    // No app-level frame cap because wgpu's
                    // `PresentMode::Fifo` already bounds presentation at
                    // display refresh (60-120 Hz), and each overlay's
                    // render thread runs on its own `ID3D12CommandQueue`
                    // / Metal queue / Vulkan queue, so a hover storm on
                    // the settings window can't starve overlay presents
                    sw.request_redraw();
                    self.next_settings_repaint = None;
                }
                match &event {
                    WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                        // Persist position + height via the shared
                        // capture helper.  Height is stored as logical
                        // points (settings window has a fixed width;
                        // logical-vs-physical matters because next
                        // launch feeds height back through
                        // `LogicalSize::new(...)`).
                        let placement = placement::capture_placement_logical_height(
                            event_loop, &sw.window,
                        );
                        let mut s = self.settings.write_or_recover();
                        s.set_settings_window_placement(placement);
                        self.settings_manager.mark_dirty();
                    }
                    WindowEvent::RedrawRequested => {
                        // Snapshot settings into a local so the render pass
                        // (egui build + GPU submit + present — several ms)
                        // never holds the shared RwLock.  All settings
                        // mutations happen on this thread, so the only
                        // concurrent access is readers (controller,
                        // settings-writer) — they can run freely while egui
                        // works on the clone.  Commit-back at the end is a
                        // ~200-byte memcpy held for microseconds.
                        //
                        // Before this fix: `settings.write()` was held for
                        // the entire paint pass and the controller's per-
                        // tick `settings.read()` hit `lock_contended` ~17 %
                        // of the time, which was the dominant component of
                        // the idle CPU baseline.
                        let before = self.settings.read_or_recover().clone();
                        let mut settings = before.clone();

                        let repaint_delay = match sw.render(&mut settings, &self.settings_manager) {
                            Ok(d)  => d,
                            Err(e) => { error!("settings render: {e}"); std::time::Duration::MAX }
                        };

                        // Quit-button clicks dispatch directly through the
                        // `on_quit` callback passed in at SettingsWindow
                        // construction — no flag polling here

                        // Diff before/after in one pass.  Adding a new
                        // setting is a one-line edit to
                        // `SettingsDiff::from` in exhale-core
                        let diff = exhale_core::settings::SettingsDiff::from(&before, &settings);

                        // Commit the (possibly-mutated) snapshot back.  The
                        // write lock is held only for a struct assignment,
                        // not for the whole paint pass.
                        *self.settings.write_or_recover() = settings.clone();

                        // Schedule egui's requested next repaint via a
                        // deadline checked in about_to_wait.  `MAX` means
                        // egui has nothing animating — the window can sit
                        // idle until the next input event.
                        self.next_settings_repaint = if repaint_delay == std::time::Duration::MAX {
                            None
                        } else {
                            Some(Instant::now() + repaint_delay)
                        };

                        // Reschedule timers if relevant settings changed.
                        if diff.auto_stop_changed || diff.animating_changed {
                            self.timers.reschedule_auto_stop(&settings);
                            self.update_tray_state(&settings);
                        }
                        // Sync windowed-fallback visibility with
                        // `is_animating`.  Settings-panel Start /
                        // Stop buttons mutate `is_animating` here and
                        // need the overlay's window to show / hide to
                        // match — `do_start` / `do_stop` (the global
                        // hotkey, tray-menu, and close-X paths) call
                        // `set_animation_visible` themselves, so this
                        // line specifically covers the panel-button
                        // path.  `set_animation_visible` is a no-op
                        // on alpha-capable (fullscreen click-through)
                        // overlays, so this is harmless on the
                        // transparent-overlay path on macOS /
                        // Windows 11 / X11
                        if diff.animating_changed {
                            for h in self.overlays.values() {
                                h.set_animation_visible(settings.is_animating);
                            }
                        }
                        if diff.reminder_changed {
                            self.timers.reschedule_reminder(&settings);
                            // Request notification permission when reminders are first enabled
                            // (macOS-only; no-op elsewhere).
                            if settings.reminder_interval_minutes > 0.0 {
                                platform::request_notification_permission();
                            }
                        }

                        let should_restart = diff.should_restart_animation(&settings);
                        let should_redraw  = diff.should_redraw_overlay();

                        // Apply platform-specific visibility: macOS activation policy,
                        // Windows taskbar entry, Linux SKIP_TASKBAR/SKIP_PAGER.
                        if diff.visibility_changed {
                            let settings_win = self.settings_win.as_ref().map(|sw| sw.window.as_ref());
                            platform::apply_app_visibility(diff.new_visibility, settings_win);
                            self.sync_tray_to_visibility(diff.new_visibility);
                        }

                        // Restart animation from inhale-phase-0 when visual settings change —
                        // matches Swift ContentView's triggerAnimationReset() behavior.
                        if should_restart {
                            if let Some(c) = &self.controller { c.restart(); }
                        }
                        // Only fire an immediate redraw when we are
                        // NOT also restarting the controller.
                        //
                        // `c.restart()` now wakes the controller via
                        // `unpark()` and produces a fresh frame
                        // through its own `request_draw` path with
                        // the post-reset `BreathingState`.  If we
                        // ALSO `wake_render` here, the render thread
                        // receives our message first — reads whatever
                        // stale state is still sitting in the shared
                        // mutex from before the Stop, paints it, and
                        // the user sees a one-frame flash of the
                        // previous cycle's animation before the
                        // controller's reset-driven frame arrives.
                        // Skipping `wake_render` in the restart case
                        // means the next visible frame is always the
                        // post-reset one.  Non-restart redraws
                        // (visual / paused changes while idle) still
                        // need the explicit nudge since the
                        // controller is sleeping for the long
                        // not-animating interval
                        if should_redraw && !should_restart {
                            for h in self.overlays.values() { h.wake_render(); }
                        }
                    }
                    WindowEvent::CloseRequested => {
                        // Platform-conventional behavior:
                        //   macOS / Windows — closing the settings
                        //     window HIDES it; the menu bar (mac) / tray
                        //     icon (win) keeps the app alive. Matches
                        //     NSApp + tray-resident-app conventions.
                        //   Linux — closing the window QUITS the app.
                        //     Tray-icon support is unreliable across DEs
                        //     (Ubuntu has it via AppIndicator extension;
                        //     many distros / sessions don't show the
                        //     tray icon at all), and on Wayland sessions
                        //     where overlay click-through isn't honored,
                        //     the close button may be the only way the
                        //     user can dismiss the app.  Quit-on-close
                        //     gives them a guaranteed escape hatch.
                        #[cfg(all(unix, not(target_os = "macos")))]
                        {
                            self.shutdown(event_loop);
                        }
                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        {
                            if let Some(sw) = &self.settings_win {
                                sw.window.set_visible(false);
                            }
                        }
                    }
                    _ => {}
                }
                if consumed { return; }
            }
            return;
        }

        // ── Overlay window events ──────────────────────────────────────────
        //
        // Overlay windows: rendering lives on a dedicated thread per
        // window on macOS / Windows / Linux X11 (see `OverlayHandle`),
        // so the main thread only forwards resizes there.  On Wayland
        // the renderer lives on the handle and is driven from
        // `RedrawRequested` here, because wgpu's surface acquisition
        // must run synchronized with the compositor's frame_callback
        // protocol which only arrives through this event loop —
        // bypassing it from a background thread leaves the
        // xdg_toplevel in a "configured but unmapped" state
        if let Some(handle) = self.overlays.get(&window_id) {
            // Persist windowed-mode placement on every Moved /
            // Resized via the shared capture helper.  No-op for
            // fullscreen overlay windows: they're sized to the
            // monitor at creation and the user can't drag them, so
            // their `Moved` / `Resized` only fires on monitor
            // hot-plug — already handled by `rescan_monitors`,
            // doesn't need to bleed into the placement file
            let is_windowed_mode = !handle.alpha_capable;
            let persist_placement = is_windowed_mode && matches!(
                &event,
                WindowEvent::Moved(_) | WindowEvent::Resized(_),
            );
            // In windowed-fallback mode (Wayland, Windows 10 Vulkan
            // without alpha support, WARP / remote-desktop) the
            // overlay is a regular OS window with title-bar X / Alt-
            // F4 / red dot.  Native conventions are that clicking
            // close on a movable app window dismisses it; treating
            // that as a Stop press (hide window, halt animation,
            // tray + settings stay alive) matches every other "Stop"
            // input source — keyboard hotkey, tray menu, Stop
            // button.  Fullscreen click-through overlays (alpha-
            // capable path) never receive `CloseRequested` because
            // they have no chrome to close from
            let was_close = matches!(&event, WindowEvent::CloseRequested);
            match event {
                WindowEvent::Resized(size)   => handle.resize(size),
                WindowEvent::RedrawRequested => handle.render_on_main(),
                _ => {}
            }
            if persist_placement {
                let placement = placement::capture_placement(
                    event_loop, &handle.window,
                );
                let mut s = self.settings.write_or_recover();
                s.set_animation_window_placement(placement);
                self.settings_manager.mark_dirty();
            }
            if was_close && is_windowed_mode {
                // Route through `do_stop` (not raw `set_visible`) so
                // we also clear `is_animating`, reschedule auto-stop,
                // update tray menu state, etc. — identical effect to
                // pressing the Stop button in the settings panel
                self.do_stop();
            }
            return;
        }

        // ── Global close / quit ────────────────────────────────────────────
        if let WindowEvent::CloseRequested = event {
            self.shutdown(event_loop);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // macOS: show settings when the user clicks the Dock icon while running.
        // DOCK_REOPEN is always defined but only ever set by the macOS handler.
        if platform::DOCK_REOPEN.swap(false, std::sync::atomic::Ordering::Relaxed) {
            let _ = self.proxy.send_event(AppEvent::ShowSettings);
        }

        // Re-assert topmost ordering on a 1-second cadence.  Used to ride
        // along with each overlay render pass, but now that overlays
        // render on their own threads, the main thread owns this beat.
        // Windows-only — the underlying `reassert_overlay_topmost` is a
        // no-op elsewhere, so don't bother waking the loop at 1 Hz on
        // macOS / Linux for nothing.
        #[cfg(target_os = "windows")]
        self.maybe_reassert_topmost();

        // Monitor hot-plug rescan on a ~2 s cadence.  Detects monitors
        // being connected or disconnected at runtime so the overlay
        // count stays in sync without subscribing to platform-specific
        // notifications (NSApplicationDidChangeScreenParameters /
        // WM_DISPLAYCHANGE / XRandR).  Cheap: `available_monitors()`
        // resolves in microseconds and the diff is a HashSet compare
        if let Some(deadline) = self.next_monitor_scan {
            if Instant::now() >= deadline {
                self.rescan_monitors(event_loop);
                self.next_monitor_scan = Some(Instant::now() + Duration::from_secs(2));
            }
        }

        // Linux: pump pending GTK events without blocking so the tray
        // icon's libayatana-appindicator backend can process clicks /
        // theme changes.  We run GTK on the main thread alongside winit
        // (instead of on a dedicated thread) because tray-icon's API
        // requires the menu to be built and serviced from the same
        // thread that called `gtk::init()` — which is the main thread
        // for us.
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }

        // Poll tray menu events — drain the channel so multiple
        // queued events (rare but possible if the user clicks
        // through several menu items between event-loop ticks)
        // get processed in one pass instead of one per call.
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(ids) = &self.tray_ids {
                let id = &event.id;
                // Check the "Keyboard Shortcuts ▶" submenu first —
                // those items don't execute the action, they open
                // the settings window in capture mode for the
                // matching `ShortcutAction`.  `kb_action_for`
                // returns `None` for any non-submenu id, falling
                // through to the action-execution branches below
                if let Some(action) = ids.kb_action_for(id) {
                    let _ = self.proxy.send_event(AppEvent::BeginCapturingShortcut(action));
                }
                else if id == &ids.preferences { let _ = self.proxy.send_event(AppEvent::ShowSettings); }
                else if id == &ids.start  { let _ = self.proxy.send_event(AppEvent::StartAnimation); }
                else if id == &ids.stop   { let _ = self.proxy.send_event(AppEvent::StopAnimation); }
                else if id == &ids.reset  { let _ = self.proxy.send_event(AppEvent::ResetDefaults); }
                else if id == &ids.quit   { let _ = self.proxy.send_event(AppEvent::Quit); }
            }
        }

        // Poll global hotkey events.  Drain the channel completely:
        // pre-drain, the loop popped only ONE event per
        // `about_to_wait` tick.  Since each hotkey press generates
        // both a Pressed and Released event, and the loop's idle
        // wake cadence is 2 s (monitor scan), a quick sequence of
        // keypresses could pile up many events and the visible
        // result was "the first hotkey works, later ones lag by
        // seconds or never get processed if the loop kept finding
        // other things to do."  Draining here turns that into
        // batch dispatch on the next tick instead.
        //
        // Within a single drain we also COALESCE duplicate ids so
        // mashing the same hotkey twice in the same tick only
        // sends one UserEvent.  Without this, two rapid Ctrl+Shift+F
        // presses queued two reset-confirm alerts: the user would
        // dismiss the first and a second alert would immediately
        // pop up.  Coalescing matches the Swift app's behaviour
        // where rapid presses while a modal is up are no-ops
        #[cfg(feature = "global-hotkeys")]
        {
            // Suppress action dispatch while the settings window is in
            // shortcut-capture mode — otherwise pressing the user's
            // currently-bound combo to "see what it does" or as part
            // of rebinding would fire BOTH the captured-key handler
            // (which writes a new binding) and the existing global
            // hotkey (which runs the old action).  Events still get
            // drained so the channel stays empty on capture exit
            let suppress = self.settings_win
                .as_ref()
                .is_some_and(|sw| sw.is_capturing_shortcut());
            let mut sent_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                use global_hotkey::HotKeyState;
                // We only act on Pressed; Released events for the same
                // hotkey still get drained here (just no-op'd) so they
                // don't queue up indefinitely.
                if event.state != HotKeyState::Pressed {
                    continue;
                }
                if suppress {
                    log::debug!("global hotkey id={} ignored — capture overlay active", event.id);
                    continue;
                }
                let Some(ids) = &self.hotkey_ids else { continue; };
                let id = event.id;
                if !sent_ids.insert(id) {
                    log::debug!("global hotkey id={id} already dispatched this tick, coalescing");
                    continue;
                }
                let (app_event, label): (AppEvent, &str) =
                         if Some(id) == ids.preferences { (AppEvent::ShowSettings,               "Show settings") }
                    else if Some(id) == ids.start       { (AppEvent::StartAnimation,            "Start animation") }
                    else if Some(id) == ids.stop        { (AppEvent::StopAnimation,             "Stop animation") }
                    else if Some(id) == ids.reset       { (AppEvent::ResetDefaultsWithConfirm,  "Reset to defaults (confirm)") }
                    else if Some(id) == ids.quit        { (AppEvent::Quit,                      "Quit") }
                    else {
                        log::debug!("global hotkey event with unrecognised id={id}");
                        continue;
                    };
                log::debug!("global hotkey fired: {label} (id={id})");
                let _ = self.proxy.send_event(app_event);
            }
        }

        // Tick timers.
        let events = {
            let s = self.settings.read_or_recover();
            self.timers.tick(&s)
        };
        if events.auto_stop {
            self.do_stop();
        }
        if events.reminder {
            timers::send_reminder();
        }

        // Fire a settings-window redraw only if egui has asked for one via
        // its `repaint_delay` (tooltip fade, button-press animation, etc.).
        // Previously this block unconditionally called `sw.request_redraw()`
        // every idle tick, which spun the event loop at the display's
        // refresh rate and drove a full egui + GPU paint pass ~60 times per
        // second while the settings window was open — the dominant cause of
        // the ~18 % idle CPU baseline.  Now the window sits idle until
        // there's an input event or a scheduled animation frame.
        // Handle a fired settings-repaint deadline first.
        if let Some(deadline) = self.next_settings_repaint {
            if Instant::now() >= deadline {
                if let Some(sw) = &self.settings_win {
                    if sw.window.is_visible().unwrap_or(false) {
                        sw.request_redraw();
                    }
                }
                self.next_settings_repaint = None;
            }
        }

        // Compute the earliest deadline we need the event loop to wake for:
        // an egui-requested repaint OR an auto-stop/reminder firing.  With
        // `ControlFlow::Wait` alone the loop would sleep indefinitely and
        // miss these, since nothing else wakes it on an idle desktop.
        let timer_deadline = {
            let s = self.settings.read_or_recover();
            self.timers.next_deadline(&s)
        };
        // Windows: include the topmost re-assert beat in the wake schedule
        // so the loop wakes once per second to bump our overlay (and
        // settings, when visible) back to the top of the topmost band.
        #[cfg(target_os = "windows")]
        let topmost_deadline = self.next_topmost_reassert;
        #[cfg(not(target_os = "windows"))]
        let topmost_deadline: Option<Instant> = None;
        let next = [
            self.next_settings_repaint,
            timer_deadline,
            topmost_deadline,
            self.next_monitor_scan,
        ]
            .into_iter()
            .flatten()
            .min();
        match next {
            Some(d) => event_loop.set_control_flow(ControlFlow::WaitUntil(d)),
            None    => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }
}

impl App {
    fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
        info!("shutting down — flushing settings");
        if let Some(c) = self.controller.as_mut() { c.stop(); }
        if let Err(e) = self.settings_manager.flush_sync() { error!("flush: {e}"); }
        event_loop.exit();
    }
}

// ─── main ─────────────────────────────────────────────────────────────────────


fn main() -> Result<()> {
    use bootstrap::{
        install_panic_logger, pick_log_path, single_instance_guard,
        InstanceGuard, TeeLogWriter,
    };

    let log_path = pick_log_path();
    // Default filter: our crates at INFO, but cap the GPU stack at
    // WARN so wgpu's per-frame `Device::maintain: waiting for
    // submission index N` chatter doesn't flood the log file at ~10
    // fps.  `wgpu_hal::metal::adapter` is pinned to ERROR
    // specifically to suppress the per-frame "Unable to get the
    // current view dimensions on a non-main thread" WARN — that
    // warning is benign for our use case (wgpu's fallback uses the
    // cached size from the last `configure()`, which we DO call
    // from the main thread on every Resized event).
    //
    // `sctk_adwaita` is the Wayland client-side decoration crate;
    // some compositors send button events it doesn't recognise and
    // it logs a WARN per event ("Ignoring unknown button type:").
    // Not actionable from our side, so capped at ERROR.
    //
    // Override via `RUST_LOG=info` to see everything when actually
    // debugging wgpu/naga/wayland issues.  Re-enable just the metal
    // adapter chatter with `RUST_LOG=wgpu_hal::metal=warn`.
    let mut builder = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(
            "info,\
             wgpu_core=warn,\
             wgpu_hal=warn,\
             wgpu_hal::metal::adapter=error,\
             naga=warn,\
             sctk_adwaita=error",
        ),
    );
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true).write(true).truncate(true)
        .open(&log_path)
    {
        builder.target(env_logger::Target::Pipe(
            Box::new(TeeLogWriter { file }),
        ));
    }
    builder.init();
    install_panic_logger(log_path.clone());
    info!("logging to {}", log_path.display());

    // Linux: initialise GTK before anything in the tray-icon path runs.
    // `tray-icon` builds on top of GTK + libayatana-appindicator on
    // Linux; constructing menu items without a prior `gtk::init()`
    // panics with "GTK has not been initialized".  This call is
    // cheap when GTK is already up, and we pump its event loop
    // non-blockingly inside `about_to_wait` so menu clicks dispatch.
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Err(e) = gtk::init() {
            log::error!("gtk::init failed: {e}; tray menu will be unavailable");
        }
    }

    let settings_manager = Arc::new(SettingsManager::new()?);
    info!("settings: {}", settings_manager.config_path().display());

    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    let proxy      = event_loop.create_proxy();

    let _instance_guard = match single_instance_guard(&proxy) {
        InstanceGuard::First(g)    => Some(g),
        InstanceGuard::Secondary   => return Ok(()),
        InstanceGuard::Unavailable => None,
    };

    let mut app = App::new(proxy, settings_manager);
    event_loop.run_app(&mut app)?;
    Ok(())
}
