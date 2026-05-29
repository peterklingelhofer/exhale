//! Shared window-placement helpers used by both the settings window
//! and the windowed-mode animation window (Wayland fallback path).
//!
//! Each window persists its position as an offset relative to a named
//! monitor — when that monitor is still connected on next launch the
//! window comes back where you left it, and when the monitor's gone
//! (laptop unplugged from a dock, resolution shrunk, external display
//! disconnected) we clamp the rect to whichever monitor is closest
//! rather than restoring to dead coordinates that put the window
//! mostly or entirely off-screen.
//!
//! Two halves:
//!
//!   - **Pure geometry** ([`clamp_position_against_monitors`]) — no
//!     winit dependency, unit-testable.
//!   - **Winit glue** ([`apply_placement`] / [`capture_placement`] /
//!     [`clamp_position_to_visible`]) — reads from / writes to the
//!     live `Window` and `ActiveEventLoop`, used by both windows
//!     from `main.rs` and from each window's constructor.

use exhale_core::WindowPlacement;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::ActiveEventLoop,
    window::Window,
};

/// A monitor's physical rectangle in screen coordinates.  Used by
/// [`clamp_position_against_monitors`] so its geometry logic can be
/// unit-tested without a winit event loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// Apply the persisted POSITION from [`WindowPlacement`] to a
/// freshly-created window.  Resolves the saved monitor name to a live
/// `MonitorHandle` and anchors the offset against that monitor's
/// origin so a rearranged display arrangement still puts the window
/// on the right screen.  When the saved monitor is gone, falls back
/// to treating the offset as absolute and then clamps via
/// [`clamp_position_to_visible`].
///
/// Size is intentionally NOT restored here.  Each window already
/// passes its restored size to `Window::default_attributes()
/// .with_inner_size(...)` at creation time, using the right
/// `LogicalSize` / `PhysicalSize` variant for the units its
/// placement field stores in (settings window persists height in
/// logical points; animation window persists in physical pixels).
/// Re-applying via `request_inner_size(PhysicalSize::new(...))` here
/// would treat the logical points as physical pixels, halving the
/// height on Retina / 2× displays.
pub fn apply_placement(
    event_loop: &ActiveEventLoop,
    window:     &Window,
    placement:  &WindowPlacement,
) {
    // Without a saved position we let the OS / compositor place the
    // window — on macOS / X11 / Windows this lands centred on the
    // primary monitor; on Wayland the compositor places however it
    // likes.
    let (Some(x), Some(y)) = (placement.x, placement.y) else { return; };

    // Resolve the offset against the saved monitor when it's still
    // present.  When it's gone, treat the offset as absolute — the
    // clamp step below pulls it back onto a visible monitor regardless.
    let (abs_x, abs_y) = match &placement.screen {
        Some(name) => {
            let matching = event_loop.available_monitors()
                .find(|m| m.name().as_deref() == Some(name.as_str()));
            match matching {
                Some(m) => {
                    let o = m.position();
                    (o.x + x, o.y + y)
                }
                None => (x, y),
            }
        }
        None => (x, y),
    };

    // Use the window's CURRENT outer size (just set by the caller's
    // `with_inner_size` attr) so the clamp respects the actual
    // window dimensions in physical pixels regardless of which units
    // the placement is stored in.
    let outer = window.outer_size();
    let (clamped_x, clamped_y) =
        clamp_position_to_visible(event_loop, abs_x, abs_y, outer.width, outer.height);
    window.set_outer_position(PhysicalPosition::new(clamped_x, clamped_y));
}

/// Capture the live window's outer position + inner size as a
/// [`WindowPlacement`] suitable for persisting in
/// [`exhale_core::Settings`].  Position is stored as an offset
/// relative to the monitor the window's centre currently lies on, so
/// a saved placement survives the monitor being moved in the OS
/// display-arrangement panel.
pub fn capture_placement(
    event_loop: &ActiveEventLoop,
    window:     &Window,
) -> WindowPlacement {
    let outer = window.outer_position().unwrap_or(PhysicalPosition::new(0, 0));
    let inner = window.inner_size();
    // The settings window persists its height in LOGICAL points (so
    // a 2x display doesn't re-apply scale on next launch).  This
    // helper persists in PHYSICAL pixels, which is fine for the
    // animation window — the next launch's monitor will have the
    // same physical pixel dimensions, and DPI-affected windows can
    // override `width` / `height` via the placement they pass back
    // through `set_*_window_placement`.
    let (x, y, screen) = current_monitor_offset(event_loop, &outer, &inner);

    WindowPlacement {
        x:      Some(x),
        y:      Some(y),
        width:  Some(inner.width),
        height: Some(inner.height),
        screen,
    }
}

