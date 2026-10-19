//! The study file: a lab's pacing lock, read from disk beside `settings.toml`
//!
//! A `study.toml` next to the settings file pins the eleven numeric
//! pacing fields and the nine appearance and display fields, and marks
//! the running app as a study build. The file is the only source of the
//! lock, so deleting it lifts the lock on the next launch and no
//! uninstall is needed
//!
//! The appearance and display fields have the names and value types
//! `settings.toml` uses, so a lab copies those lines across as they
//! stand. A field the file leaves out is locked at the shipped default,
//! which keeps two participants' screens the same unless the lab says
//! otherwise. Only Show In (`app_visibility`) stays the participant's
//!
//! Nothing here interprets the `condition` code. It's an opaque string
//! the lab assigns, carried into the session log so a log can be matched
//! back to an arm without the app knowing which arm it's running
//!
//! A study file that exists and can't be used is never ignored. The
//! app fails closed: [`load`] returns a [`StudyError`], and
//! [`StudyStatus::Broken`] tells the app to hold the overlay still and
//! show the error. The error carries two texts, because the participant
//! mustn't be able to read the pace off an error card but the debug log
//! has to say what went wrong

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    controller::{MAX_PHASE_SECS, MIN_PHASE_SECS},
    session_log::MAX_PARTICIPANT_ID_CHARS,
    settings::Settings,
    types::{AnimationMode, AnimationShape, ColorFillGradient, DisplayMode, HoldRippleMode},
};

/// Name of the study file, read from the directory that holds `settings.toml`
pub const STUDY_FILE_NAME: &str = "study.toml";

/// Longest auto-stop or reminder interval, in minutes (one week)
const MAX_TIMER_MINUTES: f64 = 10_080.0;
/// Shortest auto-stop, in minutes (6 seconds)
const MIN_AUTO_STOP_MINUTES: f64 = 0.1;
/// Shortest reminder interval, in minutes
const MIN_REMINDER_MINUTES: f64 = 1.0;

/// Why a study file couldn't be used
///
/// Two texts. [`StudyError::detail`] is for the app's debug log and may
/// hold the values the file configured. [`StudyError::summary`] is what
/// goes on screen: it names the file, and the line and the field when
/// they're known, and it never quotes a configured value or a piece of
/// the file. `Display` prints the summary, so a plain `{error}` can't
/// leak a value onto the screen
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StudyError {
    detail:  String,
    field:   Option<String>,
    line:    Option<usize>,
    problem: String,
}

impl StudyError {
    /// A problem with one field. `detail` is the debug log's wording
    fn field(field: &str, problem: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            detail:  detail.into(),
            field:   Some(field.to_string()),
            line:    None,
            problem: problem.into(),
        }
    }

    /// A problem with a number, or a list of them. The detail quotes it
    /// and the summary never does
    ///
    /// The value prints with `{:?}`, which writes `1e308` for the number
    /// whose `{}` form is 309 digits long
    fn out_of_range(field: &str, value: impl std::fmt::Debug, rule: &str) -> Self {
        Self::field(field, rule, format!("{field} is {value:?}, which {rule}"))
    }

    /// The session log a study needs can't be opened
    ///
    /// For the app to build when `SessionLog::open` fails, so the same
    /// card that reports a bad study file reports this. The summary
    /// names no file and no folder. The detail holds the OS's reason,
    /// which carries neither: the log's file name has the participant
    /// id in it, and the debug log holds no id
    pub fn log_unavailable(error: &std::io::Error) -> Self {
        Self::file(
            "the session log can't be written",
            format!("the session log couldn't be opened: {error}"),
        )
    }

    /// A problem with the file as a whole
    fn file(problem: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            detail:  detail.into(),
            field:   None,
            line:    None,
            problem: problem.into(),
        }
    }

    fn at_line(mut self, line: Option<usize>) -> Self {
        self.line = line;
        self
    }

    /// What went wrong, with the values the file configured
    ///
    /// For the app's debug log only. It holds no participant id and no
    /// path, but it can quote a pace
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// What the screen shows: the file, then the line and the field
    /// when known, then what's wrong
    ///
    /// Never a configured value and never a piece of the file
    pub fn summary(&self) -> String {
        let mut out = String::from(STUDY_FILE_NAME);
        if let Some(line) = self.line {
            out.push_str(&format!(", line {line}"));
        }
        if let Some(field) = &self.field {
            out.push_str(&format!(", field {field}"));
        }
        out.push_str(": ");
        out.push_str(&self.problem);
        out
    }

    /// The field the problem is in, when it's in one
    pub fn field_name(&self) -> Option<&str> {
        self.field.as_deref()
    }

    /// The line the problem is on, when the file's text showed one
    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

impl std::fmt::Display for StudyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.summary())
    }
}

impl std::error::Error for StudyError {}

/// What the study file beside `settings.toml` amounts to
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StudyStatus {
    /// There's no study file, which is every build that isn't running a study
    Off,
    /// A valid study file locked the pacing
    Locked,
    /// A study file, or one saved under a wrong name, is there and
    /// can't be used
    ///
    /// The app holds the overlay still and shows the error, because the
    /// alternative is a session at a pace the lab didn't assign
    Broken(StudyError),
}

impl StudyStatus {
    /// The error, when the file is unusable
    pub fn error(&self) -> Option<&StudyError> {
        match self {
            StudyStatus::Broken(e) => Some(e),
            _                      => None,
        }
    }
}

fn default_drift() -> f64 { 1.0 }

/// Read a color as exactly four numbers
///
/// `[f32; 4]` on its own takes the first four of a longer array and
/// drops the rest. In a study file that's a color the lab didn't write,
/// and a participant's screen would show it without a word
fn four_numbers<'de, D>(deserializer: D) -> Result<Option<[f32; 4]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Some(list) = Option::<Vec<f32>>::deserialize(deserializer)? else {
        return Ok(None);
    };
    match <[f32; 4]>::try_from(list.as_slice()) {
        Ok(color) => Ok(Some(color)),
        Err(_)    => Err(serde::de::Error::invalid_length(list.len(), &"an array of length 4")),
    }
}

/// The color fields, which report any mistake as the same rule
const COLOR_FIELDS: [&str; 3] = ["inhale_color", "exhale_color", "background_color"];

/// One lab's assigned configuration for one participant
///
/// Required in the file: `participant_id`, `condition` and the four
/// durations. Everything else defaults. `deny_unknown_fields` makes a
/// misspelled optional field fail at load, because it would otherwise
/// be a silently wrong pace for a whole session
///
/// The last nine fields are appearance and display, with the names and
/// types of the same settings in `settings.toml`. `None` means the file
/// leaves the field out, and the participant's screen gets the shipped
/// default for it, never whatever their own `settings.toml` holds
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

    // Appearance and display. Linear RGBA, components 0 to 1
    #[serde(default, deserialize_with = "four_numbers", skip_serializing_if = "Option::is_none")]
    pub inhale_color:        Option<[f32; 4]>,
    #[serde(default, deserialize_with = "four_numbers", skip_serializing_if = "Option::is_none")]
    pub exhale_color:        Option<[f32; 4]>,
    #[serde(default, deserialize_with = "four_numbers", skip_serializing_if = "Option::is_none")]
    pub background_color:    Option<[f32; 4]>,
    /// 0 to 1
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay_opacity:     Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape:               Option<AnimationShape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_fill_gradient: Option<ColorFillGradient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_mode:      Option<AnimationMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_ripple_mode:    Option<HoldRippleMode>,
    /// `window` opens the animation as a regular window, full screen
    /// and borderless on the primary display
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_mode:        Option<DisplayMode>,
}

/// Where the study file lives for a given settings file
pub fn study_file_path(settings_path: &Path) -> PathBuf {
    settings_path.with_file_name(STUDY_FILE_NAME)
}

