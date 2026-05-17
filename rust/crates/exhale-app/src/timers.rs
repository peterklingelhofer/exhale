use std::time::{Duration, Instant};

use exhale_core::settings::Settings;
use log::info;

/// All timer state, checked each event-loop tick in `about_to_wait`.
pub struct Timers {
    pub auto_stop_deadline: Option<Instant>,
    pub last_reminder:      Option<Instant>,
}

impl Timers {
    pub fn new() -> Self {
        Self { auto_stop_deadline: None, last_reminder: None }
    }

    /// Call whenever `is_animating` or `auto_stop_minutes` changes.
    pub fn reschedule_auto_stop(&mut self, settings: &Settings) {
        self.auto_stop_deadline = if settings.auto_stop_minutes > 0.0 && settings.is_animating {
            let secs = settings.auto_stop_minutes * 60.0;
            Some(Instant::now() + Duration::from_secs_f64(secs))
        } else {
            None
        };
    }

    /// Call whenever `reminder_interval_minutes` changes.
    pub fn reschedule_reminder(&mut self, settings: &Settings) {
        self.last_reminder = if settings.reminder_interval_minutes > 0.0 {
            Some(Instant::now())
        } else {
            None
        };
    }

    /// Earliest time the event loop must wake to service a pending timer.
    /// Returned to `about_to_wait` so it can configure `ControlFlow::WaitUntil`
    /// — without this the loop would sleep forever on idle and miss auto-stop
    /// / reminder firings now that the old per-tick redraw loop no longer
    /// wakes it every frame.
    pub fn next_deadline(&self, settings: &Settings) -> Option<Instant> {
        let mut next: Option<Instant> = self.auto_stop_deadline;
        if settings.reminder_interval_minutes > 0.0 {
            if let Some(last) = self.last_reminder {
                let due = last + Duration::from_secs_f64(
                    settings.reminder_interval_minutes * 60.0,
                );
                next = Some(match next {
                    Some(n) => n.min(due),
                    None    => due,
                });
            }
        }
        next
    }

    /// Returns `true` if the animation should be stopped (auto-stop deadline hit).
    /// Returns `true` if a reminder notification should fire.
    /// Caller is responsible for applying the stop and sending the notification.
    pub fn tick(&mut self, settings: &Settings) -> TimerEvents {
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
            let interval = Duration::from_secs_f64(settings.reminder_interval_minutes * 60.0);
            let due = self.last_reminder
                .map(|t| now >= t + interval)
                .unwrap_or(false);
            if due {
                self.last_reminder = Some(now);
                events.reminder = true;
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

/// Cross-platform desktop notification ("Remember to breathe").
///
/// macOS uses the native `UNUserNotifications` framework (required for Mac
/// App Store distribution — `NSUserNotification` is deprecated and
/// `notify-rust`'s macOS backend relies on it).  Other platforms continue
/// to use `notify-rust` (D-Bus on Linux, WinRT toasts on Windows).
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

/// Deliver a local notification via `UNUserNotificationCenter`.
///
/// Mirrors the Swift AppDelegate's `sendReminderNotification()`: builds a
/// `UNMutableNotificationContent` with title, body, and default sound, wraps
/// it in a `UNNotificationRequest` with a fresh UUID identifier and a `nil`
/// trigger (deliver immediately), then hands it to the shared center.
///
/// Requires the bundle to be code-signed and to have been granted
/// `.alert | .sound` authorization (see `platform::request_notification_permission`).
/// In an unsigned `cargo run` build the center silently drops the request
/// — that's fine for development.
#[cfg(target_os = "macos")]
fn send_reminder_macos() {
    use block2::StackBlock;
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    // SAFETY: framework class lookups are fallible — skip silently
    // when `UserNotifications.framework` isn't linked (e.g. in a
    // bare `cargo test` binary).  Production app bundle has it.
    let (Some(unc_cls), Some(content_cls), Some(sound_cls), Some(req_cls)) = (
        AnyClass::get(c"UNUserNotificationCenter"),
        AnyClass::get(c"UNMutableNotificationContent"),
        AnyClass::get(c"UNNotificationSound"),
        AnyClass::get(c"UNNotificationRequest"),
    ) else { return; };

    unsafe {
        let content: *mut AnyObject = msg_send![content_cls, alloc];
        let content: *mut AnyObject = msg_send![content, init];
        if content.is_null() { return; }

        let ns_string = objc2::class!(NSString);
        // C-string literals via `c"…"` are guaranteed nul-terminated
        // at compile time — no runtime `CString::new(...).unwrap()`,
        // no allocation per reminder fire.
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
        }

        // Balance the +1 retain from `[UNMutableNotificationContent alloc] init]`.
        // `requestWithIdentifier:…` retains `content` internally, and the
        // request / sound / strings / uuid are autoreleased convenience
        // returns that we don't own.
        let _: () = msg_send![content, release];
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// Cover the `Timers` state machine that the audit flagged as having
// zero coverage despite being a real bug-source: auto-stop firing,
// reminder firing, deadline computation, edge cases around 0-value
// (off) settings
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
        // 2 minutes = 120s; allow ±100ms for clock noise.
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
        // Reminder fires in ~1 min, auto-stop in ~10 min → next should be ~1 min.
        let dt = d.duration_since(now);
        assert!(dt < Duration::from_secs(90),
            "earliest should be the 1-minute reminder, got {dt:?}");
    }

    #[test]
    fn tick_fires_auto_stop_when_deadline_passes() {
        let mut t = Timers::new();
        // Use a tiny duration we can wait past.  We can't sleep in
        // tests reliably, so backdate the deadline directly.
        t.auto_stop_deadline = Some(Instant::now() - Duration::from_millis(1));
        let events = t.tick(&settings_with(5.0, 0.0, true));
        assert!(events.auto_stop, "auto-stop event must fire when deadline passed");
        assert!(t.auto_stop_deadline.is_none(), "deadline cleared after firing");
    }

    #[test]
    fn tick_does_not_fire_auto_stop_early() {
        let mut t = Timers::new();
        t.auto_stop_deadline = Some(Instant::now() + Duration::from_secs(60));
        let events = t.tick(&settings_with(5.0, 0.0, true));
        assert!(!events.auto_stop);
        assert!(t.auto_stop_deadline.is_some());
    }

    #[test]
    fn tick_fires_reminder_at_interval_and_resets_clock() {
        let mut t = Timers::new();
        // Backdate so the interval has just passed.
        t.last_reminder = Some(Instant::now() - Duration::from_secs(61));
        let s = settings_with(0.0, 1.0, true);
        let events = t.tick(&s);
        assert!(events.reminder, "reminder fires after 1-min interval elapsed");
        let new_last = t.last_reminder.expect("reset");
        assert!(new_last > Instant::now() - Duration::from_secs(1),
            "last_reminder reset to ~now after firing");
    }

    #[test]
    fn tick_does_not_fire_reminder_when_interval_is_off() {
        let mut t = Timers::new();
        t.last_reminder = Some(Instant::now() - Duration::from_secs(3600));
        let s = settings_with(0.0, 0.0, true); // reminder = off
        let events = t.tick(&s);
        assert!(!events.reminder);
    }
}
