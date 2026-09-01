//! Reusable egui widget primitives for the settings window: custom
//! stepper, segmented picker, control button, section card, color
//! helpers, formatting.
//!
//! Everything here is `pub(super)` because it's only consumed by
//! the parent `settings_window` module
use std::time::{Duration, Instant};

pub(super) fn section(ui: &mut egui::Ui, header: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let dark_mode = ui.visuals().dark_mode;

    // Swift's SectionCard fill is `Color(NSColor.controlBackgroundColor)
    // .opacity(0.55)` — over an NSVisualEffectView .hudWindow backdrop that
    // renders at ~80% (dark) or ~85% (light) luminance, this produces cards
    // that are *barely* distinguishable from the vibrancy.  Matching that
    // with a hand-tuned premul-unaware fill:
    //   dark  — controlBackgroundColor ≈ #1E1E1E, .55 alpha ≈ 86 out of 255
    //           but vibrancy already tints toward dark, so the visible delta
    // EXACT match for Swift's `SectionCard.fill`:
    //   Color(NSColor.controlBackgroundColor).opacity(0.55)
    // `controlBackgroundColor`:
    //   dark  → (0.118, 0.118, 0.118, 1.0) ≈ #1E1E1E (RGB 30, 30, 30)
    //   light → #FFFFFF (RGB 255, 255, 255)
    // `.opacity(0.55)` → alpha = 140 / 255.  Composited over the
    // NSVisualEffectView's popover/hudWindow material, this gives
    // Swift's "dark dark gray" card in dark mode and a translucent
    // white card in light mode.
    let fill = card_fill(dark_mode);
    // Constrain the card to exactly the scroll area's viewport width so every
    // section (Controls, Appearance, Timing, Randomization, Timers) aligns at
    // the same left and right gutters.
    let target_w = (super::SETTINGS_WIDTH as f32 - 2.0 * OUTER_PAD).min(ui.available_width());
    ui.allocate_ui_with_layout(
        egui::vec2(target_w, 0.0),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_max_width(target_w);
            // No stroke on the section Frame — the translucent `fill` alone
            // separates the card from the vibrancy gutters.  The outlined
            // look is reserved for the control buttons inside the top card,
            // matching Swift's `ControlButton.strokeBorder` treatment.
            egui::Frame::none()
                .inner_margin(CARD_PAD)
                .rounding(CARD_RADIUS)
                .fill(fill)
                .show(ui, |ui| {
                    ui.set_max_width(target_w - 2.0 * CARD_PAD);
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = ROW_GAP;

                    if !header.is_empty() {
                        section_header(ui, header);
                    }
                    add_contents(ui);
                });
        },
    );
}

/// Uppercase, letter-spaced, `.secondary` header mimicking SwiftUI's
/// `.font(.system(size: 10, weight: .semibold)).foregroundColor(.secondary).tracking(0.8)`.
pub(super) fn section_header(ui: &mut egui::Ui, text: &str) {
    use egui::text::{LayoutJob, TextFormat};

    let dark_mode = ui.visuals().dark_mode;
    // `.secondary` ≈ 60% of primary.  Picked by eye to match SwiftUI's
    // secondaryLabelColor (#EBEBF599 on dark, #3C3C4399 on light).
    let color = if dark_mode {
        egui::Color32::from_rgb(160, 160, 166)
    } else {
        egui::Color32::from_rgb(99, 99, 106)
    };

    let mut job = LayoutJob::default();
    job.append(
        &text.to_uppercase(),
        0.0,
        TextFormat {
            font_id: egui::FontId::proportional(10.0),
            color,
            // SwiftUI tracking(0.8) → 0.8 pt extra between glyphs.
            extra_letter_spacing: 0.8,
            ..Default::default()
        },
    );
    ui.label(job);
    ui.add_space(2.0);
}

