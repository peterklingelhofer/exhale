use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{error, info, warn};

use crate::poison::RwLockPoisonExt;
use crate::settings::Settings;
use crate::study::{self, StudyError, StudyStatus};

/// Manages loading, saving, and sharing of [`Settings`]
///
/// Settings are stored as TOML in the platform config directory
/// - macOS:   `~/Library/Application Support/exhale/settings.toml`
/// - Windows: `%APPDATA%\exhale\settings.toml`
/// - Linux:   `~/.config/exhale/settings.toml`
///
/// Writes are coalesced: a background thread opens a fixed 500 ms window
/// at the first dirty mark and flushes to disk when it closes, matching
/// UserDefaults coalescing behaviour
pub struct SettingsManager {
    pub settings: Arc<RwLock<Settings>>,
    config_path:  PathBuf,
    /// What the study file beside `settings.toml` amounted to at launch,
    /// until [`SettingsManager::mark_study_broken`] says otherwise. The
    /// file is read once, so one installed while the app runs takes
    /// effect at the next launch
    study_status: RwLock<StudyStatus>,
    dirty_tx:     std::sync::mpsc::Sender<()>,
    _writer:      std::thread::JoinHandle<()>,
}

impl SettingsManager {
    /// Load settings from disk (or use defaults if no file exists) and start
    /// the coalescing write-back thread
    pub fn new() -> Result<Self> {
        Self::with_config_path(config_file_path()?)
    }

    /// [`SettingsManager::new`] for a settings file at `config_path`
    ///
    /// The app always goes through `new`. This is for a test or a tool
    /// that needs a manager on a folder of its own
    pub fn with_config_path(config_path: PathBuf) -> Result<Self> {
        let (settings, study_status) = load_with_study(&config_path);
        let shared = Arc::new(RwLock::new(settings));

        let (dirty_tx, dirty_rx) = std::sync::mpsc::channel::<()>();
        let path_clone    = config_path.clone();
        let settings_clone = Arc::clone(&shared);

        let writer = std::thread::Builder::new()
            .name("exhale-settings-writer".to_string())
            .spawn(move || {
                write_back_loop(settings_clone, path_clone, dirty_rx);
            })
            .context(
                "exhale-settings-writer: thread::spawn failed (system thread limit / OOM)",
            )?;

        Ok(Self {
            settings: shared,
            config_path,
            study_status: RwLock::new(study_status),
            dirty_tx,
            _writer: writer,
        })
    }

    /// Mark settings as dirty. Will be flushed within ~500 ms
    pub fn mark_dirty(&self) {
        let _ = self.dirty_tx.send(());
    }

    /// Path where the settings file is stored
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Directory holding the settings file, the study file and the
    /// session log
    pub fn config_dir(&self) -> &Path {
        self.config_path.parent().unwrap_or_else(|| Path::new("."))
    }

    /// Off, locked by a valid study file, or broken by one that can't be
    /// used
    ///
    /// A broken study file fails closed: [`load_with_study`] has already
    /// stopped the animation, and the app has to keep it stopped and say
    /// why on screen. A silent fallback would hand a participant the
    /// shipped default pace while the lab believes they're on an arm.
    /// Returned by value, because the status can change under a reader
    /// (see [`SettingsManager::mark_study_broken`]) and a borrow would
    /// hold the lock for as long as the caller kept it
    pub fn study_status(&self) -> StudyStatus {
        self.study_status.read_or_recover().clone()
    }

    /// Move the study to Broken after launch, for a failure only the app
    /// can see. The one that exists is a session log that won't open: a
    /// study without its log is a lost measurement, so the app treats it
    /// as it treats a bad study file
    ///
    /// Stops the animation, as the load-time path does. The lock itself
    /// stays in force (`settings.study` is untouched), so the pace and the
    /// appearance stay hidden and fixed while the card explains. Nothing
    /// persists: the next launch reads the files afresh. Takes the write
    /// lock on the settings, so call it with no guard on them held
    pub fn mark_study_broken(&self, error: StudyError) {
        error!("study broken after launch, the animation stays stopped: {}", error.detail());
        *self.study_status.write_or_recover() = StudyStatus::Broken(error);
        self.settings.write_or_recover().is_animating = false;
    }

