// overlay.wgsl — full port of OverlayShaders.metal
//
// Uniform struct layout (112 bytes).
// WGSL auto-pads 12 bytes between ripple_enabled and background_color so that
// background_color sits at offset 64 (vec4<f32> alignment = 16).  The Rust
// OverlayUniforms struct adds the same 12 bytes as explicit _pad0/1/2 fields
// so both sides agree exactly.

struct OverlayUniforms {
    viewport_size:         vec2<f32>,  // offset  0
    overlay_opacity:       f32,        // offset  8
    background_opacity:    f32,        // offset 12
    max_circle_scale:      f32,        // offset 16
    shape:                 u32,        // offset 20
    gradient_mode:         u32,        // offset 24
    phase:                 u32,        // offset 28
    progress:              f32,        // offset 32
    hold_time:             f32,        // offset 36
    rectangle_scale:       f32,        // offset 40
    circle_gradient_scale: f32,        // offset 44
    ripple_enabled:        u32,        // offset 48
    display_mode:          u32,        // offset 52  0=normal 1=paused 2=stopped
    // WGSL inserts 8 bytes of implicit padding here (offsets 56–63)
    // so that the vec4 below lands at offset 64.
    background_color:      vec4<f32>,  // offset 64
    inhale_color:          vec4<f32>,  // offset 80
    exhale_color:          vec4<f32>,  // offset 96
}

@group(0) @binding(0)
var<uniform> u: OverlayUniforms;

// ─── Vertex shader ────────────────────────────────────────────────────────────

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0)       ndc:      vec2<f32>,
}

/// Fullscreen triangle — no vertex buffer required.
/// Three vertices cover the entire clip-space quad.
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    let pos = positions[vi];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.ndc      = pos;
    return out;
}

// ─── Helper functions ─────────────────────────────────────────────────────────

fn lerp_color(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return a + (b - a) * t;
}