/// SwiftUI `ControlButton`: an icon + 12 pt medium label on a 7 px rounded rect
/// with theme-aware translucent fill + stroke, and a hover/press tint that
/// brightens by ~5% / 8% respectively.  Buttons expand equally across the row
/// (`.frame(maxWidth: .infinity)` in Swift) — we achieve that by dividing the
/// ui's available width by the number of siblings, which egui's horizontal
/// layout with equal allocations gives us for free via `allocate_exact_size`.
///
/// Two rendering paths depending on what icon material is available:
///   - **macOS (SF Symbol texture present)**: paint the whole
///     `.circle.fill` symbol directly.  Apple has done the optical
///     centring of the inner glyph against the surrounding ring, so
///     the rendering matches Swift's `Image(systemName:)` output
///     pixel-for-pixel and no per-icon offset is needed.
///   - **Windows / Linux (no texture)**: paint a filled circle in
///     the foreground colour then composite the Unicode glyph
///     (`▶ ■ ↺ ×`) on top in a muted contrasting colour, mimicking
///     SF Symbol's transparent cutout look.  Per-glyph
///     `unicode_y_offset` tunes positioning against system fonts'
///     varied glyph metrics.
///
/// `icon_font_size_override` lets a caller bump the font size used for
/// the Unicode fallback glyph specifically — useful when the chosen
/// glyph lives in a Unicode block (e.g. Latin-1 Supplement, where `×`
/// is sized to lowercase x-height) that renders visually smaller than
/// the Geometric Shapes glyphs the other buttons use (`▶ ■ ↺`, all
/// full-em).  `None` keeps the default 8 pt sizing.  Has no effect
/// when an SF Symbol texture is in use.
///
/// `unicode_y_offset` shifts the Unicode fallback glyph vertically
/// inside the painted ring (px, negative = up).  Has no effect on
/// the SF Symbol texture path because Apple's symbol design is the
/// source of truth for inner-glyph positioning there.
///
/// `draw_inner_square` and `draw_inner_triangle` paint the inner
/// shape as an egui primitive instead of using the texture / glyph.
/// The two have **different precedence** to reflect different
/// rendering issues:
///
///   - `draw_inner_square` overrides BOTH the texture and Unicode
///     paths.  Used by Stop because Apple's `stop.circle.fill`
///     rasterises the inner square slightly high in our pipeline
///     (visible side-by-side with Swift) AND the Unicode U+25A0
///     glyph isn't reliably centred across system fonts.
///   - `draw_inner_triangle` overrides only the Unicode path,
///     yielding to the SF Symbol texture when one is available.
///     Used by Play because Apple's `play.circle.fill` renders
///     correctly on macOS — the issue is only Segoe UI / Linux
///     fonts positioning U+25B6 low-left in its em-box (the glyph
///     was designed as a dropdown indicator, not a play button)
#[allow(clippy::too_many_arguments)]
pub(super) fn control_button(
    ui:                      &mut egui::Ui,
    width:                   f32,
    icon:                    &str,
    icon_texture:            Option<&egui::TextureHandle>,
    icon_font_size_override: Option<f32>,
    unicode_y_offset:        f32,
    draw_inner_square:       bool,
    draw_inner_triangle:     bool,
    text:                    &str,
    help:                    &str,
) -> egui::Response {
    // Stock `egui::Button` chrome — themed fill / stroke / hover /
    // pressed states / focus indicator — same widget the inline
    // reset-confirmation Cancel and Reset buttons render with, so
    // all six buttons in the Controls section land in a single
    // visually-consistent style.  The button is laid out with an
    // empty label so its centred-text slot is empty; we paint the
    // icon + label ourselves on top of the chrome below.  Pre-fix
    // this routine painted its OWN translucent Swift-style fill +
    // stroke, which read as off-brand next to the egui-default
    // Cancel / Reset buttons.  Button height drops from `ROW_H + 6`
    // (Swift's tall 28-pt tile) to `ROW_H` (egui's default 22 pt)
    // so the top row matches the Cancel / Reset pair height
    // exactly; the icon + label rendering below still fits since
    // we paint at 16-pt icon and 12-pt label, both within the
    // 22-pt bounds
    let size     = egui::vec2(width, ROW_H);
    let response = ui.add_sized(size, egui::Button::new(""));
    let rect     = response.rect;

    // When Tab moves focus to a control button that's currently
    // scrolled out of the settings ScrollArea's viewport, the focus
    // halo would render off-screen and the user would think Tab
    // skipped past — `scroll_to_me(None)` nudges the ScrollArea just
    // enough to bring the focused widget into view, no further.
    // `gained_focus()` is true only on the FRAME focus arrived, so
    // we don't re-scroll every subsequent frame the button is
    // focused
    if response.gained_focus() {
        response.scroll_to_me(None);
    }

    let enabled   = ui.is_enabled();
    let pressed   = response.is_pointer_button_down_on() && enabled;
    let dark_mode = ui.visuals().dark_mode;

    // `primary` and `with_alpha` are still used by the icon
    // painting paths below — white-in-dark / black-in-light is the
    // colour we paint the `.circle.fill` ring + the label, matching
    // the unchanged-from-before icon rendering the user explicitly
    // asked us to preserve.  The egui Button chrome above paints
    // its own fill / stroke independently of these
    let primary = if dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK };
    let with_alpha = |base: egui::Color32, a: u8| {
        egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), a)
    };

    let painter = ui.painter().clone();

    // Keyboard-focus indicator.  When the button is reached via Tab
    // (response.has_focus()), draw a subtle outer ring in the
    // theme-inverse colour so the user can see where focus landed.
    // The fill alone doesn't change visibly when focused — without
    // this ring the user has to remember which button they just
    // tabbed onto, especially in the top-row controls where every
    // button shares the same chrome.  Multi-layer soft halo (3 px
    // outside, mid-alpha) reads as a glow rather than a hard
    // outline, matching the user's "subtle drop shadow glow"
    // request
    if response.has_focus() {
        // Halo colour matches the `primary` foreground (white in
        // dark mode, black in light mode) but at low alpha so it
        // composites as a soft outer glow over whatever sits behind
        // the panel.  Two stacked stroked rects with decreasing
        // alpha produce the falloff a single thicker stroke can't
        for (i, alpha) in [(1.0_f32, 110_u8), (2.5_f32, 60_u8), (4.0_f32, 28_u8)] {
            painter.rect_stroke(
                rect.expand(i),
                BUTTON_RADIUS + i,
                egui::Stroke::new(1.0, with_alpha(primary, alpha)),
            );
        }
    }

    // Pressed state: Swift uses `.opacity(0.7)` + `.scaleEffect(0.97)`.  Scale
    // is awkward in immediate-mode; drop opacity instead — the user still gets
    // a clear "pressed" read.
    let content_alpha: u8 = if pressed { 178 } else if enabled { 255 } else { 110 };
    let content_color = with_alpha(primary, content_alpha);

    let font_label = egui::FontId::proportional(12.0);
    // Unicode-fallback glyph size: ~50% of the surrounding ring's
    // diameter, matching the SF Symbol `.circle.fill` family's
    // inner-glyph proportion.  Caller can override for glyphs that
    // read smaller at the same em (Latin-1 `×`, lowercase x-height)
    // so they hit the same visual weight as Geometric Shapes glyphs
    // (`▶ ■ ↺`)
    let icon_font_size = icon_font_size_override.unwrap_or(8.0);
    let font_icon = egui::FontId::proportional(icon_font_size);
    // Ring diameter — dropped from 16 pt to 13 pt because at 16 pt
    // the filled circle visibly dominated the 22 pt button height
    // next to the 12 pt label text (icon read as larger than the
    // word it sat beside).  13 pt leaves the ring just a touch
    // taller than the label cap-height — same family of weights
    // egui's segmented-picker glyphs use elsewhere in the panel
    let icon_w     = 13.0_f32;
    let label_size = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font_label.clone(), content_color).size());
    let gap        = 6.0_f32;
    let total_w    = icon_w + gap + label_size.x;
    let start_x    = rect.center().x - total_w * 0.5;
    let baseline_y = rect.center().y;

    // Reused by the painted (non-SF-Symbol) paths: a muted
    // dark / light grey for the inner glyph / shape so it reads
    // as the SF Symbol's natural inner-shape colour against the
    // foreground-coloured ring.  Survives both opaque and vibrancy
    // backgrounds
    let cutout_base = if dark_mode {
        egui::Color32::from_gray(28)
    } else {
        egui::Color32::from_gray(240)
    };
    let cutout_alpha = (content_alpha as u32 * cutout_base.a() as u32 / 255) as u8;
    let cutout = egui::Color32::from_rgba_unmultiplied(
        cutout_base.r(), cutout_base.g(), cutout_base.b(), cutout_alpha,
    );

    if draw_inner_square {
        // Stop button: paint the outer ring AND the inner square as
        // egui primitives so the square's geometric centre lands
        // exactly on the ring's geometric centre.  We do this
        // instead of using `stop.circle.fill` (macOS) or the
        // Unicode U+25A0 glyph (Win/Linux) because both of those
        // rasterise the square slightly above the ring's centre in
        // our rendering pipeline — visible side-by-side with Swift
        // even though both apps load the same SF Symbol.  Drawing
        // the rectangle ourselves trades a tiny anti-aliasing
        // difference vs Apple's hand-tuned symbol for guaranteed
        // pixel-perfect centring across every OS
        let icon_center = egui::pos2(start_x + icon_w * 0.5, baseline_y);
        painter.circle_filled(icon_center, icon_w * 0.5, content_color);
        // Square sized to ~34% of the ring's diameter so it reads
        // with similar visual weight to the inner glyphs in the
        // adjacent `.circle.fill` SF Symbols (whose negative-space
        // play triangle, reset arrow, and power glyph all sit a
        // touch smaller than half the ring's diameter)
        // Square scaled proportionally with `icon_w` (13 / 16 ≈ 0.81)
        // so the inner shape keeps its ~31 % ring-fill ratio
        let square_size = 4.0_f32;
        let square_rect = egui::Rect::from_center_size(
            icon_center,
            egui::vec2(square_size, square_size),
        );
        painter.rect_filled(square_rect, 0.0, cutout);
    } else if let Some(tex) = icon_texture {
        // macOS: render the whole `.circle.fill` SF Symbol as one
        // image.  Apple has done the optical centring of the inner
        // glyph against the ring, so this matches Swift's
        // `Image(systemName:)` rendering pixel-for-pixel without any
        // per-icon offset on our side.  `content_color` tint
        // modulates pressed / disabled states the same way the label
        // dims — the texture itself is a white silhouette in dark
        // mode and black in light mode (see `render_sf_symbol`'s
        // template + SourceAtop pass)
        // Match the icon_w ring-painting paths so the SF Symbol
        // texture renders at the same diameter as the Win / Linux
        // primitives below; both targets land at 13 pt
        let icon_size = 13.0_f32;
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(start_x, baseline_y - icon_size / 2.0),
            egui::vec2(icon_size, icon_size),
        );
        painter.image(
            tex.id(),
            icon_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            content_color,
        );
    } else if draw_inner_triangle {
        // Play button on Win / Linux: paint the outer ring AND a
        // right-pointing triangle as egui primitives.  We don't
        // override the SF Symbol texture path (Apple's
        // `play.circle.fill` renders correctly), but the Unicode
        // U+25B6 BLACK RIGHT-POINTING TRIANGLE glyph is positioned
        // low-left in Segoe UI's em-box (the glyph was designed as
        // a dropdown indicator, not a play button), so painting our
        // own triangle is the only reliable cross-font centring.
        let icon_center = egui::pos2(start_x + icon_w * 0.5, baseline_y);
        painter.circle_filled(icon_center, icon_w * 0.5, content_color);
        // Triangle bounding box ~5.5×5.5 pt, matching the stop
        // square's visual weight.  Shifted RIGHT ~0.9 px so the
        // centroid (true centre of mass) lands on the ring's centre
        // rather than the bounding box.  An isoceles right-pointing
        // triangle's centroid sits at 1/3 of its width from the
        // base, i.e. left of bounding-box centre by `width / 6` —
        // for a 5.5 pt triangle that's ~0.9 px.  Adding the shift
        // moves the centroid to the ring's centre, where the eye
        // expects "centred" to mean
        // Triangle scaled proportionally with the smaller ring
        // (4.5 ≈ 5.5 × 13 / 16) so its visual weight relative to
        // the ring matches the stop-square ratio above
        let tri_size = 4.5_f32;
        let cx = icon_center.x + tri_size / 6.0;
        let cy = icon_center.y;
        let points = vec![
            egui::pos2(cx - tri_size * 0.5, cy - tri_size * 0.5),
            egui::pos2(cx - tri_size * 0.5, cy + tri_size * 0.5),
            egui::pos2(cx + tri_size * 0.5, cy),
        ];
        painter.add(egui::Shape::convex_polygon(points, cutout, egui::Stroke::NONE));
    } else {
        // Windows / Linux: paint a filled ring in the foreground
        // colour and composite the Unicode glyph inside it in the
        // cutout colour, mimicking the SF Symbol's transparent
        // cutout against the card.  Per-glyph `unicode_y_offset`
        // tunes positioning against the system font's varied
        // metrics for `↺ ×` (each block sits at a different
        // vertical position in its em-box)
        let icon_center = egui::pos2(start_x + icon_w * 0.5, baseline_y);
        painter.circle_filled(icon_center, icon_w * 0.5, content_color);
        painter.text(
            icon_center + egui::vec2(0.0, unicode_y_offset),
            egui::Align2::CENTER_CENTER,
            icon,
            font_icon,
            cutout,
        );
    }

    painter.text(
        egui::pos2(start_x + icon_w + gap + label_size.x * 0.5, baseline_y),
        egui::Align2::CENTER_CENTER,
        text, font_label, content_color,
    );

    response.on_hover_text(help)
}

// Layout constants mirror SwiftUI SettingsView.swift:
//   label column width (115) → SettingsView.settingLabelWidth
//   outer padding    (14)    → .padding(.horizontal, 14) + .padding(.top/.bottom, 14)
//   card padding     (12)    → SectionCard.padding(12)
//   section gap      (10)    → sectionSpacing
//   row gap          (8)     → rowSpacing
//   card radius      (10)    → RoundedRectangle(cornerRadius: 10, style: .continuous)
//   button radius    (7)     → ControlButton's RoundedRectangle(cornerRadius: 7)
//   stepper field    (56)    → CombinedStepperTextField TextField .frame(width: 56)
// Label column width.  Trade-off: shorter keeps the segmented pickers
// (Rectangle/Circle/Full, Gradient/Stark/Off, etc.) from wrapping their
// widest option text, at the cost of ellipsising a few long labels like
// "Overlay Opacity (%)" and "Background Color".  Swift's SettingsView
// uses 115 pt with `.lineLimit(1)` — same behaviour.  The pickers at this
// width get ~191 px to share across 3 segments (≈63 px each), which fits
// "Rectangle" (≈55 px natural) with room to spare when we render buttons
// with `button_padding = 0` inside the segmented row.
pub(super) const LABEL_W:          f32 = 115.0;
pub(super) const ROW_H:            f32 = 22.0;
pub(super) const OUTER_PAD:        f32 = 14.0;
pub(super) const CARD_PAD:         f32 = 12.0;
pub(super) const SECTION_GAP:      f32 = 10.0;
pub(super) const ROW_GAP:          f32 = 8.0;
pub(super) const CARD_RADIUS:      f32 = 10.0;
pub(super) const BUTTON_RADIUS:    f32 = 7.0;
// (`TEXT_EDIT_RADIUS` lives in the `theme` submodule — it's used by
// `visuals_for_theme` and nothing else, no reason to expose it here)
pub(super) const STEPPER_FIELD_W:  f32 = 56.0;

