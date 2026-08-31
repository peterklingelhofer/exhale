//! The sentences the settings window is allowed to say about pacing.
//!
//! This module exists so that every user-visible statement about the
//! research is derived, testable, and covered by CI. It lives in
//! `exhale-core` rather than next to the widget that paints it for one
//! blunt reason: CI runs `cargo test -p exhale-core` and does not
//! compile `exhale-app`, which needs wgpu, winit and GTK. Copy that
//! makes a coverage statement should not be the only text in the
//! project with no test behind it.
//!
//! **What the binary is permitted to assert.** Numbers it computed
//! itself, one range, and nothing else. No effect, no benefit, no
//! condition, no outcome measure. The corpus contains far stronger
//! warrants than anything here, and they stay out of the binary on
//! purpose: a store-reviewed app carries a retraction latency measured
//! in weeks, so a claim compiled into it cannot be withdrawn at the
//! speed the evidence can change. Arithmetic has no such problem,
//! because arithmetic cannot be retracted. See `docs/CITATIONS.md`.

use crate::settings::Settings;

/// Slowest rate with direct experimental support, in breaths per minute.
///
/// From `you2023-respiratory-frequency`, which tested 5, 5.5, 6, 6.5
/// and 7 cycles per minute. The endpoints are the endpoints of what was
/// actually run, not a recommendation and not a safe range: outside it
/// means untested here, which is different from tested and found
/// wanting. Gaps ledger item 2
pub const TESTED_MIN_BPM: f64 = 5.0;
/// Fastest rate with direct experimental support. See [`TESTED_MIN_BPM`]
pub const TESTED_MAX_BPM: f64 = 7.0;

/// Where a rate sits relative to the directly tested range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    Slower,
    Inside,
    Faster,
}

impl Coverage {
    /// Classify a rate that has ALREADY been rounded for display.
    ///
    /// Rounding first is deliberate. Classifying the raw value lets the
    /// panel print "5.0 breaths a minute" directly above "slower than
    /// any of them" for a true rate of 4.96, which reads as a bug and
    /// costs the reader their trust in the rest of the line
    pub fn of_displayed(bpm: f64) -> Self {
        if bpm < TESTED_MIN_BPM {
            Self::Slower
        } else if bpm > TESTED_MAX_BPM {
            Self::Faster
        } else {
            Self::Inside
        }
    }

    fn phrase(self) -> &'static str {
        match self {
            Self::Slower => "slower than any of them",
            Self::Inside => "one of them",
            Self::Faster => "faster than any of them",
        }
    }

    fn short_phrase(self) -> &'static str {
        match self {
            Self::Slower => "slower than all of them",
            Self::Inside => "inside it",
            Self::Faster => "faster than all of them",
        }
    }
}

/// One decimal place, which is the resolution the arithmetic deserves
/// and the resolution a person can act on
fn round_bpm(bpm: f64) -> f64 {
    (bpm * 10.0).round() / 10.0
}

/// Render a count of minutes at the coarsest unit that still says
/// something. The drift stepper spans four orders of magnitude, so a
/// single unit is either meaningless at one end or unreadable at the
/// other
fn humanize_minutes(minutes: f64) -> String {
    const HOUR: f64 = 60.0;
    const DAY:  f64 = 24.0 * HOUR;
    if minutes < 1.0 {
        "under a minute".to_string()
    } else if minutes < 90.0 {
        format!("{:.0} minutes", minutes)
    } else if minutes < 2.0 * DAY {
        format!("{:.1} hours", minutes / HOUR)
    } else {
        format!("{:.0} days", minutes / DAY)
    }
}

/// Minutes of continuous running before the cycle takes twice as long
/// as it does now, or `None` when drift is off or shortening.
///
/// Doubling is the honest unit for a compounding setting. "0.1 % per
/// cycle" tells nobody anything; "the breath is twice as long after
/// four hours" tells them whether the number they just typed is gentle
/// or runaway, which is the only question the stepper actually raises
pub fn minutes_to_double(settings: &Settings) -> Option<f64> {
    let cycle = settings.cycle_secs();
    if cycle <= 0.0 || settings.drift <= 1.0 {
        return None;
    }
    // From `Settings::breaths_per_min_after`: the projected cycle is
    // `c + 60·T·(d − 1)`, so it reaches `2c` at `T = c / (60·(d − 1))`
    Some(cycle / (60.0 * (settings.drift - 1.0)))
}

