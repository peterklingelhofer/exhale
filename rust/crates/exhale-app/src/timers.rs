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
pub fn send_reminder() {
    info!("reminder: Remember to breathe");
    let mut n = notify_rust::Notification::new();
    n.summary("exhale").body("Remember to breathe");
    #[cfg(target_os = "macos")]
    n.sound_name("default");
    if let Err(e) = n.show() {
        log::warn!("notification: {e}");
    }
}