/// macOS-native selection pill: a slightly inset rounded rect filled in
/// gray (lighter than the container in dark mode, near-white in light
/// mode), matching AppKit's `NSSegmentedControl`
/// `.selectedContentBackground`.
///
/// Shared by the segmented pickers and the preset chips so there is one
/// selected colour in the window rather than two that drift apart, and
/// so the contrast test in this file covers both. It is measured rather
/// than eyeballed: see `selected_pill_text_meets_wcag_aa`
/// Translucent `SectionCard` fill, composited over the platform
/// vibrancy backdrop. Hoisted out of [`section`] so the contrast test
/// can composite against the real value rather than a guess at it
pub(super) fn card_fill(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgba_unmultiplied(30, 30, 30, 140)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 140)
    }
}

pub(super) fn selected_pill_fill(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        // ~rgb(110,110,110) at 90% — reads as a clear lighter gray over
        // the dark vibrancy without going washed-out.
        egui::Color32::from_rgba_unmultiplied(110, 110, 110, 230)
    } else {
        // Near-white selection on a light translucent backdrop.
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 235)
    }
}

/// Text painted on top of [`selected_pill_fill`].
pub(super) fn selected_pill_text(dark_mode: bool) -> egui::Color32 {
    if dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK }
}


/// Measure every segmented picker in a single frame and return the largest
/// natural column width across them.  Buttons within a single picker get
/// equal width (so all options in that picker fit their widest text); the
/// column width is then max-of-natural-widths so that every picker in the
/// settings window shares the same left AND right bounds.
///
/// `SEGMENT_SLACK_PX` adds a small per-segment breathing room so the
/// measurement is always wide enough for the actual rendered text — the
/// `layout_no_wrap` measure and the on-render glyph layout can disagree by
/// a couple of pixels due to font hinting and sub-pixel positioning, which
/// was enough to let "Rectangle" wrap onto a second line inside a segment
/// that measured as "just big enough".
pub(super) fn uniform_picker_column_width(ui: &egui::Ui, pickers: &[&[&str]]) -> f32 {
    const SEGMENT_SLACK_PX: f32 = 10.0;
    let pad_x   = ui.spacing().button_padding.x * 2.0;
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let measure = |s: &str| ui.fonts(|f|
        f.layout_no_wrap(s.to_string(), font_id.clone(), egui::Color32::WHITE).size().x
    );

    let mut max_col: f32 = 0.0;
    for opts in pickers {
        if opts.is_empty() { continue; }
        let max_text = opts.iter().map(|&s| measure(s)).fold(0.0_f32, f32::max);
        let btn_w    = (max_text + pad_x + SEGMENT_SLACK_PX).ceil();
        let col_w    = btn_w * opts.len() as f32;
        if col_w > max_col { max_col = col_w; }
    }
    max_col.ceil()
}

/// Two-cell row layout for non-picker rows: fixed-width label on the left,
/// DragValue / ColorPicker / etc. right-aligned against the row's trailing
/// edge.  Everything to the right of the label cell sits in a `right_to_left`
/// layout so the widget hugs the right edge exactly like Swift's Form.
/// Two-column row: a fixed-width label painted directly via the painter on
/// the left, and a `right_to_left` widget area on the right.
///
/// The painter-direct approach exists because `allocate_ui_with_layout` with
/// a fixed min_size collapses to the label's natural width inside a
/// horizontal layout — which left the remaining widget area wider than it
/// should be, and caused stepper TextEdits to draw over labels that were
/// still in their natural rect.  Reserving an exact-size rect and drawing
/// into it with the painter API guarantees the widget area to the right
/// starts at `LABEL_W + item_spacing`.
pub(super) fn labeled_row(ui: &mut egui::Ui, label: &str, add_widget: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        paint_label(ui, label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_widget);
    }).response
}

/// Reserve a LABEL_W × ROW_H rect and paint `label` into it flush against the
/// rect's left edge with the current theme's text colour.  Using the painter
/// directly (rather than `ui.put(rect, Label::truncate())`) pins the text
/// exactly at `rect.left()` — `Label` was adding implicit horizontal padding
/// that read as "the labels aren't left-aligned" against Swift's reference.
pub(super) fn paint_label(ui: &mut egui::Ui, label: &str) -> egui::Response {
    paint_label_with_width(ui, label, LABEL_W)
}

/// Like `paint_label` but with a caller-specified column width.  Used by
/// `segmented_row` so the picker can extend leftward into the label column
/// when its natural width would otherwise overflow the card on the right.
///
/// Returns the label's hover-Response so callers can scope tooltips
/// to the label region only (not the entire row).  Important for the
/// segmented picker: attaching `on_hover_text` to the row meant
/// hovering an option button raised a tooltip whose help text often
/// quoted the option name itself ("Animation shape: Rectangle…"),
/// which mid-fade-in read as "the label is rendered twice"
pub(super) fn paint_label_with_width(
    ui:    &mut egui::Ui,
    label: &str,
    width: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, ROW_H),
        egui::Sense::hover(),
    );
    let color = ui.visuals().text_color();
    let font  = egui::TextStyle::Body.resolve(ui.style());
    let mut job = egui::text::LayoutJob::simple_singleline(
        label.to_string(), font, color,
    );
    job.wrap = egui::text::TextWrapping {
        max_width:          rect.width(),
        max_rows:            1,
        break_anywhere:      true,
        overflow_character:  Some('…'),
    };
    let galley = ui.painter().layout_job(job);
    let text_pos = egui::pos2(
        rect.left(),
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, color);
    response
}

