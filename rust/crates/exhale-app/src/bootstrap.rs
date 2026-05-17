//! Process-bootstrap plumbing: logger setup, panic hook, single-instance
//! guard, log-path picker.  Extracted from `main.rs` to keep the
//! [`crate::App`] state and event-loop wiring readable.
//!
//! Nothing here references `App` or `AppEvent` directly — everything is
//! either a one-shot side effect (logger, panic hook) or a pure
//! lookup (log-path picker, lock-file path).  The single-instance
//! guard takes an [`winit::event_loop::EventLoopProxy`] only as a
//! placeholder for the type parameter; future bring-to-front
//! protocols may add a wire-side dispatch here, but today the
//! "secondary" path just exits.

use std::path::PathBuf;

use winit::event_loop::EventLoopProxy;

use crate::AppEvent;

/// Outcome of [`single_instance_guard`].  See enum variants for
/// platform behaviour.
pub(crate) enum InstanceGuard {
    /// First instance; holds the lockfile alive for the process
    /// lifetime.  Releasing the file (drop) releases the OS-level
    /// advisory lock so a subsequent launch can take over.
    First(std::fs::File),
    /// Another instance is already running.  On macOS we asked the
    /// running instance to activate (which fires its
    /// `applicationShouldHandleReopen:` handler and shows the
    /// settings window); on Windows/Linux the OS-level dock/taskbar
    /// re-activation does the same thing when the user clicks the
    /// existing app icon.  This process should exit immediately.
    Secondary,
    /// Couldn't open the lockfile at all (permissions, full disk).
    /// Proceed without the guard rather than refuse to start — the
    /// duplicate-instance behaviour is degraded but the app still
    /// runs.
    Unavailable,
}

/// Advisory file-lock-based single-instance guard.  Sandbox-safe on
/// every platform (sandboxed macOS apps can `flock` files in their
/// container; sandboxed Windows MSIX apps can lock files in
/// `%LOCALAPPDATA%`).  Replaces an earlier TCP-loopback design that
/// returned `EPERM` under the macOS App Sandbox because `127.0.0.1`
/// binds require `com.apple.security.network.server` — an entitlement
/// the MAS submission specifically avoids
pub(crate) fn single_instance_guard(_proxy: &EventLoopProxy<AppEvent>) -> InstanceGuard {
    let lock_path = match instance_lock_path() {
        Some(p) => p,
        None    => {
            log::warn!(
                "single_instance_guard: no lock-path candidate available; \
                 running without the guard",
            );
            return InstanceGuard::Unavailable;
        }
    };
    if let Some(parent) = lock_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = match std::fs::OpenOptions::new()
        .read(true).write(true).create(true).truncate(false).open(&lock_path)
    {
        Ok(f) => f,
        Err(e) => {
            log::warn!(
                "single_instance_guard: could not open lock file {} ({e}); \
                 running without the guard",
                lock_path.display(),
            );
            return InstanceGuard::Unavailable;
        }
    };

    match try_lock_exclusive(&file) {
        Ok(true) => {
            // First instance.  Record our PID for debugging /
            // future bring-to-front protocols, but don't rely on it
            // — the lock itself is the source of truth.
            use std::io::Write;
            let _ = file.try_clone()
                .and_then(|mut f| f.write_all(format!("{}\n", std::process::id()).as_bytes()));
            InstanceGuard::First(file)
        }
        Ok(false) => {
            // Lock is held by another running instance.  On macOS, ask
            // the system to activate the existing app — that fires our
            // AppleEvent reopen handler which sets `DOCK_REOPEN`, which
            // the running event loop drains and dispatches as
            // `AppEvent::ShowSettings`.  On other platforms, OS-level
            // launcher behaviour (taskbar pin re-click, GNOME
            // Activities, etc.) brings the existing window to focus;
            // we just exit.
            //
            // Print to stderr in addition to the log file so a
            // `cargo run --release` invocation surfaces "you're
            // running the OLD binary because the previous one is
            // still alive" instead of silently bringing the old
            // window forward.  Without this it's easy to mistake the
            // existing app reappearing for a successful relaunch of
            // freshly-built code.
            eprintln!(
                "exhale: another instance is already running; \
                 activating it and exiting.  Quit the running app first \
                 if you intended to relaunch a new build.",
            );
            log::info!(
                "single_instance_guard: another instance holds the lock at {}; \
                 bringing it to front and exiting",
                lock_path.display(),
            );
            #[cfg(target_os = "macos")]
            crate::platform::activate_running_exhale();
            InstanceGuard::Secondary
        }
        Err(e) => {
            log::warn!(
                "single_instance_guard: lock-acquire syscall failed ({e}); \
                 running without the guard",
            );
            InstanceGuard::Unavailable
        }
    }
}

