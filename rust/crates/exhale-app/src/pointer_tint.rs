//! The pointer half of the multiply tint
//!
//! While the still tint multiplies the screen, the pointer's System Settings
//! colors are multiplied by the same gains.  The user's own colors come back
//! when the tint goes, when exhale quits, and on the launch after a run that
//! didn't quit cleanly

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context;
use exhale_core::settings_manager::file_label;
use log::{info, warn};

use crate::platform;

/// The colors under System Settings > Accessibility > Display > Pointer, as
/// `com.apple.universalaccess` holds them, each `None` where the key is unset
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PointerColors {
    /// The "customized" flag.  With it off, macOS draws the stock colors
    /// whatever the two keys hold
    pub customized: bool,
    /// Red, green, blue and alpha of the fill, in sRGB
    #[serde(default)]
    pub fill:       Option<[f64; 4]>,
    /// The same for the outline
    #[serde(default)]
    pub outline:    Option<[f64; 4]>,
}

/// The fill macOS draws until the user picks another: black
const STOCK_FILL:    [f64; 4] = [0.0, 0.0, 0.0, 1.0];
/// The outline macOS draws until the user picks another: white
const STOCK_OUTLINE: [f64; 4] = [1.0, 1.0, 1.0, 1.0];

/// The file in the settings folder that keeps the user's own colors while
/// the pointer shows tinted ones
const SAVED_FILE: &str = "pointer-colors.toml";

/// How long a new tint has to hold before the pointer follows it
///
/// Setting the pointer's colors takes about 100 ms of the main thread, so
/// following each step of a dragged slider would stall the app
const SETTLE: Duration = Duration::from_millis(300);

impl PointerColors {
    /// These colors multiplied by `gains`, the way the backdrop multiplies the
    /// screen, so the stock black fill stays black and the white outline takes
    /// the tint's color
    ///
    /// The result is customized, with both colors set.  It starts from the
    /// user's colors when `customized` is on, with the stock color for a key
    /// that's unset, and from the stock colors when it's off, since keys left
    /// over from an earlier customization don't count then
    pub fn tinted(&self, gains: [f32; 3]) -> PointerColors {
        let (fill, outline) = if self.customized {
            (self.fill.unwrap_or(STOCK_FILL), self.outline.unwrap_or(STOCK_OUTLINE))
        } else {
            (STOCK_FILL, STOCK_OUTLINE)
        };
        let multiply = |[red, green, blue, alpha]: [f64; 4]| {
            [
                red   * f64::from(gains[0]),
                green * f64::from(gains[1]),
                blue  * f64::from(gains[2]),
                alpha,
            ]
        };
        PointerColors {
            customized: true,
            fill:       Some(multiply(fill)),
            outline:    Some(multiply(outline)),
        }
    }
}

/// The colors `path` holds, or `None` when there's no file
///
/// A file that can't be read or parsed is dropped after a warning, so the
/// next launch doesn't meet it again
fn read_saved(path: &Path) -> Option<PointerColors> {
    let name = file_label(path);
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            warn!("pointer tint: couldn't read {name}: {e}, dropping it");
            remove_saved(path);
            return None;
        }
    };
    match toml::from_str(&text) {
        Ok(colors) => Some(colors),
        Err(e) => {
            warn!("pointer tint: couldn't parse {name}: {e}, dropping it");
            remove_saved(path);
            None
        }
    }
}

/// Write `colors` to `path`
fn write_saved(path: &Path, colors: &PointerColors) -> anyhow::Result<()> {
    let text = toml::to_string(colors).context("serialize the pointer colors")?;
    // File names only: these messages reach the debug log, which holds no
    // folder named after the account
    std::fs::write(path, text).with_context(|| format!("write {}", file_label(path)))
}

/// Delete the file at `path`, which may not exist
fn remove_saved(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => warn!("pointer tint: couldn't remove {}: {e}", file_label(path)),
    }
}

/// What [`PointerTint::step`] finds the pointer needs
#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    /// The pointer is in line with the screen, or a change is still settling
    Nothing,
    /// Tint the pointer with these gains
    Tint([f32; 3]),
    /// Put the user's own colors back
    Restore,
}

/// Keeps the pointer's colors in line with the multiply tint the screen shows,
/// and gives the user's own back
///
/// Pointer colors picked in System Settings while the tint is up give way to
/// the colors read before it, when the tint changes or goes.  Exhale can't see
/// that change, since its own refused writes stay in its cache.  A pointer size
/// picked there gives way as well, to the size read before it, when the tint
/// goes
pub struct PointerTint {
    /// The user's own colors, read before the first tint.  `None` while the
    /// pointer shows them
    original:   Option<PointerColors>,
    /// The pointer size, read before the first tint along with `original` and
    /// put back with it.  `None` when it couldn't be read
    scale:      Option<f32>,
    /// The gains the pointer is tinted with now, `None` for no tint
    gains:      Option<[f32; 3]>,
    /// A change from one tint to another that waits to settle, with the
    /// instant it last changed
    pending:    Option<([f32; 3], Instant)>,
    /// Where `original` waits on disk, so a run that ends without quitting
    /// leaves the user's colors for the next launch
    saved_path: PathBuf,
}