/// Segmented picker row.  Label on the left; a right-aligned picker cell
/// of `column_w` wide on the right.  `column_w` is measured once per frame
/// (see `uniform_picker_column_width`) and passed identically to every
/// picker in the Appearance section, so the leftmost option button lands
/// on the same X coordinate regardless of the picker's option count or
/// text length.
pub(super) fn segmented_row<T: Copy + PartialEq>(
    ui:       &mut egui::Ui,
    label:    &str,
    help:     &str,
    enabled:  bool,
    column_w: f32,
    current:  &mut T,
    options:  &[(&str, T)],
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        // If the picker's natural column_w wouldn't fit at the standard
        // LABEL_W, shrink the label column so the picker can claim its
        // natural width — making every segmented picker share the same
        // left and right edges regardless of the widest option's text.
        // `MIN_LABEL_W` is the lower bound at which even short labels
        // ("Shape", "Gradient") stay readable after truncation.
        const MIN_LABEL_W: f32 = 70.0;
        const SAFETY_PX:   f32 = 2.0;
        let row_avail = ui.available_width();
        let label_w   = (row_avail - column_w - SAFETY_PX)
                         .clamp(MIN_LABEL_W, LABEL_W);
        // Scope the help-text tooltip to the label region only — see
        // `paint_label_with_width` docstring for why the row-wide
        // hover tooltip was producing the "duplicate label" artifact
        // when hovering individual options.
        paint_label_with_width(ui, label, label_w).on_hover_text(help);
        // `SAFETY_PX` slack on the right — the outer rect stroke I paint
        // under the picker is 1 px centered-on-edge (so 0.5 px bleed outside),
        // and sub-pixel rounding of `per_w = (picker_w / n).floor()` plus the
        // last_w remainder can occasionally push the child min_rect by
        // another fraction of a pixel.
        let remaining = (ui.available_width() - SAFETY_PX).max(0.0);
        let picker_w  = column_w.min(remaining).max(1.0);
        let gap       = (remaining - picker_w).max(0.0);
        if gap > 0.0 { ui.add_space(gap); }

        ui.add_enabled_ui(enabled, |ui| {
            let n = options.len();
            // Sub-pixel remainder is absorbed by the last segment so the
            // rightmost edge lands exactly on picker_w.
            let per_w  = (picker_w / n as f32).floor().max(1.0);
            let last_w = per_w + (picker_w - per_w * n as f32).max(0.0);

            // Pre-compute the outer rect ourselves and use `ui.put(rect, btn)`
            // for each segment.  `ui.add_sized(size, btn)` does NOT actually
            // constrain the Button to `size` — Button's `allocate_at_least`
            // grows the frame to the natural text+padding width, which was
            // the real source of the Appearance-section right-overflow (debug
            // showed picker_w=156 requested but actual=167 delivered).
            // `ui.put` positions the widget into a fixed rect without
            // letting it grow the parent's min_rect.
            let outer_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(picker_w, ROW_H),
            );

            let dark_mode = ui.visuals().dark_mode;

            // Disable egui's default Button hover/press fills — we'll paint
            // a rounded inset pill ourselves for hover/press/selected so all
            // three states share the same macOS-native pill look.
            {
                let widgets = &mut ui.visuals_mut().widgets;
                widgets.inactive.bg_stroke    = egui::Stroke::NONE;
                widgets.hovered.bg_stroke     = egui::Stroke::NONE;
                widgets.active.bg_stroke      = egui::Stroke::NONE;
                widgets.hovered.expansion     = 0.0;
                widgets.active.expansion      = 0.0;
                widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                widgets.hovered.weak_bg_fill  = egui::Color32::TRANSPARENT;
                widgets.active.weak_bg_fill   = egui::Color32::TRANSPARENT;
            }
            ui.spacing_mut().button_padding = egui::vec2(0.0, 0.0);

            let selected_fill = selected_pill_fill(dark_mode);
            const SELECTED_INSET:    f32 = 2.0;
            const SELECTED_ROUNDING: f32 = 5.0;

            // Pre-compute every segment's rect so we can interact + paint
            // pill chrome BEFORE rendering each label (the pill must sit
            // under the text, not over it).
            let mut seg_rects: Vec<egui::Rect> = Vec::with_capacity(n);
            let mut seg_x = outer_rect.min.x;
            for i in 0..n {
                let w = if i + 1 == n { last_w } else { per_w };
                seg_rects.push(egui::Rect::from_min_size(
                    egui::pos2(seg_x, outer_rect.min.y),
                    egui::vec2(w, ROW_H),
                ));
                seg_x += w;
            }

            let font_id = egui::TextStyle::Button.resolve(ui.style());

            for (i, (text, variant)) in options.iter().enumerate() {
                let is_selected = *current == *variant;
                let seg_rect    = seg_rects[i];

                // Interact first — gives us hover/press/click without drawing.
                let seg_id = ui.id().with("seg").with(i).with(*text);
                let resp = ui.interact(seg_rect, seg_id, egui::Sense::click());
                // Same scroll-into-view nudge as `control_button` —
                // see comment there.  Tab landing on an off-screen
                // segment would otherwise appear to do nothing
                if resp.gained_focus() {
                    resp.scroll_to_me(None);
                }

                // Pill chrome (selected > pressed > hovered) drawn under text.
                let pill_fill: Option<egui::Color32> = if is_selected {
                    Some(selected_fill)
                } else if resp.is_pointer_button_down_on() {
                    Some(if dark_mode {
                        egui::Color32::from_white_alpha(36)
                    } else {
                        egui::Color32::from_black_alpha(30)
                    })
                } else if resp.hovered() {
                    Some(if dark_mode {
                        egui::Color32::from_white_alpha(22)
                    } else {
                        egui::Color32::from_black_alpha(18)
                    })
                } else {
                    None
                };
                if let Some(fill) = pill_fill {
                    let pill = seg_rect.shrink(SELECTED_INSET);
                    ui.painter().rect_filled(pill, SELECTED_ROUNDING, fill);
                }

                // Keyboard-focus halo for the segment reached via Tab.
                // Drawn ABOVE the selected/hover pill so the focus
                // outline reads even when the segment is also the
                // currently-selected one.  Three stacked stroked
                // rects with decreasing alpha produce a soft glow
                // (matching the control-button halo) so the
                // segmented pickers stay legible without a hard
                // border every time the user tabs through
                if resp.has_focus() {
                    let primary_color = if dark_mode {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::BLACK
                    };
                    let pill = seg_rect.shrink(SELECTED_INSET);
                    let with_alpha = |a: u8| egui::Color32::from_rgba_unmultiplied(
                        primary_color.r(), primary_color.g(), primary_color.b(), a,
                    );
                    for (offset, alpha) in [(0.0_f32, 140_u8), (1.5_f32, 70_u8), (3.0_f32, 28_u8)] {
                        ui.painter().rect_stroke(
                            pill.expand(offset),
                            SELECTED_ROUNDING + offset,
                            egui::Stroke::new(1.0, with_alpha(alpha)),
                        );
                    }
                }

                // Selected text flips to primary; unselected uses default text color.
                let label_color = if is_selected {
                    selected_pill_text(dark_mode)
                } else {
                    ui.visuals().text_color()
                };

                // Paint the label centered in the segment via the painter,
                // matching the segment width we measured for the picker
                // column — `ui.put(rect, Button)` would re-allocate and
                // grow `min_rect`, which we deliberately avoid in this row.
                let galley = ui.painter().layout_no_wrap(
                    text.to_string(),
                    font_id.clone(),
                    label_color,
                );
                let text_pos = egui::pos2(
                    seg_rect.center().x - galley.size().x * 0.5,
                    seg_rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, label_color);

                if resp.clicked() {
                    *current = *variant;
                    changed  = true;
                }
            }

            // Explicitly allocate the outer rect so the parent's cursor
            // advances past picker_w exactly — otherwise nothing has
            // reserved the horizontal space and the scope's min_rect
            // wouldn't include the pickers (ui.put doesn't advance cursor).
            let _ = ui.allocate_rect(outer_rect, egui::Sense::hover());

            // Rounded outline around the picker's outer bounds.  The
            // outer rounding matches `SELECTED_ROUNDING + SELECTED_INSET`
            // so the rounded-rect SELECTED pill sits concentric inside
            // the rounded outer border (`pill = outer.shrink(INSET)`
            // means pill's corner-radius needs to be outer's minus
            // INSET to stay visually concentric).  AppKit's native
            // `NSSegmentedControl` uses the same look.
            let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
            ui.painter().rect_stroke(
                outer_rect,
                SELECTED_ROUNDING + SELECTED_INSET,
                egui::Stroke::new(1.0, stroke_color),
            );
        });
    });
    changed
}

/// Duration row (seconds). Swift's CombinedStepperTextField with `limits: (0, nil)`
/// and step 1.0 — so the ±-button step matches the Stepper control on macOS.
/// Preset chips: one click that moves all four Timing steppers at once.
///
/// Lives inside the Timing card, directly above the steppers it writes,
/// so the effect of a click is visible in the same glance rather than
/// having to be believed.
///
/// **Selection is derived, never stored.** Which chip is lit comes from
/// comparing the four durations in `Settings` against each preset, with
/// the epsilon `SettingsDiff::from` uses on those same fields. There is
/// no sixth "selected preset" field, so there is nothing to migrate,
/// nothing to desync, and nothing that can be stale after the user
/// nudges a stepper by hand: the chip simply goes out.
///
/// **There is no "Custom" chip.** Custom is the absence of a lit pill.
/// A chip that does nothing when clicked is still a Tab stop, and this
/// file already documents that hazard twice.
///
/// Laid out by hand rather than with `ui.horizontal_wrapped` because
/// the pill chrome has to be painted UNDER the label, which means every
/// rect has to be known before anything is drawn. Same technique as
/// `segmented_row`, which is also why the focus halo, the hover fill
/// and the selected colour are literally the same code
pub(super) fn preset_chips(
    ui:       &mut egui::Ui,
    settings: &mut exhale_core::settings::Settings,
) -> bool {
    use exhale_core::presets::{selected, PRESETS};

    let mut changed = false;
    let dark_mode   = ui.visuals().dark_mode;
    let current     = selected(settings);
    let font_id     = egui::TextStyle::Button.resolve(ui.style());

    // 7 and 5, which look arbitrary and are not. Measured at the shipped
    // 308 pt card width with the four-number labels, the five chips come
    // to 297 pt at these values and 331 pt at a roomier 10 and 6, so the
    // difference between them is whether the last chip sits in the row or
    // alone underneath it. Nothing breaks if a future label pushes past
    // the width: the wrap below is the fallback and simply gives back the
    // second row
    const CHIP_PAD_X:  f32 = 7.0;
    const CHIP_GAP:    f32 = 5.0;
    const CHIP_ROW_GAP: f32 = 6.0;
    const INSET:       f32 = 2.0;
    const ROUNDING:    f32 = 5.0;

    // Heading and caption share one line. Two separate lines put four
    // rows of chrome above four steppers, and the caption's line
    // appearing and vanishing as the selection changed made the whole
    // card jump. Merged, the row is a fixed height and reads as one
    // caption for the group
    let mut heading = String::from("Presets");
    if let Some(note) = current.and_then(|i| PRESETS[i].note) {
        heading.push(' ');
        heading.push_str(note);
    }
    ui.label(
        egui::RichText::new(heading)
            .small()
            .color(ui.visuals().weak_text_color()),
    );

    // Lay the text out once, with `PLACEHOLDER` so the same galley can be
    // painted in either the selected or the unselected colour: egui
    // substitutes the fallback colour passed to `Painter::galley` for
    // every `PLACEHOLDER` glyph
    let galleys: Vec<_> = PRESETS
        .iter()
        .map(|p| {
            ui.painter().layout_no_wrap(
                p.label.to_string(),
                font_id.clone(),
                egui::Color32::PLACEHOLDER,
            )
        })
        .collect();

    // Wrap into rows. Never truncate: a clipped pattern label is a
    // different pattern, and `paint_label_with_width`'s single-row
    // ellipsis is exactly the wrong tool here
    let avail  = ui.available_width();
    let origin = ui.cursor().min;
    let mut rects: Vec<egui::Rect> = Vec::with_capacity(PRESETS.len());
    let (mut x, mut y) = (origin.x, origin.y);
    for galley in &galleys {
        let w = galley.size().x + 2.0 * CHIP_PAD_X;
        if x > origin.x && x + w > origin.x + avail {
            x = origin.x;
            y += ROW_H + CHIP_ROW_GAP;
        }
        rects.push(egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, ROW_H)));
        x += w + CHIP_GAP;
    }
    let total_h = (y + ROW_H) - origin.y;

    for (i, preset) in PRESETS.iter().enumerate() {
        let rect = rects[i];
        let is_selected = current == Some(i);

        // Interact before painting, so hover and press states are known
        // by the time the pill goes down under the text
        let resp = ui.interact(
            rect,
            ui.id().with("preset").with(preset.id),
            egui::Sense::click(),
        );
        // Tab landing on a chip scrolled out of view would otherwise
        // look like the key did nothing. Same nudge as `segmented_row`
        if resp.gained_focus() {
            resp.scroll_to_me(None);
        }

        let pill_fill: Option<egui::Color32> = if is_selected {
            Some(selected_pill_fill(dark_mode))
        } else if resp.is_pointer_button_down_on() {
            Some(if dark_mode {
                egui::Color32::from_white_alpha(36)
            } else {
                egui::Color32::from_black_alpha(30)
            })
        } else if resp.hovered() {
            Some(if dark_mode {
                egui::Color32::from_white_alpha(22)
            } else {
                egui::Color32::from_black_alpha(18)
            })
        } else {
            None
        };
        let pill = rect.shrink(INSET);
        if let Some(fill) = pill_fill {
            ui.painter().rect_filled(pill, ROUNDING, fill);
        } else {
            // Unselected chips need a visible edge. The pickers get one
            // from the container outline around the whole control; a
            // wrapped chip row has no container, so without this an
            // unselected chip is bare text and does not read as
            // clickable at all
            ui.painter().rect_stroke(
                pill,
                ROUNDING,
                egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            );
        }

        if resp.has_focus() {
            let primary = if dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK };
            let with_alpha = |a: u8| egui::Color32::from_rgba_unmultiplied(
                primary.r(), primary.g(), primary.b(), a,
            );
            for (offset, alpha) in [(0.0_f32, 140_u8), (1.5_f32, 70_u8), (3.0_f32, 28_u8)] {
                ui.painter().rect_stroke(
                    pill.expand(offset),
                    ROUNDING + offset,
                    egui::Stroke::new(1.0, with_alpha(alpha)),
                );
            }
        }

        let label_color = if is_selected {
            selected_pill_text(dark_mode)
        } else {
            ui.visuals().text_color()
        };
        let galley = galleys[i].clone();
        let text_pos = egui::pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(text_pos, galley, label_color);

        // Space and Enter reach here too: egui 0.29 sets
        // `fake_primary_click` on any focused `Sense::click` response,
        // and `clicked()` folds that in. Covered by
        // `space_activates_a_focused_chip`
        if resp.clicked() {
            preset.apply(settings);
            changed = true;
        }
    }

    // Reserve the space the chips occupy. `ui.painter()` draws without
    // advancing the cursor, so without this the duration rows below
    // would be laid out on top of them
    let _ = ui.allocate_rect(
        egui::Rect::from_min_size(origin, egui::vec2(avail, total_h)),
        egui::Sense::hover(),
    );

    #[cfg(test)]
    test_hooks::record_chip_rects(rects);

    changed
}

