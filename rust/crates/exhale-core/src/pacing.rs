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

/// Group a count with thousands separators. Drift spans four orders of
/// magnitude, so this routinely renders numbers like 69,315 where an
/// ungrouped 69315 is a smear
fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Breaths of continuous running before one cycle takes twice as long
/// as it does now, or `None` when drift is off or shortening.
///
/// **Counted in breaths, not minutes, and that is the whole point.**
/// Cycle `k` lasts `c · dᵏ`, so `dᵏ = 2` at `k = ln2 / ln d`: the
/// starting cycle length cancels out entirely. One per cent doubles the
/// breath in 70 breaths whether the user started at 10 s or at 15 s.
///
/// Quoting a doubling *time* instead made the panel look broken. The
/// same 1 % setting reads as 17 minutes from a 10 s cycle and 25
/// minutes from a 15 s one, which invites the reader to conclude the
/// arithmetic is unreliable when in fact both are correct and the
/// question was ambiguous. A count of breaths is a property of the
/// drift value alone, so it is stable, and it is also the thing the
/// user is about to sit through.
///
/// It is a true repeat rate, not just a first milestone: the same `k`
/// takes the cycle from `2c` to `4c`
pub fn breaths_to_double(settings: &Settings) -> Option<f64> {
    if settings.cycle_secs() <= 0.0 || settings.drift <= 1.0 {
        return None;
    }
    Some(std::f64::consts::LN_2 / settings.drift.ln())
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

    if projected.is_some() {
        // Seconds, not breaths per minute. "About 1.3 a minute after an
        // hour" is arithmetically correct and unreadable: nobody holds a
        // mental picture of 1.3 breaths a minute, whereas everybody can
        // picture a breath that has gone from 10 seconds to 46. Seconds
        // are also the unit the four steppers directly above are set in,
        // so the sentence lands in the units the user just typed
        let mut drift_line = String::from("Drift: ");
        match breaths_to_double(settings) {
            Some(n) => drift_line.push_str(&format!(
                "the cycle doubles every {} breaths.",
                grouped(n.round().max(1.0) as u64),
            )),
            // Reachable only from a hand-edited settings file, where
            // drift below 1.0 shortens the breath instead
            None => drift_line.push_str("the cycle shortens every breath."),
        }

        let now  = settings.cycle_secs();
        let then = 60.0 / settings.breaths_per_min_after(PROJECTION_MINUTES).unwrap_or(f64::MAX);
        // Suppress the second clause when an hour does not move the
        // number a person could read off the screen. At 0.001 % a 15 s
        // cycle reaches 15.04 s, and "after an hour it is 15 s, not
        // 15 s" reads as a bug rather than as a very gentle setting
        if (then - now).abs() >= 0.5 {
            drift_line.push_str(&format!(
                " After an hour it is {}, not {}.",
                format_secs(then),
                format_secs(now),
            ));
        }
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
        assert_eq!(
            lines[1],
            "Drift: the cycle doubles every 70 breaths. After an hour it is 46 s, not 10 s."
        );
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
    fn the_doubling_count_separates_gentle_drift_from_runaway_drift() {
        // The contrast gaps ledger item 6 turns on, and the reason the
        // stepper moves in tenths of a percentage point rather than
        // whole ones
        let mut gentle = Settings::default();
        gentle.drift = 1.001;
        assert_eq!(breaths_to_double(&gentle).unwrap().round(), 693.0);

        let mut steep = Settings::default();
        steep.drift = 1.01;
        assert_eq!(breaths_to_double(&steep).unwrap().round(), 70.0);

        let mut barely = Settings::default();
        barely.drift = 1.00001;
        assert_eq!(breaths_to_double(&barely).unwrap().round(), 69_315.0);

        assert_eq!(breaths_to_double(&Settings::default()), None);
    }

    #[test]
    fn the_doubling_count_does_not_depend_on_the_starting_cycle() {
        // The bug this replaced. Quoted as a doubling *time*, one per
        // cent read as 17 minutes from a 10 s cycle and 25 minutes from
        // a 15 s one, and the panel looked like it could not do
        // arithmetic. Both were right; the unit was wrong
        let mut ten = Settings::default();
        ten.drift = 1.01;
        ten.exhale_duration = 5.0;
        assert_eq!(ten.cycle_secs(), 10.0);

        let mut fifteen = Settings::default();
        fifteen.drift = 1.01;
        assert_eq!(fifteen.cycle_secs(), 15.0);

        assert_eq!(breaths_to_double(&ten), breaths_to_double(&fifteen));
    }

    #[test]
    fn doubling_is_a_repeat_rate_not_a_one_off_milestone() {
        // "Every N breaths" claims the interval repeats. It does: the
        // same count takes the cycle from 2c to 4c, because `dᵏ = 2`
        // has no `c` in it
        let mut s = Settings::default();
        s.drift = 1.01;
        let k = breaths_to_double(&s).unwrap();
        let c = s.cycle_secs();
        for doublings in 1..=4 {
            let cycle = c * s.drift.powf(k * doublings as f64);
            let want  = c * 2f64.powi(doublings);
            assert!((cycle - want).abs() < 1e-6, "after {doublings} doublings: {cycle} vs {want}");
        }
    }

    #[test]
    fn thousands_are_grouped() {
        assert_eq!(grouped(7), "7");
        assert_eq!(grouped(70), "70");
        assert_eq!(grouped(693), "693");
        assert_eq!(grouped(69_315), "69,315");
        assert_eq!(grouped(6_931_472), "6,931,472");
    }

    #[test]
    fn a_drift_too_slow_to_see_within_an_hour_says_only_what_it_can() {
        // 0.001 % moves a 15 s cycle to 15.04 s in an hour. "After an
        // hour it is 15 s, not 15 s" would read as a bug
        let mut s = Settings::default();
        s.drift = 1.00001;
        let lines = readout_lines(&s);
        assert_eq!(lines[1], "Drift: the cycle doubles every 69,315 breaths.");
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
