// OverlayUniforms.swift
import simd

struct OverlayUniforms {
    var viewportSize: SIMD2<Float> = .zero

    var overlayOpacity: Float = 0
    var backgroundOpacity: Float = 0

    var maxCircleScale: Float = 1

    var shape: UInt32 = 0
    var gradientMode: UInt32 = 0
    var phase: UInt32 = 0

    var progress: Float = 0
    var rectangleScale: Float = 1
    var circleGradientScale: Float = 1

    var backgroundColor: SIMD4<Float> = SIMD4<Float>(0, 0, 0, 0)
    var inhaleColor: SIMD4<Float> = SIMD4<Float>(1, 0, 0, 1)
    var exhaleColor: SIMD4<Float> = SIMD4<Float>(0, 0, 1, 1)
}