impl StudyConfig {
    /// Check every field a lab could get wrong, naming the field
    ///
    /// Returns only the first problem: a file with two mistakes gets
    /// fixed twice either way. The error's summary names the field and
    /// never the value, and its detail names both
    pub fn validate(&self) -> Result<(), StudyError> {
        // The same trim the log file name applies, so an id padded with
        // whitespace counts by what's left
        let id_chars = self.participant_id.trim().chars().count();
        if id_chars == 0 {
            return Err(StudyError::field("participant_id", "can't be empty", "participant_id is empty"));
        }
        if id_chars > MAX_PARTICIPANT_ID_CHARS {
            let rule = format!("has to be {MAX_PARTICIPANT_ID_CHARS} characters or fewer");
            return Err(StudyError::field(
                "participant_id",
                rule.as_str(),
                format!("participant_id is {id_chars} characters, which {rule}"),
            ));
        }
        if self.condition.trim().is_empty() {
            return Err(StudyError::field("condition", "can't be empty", "condition is empty"));
        }

        // 0 takes no time. Anything else has to be a length the
        // controller runs as written, because below its floor a phase
        // runs longer than the number the log's header states
        let durations: [(&str, f64); 4] = [
            ("inhale_duration",           self.inhale_duration),
            ("post_inhale_hold_duration", self.post_inhale_hold_duration),
            ("exhale_duration",           self.exhale_duration),
            ("post_exhale_hold_duration", self.post_exhale_hold_duration),
        ];
        let duration_rule = format!("has to be 0, or between {MIN_PHASE_SECS} and {MAX_PHASE_SECS}");
        for (name, value) in durations {
            if value != 0.0 && !(MIN_PHASE_SECS..=MAX_PHASE_SECS).contains(&value) {
                return Err(StudyError::out_of_range(name, value, &duration_rule));
            }
        }
        let cycle: f64 = durations.iter().map(|(_, v)| v).sum();
        if cycle <= 0.0 {
            return Err(StudyError::file(
                "the four durations can't all be 0",
                "the four durations add up to 0, which isn't a pace",
            ));
        }

        if !self.drift.is_finite() || self.drift < 1.0 {
            return Err(StudyError::out_of_range("drift", self.drift, "has to be 1.0 or more"));
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
                return Err(StudyError::out_of_range(name, value, "has to be between 0 and 1"));
            }
        }

        // Off, or a length the timers can schedule. A value below the
        // floor fires on most ticks of the event loop, and one past a
        // week gains nothing and overflows the clock when it's huge
        let minutes: [(&str, f64, f64); 2] = [
            ("auto_stop_minutes",         self.auto_stop_minutes,         MIN_AUTO_STOP_MINUTES),
            ("reminder_interval_minutes", self.reminder_interval_minutes, MIN_REMINDER_MINUTES),
        ];
        for (name, value, floor) in minutes {
            if value != 0.0 && !(floor..=MAX_TIMER_MINUTES).contains(&value) {
                return Err(StudyError::out_of_range(
                    name,
                    value,
                    &format!("has to be 0, or between {floor} and {MAX_TIMER_MINUTES}"),
                ));
            }
        }

        // Appearance. Components and opacity are fractions of full, and
        // a value past 0 to 1 draws as nothing like what the file says
        let colors: [(&str, Option<[f32; 4]>); 3] = [
            ("inhale_color",     self.inhale_color),
            ("exhale_color",     self.exhale_color),
            ("background_color", self.background_color),
        ];
        for (name, color) in colors {
            if let Some(c) = color {
                if !c.iter().all(|x| (0.0..=1.0).contains(x)) {
                    return Err(StudyError::out_of_range(
                        name,
                        c,
                        "has to be four numbers between 0 and 1",
                    ));
                }
            }
        }
        if let Some(o) = self.overlay_opacity {
            if !(0.0..=1.0).contains(&o) {
                return Err(StudyError::out_of_range("overlay_opacity", o, "has to be between 0 and 1"));
            }
        }

        // The fullscreen shape fills the screen with the animated color.
        // With the two colors the same there's nothing to animate, and the
        // controller stops advancing phases and draws a still tint. The
        // test is the controller's own, over the colors the participant
        // will have, the shipped ones included
        let mut look = Settings::default();
        self.apply_appearance(&mut look);
        if look.shape == AnimationShape::Fullscreen && look.inhale_exhale_colors_match() {
            return Err(StudyError::field(
                "exhale_color",
                "has to differ from inhale_color when the shape is fullscreen",
                format!(
                    "inhale_color {:?} and exhale_color {:?} match, and the fullscreen shape then draws a still tint",
                    look.inhale_color, look.exhale_color,
                ),
            ));
        }

        Ok(())
    }

    /// Write the eleven pacing fields and the nine appearance and display
    /// fields into `s`, and record the lock
    ///
    /// An appearance field the file leaves out gets the shipped default.
    /// Touches nothing else: Show In (`app_visibility`), shortcuts, window
    /// placement and the runtime animating and paused flags all stay where
    /// the participant's own settings file put them
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

        self.apply_appearance(s);

        s.study = Some(self.clone());
    }

    /// The nine appearance and display fields: the file's value, or
    /// `Settings::default()`'s where the file has none
    fn apply_appearance(&self, s: &mut Settings) {
        let shipped = Settings::default();
        s.inhale_color         = self.inhale_color.unwrap_or(shipped.inhale_color);
        s.exhale_color         = self.exhale_color.unwrap_or(shipped.exhale_color);
        s.background_color     = self.background_color.unwrap_or(shipped.background_color);
        s.overlay_opacity      = self.overlay_opacity.unwrap_or(shipped.overlay_opacity);
        s.shape                = self.shape.unwrap_or(shipped.shape);
        s.color_fill_gradient  = self.color_fill_gradient.unwrap_or(shipped.color_fill_gradient);
        s.animation_mode       = self.animation_mode.unwrap_or(shipped.animation_mode);
        s.hold_ripple_mode     = self.hold_ripple_mode.unwrap_or(shipped.hold_ripple_mode);
        s.display_mode         = self.display_mode.unwrap_or(shipped.display_mode);
    }
}

/// Read the study file at `path`
///
/// `Ok(None)` when there's no file, which is every build that isn't
/// running in a study. Anything else that goes wrong is an `Err` the
/// caller shows the participant: a lab file that failed to parse has to
/// be loud, because the alternative is a session run at the wrong pace.
/// That covers a file that can't be read (a directory, no permission,
/// text that isn't UTF-8), a link to nothing, and an entry in the same
/// folder whose name starts with `study` but isn't `study.toml`, such
/// as `study.toml.txt` from an editor that hides extensions
pub fn load(path: &Path) -> Result<Option<StudyConfig>, StudyError> {
    let cfg = read_config(path)?;
    if let Some(name) = misnamed_entry(path) {
        return Err(StudyError::file(
            format!("found {name:?} in the settings folder. Rename it to {STUDY_FILE_NAME}"),
            format!("{name:?} is in the settings folder and isn't named {STUDY_FILE_NAME}"),
        ));
    }
    Ok(cfg)
}

/// The file at `path` parsed and validated, or `None` when it isn't there
fn read_config(path: &Path) -> Result<Option<StudyConfig>, StudyError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // A link to a file that isn't there reads as missing too. A
            // lab that made the link meant a study, so it isn't "no file"
            if std::fs::symlink_metadata(path).is_ok() {
                return Err(StudyError::file(
                    "is a link to a file that doesn't exist",
                    "the study file is a link whose target is missing",
                ));
            }
            return Ok(None);
        }
        Err(e) => {
            let problem = if e.kind() == std::io::ErrorKind::InvalidData {
                "isn't UTF-8 text. Save it as UTF-8".to_string()
            } else {
                format!("couldn't be read: {e}")
            };
            return Err(StudyError::file(problem, format!("the study file couldn't be read: {e}")));
        }
    };
    let cfg: StudyConfig = toml::from_str(&contents).map_err(|e| parse_error(&e, &contents))?;
    cfg.validate().map_err(|e| {
        let line = e.field_name().and_then(|name| line_of_key(&contents, name));
        e.at_line(line)
    })?;
    Ok(Some(cfg))
}

