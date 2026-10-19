//! Append-only JSON Lines log for study builds
//!
//! One flat object per line, written only when a study file locked the
//! app. The log stays on the machine until the participant presses
//! Export, which copies it somewhere they can find it. There's no
//! network code in exhale and this module adds none
//!
//! The emitter is hand-written. `serde_json` isn't in the dependency
//! graph and a log of flat objects with four value shapes doesn't
//! justify adding it: the whole encoder is `escape_into` plus a loop

use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use log::warn;

use crate::poison::MutexPoisonExt;

/// Name of the log file, written beside `settings.toml`
pub const LOG_FILE_NAME: &str = "exhale-log.jsonl";

/// One field value in a log line
///
/// `Int` and `Num` are separate so a cycle count prints as the integer
/// `3`, and so a duration keeps its decimals
#[derive(Clone, Copy, Debug)]
pub enum Value<'a> {
    Str(&'a str),
    Num(f64),
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
            Value::Int(i)  => out.push_str(&i.to_string()),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        }
    }
    out.push('}');
    out
}

/// Milliseconds since the Unix epoch, 0 if the clock is before it
fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// An open session log
///
/// Every write is best-effort. A full disk or a revoked permission
/// costs the lab a log line, and it must never cost the participant
/// their session, so failures log a warning and the app carries on
pub struct SessionLog {
    path:    PathBuf,
    file:    Mutex<File>,
    started: Instant,
}

impl SessionLog {
    /// Open (or create) the log at `path` for appending
    pub fn open(path: PathBuf) -> std::io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self { path, file: Mutex::new(file), started: Instant::now() })
    }

    /// Where the log lives
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one event
    pub fn record(&self, event: &str, fields: &[(&str, Value)]) {
        let mono_ms = self.started.elapsed().as_millis() as u64;
        let line = json_line(unix_ms_now(), mono_ms, event, fields);
        // A poisoned lock means another thread panicked mid-write. The
        // file handle itself is still usable, so recover the guard
        // and keep the panic out of the controller thread
        let mut file = self.file.lock_or_recover();
        if let Err(e) = writeln!(file, "{line}") {
            warn!("session log write failed: {e}");
        }
    }

    /// Copy the log as it stands into `dir`, returning the new path
    ///
    /// The copy is named after the participant and the wall clock so a
    /// lab receiving several files can tell them apart. An `export`
    /// event goes into the live log afterwards, so the copy never
    /// contains a record of itself. The event writes the path through
    /// `logged_path`, which keeps the account's home folder name out of
    /// the log
    pub fn export_to(&self, dir: &Path, participant_id: &str) -> std::io::Result<PathBuf> {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let target = dir.join(format!(
            "exhale-log-{}-{secs}.jsonl",
            sanitise(participant_id),
        ));
        // Flush under the lock so the copy can't catch a half-written
        // line from another thread
        {
            let mut file = self.file.lock_or_recover();
            file.flush()?;
            std::fs::copy(&self.path, &target)?;
        }
        let dirs   = directories::UserDirs::new();
        let logged = logged_path(&target, dirs.as_ref().map(|d| d.home_dir()));
        self.record("export", &[("path", Value::Str(&logged))]);
        Ok(target)
    }

    /// Empty the log in place, leaving it open for further appends
    ///
    /// The caller records a fresh header afterwards so a cleared log
    /// still says which pace produced the lines below it
    pub fn clear(&self) -> std::io::Result<()> {
        let mut file = self.file.lock_or_recover();
        file.flush()?;
        file.set_len(0)?;
        Ok(())
    }
}

/// Keep `[A-Za-z0-9_-]`, replace everything else with `_`
///
/// A participant id comes from a file a lab wrote, so it can hold a
/// path separator or a quote. This runs before it reaches a filename
fn sanitise(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    if cleaned.is_empty() { "participant".to_string() } else { cleaned }
}

/// How an exported copy's location is written to the log
///
/// The home folder is named after the OS account, and the log carries
/// no identity beyond the id the lab assigned. A path under `home`
/// becomes `~` plus the rest, joined with the platform's separator.
/// Any other path, or no known home, keeps only the file name
fn logged_path(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|h| path.strip_prefix(h).ok()) {
        Some(rest) => Path::new("~").join(rest).to_string_lossy().into_owned(),
        None => path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
    }
}

