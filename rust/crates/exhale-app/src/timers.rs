use std::time::{Duration, Instant};

use exhale_core::settings::{Settings, SettingsDiff};
use log::info;

/// `from` plus `minutes`, or `None` when that isn't a time the clock can hold
///
/// `Instant + Duration` panics on overflow, and a `settings.toml` can hold
/// any number of minutes. `try_from_secs_f64` rejects NaN, infinite and
/// negative values and anything past `Duration::MAX`. `checked_add`
/// rejects the sums that are still past the clock, which start near
/// 1.5e17 minutes on macOS and Linux. Every caller reads `None` as off
fn minutes_after(from: Instant, minutes: f64) -> Option<Instant> {
    Duration::try_from_secs_f64(minutes * 60.0)
        .ok()
        .and_then(|d| from.checked_add(d))
}

/// All timer state, checked each event-loop tick in `about_to_wait`
pub struct Timers {
    pub auto_stop_deadline: Option<Instant>,
    pub last_reminder:      Option<Instant>,
}

impl Timers {
    pub fn new() -> Self {
        Self { auto_stop_deadline: None, last_reminder: None }
    }

    /// Call whenever `is_animating` or `auto_stop_minutes` changes
    pub fn reschedule_auto_stop(&mut self, settings: &Settings) {
        self.auto_stop_deadline = if settings.auto_stop_minutes > 0.0 && settings.is_animating {
            // TOML accepts `inf` for a float setting, so a bad value
            // leaves auto-stop off
            minutes_after(Instant::now(), settings.auto_stop_minutes)
        } else {
            None
        };
    }

    /// Call whenever `reminder_interval_minutes` changes
    pub fn reschedule_reminder(&mut self, settings: &Settings) {
        self.last_reminder = if settings.reminder_interval_minutes > 0.0 {
            Some(Instant::now())
        } else {
            None
        };
    }

    /// Re-arm the timers a settings change touched, and leave the others
    /// running
    ///
    /// The settings panel and the tray's Reset both go through here. A
    /// Reset in a study changes neither timer, because the file locks
    /// them, and the auto-stop countdown counts from the latest Start:
    /// re-arming it would give the participant a fresh countdown that no
    /// Start began
    pub fn reschedule_for(&mut self, diff: &SettingsDiff, settings: &Settings) {
        if diff.auto_stop_changed || diff.animating_changed {
            self.reschedule_auto_stop(settings);
        }
        if diff.reminder_changed {
            self.reschedule_reminder(settings);
        }
    }

    /// Earliest time the event loop must wake to service a pending
    /// timer.  Returned to `about_to_wait` so it can configure
    /// `ControlFlow::WaitUntil`: without this the loop would sleep
    /// forever on idle and miss auto-stop / reminder firings, since no
    /// per-frame redraw wakes it
    pub fn next_deadline(&self, settings: &Settings) -> Option<Instant> {
        let mut next: Option<Instant> = self.auto_stop_deadline;
        if settings.reminder_interval_minutes > 0.0 {
            // A bad interval (inf, or too long for the clock) contributes
            // no deadline, same as the reminder being off
            if let Some(due) = self.last_reminder
                .and_then(|last| minutes_after(last, settings.reminder_interval_minutes))
            {
                next = Some(match next {
                    Some(n) => n.min(due),
                    None    => due,
                });
            }
        }
        next
    }

    /// Returns `true` if the animation should be stopped (auto-stop deadline hit)
    /// Returns `true` if a reminder notification should fire
    /// Caller is responsible for applying the stop and sending the notification
    ///
    /// `reminders` is false while a study can't run. The overlay is held
    /// still and the window shows an error, so a notification asking the
    /// participant to breathe would point at nothing. A reminder that comes
    /// due in that state is still used up: the interval starts over and
    /// `next_deadline` moves on. Left due, it would stay in the past and
    /// wake the event loop in a spin
    pub fn tick(&mut self, settings: &Settings, reminders: bool) -> TimerEvents {
        let now = Instant::now();
        let mut events = TimerEvents::default();

        if let Some(deadline) = self.auto_stop_deadline {
            if now >= deadline {
                self.auto_stop_deadline = None;
                events.auto_stop = true;
                info!("auto-stop timer fired");
            }
        }

        if settings.reminder_interval_minutes > 0.0 {
            // Same non-finite / too-large guard as `next_deadline`: a bad
            // interval just means the reminder never fires this tick
            let due = self.last_reminder
                .and_then(|t| minutes_after(t, settings.reminder_interval_minutes))
                .is_some_and(|due| now >= due);
            if due {
                self.last_reminder = Some(now);
                events.reminder = reminders;
            }
        }

        events
    }
}