/// The pacing readout: what the current timing settings actually work
/// out to, and how that sits against the range with direct
/// experimental support.
///
/// Every string comes from [`exhale_core::pacing::readout_lines`],
/// which is where the copy is tested. This function only paints.
///
/// Rendered unprompted, never behind a hover or a disclosure triangle.
/// The reason is specific rather than stylistic: exhale's shipped
/// default is 4.0 breaths a minute, below the tested range, and a
/// disclosure only reaches the people who go looking. The default is
/// imposed on everyone who never opens this panel at all, so the one
/// place it can honestly be qualified is the first place it is shown.
///
/// A tooltip could not carry this either. `egui`'s tooltip width
/// clamps to `ctx.screen_rect()`, which here is a 360 pt window, so
/// anything this long truncates
pub(super) fn pacing_readout(ui: &mut egui::Ui, settings: &exhale_core::settings::Settings) {
    let lines = exhale_core::pacing::readout_lines(settings);
    if lines.is_empty() {
        return;
    }
    ui.add_space(2.0);
    // Scoped so the tighter spacing does not leak into the rest of the
    // card. `section` sets `item_spacing.y = ROW_GAP` for control rows,
    // which is right between a slider and the next slider and wrong
    // between two sentences: at 8 pt these read as three unrelated
    // statements rather than one paragraph
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        for line in lines {
            // Wrapping, not truncating. `paint_label_with_width` sets
            // `max_rows: 1` with an ellipsis, which is right for a
            // control label in a fixed column and wrong for a sentence
            // whose second half is the qualification
            ui.label(
                egui::RichText::new(line)
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        }
    });
}

pub(super) fn duration_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut f64) -> bool {
    stepper_row(ui, label, help, None, value, 1.0, 0.0, None, ValueScale::Identity)
}

/// Randomised-timing percentage row.  Stored in Settings as 0.0–1.0; Swift
/// displays it multiplied by 100 with a stepper step of 1 % (== 0.01 in
/// storage).  `ValueScale::Percent` handles the ×100 / ÷100 conversion on
/// both read and write so the displayed/entered value is always a percent.
pub(super) fn pct_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut f64) -> bool {
    stepper_row(ui, label, help, None, value, 1.0, 0.0, None, ValueScale::Percent)
}

/// How a stored value is mapped to the displayed/entered number.
///
/// Swift's CombinedStepperTextField is parameterised by a Binding that
/// transforms the stored value before it reaches the TextField.  We
/// accomplish the same thing with a scale enum so callers don't have to
/// open-code `*100` / `÷100` / `(x-1)*100` conversions everywhere, and the
/// stepper's `step` field still describes the *displayed* step (e.g. 1 %).
#[derive(Clone, Copy)]
pub(super) enum ValueScale {
    /// `display = stored`
    Identity,
    /// `display = stored * 100` — randomised-timing sliders stored as fractions.
    Percent,
    /// `display = (stored - 1) * 100` — drift multiplier stored as e.g. 1.01.
    DriftPercent,
}

impl ValueScale {
    fn to_display(self, stored: f64) -> f64 {
        match self {
            Self::Identity     => stored,
            Self::Percent      => stored * 100.0,
            Self::DriftPercent => (stored - 1.0) * 100.0,
        }
    }
    #[allow(clippy::wrong_self_convention)]
    fn from_display(self, display: f64) -> f64 {
        match self {
            Self::Identity     => display,
            Self::Percent      => display / 100.0,
            Self::DriftPercent => 1.0 + display / 100.0,
        }
    }
}