/// Like [`capture_placement`] but writes height in logical points
/// instead of physical pixels.  Used by the settings window so the
/// existing `settings_window_height` field stays backward-compatible
/// (it was always logical points)
pub fn capture_placement_logical_height(
    event_loop: &ActiveEventLoop,
    window:     &Window,
) -> WindowPlacement {
    let outer = window.outer_position().unwrap_or(PhysicalPosition::new(0, 0));
    let inner = window.inner_size();
    let scale = window.scale_factor();
    let logical_h = (inner.height as f64 / scale).round() as u32;
    let (x, y, screen) = current_monitor_offset(event_loop, &outer, &inner);

    WindowPlacement {
        x:      Some(x),
        y:      Some(y),
        width:  None,
        height: Some(logical_h),
        screen,
    }
}

/// Find the monitor the window's centre currently sits on, then
/// return position as an offset relative to that monitor's origin
/// plus the monitor's name (for serialisation).
fn current_monitor_offset(
    event_loop: &ActiveEventLoop,
    outer:      &PhysicalPosition<i32>,
    inner:      &PhysicalSize<u32>,
) -> (i32, i32, Option<String>) {
    let cx = outer.x + inner.width  as i32 / 2;
    let cy = outer.y + inner.height as i32 / 2;
    let owning = event_loop.available_monitors().find(|m| {
        let mp = m.position();
        let ms = m.size();
        cx >= mp.x && cx < mp.x + ms.width  as i32 &&
        cy >= mp.y && cy < mp.y + ms.height as i32
    });
    match owning {
        Some(m) => {
            let origin = m.position();
            (outer.x - origin.x, outer.y - origin.y, m.name())
        }
        None => (outer.x, outer.y, None),
    }
}

// ─── Clamp helpers ────────────────────────────────────────────────────────────

/// Return `(x, y)` adjusted so the window rectangle is fully contained
/// inside one of the available monitors.
///
/// Algorithm: pick the monitor closest to the window's center point
/// (zero distance when the center is already inside a monitor; positive
/// distance when off-screen).  Then clamp `(x, y)` so the entire window
/// rect fits inside that monitor's bounds.
///
/// Handles every shape of "saved position is no longer good":
///   - Window dragged partially off-screen by accident, then reopened:
///     clamp pulls the rect back inside the same monitor, preserving
///     the user's "I had it on the right" intent
///   - Monitor unplugged: saved coords land in dead space; clamp pulls
///     the rect into the nearest remaining monitor, again preserving
///     directional intent
///   - Resolution shrunk: saved offset overflows the new bounds;
///     clamp pulls the rect inside the smaller rectangle
///
/// `primary` is used only as a tie-breaker when multiple monitors are
/// equidistant.  Pure geometry, no winit dependency, so unit tests can
/// drive it directly.  Returns `(x, y)` unchanged when `monitors` is
/// empty (no display info available; OS default placement takes over)
pub(crate) fn clamp_position_against_monitors(
    x: i32, y: i32, width: u32, height: u32,
    monitors: &[MonitorRect],
    primary:  Option<MonitorRect>,
) -> (i32, i32) {
    if monitors.is_empty() {
        return (x, y);
    }

    let cx = x + width  as i32 / 2;
    let cy = y + height as i32 / 2;

    let distance_sq = |m: &MonitorRect| -> i64 {
        let max_x = m.x + m.w as i32 - 1;
        let max_y = m.y + m.h as i32 - 1;
        let nx = cx.clamp(m.x, max_x);
        let ny = cy.clamp(m.y, max_y);
        let dx = (cx - nx) as i64;
        let dy = (cy - ny) as i64;
        dx*dx + dy*dy
    };

    let target = monitors.iter().copied()
        .min_by_key(|m| (distance_sq(m), if Some(*m) == primary { 0u8 } else { 1u8 }))
        .unwrap();

    let max_x = target.x + target.w as i32 - width  as i32;
    let max_y = target.y + target.h as i32 - height as i32;
    let nx = if max_x >= target.x { x.clamp(target.x, max_x) } else { target.x };
    let ny = if max_y >= target.y { y.clamp(target.y, max_y) } else { target.y };
    (nx, ny)
}

