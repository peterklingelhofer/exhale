//! Theme + visuals helpers for the settings window.
//!
//! Three responsibilities:
//!   - Install an OS-native UI font on the egui context so settings
//!     text reads as SF Pro on macOS, Segoe UI on Windows, Ubuntu/
//!     Cantarell on Linux.
//!   - Build the per-theme [`egui::Visuals`] used by the settings
//!     window (text colour, widget rounding, stepper chrome, etc.).
//!   - Pick the wgpu surface clear colour for the current theme +
//!     OS-blur availability.
use egui::ThemePreference;
use winit::window::Theme;

use crate::platform;

/// Corner radius used for egui widget chrome (TextEdit, comboboxes,
/// etc.) inside the settings window.  Hand-painted widgets (stepper,
/// segmented picker, control button) draw their own rounding via
/// `painter.rect_*` and pass their own constants — they aren't
/// affected by this value.  Kept here (not in the parent module's
/// layout constants) so [`visuals_for_theme`] can be tested in
/// isolation
const TEXT_EDIT_RADIUS: f32 = 5.0;

/// Load the OS-native UI font and register it as the default proportional
/// font on the egui context.  Each platform's system-preferences app uses a
/// specific typeface (SF Pro on macOS, Segoe UI on Windows, Ubuntu/Cantarell
/// on common Linux desktops); matching that here makes our settings window
/// read as a native part of the OS instead of egui's default Ubuntu fallback.
///
/// System fonts are NOT redistributed — we read the font file the OS ships
/// with, exactly like every native app does (NSFont on macOS, GDI on
/// Windows, fontconfig on Linux).  No licensing concern.
///
/// If no candidate path exists on the current machine, we silently keep
/// egui's default font — the window still works, it just doesn't blend in
/// quite as well.
pub(crate) fn install_system_ui_font(ctx: &egui::Context) {
    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &[
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/SFNSDisplay.ttf",
        "/System/Library/Fonts/SFNSText.ttf",
        "/Library/Fonts/SF-Pro.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];
    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\SegoeUI.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
    ];
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let candidates: &[&str] = &[
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        "/usr/share/fonts/cantarell/Cantarell-VF.otf",
        "/usr/share/fonts/cantarell/Cantarell-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];

    let Some((path, data)) = candidates.iter().find_map(|p| {
        std::fs::read(p).ok().map(|d| (*p, d))
    }) else {
        log::info!("install_system_ui_font: no candidate font readable; keeping egui default");
        return;
    };
    log::info!("install_system_ui_font: using {path}");

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "system_ui".to_owned(),
        egui::FontData::from_owned(data),
    );
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "system_ui".to_owned());
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "system_ui".to_owned());
    ctx.set_fonts(fonts);
}