/// The horizon the drift line projects to. An hour is long enough that
/// a gentle drift has visibly moved and short enough to be a session
/// somebody might actually sit through
const PROJECTION_MINUTES: f64 = 60.0;

/// Build the readout, one string per line, in display order.
///
/// Returns empty when no phase has a duration, which is not a rate of
/// anything and not a state worth narrating
pub fn readout_lines(settings: &Settings) -> Vec<String> {
    let Some(bpm) = settings.breaths_per_min() else {
        return Vec::new();
    };
    let now = round_bpm(bpm);
    let mut lines = vec![format!(
        "Now: {now:.1} breaths a minute, a {} cycle.",
        format_secs(settings.cycle_secs()),
    )];

    let projected = settings
        .drift_is_active()
        .then(|| settings.breaths_per_min_after(PROJECTION_MINUTES))
        .flatten()
        .map(round_bpm);

    if let Some(later) = projected {
        let mut drift_line = format!("Drift: about {later:.1} a minute after an hour");
        if let Some(double_at) = minutes_to_double(settings) {
            drift_line.push_str(&format!(
                ", and twice this cycle length after {}",
                humanize_minutes(double_at),
            ));
        }
        drift_line.push('.');
        lines.push(drift_line);
    }

    lines.push(coverage_line(now, projected));
    lines
}

/// The one range the binary is allowed to name, and where the current
/// setting falls against it.
///
/// Phrased about the literature throughout. It reports what was
/// measured and whether this number was among it; it does not tell the
/// user their breathing is wrong, because the corpus does not support
/// that and the app is in no position to say so
fn coverage_line(now: f64, projected: Option<f64>) -> String {
    let base = format!(
        "Rates from {TESTED_MIN_BPM:.0} to {TESTED_MAX_BPM:.0} a minute are the ones tested directly. "
    );
    let start = Coverage::of_displayed(now);
    match projected.map(Coverage::of_displayed) {
        // Drift that carries the pace out of the tested range within
        // the projection window is exactly the thing a static label
        // would hide, so it gets said rather than implied
        Some(end) if end != start => format!(
            "{base}This one starts {} and is {} within an hour.",
            match start {
                Coverage::Inside => "inside it",
                Coverage::Slower => "below it",
                Coverage::Faster => "above it",
            },
            end.short_phrase(),
        ),
        _ => format!("{base}This one is {}.", start.phrase()),
    }
}