/// SwiftUI's `CombinedStepperTextField`: a fixed-width numeric TextField with
/// a two-button vertical Stepper to its right and an optional left-hand hint
/// ("0 = off").  `step`, `min`, and `max` are in the *displayed* unit; the
/// `scale` enum maps that display value to/from the stored `value`.
///
/// The buffer is persisted in egui's temp data keyed by `label` so typing a
/// partial number ("1." on the way to "1.25") doesn't get clobbered by a
/// redraw.  When the field loses focus (or the stepper nudges the value) we
/// canonicalise the buffer via `format_num` so extraneous zeros/decimals get
/// cleaned up — matching Swift's NumberFormatter with `maximumFractionDigits: 3`.
#[allow(clippy::too_many_arguments)]
pub(super) fn stepper_row(
    ui:     &mut egui::Ui,
    label:  &str,
    help:   &str,
    hint:   Option<&str>,
    value:  &mut f64,
    step:   f64,
    min:    f64,
    max:    Option<f64>,
    scale:  ValueScale,
) -> bool {
    let mut changed = false;
    let resp = ui.horizontal(|ui| {
        // Zero item_spacing.x at the row level; we insert explicit
        // `add_space` between components so the right-alignment math
        // is exact and `widgets_w` accounts for every gap placed
        ui.spacing_mut().item_spacing.x = 0.0;

        // Label column — painter-direct, fixed LABEL_W wide.
        paint_label(ui, label);

        let stepper_btn_w = 14.0_f32;
        let field_gap:  f32 = 2.0;   // between field and ± buttons
        let hint_gap:   f32 = 4.0;   // between hint text and field
        let hint_w: f32 = if let Some(h) = hint {
            let font = egui::TextStyle::Small.resolve(ui.style());
            ui.fonts(|f| f.layout_no_wrap(h.to_string(), font, egui::Color32::WHITE).size().x).ceil()
        } else { 0.0 };
        // Exact total width of the trailing column: hint + hint_gap + field
        // + field_gap + stepper buttons.  Every component lines up with an
        // explicit add_space so this equals the actual placed width.
        let trailing_gap = if hint.is_some() { hint_gap } else { 0.0 };
        let widgets_w = hint_w + trailing_gap + STEPPER_FIELD_W + field_gap + stepper_btn_w;

        let remaining = ui.available_width();
        let gap = (remaining - widgets_w).max(0.0);
        if gap > 0.0 { ui.add_space(gap); }

        // Hint (left of field)
        if let Some(h) = hint {
            ui.label(egui::RichText::new(h).color(egui::Color32::GRAY).small());
            ui.add_space(hint_gap);
        }

        // Numeric text field
        let displayed = scale.to_display(*value);
        let max_disp  = max;
        let edit_id   = egui::Id::new(("stepper_buf", label));
        let focused   = ui.ctx().memory(|m| m.focused() == Some(edit_id));
        let mut buf: String = ui.data_mut(|d| {
            d.get_temp::<String>(edit_id).unwrap_or_else(|| format_num(displayed))
        });

        let field_resp = ui.add_sized(
            egui::vec2(STEPPER_FIELD_W, ROW_H),
            egui::TextEdit::singleline(&mut buf)
                .id(edit_id)
                .margin(egui::vec2(4.0, 2.0)),
        );
        // Scroll into view when Tab moves focus to this field while
        // it's off-screen — same pattern as `control_button` /
        // segmented-picker segment focus handling
        if field_resp.gained_focus() {
            field_resp.scroll_to_me(None);
        }
        if field_resp.changed() {
            if let Ok(parsed) = buf.trim().parse::<f64>() {
                let mut disp = parsed.max(min);
                if let Some(m) = max_disp { disp = disp.min(m); }
                let v = scale.from_display(disp);
                if (v - *value).abs() > f64::EPSILON {
                    *value  = v;
                    changed = true;
                }
            }
        }

        // Arrow-key stepping: while the text field is focused, the
        // up / down arrows nudge the value by one `step` per press.
        // `consume_key` drains EVERY matching event in the queue
        // this frame (including key-repeat fires from a held key)
        // so hold-to-step works naturally — one tick per system
        // repeat interval.  Singleline `TextEdit` doesn't do
        // anything useful with vertical arrows, so consuming them
        // here doesn't break editing
        let mut arrow_stepped = false;
        if field_resp.has_focus() {
            let mut delta_steps = 0_i32;
            ui.input_mut(|i| {
                while i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                    delta_steps += 1;
                }
                while i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                    delta_steps -= 1;
                }
            });
            if delta_steps != 0 {
                let delta = step * delta_steps as f64;
                let displayed = scale.to_display(*value);
                let mut new_disp = (displayed + delta).max(min);
                if let Some(m) = max_disp { new_disp = new_disp.min(m); }
                let v = scale.from_display(new_disp);
                if (v - *value).abs() > f64::EPSILON {
                    *value = v;
                    changed = true;
                    arrow_stepped = true;
                }
            }
        }

        // Gap between field and ± buttons (now explicit since item_spacing = 0).
        ui.add_space(field_gap);

        // Stepper buttons (right of field) — pass the TextEdit's actual
        // rendered rect so the stepper's vertical bounds match the field's
        // visible bounds exactly (otherwise `ROW_H`-sized stepper overhangs
        // the TextEdit's slightly-shorter visible rectangle).
        let field_rect = field_resp.rect;
        let stepper_changed = stepper_buttons(
            ui,
            field_rect,
            label,  // row_salt — makes interact IDs unique per stepper row
            &scale.to_display(*value),
            step, min, max_disp,
        );
        if let Some(new_disp) = stepper_changed {
            let v = scale.from_display(new_disp);
            if (v - *value).abs() > f64::EPSILON {
                *value  = v;
                changed = true;
            }
        }

        // Canonicalise the buffer when the field isn't focused, or when the
        // stepper just nudged the value (button click OR arrow-key step) —
        // prevents stale text hanging around after external state changes
        // (reset, cross-row effects) and keeps the on-screen text in
        // sync after an arrow nudge while focus remains on the field
        if !focused || stepper_changed.is_some() || arrow_stepped {
            buf = format_num(scale.to_display(*value));
        }
        ui.data_mut(|d| d.insert_temp(edit_id, buf));
    }).response;
    resp.on_hover_text(help);
    changed
}

/// Vertically stacked ▲/▼ Stepper buttons sized to match the adjacent
/// TextEdit's physical rect exactly.  `field_rect` is the TextEdit's
/// response rect — we use its `top()` and `bottom()` directly rather than
/// the parent UI's `ROW_H` so the stepper's top and bottom edges align with
/// the field's visible frame, never overhanging top or bottom.
///
/// Button widgets handle clicks and draw the chrome (fill + stroke +
/// hover/press states); triangles are drawn geometrically with the painter
/// because egui's default font (Ubuntu) doesn't include the ▲ U+25B2 /
/// ▼ U+25BC glyphs — they rendered as missing-glyph tofu boxes.
pub(super) fn stepper_buttons(
    ui:         &mut egui::Ui,
    field_rect: egui::Rect,
    row_salt:   &str,
    value:      &f64,
    step:       f64,
    min:        f64,
    max:        Option<f64>,
) -> Option<f64> {
    let max_v = max.unwrap_or(f64::MAX);
    let btn_w: f32 = 13.0;
    let total_h = field_rect.height();

    // Reserve horizontal space WITHOUT creating a widget response at the
    // full rect — `allocate_exact_size(Sense::hover())` was registering an
    // interaction zone at the whole column that could absorb pointer
    // events ahead of the per-half `ui.interact` calls below, resulting
    // in clicks never registering for the stepper halves.  `allocate_space`
    // only advances the cursor; the actual hit-testing is done exclusively
    // by the two `ui.interact` calls, whose IDs are unique to each half.
    let (_, alloc_rect) = ui.allocate_space(egui::vec2(btn_w, total_h));
    let rect = egui::Rect::from_min_size(
        egui::pos2(alloc_rect.left(), field_rect.top()),
        egui::vec2(btn_w, total_h),
    );
    let half_h  = (total_h * 0.5).floor();
    let top_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(btn_w, half_h),
    );
    let bot_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top() + half_h),
        egui::vec2(btn_w, total_h - half_h),
    );

    // Hit-testing via `ui.interact` — this is the ONLY way to get pixel-
    // perfect sub_rects.  Debug logs proved `egui::Button` ignores
    // ui.put's max_rect and draws at its own desired_size (empty-text
    // galley line-height ≈ 15 px), overhanging the 9 px sub_rect by 6 px
    // below — exactly the "gray below the input" artifact.  With raw
    // interact + painter chrome, the rect we pass IS the rect drawn.
    // Scope the interact IDs by `row_salt` (the stepper_row's label) so
    // every stepper in the window has a unique ID pair.  Using `ui.id()`
    // alone gave every stepper the SAME id because egui 0.29's default
    // UiBuilder has no id_salt, so sibling `ui.horizontal()` children of
    // a given parent all share the parent's id.  That caused egui's
    // click-tracking to silently drop every click because it couldn't
    // disambiguate which stepper was hit.
    let row_id  = ui.id().with(row_salt);
    let up_resp = ui.interact(top_rect, row_id.with("stepper_up"), egui::Sense::click());
    let dn_resp = ui.interact(bot_rect, row_id.with("stepper_dn"), egui::Sense::click());
    #[cfg(test)]
    test_hooks::record_stepper_rects(top_rect, bot_rect);

    // When Tab moves focus to a stepper half that's currently
    // scrolled out of the settings ScrollArea's viewport, nudge
    // the viewport just enough to bring it into view — same
    // pattern as `control_button` / segmented-picker segment.
    // `gained_focus()` is true only on the FRAME focus arrived
    // so we don't re-scroll every subsequent frame.
    if up_resp.gained_focus() { up_resp.scroll_to_me(None); }
    if dn_resp.gained_focus() { dn_resp.scroll_to_me(None); }

    paint_stepper_chrome(ui, top_rect, StepperDir::Up,   up_resp.hovered(), up_resp.is_pointer_button_down_on());
    paint_stepper_chrome(ui, bot_rect, StepperDir::Down, dn_resp.hovered(), dn_resp.is_pointer_button_down_on());

    // Triangles as small filled polygons (font-independent).
    let tri_color = ui.visuals().text_color();
    paint_triangle(ui, top_rect, StepperDir::Up,   tri_color);
    paint_triangle(ui, bot_rect, StepperDir::Down, tri_color);

    // Keyboard-focus halo for the stepper halves.  The button rects
    // here are tiny (~13×9 px) so the layered glow that works on
    // the top-row control buttons reads too soft — bump the inner
    // ring alpha to fully-opaque so even at glance distance the
    // user can see which half of the stepper Tab landed on (and
    // why pressing Tab three more times before hitting the next
    // control is the expected behaviour: TextEdit → ▲ → ▼ → next
    // row).  Two stacked stroked rects with a brighter inner
    // edge + a soft outer falloff
    let dark_mode = ui.visuals().dark_mode;
    let halo_color = if dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK };
    let with_alpha = |a: u8| egui::Color32::from_rgba_unmultiplied(
        halo_color.r(), halo_color.g(), halo_color.b(), a,
    );
    let paint_halo = |rect: egui::Rect| {
        // Inner crisp outline: opaque so the focused half is
        // unambiguously highlighted even with the small footprint
        for (offset, alpha, stroke_w) in [
            (0.0_f32,  220_u8, 1.5_f32),
            (1.5_f32,  120_u8, 1.0_f32),
            (3.0_f32,   55_u8, 1.0_f32),
        ] {
            ui.painter().rect_stroke(
                rect.expand(offset),
                3.0 + offset,
                egui::Stroke::new(stroke_w, with_alpha(alpha)),
            );
        }
    };
    if up_resp.has_focus() { paint_halo(top_rect); }
    if dn_resp.has_focus() { paint_halo(bot_rect); }

    // Press-and-hold auto-repeat — matches macOS NSStepper / SwiftUI
    // Stepper.  A click fires once on press-down; holding the button past
    // `INITIAL_DELAY` starts a repeat that fires every `REPEAT_INTERVAL`
    // until release.  Values match AppKit's NSStepper defaults
    // (`autorepeatDelay = 0.4s`, `autorepeatInterval = 0.075s`)
    const INITIAL_DELAY:   Duration = Duration::from_millis(400);
    const REPEAT_INTERVAL: Duration = Duration::from_millis(75);

    let up_step = step;
    let dn_step = -step;
    let mut new_val = None;
    if let Some(delta) = stepper_hold_tick(ui, &up_resp, INITIAL_DELAY, REPEAT_INTERVAL, up_step) {
        new_val = Some((*value + delta).clamp(min, max_v));
    }
    if let Some(delta) = stepper_hold_tick(ui, &dn_resp, INITIAL_DELAY, REPEAT_INTERVAL, dn_step) {
        new_val = Some((*value + delta).clamp(min, max_v));
    }
    new_val
}