/// Resolve the OS-appearance-aware egui visuals used by the settings window.
pub(crate) fn visuals_for_theme(theme: Theme) -> egui::Visuals {
    let mut v = match theme {
        Theme::Dark  => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    v.window_rounding = 10.0.into();

    // Force full-contrast text that reads over the vibrancy-tinted cards in
    // both modes.  egui's defaults (from_gray(140) dark / from_gray(60) light)
    // look washed-out against the translucent SectionCards — especially light
    // mode over hudWindow vibrancy, which is already near-white, so a dark
    // gray label reads as if someone turned the opacity down on the text.
    // Match SwiftUI `.primary` (#FFFFFF on dark, #000000 on light).
    let (fg_text, fg_subtle) = if matches!(theme, Theme::Dark) {
        (egui::Color32::from_rgb(235, 235, 240), egui::Color32::from_rgb(235, 235, 240))
    } else {
        (egui::Color32::from_rgb(20, 20, 22),    egui::Color32::from_rgb(20, 20, 22))
    };
    v.override_text_color = Some(fg_text);
    v.widgets.noninteractive.fg_stroke.color = fg_subtle;
    v.widgets.inactive.fg_stroke.color       = fg_text;
    v.widgets.hovered.fg_stroke.color        = fg_text;
    v.widgets.active.fg_stroke.color         = fg_text;

    // Light mode's default inactive widget stroke is near-invisible, so
    // segmented-picker segments bleed into the panel background.  Bump the
    // inactive stroke to a faint mid-gray so each segment's edge reads at
    // rest — matching the legibility we already get in dark mode and the
    // Swift `NSSegmentedControl` look.
    if matches!(theme, Theme::Light) {
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(180));
    }

    // Round egui widget chrome (TextEdit, checkboxes, comboboxes) to match
    // the macOS-native ~5-6 px corner.
    let r = egui::Rounding::same(TEXT_EDIT_RADIUS);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.rounding       = r;
    v.widgets.hovered.rounding        = r;
    v.widgets.active.rounding         = r;
    v.widgets.open.rounding           = r;

    // TextEdit fills + stepper chrome reads against the card-tinted
    // backdrop.  egui's stock dark mode picks near-black for both
    // (`extreme_bg_color` ≈ rgb(10,10,10), `widgets.inactive.weak_bg_fill`
    // ≈ rgb(60,60,60)) which sits darker than the card behind and
    // disappears against it.  AppKit's `NSTextField` and `NSStepper`
    // are noticeably LIGHTER than the surrounding controlBackground in
    // dark mode — they read as raised input affordances.  Match that
    // by lifting both fills several steps in dark mode; light mode's
    // defaults are already correct.
    if matches!(theme, Theme::Dark) {
        v.extreme_bg_color = egui::Color32::from_rgb(58, 58, 60);

        let stepper_rest    = egui::Color32::from_rgb(78, 78, 80);
        let stepper_hover   = egui::Color32::from_rgb(96, 96, 98);
        let stepper_press   = egui::Color32::from_rgb(120, 120, 122);
        let stepper_stroke  = egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 110, 112));
        v.widgets.inactive.weak_bg_fill = stepper_rest;
        v.widgets.inactive.bg_stroke    = stepper_stroke;
        v.widgets.hovered.weak_bg_fill  = stepper_hover;
        v.widgets.hovered.bg_stroke     = stepper_stroke;
        v.widgets.active.weak_bg_fill   = stepper_press;
        v.widgets.active.bg_stroke      = stepper_stroke;
    }

    v
}

/// Map winit's `Theme` onto egui's `ThemePreference` so the context can be
/// pinned to the exact OS appearance we just polled (bypassing egui's own
/// `System` auto-detect, which runs one frame behind).
pub(crate) fn theme_preference(theme: Theme) -> ThemePreference {
    match theme {
        Theme::Dark  => ThemePreference::Dark,
        Theme::Light => ThemePreference::Light,
    }
}

/// wgpu clear colour for the settings surface.
///
/// When `platform::is_blur_active()` is true, the OS is providing a blur
/// behind the window (macOS VEV child-window, Windows DWM acrylic, KDE
/// blur-behind region) — we clear at alpha 0 so wgpu doesn't paint
/// anything where egui hasn't drawn, letting the OS blur show through.
///
/// When blur isn't active (older Windows, GNOME, opt-out via
/// `EXHALE_DISABLE_BLUR=1`), the window is rendered opaquely — clear to
/// egui's panel fill so there's no flash between surface reconfiguration
/// and the first paint.
pub(crate) fn clear_color_for_theme(theme: Theme) -> wgpu::Color {
    if platform::is_blur_active() {
        wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    } else {
        match theme {
            Theme::Dark  => wgpu::Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 },
            Theme::Light => wgpu::Color { r: 0.96, g: 0.96, b: 0.96, a: 1.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_preference_round_trip() {
        assert!(matches!(theme_preference(Theme::Dark),  ThemePreference::Dark));
        assert!(matches!(theme_preference(Theme::Light), ThemePreference::Light));
    }

    #[test]
    fn visuals_dark_uses_high_contrast_text() {
        let v = visuals_for_theme(Theme::Dark);
        let c = v.override_text_color.expect("text color override set in dark mode");
        // RGB > 200 = near-white; we use rgb(235,235,240).
        assert!(c.r() >= 200 && c.g() >= 200 && c.b() >= 200,
            "dark-mode text should be near-white, got {c:?}");
    }

    #[test]
    fn visuals_light_uses_low_value_text() {
        let v = visuals_for_theme(Theme::Light);
        let c = v.override_text_color.expect("text color override set in light mode");
        assert!(c.r() <= 50 && c.g() <= 50 && c.b() <= 50,
            "light-mode text should be near-black, got {c:?}");
    }

    #[test]
    fn visuals_dark_and_light_differ() {
        let d = visuals_for_theme(Theme::Dark);
        let l = visuals_for_theme(Theme::Light);
        assert_ne!(d.override_text_color, l.override_text_color);
    }
}