/// Where an exported log should go: Desktop, else Documents, else
/// Downloads, else the home directory
///
/// `None` only when the platform can't name a home directory at all,
/// which is the same condition that leaves the app without a config
/// directory
pub fn export_dir() -> Option<PathBuf> {
    let dirs = directories::UserDirs::new()?;
    if let Some(d) = dirs.desktop_dir()  { return Some(d.to_path_buf()); }
    if let Some(d) = dirs.document_dir() { return Some(d.to_path_buf()); }
    if let Some(d) = dirs.download_dir() { return Some(d.to_path_buf()); }
    Some(dirs.home_dir().to_path_buf())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn records_append_one_line_each() {
        let dir  = tmp_dir("append");
        let path = dir.join(LOG_FILE_NAME);
        let log  = SessionLog::open(path.clone()).expect("open");
        log.record("app_start", &[("version", Value::Str("2.0.22"))]);
        log.record("app_stop", &[]);

        let text = std::fs::read_to_string(&path).expect("read back");
        assert!(text.ends_with('\n'), "every line ends with a newline");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(r#""event":"app_start""#), "{}", lines[0]);
        assert!(lines[0].contains(r#""version":"2.0.22""#), "{}", lines[0]);
        assert!(lines[1].contains(r#""event":"app_stop""#), "{}", lines[1]);
    }

    #[test]
    fn reopening_appends_rather_than_truncating() {
        let dir  = tmp_dir("reopen");
        let path = dir.join(LOG_FILE_NAME);
        SessionLog::open(path.clone()).expect("open").record("app_start", &[]);
        SessionLog::open(path.clone()).expect("reopen").record("app_stop", &[]);
        assert_eq!(std::fs::read_to_string(&path).expect("read").lines().count(), 2);
    }

    #[test]
    fn export_copies_the_log_and_sanitises_the_name() {
        let dir  = tmp_dir("export");
        let out  = tmp_dir("export-out");
        let path = dir.join(LOG_FILE_NAME);
        let log  = SessionLog::open(path.clone()).expect("open");
        log.record("app_start", &[]);
        let before = std::fs::read_to_string(&path).expect("read log");

        let copied = log.export_to(&out, "P 07/../x").expect("export");
        let name = copied.file_name().expect("a filename").to_string_lossy().into_owned();
        assert!(name.starts_with("exhale-log-P_07____x-"), "{name}");
        assert!(name.ends_with(".jsonl"), "{name}");
        assert_eq!(
            std::fs::read_to_string(&copied).expect("read copy"),
            before,
            "the copy is the log as it stood, without the export line",
        );

        let after = std::fs::read_to_string(&path).expect("read log again");
        assert_eq!(after.lines().count(), 2, "the export event lands in the live log");
        let export_line = after.lines().nth(1).expect("second line");
        assert!(export_line.contains(r#""event":"export""#));
        assert!(
            !export_line.contains(&*out.to_string_lossy()),
            "the export event shortens the path: {export_line}",
        );
        assert!(export_line.ends_with(&format!("{name}\"}}")), "{export_line}");
    }

    #[test]
    fn the_logged_path_hides_the_home_folder_name() {
        let sep     = std::path::MAIN_SEPARATOR;
        let tmp     = std::env::temp_dir();
        let home    = tmp.join("someone");
        let file    = "exhale-log-P07-1758400420.jsonl";
        let inside  = home.join("Desktop").join(file);
        let sibling = tmp.join("someone-else").join(file);
        let outside = tmp.join("lab").join(file);

        assert_eq!(
            logged_path(&inside, Some(&home)),
            format!("~{sep}Desktop{sep}{file}"),
        );
        assert_eq!(logged_path(&outside, Some(&home)), file);
        assert_eq!(logged_path(&sibling, Some(&home)), file, "a sibling with a longer name is outside home");
        assert_eq!(logged_path(&inside, None), file, "without a known home only the file name is kept");
    }

    #[test]
    fn an_empty_participant_id_still_makes_a_filename() {
        assert_eq!(sanitise(""), "participant");
        assert_eq!(sanitise("///"), "___");
        assert_eq!(sanitise("P-07_b"), "P-07_b");
    }

    #[test]
    fn clear_empties_the_file_and_later_records_still_append() {
        let dir  = tmp_dir("clear");
        let path = dir.join(LOG_FILE_NAME);
        let log  = SessionLog::open(path.clone()).expect("open");
        log.record("app_start", &[]);
        log.record("animation_start", &[]);

        log.clear().expect("clear");
        assert_eq!(std::fs::read_to_string(&path).expect("read").len(), 0);

        log.record("log_cleared", &[]);
        let text = std::fs::read_to_string(&path).expect("read again");
        assert_eq!(text.lines().count(), 1);
        assert!(text.contains(r#""event":"log_cleared""#), "{text}");
    }

    #[test]
    fn mono_ms_counts_from_open() {
        let dir = tmp_dir("mono");
        let log = SessionLog::open(dir.join(LOG_FILE_NAME)).expect("open");
        log.record("app_start", &[]);
        let text = std::fs::read_to_string(log.path()).expect("read");
        // Checks only that the field is there and small
        let mono = text
            .split("\"mono_ms\":").nth(1).expect("mono_ms field")
            .split(',').next().expect("its value")
            .parse::<u64>().expect("an integer");
        assert!(mono < 60_000, "mono_ms should be milliseconds since open, got {mono}");
    }
}
