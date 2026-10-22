//! Append-only JSON Lines log for study builds
//!
//! One flat object per line, written only when a study file locked the
//! app. The log stays on the machine until the participant presses
//! Export, which copies it somewhere they can find it. There's no
//! network code in exhale and this module adds none
//!
//! Each participant id has a live log of its own, named by
//! [`log_file_name`]. A machine handed to a new participant starts a new
//! file and never mixes two people's lines
//!
//! The emitter is hand-written. `serde_json` isn't in the dependency
//! graph and a log of flat objects with a handful of value shapes
//! doesn't justify adding it: the whole encoder is `escape_into` plus a
//! loop

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use log::warn;

use crate::{poison::MutexPoisonExt, settings::Settings};

/// Every log file name starts with this, the live log and an export alike
const LOG_FILE_PREFIX: &str = "exhale-log-";

/// Longest participant id, in characters, that a log file name carries
///
/// A longer id can push the name past the file system's limit once the
/// prefix, the timestamp and the extension are added. A study file with
/// a longer id is rejected at load, and `sanitise` cuts to this as
/// well so a name never depends on that check
pub const MAX_PARTICIPANT_ID_CHARS: usize = 64;

/// How many names an export tries before it gives up, `-2` up to this
const MAX_NAME_TRIES: u32 = 1000;

/// One field value in a log line
///
/// `Int` and `Num` are separate so a cycle count prints as the integer
/// `3`, and so a duration keeps its decimals. `F32` and `F32s` carry the
/// settings stored as `f32` (opacity and the color components). Widened
/// to `f64`, an opacity of 0.7 would print as `0.699999988079071`, and
/// these print `0.7`, the digits a lab typed into the study file
#[derive(Clone, Copy, Debug)]
pub enum Value<'a> {
    Str(&'a str),
    Num(f64),
    F32(f32),
    /// A JSON array of numbers, such as the four components of a color
    F32s(&'a [f32]),
    Int(i64),
    Bool(bool),
}

/// Append `s` to `out` as a quoted JSON string
fn escape_into(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Build one log line with no trailing newline
///
/// Shape: `{"t":<unix_ms>,"mono_ms":<u64>,"event":"<event>",<fields>}`.
/// Fields keep the order they were given so a log reads the same way
/// every time and a diff between two runs stays small
pub fn json_line(unix_ms: u64, mono_ms: u64, event: &str, fields: &[(&str, Value)]) -> String {
    let mut out = String::with_capacity(96 + fields.len() * 24);
    out.push_str("{\"t\":");
    out.push_str(&unix_ms.to_string());
    out.push_str(",\"mono_ms\":");
    out.push_str(&mono_ms.to_string());
    out.push_str(",\"event\":");
    escape_into(&mut out, event);
    for (key, value) in fields {
        out.push(',');
        escape_into(&mut out, key);
        out.push(':');
        match value {
            Value::Str(s)  => escape_into(&mut out, s),
            // `{}` prints 5 for 5.0 and 2.5 for 2.5, both valid JSON
            // numbers. A non-finite value has no JSON spelling at all,
            // so it becomes null, because a bare `NaN` would make the
            // whole file unparseable
            Value::Num(n)  => {
                if n.is_finite() { out.push_str(&n.to_string()); } else { out.push_str("null"); }
            }
            Value::F32(n)  => push_f32(&mut out, *n),
            Value::F32s(list) => {
                out.push('[');
                for (i, n) in list.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    push_f32(&mut out, *n);
                }
                out.push(']');
            }
            Value::Int(i)  => out.push_str(&i.to_string()),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        }
    }
    out.push('}');
    out
}

/// Append an `f32` as a JSON number, or `null` when it has no spelling
fn push_f32(out: &mut String, n: f32) {
    if n.is_finite() {
        out.push_str(&n.to_string());
    } else {
        out.push_str("null");
    }
}

/// Milliseconds since the Unix epoch, 0 if the clock is before it
fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Write one line and its newline with a single `write_all`
///
/// Two calls (the line, then the newline) leave a line with no newline
/// behind when the second one fails, and the next record runs onto it.
/// `torn` remembers that the last write failed, so the file may end in
/// the middle of a line, and the next write starts on a fresh one
fn write_line<W: Write>(out: &mut W, torn: &mut bool, line: &str) -> io::Result<()> {
    let mut bytes = String::with_capacity(line.len() + 2);
    if *torn {
        bytes.push('\n');
    }
    bytes.push_str(line);
    bytes.push('\n');
    let result = out.write_all(bytes.as_bytes());
    *torn = result.is_err();
    result
}

/// Start a new line when the file ends in the middle of one
///
/// A process killed mid-write, or a full disk, leaves the last line
/// without its newline, and the next append would run onto it. The
/// newline keeps the next line whole and leaves the cut one as the only
/// bad line in the file
fn end_open_line(file: &mut File) -> io::Result<()> {
    if file.metadata()?.len() == 0 {
        return Ok(());
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    if last[0] != b'\n' {
        file.write_all(b"\n")?;
    }
    Ok(())
}

/// Empty the file at `path` in place
///
/// Truncates through a handle opened for writing. A handle opened for
/// appending can't do it on Windows, where std opens it without the
/// right to write at an offset and `set_len` fails with "Access is
/// denied". A handle opened for appending elsewhere keeps appending,
/// now at the new end
pub fn truncate_file(path: &Path) -> io::Result<()> {
    OpenOptions::new().write(true).open(path)?.set_len(0)
}

/// Everything the log's mutex guards
struct State {
    file: File,
    /// Lines appended since open. A change in it means the log grew
    version: u64,
    /// `version` when the last successful export finished, with the
    /// export's own line counted in. `None` before the first export,
    /// and again after a Clear
    exported: Option<u64>,
    /// The last write failed, so the file may end in the middle of a line
    torn: bool,
}

/// Why [`SessionLog::clear`] left the log alone
#[derive(Debug)]
pub enum ClearError {
    /// The log holds lines that no export contains, because none has
    /// run yet or lines arrived after the last one
    Unexported,
    /// The file couldn't be emptied, or the header couldn't be written
    Io(io::Error),
}

impl std::fmt::Display for ClearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClearError::Unexported => {
                f.write_str("the log holds lines that no export contains. Export it first")
            }
            ClearError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ClearError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClearError::Unexported => None,
            ClearError::Io(e)      => Some(e),
        }
    }
}

