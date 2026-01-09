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
    float2 uv;
};

vertex VertexOut overlayVertex(uint vertexID [[vertex_id]],
                               constant OverlayUniforms& uniforms [[buffer(0)]]) {
    // Fullscreen triangle
    float2 positions[3] = {
        float2(-1.0, -1.0),
        float2( 3.0, -1.0),
        float2(-1.0,  3.0)
    };

    float2 pos = positions[vertexID];
    VertexOut out;
    out.position = float4(pos, 0.0, 1.0);

    // Map to [0,1] UV
    out.uv = pos * 0.5 + 0.5;
    return out;
}

static inline float4 premultiply(float4 c) {
    return float4(c.rgb * c.a, c.a);
}

static inline float4 lerp4(float4 a, float4 b, float t) {
    return a + (b - a) * t;
}

static inline float4 gradient3(float4 c0, float4 c1, float4 c2, float t) {
    if (t <= 0.5) {
        return lerp4(c0, c1, t * 2.0);
    }
    return lerp4(c1, c2, (t - 0.5) * 2.0);
}

fragment float4 overlayFragment(VertexOut in [[stage_in]],
                                constant OverlayUniforms& u [[buffer(0)]]) {
    // Pixel space for shape math
    float2 pixel = in.uv * u.viewportSize;
    float2 center = u.viewportSize * 0.5;
    float2 delta = pixel - center;

    float minDimension = min(u.viewportSize.x, u.viewportSize.y);

    // Background tint: without alpha, but with backgroundOpacity
    float4 background = float4(u.backgroundColor.rgb, u.backgroundOpacity);
    float4 result = premultiply(background);

    // If not animating, you can treat progress as 0 and still tint background
    float progress = clamp(u.progress, 0.0, 1.0);

    // Determine inhale/exhale color selection for phase
    bool inhalePhase = (u.phase == 0u) || (u.phase == 1u);
    float4 phaseColor = inhalePhase ? u.inhaleColor : u.exhaleColor;

    // Fullscreen: constant fill (matches your current fullscreen behavior)
    if (u.shape == 0u) {
        float4 full = float4(phaseColor.rgb, u.overlayOpacity);
        return premultiply(full);
    }

    float shapeAlpha = u.overlayOpacity;
    float4 shapeColor = float4(phaseColor.rgb, shapeAlpha);

    // Rectangle: fills from bottom upward with scale factor
    if (u.shape == 1u) {
        float heightFactor = progress * u.rectangleScale;
        if (heightFactor <= 0.0) {
            return result;
        }

        float yNorm = in.uv.y;
        bool inside = (yNorm <= heightFactor);

        if (!inside) {
            return result;
        }

        float t = clamp(yNorm / max(heightFactor, 1e-6), 0.0, 1.0); // 0 bottom -> 1 top

        if (u.gradientMode == 0u) {
            // off: constant
            shapeColor = float4(phaseColor.rgb, shapeAlpha);
        } else if (u.gradientMode == 1u) {
            // inner: LinearGradient colors [lastColor, background], start top end bottom
            // so t=1 (top) => lastColor, t=0 (bottom) => background
            float4 cTop = float4(phaseColor.rgb, shapeAlpha);
            float4 cBottom = float4(u.backgroundColor.rgb, shapeAlpha);
            shapeColor = lerp4(cBottom, cTop, t);
        } else {
            // on: LinearGradient [background, lastColor, background], start bottom end top
            float4 c0 = float4(u.backgroundColor.rgb, shapeAlpha);
            float4 c1 = float4(phaseColor.rgb, shapeAlpha);
            float4 c2 = float4(u.backgroundColor.rgb, shapeAlpha);
            shapeColor = gradient3(c0, c1, c2, t);
        }

        float4 pm = premultiply(shapeColor);
        // Over compositing with premultiplied alpha
        return pm + result * (1.0 - pm.a);
    }

    // Circle: match your baked size behavior:
    // bakedSize = minDim * progress * maxCircleScale * progress * gradientScale
    float gradientScale = (u.gradientMode == 2u) ? u.circleGradientScale : 1.0;
    float bakedSize = minDimension * progress * u.maxCircleScale * progress * gradientScale;
    float radius = bakedSize * 0.5;

    if (radius <= 0.0) {
        return result;
    }

    float dist = length(delta);
    bool insideCircle = dist <= radius;

    if (!insideCircle) {
        return result;
    }

    float tRadial = clamp(dist / max(radius, 1e-6), 0.0, 1.0); // 0 center -> 1 edge

    if (u.gradientMode == 0u) {
        shapeColor = float4(phaseColor.rgb, shapeAlpha);
    } else if (u.gradientMode == 1u) {
        // inner: RadialGradient [background, lastColor]
        float4 c0 = float4(u.backgroundColor.rgb, shapeAlpha);
        float4 c1 = float4(phaseColor.rgb, shapeAlpha);
        shapeColor = lerp4(c0, c1, tRadial);
    } else {
        // on: RadialGradient [background, lastColor, background]
        float4 c0 = float4(u.backgroundColor.rgb, shapeAlpha);
        float4 c1 = float4(phaseColor.rgb, shapeAlpha);
        float4 c2 = float4(u.backgroundColor.rgb, shapeAlpha);
        shapeColor = gradient3(c0, c1, c2, tRadial);
    }

    float4 pm = premultiply(shapeColor);
    return pm + result * (1.0 - pm.a);
}