/// Cross-platform safety net for the persisted-position restore path:
/// a user who quit exhale on a laptop+external setup and re-opens it
/// on the laptop alone can land with the saved coords pointing
/// firmly off-screen (the old external display's coordinate space no
/// longer exists).  Falls through to [`clamp_position_against_monitors`]
/// after collecting the live monitor list from `event_loop`.
pub fn clamp_position_to_visible(
    event_loop: &ActiveEventLoop,
    x: i32, y: i32, width: u32, height: u32,
) -> (i32, i32) {
    let to_rect = |m: &winit::monitor::MonitorHandle| {
        let p = m.position();
        let s = m.size();
        MonitorRect { x: p.x, y: p.y, w: s.width, h: s.height }
    };
    let monitors: Vec<MonitorRect> = event_loop.available_monitors().map(|m| to_rect(&m)).collect();
    let primary  = event_loop.primary_monitor().as_ref().map(to_rect);

    let (nx, ny) = clamp_position_against_monitors(x, y, width, height, &monitors, primary);
    if (nx, ny) != (x, y) {
        log::info!(
            "window placement: saved position ({x}, {y}) was off-screen \
             relative to the current monitor configuration; clamping to \
             ({nx}, {ny})"
        );
    }
    (nx, ny)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: u32, h: u32) -> MonitorRect {
        MonitorRect { x, y, w, h }
    }

    #[test]
    fn clamp_preserves_position_when_center_on_monitor() {
        let mons = [r(0, 0, 1920, 1080)];
        let (nx, ny) = clamp_position_against_monitors(900, 400, 200, 200, &mons, Some(mons[0]));
        assert_eq!((nx, ny), (900, 400));
    }

    #[test]
    fn clamp_snaps_to_right_edge_when_external_unplugged() {
        let mons = [r(0, 0, 1920, 1080)];
        let (nx, ny) = clamp_position_against_monitors(3000, 500, 400, 800, &mons, Some(mons[0]));
        assert_eq!(nx, 1920 - 400, "right edge should align with monitor's right");
        assert_eq!(ny, 1080 - 800);
    }

    #[test]
    fn clamp_snaps_to_left_edge_when_position_negative() {
        let mons = [r(0, 0, 1920, 1080)];
        let (nx, ny) = clamp_position_against_monitors(-1500, 400, 400, 600, &mons, Some(mons[0]));
        assert_eq!(nx, 0, "left edge should align with monitor origin");
        assert_eq!(ny, 400);
    }

    #[test]
    fn clamp_passes_through_when_no_monitors() {
        let (nx, ny) = clamp_position_against_monitors(1000, 500, 400, 600, &[], None);
        assert_eq!((nx, ny), (1000, 500));
    }

    #[test]
    fn clamp_keeps_position_on_secondary_monitor() {
        let laptop   = r(0, 0, 1440, 900);
        let external = r(1440, 0, 1920, 1080);
        let mons = [laptop, external];
        let (nx, ny) = clamp_position_against_monitors(2240, 100, 400, 600, &mons, Some(laptop));
        assert_eq!((nx, ny), (2240, 100));
    }

    #[test]
    fn clamp_picks_nearest_monitor_when_off_screen() {
        let left  = r(0,    0, 1440, 900);
        let right = r(1440, 0, 1920, 1080);
        let mons  = [left, right];
        let (nx, _) = clamp_position_against_monitors(-5000, 100, 400, 600, &mons, Some(right));
        assert_eq!(nx, 0, "nearest monitor (left) should win; snap to its left edge");
    }

    #[test]
    fn clamp_pulls_back_partially_off_right_edge() {
        let mons = [r(0, 0, 1920, 1080)];
        let (nx, ny) = clamp_position_against_monitors(1800, 400, 400, 600, &mons, Some(mons[0]));
        assert_eq!(nx, 1920 - 400);
        assert_eq!(ny, 400);
    }

    #[test]
    fn clamp_pulls_back_partially_off_left_edge() {
        let mons = [r(0, 0, 1920, 1080)];
        let (nx, ny) = clamp_position_against_monitors(-300, 200, 400, 600, &mons, Some(mons[0]));
        assert_eq!(nx, 0);
        assert_eq!(ny, 200);
    }

    #[test]
    fn clamp_pulls_back_partially_off_bottom_edge() {
        let mons = [r(0, 0, 1920, 1080)];
        let (nx, ny) = clamp_position_against_monitors(500, 900, 400, 600, &mons, Some(mons[0]));
        assert_eq!(nx, 500);
        assert_eq!(ny, 1080 - 600);
    }

    #[test]
    fn clamp_handles_window_larger_than_monitor() {
        let mons = [r(0, 0, 800, 600)];
        let (nx, ny) = clamp_position_against_monitors(50, 100, 400, 900, &mons, Some(mons[0]));
        assert_eq!(nx, 50);
        assert_eq!(ny, 0);
    }
}