/// Resolve the path used for the single-instance lock.  Picks a
/// per-user location that's writable under every platform's default
/// sandbox configuration:
///   * **macOS** — `<config_dir>/exhale.lock` where `config_dir` is
///     `~/Library/Application Support/com.peterklingelhofer.exhale`
///     (sandboxed apps get this auto-mapped to the container).
///   * **Windows** — `%LOCALAPPDATA%\peterklingelhofer\exhale\exhale.lock`
///     via the `directories` crate, same as `settings.toml`.
///   * **Linux** — `$XDG_RUNTIME_DIR/exhale.lock` if set, else
///     `~/.config/exhale/exhale.lock` (writable under every
///     mainstream sandbox / flatpak / snap configuration).
fn instance_lock_path() -> Option<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(rt) = std::env::var("XDG_RUNTIME_DIR") {
            if !rt.is_empty() {
                return Some(PathBuf::from(rt).join("exhale.lock"));
            }
        }
    }
    let dirs = directories::ProjectDirs::from("com", "peterklingelhofer", "exhale")?;
    Some(dirs.config_dir().join("exhale.lock"))
}

/// Try to acquire an exclusive non-blocking advisory lock on `file`.
/// Returns `Ok(true)` on success, `Ok(false)` if another process holds
/// the lock, `Err(_)` on any other failure.
fn try_lock_exclusive(file: &std::fs::File) -> std::io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // `flock(2)` with `LOCK_EX | LOCK_NB`.  EWOULDBLOCK means
        // another process holds the lock — that's our "secondary
        // instance" signal, not an error.
        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if rc == 0 {
            Ok(true)
        } else {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EWOULDBLOCK) {
                Ok(false)
            } else {
                Err(err)
            }
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
        };
        use windows_sys::Win32::System::IO::OVERLAPPED;
        // `LockFileEx` with `EXCLUSIVE | FAIL_IMMEDIATELY` is the
        // Win32 equivalent.  ERROR_LOCK_VIOLATION (33) means held by
        // another process.
        let mut ovl: OVERLAPPED = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            LockFileEx(
                file.as_raw_handle() as _,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                1, 0,  // lock one byte at offset 0 — enough for advisory
                &mut ovl,
            )
        };
        if ok != 0 {
            Ok(true)
        } else {
            let err = std::io::Error::last_os_error();
            const ERROR_LOCK_VIOLATION: i32 = 33;
            if err.raw_os_error() == Some(ERROR_LOCK_VIOLATION) {
                Ok(false)
            } else {
                Err(err)
            }
        }
    }
}

/// `Write` adapter that mirrors every byte to both stderr AND a backing
/// file.  We use this as `env_logger`'s target so the same log output
/// appears in the terminal (when there is one) AND on disk next to the
/// exe — needed for windowed-app debugging where stderr is nowhere
/// reachable, including the on-Windows scenario where a black-screen
/// overlay bug renders every other window invisible until the process
/// is force-killed.  Stderr writes are best-effort: if stderr is closed
/// or piped to /dev/null we still want the file log to succeed.
pub(crate) struct TeeLogWriter {
    pub(crate) file: std::fs::File,
}

impl std::io::Write for TeeLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Best-effort stderr — ignore failures so file logging always works.
        let _ = std::io::Write::write_all(&mut std::io::stderr(), buf);
        self.file.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stderr().flush();
        self.file.flush()
    }
}

/// Pick a path to write the log file at.  Preferred location is right
/// next to the exe — most users running an unsigned dev build extract
/// to Downloads / Desktop / similar, which is writable.  Fallbacks
/// progressively widen the net so we never silently lose logs:
///   1. `<exe-dir>/exhale.log`
///   2. `<temp>/exhale.log`
///   3. `./exhale.log`
pub(crate) fn pick_log_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("exhale.log");
            if std::fs::OpenOptions::new()
                .create(true).write(true).truncate(true)
                .open(&candidate).is_ok()
            {
                return candidate;
            }
        }
    }
    let tmp = std::env::temp_dir().join("exhale.log");
    if std::fs::OpenOptions::new()
        .create(true).write(true).truncate(true)
        .open(&tmp).is_ok()
    {
        return tmp;
    }
    PathBuf::from("exhale.log")
}

/// Install a panic hook that appends panic info + backtrace to the log
/// file before delegating to the default hook.  Without this, panics
/// only print to stderr and are lost when stderr isn't captured.
pub(crate) fn install_panic_logger(log_path: PathBuf) {
    use std::sync::OnceLock;
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    let _ = PATH.set(log_path);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(path) = PATH.get() {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true).create(true).open(path)
            {
                use std::io::Write;
                let _ = writeln!(f, "\n=== PANIC ===");
                let _ = writeln!(f, "{info}");
                let _ = writeln!(
                    f, "backtrace:\n{}",
                    std::backtrace::Backtrace::force_capture(),
                );
                let _ = f.flush();
            }
        }
        prev(info);
    }));
}
