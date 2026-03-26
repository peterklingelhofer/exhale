#include <metal_stdlib>
using namespace metal;

struct OverlayUniforms {
    float2 viewportSize;

    float overlayOpacity;
    float backgroundOpacity;

    float maxCircleScale;

    uint shape;
    uint gradientMode;
    uint phase;

    float progress;
    float holdTime;
    float rectangleScale;
    float circleGradientScale;
    uint rippleEnabled;

    float4 backgroundColor;
    float4 inhaleColor;
    float4 exhaleColor;
};

struct VertexOut {
    float4 position [[position]];
    float2 ndc;
};

vertex VertexOut overlayVertex(uint vertexID [[vertex_id]]) {
    float2 positions[3] = {
        float2(-1.0, -1.0),
        float2( 3.0, -1.0),
        float2(-1.0,  3.0)
    };

    VertexOut out;
    out.position = float4(positions[vertexID], 0.0, 1.0);
    out.ndc = positions[vertexID];
    return out;
}

static inline float4 lerpColor(float4 a, float4 b, float t) {
    return a + (b - a) * t;
}

static inline float clamp01(float x) {
    return clamp(x, 0.0f, 1.0f);
}

static inline float2 getPixelCoordinateFromNdc(float2 ndc, float2 viewportSize) {
    float2 uv = ndc * 0.5f + 0.5f;
    return uv * viewportSize;
}

static inline float4 applyOverlayOpacityPremultiplied(float4 color, float overlayOpacity) {
    color.a *= overlayOpacity;
    color.rgb *= color.a;
    return color;
}

static inline float4 getAnimatedPhaseColor(constant OverlayUniforms &u) {
    if (u.phase == 0u) { // inhale
        float t = clamp01(u.progress);
        return lerpColor(u.exhaleColor, u.inhaleColor, t);
    }

    if (u.phase == 2u) { // exhale
        float t = clamp01(1.0f - u.progress);
        return lerpColor(u.inhaleColor, u.exhaleColor, t);
    }

    if (u.phase == 1u) { // hold after inhale
        return u.inhaleColor;
    }

    return u.exhaleColor; // hold after exhale
}

// Gradient mode mapping:
// 0 = Off, 1 = Inner, 2 = On
static inline float4 applyGradientCircle(constant OverlayUniforms &u, float4 baseColor, float2 pixel, float4 bgColor) {
    float2 center = u.viewportSize * 0.5f;
    float2 delta = pixel - center;

    float minDim = min(u.viewportSize.x, u.viewportSize.y);
    float scaledMinRadius = (minDim * u.progress * u.maxCircleScale) * 0.5f;
    float radius = max(scaledMinRadius, 0.001f);

    float dist = length(delta);

    if (u.gradientMode == 1u) {
        float t = clamp01(dist / radius);
        return lerpColor(bgColor, baseColor, t);
    }

    float extendedRadius = radius * max(u.circleGradientScale, 1.0f);
    float t = clamp01(dist / extendedRadius);

    if (t <= 0.5f) {
        return lerpColor(bgColor, baseColor, t * 2.0f);
    }

    return lerpColor(baseColor, bgColor, (t - 0.5f) * 2.0f);
}

static inline float4 applyGradientRectangle(
    constant OverlayUniforms &u,
    float4 baseColor,
    float2 pixel,
    float rectHeight,
    float4 bgColor
) {
    float safeRectHeight = max(rectHeight, 1.0f);
    float yInRect01 = clamp01(pixel.y / safeRectHeight); // 0 bottom -> 1 top edge of rectangle

    if (u.gradientMode == 1u) {
        // Inner: top = base, bottom = background
        return lerpColor(bgColor, baseColor, yInRect01);
    }

    // On: bottom = bg, middle = base, top = bg (tied to the moving rectangle height)
    if (yInRect01 <= 0.5f) {
        return lerpColor(bgColor, baseColor, yInRect01 * 2.0f);
    }

    return lerpColor(baseColor, bgColor, (yInRect01 - 0.5f) * 2.0f);
}

