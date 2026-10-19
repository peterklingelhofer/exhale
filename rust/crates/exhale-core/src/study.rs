//! The study file: a lab's pacing lock, read from disk beside `settings.toml`
//!
//! A `study.toml` next to the settings file pins the eleven numeric
//! pacing fields and marks the running app as a study build. The file is
//! the only source of the lock, so deleting it lifts the lock on the
//! next launch and no uninstall is needed
//!
//! Nothing here interprets the `condition` code. It's an opaque string
//! the lab assigns, carried into the session log so a log can be matched
//! back to an arm without the app knowing which arm it's running

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{controller::MAX_PHASE_SECS, settings::Settings};

/// Name of the study file, read from the directory that holds `settings.toml`
pub const STUDY_FILE_NAME: &str = "study.toml";

fn default_drift() -> f64 { 1.0 }

/// One lab's assigned configuration for one participant
///
/// Required in the file: `participant_id`, `condition` and the four
/// durations. Everything else defaults. `deny_unknown_fields` makes a
/// misspelled optional field fail at load, because it would otherwise
/// be a silently wrong pace for a whole session
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyConfig {
    pub participant_id: String,
    /// Opaque code the lab assigns. The app never interprets it
    pub condition:      String,

    pub inhale_duration:           f64,
    pub post_inhale_hold_duration: f64,
    pub exhale_duration:           f64,
    pub post_exhale_hold_duration: f64,

    /// Per-cycle duration multiplier, 1.0 is off. See [`Settings::drift`]
    #[serde(default = "default_drift")]
    pub drift: f64,

    #[serde(default)]
    pub randomized_timing_inhale:           f64,
    #[serde(default)]
    pub randomized_timing_post_inhale_hold: f64,
    #[serde(default)]
    pub randomized_timing_exhale:           f64,
    #[serde(default)]
    pub randomized_timing_post_exhale_hold: f64,

    /// Minutes after which the animation stops on its own, 0 is off
    #[serde(default)]
    pub auto_stop_minutes:         f64,
    /// Minutes between reminder notifications, 0 is off
    #[serde(default)]
    pub reminder_interval_minutes: f64,
}

/// Where the study file lives for a given settings file
pub fn study_file_path(settings_path: &Path) -> PathBuf {
    settings_path.with_file_name(STUDY_FILE_NAME)
}

impl StudyConfig {
    /// Check every field a lab could get wrong, naming the field and the
    /// value it holds
    ///
    /// Returns only the first problem: the message goes into a log line
    /// and a one-line card in the settings window, and a file with two
    /// mistakes gets fixed twice either way
    pub fn validate(&self) -> Result<(), String> {
        if self.participant_id.trim().is_empty() {
            return Err("study.toml: participant_id is empty".to_string());
        }
        if self.condition.trim().is_empty() {
            return Err("study.toml: condition is empty".to_string());
        }

        let durations: [(&str, f64); 4] = [
            ("inhale_duration",           self.inhale_duration),
            ("post_inhale_hold_duration", self.post_inhale_hold_duration),
            ("exhale_duration",           self.exhale_duration),
            ("post_exhale_hold_duration", self.post_exhale_hold_duration),
        ];
        for (name, value) in durations {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "study.toml: {name} is {value}, which has to be a number 0 or more"
                ));
            }
            if value > MAX_PHASE_SECS {
                return Err(format!(
                    "study.toml: {name} is {value}, which has to be {MAX_PHASE_SECS} or less"
                ));
            }
        }
        let cycle: f64 = durations.iter().map(|(_, v)| v).sum();
        if cycle <= 0.0 {
            return Err(
                "study.toml: the four durations add up to 0, which isn't a pace".to_string()
            );
        }

        if !self.drift.is_finite() || self.drift < 1.0 {
            return Err(format!(
                "study.toml: drift is {}, which has to be 1.0 or more",
                self.drift,
            ));
        }

        // Fractions of the phase, the same scale the Randomization card
        // writes, so 0.25 means a quarter either way and 1.0 is the most
        // the app can express
        let fractions: [(&str, f64); 4] = [
            ("randomized_timing_inhale",           self.randomized_timing_inhale),
            ("randomized_timing_post_inhale_hold", self.randomized_timing_post_inhale_hold),
            ("randomized_timing_exhale",           self.randomized_timing_exhale),
            ("randomized_timing_post_exhale_hold", self.randomized_timing_post_exhale_hold),
        ];
        for (name, value) in fractions {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(format!(
                    "study.toml: {name} is {value}, which has to be between 0 and 1"
                ));
            }
        }

        let minutes: [(&str, f64); 2] = [
            ("auto_stop_minutes",         self.auto_stop_minutes),
            ("reminder_interval_minutes", self.reminder_interval_minutes),
        ];
        for (name, value) in minutes {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "study.toml: {name} is {value}, which has to be a number 0 or more"
                ));
            }
        }

        Ok(())
    }

    /// Write the eleven pacing fields into `s` and record the lock
    ///
    /// Touches nothing else. Appearance, shortcuts, window placement and
    /// the runtime animating flag all stay where the participant's own
    /// settings file put them
    pub fn apply(&self, s: &mut Settings) {
        s.inhale_duration           = self.inhale_duration;
        s.post_inhale_hold_duration = self.post_inhale_hold_duration;
        s.exhale_duration           = self.exhale_duration;
        s.post_exhale_hold_duration = self.post_exhale_hold_duration;
        s.drift                     = self.drift;

        s.randomized_timing_inhale           = self.randomized_timing_inhale;
        s.randomized_timing_post_inhale_hold = self.randomized_timing_post_inhale_hold;
        s.randomized_timing_exhale           = self.randomized_timing_exhale;
        s.randomized_timing_post_exhale_hold = self.randomized_timing_post_exhale_hold;

        s.auto_stop_minutes         = self.auto_stop_minutes;
        s.reminder_interval_minutes = self.reminder_interval_minutes;

        s.study = Some(self.clone());
    }
}