impl From<io::Error> for ClearError {
    fn from(e: io::Error) -> Self {
        ClearError::Io(e)
    }
}

/// An open session log
///
/// Every write is best-effort. A full disk or a revoked permission
/// costs the lab a log line, and it must never cost the participant
/// their session, so failures log a warning and the app carries on
pub struct SessionLog {
    path:    PathBuf,
    state:   Mutex<State>,
    started: Instant,
}

impl SessionLog {
    /// Open (or create) the live log for `participant_id` in `dir`
    ///
    /// The file is named by [`log_file_name`]. A file that ends in the
    /// middle of a line gets its newline first, so appending never runs
    /// onto a line a killed process cut short
    pub fn open(dir: &Path, participant_id: &str) -> io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(log_file_name(participant_id));
        // Readable as well, to look at the last byte. Appends still go
        // to the end whatever the read position is
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&path)?;
        end_open_line(&mut file)?;
        Ok(Self {
            path,
            state: Mutex::new(State { file, version: 0, exported: None, torn: false }),
            started: Instant::now(),
        })
    }

    /// Where the log lives
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one event, under the lock the caller already holds
    fn append(&self, state: &mut State, event: &str, fields: &[(&str, Value)]) -> io::Result<()> {
        let mono_ms = self.started.elapsed().as_millis() as u64;
        let line = json_line(unix_ms_now(), mono_ms, event, fields);
        write_line(&mut state.file, &mut state.torn, &line)?;
        state.version += 1;
        Ok(())
    }

    /// Append one event
    pub fn record(&self, event: &str, fields: &[(&str, Value)]) {
        // A poisoned lock means another thread panicked mid-write. The
        // file handle itself is still usable, so recover the guard
        // and keep the panic out of the controller thread
        let mut state = self.state.lock_or_recover();
        if let Err(e) = self.append(&mut state, event, fields) {
            warn!("session log write failed: {e}");
        }
    }

    /// The log's bytes as they stand, and its version, read under the lock
    fn snapshot(&self) -> io::Result<(Vec<u8>, u64)> {
        let mut state = self.state.lock_or_recover();
        state.file.flush()?;
        let bytes = std::fs::read(&self.path)?;
        Ok((bytes, state.version))
    }

    /// Record that an export of the log as it stood at `seen` landed
    /// under `name`
    ///
    /// The export's own line counts as covered. A line that arrived
    /// after the snapshot is in no copy, so it keeps Clear withheld
    fn finish_export(&self, seen: u64, name: &str) {
        let mut state = self.state.lock_or_recover();
        let nothing_new = state.version == seen;
        if let Err(e) = self.append(&mut state, "export", &[("path", Value::Str(name))]) {
            warn!("session log write failed: {e}");
        }
        state.exported = Some(if nothing_new { state.version } else { seen });
    }

    /// Copy the log as it stands into the first of `dirs` that takes it,
    /// returning the new path
    ///
    /// The copy is named after the participant and the wall clock so a
    /// lab receiving several files can tell them apart, and a name
    /// that's taken gets `-2`, `-3` and so on. A folder where the copy
    /// fails is skipped for the next one, and the error of the first
    /// folder comes back only when every one failed. An `export` event
    /// goes into the live log afterwards, so the copy never contains a
    /// record of itself. The event holds the file name and no folder,
    /// which keeps the account's home folder name out of the log
    pub fn export_to(&self, dirs: &[PathBuf], participant_id: &str) -> io::Result<PathBuf> {
        self.export_with(dirs, participant_id, copy_unique)
    }

    /// [`Self::export_to`] with the copy step passed in
    ///
    /// The log's lock is held only to read the file. The copy runs after
    /// it's released, because a destination that stalls (a network
    /// share, a sleeping disk) would otherwise hold up `record`, and with
    /// it the controller thread that logs every phase end
    fn export_with(
        &self,
        dirs:           &[PathBuf],
        participant_id: &str,
        copy:           impl Fn(&Path, &str, &[u8]) -> io::Result<PathBuf>,
    ) -> io::Result<PathBuf> {
        let (bytes, seen) = self.snapshot()?;
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let stem = format!("{LOG_FILE_PREFIX}{}-{secs}", sanitise(participant_id));

        let mut first_error = None;
        for dir in dirs {
            match copy(dir, &stem, &bytes) {
                Ok(target) => {
                    let name = target
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    self.finish_export(seen, &name);
                    return Ok(target);
                }
                Err(e) => {
                    warn!("export to one folder failed: {e}");
                    first_error.get_or_insert(e);
                }
            }
        }
        Err(first_error.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "no folder to export to")
        }))
    }

    /// Whether [`Self::clear`] would go ahead
    ///
    /// True only while the log is exactly what the last successful
    /// export copied, the export's own line included. Any line written
    /// since, a phase end for one, withholds it, so nobody can wipe
    /// lines that no export holds
    pub fn clear_allowed(&self) -> bool {
        let state = self.state.lock_or_recover();
        state.exported == Some(state.version)
    }

    /// Empty the log in place and write `header` as its first line,
    /// leaving it open for further appends
    ///
    /// The header goes in as a `log_cleared` event, so a cleared log
    /// still says which pace produced the lines below it. Emptying and
    /// the header share one lock, so no line from another thread can
    /// come first. Refuses with [`ClearError::Unexported`] unless
    /// [`Self::clear_allowed`]
    pub fn clear(&self, header: &[(&str, Value)]) -> Result<(), ClearError> {
        self.clear_with(header, || {})
    }

    /// [`Self::clear`] with a step that runs between the truncation and
    /// the header, under the lock
    fn clear_with(&self, header: &[(&str, Value)], between: impl FnOnce()) -> Result<(), ClearError> {
        let mut state = self.state.lock_or_recover();
        if state.exported != Some(state.version) {
            return Err(ClearError::Unexported);
        }
        state.file.flush()?;
        truncate_file(&self.path)?;
        state.torn     = false;
        state.exported = None;
        between();
        self.append(&mut state, "log_cleared", header)?;
        Ok(())
    }
}