/// The first entry in the study file's folder that looks like a study
/// file with the wrong name
///
/// Any name that starts with `study`, in any case, other than exactly
/// `study.toml`. A disk that ignores case would read `Study.toml` as the
/// study file, and it still counts here, so the same folder behaves the
/// same on every platform and the lab renames the file once. Sorted, so
/// the answer doesn't depend on the order the disk lists names in. A
/// folder that can't be listed has none
fn misnamed_entry(path: &Path) -> Option<String> {
    let dir = match path.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    };
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != STUDY_FILE_NAME && name.to_lowercase().starts_with("study"))
        .collect();
    names.sort();
    names.into_iter().next()
}

/// Turn the TOML parser's error into one that shows the participant
/// nothing the lab configured
///
/// The parser's `Display` prints the offending line, and its `message()`
/// can quote the value (`invalid type: string "2.5", expected f64`). The
/// summary built here takes only three things from them: the line, the
/// key on that line, and which kind of mistake it was. A missing or an
/// unknown field is named too, because the lab needs that name to fix
/// the file and it isn't a configured value
fn parse_error(error: &toml::de::Error, contents: &str) -> StudyError {
    let message = error.message();
    let shown   = message.lines().collect::<Vec<_>>().join(", ");

    // A missing field's span is the whole file, which has no line to name
    if let Some(name) = quoted_after(message, "missing field `") {
        return StudyError::field(name, "is required but missing", shown);
    }
    let line = error.span().map(|span| line_at(contents, span.start));
    if let Some(name) = quoted_after(message, "unknown field `") {
        let detail = with_line(line, &shown);
        return StudyError::field(name, "isn't a field exhale knows", detail).at_line(line);
    }

    let field = line.and_then(|n| contents.lines().nth(n - 1)).and_then(key_in);
    let problem: String = if message.starts_with("invalid type") || message.starts_with("invalid length") {
        if field.is_some_and(|name| COLOR_FIELDS.contains(&name)) {
            "has to be four numbers between 0 and 1".to_string()
        } else if message.ends_with("expected f64") || message.ends_with("expected f32") {
            "has to be a number".to_string()
        } else if message.ends_with("expected a string") {
            "has to be a string".to_string()
        } else {
            "has the wrong type of value".to_string()
        }
    } else if message.starts_with("wanted string") {
        "has to be a string".to_string()
    } else if message.starts_with("unknown variant") {
        // The message quotes the value, then lists what's allowed after
        // its last ", expected ". That clause is the app's own list of
        // names, whatever the value held, so it's safe to show
        match message.rsplit_once(", expected ") {
            Some((_, allowed)) => format!("has to be {}", allowed.replace('`', "")),
            None               => "has a value that isn't allowed".to_string(),
        }
    } else if message.starts_with("duplicate key") {
        "is set twice".to_string()
    } else if field.is_some() {
        "has a value that isn't valid TOML".to_string()
    } else {
        "isn't valid TOML".to_string()
    };
    // The parser quotes the value of a mistyped field, and a mistyped id
    // is a participant id. The debug log holds none, so that line says
    // only what the summary says
    let detail = if field == Some("participant_id") {
        with_line(line, &problem)
    } else {
        with_line(line, &shown)
    };
    let error = match field {
        Some(name) => StudyError::field(name, problem, detail),
        None       => StudyError::file(problem, detail),
    };
    error.at_line(line)
}

fn with_line(line: Option<usize>, text: &str) -> String {
    match line {
        Some(n) => format!("line {n}: {text}"),
        None    => text.to_string(),
    }
}

/// The text between the backticks that follow `prefix` in `message`
fn quoted_after<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    message.strip_prefix(prefix)?.split('`').next()
}

/// The 1-based line a byte offset falls on
fn line_at(contents: &str, offset: usize) -> usize {
    let end = offset.min(contents.len());
    contents.as_bytes()[..end].iter().filter(|&&b| b == b'\n').count() + 1
}

/// The key a `key = value` line assigns, when it's a bare key
///
/// `None` for a comment, a table header, a blank line, and a quoted or
/// dotted key. Bare keys only, so what comes back is made of letters,
/// digits, `_` and `-` and can't hold a value
fn key_in(line: &str) -> Option<&str> {
    let (key, _) = line.trim_start_matches('\u{feff}').split_once('=')?;
    let key = key.trim();
    let bare = !key.is_empty()
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    bare.then_some(key)
}

/// The 1-based line that assigns `key`
fn line_of_key(contents: &str, key: &str) -> Option<usize> {
    contents.lines().position(|line| key_in(line) == Some(key)).map(|i| i + 1)
}

// ─── Test fixtures ────────────────────────────────────────────────────────────

