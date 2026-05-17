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
        let title_c = std::ffi::CString::new("exhale").unwrap();
        let body_c  = std::ffi::CString::new("Remember to breathe").unwrap();
        let title:  *mut AnyObject = msg_send![ns_string, stringWithUTF8String: title_c.as_ptr()];
        let body:   *mut AnyObject = msg_send![ns_string, stringWithUTF8String: body_c.as_ptr()];
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