/// Write `bytes` to a new file in `dir`, named `<stem>.jsonl`, or
/// `<stem>-2.jsonl`, `<stem>-3.jsonl` and so on while that name is taken
///
/// `create_new` fails when the name exists, so two exports in the same
/// second never replace each other's file. A copy that fails halfway is
/// removed, so no truncated export is left behind for someone to send
fn copy_unique(dir: &Path, stem: &str, bytes: &[u8]) -> io::Result<PathBuf> {
    for n in 1..=MAX_NAME_TRIES {
        let name = if n == 1 { format!("{stem}.jsonl") } else { format!("{stem}-{n}.jsonl") };
        let target = dir.join(name);
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&target) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        return match file.write_all(bytes) {
            Ok(()) => Ok(target),
            Err(e) => {
                drop(file);
                let _ = std::fs::remove_file(&target);
                Err(e)
            }
        };
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "too many exports with the same name"))
}

/// Keep `[A-Za-z0-9_-]`, replace everything else with `_`
///
/// A participant id comes from a file a lab wrote, so it can hold a
/// path separator or a quote. This runs before it reaches a filename.
/// Surrounding whitespace is dropped first and the result is cut to
/// [`MAX_PARTICIPANT_ID_CHARS`], the trim and the cap the study file's
/// validation applies
fn sanitise(s: &str) -> String {
    let cleaned: String = s
        .trim()
        .chars()
        .take(MAX_PARTICIPANT_ID_CHARS)
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    if cleaned.is_empty() { "participant".to_string() } else { cleaned }
}

/// Name of the live log for `participant_id`, written beside `settings.toml`
///
/// The id goes through the same cleaning an export's name gets. Two ids
/// that clean to the same text (`P/07` and `P_07`) share one file, and
/// the `participant_id` in each `app_start` line tells them apart
pub fn log_file_name(participant_id: &str) -> String {
    format!("{LOG_FILE_PREFIX}{}.jsonl", sanitise(participant_id))
}

/// The fields of an `app_start` or `log_cleared` line
///
/// Everything a reader of the log needs to know what produced the lines
/// under it: the build, the machine's OS, who the lab assigned, the
/// eleven numbers the study file locked, and the nine appearance and
/// display settings it locked. Written at `app_start` and again after
/// Clear, so a cleared log is still self-describing. `version` is the
/// app's own, which the caller knows and this crate doesn't
///
/// The order is fixed, and the researchers' guide lists it:
///
/// 1. `version`, `os`, `participant_id`, `condition`
/// 2. `inhale_duration`, `post_inhale_hold_duration`, `exhale_duration`,
///    `post_exhale_hold_duration`, `drift`
/// 3. `randomized_timing_inhale`, `randomized_timing_post_inhale_hold`,
///    `randomized_timing_exhale`, `randomized_timing_post_exhale_hold`
/// 4. `auto_stop_minutes`, `reminder_interval_minutes`
/// 5. `inhale_color`, `exhale_color`, `background_color` (arrays of four
///    numbers), `overlay_opacity` (a number), then `shape`,
///    `color_fill_gradient`, `animation_mode`, `hold_ripple_mode` and
///    `display_mode`, each the snake_case string settings.toml uses
///
/// Outside a study the participant id and the condition are empty
pub fn header_fields<'a>(version: &'a str, settings: &'a Settings) -> Vec<(&'static str, Value<'a>)> {
    let (participant_id, condition) = match &settings.study {
        Some(s) => (s.participant_id.as_str(), s.condition.as_str()),
        None    => ("", ""),
    };
    vec![
        ("version",        Value::Str(version)),
        ("os",             Value::Str(std::env::consts::OS)),
        ("participant_id", Value::Str(participant_id)),
        ("condition",      Value::Str(condition)),
        ("inhale_duration",           Value::Num(settings.inhale_duration)),
        ("post_inhale_hold_duration", Value::Num(settings.post_inhale_hold_duration)),
        ("exhale_duration",           Value::Num(settings.exhale_duration)),
        ("post_exhale_hold_duration", Value::Num(settings.post_exhale_hold_duration)),
        ("drift",                     Value::Num(settings.drift)),
        ("randomized_timing_inhale",           Value::Num(settings.randomized_timing_inhale)),
        ("randomized_timing_post_inhale_hold", Value::Num(settings.randomized_timing_post_inhale_hold)),
        ("randomized_timing_exhale",           Value::Num(settings.randomized_timing_exhale)),
        ("randomized_timing_post_exhale_hold", Value::Num(settings.randomized_timing_post_exhale_hold)),
        ("auto_stop_minutes",         Value::Num(settings.auto_stop_minutes)),
        ("reminder_interval_minutes", Value::Num(settings.reminder_interval_minutes)),
        ("inhale_color",        Value::F32s(&settings.inhale_color)),
        ("exhale_color",        Value::F32s(&settings.exhale_color)),
        ("background_color",    Value::F32s(&settings.background_color)),
        ("overlay_opacity",     Value::F32(settings.overlay_opacity)),
        ("shape",               Value::Str(settings.shape.name())),
        ("color_fill_gradient", Value::Str(settings.color_fill_gradient.name())),
        ("animation_mode",      Value::Str(settings.animation_mode.name())),
        ("hold_ripple_mode",    Value::Str(settings.hold_ripple_mode.name())),
        ("display_mode",        Value::Str(settings.display_mode.name())),
    ]
}

/// Folders an export may go to, best first: Desktop, Documents,
/// Downloads, then the home directory
///
/// Only folders that exist are listed. The list is empty only when the
/// platform can't name a home directory at all, which is the same
/// condition that leaves the app without a config directory
pub fn export_dirs() -> Vec<PathBuf> {
    let Some(dirs) = directories::UserDirs::new() else {
        return Vec::new();
    };
    existing_dirs(&[
        dirs.desktop_dir(),
        dirs.document_dir(),
        dirs.download_dir(),
        Some(dirs.home_dir()),
    ])
}