/// Study files the tests of other modules in this crate build on
#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    /// A study file that sets the pace and nothing else: 5 / 0 / 5 / 0
    pub(crate) fn pace_only() -> StudyConfig {
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
            inhale_color:        None,
            exhale_color:        None,
            background_color:    None,
            overlay_opacity:     None,
            shape:               None,
            color_fill_gradient: None,
            animation_mode:      None,
            hold_ripple_mode:    None,
            display_mode:        None,
        }
    }

    /// A study file that sets all twenty locked fields, each to a value
    /// that differs from the shipped default
    pub(crate) fn everything() -> StudyConfig {
        StudyConfig {
            inhale_duration:           4.0,
            post_inhale_hold_duration: 1.0,
            exhale_duration:           6.0,
            post_exhale_hold_duration: 2.0,
            drift:                     1.002,
            randomized_timing_inhale:           0.1,
            randomized_timing_post_inhale_hold: 0.2,
            randomized_timing_exhale:           0.3,
            randomized_timing_post_exhale_hold: 0.4,
            auto_stop_minutes:         30.0,
            reminder_interval_minutes: 15.0,
            inhale_color:        Some([0.2, 0.4, 0.6, 1.0]),
            exhale_color:        Some([0.9, 0.8, 0.1, 1.0]),
            background_color:    Some([0.1, 0.1, 0.1, 0.5]),
            overlay_opacity:     Some(0.6),
            shape:               Some(AnimationShape::Circle),
            color_fill_gradient: Some(ColorFillGradient::Off),
            animation_mode:      Some(AnimationMode::Linear),
            hold_ripple_mode:    Some(HoldRippleMode::Stark),
            display_mode:        Some(DisplayMode::Window),
            ..pace_only()
        }
    }
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
        fixtures::pace_only()
    }

    /// The six required fields, one per line, so a test can append lines
    /// and know which line each lands on
    const MINIMAL: &str = concat!(
        "participant_id = \"P01\"\n",
        "condition = \"A\"\n",
        "inhale_duration = 5.0\n",
        "post_inhale_hold_duration = 0.0\n",
        "exhale_duration = 5.0\n",
        "post_exhale_hold_duration = 0.0\n",
    );

    fn write_study(dir: &Path, contents: &str) -> PathBuf {
        let path = dir.join(STUDY_FILE_NAME);
        std::fs::write(&path, contents).expect("write study file");
        path
    }

    #[test]
    fn study_file_sits_beside_the_settings_file() {
        let p = study_file_path(Path::new("/tmp/exhale/settings.toml"));
        assert_eq!(p, PathBuf::from("/tmp/exhale/study.toml"));
    }

    #[test]
    fn minimal_file_parses_and_defaults_the_rest() {
        let dir  = tmp_dir("minimal");
        let path = write_study(&dir, MINIMAL);

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
        let path = write_study(&dir, concat!(
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
        ));

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
    fn a_byte_order_mark_and_windows_line_endings_load() {
        // What Windows Notepad writes
        let dir  = tmp_dir("bom");
        let path = write_study(&dir, &format!("\u{feff}{}", MINIMAL.replace('\n', "\r\n")));
        let cfg  = load(&path).expect("load").expect("file is present");
        assert_eq!(cfg.participant_id, "P01");
    }

    #[test]
    fn every_validation_rule_names_its_field_and_never_its_value() {
        // Each bad value is a distinctive number, so a leak into the
        // summary can't pass for a coincidence
        let cases: Vec<(StudyConfig, &str, &str)> = vec![
            (StudyConfig { participant_id: "  ".into(), ..valid() }, "participant_id", ""),
            (StudyConfig { participant_id: "x".repeat(MAX_PARTICIPANT_ID_CHARS + 1), ..valid() }, "participant_id", "xxxxxxxx"),
            (StudyConfig { condition: String::new(),    ..valid() }, "condition", ""),
            (StudyConfig { inhale_duration: -7.7777,    ..valid() }, "inhale_duration", "7.7777"),
            (StudyConfig { post_inhale_hold_duration: f64::NAN, ..valid() }, "post_inhale_hold_duration", "NaN"),
            (StudyConfig { exhale_duration: f64::INFINITY, ..valid() }, "exhale_duration", "inf"),
            (StudyConfig { post_exhale_hold_duration: 0.0123, ..valid() }, "post_exhale_hold_duration", "0.0123"),
            (StudyConfig { drift: 0.9876,               ..valid() }, "drift", "0.9876"),
            (StudyConfig { drift: f64::NAN,             ..valid() }, "drift", "NaN"),
            (StudyConfig { randomized_timing_inhale: 1.5432, ..valid() }, "randomized_timing_inhale", "1.5432"),
            (StudyConfig { randomized_timing_post_inhale_hold: -0.1234, ..valid() }, "randomized_timing_post_inhale_hold", "0.1234"),
            (StudyConfig { randomized_timing_exhale: f64::NAN, ..valid() }, "randomized_timing_exhale", "NaN"),
            (StudyConfig { randomized_timing_post_exhale_hold: 2.3456, ..valid() }, "randomized_timing_post_exhale_hold", "2.3456"),
            (StudyConfig { auto_stop_minutes: -1.4321,  ..valid() }, "auto_stop_minutes", "1.4321"),
            (StudyConfig { reminder_interval_minutes: f64::NAN, ..valid() }, "reminder_interval_minutes", "NaN"),
        ];
        for (cfg, field, value) in cases {
            let err = cfg.validate().err().unwrap_or_else(|| panic!("expected {field} to be rejected"));
            let summary = err.summary();
            assert!(summary.starts_with("study.toml, field "), "{field}: {summary}");
            assert!(summary.contains(field), "expected {field} in the summary, got {summary}");
            assert_eq!(err.field_name(), Some(field));
            assert!(err.detail().contains(field), "{field}: {}", err.detail());
            if !value.is_empty() {
                assert!(!summary.contains(value), "{field}: the summary quotes {value}: {summary}");
            }
            assert_eq!(err.to_string(), summary, "Display is the summary");
        }
    }

    #[test]
    fn the_detail_keeps_the_value_short_even_when_the_number_is_huge() {
        let err = StudyConfig { inhale_duration: 1e308, ..valid() }.validate().expect_err("1e308 is out of range");
        assert_eq!(
            err.detail(),
            "inhale_duration is 1e308, which has to be 0, or between 0.1 and 86400",
        );
        assert!(err.detail().len() < 100, "{}", err.detail());
        assert_eq!(err.summary(), "study.toml, field inhale_duration: has to be 0, or between 0.1 and 86400");
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
        assert_eq!(err.summary(), "study.toml: the four durations can't all be 0");
        assert_eq!(err.detail(), "the four durations add up to 0, which isn't a pace");
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
            let err = cfg.validate().expect_err("over the cap");
            assert_eq!(
                err.detail(),
                format!("{field} is {over:?}, which has to be 0, or between 0.1 and 86400"),
            );
        }

        // The cap itself is a pace the controller runs as written
        let at_cap = StudyConfig { exhale_duration: MAX_PHASE_SECS, ..valid() };
        assert_eq!(at_cap.validate(), Ok(()));
    }

    #[test]
    fn a_duration_between_zero_and_the_controller_floor_is_rejected() {
        // The controller floors a phase it was asked for to 0.1 s, so
        // 0.05 s would run as 0.1 s while the log header said 0.05
        for tiny in [0.05, MIN_PHASE_SECS * 0.99, 1e-9, f64::MIN_POSITIVE] {
            for field in ["inhale_duration", "post_inhale_hold_duration", "exhale_duration", "post_exhale_hold_duration"] {
                let mut cfg = valid();
                match field {
                    "inhale_duration"           => cfg.inhale_duration = tiny,
                    "post_inhale_hold_duration" => cfg.post_inhale_hold_duration = tiny,
                    "exhale_duration"           => cfg.exhale_duration = tiny,
                    _                           => cfg.post_exhale_hold_duration = tiny,
                }
                let err = cfg.validate().unwrap_err();
                assert_eq!(err.field_name(), Some(field), "{tiny:?} in {field}");
            }
        }

        // The floor itself runs as written, and 0 still means "no phase"
        assert_eq!(StudyConfig { inhale_duration: MIN_PHASE_SECS, ..valid() }.validate(), Ok(()));
        assert_eq!(StudyConfig { post_inhale_hold_duration: 0.0, ..valid() }.validate(), Ok(()));
        assert_eq!(StudyConfig { post_exhale_hold_duration: -0.0, ..valid() }.validate(), Ok(()));
    }

    #[test]
    fn timer_minutes_are_off_or_inside_the_range_the_timers_can_schedule() {
        let auto = |v: f64| StudyConfig { auto_stop_minutes: v, ..valid() }.validate();
        let rem  = |v: f64| StudyConfig { reminder_interval_minutes: v, ..valid() }.validate();

        for ok in [0.0, 0.1, 1.0, 30.0, 10_080.0] {
            assert_eq!(auto(ok), Ok(()), "auto_stop_minutes = {ok}");
        }
        for bad in [0.05, 1e-9, 10_080.5, 1.54e17, 3.07e17, 1e300, -1.0, f64::INFINITY, f64::NAN] {
            let err = auto(bad).expect_err("out of range");
            assert_eq!(err.field_name(), Some("auto_stop_minutes"), "{bad:?}");
        }

        for ok in [0.0, 1.0, 15.0, 10_080.0] {
            assert_eq!(rem(ok), Ok(()), "reminder_interval_minutes = {ok}");
        }
        for bad in [0.5, 0.99, 1e-9, 10_080.5, 1.54e17, 3.07e17, -1.0, f64::INFINITY, f64::NAN] {
            let err = rem(bad).expect_err("out of range");
            assert_eq!(err.field_name(), Some("reminder_interval_minutes"), "{bad:?}");
        }
        assert_eq!(
            auto(0.05).unwrap_err().summary(),
            "study.toml, field auto_stop_minutes: has to be 0, or between 0.1 and 10080",
        );
        assert_eq!(
            rem(0.5).unwrap_err().summary(),
            "study.toml, field reminder_interval_minutes: has to be 0, or between 1 and 10080",
        );
    }

    #[test]
    fn a_participant_id_is_capped_after_trimming() {
        let at_cap = "p".repeat(MAX_PARTICIPANT_ID_CHARS);
        let padded = format!("  {at_cap}\t");
        assert_eq!(StudyConfig { participant_id: at_cap.clone(), ..valid() }.validate(), Ok(()));
        assert_eq!(StudyConfig { participant_id: padded, ..valid() }.validate(), Ok(()), "whitespace doesn't count");

        let over = format!("{at_cap}p");
        let err  = StudyConfig { participant_id: over.clone(), ..valid() }.validate().expect_err("one character over");
        assert_eq!(err.summary(), "study.toml, field participant_id: has to be 64 characters or fewer");
        assert!(!err.summary().contains(&over) && !err.detail().contains(&over), "the id stays out of both texts");

        // Characters, not bytes: 64 two-byte letters fit
        let wide = "é".repeat(MAX_PARTICIPANT_ID_CHARS);
        assert_eq!(StudyConfig { participant_id: wide, ..valid() }.validate(), Ok(()));
    }

    #[test]
    fn a_valid_file_validates() {
        assert_eq!(valid().validate(), Ok(()));
    }

    // ── What the participant is shown for a file that won't parse ──────

    #[test]
    fn an_unknown_field_is_named_with_its_line() {
        let dir  = tmp_dir("unknown");
        let path = write_study(&dir, &format!("{MINIMAL}randomised_timing_inhale = 0.2\n"));

        let err = load(&path).expect_err("a misspelled field fails loudly");
        assert_eq!(
            err.summary(),
            "study.toml, line 7, field randomised_timing_inhale: isn't a field exhale knows",
        );
        assert!(!err.summary().contains("0.2"));
        assert_eq!(err.line(), Some(7));
    }

    #[test]
    fn a_missing_field_is_named_without_a_line() {
        let dir  = tmp_dir("missing-field");
        let path = write_study(&dir, &MINIMAL.replace("exhale_duration = 5.0\n", ""));

        let err = load(&path).expect_err("a required field is missing");
        assert_eq!(err.summary(), "study.toml, field exhale_duration: is required but missing");
        assert_eq!(err.line(), None);

        // An empty file is missing everything and says so the same way
        let empty = write_study(&dir, "");
        let err   = load(&empty).expect_err("an empty file is no study");
        assert!(err.summary().ends_with(": is required but missing"), "{}", err.summary());
    }

    #[test]
    fn a_mistyped_value_is_never_quoted_on_screen() {
        // The parser's own message is `invalid type: string "2.5", expected f64`
        // and its display adds the whole line. The pace stays off the card
        let dir  = tmp_dir("mistyped");
        let path = write_study(&dir, &MINIMAL.replace("inhale_duration = 5.0", "inhale_duration = \"2.5\""));

        let err = load(&path).expect_err("a string where a number goes");
        assert_eq!(err.summary(), "study.toml, line 3, field inhale_duration: has to be a number");
        assert!(!err.summary().contains("2.5"), "{}", err.summary());
        assert!(err.detail().contains("2.5"), "the debug log keeps the value: {}", err.detail());
    }

    #[test]
    fn a_mistyped_participant_id_is_kept_out_of_both_texts() {
        let dir  = tmp_dir("mistyped-id");
        let path = write_study(&dir, &MINIMAL.replace("participant_id = \"P01\"", "participant_id = 4242"));

        let err = load(&path).expect_err("a number where a string goes");
        assert_eq!(err.summary(), "study.toml, line 1, field participant_id: has to be a string");
        assert!(!err.summary().contains("4242"), "{}", err.summary());
        assert!(!err.detail().contains("4242"), "{}", err.detail());
    }

    #[test]
    fn a_syntax_error_names_the_line_and_the_key_and_quotes_nothing() {
        let dir = tmp_dir("syntax");

        let path = write_study(&dir, &MINIMAL.replace("inhale_duration = 5.0", "inhale_duration = 2.5.5"));
        let err  = load(&path).expect_err("not a number");
        assert_eq!(err.summary(), "study.toml, line 3, field inhale_duration: has a value that isn't valid TOML");
        assert!(!err.summary().contains("2.5"), "{}", err.summary());

        // A line with no key to take: only the line is named
        let path = write_study(&dir, &MINIMAL.replace("condition = \"A\"", "condition \"A\""));
        let err  = load(&path).expect_err("no equals sign");
        assert_eq!(err.summary(), "study.toml, line 2: isn't valid TOML");

        let path = write_study(&dir, &format!("{MINIMAL}condition = \"C\"\n"));
        let err  = load(&path).expect_err("a key set twice");
        assert_eq!(err.summary(), "study.toml, line 7, field condition: is set twice");
        assert!(!err.summary().contains('C') || !err.summary().contains("\"C\""));
    }

    #[test]
    fn lines_are_counted_the_same_with_windows_line_endings_and_a_byte_order_mark() {
        let dir  = tmp_dir("crlf-line");
        let text = format!("\u{feff}{}", MINIMAL.replace("inhale_duration = 5.0", "inhale_duration = \"x\"").replace('\n', "\r\n"));
        let path = write_study(&dir, &text);
        let err  = load(&path).expect_err("a string where a number goes");
        assert_eq!(err.summary(), "study.toml, line 3, field inhale_duration: has to be a number");
    }

    #[test]
    fn a_validation_error_gets_the_line_the_field_is_on() {
        let dir  = tmp_dir("validation-line");
        let path = write_study(&dir, &format!("# a note\n{MINIMAL}drift = 0.5\n"));

        let err = load(&path).expect_err("drift below 1.0");
        assert_eq!(err.summary(), "study.toml, line 8, field drift: has to be 1.0 or more");
        assert!(!err.summary().contains("0.5"));
        assert!(err.detail().contains("0.5"), "{}", err.detail());
    }

    #[test]
    fn the_line_and_key_helpers_read_only_bare_keys() {
        assert_eq!(key_in("drift = 1.0"), Some("drift"));
        assert_eq!(key_in("  inhale-duration=5"), Some("inhale-duration"));
        assert_eq!(key_in("\u{feff}participant_id = \"x = y\""), Some("participant_id"));
        assert_eq!(key_in("# drift = 1.0"), None);
        assert_eq!(key_in("[table]"), None);
        assert_eq!(key_in("\"quoted\" = 1"), None);
        assert_eq!(key_in("a.b = 1"), None);
        assert_eq!(key_in(""), None);

        let text = "a = 1\n\nb = 2\r\nc = 3";
        assert_eq!(line_of_key(text, "c"), Some(4));
        assert_eq!(line_of_key(text, "z"), None);
        assert_eq!(line_at(text, 0), 1);
        assert_eq!(line_at(text, 7), 3);
        assert_eq!(line_at(text, 9999), 4, "an offset past the end is the last line");
    }

    // ── A file that exists and can't be used ───────────────────────────

    #[test]
    fn a_directory_in_the_place_of_the_file_is_an_error() {
        let dir = tmp_dir("directory");
        std::fs::create_dir(dir.join(STUDY_FILE_NAME)).expect("make the directory");
        let err = load(&dir.join(STUDY_FILE_NAME)).expect_err("a directory can't be read");
        assert!(err.summary().starts_with("study.toml: couldn't be read"), "{}", err.summary());
    }

    #[test]
    fn text_that_is_not_utf_8_says_to_save_it_as_utf_8() {
        // What Notepad's "Unicode" choice writes: UTF-16 with a byte order mark
        let dir  = tmp_dir("utf16");
        let path = dir.join(STUDY_FILE_NAME);
        let utf16: Vec<u8> = "participant_id = \"P01\"\n"
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect();
        std::fs::write(&path, [vec![0xFF, 0xFE], utf16].concat()).expect("write");
        let err = load(&path).expect_err("not UTF-8");
        assert_eq!(err.summary(), "study.toml: isn't UTF-8 text. Save it as UTF-8");
    }

    #[cfg(unix)]
    #[test]
    fn a_link_to_nothing_is_an_error_and_not_no_file() {
        let dir = tmp_dir("dangling");
        std::os::unix::fs::symlink(dir.join("no-such-target.toml"), dir.join(STUDY_FILE_NAME))
            .expect("make the link");
        let err = load(&dir.join(STUDY_FILE_NAME)).expect_err("a dangling link is a study that failed");
        assert_eq!(err.summary(), "study.toml: is a link to a file that doesn't exist");
    }

    #[test]
    fn a_study_file_saved_under_another_name_is_an_error() {
        // `study.toml.txt` is what Notepad makes with extensions hidden
        let wrong_names = ["study.toml.txt", "Study.toml", "STUDY.TOML", "study (1).toml", "study.toml.bak", "study.tml", "studyfile"];
        for (i, wrong) in wrong_names.into_iter().enumerate() {
            let dir = tmp_dir(&format!("misnamed-{i}"));
            // Valid content, so a case-insensitive disk that reads
            // `Study.toml` as `study.toml` trips only the name rule
            std::fs::write(dir.join(wrong), MINIMAL).expect("write");

            let err = load(&dir.join(STUDY_FILE_NAME)).expect_err(wrong);
            assert_eq!(
                err.summary(),
                format!("study.toml: found {wrong:?} in the settings folder. Rename it to study.toml"),
            );
            assert_eq!(err.field_name(), None);
            assert_eq!(err.line(), None);
        }
    }

    #[test]
    fn a_misnamed_file_beside_a_good_one_still_fails_closed() {
        // Two files and no way to tell which the lab meant
        let dir = tmp_dir("two-files");
        write_study(&dir, MINIMAL);
        std::fs::write(dir.join("study (1).toml"), MINIMAL).expect("write");
        let err = load(&dir.join(STUDY_FILE_NAME)).expect_err("a stray copy");
        assert!(err.summary().contains("study (1).toml"), "{}", err.summary());
    }

    #[test]
    fn the_misnamed_file_reported_does_not_depend_on_the_disk_order() {
        let dir = tmp_dir("sorted");
        for name in ["study-b.toml", "study-a.toml", "study-c.toml"] {
            std::fs::write(dir.join(name), "x").expect("write");
        }
        assert_eq!(misnamed_entry(&dir.join(STUDY_FILE_NAME)), Some("study-a.toml".to_string()));
    }

    #[test]
    fn other_files_in_the_folder_are_none_of_its_business() {
        let dir = tmp_dir("neighbours");
        for name in ["settings.toml", "exhale-log-P07.jsonl", "exhale.lock", "studio.toml", "my-study.toml"] {
            std::fs::write(dir.join(name), "x").expect("write");
        }
        assert_eq!(load(&dir.join(STUDY_FILE_NAME)), Ok(None));

        // And a good file with those neighbours loads
        write_study(&dir, MINIMAL);
        assert!(load(&dir.join(STUDY_FILE_NAME)).expect("load").is_some());
    }

    // ── Appearance and display ─────────────────────────────────────────

    /// The twenty fields a study file locks: eleven pace, nine appearance
    const LOCKED: [&str; 20] = [
        "inhale_duration", "post_inhale_hold_duration", "exhale_duration", "post_exhale_hold_duration",
        "drift",
        "randomized_timing_inhale", "randomized_timing_post_inhale_hold",
        "randomized_timing_exhale", "randomized_timing_post_exhale_hold",
        "auto_stop_minutes", "reminder_interval_minutes",
        "inhale_color", "exhale_color", "background_color", "overlay_opacity",
        "shape", "color_fill_gradient", "animation_mode", "hold_ripple_mode", "display_mode",
    ];

    /// The nine appearance and display names, in the order the log header lists them
    const APPEARANCE: [&str; 9] = [
        "inhale_color", "exhale_color", "background_color", "overlay_opacity",
        "shape", "color_fill_gradient", "animation_mode", "hold_ripple_mode", "display_mode",
    ];

    /// A participant's settings with every field off the values
    /// `fixtures::everything` locks, and off its shipped value too. The
    /// two fields with only two values can't be both: `animation_mode`
    /// and `display_mode` hold their shipped value here
    fn participant_settings() -> Settings {
        let mut s = Settings::default();
        s.inhale_duration           = 8.0;
        s.post_inhale_hold_duration = 3.0;
        s.exhale_duration           = 9.0;
        s.post_exhale_hold_duration = 4.0;
        s.drift                     = 1.5;
        s.randomized_timing_inhale           = 0.9;
        s.randomized_timing_post_inhale_hold = 0.9;
        s.randomized_timing_exhale           = 0.9;
        s.randomized_timing_post_exhale_hold = 0.9;
        s.auto_stop_minutes         = 99.0;
        s.reminder_interval_minutes = 77.0;
        s.inhale_color        = [0.5, 0.5, 0.5, 1.0];
        s.exhale_color        = [0.7, 0.7, 0.7, 1.0];
        s.background_color    = [0.3, 0.3, 0.3, 0.3];
        s.overlay_opacity     = 0.99;
        s.shape               = AnimationShape::Fullscreen;
        s.color_fill_gradient = ColorFillGradient::Inner;
        s.animation_mode      = AnimationMode::Sinusoidal;
        s.hold_ripple_mode    = HoldRippleMode::Off;
        s.display_mode        = DisplayMode::Overlay;
        // What the study never touches
        s.app_visibility = crate::types::AppVisibility::TopBarOnly;
        s.keyboard_shortcuts.start = Some(crate::settings::KeyboardShortcut::new(
            crate::settings::KBD_MOD_CTRL | crate::settings::KBD_MOD_SHIFT,
            "KeyG",
        ));
        s.settings_window_x       = Some(10);
        s.settings_window_y       = Some(20);
        s.settings_window_height  = Some(30);
        s.settings_window_screen  = Some("Right".to_string());
        s.animation_window_x      = Some(40);
        s.animation_window_y      = Some(50);
        s.animation_window_width  = Some(800);
        s.animation_window_height = Some(600);
        s.animation_window_screen = Some("Left".to_string());
        s.is_animating = false;
        s.is_paused    = true;
        s
    }

    /// The `Settings` fields whose value differs, by the TOML each one writes
    fn changed_fields(before: &Settings, after: &Settings) -> std::collections::BTreeSet<String> {
        let table = |s: &Settings| match toml::Value::try_from(s).expect("serialise") {
            toml::Value::Table(t) => t,
            other => panic!("settings are a table, got {other:?}"),
        };
        let (a, b) = (table(before), table(after));
        a.keys().chain(b.keys()).filter(|k| a.get(*k) != b.get(*k)).cloned().collect()
    }

    fn names(list: &[&str]) -> std::collections::BTreeSet<String> {
        list.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn apply_changes_exactly_the_twenty_locked_fields() {
        let before = participant_settings();

        let mut after = before.clone();
        fixtures::everything().apply(&mut after);

        assert_eq!(
            changed_fields(&before, &after),
            names(&LOCKED),
            "apply writes the eleven pace fields and the nine appearance fields and nothing else",
        );
        assert!(after.study.is_some(), "and records the lock, which settings.toml never holds");

        // The values are the file's
        assert_eq!(after.inhale_color,        [0.2, 0.4, 0.6, 1.0]);
        assert_eq!(after.exhale_color,        [0.9, 0.8, 0.1, 1.0]);
        assert_eq!(after.background_color,    [0.1, 0.1, 0.1, 0.5]);
        assert_eq!(after.overlay_opacity,     0.6);
        assert_eq!(after.shape,               AnimationShape::Circle);
        assert_eq!(after.color_fill_gradient, ColorFillGradient::Off);
        assert_eq!(after.animation_mode,      AnimationMode::Linear);
        assert_eq!(after.hold_ripple_mode,    HoldRippleMode::Stark);
        assert_eq!(after.display_mode,        DisplayMode::Window);

        // And what's left alone is what the participant had
        assert_eq!(after.app_visibility,     before.app_visibility);
        assert_eq!(after.keyboard_shortcuts, before.keyboard_shortcuts);
        assert_eq!(after.settings_window_placement(), before.settings_window_placement());
        assert_eq!(after.animation_window_placement(), before.animation_window_placement());
        assert_eq!((after.is_animating, after.is_paused), (false, true));
    }

    #[test]
    fn an_appearance_field_the_file_leaves_out_is_locked_at_the_shipped_default() {
        // Here the two-valued fields have to start off their shipped value
        let mut before = participant_settings();
        before.animation_mode = AnimationMode::Linear;
        before.display_mode   = DisplayMode::Window;
        let mut after = before.clone();
        valid().apply(&mut after);

        // The same twenty change, and the nine appearance fields go to the
        // shipped values whatever the participant's file held
        assert_eq!(changed_fields(&before, &after), names(&LOCKED));
        let shipped = Settings::default();
        assert_eq!(after.inhale_color,        shipped.inhale_color);
        assert_eq!(after.exhale_color,        shipped.exhale_color);
        assert_eq!(after.background_color,    shipped.background_color);
        assert_eq!(after.overlay_opacity,     shipped.overlay_opacity);
        assert_eq!(after.shape,               shipped.shape);
        assert_eq!(after.color_fill_gradient, shipped.color_fill_gradient);
        assert_eq!(after.animation_mode,      shipped.animation_mode);
        assert_eq!(after.hold_ripple_mode,    shipped.hold_ripple_mode);
        assert_eq!(after.display_mode,        shipped.display_mode);

        // A file that sets some of them locks the rest at the default
        let mut partly = before.clone();
        StudyConfig { overlay_opacity: Some(0.5), shape: Some(AnimationShape::Circle), ..valid() }
            .apply(&mut partly);
        assert_eq!(partly.overlay_opacity, 0.5);
        assert_eq!(partly.shape, AnimationShape::Circle);
        assert_eq!(partly.inhale_color, shipped.inhale_color);
        assert_eq!(partly.display_mode, shipped.display_mode);
    }

    /// The lines of `text` that assign any of `keys`, an array that runs
    /// over several lines included, the way a lab copies them
    fn copy_lines(text: &str, keys: &[&str]) -> String {
        let mut out = String::new();
        let mut inside_array = false;
        for line in text.lines() {
            if inside_array {
                out.push_str(line);
                out.push('\n');
                inside_array = line.trim() != "]";
            } else if keys.iter().any(|k| line.starts_with(&format!("{k} = "))) {
                out.push_str(line);
                out.push('\n');
                inside_array = line.trim_end().ends_with('[');
            }
        }
        out
    }

    #[test]
    fn a_settings_toml_appearance_block_pasted_into_a_study_file_validates_and_applies() {
        // What the app writes into settings.toml for a participant who
        // changed the nine settings, in the form it writes them
        let mut written = Settings::default();
        written.inhale_color        = [0.2, 0.4, 0.6, 1.0];
        written.exhale_color        = [0.9, 0.8, 0.1, 1.0];
        written.background_color    = [0.1, 0.1, 0.1, 0.5];
        written.overlay_opacity     = 0.6;
        written.shape               = AnimationShape::Circle;
        written.color_fill_gradient = ColorFillGradient::Off;
        written.animation_mode      = AnimationMode::Linear;
        written.hold_ripple_mode    = HoldRippleMode::Stark;
        written.display_mode        = DisplayMode::Window;
        written.app_visibility      = crate::types::AppVisibility::DockOnly;
        let settings_toml = toml::to_string_pretty(&written).expect("serialise");

        let block = copy_lines(&settings_toml, &APPEARANCE);
        for key in APPEARANCE {
            assert!(block.contains(&format!("{key} = ")), "the copied block holds {key}:\n{block}");
        }
        assert!(!block.contains("app_visibility"), "Show In isn't one of the nine");

        let dir  = tmp_dir("pasted");
        let path = write_study(&dir, &format!("{MINIMAL}{block}"));
        let cfg  = load(&path).expect("the pasted lines validate").expect("file is present");

        let mut applied = Settings::default();
        cfg.apply(&mut applied);
        assert_eq!(applied.inhale_color,        written.inhale_color);
        assert_eq!(applied.exhale_color,        written.exhale_color);
        assert_eq!(applied.background_color,    written.background_color);
        assert_eq!(applied.overlay_opacity,     written.overlay_opacity);
        assert_eq!(applied.shape,               written.shape);
        assert_eq!(applied.color_fill_gradient, written.color_fill_gradient);
        assert_eq!(applied.animation_mode,      written.animation_mode);
        assert_eq!(applied.hold_ripple_mode,    written.hold_ripple_mode);
        assert_eq!(applied.display_mode,        written.display_mode);
        assert_eq!(applied.app_visibility, Settings::default().app_visibility, "and Show In stays the participant's");

        // The same block with the colors on one line each, and with integers
        let inline = "inhale_color = [1, 0, 0, 1]\nexhale_color = [0, 1, 0, 1]\nbackground_color = [0, 0, 0, 0]\noverlay_opacity = 1\n";
        let path   = write_study(&dir, &format!("{MINIMAL}{inline}"));
        let cfg    = load(&path).expect("integers are numbers").expect("file is present");
        assert_eq!(cfg.exhale_color, Some([0.0, 1.0, 0.0, 1.0]));
        assert_eq!(cfg.overlay_opacity, Some(1.0));
    }

    #[test]
    fn a_study_file_with_every_field_set_writes_and_reads_back_the_same() {
        // The Serialize side, which the app never calls. Written fields
        // come back, and a field that's None is absent from the text
        let full = fixtures::everything();
        let text = toml::to_string(&full).expect("serialise");
        assert_eq!(toml::from_str::<StudyConfig>(&text).expect("deserialise"), full);

        let text = toml::to_string(&valid()).expect("serialise");
        for key in APPEARANCE {
            assert!(!text.contains(key), "{key} is absent when the file leaves it out: {text}");
        }
        assert_eq!(toml::from_str::<StudyConfig>(&text).expect("deserialise"), valid());
    }

    #[test]
    fn every_appearance_rule_names_its_field_and_never_its_value() {
        let cases: Vec<(StudyConfig, &str, &str)> = vec![
            (StudyConfig { inhale_color: Some([1.5321, 0.0, 0.0, 1.0]), ..valid() }, "inhale_color", "1.5321"),
            (StudyConfig { inhale_color: Some([0.0, 0.0, 0.0, -0.4321]), ..valid() }, "inhale_color", "0.4321"),
            (StudyConfig { exhale_color: Some([f32::NAN, 0.0, 0.0, 1.0]), ..valid() }, "exhale_color", "NaN"),
            (StudyConfig { exhale_color: Some([0.0, f32::INFINITY, 0.0, 1.0]), ..valid() }, "exhale_color", "inf"),
            (StudyConfig { background_color: Some([0.0, 0.0, 2.7654, 0.0]), ..valid() }, "background_color", "2.7654"),
            (StudyConfig { overlay_opacity: Some(1.2345), ..valid() }, "overlay_opacity", "1.2345"),
            (StudyConfig { overlay_opacity: Some(-0.5432), ..valid() }, "overlay_opacity", "0.5432"),
            (StudyConfig { overlay_opacity: Some(f32::NAN), ..valid() }, "overlay_opacity", "NaN"),
        ];
        for (cfg, field, value) in cases {
            let err = cfg.validate().err().unwrap_or_else(|| panic!("expected {field} to be rejected"));
            let summary = err.summary();
            assert!(summary.starts_with("study.toml, field "), "{field}: {summary}");
            assert!(summary.contains(field), "expected {field} in the summary, got {summary}");
            assert_eq!(err.field_name(), Some(field));
            assert!(!summary.contains(value), "{field}: the summary quotes {value}: {summary}");
            assert!(err.detail().contains(value), "{field}: the detail keeps {value}: {}", err.detail());
        }
        assert_eq!(
            StudyConfig { overlay_opacity: Some(1.5), ..valid() }.validate().unwrap_err().summary(),
            "study.toml, field overlay_opacity: has to be between 0 and 1",
        );
        assert_eq!(
            StudyConfig { inhale_color: Some([2.0; 4]), ..valid() }.validate().unwrap_err().summary(),
            "study.toml, field inhale_color: has to be four numbers between 0 and 1",
        );
    }

    #[test]
    fn the_edges_of_the_appearance_ranges_are_valid() {
        let cfg = StudyConfig {
            inhale_color:     Some([0.0, 0.0, 0.0, 0.0]),
            exhale_color:     Some([1.0, 1.0, 1.0, 1.0]),
            background_color: Some([0.0, 1.0, 0.0, 1.0]),
            overlay_opacity:  Some(0.0),
            ..valid()
        };
        assert_eq!(cfg.validate(), Ok(()));
        assert_eq!(StudyConfig { overlay_opacity: Some(1.0), ..valid() }.validate(), Ok(()));
    }

    #[test]
    fn the_fullscreen_shape_needs_two_different_colors() {
        let same = Some([0.5, 0.5, 0.5, 1.0]);
        let fullscreen = Some(AnimationShape::Fullscreen);

        let err = StudyConfig { shape: fullscreen, inhale_color: same, exhale_color: same, ..valid() }
            .validate()
            .expect_err("a still tint isn't a breath");
        assert_eq!(
            err.summary(),
            "study.toml, field exhale_color: has to differ from inhale_color when the shape is fullscreen",
        );
        assert!(err.detail().contains("0.5"), "the detail holds the colors: {}", err.detail());

        // Colors within the controller's tolerance count as the same
        let near = Some([0.5004, 0.5, 0.5, 1.0]);
        assert!(StudyConfig { shape: fullscreen, inhale_color: same, exhale_color: near, ..valid() }.validate().is_err());
        let apart = Some([0.502, 0.5, 0.5, 1.0]);
        assert_eq!(StudyConfig { shape: fullscreen, inhale_color: same, exhale_color: apart, ..valid() }.validate(), Ok(()));

        // The same colors under any other shape are an animation like any other
        for shape in [AnimationShape::Rectangle, AnimationShape::Circle] {
            let cfg = StudyConfig { shape: Some(shape), inhale_color: same, exhale_color: same, ..valid() };
            assert_eq!(cfg.validate(), Ok(()), "{shape:?}");
        }

        // The shipped colors differ, so the shape alone is fine
        assert_eq!(StudyConfig { shape: fullscreen, ..valid() }.validate(), Ok(()));
        // One color set to the shipped value of the other is the same trap
        let shipped_exhale = Settings::default().exhale_color;
        let err = StudyConfig { shape: fullscreen, inhale_color: Some(shipped_exhale), ..valid() }
            .validate()
            .expect_err("inhale now matches the exhale the file doesn't set");
        assert_eq!(err.field_name(), Some("exhale_color"));
        // And so is no shape at all, when the shipped shape is the fullscreen one
        let shipped = Settings::default();
        assert_eq!(
            StudyConfig { inhale_color: same, exhale_color: same, ..valid() }.validate().is_err(),
            shipped.shape == AnimationShape::Fullscreen,
        );
    }

    #[test]
    fn the_fullscreen_error_points_at_the_exhale_color_line() {
        let dir  = tmp_dir("fullscreen-line");
        let text = format!("{MINIMAL}shape = \"fullscreen\"\ninhale_color = [0.5, 0.5, 0.5, 1]\nexhale_color = [0.5, 0.5, 0.5, 1]\n");
        let err  = load(&write_study(&dir, &text)).expect_err("a still tint");
        assert_eq!(err.line(), Some(9));
        assert!(err.summary().starts_with("study.toml, line 9, field exhale_color: "), "{}", err.summary());
    }

    #[test]
    fn a_color_is_exactly_four_numbers() {
        let dir = tmp_dir("four-numbers");
        let expect = "study.toml, line 7, field inhale_color: has to be four numbers between 0 and 1";
        for (value, label) in [
            ("[1, 0, 0]", "three"),
            ("[1, 0, 0, 1, 1]", "five, the case `[f32; 4]` alone lets through"),
            ("[]", "none"),
            ("\"red\"", "a string"),
            ("[1, \"x\", 0, 1]", "a string among them"),
            ("[[1], 0, 0, 1]", "a list among them"),
            ("5", "one number"),
        ] {
            let err = load(&write_study(&dir, &format!("{MINIMAL}inhale_color = {value}\n")))
                .expect_err(label);
            assert_eq!(err.summary(), expect, "{label}");
            for quoted in ["red", "\"x\""] {
                assert!(!err.summary().contains(quoted), "{label}: {}", err.summary());
            }
        }
        // Four is fine, with integers or floats
        let cfg = load(&write_study(&dir, &format!("{MINIMAL}inhale_color = [1, 0.5, 0, 1]\n")))
            .expect("four numbers")
            .expect("file is present");
        assert_eq!(cfg.inhale_color, Some([1.0, 0.5, 0.0, 1.0]));
    }

    #[test]
    fn a_mistake_in_an_appearance_value_lists_what_is_allowed_and_never_quotes_the_value() {
        let dir = tmp_dir("appearance-parse");
        let cases = [
            ("shape = \"triangle\"", "triangle", "has to be one of fullscreen, rectangle, circle"),
            ("color_fill_gradient = \"half\"", "half", "has to be one of off, inner, on"),
            ("hold_ripple_mode = \"big\"", "big", "has to be one of off, stark, gradient"),
            ("animation_mode = \"bouncy\"", "bouncy", "has to be linear or sinusoidal"),
            ("display_mode = \"fullscreen\"", "fullscreen\"", "has to be overlay or window"),
            // A value built to look like the list that follows it
            ("shape = \"x`, expected `evil\"", "evil", "has to be one of fullscreen, rectangle, circle"),
            ("shape = \"x`, expected one of `evil\"", "evil", "has to be one of fullscreen, rectangle, circle"),
            ("shape = 3", "3", "has to be a string"),
            ("overlay_opacity = \"high\"", "high", "has to be a number"),
        ];
        for (line, value, phrase) in cases {
            let err = load(&write_study(&dir, &format!("{MINIMAL}{line}\n"))).expect_err(line);
            let summary = err.summary();
            assert!(summary.starts_with("study.toml, line 7, field "), "{line}: {summary}");
            assert!(summary.ends_with(phrase), "{line}: {summary}");
            assert!(!summary.contains(value), "{line}: the summary quotes {value}: {summary}");
        }
    }

    // ── The session log that won't open ────────────────────────────────

    #[test]
    fn a_session_log_that_cannot_open_has_a_summary_with_no_path_and_a_detail_with_no_folder() {
        // A real failure: a directory where the log file belongs
        let dir = tmp_dir("log-unavailable");
        std::fs::create_dir(dir.join(crate::session_log::log_file_name("P-SECRET-9"))).expect("make the directory");
        let io_error = crate::session_log::SessionLog::open(&dir, "P-SECRET-9").err().expect("can't open a directory");

        let err = StudyError::log_unavailable(&io_error);
        assert_eq!(err.summary(), "study.toml: the session log can't be written");
        assert_eq!(err.to_string(), err.summary());
        assert_eq!(err.field_name(), None);
        assert_eq!(err.line(), None);
        assert!(err.detail().starts_with("the session log couldn't be opened: "), "{}", err.detail());
        assert!(err.detail().len() > "the session log couldn't be opened: ".len(), "it carries the OS's reason");
        for text in [err.summary(), err.detail().to_string()] {
            assert!(!text.contains(&*dir.to_string_lossy()), "no folder: {text}");
            assert!(!text.contains("P-SECRET-9"), "no participant id: {text}");
        }

        // Any error reads the same way, whatever it says
        let permission = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(StudyError::log_unavailable(&permission).summary(), err.summary());
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
