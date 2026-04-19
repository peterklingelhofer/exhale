use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{info, warn};

use crate::settings::Settings;

/// Manages loading, saving, and sharing of [`Settings`].
///
/// Settings are stored as TOML in the platform config directory:
/// - macOS:   `~/Library/Application Support/exhale/settings.toml`
/// - Windows: `%APPDATA%\exhale\settings.toml`
/// - Linux:   `~/.config/exhale/settings.toml`
///
/// Writes are coalesced: a background thread waits 500 ms of silence before
/// flushing to disk, matching UserDefaults coalescing behaviour.
pub struct SettingsManager {
    pub settings: Arc<RwLock<Settings>>,
    config_path:  PathBuf,
    dirty_tx:     std::sync::mpsc::Sender<()>,
    _writer:      std::thread::JoinHandle<()>,
}

impl SettingsManager {
    /// Load settings from disk (or use defaults if no file exists) and start
    /// the coalescing write-back thread.
    pub fn new() -> Result<Self> {
        let config_path = config_file_path()?;
        let settings = load_or_default(&config_path);
        let shared = Arc::new(RwLock::new(settings));

        let (dirty_tx, dirty_rx) = std::sync::mpsc::channel::<()>();
        let path_clone    = config_path.clone();
        let settings_clone = Arc::clone(&shared);

        let writer = std::thread::Builder::new()
            .name("exhale-settings-writer".to_string())
            .spawn(move || {
                write_back_loop(settings_clone, path_clone, dirty_rx);
            })
            .expect("spawn settings writer thread");

        Ok(Self {
            settings: shared,
            config_path,
            dirty_tx,
            _writer: writer,
        })
    }

    /// Mark settings as dirty; will be flushed within ~500 ms.
    pub fn mark_dirty(&self) {
        let _ = self.dirty_tx.send(());
    }

    /// Path where the settings file is stored.
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Synchronously flush settings to disk right now.
    /// Useful on shutdown to avoid losing the last write.
    pub fn flush_sync(&self) -> Result<()> {
        let s = self.settings.read().unwrap().clone();
        save_settings(&s, &self.config_path)
    }
}

// ─── File I/O ─────────────────────────────────────────────────────────────────

fn config_file_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "peterklingelhofer", "exhale")
        .context("could not determine config directory")?;
    let dir = dirs.config_dir();
    std::fs::create_dir_all(dir)
        .with_context(|| format!("create config dir {}", dir.display()))?;
    Ok(dir.join("settings.toml"))
}

fn load_or_default(path: &PathBuf) -> Settings {
    let mut s = match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<Settings>(&contents) {
            Ok(s)  => { info!("loaded settings from {}", path.display()); s }
            Err(e) => {
                warn!("failed to parse {}: {e}; using defaults", path.display());
                Settings::default()
            }
        },
        Err(_) => {
            info!("no settings file at {}; using defaults", path.display());
            Settings::default()
        }
    };
    // Always start animating on launch; never restore a stopped or paused state.
    // Matches Swift SettingsModel.init() which hardcodes isAnimating = true
    // and never loads isAnimating/isPaused back from UserDefaults.
    s.is_animating = true;
    s.is_paused    = false;
    s
}

fn save_settings(settings: &Settings, path: &PathBuf) -> Result<()> {
    let contents = toml::to_string_pretty(settings)
        .context("serialise settings")?;
    // Write to a temp file then rename — atomic on POSIX, best-effort on Windows.
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, &contents)
        .with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} → {}", tmp.display(), path.display()))?;
    Ok(())
}

fn write_back_loop(
    settings: Arc<RwLock<Settings>>,
    path:     PathBuf,
    rx:       std::sync::mpsc::Receiver<()>,
) {
    // Drain the channel with a 500 ms timeout; flush once there are no more
    // events for that window (coalescing).
    loop {
        match rx.recv() {
            Err(_) => break, // sender dropped → manager is gone
            Ok(()) => {
                // Drain further events within the coalesce window.
                let deadline = std::time::Instant::now() + Duration::from_millis(500);
                loop {
                    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                    if remaining.is_zero() { break; }
                    match rx.recv_timeout(remaining) {
                        Ok(())  => continue,
                        Err(_)  => break,
                    }
                }
                let snap = settings.read().unwrap().clone();
                if let Err(e) = save_settings(&snap, &path) {
                    warn!("settings write failed: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
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
}
