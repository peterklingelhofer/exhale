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
    float rectangleScale;
    float circleGradientScale;

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
static inline float4 applyGradientCircle(constant OverlayUniforms &u, float4 baseColor, float2 pixel) {
    float2 center = u.viewportSize * 0.5f;
    float2 delta = pixel - center;

    float minDim = min(u.viewportSize.x, u.viewportSize.y);
    float scaledMinRadius = (minDim * u.progress * u.maxCircleScale) * 0.5f;
    float radius = max(scaledMinRadius, 0.001f);

    float dist = length(delta);

    if (u.gradientMode == 1u) {
        float t = clamp01(dist / radius);
        return lerpColor(u.backgroundColor, baseColor, t);
    }

    float extendedRadius = radius * max(u.circleGradientScale, 1.0f);
    float t = clamp01(dist / extendedRadius);

    if (t <= 0.5f) {
        return lerpColor(u.backgroundColor, baseColor, t * 2.0f);
    }

    return lerpColor(baseColor, u.backgroundColor, (t - 0.5f) * 2.0f);
}

static inline float4 applyGradientRectangle(
    constant OverlayUniforms &u,
    float4 baseColor,
    float2 pixel,
    float rectHeight
) {
    float safeRectHeight = max(rectHeight, 1.0f);
    float yInRect01 = clamp01(pixel.y / safeRectHeight); // 0 bottom -> 1 top edge of rectangle

    if (u.gradientMode == 1u) {
        // Inner: top = base, bottom = background
        return lerpColor(u.backgroundColor, baseColor, yInRect01);
    }

    // On: bottom = bg, middle = base, top = bg (tied to the moving rectangle height)
    if (yInRect01 <= 0.5f) {
        return lerpColor(u.backgroundColor, baseColor, yInRect01 * 2.0f);
    }

    return lerpColor(baseColor, u.backgroundColor, (yInRect01 - 0.5f) * 2.0f);
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

    // 0 = fullscreen, 1 = rectangle, 2 = circle
    if (u.shape == 0u) {
        return applyOverlayOpacityPremultiplied(phaseColor, u.overlayOpacity);
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
                shapeColor = applyGradientRectangle(u, phaseColor, pixel, rectHeight);
            }

            outColor = shapeColor;
        }

        return applyOverlayOpacityPremultiplied(outColor, u.overlayOpacity);
    }

    // Circle
    float4 shapeColor = phaseColor;

    if (u.gradientMode != 0u) {
        shapeColor = applyGradientCircle(u, phaseColor, pixel);
    }

    float2 center = u.viewportSize * 0.5f;
    float2 delta = pixel - center;

    float minDim = min(u.viewportSize.x, u.viewportSize.y);
    float radius = (minDim * u.progress * u.maxCircleScale) * 0.5f;
    radius = max(radius, 0.0f);

    bool inside = length(delta) <= radius;

    float4 outColor = inside ? shapeColor : background;
    return applyOverlayOpacityPremultiplied(outColor, u.overlayOpacity);
}