#[derive(Default)]
pub struct TimerEvents {
    pub auto_stop: bool,
    pub reminder:  bool,
}

/// Cross-platform desktop notification ("Remember to breathe")
///
/// macOS uses the native `UNUserNotifications` framework (required for Mac
/// App Store distribution: `NSUserNotification` is deprecated and
/// `notify-rust`'s macOS backend relies on it).  Other platforms continue
/// to use `notify-rust` (D-Bus on Linux, WinRT toasts on Windows)
pub fn send_reminder() {
    info!("reminder: Remember to breathe");
    #[cfg(target_os = "macos")]
    send_reminder_macos();
    #[cfg(not(target_os = "macos"))]
    send_reminder_other();
}

#[cfg(not(target_os = "macos"))]
fn send_reminder_other() {
    let mut n = notify_rust::Notification::new();
    n.summary("exhale").body("Remember to breathe");
    if let Err(e) = n.show() {
        log::warn!("notification: {e}");
    }
}

// Force-link `UserNotifications.framework`.  Below we look up `UN*`
// classes by name through `AnyClass` / `msg_send!`, so without a
// reference to the crate here rustc drops it from the build and the
// framework's classes never register with the runtime
#[cfg(target_os = "macos")]
use objc2_user_notifications as _;

/// Deliver a local notification via `UNUserNotificationCenter`
///
/// Mirrors the Swift AppDelegate's `sendReminderNotification()`: builds a
/// `UNMutableNotificationContent` with title, body, and default sound, wraps
/// it in a `UNNotificationRequest` with a fresh UUID identifier and a `nil`
/// trigger (deliver immediately), then hands it to the shared center
///
/// Requires the bundle to be code-signed and to have been granted
/// `.alert | .sound` authorization (see
/// `platform::request_notification_permission`).  In an unsigned
/// `cargo run` build the center silently drops the request, which
/// doesn't affect development
#[cfg(target_os = "macos")]
fn send_reminder_macos() {
    use block2::StackBlock;
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    // SAFETY: framework class lookups are fallible.  Warn and skip when
    // `UserNotifications.framework` isn't linked.  Shouldn't happen now
    // that the crate is referenced above, kept as a defensive fallback
    let (Some(unc_cls), Some(content_cls), Some(sound_cls), Some(req_cls)) = (
        AnyClass::get(c"UNUserNotificationCenter"),
        AnyClass::get(c"UNMutableNotificationContent"),
        AnyClass::get(c"UNNotificationSound"),
        AnyClass::get(c"UNNotificationRequest"),
    ) else {
        log::warn!("notification: UserNotifications framework not linked, dropping reminder");
        return;
    };

    // SAFETY: `currentNotificationCenter` raises
    // `NSInternalInconsistencyException` ("bundleProxyForCurrentProcess is
    // nil") and aborts the process when we're not running from a bundled
    // .app, which is the case for `cargo run` and the bare debug binary.
    // Bail out before calling it.  Logged at info here since it repeats
    // on every reminder while running unbundled during development
    if !crate::platform::running_in_app_bundle() {
        log::info!("notification: not running from an app bundle, dropping reminder");
        return;
    }

    unsafe {
        let content: *mut AnyObject = msg_send![content_cls, alloc];
        let content: *mut AnyObject = msg_send![content, init];
        if content.is_null() {
            log::warn!("notification: UNMutableNotificationContent init returned nil");
            return;
        }

        let ns_string = objc2::class!(NSString);
        // C-string literals via `c"..."` are guaranteed nul-terminated
        // at compile time, no runtime `CString::new(...).unwrap()`,
        // no allocation per reminder fire
        let title:  *mut AnyObject = msg_send![ns_string, stringWithUTF8String: c"exhale".as_ptr()];
        let body:   *mut AnyObject = msg_send![ns_string, stringWithUTF8String: c"Remember to breathe".as_ptr()];
        let _: () = msg_send![content, setTitle: title];
        let _: () = msg_send![content, setBody:  body];

        let sound: *mut AnyObject = msg_send![sound_cls, defaultSound];
        let _: () = msg_send![content, setSound: sound];

        let uuid:       *mut AnyObject = msg_send![objc2::class!(NSUUID), UUID];
        let identifier: *mut AnyObject = msg_send![uuid, UUIDString];

        let trigger: *mut AnyObject = std::ptr::null_mut();
        let request: *mut AnyObject = msg_send![
            req_cls,
            requestWithIdentifier: identifier,
            content:               content,
            trigger:               trigger,
        ];

        let center: *mut AnyObject = msg_send![unc_cls, currentNotificationCenter];
        if !center.is_null() && !request.is_null() {
            let block = StackBlock::new(|err: *mut AnyObject| {
                if !err.is_null() {
                    log::warn!("notification delivery returned an NSError");
                }
            });
            let block = block.copy();
            let _: () = msg_send![
                center,
                addNotificationRequest: request,
                withCompletionHandler:  &*block,
            ];
        } else {
            log::warn!("notification: center or request unavailable, dropping reminder");
        }

        // Balance the +1 retain from `[UNMutableNotificationContent alloc] init]`.
        // `requestWithIdentifier:...` retains `content` internally, and the
        // request / sound / strings / uuid are autoreleased convenience
        // returns that we don't own
        let _: () = msg_send![content, release];
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// Cover the `Timers` state machine: auto-stop firing, reminder firing,
// deadline computation, edge cases around 0-value (off) settings
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    fn settings_with(auto_stop: f64, reminder: f64, animating: bool) -> Settings {
        let mut s = Settings::default();
        s.auto_stop_minutes        = auto_stop;
        s.reminder_interval_minutes = reminder;
        s.is_animating              = animating;
        s
    }

    #[test]
    fn reschedule_auto_stop_off_when_zero() {
        let mut t = Timers::new();
        t.reschedule_auto_stop(&settings_with(0.0, 0.0, true));
        assert!(t.auto_stop_deadline.is_none(), "0 minutes = off");
    }

    #[test]
    fn reschedule_auto_stop_off_when_not_animating() {
        let mut t = Timers::new();
        t.reschedule_auto_stop(&settings_with(5.0, 0.0, false));
        assert!(t.auto_stop_deadline.is_none(),
            "auto-stop must be inactive while not animating");
    }

    #[test]
    fn reschedule_auto_stop_sets_deadline_in_future() {
        let mut t = Timers::new();
        let before = Instant::now();
        t.reschedule_auto_stop(&settings_with(2.0, 0.0, true));
        let deadline = t.auto_stop_deadline.expect("deadline set");
        let dt = deadline.duration_since(before);
        // 2 minutes = 120s.  Allow ±100ms for clock noise
        assert!(dt >= Duration::from_secs_f64(119.9));
        assert!(dt <= Duration::from_secs_f64(120.1));
    }

    #[test]
    fn reschedule_reminder_off_when_zero() {
        let mut t = Timers::new();
        t.reschedule_reminder(&settings_with(0.0, 0.0, true));
        assert!(t.last_reminder.is_none());
    }

    #[test]
    fn reschedule_reminder_resets_last_to_now() {
        let mut t = Timers::new();
        let before = Instant::now();
        t.reschedule_reminder(&settings_with(0.0, 1.0, true));
        let last = t.last_reminder.expect("last set");
        assert!(last >= before);
    }

    #[test]
    fn next_deadline_none_when_no_timers_active() {
        let t = Timers::new();
        let s = settings_with(0.0, 0.0, true);
        assert!(t.next_deadline(&s).is_none());
    }

    #[test]
    fn next_deadline_picks_earliest_of_auto_stop_and_reminder() {
        let mut t = Timers::new();
        let s = settings_with(10.0, 1.0, true); // auto-stop 10 min, reminder 1 min
        t.reschedule_auto_stop(&s);
        t.reschedule_reminder(&s);
        let d = t.next_deadline(&s).expect("some deadline");
        let now = Instant::now();
        // Reminder fires in ~1 min, auto-stop in ~10 min -> next should be ~1 min
        let dt = d.duration_since(now);
        assert!(dt < Duration::from_secs(90),
            "earliest should be the 1-minute reminder, got {dt:?}");
    }

    #[test]
    fn tick_fires_auto_stop_when_deadline_passes() {
        let mut t = Timers::new();
        // Use a tiny duration we can wait past.  We can't sleep in
        // tests reliably, so backdate the deadline directly
        t.auto_stop_deadline = Some(Instant::now() - Duration::from_millis(1));
        let events = t.tick(&settings_with(5.0, 0.0, true), true);
        assert!(events.auto_stop, "auto-stop event must fire when deadline passed");
        assert!(t.auto_stop_deadline.is_none(), "deadline cleared after firing");
    }

    #[test]
    fn tick_does_not_fire_auto_stop_early() {
        let mut t = Timers::new();
        t.auto_stop_deadline = Some(Instant::now() + Duration::from_secs(60));
        let events = t.tick(&settings_with(5.0, 0.0, true), true);
        assert!(!events.auto_stop);
        assert!(t.auto_stop_deadline.is_some());
    }

    #[test]
    fn tick_fires_reminder_at_interval_and_resets_clock() {
        let mut t = Timers::new();
        // Backdate so the interval has just passed
        t.last_reminder = Some(Instant::now() - Duration::from_secs(61));
        let s = settings_with(0.0, 1.0, true);
        let events = t.tick(&s, true);
        assert!(events.reminder, "reminder fires after 1-min interval elapsed");
        let new_last = t.last_reminder.expect("reset");
        assert!(new_last > Instant::now() - Duration::from_secs(1),
            "last_reminder reset to ~now after firing");
    }

    #[test]
    fn a_muted_reminder_is_used_up_and_never_fires() {
        let mut t = Timers::new();
        t.last_reminder = Some(Instant::now() - Duration::from_secs(61));
        let s = settings_with(0.0, 1.0, true);

        let events = t.tick(&s, false);

        assert!(!events.reminder, "a study that can't run sends no reminder");
        // Used up: the clock started over, so the next one is most of an
        // interval away and the event loop isn't woken for one already due
        let next = t.next_deadline(&s).expect("the interval is still counting");
        assert!(next > Instant::now() + Duration::from_secs(50), "{next:?}");
        assert!(!t.tick(&s, false).reminder, "and nothing is due on the next tick");
    }

    #[test]
    fn muting_the_reminders_leaves_the_auto_stop_alone() {
        let mut t = Timers::new();
        t.auto_stop_deadline = Some(Instant::now() - Duration::from_millis(1));
        t.last_reminder      = Some(Instant::now() - Duration::from_secs(61));

        let events = t.tick(&settings_with(5.0, 1.0, true), false);

        assert!(events.auto_stop, "the mute is for reminders only");
        assert!(!events.reminder);
    }

    #[test]
    fn tick_does_not_fire_reminder_when_interval_is_off() {
        let mut t = Timers::new();
        t.last_reminder = Some(Instant::now() - Duration::from_secs(3600));
        let s = settings_with(0.0, 0.0, true); // reminder = off
        let events = t.tick(&s, true);
        assert!(!events.reminder);
    }

    #[test]
    fn reschedule_auto_stop_non_finite_or_huge_minutes_does_not_panic() {
        let mut t = Timers::new();
        t.reschedule_auto_stop(&settings_with(f64::INFINITY, 0.0, true));
        assert!(t.auto_stop_deadline.is_none(), "inf minutes must not panic, treated as off");

        t.reschedule_auto_stop(&settings_with(f64::NAN, 0.0, true));
        assert!(t.auto_stop_deadline.is_none(), "NaN minutes must not panic, treated as off");

        // 1e18 minutes * 60 is finite but past `Duration::MAX` (~584 billion years)
        t.reschedule_auto_stop(&settings_with(1e18, 0.0, true));
        assert!(t.auto_stop_deadline.is_none(),
            "minutes past Duration::MAX must not panic, treated as off");
    }

    #[test]
    fn minutes_that_overflow_the_clock_do_not_panic() {
        // 2e17 minutes is 1.2e19 seconds: a valid `Duration`, but past
        // what `Instant` can add on macOS and Linux, where the sum used
        // to panic. Windows' `Instant` takes it, so what's asserted is
        // that nothing panics and that a timer that far out never fires
        let mut t = Timers::new();
        t.reschedule_auto_stop(&settings_with(2e17, 0.0, true));
        assert!(!t.tick(&settings_with(2e17, 0.0, true), true).auto_stop, "a deadline that far out never arrives");

        let s = settings_with(0.0, 2e17, true);
        t.reschedule_reminder(&s);
        let _ = t.next_deadline(&s);
        assert!(!t.tick(&s, true).reminder, "an interval that long never elapses");

        // Backdated so a sum that did fit would be long overdue
        t.last_reminder = Some(Instant::now() - Duration::from_secs(3600));
        let _ = t.next_deadline(&s);
        assert!(!t.tick(&s, true).reminder);
    }

    #[test]
    fn minutes_after_reads_a_length_the_clock_cannot_hold_as_none() {
        // `minutes_after` is the one place a minutes value meets the clock
        let now = Instant::now();
        assert!(minutes_after(now, 1.0).is_some_and(|t| t > now));
        assert!(minutes_after(now, 0.0).is_some_and(|t| t == now));
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 1e30, 3.1e17] {
            assert_eq!(minutes_after(now, bad), None, "{bad}");
        }
        // Whatever the platform's clock makes of a huge value, it's an
        // `Option`, never a panic
        let _ = minutes_after(now, 2e17);
        let _ = minutes_after(now, 1.54e17);
    }

    // ── Re-arming after a settings change ─────────────────────────

    /// Timers set to values a re-arm would replace: an auto-stop a second
    /// out (a re-arm puts it half an hour out) and a reminder clock ten
    /// seconds old (a re-arm puts it at now)
    fn backdated_timers() -> (Timers, Settings) {
        let s = settings_with(30.0, 15.0, true);
        let mut t = Timers::new();
        t.auto_stop_deadline = Some(Instant::now() + Duration::from_secs(1));
        t.last_reminder      = Some(Instant::now() - Duration::from_secs(10));
        (t, s)
    }

    #[test]
    fn a_change_that_touches_neither_timer_leaves_both_running() {
        // What Reset does in a study: the file locks both timers, so
        // nothing about them changes and nothing starts over
        let (mut t, s) = backdated_timers();
        let (stop, last) = (t.auto_stop_deadline, t.last_reminder);

        t.reschedule_for(&SettingsDiff::from(&s, &s), &s);

        assert_eq!(t.auto_stop_deadline, stop, "the auto-stop countdown keeps counting");
        assert_eq!(t.last_reminder, last, "the reminder clock keeps its start");
    }

    #[test]
    fn a_new_auto_stop_length_re_arms_the_auto_stop_and_nothing_else() {
        let (mut t, before) = backdated_timers();
        let (stop, last) = (t.auto_stop_deadline.expect("armed"), t.last_reminder);
        let mut after = before.clone();
        after.auto_stop_minutes = 5.0;

        t.reschedule_for(&SettingsDiff::from(&before, &after), &after);

        let re_armed = t.auto_stop_deadline.expect("still armed");
        assert!(re_armed > stop + Duration::from_secs(200), "five minutes out now");
        assert_eq!(t.last_reminder, last);
    }

    #[test]
    fn a_new_reminder_length_re_arms_the_reminder_and_nothing_else() {
        let (mut t, before) = backdated_timers();
        let (stop, last) = (t.auto_stop_deadline, t.last_reminder.expect("armed"));
        let mut after = before.clone();
        after.reminder_interval_minutes = 20.0;

        t.reschedule_for(&SettingsDiff::from(&before, &after), &after);

        assert_eq!(t.auto_stop_deadline, stop);
        assert!(t.last_reminder.expect("still armed") > last, "the clock starts over");
    }

    #[test]
    fn starting_and_stopping_re_arm_the_auto_stop() {
        let (mut t, running) = backdated_timers();
        let mut stopped = running.clone();
        stopped.is_animating = false;

        t.reschedule_for(&SettingsDiff::from(&running, &stopped), &stopped);
        assert!(t.auto_stop_deadline.is_none(), "a stopped animation has no countdown");

        t.reschedule_for(&SettingsDiff::from(&stopped, &running), &running);
        assert!(t.auto_stop_deadline.is_some(), "a started one counts from now");
    }

    #[test]
    fn tick_and_next_deadline_non_finite_or_huge_reminder_interval_does_not_panic() {
        let mut t = Timers::new();
        t.last_reminder = Some(Instant::now() - Duration::from_secs(3600));

        for bad_interval in [f64::INFINITY, f64::NAN, 1e18] {
            let s = settings_with(0.0, bad_interval, true);
            assert!(t.next_deadline(&s).is_none(),
                "bad interval {bad_interval} must not panic, contributes no deadline");
            let events = t.tick(&s, true);
            assert!(!events.reminder, "bad interval {bad_interval} must not panic or fire");
        }
    }
}
