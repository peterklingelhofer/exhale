//! The breathing patterns the settings window offers as one click.
//!
//! Five entries, hand-written, in `exhale-core` rather than in the UI
//! crate for the same reason [`crate::pacing`] is: CI compiles and
//! tests this crate and does not compile `exhale-app`.
//!
//! **A preset sets the four durations and nothing else.** Not drift,
//! not the randomisation sliders. Clicking one can therefore never
//! discard a value the user tuned by hand, which is the property that
//! lets selection be *derived* by comparing four numbers instead of
//! stored in a sixth settings field that could fall out of sync with
//! the five it summarises. Drift and jitter keep whatever the user set,
//! and the pacing readout underneath already reports what drift does
//! to the rate, so nothing about the resulting pace goes unsaid.
//!
//! **Labels name the pattern, never the rate.** A chip reading "6 a
//! minute" would be a claim about which rate is worth choosing; a chip
//! reading "5/0/5/0" is a description of what the four steppers below
//! it are about to say, in their own order: inhale, post-inhale hold,
//! exhale, post-exhale hold. It is the notation the README already
//! uses, and it is terse enough that all five chips fit a single row
//! of a 308 pt card, where the spelled-out form took three. The rate,
//! and how it sits
//! against the range anyone has tested, is computed live by
//! [`crate::pacing::readout_lines`] for whichever pattern is active.
//! That is why no preset carries its own evidentiary caption: the
//! panel already volunteers "slower than any of them" the instant box
//! breathing is selected, for every pattern rather than only the ones
//! someone remembered to annotate.
//!
//! `citekey` is provenance, not display. It never reaches the screen.
//! It exists so `scripts/generate-citations.py` can refuse to build
//! when a shipped preset points at a record that has been retracted,
//! downgraded to tier E, or marked as one the binary may not lean on.

use crate::settings::Settings;

/// One offered pattern. Four durations, a label describing them, and a
/// corpus record that has to still hold up for the preset to ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    /// Stable identifier. Not shown; used by tests and by any future
    /// telemetry-free bookkeeping. Never renamed casually, because a
    /// rename is invisible in the UI and silent in the diff
    pub id: &'static str,
    /// What the chip says. Describes the pattern, never the rate
    pub label: &'static str,
    /// A name or a fact about identity, never about effect. `None`
    /// when the pattern has nothing to add that the readout below does
    /// not already say better
    pub note: Option<&'static str>,
    pub inhale: f64,
    pub post_inhale_hold: f64,
    pub exhale: f64,
    pub post_exhale_hold: f64,
    /// Corpus entry backing this pattern's presence in the list. See
    /// the module comment: provenance, not display
    pub citekey: &'static str,
}

impl Preset {
    /// True when `settings` currently holds exactly this pattern.
    ///
    /// The epsilon is the one `SettingsDiff::from` uses on these same
    /// four fields. Sharing it is deliberate: "this chip looks
    /// selected" and "changing this field marks settings dirty" must
    /// agree, or a chip can appear selected while a save is pending
    /// that will unselect it
    pub fn matches(&self, settings: &Settings) -> bool {
        const EPS: f64 = 1e-9;
        (settings.inhale_duration - self.inhale).abs() < EPS
            && (settings.post_inhale_hold_duration - self.post_inhale_hold).abs() < EPS
            && (settings.exhale_duration - self.exhale).abs() < EPS
            && (settings.post_exhale_hold_duration - self.post_exhale_hold).abs() < EPS
    }

    /// Write this pattern's four durations into `settings`, leaving
    /// every other field alone. See the module comment for why the
    /// omission is the point
    pub fn apply(&self, settings: &mut Settings) {
        settings.inhale_duration = self.inhale;
        settings.post_inhale_hold_duration = self.post_inhale_hold;
        settings.exhale_duration = self.exhale;
        settings.post_exhale_hold_duration = self.post_exhale_hold;
    }
}