/// State held in egui memory for one stepper half between frames.
/// Records when the press started and when we last fired a step, so the
/// hold-repeat logic can drive auto-repeat without recomputing time
/// deltas from scratch each frame
#[derive(Clone, Copy)]
pub(super) struct StepperHoldState {
    press_start: Instant,
    last_tick:   Instant,
}

/// Decide whether one stepper half should fire a step this frame.
///
/// Returns `Some(delta)` (the signed step amount) when the button
/// should step the value:
///   * on the initial press-down edge,
///   * on each `repeat_interval` boundary after the press has been
///     held for at least `initial_delay`,
///   * on a same-frame press-and-release (catches the synthetic
///     down+up-in-one-frame events used in unit tests, and real
///     trackpad taps that egui collapses to a single frame).
///
/// Returns `None` while the button is idle or in the initial-delay
/// dead-zone of a hold.  Requests a repaint deadline while the
/// button is held so the event loop wakes in time to fire the next
/// auto-repeat tick
pub(super) fn stepper_hold_tick(
    ui:              &egui::Ui,
    resp:            &egui::Response,
    initial_delay:   Duration,
    repeat_interval: Duration,
    step:            f64,
) -> Option<f64> {
    let id   = resp.id;
    let now  = Instant::now();
    let prev = ui.data(|d| d.get_temp::<StepperHoldState>(id));
    // The stepper-button hit rect is small (13 × 11 pt per half).
    // `Response::is_pointer_button_down_on` returns false the moment
    // the cursor drifts off that rect — which during a slow-and-shaky
    // hold flickers between true/false at the boundary.  Each flicker
    // would clear our hold state and re-treat the next frame as a
    // fresh press, dropping the user back into the initial-delay
    // dead-zone and producing the "stops after 3-4 increments" bug.
    //
    // Robust signal: combine "the primary pointer button is currently
    // down somewhere" (a global state independent of widget hit-test)
    // with our own memo "the press started on this widget" (set
    // either by an in-rect press-down THIS frame or by an existing
    // `StepperHoldState` carried over from a previous frame).  As
    // long as the user keeps the button down, `held` stays true
    // regardless of small cursor drift
    let primary_down       = ui.ctx().input(|i| i.pointer.primary_down());
    let pressed_in_widget  = resp.is_pointer_button_down_on();
    let press_started_here = pressed_in_widget || prev.is_some();
    let held               = primary_down && press_started_here;

    if held {
        if let Some(state) = prev {
            // Already holding — check whether enough time has passed
            // since press for auto-repeat to engage, and whether we're
            // past the next `repeat_interval` boundary since the last
            // tick we fired.
            ui.ctx().request_repaint_after(repeat_interval);
            if now.duration_since(state.press_start) >= initial_delay
                && now.duration_since(state.last_tick) >= repeat_interval
            {
                ui.data_mut(|d| d.insert_temp(id, StepperHoldState {
                    press_start: state.press_start,
                    last_tick:   now,
                }));
                return Some(step);
            }
            None
        } else {
            // Fresh press-down — fire one immediate step and start
            // tracking the hold.  This matches NSStepper, which sends
            // its `action` on press-down, not on release.
            ui.data_mut(|d| d.insert_temp(id, StepperHoldState {
                press_start: now,
                last_tick:   now,
            }));
            ui.ctx().request_repaint_after(initial_delay);
            Some(step)
        }
    } else {
        if prev.is_some() {
            // Released — we already fired on the press-down edge and
            // any auto-repeat ticks during the hold, so just clear the
            // hold state.  Don't re-fire on release.
            ui.data_mut(|d| d.remove::<StepperHoldState>(id));
            None
        } else if resp.clicked() {
            // Same-frame press+release (`is_pointer_button_down_on()` is
            // false at end-of-frame, but `clicked()` registered a full
            // down+up cycle).  Trackpad taps and synthetic test inputs
            // land here — fire a single step
            Some(step)
        } else {
            None
        }
    }
}

pub(super) fn paint_stepper_chrome(
    ui:      &egui::Ui,
    rect:    egui::Rect,
    dir:     StepperDir,
    hovered: bool,
    pressed: bool,
) {
    let v = ui.visuals();
    let style = if pressed {
        &v.widgets.active
    } else if hovered {
        &v.widgets.hovered
    } else {
        &v.widgets.inactive
    };
    // Round only the outer corners so the up + down halves merge into a
    // single rounded-rect column with a flush mid-edge.  A 3-px radius
    // matches the subtle macOS-native NSStepper look.
    const STEPPER_RADIUS: f32 = 3.0;
    let rounding = match dir {
        StepperDir::Up   => egui::Rounding { nw: STEPPER_RADIUS, ne: STEPPER_RADIUS, sw: 0.0, se: 0.0 },
        StepperDir::Down => egui::Rounding { nw: 0.0, ne: 0.0, sw: STEPPER_RADIUS, se: STEPPER_RADIUS },
    };
    ui.painter().rect(
        rect,
        rounding,
        style.weak_bg_fill,
        style.bg_stroke,
    );
}

#[derive(Copy, Clone)]
pub(super) enum StepperDir { Up, Down }