impl PointerTint {
    /// A `PointerTint` that keeps its saved colors in `dir`, the settings
    /// folder
    ///
    /// Colors a run that didn't quit cleanly left in that folder go back on
    /// the pointer first.  Main thread only
    pub fn new(dir: &Path) -> Self {
        let saved_path = dir.join(SAVED_FILE);
        if let Some(saved) = read_saved(&saved_path) {
            info!("pointer tint: the last run left the pointer tinted, putting the colors back");
            if platform::set_pointer_colors(&saved) {
                remove_saved(&saved_path);
            }
        }
        Self { original: None, scale: None, gains: None, pending: None, saved_path }
    }

    /// Bring the pointer in line with `gains`, the multiply tint the screen
    /// shows at `now`, `None` for none
    ///
    /// A tint that turns on or goes off reaches the pointer at once.  A change
    /// from one tint to another waits until it has held for [`SETTLE`].  Cheap
    /// when nothing has changed since the last call, so the event loop can run
    /// it on every pass.  Main thread only
    pub fn sync(&mut self, gains: Option<[f32; 3]>, now: Instant) {
        match self.step(gains, now) {
            Step::Nothing => {}
            Step::Tint(gains) => self.tint(gains),
            Step::Restore => self.restore(),
        }
    }

    /// What the pointer needs now that the screen shows `gains` at `now`
    ///
    /// The decision behind [`Self::sync`], which never touches the platform
    ///
    /// A tint that turns on or goes off takes the pointer along at once.  A
    /// change from one tint to another has to hold for [`SETTLE`] first, and
    /// each new value restarts that wait, so the pointer follows a dragged
    /// slider once it stops.  Going back to the gains the pointer shows ends
    /// the wait
    fn step(&mut self, gains: Option<[f32; 3]>, now: Instant) -> Step {
        let Some(wanted) = gains else {
            self.pending = None;
            return if self.gains.is_some() { Step::Restore } else { Step::Nothing };
        };
        match self.gains {
            None => {
                self.pending = None;
                Step::Tint(wanted)
            }
            Some(shown) if shown == wanted => {
                self.pending = None;
                Step::Nothing
            }
            Some(_) => match self.pending {
                Some((waiting, since)) if waiting == wanted => {
                    if now.saturating_duration_since(since) >= SETTLE {
                        self.pending = None;
                        Step::Tint(wanted)
                    } else {
                        Step::Nothing
                    }
                }
                _ => {
                    self.pending = Some((wanted, now));
                    Step::Nothing
                }
            },
        }
    }

    /// Tint the pointer with `gains`, after saving the user's colors if the
    /// pointer still shows them
    fn tint(&mut self, gains: [f32; 3]) {
        // Recorded before anything can fail, so a failure logs once for a
        // tint and the later passes of the event loop find nothing to do
        self.gains = Some(gains);
        let original = match self.original {
            Some(original) => original,
            None => {
                let Some(read) = platform::pointer_colors() else { return; };
                // On disk before the pointer changes, so a run that ends
                // while it's tinted can still put the colors back
                if let Err(e) = write_saved(&self.saved_path, &read) {
                    warn!("pointer tint: {e:#}, so the pointer stays as it is");
                    return;
                }
                self.original = Some(read);
                self.scale = platform::pointer_scale();
                read
            }
        };
        platform::set_pointer_colors(&original.tinted(gains));
        // Only the registered cursors carry the tint.  The preferences get the
        // user's colors back at once, so System Settings never shows the tint
        // and a run that ends early can't leave it saved there
        platform::store_pointer_colors(&original);
    }

    /// Put the user's own colors, and the pointer size, back on the pointer,
    /// if it shows others
    ///
    /// Main thread only
    pub fn restore(&mut self) {
        if let Some(original) = self.original.take() {
            platform::set_pointer_colors(&original);
            // The first registration over a stock pointer grows it by a
            // tenth, and a block on the main queue puts the size back 100 ms
            // later, which a quit inside that time never runs
            if let Some(scale) = self.scale.take() {
                platform::set_pointer_scale(scale);
            }
            remove_saved(&self.saved_path);
        }
        self.gains = None;
        self.pending = None;
    }