fn clamp01(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn pixel_from_ndc(ndc: vec2<f32>) -> vec2<f32> {
    let uv = ndc * 0.5 + 0.5;
    return uv * u.viewport_size;
}

/// Premultiply alpha and apply overlay opacity in one step.
/// Matches Metal's `applyOverlayOpacityPremultiplied`.
fn apply_premultiplied(color: vec4<f32>) -> vec4<f32> {
    let a = color.a * u.overlay_opacity;
    return vec4<f32>(color.rgb * a, a);
}

// ─── Phase color ──────────────────────────────────────────────────────────────

/// Return the flat phase color — inhale_color during inhale/holdAfterInhale,
/// exhale_color during exhale/holdAfterExhale.
///
/// Matches Swift ContentView's `colorTransitionFill` which uses `lastColor` =
/// a flat phase color.  The color switches INSTANTLY at phase boundaries;
/// only the shape size/progress animates.
fn phase_color() -> vec4<f32> {
    if u.phase == 0u || u.phase == 1u {
        return u.inhale_color;
    }
    return u.exhale_color;
}

/// Color used for the screen-edge ripple.
///
/// Swift's `isExhale` is only true during HoldAfterExhale (phase 3), so the
/// ripple color is exhaleColor only there; in every other phase — including
/// the exhale cross-phase fade-out where the ripple is freezing the end of
/// HoldAfterInhale — it's inhaleColor.
fn ripple_color() -> vec4<f32> {
    if u.phase == 3u {
        return u.exhale_color;
    }
    return u.inhale_color;
}

// ─── Gradient helpers ─────────────────────────────────────────────────────────

/// Gradient mode: Off=0, Inner=1, On=2

fn gradient_circle(base: vec4<f32>, pixel: vec2<f32>, bg: vec4<f32>) -> vec4<f32> {
    let center   = u.viewport_size * 0.5;
    let min_dim  = min(u.viewport_size.x, u.viewport_size.y);
    let prog_sq  = u.progress * u.progress;
    let radius   = max((min_dim * prog_sq * u.max_circle_scale) * 0.5, 0.001);
    let dist    = length(pixel - center);

    if u.gradient_mode == 1u { // Inner
        return lerp_color(bg, base, clamp01(dist / radius));
    }

    // On: radial from center, peaks at midpoint
    let ext_r = radius * max(u.circle_gradient_scale, 1.0);
    let t     = clamp01(dist / ext_r);
    if t <= 0.5 {
        return lerp_color(bg, base, t * 2.0);
    }
    return lerp_color(base, bg, (t - 0.5) * 2.0);
}

fn gradient_rectangle(base: vec4<f32>, pixel: vec2<f32>, rect_h: f32, bg: vec4<f32>) -> vec4<f32> {
    let y01 = clamp01(pixel.y / max(rect_h, 1.0));

    if u.gradient_mode == 1u { // Inner: bottom=bg → top=base
        return lerp_color(bg, base, y01);
    }

    // On: bottom=bg, middle=base, top=bg
    if y01 <= 0.5 {
        return lerp_color(bg, base, y01 * 2.0);
    }
    return lerp_color(base, bg, (y01 - 0.5) * 2.0);
}

// ─── Screen-edge ripple ───────────────────────────────────────────────────────

/// Gaussian band sweeping the screen perimeter during hold phases.
/// Returns a scalar [0, 1] brightness contribution.
///
/// Perimeter parameterisation (same as Metal):
///   0 = bottom-center, 0.5 = top-center (mirrored left/right)
///   HoldAfterInhale: front sweeps 0 → 1
///   HoldAfterExhale: front sweeps 1 → 0
fn screen_edge_ripple(pixel: vec2<f32>) -> f32 {
    let W = u.viewport_size.x;
    let H = u.viewport_size.y;

    let dB = pixel.y;
    let dT = H - pixel.y;
    let dL = pixel.x;
    let dR = W - pixel.x;

    let min_dist    = min(min(dB, dT), min(dL, dR));
    // Swift: borderUnit = min(w, h) * 0.04
    let border_unit  = min(W, H) * 0.04;

    // Stroke + blur half-extents. Stark: 2× borderUnit stroke, no blur.
    // Gradient: 3× borderUnit stroke + 2× borderUnit blur.
    let use_gradient = u.ripple_enabled == 2u;
    let stroke_half  = select(border_unit, border_unit * 1.5, use_gradient);
    let blur_radius  = select(0.0, border_unit * 2.0, use_gradient);

    if min_dist > stroke_half + blur_radius * 3.0 { return 0.0; }

    let nx = (pixel.x - W * 0.5) / max(W * 0.5, 1.0);
    let ny = (pixel.y - H * 0.5) / max(H * 0.5, 1.0);

    let right_half  = pixel.x >= W * 0.5;
    let half_perim  = W + H;
    var perim_param: f32;

    if abs(ny) >= abs(nx) {
        if ny < 0.0 {
            // Bottom edge
            perim_param = select(W * 0.5 - pixel.x, pixel.x - W * 0.5, right_half);
        } else {
            // Top edge
            perim_param = W * 0.5 + H + select(pixel.x, W - pixel.x, right_half);
        }
    } else {
        // Side edge
        perim_param = W * 0.5 + pixel.y;
    }

    let perim = clamp(perim_param / half_perim, 0.0, 1.0);

    // Swift's holdProgress per phase:
    //   HoldAfterInhale (phase 1): 0 → 1
    //   HoldAfterExhale (phase 3): 1 → 0
    //   Exhale (phase 2), cross-phase fade: frozen at 1 (end of inhale-hold)
    //   Inhale (phase 0), cross-phase fade: frozen at 0 (end of exhale-hold —
    //     but trail/band collapse to [0,0] so nothing draws; Swift quirk preserved)
    var hp: f32;
    if u.phase == 1u {
        hp = u.hold_time;
    } else if u.phase == 3u {
        hp = 1.0 - u.hold_time;
    } else if u.phase == 2u {
        hp = 1.0;
    } else {
        hp = 0.0;
    }

    // Swift's isExhale is true only during HoldAfterExhale (phase 3).
    // During inhale/exhale, isExhale is false because holdProgress is 0 in that
    // branch. Trail/band formulas below follow Swift exactly.
    let is_exhale_hold = u.phase == 3u;

    var trail_from: f32;
    var trail_to:   f32;
    var band_from:  f32;
    var band_to:    f32;
    if is_exhale_hold {
        trail_from = hp;
        trail_to   = 1.0;
        band_from  = hp;
        band_to    = min(1.0, hp + 0.12);
    } else {
        trail_from = 0.0;
        trail_to   = hp;
        band_from  = max(0.0, hp - 0.12);
        band_to    = hp;
    }

    // Along-path edge softening: blur radius in pixels → perim-param units.
    let along_b  = max(blur_radius / max(half_perim, 1.0), 1e-4);
    let in_trail = smoothstep(trail_from - along_b, trail_from + along_b, perim)
                 * (1.0 - smoothstep(trail_to - along_b, trail_to + along_b, perim));
    let in_band  = smoothstep(band_from - along_b, band_from + along_b, perim)
                 * (1.0 - smoothstep(band_to - along_b, band_to + along_b, perim));

    // Perpendicular stroke — crisp for stark, blur-softened edge for gradient.
    let perp_b = max(blur_radius, 1e-4);
    let perp   = 1.0 - smoothstep(stroke_half - perp_b, stroke_half + perp_b, min_dist);

    // Swift trail opacity = 0.25, band opacity = 0.8. Take the max so the band
    // rides on top of the trail rather than double-counting.
    let intensity = max(in_trail * 0.25, in_band * 0.80) * perp;

    // rippleOpacity: Swift holds it at 1 during the hold phase, then fades it
    // 1→0 over the first 10% of the following inhale/exhale. hold_time carries
    // the LINEAR phase progress (not eased) so 10% of hold_time == 10% of the
    // wall-clock phase, matching Swift's `.linear(duration: duration * 0.1)`.
    var ripple_opacity: f32 = 1.0;
    if u.phase == 0u || u.phase == 2u {
        ripple_opacity = 1.0 - smoothstep(0.0, 0.10, u.hold_time);
    }
    return intensity * ripple_opacity;
}

// ─── Fragment shader ──────────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Stopped (display_mode == 2): overlay is completely transparent — matches
    // Swift ContentView `Color.clear` when `!isAnimating && !isPaused`.
    if u.display_mode == 2u || u.overlay_opacity <= 0.0001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Paused (display_mode == 1): flat background-colour tint, no animation shape —
    // matches Swift ContentView showing `backgroundColorWithoutAlpha.opacity(overlayOpacity)`.
    if u.display_mode == 1u {
        let a = u.overlay_opacity;
        return vec4<f32>(u.background_color.rgb * a, a);   // premultiplied
    }

    let pixel = pixel_from_ndc(in.ndc);

    // background_opacity = min(bg.a, overlay_opacity) — the final background alpha,
    // matching Swift's .opacity(min(bgAlpha, overlayOpacity)) on the background layer.
    // Background pixels must NOT have overlay_opacity applied again; only shape pixels do.
    var bg = vec4<f32>(u.background_color.rgb, u.background_opacity);

    let pc        = phase_color();
    let rpc       = ripple_color();
    // ripple_enabled is toggled on by from_state for hold phases AND during the
    // first 10% of the following inhale/exhale (cross-phase fade). The shader
    // itself applies the fade envelope; this gate just avoids paying the cost
    // outside those windows.
    let do_ripple = u.ripple_enabled != 0u;

    // ── Fullscreen (shape == 0) ───────────────────────────────────────────────
    // Flat inhale/exhale color with no progress-based transition, matching
    // Swift ContentView which switches immediately at phase boundaries. The
    // ripple sits on top in the ripple_color — invisible when it matches the
    // fill color (phases 0/1/3) and a visible cross-phase fade during phase 2
    // where fill=exhale but ripple=inhale.
    if u.shape == 0u {
        var out = pc;
        if do_ripple {
            let r = screen_edge_ripple(pixel);
            out = lerp_color(out, rpc, r);
        }
        return apply_premultiplied(out);
    }

    // ── Rectangle (shape == 1) ────────────────────────────────────────────────
    if u.shape == 1u {
        let height      = max(u.viewport_size.y, 1.0);
        let scale_limit = max(u.rectangle_scale, 1.0);
        let scaled_prog = clamp(u.progress * scale_limit, 0.0, scale_limit);
        let rect_h      = height * scaled_prog;

        if pixel.y <= rect_h {
            // Inside shape: apply overlay_opacity via apply_premultiplied.
            var sc = pc;
            if u.gradient_mode != 0u {
                sc = gradient_rectangle(pc, pixel, rect_h, bg);
            }
            if do_ripple {
                // Swift band alpha = 0.8, trail = 0.25; screen_edge_ripple returns
                // intensity in [0, 0.8] so lerp factor is the intensity directly.
                // Target color is ripple_color (inhale except during HoldAfterExhale),
                // not the shape fill color — see Swift's `phaseColor = isExhale ? ...`.
                let r = screen_edge_ripple(pixel);
                sc = lerp_color(sc, rpc, r);
            }
            return apply_premultiplied(sc);
        }
        if do_ripple {
            let r = screen_edge_ripple(pixel);
            return apply_premultiplied(lerp_color(bg, rpc, r));
        }
        // Background-only pixel: alpha is already background_opacity = min(bg.a, overlay_opacity).
        // Do NOT multiply by overlay_opacity again — matches Swift's separate background layer.
        return vec4<f32>(bg.rgb * bg.a, bg.a);
    }

    // ── Circle (shape == 2) ───────────────────────────────────────────────────
    var sc = pc;
    if u.gradient_mode != 0u {
        sc = gradient_circle(pc, pixel, bg);
    }

    let center    = u.viewport_size * 0.5;
    let min_dim   = min(u.viewport_size.x, u.viewport_size.y);
    let prog_sq   = u.progress * u.progress;
    let inner_r   = max((min_dim * prog_sq * u.max_circle_scale) * 0.5, 0.0);
    // "On" gradient doubles the visible circle radius (matches Swift gradientScale=2 * bakedSize).
    let visible_r = inner_r * max(u.circle_gradient_scale, 1.0);

    if length(pixel - center) <= visible_r {
        // Inside shape: apply overlay_opacity via apply_premultiplied.
        if do_ripple {
            let r = screen_edge_ripple(pixel);
            sc = lerp_color(sc, pc, r);
        }
        return apply_premultiplied(sc);
    }
    if do_ripple {
        let r = screen_edge_ripple(pixel);
        return apply_premultiplied(lerp_color(bg, pc, r));
    }
    // Background-only pixel — same fix as rectangle above.
    return vec4<f32>(bg.rgb * bg.a, bg.a);
}