/// The candidates that are folders on disk, in the order given
///
/// `directories` names Desktop even on a machine that has none
fn existing_dirs(candidates: &[Option<&Path>]) -> Vec<PathBuf> {
    candidates
        .iter()
        .flatten()
        .filter(|d| d.is_dir())
        .map(|d| d.to_path_buf())
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "exhale-log-test-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    impl Value<'_> {
        /// One value as it prints, for a test that compares spellings
        fn as_json(&self) -> String {
            let line = json_line(0, 0, "e", &[("v", *self)]);
            line.strip_prefix("{\"t\":0,\"mono_ms\":0,\"event\":\"e\",\"v\":")
                .and_then(|rest| rest.strip_suffix('}'))
                .expect("one field")
                .to_string()
        }
    }

    fn open(dir: &Path) -> SessionLog {
        SessionLog::open(dir, "P07").expect("open")
    }

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).expect("read back")
    }

    #[test]
    fn a_line_with_no_fields_is_the_header_only() {
        assert_eq!(
            json_line(1758400000123, 3, "app_stop", &[]),
            r#"{"t":1758400000123,"mono_ms":3,"event":"app_stop"}"#,
        );
    }

    #[test]
    fn every_value_variant_has_its_json_spelling() {
        let line = json_line(1, 2, "e", &[
            ("s", Value::Str("hi")),
            ("whole", Value::Num(5.0)),
            ("frac",  Value::Num(2.5)),
            ("i", Value::Int(-7)),
            ("b", Value::Bool(true)),
            ("f", Value::Bool(false)),
        ]);
        assert_eq!(
            line,
            r#"{"t":1,"mono_ms":2,"event":"e","s":"hi","whole":5,"frac":2.5,"i":-7,"b":true,"f":false}"#,
        );
    }

    #[test]
    fn strings_are_escaped() {
        let line = json_line(0, 0, "e", &[
            ("quote",   Value::Str("a\"b")),
            ("slash",   Value::Str("a\\b")),
            ("newline", Value::Str("a\nb")),
            ("tab",     Value::Str("a\tb")),
            ("cr",      Value::Str("a\rb")),
            ("ctrl",    Value::Str("a\u{0001}b")),
        ]);
        assert_eq!(
            line,
            "{\"t\":0,\"mono_ms\":0,\"event\":\"e\",\
             \"quote\":\"a\\\"b\",\"slash\":\"a\\\\b\",\"newline\":\"a\\nb\",\
             \"tab\":\"a\\tb\",\"cr\":\"a\\rb\",\"ctrl\":\"a\\u0001b\"}",
        );
    }

    #[test]
    fn a_key_is_escaped_too() {
        let line = json_line(0, 0, "e", &[("a\"b", Value::Int(1))]);
        assert_eq!(line, r#"{"t":0,"mono_ms":0,"event":"e","a\"b":1}"#);
    }

    #[test]
    fn a_non_finite_number_is_null() {
        let line = json_line(0, 0, "e", &[
            ("nan",  Value::Num(f64::NAN)),
            ("inf",  Value::Num(f64::INFINITY)),
            ("ninf", Value::Num(f64::NEG_INFINITY)),
        ]);
        assert_eq!(
            line,
            r#"{"t":0,"mono_ms":0,"event":"e","nan":null,"inf":null,"ninf":null}"#,
        );
    }

    #[test]
    fn an_f32_prints_the_digits_that_read_back_as_the_same_f32() {
        let line = json_line(0, 0, "e", &[
            ("seven",   Value::F32(0.7)),
            ("quarter", Value::F32(0.25)),
            ("one",     Value::F32(1.0)),
            ("zero",    Value::F32(0.0)),
            ("tiny",    Value::F32(0.000_001)),
            ("nan",     Value::F32(f32::NAN)),
            ("inf",     Value::F32(f32::INFINITY)),
        ]);
        assert_eq!(
            line,
            r#"{"t":0,"mono_ms":0,"event":"e","seven":0.7,"quarter":0.25,"one":1,"zero":0,"tiny":0.000001,"nan":null,"inf":null}"#,
        );
        // Widened to f64 first, 0.7 would print as the 0.699999988079071 below
        assert_eq!(Value::Num(f64::from(0.7_f32)).as_json(), "0.699999988079071");
    }

    #[test]
    fn a_list_of_f32_is_a_json_array_of_numbers() {
        let line = json_line(0, 0, "e", &[
            ("red",    Value::F32s(&[1.0, 0.0, 0.0, 1.0])),
            ("teal",   Value::F32s(&[0.2, 0.4, 0.6, 1.0])),
            ("none",   Value::F32s(&[])),
            ("broken", Value::F32s(&[f32::NAN, 0.5, f32::NEG_INFINITY])),
            ("after",  Value::Int(5)),
        ]);
        assert_eq!(
            line,
            r#"{"t":0,"mono_ms":0,"event":"e","red":[1,0,0,1],"teal":[0.2,0.4,0.6,1],"none":[],"broken":[null,0.5,null],"after":5}"#,
        );
        // The key in front of an array is escaped like any other
        let line = json_line(0, 0, "e", &[("a\"b", Value::F32s(&[1.0])), ("c\nd", Value::F32(2.0))]);
        assert_eq!(line, "{\"t\":0,\"mono_ms\":0,\"event\":\"e\",\"a\\\"b\":[1],\"c\\nd\":2}");
    }

    // ── The header of an app_start or log_cleared line ─────────────────

    /// The 24 header fields, in the order the researchers' guide lists them
    const HEADER_ORDER: [&str; 24] = [
        "version", "os", "participant_id", "condition",
        "inhale_duration", "post_inhale_hold_duration", "exhale_duration", "post_exhale_hold_duration",
        "drift",
        "randomized_timing_inhale", "randomized_timing_post_inhale_hold",
        "randomized_timing_exhale", "randomized_timing_post_exhale_hold",
        "auto_stop_minutes", "reminder_interval_minutes",
        "inhale_color", "exhale_color", "background_color", "overlay_opacity",
        "shape", "color_fill_gradient", "animation_mode", "hold_ripple_mode", "display_mode",
    ];

    fn locked_settings() -> Settings {
        let mut s = Settings::default();
        crate::study::fixtures::everything().apply(&mut s);
        s
    }

    #[test]
    fn the_header_lists_its_fields_in_the_documented_order() {
        for settings in [locked_settings(), Settings::default()] {
            let fields = header_fields("2.0.28", &settings);
            let names: Vec<&str> = fields.iter().map(|(name, _)| *name).collect();
            assert_eq!(names, HEADER_ORDER);
        }
    }

    #[test]
    fn a_study_header_line_is_exactly_this() {
        let settings = locked_settings();
        let line = json_line(1, 2, "app_start", &header_fields("2.0.28", &settings));
        let os   = std::env::consts::OS;
        assert_eq!(
            line,
            format!(
                "{{\"t\":1,\"mono_ms\":2,\"event\":\"app_start\",\"version\":\"2.0.28\",\"os\":\"{os}\",\
                 \"participant_id\":\"P07\",\"condition\":\"B\",\
                 \"inhale_duration\":4,\"post_inhale_hold_duration\":1,\"exhale_duration\":6,\
                 \"post_exhale_hold_duration\":2,\"drift\":1.002,\
                 \"randomized_timing_inhale\":0.1,\"randomized_timing_post_inhale_hold\":0.2,\
                 \"randomized_timing_exhale\":0.3,\"randomized_timing_post_exhale_hold\":0.4,\
                 \"auto_stop_minutes\":30,\"reminder_interval_minutes\":15,\
                 \"inhale_color\":[0.2,0.4,0.6,1],\"exhale_color\":[0.9,0.8,0.1,1],\
                 \"background_color\":[0.1,0.1,0.1,0.5],\"overlay_opacity\":0.6,\
                 \"shape\":\"circle\",\"color_fill_gradient\":\"off\",\"animation_mode\":\"linear\",\
                 \"hold_ripple_mode\":\"stark\",\"display_mode\":\"window\"}}"
            ),
        );
    }

    #[test]
    fn a_header_outside_a_study_has_no_id_and_the_shipped_look() {
        let settings = Settings::default();
        let line = json_line(0, 0, "app_start", &header_fields("2.0.28", &settings));
        assert!(line.contains("\"participant_id\":\"\",\"condition\":\"\","), "{line}");
        assert!(
            line.ends_with(
                "\"inhale_color\":[1,0,0,1],\"exhale_color\":[0,0,1,1],\"background_color\":[0,0,0,0],\
                 \"overlay_opacity\":0.25,\"shape\":\"rectangle\",\"color_fill_gradient\":\"on\",\
                 \"animation_mode\":\"sinusoidal\",\"hold_ripple_mode\":\"gradient\",\
                 \"display_mode\":\"overlay\"}"
            ),
            "{line}",
        );
    }

    #[test]
    fn log_cleared_carries_the_same_header_as_app_start() {
        let dir = tmp_dir("header-twice");
        let out = tmp_dir("header-twice-out");
        let settings = locked_settings();
        let log = open(&dir);
        log.record("app_start", &header_fields("2.0.28", &settings));
        log.export_to(std::slice::from_ref(&out), "P07").expect("export");
        log.clear(&header_fields("2.0.28", &settings)).expect("clear");

        let first = read(log.path());
        let cleared = first.lines().next().expect("the header line");
        assert!(cleared.contains(r#""event":"log_cleared""#), "{cleared}");
        // Past `event`, the two lines are the same text
        let tail = |line: &str| line.split_once("\"event\":").map(|(_, t)| t.split_once(',').map(|(_, rest)| rest.to_string())).unwrap().unwrap();
        let exported = read(&std::fs::read_dir(&out).expect("list").next().expect("a copy").expect("entry").path());
        assert_eq!(tail(cleared), tail(exported.lines().next().expect("app_start")));
        assert!(cleared.contains(r#""inhale_color":[0.2,0.4,0.6,1]"#), "{cleared}");
    }

    // ── The live log's file ────────────────────────────────────────────

    #[test]
    fn records_append_one_line_each() {
        let dir = tmp_dir("append");
        let log = open(&dir);
        log.record("app_start", &[("version", Value::Str("2.0.22"))]);
        log.record("app_stop", &[]);

        let text = read(log.path());
        assert!(text.ends_with('\n'), "every line ends with a newline");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(r#""event":"app_start""#), "{}", lines[0]);
        assert!(lines[0].contains(r#""version":"2.0.22""#), "{}", lines[0]);
        assert!(lines[1].contains(r#""event":"app_stop""#), "{}", lines[1]);
    }

    #[test]
    fn reopening_appends_rather_than_truncating() {
        let dir = tmp_dir("reopen");
        open(&dir).record("app_start", &[]);
        let log = open(&dir);
        log.record("app_stop", &[]);
        assert_eq!(read(log.path()).lines().count(), 2);
    }

    #[test]
    fn each_participant_id_has_a_log_of_its_own() {
        let dir = tmp_dir("per-id");
        let one = SessionLog::open(&dir, "P07").expect("open P07");
        let two = SessionLog::open(&dir, "P08").expect("open P08");
        one.record("app_start", &[]);
        two.record("app_start", &[]);
        two.record("app_stop", &[]);

        assert_eq!(one.path().file_name().and_then(|n| n.to_str()), Some("exhale-log-P07.jsonl"));
        assert_eq!(two.path().file_name().and_then(|n| n.to_str()), Some("exhale-log-P08.jsonl"));
        assert_eq!(read(one.path()).lines().count(), 1);
        assert_eq!(read(two.path()).lines().count(), 2);
    }

    #[test]
    fn the_name_function_is_what_open_uses() {
        let dir = tmp_dir("name");
        for id in ["P07", "P 07/../x", "  P07  ", ""] {
            let log = SessionLog::open(&dir, id).expect("open");
            assert_eq!(
                log.path().file_name().and_then(|n| n.to_str()),
                Some(log_file_name(id).as_str()),
                "id {id:?}",
            );
        }
        assert_eq!(log_file_name("P07"), "exhale-log-P07.jsonl");
        assert_eq!(log_file_name("P 07/../x"), "exhale-log-P_07____x.jsonl");
    }

    #[test]
    fn a_long_or_padded_id_is_trimmed_and_capped_in_the_name() {
        let long = "a".repeat(500);
        let name = log_file_name(&long);
        assert_eq!(name, format!("exhale-log-{}.jsonl", "a".repeat(MAX_PARTICIPANT_ID_CHARS)));

        // Whitespace around an id of exactly the cap doesn't cost it characters
        let padded = format!("   {}   ", "b".repeat(MAX_PARTICIPANT_ID_CHARS));
        assert_eq!(log_file_name(&padded), format!("exhale-log-{}.jsonl", "b".repeat(MAX_PARTICIPANT_ID_CHARS)));
        assert_eq!(log_file_name("  P07  "), "exhale-log-P07.jsonl");

        // An export's name for the longest id and a late timestamp stays
        // far below the 255 bytes most file systems allow
        let longest = format!("{LOG_FILE_PREFIX}{}-{}-{MAX_NAME_TRIES}.jsonl", "a".repeat(MAX_PARTICIPANT_ID_CHARS), u64::MAX);
        assert!(longest.len() < 128, "{} bytes", longest.len());

        // Characters count, not bytes: 64 emoji clean to 64 underscores
        let emoji = "\u{1F600}".repeat(MAX_PARTICIPANT_ID_CHARS + 5);
        assert_eq!(sanitise(&emoji), "_".repeat(MAX_PARTICIPANT_ID_CHARS));
    }

    #[test]
    fn an_empty_participant_id_still_makes_a_filename() {
        assert_eq!(sanitise(""), "participant");
        assert_eq!(sanitise("   "), "participant");
        assert_eq!(sanitise("///"), "___");
        assert_eq!(sanitise("P-07_b"), "P-07_b");
    }

    #[test]
    fn a_log_that_cannot_open_says_so() {
        let dir = tmp_dir("cannot-open");
        // A directory where the log file belongs
        std::fs::create_dir(dir.join(log_file_name("P07"))).expect("make the directory");
        assert!(SessionLog::open(&dir, "P07").is_err());
    }

    // ── One write, and a newline repaired on open ──────────────────────

    /// Counts the `write` calls it gets and keeps the bytes
    #[derive(Default)]
    struct CountingWriter {
        calls: usize,
        bytes: Vec<u8>,
        fail:  bool,
    }

    impl Write for CountingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.fail {
                return Err(io::Error::other("disk full"));
            }
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    #[test]
    fn a_record_is_one_write_that_ends_in_a_newline() {
        let mut out   = CountingWriter::default();
        let mut torn  = false;
        write_line(&mut out, &mut torn, r#"{"event":"x"}"#).expect("write");
        assert_eq!(out.calls, 1, "the line and its newline go out together");
        assert_eq!(out.bytes, b"{\"event\":\"x\"}\n");
        assert!(!torn);
    }

    #[test]
    fn a_failed_write_cannot_run_the_next_line_on() {
        let mut out  = CountingWriter { fail: true, ..CountingWriter::default() };
        let mut torn = false;
        assert!(write_line(&mut out, &mut torn, "{\"a\":1}").is_err());
        assert!(torn, "a failed write may have left half a line");

        out.fail = false;
        write_line(&mut out, &mut torn, "{\"b\":2}").expect("write");
        assert_eq!(out.bytes, b"\n{\"b\":2}\n", "the next line starts on a fresh one");
        assert!(!torn);

        write_line(&mut out, &mut torn, "{\"c\":3}").expect("write");
        assert!(out.bytes.ends_with(b"{\"b\":2}\n{\"c\":3}\n"), "and later lines are plain again");
    }

    #[test]
    fn open_ends_a_line_that_a_killed_write_left_open() {
        let dir  = tmp_dir("repair");
        let path = dir.join(log_file_name("P07"));
        std::fs::write(&path, "{\"t\":1,\"event\":\"a\"}\n{\"t\":2,\"event\":\"cut").expect("seed");

        let log = open(&dir);
        log.record("app_start", &[]);

        let text = read(&path);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "{text}");
        assert_eq!(lines[1], "{\"t\":2,\"event\":\"cut", "the cut line stays as it was");
        assert!(lines[2].starts_with("{\"t\":") && lines[2].contains("\"app_start\""), "{}", lines[2]);
    }

    #[test]
    fn open_adds_no_newline_to_a_file_that_needs_none() {
        let dir  = tmp_dir("no-repair");
        let path = dir.join(log_file_name("P07"));

        // A new file stays empty until something is written
        let log = open(&dir);
        assert_eq!(read(&path), "");
        drop(log);

        // A file that ends in a newline gets no blank line
        std::fs::write(&path, "{\"t\":1}\n").expect("seed");
        let log = open(&dir);
        log.record("app_stop", &[]);
        assert_eq!(read(&path).lines().count(), 2);
        assert!(!read(&path).contains("\n\n"));
    }

    // ── Export ─────────────────────────────────────────────────────────

    #[test]
    fn export_copies_the_log_and_sanitises_the_name() {
        let dir = tmp_dir("export");
        let out = tmp_dir("export-out");
        let log = SessionLog::open(&dir, "P 07/../x").expect("open");
        log.record("app_start", &[]);
        let before = read(log.path());

        let copied = log.export_to(std::slice::from_ref(&out), "P 07/../x").expect("export");
        let name = copied.file_name().expect("a filename").to_string_lossy().into_owned();
        assert!(name.starts_with("exhale-log-P_07____x-"), "{name}");
        assert!(name.ends_with(".jsonl"), "{name}");
        assert_eq!(copied.parent(), Some(out.as_path()));
        assert_eq!(read(&copied), before, "the copy is the log as it stood, without the export line");

        let after = read(log.path());
        assert_eq!(after.lines().count(), 2, "the export event lands in the live log");
        let export_line = after.lines().nth(1).expect("second line");
        assert!(export_line.contains(r#""event":"export""#));
        assert!(
            export_line.ends_with(&format!(r#""path":"{name}"}}"#)),
            "the event's path is the file name and nothing else: {export_line}",
        );
        assert!(
            !export_line.contains(&*out.to_string_lossy()),
            "no folder in the event: {export_line}",
        );
    }

    #[test]
    fn the_export_event_never_holds_a_folder_or_a_home() {
        let dir  = tmp_dir("export-path");
        let out  = tmp_dir("export-path-out").join("OneDrive - Some Lab").join("Desktop");
        std::fs::create_dir_all(&out).expect("make the nested folder");
        let log  = open(&dir);
        log.record("app_start", &[]);
        log.export_to(&[out], "P07").expect("export");

        let text = read(log.path());
        let line = text.lines().last().expect("the export line");
        assert!(!line.contains("OneDrive"), "{line}");
        assert!(!line.contains("Desktop"), "{line}");
        assert!(!line.contains('/') && !line.contains("\\\\") && !line.contains('~'), "{line}");
    }

    #[test]
    fn exports_never_replace_one_another() {
        let dir = tmp_dir("twice");
        let out = tmp_dir("twice-out");
        let log = open(&dir);
        log.record("app_start", &[]);

        let first  = log.export_to(std::slice::from_ref(&out), "P07").expect("first export");
        log.record("animation_start", &[]);
        let second = log.export_to(std::slice::from_ref(&out), "P07").expect("second export");
        let third  = log.export_to(std::slice::from_ref(&out), "P07").expect("third export");

        assert_ne!(first, second);
        assert_ne!(second, third);
        let names: Vec<String> = [&first, &second, &third]
            .iter()
            .map(|p| p.file_name().expect("name").to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().all(|n| n.starts_with("exhale-log-P07-") && n.ends_with(".jsonl")), "{names:?}");
        assert_eq!(std::fs::read_dir(&out).expect("list").count(), 3, "no export replaced another: {names:?}");
        assert_eq!(read(&first).lines().count(), 1);
        assert_eq!(read(&second).lines().count(), 3, "the second copy holds what the first one's export line added");
    }

    #[test]
    fn copy_unique_counts_up_from_two() {
        let out = tmp_dir("unique");
        let stem = "exhale-log-P07-1758400420";
        let paths: Vec<PathBuf> = (0..4)
            .map(|i| copy_unique(&out, stem, format!("copy {i}").as_bytes()).expect("copy"))
            .collect();
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().expect("name").to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, [
            "exhale-log-P07-1758400420.jsonl",
            "exhale-log-P07-1758400420-2.jsonl",
            "exhale-log-P07-1758400420-3.jsonl",
            "exhale-log-P07-1758400420-4.jsonl",
        ]);
        for (i, p) in paths.iter().enumerate() {
            assert_eq!(read(p), format!("copy {i}"), "each file keeps its own content");
        }
    }

    #[test]
    fn a_directory_in_the_way_moves_the_name_on() {
        let out = tmp_dir("dir-in-the-way");
        std::fs::create_dir(out.join("exhale-log-P07-1.jsonl")).expect("make the directory");
        let path = copy_unique(&out, "exhale-log-P07-1", b"x").expect("copy");
        assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("exhale-log-P07-1-2.jsonl"));
    }

    #[test]
    fn export_moves_on_when_a_folder_fails_and_reports_the_first_error_when_all_do() {
        let dir     = tmp_dir("fallback");
        let good    = tmp_dir("fallback-good");
        let missing = dir.join("no-such-folder");
        let also    = dir.join("also-missing");
        let log     = open(&dir);
        log.record("app_start", &[]);

        // The first folder can't take the copy, the second can
        let copied = log.export_to(&[missing.clone(), good.clone()], "P07").expect("export");
        assert_eq!(copied.parent(), Some(good.as_path()));
        assert!(log.clear_allowed(), "an export that landed arms Clear");

        // Every folder fails: an error, and nothing was armed
        let log2 = SessionLog::open(&dir, "P08").expect("open");
        log2.record("app_start", &[]);
        let err = log2.export_to(&[missing, also], "P08").expect_err("no folder took it");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(!log2.clear_allowed(), "a failed export arms nothing");
        assert_eq!(read(log2.path()).lines().count(), 1, "and writes no export line");
    }

    #[test]
    fn export_with_no_folder_to_go_to_is_an_error() {
        let dir = tmp_dir("no-folder");
        let log = open(&dir);
        log.record("app_start", &[]);
        let err = log.export_to(&[], "P07").expect_err("nowhere to export to");
        assert_eq!(err.to_string(), "no folder to export to");
    }

    #[test]
    fn only_folders_that_exist_are_offered() {
        let root    = tmp_dir("offered");
        let real    = root.join("real");
        let missing = root.join("missing");
        let a_file  = root.join("file.txt");
        std::fs::create_dir(&real).expect("make a folder");
        std::fs::write(&a_file, "x").expect("make a file");

        let offered = existing_dirs(&[Some(&missing), None, Some(&a_file), Some(&real)]);
        assert_eq!(offered, vec![real.clone()]);
        assert_eq!(existing_dirs(&[Some(&real), Some(&root)]), vec![real, root], "order is kept");
        assert!(existing_dirs(&[]).is_empty());
    }

    #[test]
    fn the_log_lock_is_free_while_the_copy_is_written() {
        let dir = tmp_dir("lock-free");
        let out = tmp_dir("lock-free-out");
        let log = Arc::new(open(&dir));
        log.record("app_start", &[]);

        let copied = log
            .export_with(&[out], "P07", |dest, stem, bytes| {
                // A stalled destination would sit here. Another thread
                // has to be able to record in the meantime. Asserted in
                // place, so a lock held across the copy fails the test
                // and doesn't hang it
                let (tx, rx) = mpsc::channel();
                let other = Arc::clone(&log);
                std::thread::spawn(move || {
                    other.record("phase_end", &[]);
                    let _ = tx.send(());
                });
                assert!(
                    rx.recv_timeout(Duration::from_secs(5)).is_ok(),
                    "record() waited on the log lock while the destination was written",
                );
                copy_unique(dest, stem, bytes)
            })
            .expect("export");

        assert!(read(log.path()).contains(r#""event":"phase_end""#));
        assert!(
            !read(&copied).contains("phase_end"),
            "the copy is the log as it was when it was read",
        );
    }

    // ── Clear ──────────────────────────────────────────────────────────

    fn header() -> [(&'static str, Value<'static>); 2] {
        [("participant_id", Value::Str("P07")), ("condition", Value::Str("B"))]
    }

    #[test]
    fn clear_empties_the_file_and_writes_the_header_first() {
        let dir = tmp_dir("clear");
        let out = tmp_dir("clear-out");
        let log = open(&dir);
        log.record("app_start", &[]);
        log.record("animation_start", &[]);
        log.export_to(&[out], "P07").expect("export");
        assert!(log.clear_allowed());

        log.clear(&header()).expect("clear");

        let text = read(log.path());
        assert_eq!(text.lines().count(), 1, "{text}");
        assert!(text.contains(r#""event":"log_cleared""#), "{text}");
        assert!(text.contains(r#""participant_id":"P07""#), "{text}");

        // Later records still append
        log.record("animation_stop", &[]);
        assert_eq!(read(log.path()).lines().count(), 2);
    }

    #[test]
    fn clear_refuses_a_log_nobody_exported() {
        let dir = tmp_dir("clear-never");
        let log = open(&dir);
        log.record("app_start", &[]);
        assert!(!log.clear_allowed());

        let before = read(log.path());
        let err = log.clear(&header()).expect_err("never exported");
        assert!(matches!(err, ClearError::Unexported), "{err}");
        assert_eq!(read(log.path()), before, "a refused clear touches nothing");
    }

    #[test]
    fn a_line_after_the_export_withholds_clear() {
        let dir = tmp_dir("clear-grew");
        let out = tmp_dir("clear-grew-out");
        let log = open(&dir);
        log.record("app_start", &[]);
        log.export_to(std::slice::from_ref(&out), "P07").expect("export");
        assert!(log.clear_allowed());

        // The breath goes on and a phase ends
        log.record("phase_end", &[("phase", Value::Str("inhale"))]);
        assert!(!log.clear_allowed(), "lines arrived since the export");
        let before = read(log.path());
        let err = log.clear(&header()).expect_err("the log grew");
        assert!(matches!(err, ClearError::Unexported), "{err}");
        assert_eq!(read(log.path()), before, "so every line is still there");

        // Exporting again covers them and gives Clear back
        log.export_to(&[out], "P07").expect("second export");
        assert!(log.clear_allowed());
        log.clear(&header()).expect("clear");
    }

    #[test]
    fn a_line_between_the_read_and_the_export_event_withholds_clear() {
        let dir = tmp_dir("clear-race");
        let log = open(&dir);
        log.record("app_start", &[]);

        let (_bytes, seen) = log.snapshot().expect("snapshot");
        // A phase ends while the copy is being written
        log.record("phase_end", &[]);
        log.finish_export(seen, "exhale-log-P07-1.jsonl");

        assert!(!log.clear_allowed(), "that line is in no copy");
    }

    #[test]
    fn clear_needs_a_fresh_export_each_time() {
        let dir = tmp_dir("clear-twice");
        let out = tmp_dir("clear-twice-out");
        let log = open(&dir);
        log.record("app_start", &[]);
        log.export_to(std::slice::from_ref(&out), "P07").expect("export");
        log.clear(&header()).expect("first clear");

        assert!(!log.clear_allowed(), "the header line is in no export yet");
        assert!(matches!(log.clear(&header()), Err(ClearError::Unexported)));

        log.export_to(&[out], "P07").expect("export again");
        log.clear(&header()).expect("second clear");
    }

    #[test]
    fn nothing_from_another_thread_lands_ahead_of_the_header() {
        let dir = tmp_dir("clear-order");
        let out = tmp_dir("clear-order-out");
        let log = Arc::new(open(&dir));
        log.record("app_start", &[]);
        log.export_to(&[out], "P07").expect("export");

        // While the emptied file waits for its header, another thread
        // tries to record. It has to queue behind the lock
        let (tx, rx) = mpsc::channel();
        log.clear_with(&header(), || {
            let other = Arc::clone(&log);
            std::thread::spawn(move || {
                other.record("phase_end", &[]);
                let _ = tx.send(());
            });
            assert!(
                rx.recv_timeout(Duration::from_millis(200)).is_err(),
                "a record got in between the truncation and the header",
            );
        })
        .expect("clear");

        // Once the lock is free the line goes in, behind the header
        assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
        let text  = read(log.path());
        let first = text.lines().next().expect("a first line");
        assert!(first.contains(r#""event":"log_cleared""#), "{text}");
        assert!(text.contains(r#""event":"phase_end""#), "{text}");
    }

    #[test]
    fn truncating_works_while_an_append_handle_is_open() {
        let dir  = tmp_dir("truncate");
        let path = dir.join("exhale.log");
        // How the debug log and the session log hold their file
        let mut appender = OpenOptions::new().create(true).append(true).open(&path).expect("open");
        appender.write_all(b"old line one\nold line two\n").expect("write");

        truncate_file(&path).expect("truncate through a second handle");
        assert_eq!(read(&path), "");

        appender.write_all(b"new line\n").expect("append after the truncation");
        assert_eq!(read(&path), "new line\n", "the append handle continues at the new end");
    }

    #[test]
    fn truncating_a_missing_file_is_an_error_and_creates_nothing() {
        let dir  = tmp_dir("truncate-missing");
        let path = dir.join("nothing.log");
        assert!(truncate_file(&path).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn mono_ms_counts_from_open() {
        let dir = tmp_dir("mono");
        let log = open(&dir);
        log.record("app_start", &[]);
        let text = read(log.path());
        // Checks only that the field is there and small
        let mono = text
            .split("\"mono_ms\":").nth(1).expect("mono_ms field")
            .split(',').next().expect("its value")
            .parse::<u64>().expect("an integer");
        assert!(mono < 60_000, "mono_ms should be milliseconds since open, got {mono}");
    }
}