// Screen-edge ripple during hold phases.
// A glowing band sweeps around the screen perimeter.
// Inhale hold: bottom-center → both sides → top-center (holdTime 0→1)
// Exhale hold: top-center → both sides → bottom-center (reversed)
//
// Perimeter parameterization (0 = bottom center, 1 = top center):
//   Segment 1: bottom edge half  (bottom-center → corner)  length W/2
//   Segment 2: side edge         (bottom-corner → top-corner) length H
//   Segment 3: top edge half     (top-corner → top-center)   length W/2
//   Half-perimeter = W + H
// Both left and right halves are mirrored (same param).
static inline float screenEdgeRipple(float2 pixel, float2 viewportSize, float holdTime, uint phase) {
    float W = viewportSize.x;
    float H = viewportSize.y;

    // Distance from each screen edge
    float dB = pixel.y;           // distance from bottom
    float dT = H - pixel.y;      // distance from top
    float dL = pixel.x;           // distance from left
    float dR = W - pixel.x;      // distance from right

    float minDist = min(min(dB, dT), min(dL, dR));

    // Border glow depth — how far inward from the edge the glow reaches
    float borderDepth = min(W, H) * 0.035f;

    // Early exit for pixels far from any edge
    if (minDist > borderDepth * 3.0f) return 0.0f;

    // Determine which half of the screen (left or right) for symmetry
    bool rightHalf = pixel.x >= W * 0.5f;

    // Use aspect-ratio-adjusted diagonals for robust sector detection.
    // This maps each pixel to its nearest perimeter segment without
    // fragile float equality checks at corners.
    float nx = (pixel.x - W * 0.5f) / max(W * 0.5f, 1.0f);  // -1 to 1
    float ny = (pixel.y - H * 0.5f) / max(H * 0.5f, 1.0f);  // -1 to 1

    float halfPerim = W + H;
    float perimParam;

    if (abs(ny) >= abs(nx)) {
        // Top or bottom sector
        if (ny < 0.0f) {
            // Bottom edge sector
            perimParam = rightHalf ? (pixel.x - W * 0.5f) : (W * 0.5f - pixel.x);
        } else {
            // Top edge sector
            perimParam = W * 0.5f + H + (rightHalf ? (W - pixel.x) : pixel.x);
        }
    } else {
        // Left or right side sector
        perimParam = W * 0.5f + pixel.y;
    }

    perimParam = clamp(perimParam / halfPerim, 0.0f, 1.0f);

    // Ripple front: inhale hold sweeps 0→1, exhale hold sweeps 1→0
    float front = (phase == 3u) ? (1.0f - holdTime) : holdTime;

    // Gaussian-shaped glowing band centered on the front
    float bandWidth = 0.10f;
    float d = abs(perimParam - front);
    float bandGlow = exp(-(d * d) / (2.0f * bandWidth * bandWidth));

    // Soft trail behind the front (area already swept glows dimly)
    float trailGlow = 0.0f;
    if (phase == 3u) {
        // Exhale hold: trail is above the front (perimParam > front)
        trailGlow = smoothstep(front, front + 0.4f, perimParam) * 0.25f;
    } else {
        // Inhale hold: trail is below the front (perimParam < front)
        trailGlow = smoothstep(front, front - 0.4f, perimParam) * 0.25f;
    }

    float perimGlow = max(bandGlow, trailGlow);

    // Radial falloff from screen edge (exponential decay inward)
    float edgeGlow = exp(-minDist / max(borderDepth, 0.1f));

    // Envelope: smooth fade-in at hold start, fade-out near hold end
    float envelope = smoothstep(0.0f, 0.08f, holdTime) * smoothstep(1.0f, 0.92f, holdTime);

    return perimGlow * edgeGlow * envelope;
}

fragment float4 overlayFragment(
    VertexOut in [[stage_in]],
    constant OverlayUniforms &u [[buffer(0)]]
) {
    if (u.overlayOpacity <= 0.0001f) {
        return float4(0.0f);
    }

    float2 pixel = getPixelCoordinateFromNdc(in.ndc, u.viewportSize);

    float4 background = u.backgroundColor;
    background.a *= u.backgroundOpacity;

    float4 phaseColor = getAnimatedPhaseColor(u);

    bool isHold = (u.phase == 1u || u.phase == 3u);
    bool doRipple = isHold && u.rippleEnabled != 0u;

    // 0 = fullscreen, 1 = rectangle, 2 = circle
    if (u.shape == 0u) {
        float4 outColor = phaseColor;
        if (doRipple) {
            float r = screenEdgeRipple(pixel, u.viewportSize, u.holdTime, u.phase);
            // Brighten the phase color at the ripple band
            outColor = lerpColor(outColor, float4(1.0f, 1.0f, 1.0f, 1.0f), r * 0.5f);
        }
        return applyOverlayOpacityPremultiplied(outColor, u.overlayOpacity);
    }

    if (u.shape == 1u) {
        float height = max(u.viewportSize.y, 1.0f);

        // FIX: do not clamp to 1.0 when rectangleScale is > 1 (Gradient On uses 2x)
        float scaleLimit = max(u.rectangleScale, 1.0f);
        float scaledProgress = clamp(u.progress * scaleLimit, 0.0f, scaleLimit);

        float rectHeight = height * scaledProgress;

        bool inside = pixel.y <= rectHeight;

        float4 outColor = background;

        if (inside) {
            float4 shapeColor = phaseColor;

            if (u.gradientMode != 0u) {
                shapeColor = applyGradientRectangle(u, phaseColor, pixel, rectHeight, background);
            }

            outColor = shapeColor;
        }

        if (doRipple) {
            float r = screenEdgeRipple(pixel, u.viewportSize, u.holdTime, u.phase);
            outColor = lerpColor(outColor, phaseColor, r * 0.7f);
        }

        return applyOverlayOpacityPremultiplied(outColor, u.overlayOpacity);
    }

    // Circle
    float4 shapeColor = phaseColor;

    if (u.gradientMode != 0u) {
        shapeColor = applyGradientCircle(u, phaseColor, pixel, background);
    }

    float2 center = u.viewportSize * 0.5f;
    float2 delta = pixel - center;

    float minDim = min(u.viewportSize.x, u.viewportSize.y);
    float radius = (minDim * u.progress * u.maxCircleScale) * 0.5f;
    radius = max(radius, 0.0f);

    float dist = length(delta);
    bool inside = dist <= radius;

    float4 outColor = inside ? shapeColor : background;

    if (doRipple) {
        float r = screenEdgeRipple(pixel, u.viewportSize, u.holdTime, u.phase);
        outColor = lerpColor(outColor, phaseColor, r * 0.7f);
    }

    return applyOverlayOpacityPremultiplied(outColor, u.overlayOpacity);
}