/// The offered set, in display order.
///
/// Ordered gentlest-first by the standard the corpus actually
/// supports, which is the tested range rather than apparent
/// simplicity. The two patterns that fall outside it come last and are
/// still offered: `4 / 4 / 4 / 4` because people arrive looking for it
/// by name, and `5 s in, 10 s out` because it is what exhale has
/// shipped for years and removing it from the list would not remove it
/// from anybody's installed configuration. A list that quietly omitted
/// its own default would be the least honest version of this feature
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "even-5",
        label: "5/0/5/0",
        note: Some("· exhale's default."),
        inhale: 5.0,
        post_inhale_hold: 0.0,
        exhale: 5.0,
        post_exhale_hold: 0.0,
        citekey: "you2023-respiratory-frequency",
    },
    Preset {
        id: "long-exhale-4-6",
        label: "4/0/6/0",
        note: None,
        inhale: 4.0,
        post_inhale_hold: 0.0,
        exhale: 6.0,
        post_exhale_hold: 0.0,
        citekey: "vandiest2014-ie-ratio-relaxation",
    },
    Preset {
        id: "a52",
        label: "5/0/5/2",
        // The name is the reason anyone would look for this pattern,
        // and naming it costs nothing. The review that popularised the
        // name is blocklisted from the binary, so the citekey points
        // at the study that tested this rate instead
        note: Some("· sometimes called A52."),
        inhale: 5.0,
        post_inhale_hold: 0.0,
        exhale: 5.0,
        post_exhale_hold: 2.0,
        citekey: "you2023-respiratory-frequency",
    },
    Preset {
        id: "box",
        label: "4/4/4/4",
        note: Some("· sometimes called box breathing."),
        inhale: 4.0,
        post_inhale_hold: 4.0,
        exhale: 4.0,
        post_exhale_hold: 4.0,
        citekey: "marchant2025-square-478-six",
    },
    Preset {
        // Kept in the list because it was the default before 5/0/5/0 and
        // is still what every existing settings.toml holds. Removing it
        // from the offered set would not remove it from anyone's machine,
        // it would just make their own configuration unnameable
        id: "long-exhale-5-10",
        label: "5/0/10/0",
        note: None,
        inhale: 5.0,
        post_inhale_hold: 0.0,
        exhale: 10.0,
        post_exhale_hold: 0.0,
        citekey: "vandiest2014-ie-ratio-relaxation",
    },
];