/// Whole seconds when the cycle is whole, one decimal when it is not.
/// Timing steppers move in whole seconds, so "15 s" is the normal case
/// and "15.5 s" only shows up for a typed value
fn format_secs(secs: f64) -> String {
    if (secs - secs.round()).abs() < 1e-9 {
        format!("{secs:.0} s")
    } else {
        format!("{secs:.1} s")
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_default_volunteers_that_it_is_untested() {
        // The point of the readout. exhale's own default is 4.0 a
        // minute, below the band, and the panel says so on first open
        // without being asked. Gaps ledger item 2
        let lines = readout_lines(&Settings::default());
        assert_eq!(lines.len(), 2, "{lines:#?}");
        assert_eq!(lines[0], "Now: 4.0 breaths a minute, a 15 s cycle.");
        assert!(lines[1].ends_with("This one is slower than any of them."), "{}", lines[1]);
    }

    #[test]
    fn box_breathing_is_reported_as_slower_than_it_looks() {
        // 4/4/4/4 is 3.75 a minute. The holds hide that from anyone
        // doing the arithmetic in their head. Gaps ledger item 3
        let mut s = Settings::default();
        s.inhale_duration           = 4.0;
        s.post_inhale_hold_duration = 4.0;
        s.exhale_duration           = 4.0;
        s.post_exhale_hold_duration = 4.0;
        let lines = readout_lines(&s);
        assert_eq!(lines[0], "Now: 3.8 breaths a minute, a 16 s cycle.");
        assert!(lines[1].contains("slower than any of them"));
    }

    #[test]
    fn an_in_band_pace_is_named_as_one_of_the_tested_rates() {
        let mut s = Settings::default();
        s.exhale_duration = 5.0; // 5/0/5/0 = 6 a minute
        let lines = readout_lines(&s);
        assert_eq!(lines[0], "Now: 6.0 breaths a minute, a 10 s cycle.");
        assert!(lines[1].ends_with("This one is one of them."), "{}", lines[1]);
    }

    #[test]
    fn drift_that_leaves_the_band_says_so_rather_than_reporting_the_start() {
        // Starts at 6 a minute, inside the range, and 1 % drift takes
        // it out within the hour. A line computed only from the
        // starting pace would read "one of them" and be misleading
        let mut s = Settings::default();
        s.exhale_duration = 5.0;
        s.drift           = 1.01;
        let lines = readout_lines(&s);
        assert_eq!(lines.len(), 3, "{lines:#?}");
        assert!(lines[1].starts_with("Drift: about "), "{}", lines[1]);
        assert!(
            lines[2].ends_with("This one starts inside it and is slower than all of them within an hour."),
            "{}", lines[2]
        );
    }

    #[test]
    fn drift_off_produces_no_drift_line() {
        let lines = readout_lines(&Settings::default());
        assert!(!lines.iter().any(|l| l.contains("Drift")), "{lines:#?}");
    }

    #[test]
    fn the_doubling_time_separates_gentle_drift_from_runaway_drift() {
        // The contrast gaps ledger item 6 turns on, and the reason the
        // stepper moves in tenths of a percentage point rather than
        // whole ones
        let mut gentle = Settings::default();
        gentle.drift = 1.001;
        assert_eq!(humanize_minutes(minutes_to_double(&gentle).unwrap()), "4.2 hours");

        let mut steep = Settings::default();
        steep.drift = 1.01;
        assert_eq!(humanize_minutes(minutes_to_double(&steep).unwrap()), "25 minutes");

        let mut barely = Settings::default();
        barely.drift = 1.00001;
        assert_eq!(humanize_minutes(minutes_to_double(&barely).unwrap()), "17 days");

        assert_eq!(minutes_to_double(&Settings::default()), None);
    }

    #[test]
    fn the_rate_shown_and_the_range_named_never_contradict_each_other() {
        // A true rate of 4.96 displays as 5.0. Classifying the raw
        // value would print "5.0 breaths a minute" above "slower than
        // any of them"
        let mut s = Settings::default();
        s.inhale_duration = 5.0;
        s.exhale_duration = 7.096_774; // ≈ 4.96 a minute
        let lines = readout_lines(&s);
        assert!(lines[0].contains("5.0 breaths a minute"), "{}", lines[0]);
        assert!(lines[1].ends_with("This one is one of them."), "{}", lines[1]);
    }

    #[test]
    fn a_zero_length_cycle_says_nothing_at_all() {
        let mut s = Settings::default();
        s.inhale_duration           = 0.0;
        s.post_inhale_hold_duration = 0.0;
        s.exhale_duration           = 0.0;
        s.post_exhale_hold_duration = 0.0;
        assert!(readout_lines(&s).is_empty());
    }

    #[test]
    fn no_line_names_an_effect_a_benefit_or_a_condition() {
        // A denylist, not a style rule. `scripts/generate-citations.py`
        // keeps retracted phrasing out of the store listings; this
        // keeps outcome vocabulary out of the binary, which is the
        // constraint that rules out quoting `custom.backsClaims`
        // verbatim however well-sourced those strings are.
        // `docs/CITATIONS.md` may say all of this; the app may not
        const BANNED: &[&str] = &[
            "anxiety", "depress", "stress", "calm", "relax", "vagal", "parasympathetic",
            "blood pressure", "heart rate variability", "hrv", "mood", "sleep",
            "benefit", "improve", "treat", "therapy", "symptom", "health",
            "recommend", "should", "optimal", "best", "correct",
        ];
        let mut cases = vec![Settings::default()];
        for drift in [1.001, 1.01] {
            for exhale in [5.0, 10.0, 1.0] {
                let mut s = Settings::default();
                s.drift = drift;
                s.exhale_duration = exhale;
                cases.push(s);
            }
        }
        for s in cases {
            for line in readout_lines(&s) {
                let lowered = line.to_lowercase();
                for word in BANNED {
                    assert!(
                        !lowered.contains(word),
                        "readout line asserts {word:?}, which the binary may not say: {line:?}"
                    );
                }
            }
        }
    }
}