    /// Synchronously flush settings to disk right now. Useful on
    /// shutdown to avoid losing the last write
    pub fn flush_sync(&self) -> Result<()> {
        let s = self.settings.read_or_recover().clone();
        // Cloning before persisting means the lock is released for the
        // duration of the (potentially slow) disk write. Readers and
        // writers from other threads aren't blocked on I/O
        save_settings(&s, &self.config_path)
    }
}

// ─── File I/O ─────────────────────────────────────────────────────────────────

fn config_file_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "peterklingelhofer", "exhale")
        .context("couldn't determine config directory")?;
    let dir = dirs.config_dir();
    std::fs::create_dir_all(dir)
        .with_context(|| format!("create config dir {}", dir.display()))?;
    Ok(dir.join("settings.toml"))
}

/// The last component of `path`, for a debug-log line
///
/// The folders above it include the home folder, which is named after the
/// OS account. Public so the app's own debug-log lines can name a file the
/// same way
pub fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn load_or_default(path: &Path) -> Settings {
    let name = file_label(path);
    let mut s = match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<Settings>(&contents) {
            Ok(s)  => { info!("loaded {name}"); s }
            Err(e) => {
                warn!("failed to parse {name}: {e}, using defaults");
                Settings::default()
            }
        },
        Err(_) => {
            info!("no {name}, using defaults");
            Settings::default()
        }
    };
    // Always start animating on launch, never restore a stopped or
    // paused state. Matches Swift SettingsModel.init() which hardcodes
    // isAnimating = true and never loads isAnimating/isPaused back
    // from UserDefaults
    s.is_animating = true;
    s.is_paused    = false;
    s
}

/// Load the settings file, then apply the study file beside it
///
/// The second element says what the study file amounted to. `Off` is
/// every build that isn't running a study. `Locked` means the file's
/// pacing is in the returned settings, with `settings.study` set.
/// `Broken` means a study file is there and can't be used, and then the
/// animation is stopped (`is_animating` false), and nothing is locked:
/// the pace in the returned settings is whatever `settings.toml` held,
/// and the app mustn't run it. The debug log gets the error's detail,
/// which holds no participant id and no folder name
pub fn load_with_study(settings_path: &Path) -> (Settings, StudyStatus) {
    let mut settings = load_or_default(settings_path);
    let study_path   = study::study_file_path(settings_path);
    match study::load(&study_path) {
        Ok(Some(cfg)) => {
            info!("study mode: {} locked the pacing fields", study::STUDY_FILE_NAME);
            cfg.apply(&mut settings);
            (settings, StudyStatus::Locked)
        }
        Ok(None) => (settings, StudyStatus::Off),
        Err(e) => {
            error!("study file unusable, the animation stays stopped: {}", e.detail());
            settings.is_animating = false;
            (settings, StudyStatus::Broken(e))
        }
    }
}

fn save_settings(settings: &Settings, path: &PathBuf) -> Result<()> {
    let contents = toml::to_string_pretty(settings)
        .context("serialise settings")?;
    // Write to a temp file then rename: atomic on POSIX, best-effort on Windows
    let tmp = path.with_extension("toml.tmp");
    // File names only: these messages reach the debug log, which holds
    // no folder named after the account
    std::fs::write(&tmp, &contents)
        .with_context(|| format!("write {}", file_label(&tmp)))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", file_label(&tmp), file_label(path)))?;
    Ok(())
}

