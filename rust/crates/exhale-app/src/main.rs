// On Windows, suppress the console window that pops up alongside the
// app when launched from Explorer / Start.  `release` builds get the
// `windows` subsystem (no console), `debug` builds keep the console so
// `cargo run` and `RUST_LOG` output remain visible while developing.
//
// Linux + macOS aren't affected — they don't have the implicit-console
// behavior Windows does for entry-point executables.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

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
    time::Instant,
};

use anyhow::Result;
use exhale_core::{
    controller::BreathingController,
    settings::Settings,
    settings_manager::SettingsManager,
};
use exhale_render::GpuContext;
#[cfg(feature = "global-hotkeys")]
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use log::{error, info};
use overlay::OverlayHandle;
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

// ─── User event ───────────────────────────────────────────────────────────────

#[derive(Debug)]
enum AppEvent {
    ShowSettings,
    StartAnimation,
    StopAnimation,
    ResetDefaults,
    #[cfg(feature = "global-hotkeys")]
    ResetDefaultsWithConfirm,
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
    // returned by egui's `repaint_delay`.  `None` means no scheduled repaint
    // — the window sits idle until an input event or external mutation.
    // Replaces the old per-tick `request_redraw` loop that spun the event
    // loop at refresh rate while the settings window was visible.
    next_settings_repaint: Option<Instant>,

    // Wall-clock time of the most recent settings-window render.  Used
    // to cap the settings repaint cadence at ~30 fps (33 ms minimum
    // between renders), so high-frequency egui repaint requests from
    // mouse-hover storms don't burn idle CPU or crowd out the
    // overlay's main-thread / wgpu-queue / DXGI-present slots.  Always
    // on (not gated to "while animating") — 33 ms is below human
    // perception for hover responsiveness, and the cap protects
    // battery life on idle settings panels too.  `None` means we
    // haven't rendered yet.
    last_settings_render: Option<Instant>,