pub(super) fn paint_triangle(ui: &egui::Ui, rect: egui::Rect, dir: StepperDir, color: egui::Color32) {
    let c = rect.center();
    let half_w: f32 = 3.0;
    let half_h: f32 = 2.0;
    let points = match dir {
        StepperDir::Up => vec![
            egui::pos2(c.x - half_w, c.y + half_h),
            egui::pos2(c.x + half_w, c.y + half_h),
            egui::pos2(c.x,          c.y - half_h),
        ],
        StepperDir::Down => vec![
            egui::pos2(c.x - half_w, c.y - half_h),
            egui::pos2(c.x + half_w, c.y - half_h),
            egui::pos2(c.x,          c.y + half_h),
        ],
    };
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

/// Swift NumberFormatter equivalent: decimal with `maximumFractionDigits: 3`,
/// `usesGroupingSeparator = false`, trailing zeros stripped so `5.0` shows as
/// "5" and `25.50` shows as "25.5".
pub(super) fn format_num(v: f64) -> String {
    if v.fract().abs() < 1e-9 {
        // Whole number path — avoid the "5.000" → "5" string thrash for the
        // common case where every setting starts as an integer default.
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{:.3}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

// ─── Color conversion ─────────────────────────────────────────────────────────
// Settings stores sRGB [f32;4] in 0..1 (not linear), matching SwiftUI's Color
// values (NSColor/CGColor in the deviceRGB space). The shader treats channel
// values as sRGB and writes them to an 8-bit UNORM framebuffer as-is, which
// the OS compositor displays as sRGB — identical to Swift's MTKView
// (`colorPixelFormat = .bgra8Unorm`) pipeline. Storing sRGB also makes
// gradient lerps interpolate in gamma space, matching SwiftUI's
// LinearGradient/RadialGradient default behaviour.

pub(super) fn to_color32(c: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (c[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (c[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

pub(super) fn from_color32(c: egui::Color32) -> [f32; 4] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
        c.a() as f32 / 255.0,
    ]
}

/// Like from_color32 but forces alpha=1.0 (for inhale/exhale colors).
pub(super) fn from_color32_opaque(c: egui::Color32) -> [f32; 4] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
        1.0,
    ]
}

// ─── Test hooks ─────────────────────────────────────────────────────────
//
// A handful of test-only atomics and helpers so unit tests can observe
// where stepper_buttons actually placed its interact rects during the
// previous frame.  Used only under `#[cfg(test)]`.
#[cfg(test)]
pub(super) mod test_hooks {
    use std::cell::RefCell;
    thread_local! {
        static LAST: RefCell<Option<(egui::Rect, egui::Rect)>> = const { RefCell::new(None) };
    }

    pub fn record_stepper_rects(top: egui::Rect, bot: egui::Rect) {
        LAST.with(|c| *c.borrow_mut() = Some((top, bot)));
    }

    pub fn take_stepper_rects() -> Option<(egui::Rect, egui::Rect)> {
        LAST.with(|c| c.borrow_mut().take())
    }

    // Unlike the stepper hook above, the chip hook is `cfg(test)`: it is
    // written on every frame the chips are laid out, and a thread-local
    // borrow plus a Vec clone per frame is not worth paying for in a
    // release build to support a test
    #[cfg(test)]
    thread_local! {
        static CHIPS: RefCell<Option<Vec<egui::Rect>>> = const { RefCell::new(None) };
    }

    #[cfg(test)]
    pub fn record_chip_rects(rects: Vec<egui::Rect>) {
        CHIPS.with(|c| *c.borrow_mut() = Some(rects));
    }

    #[cfg(test)]
    pub fn take_chip_rects() -> Option<Vec<egui::Rect>> {
        CHIPS.with(|c| c.borrow_mut().take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(scale: ValueScale, stored: f64) {
        let displayed = scale.to_display(stored);
        let back      = scale.from_display(displayed);
        assert!((back - stored).abs() < 1e-9,
            "{:?}: {} → {} → {} (roundtrip drift {})",
            scale_name(scale), stored, displayed, back, back - stored);
    }

    fn scale_name(s: ValueScale) -> &'static str {
        match s {
            ValueScale::Identity     => "Identity",
            ValueScale::Percent      => "Percent",
            ValueScale::DriftPercent => "DriftPercent",
        }
    }

    /// sRGB relative luminance, WCAG 2.1 definition.
    fn luminance(c: egui::Color32) -> f64 {
        let f = |v: u8| {
            let v = v as f64 / 255.0;
            if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }

    fn contrast(a: egui::Color32, b: egui::Color32) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        let (hi, lo) = if x > y { (x, y) } else { (y, x) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Source-over composite of `fg` (with alpha) onto opaque `bg`.
    fn over(fg: egui::Color32, bg: egui::Color32) -> egui::Color32 {
        // `Color32::from_rgba_unmultiplied` stores premultiplied bytes, so
        // read the alpha back out and un-premultiply before compositing,
        // or the result is wrong in exactly the direction that flatters
        let a = fg.a() as f64 / 255.0;
        let ch = |f: u8, b: u8| {
            let straight = if a > 0.0 { (f as f64 / 255.0) / a } else { 0.0 };
            ((straight * a + (b as f64 / 255.0) * (1.0 - a)) * 255.0).round() as u8
        };
        egui::Color32::from_rgb(ch(fg.r(), bg.r()), ch(fg.g(), bg.g()), ch(fg.b(), bg.b()))
    }

    #[test]
    fn selected_pill_text_meets_wcag_aa_over_any_backdrop() {
        // The handoff estimated the dark-mode selected pill at ~4.0:1,
        // under the 4.5:1 AA threshold for body-size text, and asked for
        // a measurement. Measured, it clears: the pill is drawn at alpha
        // 230, so what shows through the card and the vibrancy behind it
        // moves the result by less than a point of contrast.
        //
        // Swept across every possible backdrop rather than one assumed
        // desktop grey, because the vibrancy material composites against
        // whatever is behind the window and nobody controls that. Both
        // segmented pickers and preset chips use these colours, so this
        // covers both
        for dark_mode in [true, false] {
            let text = selected_pill_text(dark_mode);
            let (mut worst, mut worst_bg) = (f64::INFINITY, 0u8);
            for b in 0..=255u8 {
                let backdrop = egui::Color32::from_gray(b);
                let card = over(card_fill(dark_mode), backdrop);
                let pill = over(selected_pill_fill(dark_mode), card);
                let c = contrast(text, pill);
                if c < worst {
                    worst = c;
                    worst_bg = b;
                }
            }
            assert!(
                worst >= 4.5,
                "dark_mode={dark_mode}: selected pill text is {worst:.2}:1 over backdrop \
                 gray {worst_bg}, below the 4.5:1 AA threshold"
            );
        }
    }

    #[test]
    fn chip_text_meets_wcag_aa_on_the_backdrop_the_app_controls() {
        // With blur unavailable (older Windows, GNOME, EXHALE_DISABLE_BLUR)
        // the window is opaque and `clear_color_for_theme` picks the
        // backdrop, so this is a contrast floor exhale can actually
        // promise rather than one that depends on the wallpaper
        for (dark_mode, clear) in [
            (true,  egui::Color32::from_gray((0.12 * 255.0) as u8)),
            (false, egui::Color32::from_gray((0.96 * 255.0) as u8)),
        ] {
            let card = over(card_fill(dark_mode), clear);
            let body = if dark_mode {
                egui::Color32::from_rgb(235, 235, 240)
            } else {
                egui::Color32::from_rgb(20, 20, 22)
            };
            assert!(
                contrast(body, card) >= 4.5,
                "dark_mode={dark_mode}: unselected chip text is {:.2}:1 on the opaque backdrop",
                contrast(body, card)
            );
            let pill = over(selected_pill_fill(dark_mode), card);
            assert!(
                contrast(selected_pill_text(dark_mode), pill) >= 4.5,
                "dark_mode={dark_mode}: selected chip text is {:.2}:1 on the opaque backdrop",
                contrast(selected_pill_text(dark_mode), pill)
            );
        }
    }

    #[test]
    fn unselected_chip_text_holds_up_wherever_the_vibrancy_can_land() {
        // Unselected chips paint `visuals().text_color()` directly on the
        // translucent card, so unlike the selected pill they inherit
        // whatever the OS blur composites behind the window.
        //
        // The bound swept here is that the material does not invert: a
        // dark material never renders lighter than mid-grey and a light
        // one never renders darker. That is what "dark material" means,
        // and it is the strongest honest assumption available from
        // inside the process, since `NSVisualEffectView` gives back no
        // sampled colour to test against.
        //
        // Past that bound the guarantee does lapse: near-white behind a
        // dark-mode window measures about 3.0:1. That is a property of
        // every `ui.label` in this window rather than anything the chips
        // introduced, and it is the same accessibility debt as the
        // missing AccessKit tree. Recorded here so it is a known number
        // instead of a surprise
        for (dark_mode, range) in [(true, 0..=128u8), (false, 128..=255u8)] {
            let text = if dark_mode {
                egui::Color32::from_rgb(235, 235, 240)
            } else {
                egui::Color32::from_rgb(20, 20, 22)
            };
            let mut worst = f64::INFINITY;
            for b in range {
                let card = over(card_fill(dark_mode), egui::Color32::from_gray(b));
                worst = worst.min(contrast(text, card));
            }
            assert!(
                worst >= 4.5,
                "dark_mode={dark_mode}: unselected chip text bottoms out at {worst:.2}:1"
            );
        }
    }

    #[test]
    fn identity_is_a_no_op() {
        for v in [0.0, 0.5, 1.0, 5.0, 100.0, -3.0] {
            assert_eq!(ValueScale::Identity.to_display(v), v);
            assert_eq!(ValueScale::Identity.from_display(v), v);
        }
    }

    #[test]
    fn percent_scales_by_100() {
        assert!((ValueScale::Percent.to_display(0.25) - 25.0).abs() < 1e-9);
        assert!((ValueScale::Percent.from_display(25.0) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn drift_percent_shifts_by_one() {
        // 1.01 stored → 1.0 displayed (1 % drift).
        assert!((ValueScale::DriftPercent.to_display(1.01) - 1.0).abs() < 1e-9);
        // 1 % displayed → 1.01 stored.
        assert!((ValueScale::DriftPercent.from_display(1.0) - 1.01).abs() < 1e-9);
        // 0 % displayed → 1.0 stored (no drift).
        assert!((ValueScale::DriftPercent.from_display(0.0) - 1.0).abs() < 1e-9);
    }

    /// The Drift row shows a percentage, never the multiplier it stores.
    /// Zero must mean "no drift" so that "off" is legible without the user
    /// knowing anything about how it is stored, and one 0.1 step up from off
    /// must land on 1.001 rather than anything coarser.
    #[test]
    fn drift_zero_is_off_and_one_step_is_a_tenth_of_a_percent() {
        const STEP: f64 = 0.1; // matches the Drift stepper_row in settings_window.rs

        // Off: the user sees 0, the controller multiplies by 1.0, a no-op
        assert_eq!(ValueScale::DriftPercent.from_display(0.0), 1.0);
        assert_eq!(ValueScale::DriftPercent.to_display(1.0), 0.0);

        // One press of the up arrow from off
        let after_one = ValueScale::DriftPercent.from_display(0.0 + STEP);
        assert!(
            (after_one - 1.001).abs() < 1e-12,
            "one 0.1 % step should store 1.001, got {after_one}"
        );

        // And it keeps stepping in tenths rather than snapping to whole percents
        let after_two = ValueScale::DriftPercent.from_display(0.0 + STEP * 2.0);
        assert!((after_two - 1.002).abs() < 1e-12, "two steps should store 1.002, got {after_two}");

        // Ten steps reaches the old minimum, 1 %
        let after_ten = ValueScale::DriftPercent.from_display(0.0 + STEP * 10.0);
        assert!((after_ten - 1.01).abs() < 1e-12, "ten steps should store 1.01, got {after_ten}");
    }

    #[test]
    fn round_trips_at_sample_points() {
        for stored in [0.0, 0.1, 0.5, 1.0, 1.01, 2.5, 5.0, 100.0] {
            round_trip(ValueScale::Identity, stored);
            round_trip(ValueScale::Percent, stored);
            round_trip(ValueScale::DriftPercent, stored);
        }
    }

    #[test]
    fn display_then_store_is_idempotent() {
        // The settings UI does this: read stored → show → user edits →
        // convert back → store.  If the user doesn't change anything,
        // stored should equal its prior value to machine epsilon.
        let cases = [
            (ValueScale::Identity,     5.0_f64),
            (ValueScale::Percent,      0.42_f64),
            (ValueScale::DriftPercent, 1.05_f64),
        ];
        for (scale, stored) in cases {
            let displayed = scale.to_display(stored);
            // Simulate the user re-entering the same displayed value.
            let stored_again = scale.from_display(displayed);
            assert!((stored_again - stored).abs() < 1e-9,
                "{:?}: round-trip drift on {stored}", scale_name(scale));
        }
    }
}