fn write_back_loop(
    settings: Arc<RwLock<Settings>>,
    path:     PathBuf,
    rx:       std::sync::mpsc::Receiver<()>,
) {
    // Drain the channel with a 500 ms timeout, flush once there are no more
    // events for that window (coalescing)
    loop {
        match rx.recv() {
            Err(_) => break, // sender dropped -> manager is gone
            Ok(()) => {
                // Drain further events within the coalesce window
                let deadline = std::time::Instant::now() + Duration::from_millis(500);
                loop {
                    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                    if remaining.is_zero() { break; }
                    match rx.recv_timeout(remaining) {
                        Ok(())  => continue,
                        Err(_)  => break,
                    }
                }
                let snap = settings.read_or_recover().clone();
                if let Err(e) = save_settings(&snap, &path) {
                    warn!("settings write failed: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::types::AnimationShape;

    fn tmp_path() -> PathBuf {
        let dir = std::env::temp_dir().join("exhale_settings_test");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("settings.toml")
    }

    #[test]
    fn save_and_load_round_trip() {
        let path = tmp_path();
        let mut s = Settings::default();
        s.inhale_duration = 7.5;
        s.shape = AnimationShape::Circle;

        save_settings(&s, &path).unwrap();
        let loaded = load_or_default(&path);
        assert_eq!(loaded.inhale_duration, 7.5);
        assert_eq!(loaded.shape, AnimationShape::Circle);
    }

    #[test]
    fn load_default_when_file_missing() {
        let path = PathBuf::from("/non/existent/path/settings.toml");
        let s = load_or_default(&path);
        assert_eq!(s.inhale_duration, Settings::default().inhale_duration);
    }

    #[test]
    fn load_default_on_corrupt_file() {
        let path = tmp_path().with_extension("corrupt.toml");
        std::fs::write(&path, b"not valid toml !!!##").unwrap();
        let s = load_or_default(&path);
        assert_eq!(s.exhale_duration, Settings::default().exhale_duration);
    }

    // ── Study file beside the settings file ───────────────────────────

    /// A directory of its own per call so a study file written by one
    /// test can't reach another
    fn study_tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "exhale-manager-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// A settings file whose appearance, Show In and timing all differ
    /// from the defaults, so a test can tell "the study file won" apart
    /// from "everything was reset"
    fn write_settings_file(path: &Path) {
        let mut s = Settings::default();
        s.shape           = AnimationShape::Circle;
        s.overlay_opacity = 0.9;
        s.inhale_duration = 7.5;
        s.exhale_duration = 7.5;
        s.app_visibility  = crate::types::AppVisibility::TopBarOnly;
        save_settings(&s, &path.to_path_buf()).expect("write settings");
    }

    const FULL_STUDY_FILE: &str = concat!(
        "participant_id = \"P07\"\n",
        "condition = \"B\"\n",
        "inhale_duration = 4.0\n",
        "post_inhale_hold_duration = 1.0\n",
        "exhale_duration = 6.0\n",
        "post_exhale_hold_duration = 2.0\n",
        "drift = 1.002\n",
        "randomized_timing_inhale = 0.1\n",
        "randomized_timing_post_inhale_hold = 0.2\n",
        "randomized_timing_exhale = 0.3\n",
        "randomized_timing_post_exhale_hold = 0.4\n",
        "auto_stop_minutes = 30.0\n",
        "reminder_interval_minutes = 15.0\n",
    );

    #[test]
    fn a_study_file_locks_the_pacing_and_the_appearance_and_nothing_else() {
        let dir      = study_tmp_dir("locked");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);
        std::fs::write(dir.join(study::STUDY_FILE_NAME), FULL_STUDY_FILE)
            .expect("write study file");

        let (s, status) = load_with_study(&settings);
        assert_eq!(status, StudyStatus::Locked);

        assert_eq!(s.inhale_duration,           4.0);
        assert_eq!(s.post_inhale_hold_duration, 1.0);
        assert_eq!(s.exhale_duration,           6.0);
        assert_eq!(s.post_exhale_hold_duration, 2.0);
        assert_eq!(s.drift,                     1.002);
        assert_eq!(s.randomized_timing_inhale,           0.1);
        assert_eq!(s.randomized_timing_post_inhale_hold, 0.2);
        assert_eq!(s.randomized_timing_exhale,           0.3);
        assert_eq!(s.randomized_timing_post_exhale_hold, 0.4);
        assert_eq!(s.auto_stop_minutes,         30.0);
        assert_eq!(s.reminder_interval_minutes, 15.0);

        // The file says nothing about appearance, so the participant's own
        // choices in settings.toml don't reach the screen: it's the shipped look
        let shipped = Settings::default();
        assert_eq!(s.shape, shipped.shape);
        assert_eq!(s.overlay_opacity, shipped.overlay_opacity);
        assert_eq!(s.display_mode, shipped.display_mode);
        // Show In is the one setting that stays theirs, and so does the launch state
        assert_eq!(s.app_visibility, crate::types::AppVisibility::TopBarOnly);
        assert!(s.is_animating, "launch always starts animating");

        let study = s.study.expect("the lock is recorded");
        assert_eq!(study.participant_id, "P07");
        assert_eq!(study.condition, "B");
    }

    #[test]
    fn the_appearance_a_study_file_sets_wins_over_the_participants_settings() {
        let dir      = study_tmp_dir("locked-look");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);
        std::fs::write(
            dir.join(study::STUDY_FILE_NAME),
            format!("{FULL_STUDY_FILE}shape = \"fullscreen\"\noverlay_opacity = 0.5\ndisplay_mode = \"window\"\n"),
        ).expect("write study file");

        let (s, status) = load_with_study(&settings);
        assert_eq!(status, StudyStatus::Locked);
        assert_eq!(s.shape, AnimationShape::Fullscreen, "the file's shape wins over the settings file's circle");
        assert_eq!(s.overlay_opacity, 0.5);
        assert_eq!(s.display_mode, crate::types::DisplayMode::Window);
        // What the file leaves out is the shipped default and not the participant's
        assert_eq!(s.inhale_color, Settings::default().inhale_color);
        assert_eq!(s.app_visibility, crate::types::AppVisibility::TopBarOnly);
    }

    const INVALID_STUDY_FILE: &str = concat!(
        "participant_id = \"P07\"\n",
        "condition = \"\"\n",
        "inhale_duration = 5.0\n",
        "post_inhale_hold_duration = 0.0\n",
        "exhale_duration = 5.0\n",
        "post_exhale_hold_duration = 0.0\n",
    );

    #[test]
    fn an_invalid_study_file_fails_closed() {
        let dir      = study_tmp_dir("invalid");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);
        std::fs::write(dir.join(study::STUDY_FILE_NAME), INVALID_STUDY_FILE)
            .expect("write study file");

        let (s, status) = load_with_study(&settings);
        let StudyStatus::Broken(err) = status else { panic!("expected Broken, got {status:?}") };
        assert_eq!(err.summary(), "study.toml, line 2, field condition: can't be empty");

        assert!(!s.is_animating, "an unusable study file leaves the animation stopped");
        assert!(!s.is_paused);
        assert!(s.study.is_none(), "and locks nothing");
        // The settings file's pace is loaded, and it's the app's job to keep it from running
        assert_eq!(s.inhale_duration, load_or_default(&settings).inhale_duration);
        assert_eq!(s.shape, AnimationShape::Circle);
    }

    #[test]
    fn every_kind_of_unusable_study_file_stops_the_animation() {
        for tag in ["syntax", "validation", "directory", "misnamed"] {
            let dir      = study_tmp_dir(tag);
            let settings = dir.join("settings.toml");
            write_settings_file(&settings);
            match tag {
                "syntax"     => std::fs::write(dir.join("study.toml"), "participant_id = ").expect("write"),
                "validation" => std::fs::write(dir.join("study.toml"), INVALID_STUDY_FILE).expect("write"),
                "directory"  => std::fs::create_dir(dir.join("study.toml")).expect("mkdir"),
                _            => std::fs::write(dir.join("study.toml.txt"), FULL_STUDY_FILE).expect("write"),
            }

            let (s, status) = load_with_study(&settings);
            assert!(matches!(status, StudyStatus::Broken(_)), "{tag}: {status:?}");
            assert!(!s.is_animating, "{tag}");
            assert!(s.study.is_none(), "{tag}");
        }
    }

    #[test]
    fn a_misnamed_study_file_names_itself_in_the_error() {
        let dir      = study_tmp_dir("misnamed");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);
        std::fs::write(dir.join("study.toml.txt"), FULL_STUDY_FILE).expect("write");

        let (s, status) = load_with_study(&settings);
        let err = status.error().expect("a misnamed file breaks the study");
        assert_eq!(
            err.summary(),
            "study.toml: found \"study.toml.txt\" in the settings folder. Rename it to study.toml",
        );
        assert!(!s.is_animating);
        assert_eq!(status.error(), Some(err));
    }

    #[test]
    fn no_study_file_is_the_ordinary_case() {
        let dir      = study_tmp_dir("none");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);

        let (s, status) = load_with_study(&settings);
        assert_eq!(status, StudyStatus::Off);
        assert!(status.error().is_none());
        assert!(s.study.is_none());
        assert!(s.is_animating, "an ordinary launch starts animating");
        assert_eq!(s.inhale_duration, 7.5);
        assert_eq!(s.exhale_duration, 7.5);
        assert_eq!(s.shape, AnimationShape::Circle);
    }

    // ── The study status the app reads ────────────────────────────────

    #[test]
    fn the_manager_reports_the_status_it_loaded() {
        // The manager's own writer thread is the cost, so one folder per case
        let off = study_tmp_dir("manager-off");
        write_settings_file(&off.join("settings.toml"));
        let manager = SettingsManager::with_config_path(off.join("settings.toml")).expect("manager");
        assert_eq!(manager.study_status(), StudyStatus::Off);
        assert_eq!(manager.config_dir(), off);

        let locked = study_tmp_dir("manager-locked");
        write_settings_file(&locked.join("settings.toml"));
        std::fs::write(locked.join(study::STUDY_FILE_NAME), FULL_STUDY_FILE).expect("write study file");
        let manager = SettingsManager::with_config_path(locked.join("settings.toml")).expect("manager");
        assert_eq!(manager.study_status(), StudyStatus::Locked);
        assert!(manager.settings.read_or_recover().study.is_some());

        let broken = study_tmp_dir("manager-broken");
        write_settings_file(&broken.join("settings.toml"));
        std::fs::write(broken.join(study::STUDY_FILE_NAME), INVALID_STUDY_FILE).expect("write study file");
        let manager = SettingsManager::with_config_path(broken.join("settings.toml")).expect("manager");
        let status = manager.study_status();
        assert_eq!(
            status.error().map(|e| e.summary()),
            Some("study.toml, line 2, field condition: can't be empty".to_string()),
        );
        assert!(!manager.settings.read_or_recover().is_animating);
    }

    #[test]
    fn a_study_marked_broken_after_launch_stops_the_animation_and_keeps_the_lock() {
        let dir = study_tmp_dir("mark-broken");
        write_settings_file(&dir.join("settings.toml"));
        std::fs::write(dir.join(study::STUDY_FILE_NAME), FULL_STUDY_FILE).expect("write study file");
        let manager = SettingsManager::with_config_path(dir.join("settings.toml")).expect("manager");
        assert_eq!(manager.study_status(), StudyStatus::Locked);
        assert!(manager.settings.read_or_recover().is_animating, "a locked study launches animating");

        // What `main` does when the session log won't open
        let reason = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let error  = StudyError::log_unavailable(&reason);
        manager.mark_study_broken(error.clone());

        let status = manager.study_status();
        assert_eq!(status, StudyStatus::Broken(error));
        assert_eq!(
            status.error().map(|e| e.summary()),
            Some("study.toml: the session log can't be written".to_string()),
        );

        // The same effect on the animation as a bad file has at load
        let s = manager.settings.read_or_recover();
        assert!(!s.is_animating);
        // And the lock holds: the pace and the look stay fixed and hidden while the card explains
        assert!(s.study.is_some());
        assert_eq!(s.inhale_duration, 4.0);
        assert_eq!(s.shape, Settings::default().shape);
        drop(s);

        // Nothing persists. A launch reads the files afresh
        let relaunch = SettingsManager::with_config_path(dir.join("settings.toml")).expect("manager");
        assert_eq!(relaunch.study_status(), StudyStatus::Locked);
        assert!(relaunch.settings.read_or_recover().is_animating);
    }

    #[test]
    fn marking_a_study_broken_leaves_a_paused_flag_and_the_rest_alone() {
        let dir = study_tmp_dir("mark-broken-rest");
        write_settings_file(&dir.join("settings.toml"));
        std::fs::write(dir.join(study::STUDY_FILE_NAME), FULL_STUDY_FILE).expect("write study file");
        let manager = SettingsManager::with_config_path(dir.join("settings.toml")).expect("manager");
        let before = manager.settings.read_or_recover().clone();

        manager.mark_study_broken(StudyError::log_unavailable(&std::io::Error::other("full")));

        let mut expected = before;
        expected.is_animating = false;
        let after = manager.settings.read_or_recover().clone();
        assert_eq!(
            toml::Value::try_from(&after).expect("serialise"),
            toml::Value::try_from(&expected).expect("serialise"),
            "only is_animating changed",
        );
    }

    // ── What the debug log is allowed to say ──────────────────────────

    /// Collects every line the `log` macros emit while a test runs
    struct CaptureLogger;

    static CAPTURED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

    impl log::Log for CaptureLogger {
        fn enabled(&self, _: &log::Metadata) -> bool { true }
        fn log(&self, record: &log::Record) {
            CAPTURED.lock().unwrap().push(format!("{}", record.args()));
        }
        fn flush(&self) {}
    }

    /// Install the capture once. Another test may have got there first,
    /// which only matters if it installed a different logger, and none does
    fn capture_logs() {
        let _ = log::set_logger(&CaptureLogger);
        log::set_max_level(log::LevelFilter::Trace);
    }

    #[test]
    fn the_debug_log_names_no_participant_id_and_no_folder() {
        capture_logs();
        let secret_id = "P-SECRET-4711";
        let secret_arm = "ARM-SECRET-4711";
        let valid = FULL_STUDY_FILE
            .replace("P07", secret_id)
            .replace("\"B\"", &format!("\"{secret_arm}\""));
        // A mistyped id is the one parse error whose message quotes it
        let mistyped_id = valid.replace(&format!("\"{secret_id}\""), "47110000");

        let mut folders = Vec::new();
        for (tag, study_file) in [
            ("locked", Some(valid.clone())),
            ("invalid", Some(valid.replace("inhale_duration = 4.0", "inhale_duration = 0.05"))),
            ("mistyped-id", Some(mistyped_id)),
            ("none", None),
        ] {
            let dir      = study_tmp_dir(&format!("log-{tag}"));
            let settings = dir.join("settings.toml");
            write_settings_file(&settings);
            if let Some(text) = study_file {
                std::fs::write(dir.join(study::STUDY_FILE_NAME), text).expect("write");
            }
            let _ = load_with_study(&settings);
            // A settings write that fails names a folder in its error
            let missing = dir.join("no-such-folder").join("settings.toml");
            let err = save_settings(&Settings::default(), &missing).expect_err("no such folder");
            assert!(!err.to_string().contains(&*dir.to_string_lossy()), "{err}");
            folders.push(dir);
        }

        let lines = CAPTURED.lock().unwrap().clone();
        assert!(!lines.is_empty(), "the capture saw nothing, so it proves nothing");
        for line in &lines {
            for secret in [secret_id, secret_arm, "47110000"] {
                assert!(!line.contains(secret), "the debug log holds {secret}: {line}");
            }
            for dir in &folders {
                let text = dir.to_string_lossy();
                assert!(!line.contains(&*text), "the debug log holds a folder: {line}");
            }
        }
        // And it does say what happened to the file
        assert!(
            lines.iter().any(|l| l.contains("study file unusable") && l.contains("0.05")),
            "the unusable file's detail reaches the debug log: {lines:?}",
        );
    }
}