/// Read the study file at `path`
///
/// `Ok(None)` when there's no file, which is every build that isn't
/// running in a study. Anything else that goes wrong is an `Err` the
/// caller shows the participant: a lab file that failed to parse has to
/// be loud, because the alternative is a session run at the wrong pace
pub fn load(path: &Path) -> Result<Option<StudyConfig>, String> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("study.toml: {} couldn't be read: {e}", path.display())),
    };
    // A parse error ends in a newline, which the log line and the Study
    // card would both carry
    let cfg: StudyConfig = toml::from_str(&contents)
        .map_err(|e| format!("study.toml: {}", e.to_string().trim_end()))?;
    cfg.validate()?;
    Ok(Some(cfg))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    /// A fresh directory per call, keyed by process id and a nanosecond
    /// timestamp so parallel test runs never collide
    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "exhale-study-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn valid() -> StudyConfig {
        StudyConfig {
            participant_id: "P07".to_string(),
            condition:      "B".to_string(),
            inhale_duration:           5.0,
            post_inhale_hold_duration: 0.0,
            exhale_duration:           5.0,
            post_exhale_hold_duration: 0.0,
            drift:                     1.0,
            randomized_timing_inhale:           0.0,
            randomized_timing_post_inhale_hold: 0.0,
            randomized_timing_exhale:           0.0,
            randomized_timing_post_exhale_hold: 0.0,
            auto_stop_minutes:         0.0,
            reminder_interval_minutes: 0.0,
        }
    }

    #[test]
    fn study_file_sits_beside_the_settings_file() {
        let p = study_file_path(Path::new("/tmp/exhale/settings.toml"));
        assert_eq!(p, PathBuf::from("/tmp/exhale/study.toml"));
    }

    #[test]
    fn minimal_file_parses_and_defaults_the_rest() {
        let dir  = tmp_dir("minimal");
        let path = dir.join(STUDY_FILE_NAME);
        std::fs::write(&path, concat!(
            "participant_id = \"P01\"\n",
            "condition = \"A\"\n",
            "inhale_duration = 5.0\n",
            "post_inhale_hold_duration = 0.0\n",
            "exhale_duration = 5.0\n",
            "post_exhale_hold_duration = 0.0\n",
        )).expect("write study file");

        let cfg = load(&path).expect("load").expect("file is present");
        assert_eq!(cfg.participant_id, "P01");
        assert_eq!(cfg.condition, "A");
        assert_eq!(cfg.drift, 1.0, "drift defaults to off");
        assert_eq!(cfg.randomized_timing_inhale, 0.0);
        assert_eq!(cfg.randomized_timing_post_inhale_hold, 0.0);
        assert_eq!(cfg.randomized_timing_exhale, 0.0);
        assert_eq!(cfg.randomized_timing_post_exhale_hold, 0.0);
        assert_eq!(cfg.auto_stop_minutes, 0.0);
        assert_eq!(cfg.reminder_interval_minutes, 0.0);
    }

    #[test]
    fn full_file_parses_and_round_trips_through_apply() {
        let dir  = tmp_dir("full");
        let path = dir.join(STUDY_FILE_NAME);
        std::fs::write(&path, concat!(
            "participant_id = \"P-12_b\"\n",
            "condition = \"slow\"\n",
            "inhale_duration = 4.0\n",
            "post_inhale_hold_duration = 1.5\n",
            "exhale_duration = 6.0\n",
            "post_exhale_hold_duration = 0.5\n",
            "drift = 1.001\n",
            "randomized_timing_inhale = 0.1\n",
            "randomized_timing_post_inhale_hold = 0.2\n",
            "randomized_timing_exhale = 0.3\n",
            "randomized_timing_post_exhale_hold = 0.4\n",
            "auto_stop_minutes = 30.0\n",
            "reminder_interval_minutes = 15.0\n",
        )).expect("write study file");

        let cfg = load(&path).expect("load").expect("file is present");
        let mut s = Settings::default();
        cfg.apply(&mut s);

        assert_eq!(s.inhale_duration,           4.0);
        assert_eq!(s.post_inhale_hold_duration, 1.5);
        assert_eq!(s.exhale_duration,           6.0);
        assert_eq!(s.post_exhale_hold_duration, 0.5);
        assert_eq!(s.drift,                     1.001);
        assert_eq!(s.randomized_timing_inhale,           0.1);
        assert_eq!(s.randomized_timing_post_inhale_hold, 0.2);
        assert_eq!(s.randomized_timing_exhale,           0.3);
        assert_eq!(s.randomized_timing_post_exhale_hold, 0.4);
        assert_eq!(s.auto_stop_minutes,         30.0);
        assert_eq!(s.reminder_interval_minutes, 15.0);
        assert_eq!(s.study.as_ref(), Some(&cfg));
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let dir = tmp_dir("missing");
        assert_eq!(load(&dir.join(STUDY_FILE_NAME)), Ok(None));
    }

    #[test]
    fn every_validation_rule_names_its_field() {
        let cases: Vec<(StudyConfig, &str)> = vec![
            (StudyConfig { participant_id: "  ".into(), ..valid() }, "participant_id"),
            (StudyConfig { condition: String::new(),    ..valid() }, "condition"),
            (StudyConfig { inhale_duration: -1.0,       ..valid() }, "inhale_duration"),
            (StudyConfig { post_inhale_hold_duration: f64::NAN, ..valid() }, "post_inhale_hold_duration"),
            (StudyConfig { exhale_duration: f64::INFINITY, ..valid() }, "exhale_duration"),
            (StudyConfig { post_exhale_hold_duration: -0.5, ..valid() }, "post_exhale_hold_duration"),
            (StudyConfig { drift: 0.99,                 ..valid() }, "drift"),
            (StudyConfig { drift: f64::NAN,             ..valid() }, "drift"),
            (StudyConfig { randomized_timing_inhale: 1.5, ..valid() }, "randomized_timing_inhale"),
            (StudyConfig { randomized_timing_post_inhale_hold: -0.1, ..valid() }, "randomized_timing_post_inhale_hold"),
            (StudyConfig { randomized_timing_exhale: f64::NAN, ..valid() }, "randomized_timing_exhale"),
            (StudyConfig { randomized_timing_post_exhale_hold: 2.0, ..valid() }, "randomized_timing_post_exhale_hold"),
            (StudyConfig { auto_stop_minutes: -1.0,     ..valid() }, "auto_stop_minutes"),
            (StudyConfig { reminder_interval_minutes: f64::NAN, ..valid() }, "reminder_interval_minutes"),
        ];
        for (cfg, field) in cases {
            let err = cfg.validate().err().unwrap_or_else(|| panic!("expected {field} to be rejected"));
            assert!(err.starts_with("study.toml: "), "{field}: {err}");
            assert!(err.contains(field), "expected {field} in the message, got {err}");
        }
    }

    #[test]
    fn four_zero_durations_are_not_a_pace() {
        let cfg = StudyConfig {
            inhale_duration:           0.0,
            post_inhale_hold_duration: 0.0,
            exhale_duration:           0.0,
            post_exhale_hold_duration: 0.0,
            ..valid()
        };
        let err = cfg.validate().expect_err("all-zero durations are rejected");
        assert_eq!(err, "study.toml: the four durations add up to 0, which isn't a pace");
    }

    #[test]
    fn a_duration_past_the_controller_cap_is_rejected() {
        let over = MAX_PHASE_SECS + 1.0;
        let cases = [
            (StudyConfig { inhale_duration: over,           ..valid() }, "inhale_duration"),
            (StudyConfig { post_inhale_hold_duration: over, ..valid() }, "post_inhale_hold_duration"),
            (StudyConfig { exhale_duration: over,           ..valid() }, "exhale_duration"),
            (StudyConfig { post_exhale_hold_duration: over, ..valid() }, "post_exhale_hold_duration"),
        ];
        for (cfg, field) in cases {
            assert_eq!(
                cfg.validate(),
                Err(format!("study.toml: {field} is {over}, which has to be {MAX_PHASE_SECS} or less")),
            );
        }

        // The cap itself is a pace the controller runs as written
        let at_cap = StudyConfig { exhale_duration: MAX_PHASE_SECS, ..valid() };
        assert_eq!(at_cap.validate(), Ok(()));
    }

    #[test]
    fn a_valid_file_validates() {
        assert_eq!(valid().validate(), Ok(()));
    }

    #[test]
    fn an_unknown_field_is_rejected() {
        let dir  = tmp_dir("unknown");
        let path = dir.join(STUDY_FILE_NAME);
        std::fs::write(&path, concat!(
            "participant_id = \"P01\"\n",
            "condition = \"A\"\n",
            "inhale_duration = 5.0\n",
            "post_inhale_hold_duration = 0.0\n",
            "exhale_duration = 5.0\n",
            "post_exhale_hold_duration = 0.0\n",
            "randomised_timing_inhale = 0.2\n",
        )).expect("write study file");

        let err = load(&path).expect_err("a misspelled field fails loudly");
        assert!(err.starts_with("study.toml: "), "{err}");
        assert!(err.contains("randomised_timing_inhale"), "{err}");
    }

    #[test]
    fn apply_leaves_appearance_and_shortcuts_alone() {
        let mut s = Settings::default();
        s.inhale_color    = [0.2, 0.4, 0.6, 1.0];
        s.overlay_opacity = 0.75;
        s.shape           = crate::types::AnimationShape::Circle;
        s.keyboard_shortcuts.start = Some(crate::settings::KeyboardShortcut::new(
            crate::settings::KBD_MOD_CTRL | crate::settings::KBD_MOD_SHIFT,
            "KeyG",
        ));
        s.is_animating = true;
        let before = s.clone();

        valid().apply(&mut s);

        assert_eq!(s.inhale_color,       before.inhale_color);
        assert_eq!(s.overlay_opacity,    before.overlay_opacity);
        assert_eq!(s.shape,              before.shape);
        assert_eq!(s.keyboard_shortcuts, before.keyboard_shortcuts);
        assert!(s.is_animating);
        assert!(s.study.is_some(), "apply records the lock");
    }

    #[test]
    fn a_normal_pace_control_arm_is_twelve_a_minute() {
        // The only control arm the file has to express: the same overlay
        // at a pace nobody would call slow. Rate is the single
        // difference between the arms
        let cfg = StudyConfig {
            inhale_duration:           2.5,
            post_inhale_hold_duration: 0.0,
            exhale_duration:           2.5,
            post_exhale_hold_duration: 0.0,
            ..valid()
        };
        assert_eq!(cfg.validate(), Ok(()));
        let mut s = Settings::default();
        cfg.apply(&mut s);
        assert!((s.breaths_per_min().expect("a rate") - 12.0).abs() < 1e-9);
    }
}
