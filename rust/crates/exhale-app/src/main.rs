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
    RequestRedraw,
    ShowSettings,
    StartAnimation,
    StopAnimation,
    TogglePause,
    ResetDefaults,
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

    // Breathing controller.
    controller: Option<BreathingController>,

    // System tray.
    _tray:    Option<tray_icon::TrayIcon>,
    tray_ids: Option<TrayMenuIds>,

    // Global hotkeys.
    hotkey_manager: Option<GlobalHotKeyManager>,
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
            controller:       None,
            _tray:            None,
            tray_ids:         None,
            hotkey_manager:   None,
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
        self.request_overlay_redraw();
        self.request_settings_redraw();
    }

    fn do_toggle_pause(&mut self) {
        let mut s = self.settings.write().unwrap();
        if s.is_animating {
            s.is_paused = !s.is_paused;
            self.settings_manager.mark_dirty();
            self.update_tray_state(&s);
            drop(s);
            // Repaint immediately so the paused/resumed state is visible.
            self.request_overlay_redraw();
            self.request_settings_redraw();
        }
    }

    fn update_tray_state(&self, s: &exhale_core::settings::Settings) {
        if let Some(ids) = &self.tray_ids {
            // Start disabled while animating (matches Swift AppDelegate).
            ids.start_item.set_enabled(!s.is_animating);
            // Stop enabled when animating OR paused (matches Swift AppDelegate).
            ids.stop_item.set_enabled(s.is_animating || s.is_paused);
            // Pause only meaningful while animating.
            ids.pause_item.set_enabled(s.is_animating);
            let label = if s.is_paused { "Resume" } else { "Pause" };
            ids.pause_item.set_text(label);
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

    fn render_overlays(&mut self) {
        let state    = self.controller.as_ref().and_then(|c| c.get_state());
        let settings = self.settings.read().unwrap().clone();

        // Use a zero-progress state when stopped/paused so the shader only needs
        // display_mode to decide what to draw. Fallback to a default state if the
        // controller hasn't ticked yet (before first frame).
        let state = state.unwrap_or_else(|| exhale_core::controller::BreathingState {
            phase:     exhale_core::types::BreathingPhase::Inhale,
            progress:  0.0,
            hold_time: 0.0,
        });

        for handle in self.overlays.values_mut() {
            if let Err(e) = handle.render(&state, &settings, self.primary_max_circle_scale) {
                error!("overlay render: {e}");
            }
        }
    }

    fn request_overlay_redraw(&self) {
        for h in self.overlays.values() { h.window.request_redraw(); }
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
        let handles = OverlayHandle::create_all(event_loop, Arc::clone(&gpu));
        if handles.is_empty() {
            error!("no overlay windows created");
            event_loop.exit();
            return;
        }
        for h in handles {
            self.overlays.insert(h.window.id(), h);
        }
        info!("{} overlay window(s) created", self.overlays.len());

        // ── Start breathing controller ────────────────────────────────────────
        let proxy = self.proxy.clone();
        self.controller = Some(BreathingController::start(
            Arc::clone(&self.settings),
            Arc::new(move || { let _ = proxy.send_event(AppEvent::RequestRedraw); }),
        ));

        // ── System tray ───────────────────────────────────────────────────────
        {
            let vis = self.settings.read().unwrap().app_visibility;
            self.sync_tray_to_visibility(vis);
        }

        // ── Global hotkeys ────────────────────────────────────────────────────
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
            AppEvent::RequestRedraw => {
                for h in self.overlays.values() { h.window.request_redraw(); }
            }
            AppEvent::ShowSettings   => self.toggle_settings(event_loop),
            AppEvent::StartAnimation => self.do_start(),
            AppEvent::StopAnimation  => self.do_stop(),
            AppEvent::TogglePause    => self.do_toggle_pause(),
            AppEvent::ResetDefaults  => self.do_reset(),
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
                    sw.request_redraw();
                    self.next_settings_repaint = None;
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
                        let mut s = self.settings.write().unwrap();
                        s.settings_window_height = Some(size.height);
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
                            self.request_overlay_redraw();
                        }
                    }
                    WindowEvent::CloseRequested => {
                        if let Some(sw) = &self.settings_win {
                            sw.window.set_visible(false);
                        }
                    }
                    _ => {}
                }
                if consumed { return; }
            }
            return;
        }

        // ── Overlay window events ──────────────────────────────────────────
        if let Some(handle) = self.overlays.get_mut(&window_id) {
            match event {
                WindowEvent::RedrawRequested => self.render_overlays(),
                WindowEvent::Resized(size)   => handle.resize(size),
                _ => {}
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

        // Poll tray menu events.
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(ids) = &self.tray_ids {
                let id = &event.id;
                if id == &ids.preferences { let _ = self.proxy.send_event(AppEvent::ShowSettings); }
                else if id == &ids.start  { let _ = self.proxy.send_event(AppEvent::StartAnimation); }
                else if id == &ids.stop   { let _ = self.proxy.send_event(AppEvent::StopAnimation); }
                else if id == &ids.pause  { let _ = self.proxy.send_event(AppEvent::TogglePause); }
                else if id == &ids.reset  { let _ = self.proxy.send_event(AppEvent::ResetDefaults); }
                else if id == &ids.quit   { let _ = self.proxy.send_event(AppEvent::Quit); }
            }
        }

        // Poll global hotkey events.
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
        let next = match (self.next_settings_repaint, timer_deadline) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None)    => Some(a),
            (None,    Some(b)) => Some(b),
            (None,    None)    => None,
        };
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

/// Bind a localhost port to enforce a single running instance.
/// Returns the listener (must stay alive for the lifetime of the process).
/// If another instance is already running, signals it to show settings, then returns None.
fn single_instance_guard(proxy: &EventLoopProxy<AppEvent>) -> Option<std::net::TcpListener> {
    match std::net::TcpListener::bind(format!("127.0.0.1:{INSTANCE_PORT}")) {
        Ok(listener) => {
            // Start a thread that accepts one-shot "show-settings" signals from
            // secondary launches, matching Swift's DistributedNotification behavior.
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
            Some(listener)
        }
        Err(_) => {
            // Signal the running instance to surface its settings window, then exit.
            use std::io::Write;
            if let Ok(mut s) = std::net::TcpStream::connect(format!("127.0.0.1:{INSTANCE_PORT}")) {
                let _ = s.write_all(b"show");
            }
            None
        }
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let settings_manager = Arc::new(SettingsManager::new()?);
    info!("settings: {}", settings_manager.config_path().display());

    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    let proxy      = event_loop.create_proxy();

    let _instance_guard = match single_instance_guard(&proxy) {
        Some(g) => g,
        None    => return Ok(()),
    };

    let mut app = App::new(proxy, settings_manager);
    event_loop.run_app(&mut app)?;
    Ok(())
}