/// Index of the preset the current settings match, if any.
///
/// `None` is a first-class answer and renders as no chip selected.
/// There is deliberately no "Custom" chip to fall back on: a chip that
/// cannot be clicked to any effect is still a Tab stop, and this
/// window already documents that hazard twice
pub fn selected(settings: &Settings) -> Option<usize> {
    PRESETS.iter().position(|p| p.matches(settings))
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::pacing;

    #[test]
    fn the_shipped_default_is_the_first_offered_pattern() {
        // If the default matched no preset, the panel would show no
        // selection out of the box, which reads as "your configuration
        // is unrecognised" on first ever launch
        let s = Settings::default();
        let i = selected(&s).expect("default settings match no preset");
        assert_eq!(PRESETS[i].id, "even-5");
        assert_eq!(i, 0, "the default should be the first chip, not buried mid-row");
    }

    #[test]
    fn the_previous_default_is_still_offered() {
        // 5/0/10/0 shipped as the default for years, so it is what every
        // existing settings.toml holds. It has to stay nameable
        let p = PRESETS.iter().find(|p| p.id == "long-exhale-5-10").unwrap();
        let mut s = Settings::default();
        p.apply(&mut s);
        assert!((s.breaths_per_min().unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn every_preset_round_trips_through_apply_and_matches() {
        for p in PRESETS {
            let mut s = Settings::default();
            p.apply(&mut s);
            assert!(p.matches(&s), "{} does not match itself after apply", p.id);
            assert_eq!(selected(&s), Some(PRESETS.iter().position(|q| q.id == p.id).unwrap()));
        }
    }

    #[test]
    fn no_two_presets_share_a_pattern_or_an_id() {
        // Two chips matching the same four numbers would both light up
        // and `selected` would silently pick the first
        for (i, a) in PRESETS.iter().enumerate() {
            for b in &PRESETS[i + 1..] {
                assert_ne!(a.id, b.id);
                assert_ne!(a.label, b.label);
                let mut s = Settings::default();
                a.apply(&mut s);
                assert!(!b.matches(&s), "{} and {} are the same pattern", a.id, b.id);
            }
        }
    }

    #[test]
    fn applying_a_preset_leaves_drift_and_jitter_alone() {
        // The property that makes derived selection safe. A preset
        // that also zeroed these would discard tuning the user did on
        // purpose, and the chip would then be describing a pattern the
        // app had just silently changed out from under them
        let mut s = Settings::default();
        s.drift = 1.001;
        s.randomized_timing_inhale = 0.2;
        s.randomized_timing_post_inhale_hold = 0.1;
        s.randomized_timing_exhale = 0.3;
        s.randomized_timing_post_exhale_hold = 0.05;
        let before = s.clone();

        PRESETS[0].apply(&mut s);

        assert_eq!(s.drift, before.drift);
        assert_eq!(s.randomized_timing_inhale, before.randomized_timing_inhale);
        assert_eq!(s.randomized_timing_post_inhale_hold, before.randomized_timing_post_inhale_hold);
        assert_eq!(s.randomized_timing_exhale, before.randomized_timing_exhale);
        assert_eq!(s.randomized_timing_post_exhale_hold, before.randomized_timing_post_exhale_hold);
        assert!(selected(&s).is_some(), "drift and jitter must not affect which chip is selected");
    }

    #[test]
    fn a_hand_tuned_pattern_selects_nothing() {
        let mut s = Settings::default();
        s.exhale_duration = 7.0;
        assert_eq!(selected(&s), None);
    }

    #[test]
    fn no_label_or_note_states_a_rate_an_effect_or_a_recommendation() {
        // Same denylist discipline as `pacing`, for the same reason.
        // "a minute", "bpm" and "breaths per" are here because a rate
        // baked into a label is a claim about which rate to choose,
        // and because it would go stale against the live readout the
        // moment either changed
        const BANNED: &[&str] = &[
            "a minute", "per minute", "bpm", "breaths per",
            "anxiety", "stress", "calm", "relax", "vagal", "parasympathetic",
            "blood pressure", "hrv", "mood", "sleep",
            "benefit", "improve", "treat", "therapy", "symptom", "health",
            "recommend", "should", "optimal", "best", "beginner", "advanced",
            "tested", "evidence", "study", "studies", "research", "proven",
        ];
        for p in PRESETS {
            for text in [Some(p.label), p.note].into_iter().flatten() {
                let lowered = text.to_lowercase();
                for word in BANNED {
                    assert!(
                        !lowered.contains(word),
                        "{}: {text:?} states {word:?}, which a chip may not say",
                        p.id
                    );
                }
            }
        }
    }

    #[test]
    fn every_preset_names_a_corpus_entry() {
        // The Rust half of the gate. The other half lives in
        // `scripts/generate-citations.py`, which resolves these
        // against the corpus and rejects a retracted, tier-E or
        // non-in-app-citable record. This side only proves the field
        // is populated and plausibly shaped, so a typo'd empty string
        // fails here rather than being silently skipped there
        for p in PRESETS {
            assert!(!p.citekey.is_empty(), "{} has no citekey", p.id);
            assert!(
                p.citekey.contains('-') && p.citekey.chars().any(|c| c.is_ascii_digit()),
                "{}: {:?} is not shaped like a citekey", p.id, p.citekey
            );
        }
    }

    #[test]
    fn the_two_out_of_band_presets_are_the_ones_that_announce_it() {
        // The list ships two patterns outside the range with direct
        // support, and neither hides it: selecting either makes the
        // readout say so. This test is the link between the two
        // modules, so deleting the coverage line from `pacing` fails
        // here rather than quietly making the chip list a
        // recommendation
        for (id, expect_in_band) in [
            ("even-5", true),
            ("long-exhale-4-6", true),
            ("a52", true),
            ("box", false),
            ("long-exhale-5-10", false),
        ] {
            let p = PRESETS.iter().find(|p| p.id == id).unwrap();
            let mut s = Settings::default();
            p.apply(&mut s);
            let coverage = pacing::Coverage::of_displayed(
                (s.breaths_per_min().unwrap() * 10.0).round() / 10.0,
            );
            let in_band = coverage == pacing::Coverage::Inside;
            assert_eq!(in_band, expect_in_band, "{id} coverage was {coverage:?}");

            let lines = pacing::readout_lines(&s);
            if !in_band {
                assert!(
                    lines.last().unwrap().contains("slower than any of them"),
                    "{id}: readout does not disclose it is outside the range: {lines:#?}"
                );
            }
        }
    }
}