    // Throttle for `platform::reassert_overlay_topmost` on Windows —
    // Windows orders topmost windows by activation, so a newly-opened
    // app can land above our overlay until we re-bump it to the front
    // of the topmost band.  We re-assert at most once per second
    // (negligible CPU vs every-frame, still imperceptible latency
    // before the overlay reclaims the top).  None = never re-asserted
    // yet; gets set after the first call.
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
            primary_max_circle_scale: 1.0,
            settings_win:     None,
            settings_win_id:  None,
            next_settings_repaint: None,
            last_settings_render:  None,
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
            let settings_snap = self.settings.read().unwrap().clone();
            match SettingsWindow::new(event_loop, Arc::clone(gpu), &settings_snap) {
                Ok(sw) => {
                    self.settings_win_id = Some(sw.window.id());
                    self.settings_win    = Some(sw);
                }
                Err(e) => error!("settings window: {e}"),
            }
        }
    }

    fn do_start(&mut self) {
        let mut s = self.settings.write().unwrap();
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
        self.request_settings_redraw();
    }

    fn do_stop(&mut self) {
        let mut s = self.settings.write().unwrap();
        s.is_animating = false;
        s.is_paused    = false;
        self.timers.reschedule_auto_stop(&s);
        self.settings_manager.mark_dirty();
        self.update_tray_state(&s);
        drop(s);
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
            match tray::build_tray() {
                Ok((t, ids)) => {
                    let s = self.settings.read().unwrap();
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
        let mut s = self.settings.write().unwrap();
        // Preserve runtime + window-placement state, matching the in-window
        // ↺ Reset button.  Swift's resetToDefaults only keeps isAnimating; we
        // also keep is_paused and the persisted settings-window geometry so
        // the user doesn't lose their window position on a defaults reset.
        let was_animating  = s.is_animating;
        let was_paused     = s.is_paused;
        let win_x          = s.settings_window_x;
        let win_y          = s.settings_window_y;
        let win_h          = s.settings_window_height;
        let win_screen     = s.settings_window_screen.clone();
        *s = Settings::default();
        s.is_animating            = was_animating;
        s.is_paused               = was_paused;
        s.settings_window_x       = win_x;
        s.settings_window_y       = win_y;
        s.settings_window_height  = win_h;
        s.settings_window_screen  = win_screen;
        self.timers.reschedule_auto_stop(&s);
        self.timers.reschedule_reminder(&s);
        self.settings_manager.mark_dirty();
        self.update_tray_state(&s);
        drop(s);
        self.request_settings_redraw();
    }

    /// Open the settings window (if hidden) and raise the reset confirmation
    /// dialog.  Matches Swift's `showResetConfirmation()` on the Ctrl+Shift+F
    /// global hotkey — an `NSAlert` is shown before `resetToDefaults` runs.
    #[cfg(feature = "global-hotkeys")]
    fn do_reset_with_confirm(&mut self, event_loop: &ActiveEventLoop) {
        // Ensure the settings window exists and is visible.
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

    /// Re-assert topmost ordering at most once per second on Windows.
    ///
    /// Windows orders topmost windows by activation, so a newly-opened
    /// app can land above our overlay until we re-bump it to the front
    /// of the topmost band.  After bumping the overlay forward, we
    /// also bump the settings window (when visible) so the settings
    /// panel ends up ABOVE the overlay — otherwise `overlay_opacity =
    /// 1.0` would lock the user out by covering the controls.  No-op
    /// on non-Windows.  Driven from `about_to_wait` now that overlay
    /// rendering lives on dedicated threads — previously this rode
    /// along with each overlay render pass on the main thread.
    #[cfg(target_os = "windows")]
    fn maybe_reassert_topmost(&mut self) {
        let now = Instant::now();
        let due = self.next_topmost_reassert.map_or(true, |t| now >= t);
        if !due { return; }
        for handle in self.overlays.values() {
            platform::reassert_overlay_topmost(&handle.window);
        }
        if let Some(sw) = &self.settings_win {
            if sw.window.is_visible().unwrap_or(false) {
                platform::reassert_overlay_topmost(&sw.window);
            }
        }
        self.next_topmost_reassert = Some(now + std::time::Duration::from_secs(1));
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
        let frame_senders: Vec<_> = handles.iter().map(|h| h.frame_sender()).collect();
        for h in handles {
            self.overlays.insert(h.window.id(), h);
        }
        info!("{} overlay window(s) created", self.overlays.len());

        // ── Start breathing controller ────────────────────────────────────────
        //
        // `request_draw` fans out a Frame message to every overlay's
        // render thread, bypassing the main event loop.  No WM_PAINT
        // dance and no risk of WM_MOUSEMOVE starvation.
        self.controller = Some(BreathingController::start(
            Arc::clone(&self.settings),
            breathing_state,
            Arc::new(move || {
                for tx in &frame_senders {
                    tx.send_frame();
                }
            }),
        ));

        // ── System tray ───────────────────────────────────────────────────────
        {
            let vis = self.settings.read().unwrap().app_visibility;
            self.sync_tray_to_visibility(vis);
        }

        // ── Global hotkeys ────────────────────────────────────────────────────
        // Gated behind the `global-hotkeys` feature so the Mac App Store
        // build (`--no-default-features`) ships without the Carbon-based
        // hotkey registration as a hedge against App Review flagging it.
        #[cfg(feature = "global-hotkeys")]
        match GlobalHotKeyManager::new() {
            Ok(mgr) => {
                match hotkeys::register_hotkeys(&mgr) {
                    Ok(ids) => { self.hotkey_ids = Some(ids); }
                    Err(e)  => error!("hotkey registration: {e}"),
                }
                self.hotkey_manager = Some(mgr);
            }
            Err(e) => error!("hotkey manager: {e}"),
        }

        // ── Timer init ────────────────────────────────────────────────────────
        {
            let s = self.settings.read().unwrap();
            self.timers.reschedule_auto_stop(&s);
            self.timers.reschedule_reminder(&s);
        }

        // ── Dock-icon reopen handler (macOS-only internally; no-op elsewhere) ─
        platform::register_reopen_handler();

        // ── Notification permission (macOS-only internally; no-op elsewhere) ──
        if self.settings.read().unwrap().reminder_interval_minutes > 0.0 {
            platform::request_notification_permission();
        }

        // Show the settings window on first launch (creates it as a side effect).
        self.toggle_settings(event_loop);

        // ── Activation policy / taskbar presence ──────────────────────────────
        // Cross-platform: macOS toggles NSApp activation policy; Windows toggles
        // the settings window's taskbar entry; Linux toggles SKIP_TASKBAR/PAGER.
        // Applied AFTER the settings window exists so Windows/Linux can see it.
        {
            let vis = self.settings.read().unwrap().app_visibility;
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
                // (mouse move, click, focus change, tooltip, etc).  Drive
                // our redraw off that instead of the old per-tick polling.
                //
                // BUT egui_winit returns `repaint=true` for `RedrawRequested`
                // itself — if we honour that, every paint schedules another
                // paint and we spin at refresh rate.  Skip it: we're already
                // about to render below, no second request needed.
                if wants_repaint && !matches!(event, WindowEvent::RedrawRequested) {
                    // Always cap the settings window's repaint cadence
                    // at ~30 fps (33 ms minimum between renders).  egui
                    // fires `wants_repaint = true` on every
                    // `CursorMoved` event so it can refresh hover
                    // colours / tooltips — moving the mouse fast over
                    // the settings window produces 100+ repaint
                    // requests per second otherwise.  At that rate:
                    //   • on Windows, repaints contend with the overlay
                    //     for the main thread, the shared wgpu device
                    //     queue, and DXGI flip-model present sync,
                    //     producing visible animation lag.
                    //   • on every platform, it burns idle CPU /
                    //     battery for hover effects nobody would
                    //     notice rendered at 1000 fps vs 30 fps.
                    // 33 ms is well below the ~80 ms human perception
                    // threshold for hover responsiveness, so the
                    // throttle is invisible to users.  Discrete
                    // controls (steppers, color picker, checkboxes)
                    // don't benefit from higher cadences than this.
                    const MIN_REPAINT_INTERVAL: std::time::Duration
                        = std::time::Duration::from_millis(33);
                    let now = Instant::now();
                    let throttled_until = self.last_settings_render
                        .map(|t| t + MIN_REPAINT_INTERVAL);
                    if let Some(t) = throttled_until.filter(|&t| now < t) {
                        // Inside the throttle window — defer the
                        // repaint to `about_to_wait`, which already
                        // wakes the event loop at `next_settings_repaint`.
                        self.next_settings_repaint = Some(
                            self.next_settings_repaint.map_or(t, |prev| prev.min(t))
                        );
                    } else {
                        sw.request_redraw();
                        self.next_settings_repaint = None;
                    }
                }
                match &event {
                    WindowEvent::Moved(pos) => {
                        // Persist the window position as an offset relative to
                        // its current monitor, matching Swift's
                        // AppDelegate.windowDidMove which saves screen name +
                        // screen-relative x/y.  This survives monitor
                        // reconfiguration: when the saved screen is gone we
                        // fall back to default placement rather than restoring
                        // off-screen coordinates.
                        let mut s = self.settings.write().unwrap();
                        if let Some(mon) = sw.window.current_monitor() {
                            let origin = mon.position();
                            s.settings_window_x      = Some(pos.x - origin.x);
                            s.settings_window_y      = Some(pos.y - origin.y);
                            s.settings_window_screen = mon.name();
                        } else {
                            s.settings_window_x      = Some(pos.x);
                            s.settings_window_y      = Some(pos.y);
                            s.settings_window_screen = None;
                        }
                        self.settings_manager.mark_dirty();
                    }
                    WindowEvent::Resized(size) => {
                        // Persist the height as LOGICAL points, not
                        // physical pixels.  `size.height` is physical;
                        // the next `SettingsWindow::new` reads this back
                        // and feeds it into `LogicalSize::new(...)`.  If
                        // we saved physical, every launch on a 2× display
                        // would double the value (load-as-logical
                        // multiplies by the scale factor at window
                        // creation), eventually blowing past wgpu's
                        // 16384-pixel surface limit and crashing on
                        // configure.  Round to nearest integer point.
                        let scale  = sw.window.scale_factor();
                        let logical = (size.height as f64 / scale).round() as u32;
                        let mut s = self.settings.write().unwrap();
                        s.settings_window_height = Some(logical);
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
                        let mut settings = self.settings.read().unwrap().clone();
                        let prev_animating   = settings.is_animating;
                        let prev_paused      = settings.is_paused;
                        let prev_auto_stop   = settings.auto_stop_minutes;
                        let prev_reminder    = settings.reminder_interval_minutes;
                        let prev_visibility  = settings.app_visibility;
                        // Track visual settings for Swift-parity animation reset.
                        let prev_shape              = settings.shape;
                        let prev_gradient           = settings.color_fill_gradient;
                        let prev_anim_mode          = settings.animation_mode;
                        let prev_inhale             = settings.inhale_color;
                        let prev_exhale             = settings.exhale_color;
                        let prev_opacity            = settings.overlay_opacity;
                        // Track timing settings — Swift triggerAnimationReset() fires for these too.
                        let prev_inhale_dur         = settings.inhale_duration;
                        let prev_post_inhale_dur    = settings.post_inhale_hold_duration;
                        let prev_exhale_dur         = settings.exhale_duration;
                        let prev_post_exhale_dur    = settings.post_exhale_hold_duration;
                        let prev_drift              = settings.drift;
                        let prev_rand_inhale        = settings.randomized_timing_inhale;
                        let prev_rand_post_inhale   = settings.randomized_timing_post_inhale_hold;
                        let prev_rand_exhale        = settings.randomized_timing_exhale;
                        let prev_rand_post_exhale   = settings.randomized_timing_post_exhale_hold;
                        let prev_ripple_mode        = settings.hold_ripple_mode;

                        let repaint_delay = match sw.render(&mut settings, &self.settings_manager) {
                            Ok(d)  => d,
                            Err(e) => { error!("settings render: {e}"); std::time::Duration::MAX }
                        };
                        // Timestamp the actual render so the
                        // hover-storm throttle (above, in the
                        // `wants_repaint` branch) can keep the
                        // settings window at ≤ 30 fps while the
                        // overlay animation is active.
                        self.last_settings_render = Some(Instant::now());

                        // If the user clicked the Quit button in the
                        // Controls row, the render path set
                        // `pending_quit` on the SettingsWindow.  Drain
                        // it here and dispatch the same Quit event the
                        // tray-menu path uses, so all teardown runs
                        // through `shutdown` in the canonical order.
                        if sw.take_pending_quit() {
                            let _ = self.proxy.send_event(AppEvent::Quit);
                        }

                        // Commit the (possibly-mutated) snapshot back.  The
                        // write lock is held only for a struct assignment,
                        // not for the whole paint pass.
                        *self.settings.write().unwrap() = settings.clone();

                        // Schedule egui's requested next repaint via a
                        // deadline checked in about_to_wait.  `MAX` means
                        // egui has nothing animating — the window can sit
                        // idle until the next input event.
                        self.next_settings_repaint = if repaint_delay == std::time::Duration::MAX {
                            None
                        } else {
                            Some(Instant::now() + repaint_delay)
                        };

                        let started        = !prev_animating && settings.is_animating;
                        let paused_changed = settings.is_paused != prev_paused;
                        let vis_changed    = settings.app_visibility != prev_visibility;
                        let new_visibility = settings.app_visibility;
                        let visual_changed = settings.shape             != prev_shape
                            || settings.color_fill_gradient != prev_gradient
                            || settings.animation_mode      != prev_anim_mode
                            || settings.inhale_color        != prev_inhale
                            || settings.exhale_color        != prev_exhale
                            || (settings.overlay_opacity - prev_opacity).abs() > 1e-4;
                        let timing_changed =
                            (settings.inhale_duration - prev_inhale_dur).abs()                         > 1e-9
                            || (settings.post_inhale_hold_duration - prev_post_inhale_dur).abs()       > 1e-9
                            || (settings.exhale_duration - prev_exhale_dur).abs()                      > 1e-9
                            || (settings.post_exhale_hold_duration - prev_post_exhale_dur).abs()       > 1e-9
                            || (settings.drift - prev_drift).abs()                                     > 1e-9
                            || (settings.randomized_timing_inhale - prev_rand_inhale).abs()            > 1e-9
                            || (settings.randomized_timing_post_inhale_hold - prev_rand_post_inhale).abs() > 1e-9
                            || (settings.randomized_timing_exhale - prev_rand_exhale).abs()            > 1e-9
                            || (settings.randomized_timing_post_exhale_hold - prev_rand_post_exhale).abs() > 1e-9
                            || settings.hold_ripple_mode != prev_ripple_mode;

                        // Reschedule timers if relevant settings changed.
                        if settings.auto_stop_minutes != prev_auto_stop
                            || settings.is_animating  != prev_animating
                        {
                            self.timers.reschedule_auto_stop(&settings);
                            self.update_tray_state(&settings);
                        }
                        if settings.reminder_interval_minutes != prev_reminder {
                            self.timers.reschedule_reminder(&settings);
                            // Request notification permission when reminders are first enabled
                            // (macOS-only; no-op elsewhere).
                            if settings.reminder_interval_minutes > 0.0 {
                                platform::request_notification_permission();
                            }
                        }

                        let animating_changed = settings.is_animating != prev_animating;
                        let should_restart = started
                            || ((visual_changed || timing_changed) && settings.is_animating && !settings.is_paused);
                        let should_redraw = paused_changed || animating_changed || visual_changed || timing_changed;

                        // Apply platform-specific visibility: macOS activation policy,
                        // Windows taskbar entry, Linux SKIP_TASKBAR/SKIP_PAGER.
                        if vis_changed {
                            let settings_win = self.settings_win.as_ref().map(|sw| sw.window.as_ref());
                            platform::apply_app_visibility(new_visibility, settings_win);
                            self.sync_tray_to_visibility(new_visibility);
                        }

                        // Restart animation from inhale-phase-0 when visual settings change —
                        // matches Swift ContentView's triggerAnimationReset() behavior.
                        if should_restart {
                            if let Some(c) = &self.controller { c.restart(); }
                        }
                        if should_redraw {
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
        // Rendering for overlay windows lives on a dedicated thread per
        // window — see `OverlayHandle` — so the main thread only
        // forwards resizes here.  WM_PAINT for overlay windows is
        // effectively ignored: the render thread is the single source
        // of frames, and it's woken by the controller via a channel
        // that bypasses the OS message queue.  Trying to also paint on
        // WM_PAINT would race with the render thread on the surface.
        if let Some(handle) = self.overlays.get(&window_id) {
            if let WindowEvent::Resized(size) = event {
                handle.resize(size);
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

        // Poll tray menu events.
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(ids) = &self.tray_ids {
                let id = &event.id;
                if id == &ids.preferences { let _ = self.proxy.send_event(AppEvent::ShowSettings); }
                else if id == &ids.start  { let _ = self.proxy.send_event(AppEvent::StartAnimation); }
                else if id == &ids.stop   { let _ = self.proxy.send_event(AppEvent::StopAnimation); }
                else if id == &ids.reset  { let _ = self.proxy.send_event(AppEvent::ResetDefaults); }
                else if id == &ids.quit   { let _ = self.proxy.send_event(AppEvent::Quit); }
            }
        }

        // Poll global hotkey events.
        #[cfg(feature = "global-hotkeys")]
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            use global_hotkey::HotKeyState;
            if event.state == HotKeyState::Pressed {
                if let Some(ids) = &self.hotkey_ids {
                    let id = event.id;
                    if id == ids.preferences || id == ids.preferences2 {
                        let _ = self.proxy.send_event(AppEvent::ShowSettings);
                    } else if id == ids.start { let _ = self.proxy.send_event(AppEvent::StartAnimation); }
                    else if id == ids.stop    { let _ = self.proxy.send_event(AppEvent::StopAnimation); }
                    else if id == ids.reset   { let _ = self.proxy.send_event(AppEvent::ResetDefaultsWithConfirm); }
                }
            }
        }

        // Tick timers.
        let events = {
            let s = self.settings.read().unwrap();
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
            let s = self.settings.read().unwrap();
            self.timers.next_deadline(&s)
        };
        // Windows: include the topmost re-assert beat in the wake schedule
        // so the loop wakes once per second to bump our overlay (and
        // settings, when visible) back to the top of the topmost band.
        #[cfg(target_os = "windows")]
        let topmost_deadline = self.next_topmost_reassert;
        #[cfg(not(target_os = "windows"))]
        let topmost_deadline: Option<Instant> = None;
        let next = [self.next_settings_repaint, timer_deadline, topmost_deadline]
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

const INSTANCE_PORT: u16 = 47462;

enum InstanceGuard {
    /// First instance; holds the listener alive for the process lifetime.
    First(std::net::TcpListener),
    /// Another instance is already running; it has been signalled.
    Secondary,
    /// Neither bind nor connect succeeded. Most likely a sandboxed build
    /// (macOS MAS) where BSD sockets are denied. Proceed without the guard.
    Unavailable,
}

fn single_instance_guard(proxy: &EventLoopProxy<AppEvent>) -> InstanceGuard {
    match std::net::TcpListener::bind(format!("127.0.0.1:{INSTANCE_PORT}")) {
        Ok(listener) => {
            let proxy = proxy.clone();
            let listener2 = listener.try_clone().expect("clone listener");
            std::thread::spawn(move || {
                for stream in listener2.incoming() {
                    if let Ok(mut s) = stream {
                        use std::io::Read;
                        let mut buf = [0u8; 16];
                        if let Ok(n) = s.read(&mut buf) {
                            if buf[..n].starts_with(b"show") {
                                let _ = proxy.send_event(AppEvent::ShowSettings);
                            }
                        }
                    }
                }
            });
            InstanceGuard::First(listener)
        }
        Err(_) => {
            use std::io::Write;
            match std::net::TcpStream::connect(format!("127.0.0.1:{INSTANCE_PORT}")) {
                Ok(mut s) => {
                    let _ = s.write_all(b"show");
                    InstanceGuard::Secondary
                }
                Err(_) => InstanceGuard::Unavailable,
            }
        }
    }
}

/// `Write` adapter that mirrors every byte to both stderr AND a backing
/// file.  We use this as `env_logger`'s target so the same log output
/// appears in the terminal (when there is one) AND on disk next to the
/// exe — needed for windowed-app debugging where stderr is nowhere
/// reachable, including the on-Windows scenario where a black-screen
/// overlay bug renders every other window invisible until the process
/// is force-killed.  Stderr writes are best-effort: if stderr is closed
/// or piped to /dev/null we still want the file log to succeed.
struct TeeLogWriter {
    file: std::fs::File,
}

impl std::io::Write for TeeLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Best-effort stderr — ignore failures so file logging always works.
        let _ = std::io::Write::write_all(&mut std::io::stderr(), buf);
        self.file.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stderr().flush();
        self.file.flush()
    }
}

/// Pick a path to write the log file at.  Preferred location is right
/// next to the exe — most users running an unsigned dev build extract
/// to Downloads / Desktop / similar, which is writable.  Fallbacks
/// progressively widen the net so we never silently lose logs:
///   1. `<exe-dir>/exhale.log`
///   2. `<temp>/exhale.log`
///   3. `./exhale.log`
fn pick_log_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("exhale.log");
            if std::fs::OpenOptions::new()
                .create(true).write(true).truncate(true)
                .open(&candidate).is_ok()
            {
                return candidate;
            }
        }
    }
    let tmp = std::env::temp_dir().join("exhale.log");
    if std::fs::OpenOptions::new()
        .create(true).write(true).truncate(true)
        .open(&tmp).is_ok()
    {
        return tmp;
    }
    std::path::PathBuf::from("exhale.log")
}

/// Install a panic hook that appends panic info + backtrace to the log
/// file before delegating to the default hook.  Without this, panics
/// only print to stderr and are lost when stderr isn't captured.
fn install_panic_logger(log_path: std::path::PathBuf) {
    use std::sync::OnceLock;
    static PATH: OnceLock<std::path::PathBuf> = OnceLock::new();
    let _ = PATH.set(log_path);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(path) = PATH.get() {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true).create(true).open(path)
            {
                use std::io::Write;
                let _ = writeln!(f, "\n=== PANIC ===");
                let _ = writeln!(f, "{info}");
                let _ = writeln!(
                    f, "backtrace:\n{}",
                    std::backtrace::Backtrace::force_capture(),
                );
                let _ = f.flush();
            }
        }
        prev(info);
    }));
}

fn main() -> Result<()> {
    let log_path = pick_log_path();
    // Default filter: our crates at INFO, but cap the GPU stack at
    // WARN so wgpu's per-frame `Device::maintain: waiting for
    // submission index N` chatter doesn't flood the log file at ~10
    // fps.  The submission index itself is just a monotonic u64
    // counter — not a memory leak — but logging it every frame
    // bloats the log file and obscures actual events.  Override via
    // `RUST_LOG=info` to see everything when actually debugging
    // wgpu/naga/wayland issues.
    let mut builder = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(
            "info,\
             wgpu_core=warn,\
             wgpu_hal=warn,\
             naga=warn",
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