    /// When the event loop has to wake so a settled change reaches the
    /// pointer, `None` when no change is waiting
    pub fn next_deadline(&self) -> Option<Instant> {
        self.pending.map(|(_, since)| since + SETTLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gains a float holds exactly, so the step from f32 to f64 adds no error
    const GAINS: [f32; 3] = [1.0, 0.5, 0.25];

    #[test]
    fn tinted_colors_multiply_and_saved_colors_read_back() {
        let stock = PointerColors { customized: false, fill: None, outline: None };
        let stock_tinted = PointerColors {
            customized: true,
            fill:       Some([0.0, 0.0, 0.0, 1.0]),
            outline:    Some([1.0, 0.5, 0.25, 1.0]),
        };
        assert_eq!(stock.tinted(GAINS), stock_tinted, "a black fill stays black");

        let yellow = Some([1.0, 1.0, 0.0, 1.0]);
        let customized = PointerColors { customized: true, fill: yellow, outline: None };
        assert_eq!(
            customized.tinted(GAINS),
            PointerColors { fill: Some([1.0, 0.5, 0.0, 1.0]), ..stock_tinted },
            "the user's fill is multiplied, and an unset outline is the stock one",
        );

        // A leftover key, which macOS ignores while the flag is off
        let leftover = PointerColors { customized: false, fill: yellow, outline: None };
        assert_eq!(leftover.tinted(GAINS), stock_tinted);

        let dir = std::env::temp_dir()
            .join(format!("exhale-pointer-tint-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a test folder");
        let path = dir.join(SAVED_FILE);
        assert_eq!(read_saved(&path), None, "no file yet");

        let both = PointerColors {
            customized: true,
            fill:       Some([1.0, 0.1, 0.2, 0.9]),
            outline:    Some([0.0, 0.3, 0.6, 1.0]),
        };
        for colors in [PointerColors { fill: None, ..both }, both] {
            write_saved(&path, &colors).expect("a written file");
            assert_eq!(read_saved(&path), Some(colors));
        }

        std::fs::write(&path, "customized = [").expect("a bad file");
        assert_eq!(read_saved(&path), None);
        assert!(!path.exists(), "a file that doesn't parse is dropped");
        std::fs::remove_dir_all(&dir).expect("a clean test folder");
    }

    #[test]
    fn a_tint_reaches_the_pointer_at_once_and_a_change_between_tints_once_it_settles() {
        const A: [f32; 3] = [1.0, 0.8, 0.6];
        const B: [f32; 3] = [1.0, 0.6, 0.3];
        const C: [f32; 3] = [0.9, 0.5, 0.2];
        let t0 = Instant::now();
        let just_before = SETTLE - Duration::from_millis(1);
        let mut pointer = PointerTint {
            original:   None,
            scale:      None,
            gains:      None,
            pending:    None,
            saved_path: PathBuf::new(),
        };

        // Off to on at once.  `gains` is set by hand, the way `tint` sets it
        assert_eq!(pointer.step(None, t0), Step::Nothing);
        assert_eq!(pointer.step(Some(A), t0), Step::Tint(A));
        pointer.gains = Some(A);

        // A change between tints waits, and lands once it has held for SETTLE
        assert_eq!(pointer.step(Some(B), t0), Step::Nothing);
        assert_eq!(pointer.next_deadline(), Some(t0 + SETTLE));
        assert_eq!(pointer.step(Some(B), t0 + just_before), Step::Nothing);

        // A new value restarts the wait.  The first value's time is up here,
        // and the second one's isn't
        assert_eq!(pointer.step(Some(C), t0 + just_before), Step::Nothing);
        assert_eq!(pointer.next_deadline(), Some(t0 + just_before + SETTLE));
        assert_eq!(pointer.step(Some(C), t0 + SETTLE), Step::Nothing);
        assert_eq!(pointer.step(Some(C), t0 + just_before + SETTLE), Step::Tint(C));
        assert_eq!(pointer.next_deadline(), None);
        pointer.gains = Some(C);

        // Going back to the shown gains cancels the wait.  Had it survived,
        // A would land at the second step
        let t1 = t0 + 3 * SETTLE;
        assert_eq!(pointer.step(Some(A), t1), Step::Nothing);
        assert_eq!(pointer.step(Some(C), t1 + SETTLE / 2), Step::Nothing);
        assert_eq!(pointer.next_deadline(), None);
        assert_eq!(pointer.step(Some(A), t1 + SETTLE), Step::Nothing);
        assert_eq!(pointer.next_deadline(), Some(t1 + 2 * SETTLE));

        // A tint that goes restores at once and drops the wait
        assert_eq!(pointer.step(None, t1 + SETTLE + SETTLE / 2), Step::Restore);
        assert_eq!(pointer.next_deadline(), None);
    }
}
