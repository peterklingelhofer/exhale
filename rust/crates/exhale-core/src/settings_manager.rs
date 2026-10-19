use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{info, warn};

use crate::poison::RwLockPoisonExt;
use crate::settings::Settings;
use crate::study;

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
    /// Why the study file beside `settings.toml` was ignored, when one
    /// was there and couldn't be used. The settings window shows this
    /// so a lab installing a file finds out it didn't take
    study_error:  Option<String>,
    dirty_tx:     std::sync::mpsc::Sender<()>,
    _writer:      std::thread::JoinHandle<()>,
}

impl SettingsManager {
    /// Load settings from disk (or use defaults if no file exists) and start
    /// the coalescing write-back thread
    pub fn new() -> Result<Self> {
        let config_path = config_file_path()?;
        let (settings, study_error) = load_with_study(&config_path);
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
            study_error,
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

    /// `Some(message)` when a study file was present and unusable
    ///
    /// The app runs without the lock in that case, so the message has
    /// to reach the screen: a silent fallback would hand a participant
    /// the shipped default pace while the lab believes they're on an arm
    pub fn study_error(&self) -> Option<&str> {
        self.study_error.as_deref()
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

fn load_or_default(path: &Path) -> Settings {
    let mut s = match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<Settings>(&contents) {
            Ok(s)  => { info!("loaded settings from {}", path.display()); s }
            Err(e) => {
                warn!("failed to parse {}: {e}, using defaults", path.display());
                Settings::default()
            }
        },
        Err(_) => {
            info!("no settings file at {}, using defaults", path.display());
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
/// The second element is the message to show when a study file was
/// there and couldn't be used. `None` covers both "no study file" and
/// "the study file loaded cleanly", which the caller tells apart by
/// looking at `settings.study`
pub fn load_with_study(settings_path: &Path) -> (Settings, Option<String>) {
    let mut settings = load_or_default(settings_path);
    let study_path   = study::study_file_path(settings_path);
    match study::load(&study_path) {
        Ok(Some(cfg)) => {
            info!(
                "study mode: {} locked the pacing fields for participant {}",
                study_path.display(),
                cfg.participant_id,
            );
            cfg.apply(&mut settings);
            (settings, None)
        }
        Ok(None) => (settings, None),
        Err(msg) => {
            warn!("{msg}. Pacing isn't locked");
            (settings, Some(msg))
        }
    }
}

fn save_settings(settings: &Settings, path: &PathBuf) -> Result<()> {
    let contents = toml::to_string_pretty(settings)
        .context("serialise settings")?;
    // Write to a temp file then rename: atomic on POSIX, best-effort on Windows
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, &contents)
        .with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
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

    /// A settings file whose appearance and timing both differ from the
    /// defaults, so a test can tell "the study file won" apart from
    /// "everything was reset"
    fn write_settings_file(path: &Path) {
        let mut s = Settings::default();
        s.shape           = AnimationShape::Circle;
        s.overlay_opacity = 0.9;
        s.inhale_duration = 7.5;
        s.exhale_duration = 7.5;
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
    fn a_study_file_overrides_the_pacing_and_nothing_else() {
        let dir      = study_tmp_dir("locked");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);
        std::fs::write(dir.join(study::STUDY_FILE_NAME), FULL_STUDY_FILE)
            .expect("write study file");

        let (s, err) = load_with_study(&settings);
        assert_eq!(err, None);

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

        // The settings file's own values survive everywhere else
        assert_eq!(s.shape, AnimationShape::Circle);
        assert_eq!(s.overlay_opacity, 0.9);
        assert!(s.is_animating, "launch always starts animating");

        let study = s.study.expect("the lock is recorded");
        assert_eq!(study.participant_id, "P07");
        assert_eq!(study.condition, "B");
    }

    #[test]
    fn an_invalid_study_file_reports_and_leaves_the_app_unlocked() {
        let dir      = study_tmp_dir("invalid");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);
        std::fs::write(
            dir.join(study::STUDY_FILE_NAME),
            "participant_id = \"P07\"\ncondition = \"\"\ninhale_duration = 5.0\n\
             post_inhale_hold_duration = 0.0\nexhale_duration = 5.0\n\
             post_exhale_hold_duration = 0.0\n",
        ).expect("write study file");

        let (s, err) = load_with_study(&settings);
        let msg = err.expect("an unusable study file reports why");
        assert!(msg.contains("condition"), "{msg}");
        assert!(s.study.is_none(), "an unusable file locks nothing");
        assert_eq!(s.inhale_duration, load_or_default(&settings).inhale_duration);
        assert_eq!(s.shape, AnimationShape::Circle);
    }

    #[test]
    fn no_study_file_is_the_ordinary_case() {
        let dir      = study_tmp_dir("none");
        let settings = dir.join("settings.toml");
        write_settings_file(&settings);

        let (s, err) = load_with_study(&settings);
        assert_eq!(err, None);
        assert!(s.study.is_none());
        assert_eq!(s.inhale_duration, 7.5);
        assert_eq!(s.exhale_duration, 7.5);
        assert_eq!(s.shape, AnimationShape::Circle);
    }
}
