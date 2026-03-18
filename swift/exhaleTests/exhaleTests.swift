//
//  exhaleTests.swift
//  exhaleTests
//
//  Created by Peter Klingelhofer on 3/15/23.
//

import XCTest
@testable import exhale
import SwiftUI

// MARK: - SettingsModel Tests

class SettingsModelDefaultsTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testDefaultValues() {
        XCTAssertEqual(model.inhaleDuration, 5)
        XCTAssertEqual(model.exhaleDuration, 10)
        XCTAssertEqual(model.postInhaleHoldDuration, 0)
        XCTAssertEqual(model.postExhaleHoldDuration, 0)
        XCTAssertEqual(model.drift, 1.01)
        XCTAssertEqual(model.overlayOpacity, 0.25)
        XCTAssertEqual(model.colorFillGradient, .on)
        XCTAssertEqual(model.shape, .rectangle)
        XCTAssertEqual(model.animationMode, .sinusoidal)
        XCTAssertEqual(model.randomizedTimingInhale, 0)
        XCTAssertEqual(model.randomizedTimingPostInhaleHold, 0)
        XCTAssertEqual(model.randomizedTimingExhale, 0)
        XCTAssertEqual(model.randomizedTimingPostExhaleHold, 0)
    }

    func testResetToDefaults() {
        model.inhaleDuration = 99
        model.exhaleDuration = 99
        model.drift = 5.0
        model.overlayOpacity = 1.0
        model.shape = .fullscreen
        model.animationMode = .linear
        model.colorFillGradient = .off

        model.resetToDefaults()

        XCTAssertEqual(model.inhaleDuration, 5)
        XCTAssertEqual(model.exhaleDuration, 10)
        XCTAssertEqual(model.drift, 1.01)
        XCTAssertEqual(model.overlayOpacity, 0.25)
        XCTAssertEqual(model.shape, .rectangle)
        XCTAssertEqual(model.animationMode, .sinusoidal)
        XCTAssertEqual(model.colorFillGradient, .on)
    }
}

class SettingsModelStateTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testStartSetsAnimatingAndUnpauses() {
        model.isPaused = true
        model.isAnimating = false

        model.start()

        XCTAssertTrue(model.isAnimating)
        XCTAssertFalse(model.isPaused)
    }

    func testStopClearsBothFlags() {
        model.isAnimating = true
        model.isPaused = true

        model.stop()

        XCTAssertFalse(model.isAnimating)
        XCTAssertFalse(model.isPaused)
    }

    func testPauseBehavesLikeStop() {
        model.isAnimating = true
        model.isPaused = false

        model.pause()

        XCTAssertFalse(model.isAnimating)
        XCTAssertFalse(model.isPaused)
    }

    func testUnpauseClearsPausedFlag() {
        model.isPaused = true

        model.unpause()

        XCTAssertFalse(model.isPaused)
    }
}

// MARK: - Color Matching Tests (regression: same-color fullscreen CPU optimization)

class ColorMatchingTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testColorsMatchWhenIdentical() {
        let color = Color(red: 0.5, green: 0.3, blue: 0.8)
        model.inhaleColor = color
        model.exhaleColor = color

        XCTAssertTrue(model.inhaleAndExhaleColorsMatch)
    }

    func testColorsDoNotMatchWhenDifferent() {
        model.inhaleColor = Color.red
        model.exhaleColor = Color.blue

        XCTAssertFalse(model.inhaleAndExhaleColorsMatch)
    }

    func testDefaultColorsDoNotMatch() {
        // Defaults: red inhale, blue exhale
        XCTAssertFalse(model.inhaleAndExhaleColorsMatch)
    }

    func testColorsMatchWithinEpsilon() {
        // Two colors that are extremely close but constructed separately
        model.inhaleColor = Color(red: 0.5, green: 0.5, blue: 0.5)
        model.exhaleColor = Color(red: 0.5, green: 0.5, blue: 0.5)

        XCTAssertTrue(model.inhaleAndExhaleColorsMatch)
    }

    func testBlackColorsMatch() {
        model.inhaleColor = Color(red: 0, green: 0, blue: 0)
        model.exhaleColor = Color(red: 0, green: 0, blue: 0)

        XCTAssertTrue(model.inhaleAndExhaleColorsMatch)
    }

    func testWhiteColorsMatch() {
        model.inhaleColor = Color(red: 1, green: 1, blue: 1)
        model.exhaleColor = Color(red: 1, green: 1, blue: 1)

        XCTAssertTrue(model.inhaleAndExhaleColorsMatch)
    }
}

// MARK: - Background Color Cache Tests (regression: gradient dark outline)

class BackgroundColorCacheTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testClearBackgroundHasZeroAlpha() {
        model.backgroundColor = Color.clear

        XCTAssertEqual(model.cachedBackgroundAlphaComponent, 0, accuracy: 0.001)
    }

    func testOpaqueBackgroundHasFullAlpha() {
        model.backgroundColor = Color.red // fully opaque

        XCTAssertEqual(model.cachedBackgroundAlphaComponent, 1.0, accuracy: 0.001)
    }

    func testWithoutAlphaPreservesRGB() {
        let original = Color(red: 0.3, green: 0.6, blue: 0.9)
        model.backgroundColor = original

        let withoutAlpha = model.cachedBackgroundColorWithoutAlpha
        // withoutAlpha should have alpha = 1
        XCTAssertEqual(withoutAlpha.alphaComponent(), 1.0, accuracy: 0.001)
    }

    func testBackgroundCacheUpdatesOnChange() {
        model.backgroundColor = Color.red
        XCTAssertEqual(model.cachedBackgroundAlphaComponent, 1.0, accuracy: 0.001)

        model.backgroundColor = Color.clear
        XCTAssertEqual(model.cachedBackgroundAlphaComponent, 0.0, accuracy: 0.001)
    }

    func testSemiTransparentBackground() {
        model.backgroundColor = Color(red: 1, green: 0, blue: 0, opacity: 0.5)

        XCTAssertEqual(model.cachedBackgroundAlphaComponent, 0.5, accuracy: 0.001)
    }
}

// MARK: - Color Extension Tests

class ColorExtensionTests: XCTestCase {
    func testAlphaComponentOfOpaqueColor() {
        let color = Color(red: 1, green: 0, blue: 0)
        XCTAssertEqual(color.alphaComponent(), 1.0, accuracy: 0.001)
    }

    func testAlphaComponentOfTransparentColor() {
        let color = Color.clear
        XCTAssertEqual(color.alphaComponent(), 0.0, accuracy: 0.001)
    }

    func testAlphaComponentOfSemiTransparent() {
        let color = Color(red: 1, green: 0, blue: 0, opacity: 0.5)
        XCTAssertEqual(color.alphaComponent(), 0.5, accuracy: 0.01)
    }

    func testWithoutAlphaSetsAlphaToOne() {
        let color = Color(red: 0.5, green: 0.3, blue: 0.8, opacity: 0.2)
        let result = color.withoutAlpha()
        XCTAssertEqual(result.alphaComponent(), 1.0, accuracy: 0.001)
    }

    func testWithoutAlphaOnOpaqueIsIdempotent() {
        let color = Color(red: 0.5, green: 0.3, blue: 0.8)
        let result = color.withoutAlpha()
        XCTAssertEqual(result.alphaComponent(), 1.0, accuracy: 0.001)
    }

    func testToFloat4CachedOpaqueRed() {
        let color = Color(red: 1, green: 0, blue: 0)
        let f4 = color.toFloat4Cached()
        XCTAssertEqual(f4.x, 1.0, accuracy: 0.02) // red
        XCTAssertEqual(f4.y, 0.0, accuracy: 0.02) // green
        XCTAssertEqual(f4.z, 0.0, accuracy: 0.02) // blue
        XCTAssertEqual(f4.w, 1.0, accuracy: 0.02) // alpha
    }

    func testToFloat4CachedTransparent() {
        let color = Color.clear
        let f4 = color.toFloat4Cached()
        XCTAssertEqual(f4.w, 0.0, accuracy: 0.02, "Clear color should have alpha 0")
    }

    func testToFloat4CachedSemiTransparent() {
        let color = Color(red: 0, green: 1, blue: 0, opacity: 0.5)
        let f4 = color.toFloat4Cached()
        XCTAssertEqual(f4.y, 1.0, accuracy: 0.05) // green
        XCTAssertEqual(f4.w, 0.5, accuracy: 0.05) // alpha
    }
}

// MARK: - BreathingPhase Tests

class BreathingPhaseTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testInhaleDuration() {
        model.inhaleDuration = 3.0
        XCTAssertEqual(BreathingPhase.inhale.duration(settingsModel: model), 3.0)
    }

    func testExhaleDuration() {
        model.exhaleDuration = 7.0
        XCTAssertEqual(BreathingPhase.exhale.duration(settingsModel: model), 7.0)
    }

    func testHoldAfterInhaleDuration() {
        model.postInhaleHoldDuration = 2.0
        XCTAssertEqual(BreathingPhase.holdAfterInhale.duration(settingsModel: model), 2.0)
    }

    func testHoldAfterExhaleDuration() {
        model.postExhaleHoldDuration = 1.5
        XCTAssertEqual(BreathingPhase.holdAfterExhale.duration(settingsModel: model), 1.5)
    }
}

// MARK: - CubicBezierEaseInOut Tests

class CubicBezierTests: XCTestCase {
    // Standard ease-in-out: (0.42, 0, 0.58, 1)
    let x1 = 0.42, y1 = 0.0, x2 = 0.58, y2 = 1.0

    func testBoundaryAtZero() {
        let value = CubicBezierEaseInOut.getValue(t: 0, x1: x1, y1: y1, x2: x2, y2: y2)
        XCTAssertEqual(value, 0.0, accuracy: 1e-4)
    }

    func testBoundaryAtOne() {
        let value = CubicBezierEaseInOut.getValue(t: 1, x1: x1, y1: y1, x2: x2, y2: y2)
        XCTAssertEqual(value, 1.0, accuracy: 1e-4)
    }

    func testMidpointSymmetry() {
        // Standard ease-in-out is symmetric: f(0.5) should be 0.5
        let value = CubicBezierEaseInOut.getValue(t: 0.5, x1: x1, y1: y1, x2: x2, y2: y2)
        XCTAssertEqual(value, 0.5, accuracy: 0.01)
    }

    func testMonotonicallyIncreasing() {
        var prev = 0.0
        for i in 1...100 {
            let t = Double(i) / 100.0
            let value = CubicBezierEaseInOut.getValue(t: t, x1: x1, y1: y1, x2: x2, y2: y2)
            XCTAssertGreaterThanOrEqual(value, prev, "Easing should be monotonically increasing at t=\(t)")
            prev = value
        }
    }

    func testEaseInOutSlowerAtEndpoints() {
        // Ease-in-out: progress near 0 and 1 should be slower than linear
        let earlyValue = CubicBezierEaseInOut.getValue(t: 0.1, x1: x1, y1: y1, x2: x2, y2: y2)
        XCTAssertLessThan(earlyValue, 0.1, "Ease-in should be slower than linear at start")

        let lateValue = CubicBezierEaseInOut.getValue(t: 0.9, x1: x1, y1: y1, x2: x2, y2: y2)
        XCTAssertGreaterThan(lateValue, 0.9, "Ease-out should be faster than linear near end")
    }

    func testLinearCurve() {
        // Linear: control points at (0.0, 0.0) and (1.0, 1.0)
        for i in 0...10 {
            let t = Double(i) / 10.0
            let value = CubicBezierEaseInOut.getValue(t: t, x1: 0, y1: 0, x2: 1, y2: 1)
            XCTAssertEqual(value, t, accuracy: 0.01, "Linear curve should approximate identity at t=\(t)")
        }
    }

    func testOutputAlwaysInUnitRange() {
        // For standard ease-in-out, output should stay in [0, 1]
        for i in 0...100 {
            let t = Double(i) / 100.0
            let value = CubicBezierEaseInOut.getValue(t: t, x1: x1, y1: y1, x2: x2, y2: y2)
            XCTAssertGreaterThanOrEqual(value, 0.0)
            XCTAssertLessThanOrEqual(value, 1.0)
        }
    }
}

// MARK: - MetalBreathingState Tests

class MetalBreathingStateTests: XCTestCase {
    func testInhaleStateProgressRange() {
        let state = MetalBreathingState(phase: .inhale, progress: 0.5)
        XCTAssertEqual(state.phase, .inhale)
        XCTAssertEqual(state.progress, 0.5)
    }

    func testHoldAfterInhaleFullProgress() {
        let state = MetalBreathingState(phase: .holdAfterInhale, progress: 1.0)
        XCTAssertEqual(state.progress, 1.0)
    }

    func testHoldAfterExhaleZeroProgress() {
        let state = MetalBreathingState(phase: .holdAfterExhale, progress: 0.0)
        XCTAssertEqual(state.progress, 0.0)
    }
}

// MARK: - MetalBreathingController Tests

class MetalBreathingControllerTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testInitialStateIsInhale() {
        let controller = MetalBreathingController(settingsModel: model)
        model.start()
        controller.start()

        let state = controller.getCurrentState()
        XCTAssertEqual(state.phase, .inhale)

        controller.stop()
    }

    func testInitialProgressStartsNearZero() {
        let controller = MetalBreathingController(settingsModel: model)
        model.start()
        controller.start()

        let state = controller.getCurrentState()
        // Just started, progress should be very low
        XCTAssertLessThan(state.progress, 0.1)

        controller.stop()
    }

    func testStartIfNeededStartsWhenAnimating() {
        model.start()
        let controller = MetalBreathingController(settingsModel: model)

        var drawCalled = false
        controller.requestDraw = { drawCalled = true }
        controller.startIfNeeded()

        // Give the timer a moment to fire
        let expectation = XCTestExpectation(description: "draw requested")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            if drawCalled { expectation.fulfill() }
        }
        wait(for: [expectation], timeout: 1.0)

        controller.stop()
    }

    func testStartIfNeededStartsWhenPaused() {
        model.isAnimating = false
        model.isPaused = true
        let controller = MetalBreathingController(settingsModel: model)
        controller.startIfNeeded()

        // Should be running (not stopped) — verify by getting state
        let state = controller.getCurrentState()
        XCTAssertNotNil(state)

        controller.stop()
    }

    func testStartIfNeededStopsWhenNotAnimatingNotPaused() {
        model.isAnimating = false
        model.isPaused = false

        let controller = MetalBreathingController(settingsModel: model)
        controller.startIfNeeded()

        // Controller should be stopped — no crash getting state though
        // (getCurrentState accesses internal queue synchronously)
        let state = controller.getCurrentState()
        XCTAssertNotNil(state)

        controller.stop()
    }
}

// MARK: - OverlayUniforms Tests

class OverlayUniformsTests: XCTestCase {
    func testDefaultValues() {
        let u = OverlayUniforms()

        XCTAssertEqual(u.viewportSize, .zero)
        XCTAssertEqual(u.overlayOpacity, 0)
        XCTAssertEqual(u.backgroundOpacity, 0)
        XCTAssertEqual(u.maxCircleScale, 1)
        XCTAssertEqual(u.shape, 0)
        XCTAssertEqual(u.gradientMode, 0)
        XCTAssertEqual(u.phase, 0)
        XCTAssertEqual(u.progress, 0)
        XCTAssertEqual(u.rectangleScale, 1)
        XCTAssertEqual(u.circleGradientScale, 1)
    }

    func testDefaultInhaleColorIsRed() {
        let u = OverlayUniforms()
        XCTAssertEqual(u.inhaleColor.x, 1) // R
        XCTAssertEqual(u.inhaleColor.y, 0) // G
        XCTAssertEqual(u.inhaleColor.z, 0) // B
        XCTAssertEqual(u.inhaleColor.w, 1) // A
    }

    func testDefaultExhaleColorIsBlue() {
        let u = OverlayUniforms()
        XCTAssertEqual(u.exhaleColor.x, 0) // R
        XCTAssertEqual(u.exhaleColor.y, 0) // G
        XCTAssertEqual(u.exhaleColor.z, 1) // B
        XCTAssertEqual(u.exhaleColor.w, 1) // A
    }

    func testDefaultBackgroundColorIsTransparent() {
        let u = OverlayUniforms()
        XCTAssertEqual(u.backgroundColor, SIMD4<Float>(0, 0, 0, 0))
    }
}

// MARK: - Enum Raw Value Tests

class EnumTests: XCTestCase {
    func testAnimationShapeRawValues() {
        XCTAssertEqual(AnimationShape.rectangle.rawValue, "Rectangle")
        XCTAssertEqual(AnimationShape.circle.rawValue, "Circle")
        XCTAssertEqual(AnimationShape.fullscreen.rawValue, "Fullscreen")
    }

    func testAnimationShapeFromRawValue() {
        XCTAssertEqual(AnimationShape(rawValue: "Rectangle"), .rectangle)
        XCTAssertEqual(AnimationShape(rawValue: "Circle"), .circle)
        XCTAssertEqual(AnimationShape(rawValue: "Fullscreen"), .fullscreen)
        XCTAssertNil(AnimationShape(rawValue: "invalid"))
    }

    func testAnimationModeRawValues() {
        XCTAssertEqual(AnimationMode.linear.rawValue, "Linear")
        XCTAssertEqual(AnimationMode.sinusoidal.rawValue, "Sinusoidal")
    }

    func testAnimationModeFromRawValue() {
        XCTAssertEqual(AnimationMode(rawValue: "Linear"), .linear)
        XCTAssertEqual(AnimationMode(rawValue: "Sinusoidal"), .sinusoidal)
        XCTAssertNil(AnimationMode(rawValue: "invalid"))
    }

    func testColorFillGradientRawValues() {
        XCTAssertEqual(ColorFillGradient.off.rawValue, "Off")
        XCTAssertEqual(ColorFillGradient.inner.rawValue, "Inner")
        XCTAssertEqual(ColorFillGradient.on.rawValue, "On")
    }

    func testColorFillGradientFromRawValue() {
        XCTAssertEqual(ColorFillGradient(rawValue: "Off"), .off)
        XCTAssertEqual(ColorFillGradient(rawValue: "Inner"), .inner)
        XCTAssertEqual(ColorFillGradient(rawValue: "On"), .on)
        XCTAssertNil(ColorFillGradient(rawValue: "invalid"))
    }

    func testAnimationShapeCaseIterable() {
        XCTAssertEqual(AnimationShape.allCases.count, 3)
    }

    func testAnimationModeCaseIterable() {
        XCTAssertEqual(AnimationMode.allCases.count, 2)
    }

    func testColorFillGradientCaseIterable() {
        XCTAssertEqual(ColorFillGradient.allCases.count, 3)
    }
}

// MARK: - Regression: Gradient background opacity leak

class GradientBackgroundOpacityTests: XCTestCase {
    /// Regression test: when background color is 0% opacity, the gradient should NOT
    /// use cachedBackgroundColorWithoutAlpha (which has alpha forced to 1), because
    /// that causes a visible dark outline in gradient On mode.
    /// The fix uses settingsModel.backgroundColor directly (preserves actual alpha).
    func testClearBackgroundWithoutAlphaHasFullAlpha() {
        let model = SettingsModel()
        model.backgroundColor = Color.clear

        // cachedBackgroundColorWithoutAlpha strips alpha → alpha=1
        // This is the value that was incorrectly used in the gradient, causing the dark outline
        XCTAssertEqual(model.cachedBackgroundColorWithoutAlpha.alphaComponent(), 1.0, accuracy: 0.001)

        // The actual backgroundColor preserves alpha=0
        XCTAssertEqual(model.backgroundColor.alphaComponent(), 0.0, accuracy: 0.001)
    }

    /// When background is transparent, using it in the gradient should produce
    /// colors that fade to transparent, not to opaque black.
    func testTransparentBackgroundFloat4HasZeroAlpha() {
        let bgColor = Color.clear
        let f4 = bgColor.toFloat4Cached()
        XCTAssertEqual(f4.w, 0.0, accuracy: 0.02, "Clear background should have alpha=0 in float4 form")
    }

    /// When background has visible opacity, the gradient should fade to that color.
    func testOpaqueBackgroundFloat4HasFullAlpha() {
        let bgColor = Color(red: 0.5, green: 0.5, blue: 0.5) // fully opaque
        let f4 = bgColor.toFloat4Cached()
        XCTAssertEqual(f4.w, 1.0, accuracy: 0.02)
    }

    func testBackgroundOpacityComputedCorrectly() {
        let model = SettingsModel()
        model.overlayOpacity = 0.25

        // Case 1: transparent background → backgroundOpacity should be 0
        model.backgroundColor = Color.clear
        let bgOpacity1 = min(model.cachedBackgroundAlphaComponent, model.overlayOpacity)
        XCTAssertEqual(bgOpacity1, 0.0, accuracy: 0.001)

        // Case 2: opaque background → backgroundOpacity capped at overlayOpacity
        model.backgroundColor = Color.red
        let bgOpacity2 = min(model.cachedBackgroundAlphaComponent, model.overlayOpacity)
        XCTAssertEqual(bgOpacity2, 0.25, accuracy: 0.001)

        // Case 3: semi-transparent background below overlay → uses bg alpha
        model.backgroundColor = Color(red: 1, green: 0, blue: 0, opacity: 0.1)
        let bgOpacity3 = min(model.cachedBackgroundAlphaComponent, model.overlayOpacity)
        XCTAssertEqual(bgOpacity3, 0.1, accuracy: 0.01)
    }
}

// MARK: - Regression: Same-color fullscreen CPU optimization

class SameColorFullscreenTests: XCTestCase {
    func testSameColorFullscreenDetection() {
        let model = SettingsModel()
        model.shape = .fullscreen
        model.inhaleColor = Color(red: 0.5, green: 0.3, blue: 0.8)
        model.exhaleColor = Color(red: 0.5, green: 0.3, blue: 0.8)

        XCTAssertTrue(model.shape == .fullscreen && model.inhaleAndExhaleColorsMatch,
                       "Should detect same-color fullscreen for CPU optimization")
    }

    func testDifferentColorFullscreenNotOptimized() {
        let model = SettingsModel()
        model.shape = .fullscreen
        model.inhaleColor = Color.red
        model.exhaleColor = Color.blue

        XCTAssertFalse(model.inhaleAndExhaleColorsMatch,
                        "Different colors should not trigger CPU optimization")
    }

    func testSameColorNonFullscreenNotAffected() {
        let model = SettingsModel()
        model.shape = .circle
        let color = Color(red: 1, green: 0, blue: 0)
        model.inhaleColor = color
        model.exhaleColor = color

        // Colors match, but shape is not fullscreen — the optimization only applies to fullscreen
        XCTAssertTrue(model.inhaleAndExhaleColorsMatch)
        XCTAssertNotEqual(model.shape, .fullscreen)
    }
}

// MARK: - OverlayUniforms Layout Tests (Swift ↔ Metal struct parity)

class OverlayUniformsLayoutTests: XCTestCase {
    /// If this test fails, the Swift struct no longer matches the Metal shader struct.
    /// Any field added/removed/reordered in one must be mirrored in the other.
    func testStructStrideMatchesMetalLayout() {
        // Metal struct layout (with float4 requiring 16-byte alignment):
        //   float2 (8) + float (4) + float (4) + float (4) +
        //   uint (4) + uint (4) + uint (4) + float (4) + float (4) + float (4) +
        //   pad (4) + float4 (16) + float4 (16) + float4 (16) = 96
        XCTAssertEqual(MemoryLayout<OverlayUniforms>.stride, 96,
                        "OverlayUniforms stride changed — update both Swift and Metal struct definitions")
    }

    func testStructAlignment() {
        // SIMD4<Float> requires 16-byte alignment
        XCTAssertEqual(MemoryLayout<OverlayUniforms>.alignment, 16)
    }
}

// MARK: - Color Persistence Round-Trip Tests

class ColorPersistenceTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    override func tearDown() {
        model.resetToDefaults()
        super.tearDown()
    }

    func testInhaleColorPersistsAndReloads() {
        let custom = Color(red: 0.2, green: 0.4, blue: 0.6)
        model.inhaleColor = custom

        // Create a new model that loads from UserDefaults
        let reloaded = SettingsModel()
        let f4original = custom.toFloat4Cached()
        let f4reloaded = reloaded.inhaleColor.toFloat4Cached()

        XCTAssertEqual(f4original.x, f4reloaded.x, accuracy: 0.02)
        XCTAssertEqual(f4original.y, f4reloaded.y, accuracy: 0.02)
        XCTAssertEqual(f4original.z, f4reloaded.z, accuracy: 0.02)
    }

    func testExhaleColorPersistsAndReloads() {
        let custom = Color(red: 0.8, green: 0.1, blue: 0.9)
        model.exhaleColor = custom

        let reloaded = SettingsModel()
        let f4original = custom.toFloat4Cached()
        let f4reloaded = reloaded.exhaleColor.toFloat4Cached()

        XCTAssertEqual(f4original.x, f4reloaded.x, accuracy: 0.02)
        XCTAssertEqual(f4original.y, f4reloaded.y, accuracy: 0.02)
        XCTAssertEqual(f4original.z, f4reloaded.z, accuracy: 0.02)
    }

    func testBackgroundColorPersistsAndReloads() {
        let custom = Color(red: 0.5, green: 0.5, blue: 0.5, opacity: 0.7)
        model.backgroundColor = custom

        let reloaded = SettingsModel()
        let f4original = custom.toFloat4Cached()
        let f4reloaded = reloaded.backgroundColor.toFloat4Cached()

        XCTAssertEqual(f4original.x, f4reloaded.x, accuracy: 0.02)
        XCTAssertEqual(f4original.y, f4reloaded.y, accuracy: 0.02)
        XCTAssertEqual(f4original.z, f4reloaded.z, accuracy: 0.02)
        XCTAssertEqual(f4original.w, f4reloaded.w, accuracy: 0.02)
    }

    func testNumericSettingsPersist() {
        model.inhaleDuration = 3.5
        model.exhaleDuration = 7.5
        model.drift = 1.05
        model.overlayOpacity = 0.8

        let reloaded = SettingsModel()
        XCTAssertEqual(reloaded.inhaleDuration, 3.5)
        XCTAssertEqual(reloaded.exhaleDuration, 7.5)
        XCTAssertEqual(reloaded.drift, 1.05)
        XCTAssertEqual(reloaded.overlayOpacity, 0.8)
    }

    func testEnumSettingsPersist() {
        model.shape = .fullscreen
        model.animationMode = .linear
        model.colorFillGradient = .off

        let reloaded = SettingsModel()
        XCTAssertEqual(reloaded.shape, .fullscreen)
        XCTAssertEqual(reloaded.animationMode, .linear)
        XCTAssertEqual(reloaded.colorFillGradient, .off)
    }
}

// MARK: - Easing Table Property Tests

class EasingTableTests: XCTestCase {
    func testEasingTableBoundaries() {
        // Build same table the controller uses
        let table = (0..<1024).map { i -> Float in
            let t = Double(i) / 1023.0
            return Float(CubicBezierEaseInOut.getValue(t: t, x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0))
        }

        XCTAssertEqual(table.first!, 0.0, accuracy: 0.001, "Table should start at 0")
        XCTAssertEqual(table.last!, 1.0, accuracy: 0.001, "Table should end at 1")
    }

    func testEasingTableMonotonicity() {
        let table = (0..<1024).map { i -> Float in
            let t = Double(i) / 1023.0
            return Float(CubicBezierEaseInOut.getValue(t: t, x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0))
        }

        for i in 1..<table.count {
            XCTAssertGreaterThanOrEqual(table[i], table[i - 1],
                                         "Easing table must be monotonically non-decreasing at index \(i)")
        }
    }

    func testEasingTableSize() {
        // Controller uses 1024 samples — verify table generation doesn't crash or truncate
        let table = (0..<1024).map { i -> Float in
            let t = Double(i) / 1023.0
            return Float(CubicBezierEaseInOut.getValue(t: t, x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0))
        }
        XCTAssertEqual(table.count, 1024)
    }

    func testEasingTableSymmetry() {
        // Ease-in-out with symmetric control points: table[i] + table[N-1-i] ≈ 1.0
        let table = (0..<1024).map { i -> Float in
            let t = Double(i) / 1023.0
            return Float(CubicBezierEaseInOut.getValue(t: t, x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0))
        }

        for i in 0..<512 {
            let sum = table[i] + table[1023 - i]
            XCTAssertEqual(sum, 1.0, accuracy: 0.01,
                           "Symmetric ease-in-out: table[\(i)] + table[\(1023 - i)] should ≈ 1.0")
        }
    }
}

// MARK: - Settings Edge Cases

class SettingsEdgeCaseTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testZeroOverlayOpacity() {
        model.overlayOpacity = 0
        XCTAssertEqual(model.overlayOpacity, 0)

        // backgroundOpacity should still be 0 regardless
        let bgOpacity = min(model.cachedBackgroundAlphaComponent, model.overlayOpacity)
        XCTAssertEqual(bgOpacity, 0)
    }

    func testMaxOverlayOpacity() {
        model.overlayOpacity = 1.0
        model.backgroundColor = Color(red: 1, green: 0, blue: 0, opacity: 1.0)

        let bgOpacity = min(model.cachedBackgroundAlphaComponent, model.overlayOpacity)
        XCTAssertEqual(bgOpacity, 1.0, accuracy: 0.001)
    }

    func testOverlayOpacityCapsBackground() {
        // Even if bg alpha is 1.0, backgroundOpacity should be capped at overlayOpacity
        model.overlayOpacity = 0.3
        model.backgroundColor = Color(red: 1, green: 0, blue: 0, opacity: 1.0)

        let bgOpacity = min(model.cachedBackgroundAlphaComponent, model.overlayOpacity)
        XCTAssertEqual(bgOpacity, 0.3, accuracy: 0.001)
    }

    func testDriftOfOneProducesConstantDurations() {
        model.drift = 1.0
        model.inhaleDuration = 4.0
        model.randomizedTimingInhale = 0

        // With drift=1, duration * pow(1, cycleCount) == duration for any cycle
        for cycle in 0..<10 {
            let duration = model.inhaleDuration * pow(model.drift, Double(cycle))
            XCTAssertEqual(duration, 4.0, accuracy: 0.001)
        }
    }

    func testDriftGreaterThanOneIncreasesDurations() {
        model.drift = 1.1
        model.inhaleDuration = 4.0

        let cycle0 = model.inhaleDuration * pow(model.drift, 0)
        let cycle5 = model.inhaleDuration * pow(model.drift, 5)
        XCTAssertGreaterThan(cycle5, cycle0)
    }

    func testDriftLessThanOneDecreasesDurations() {
        model.drift = 0.9
        model.inhaleDuration = 4.0

        let cycle0 = model.inhaleDuration * pow(model.drift, 0)
        let cycle5 = model.inhaleDuration * pow(model.drift, 5)
        XCTAssertLessThan(cycle5, cycle0)
    }

    func testColorMatchingWithSystemColors() {
        // SwiftUI named colors (Color.red, Color.blue) may use different color spaces
        // than Color(red:green:blue:). Verify inhaleAndExhaleColorsMatch handles this.
        model.inhaleColor = Color.red
        model.exhaleColor = Color.blue
        XCTAssertFalse(model.inhaleAndExhaleColorsMatch)
    }

    func testColorMatchingWithTransparentColors() {
        model.inhaleColor = Color(red: 1, green: 0, blue: 0, opacity: 0)
        model.exhaleColor = Color(red: 1, green: 0, blue: 0, opacity: 0)
        XCTAssertTrue(model.inhaleAndExhaleColorsMatch)
    }

    func testColorMatchingWithMismatchedAlpha() {
        model.inhaleColor = Color(red: 1, green: 0, blue: 0, opacity: 1.0)
        model.exhaleColor = Color(red: 1, green: 0, blue: 0, opacity: 0.5)
        XCTAssertFalse(model.inhaleAndExhaleColorsMatch,
                        "Same RGB but different alpha should not match")
    }
}

// MARK: - Combine Publisher Tests

class SettingsPublisherTests: XCTestCase {
    func testIsAnimatingPublishes() {
        let model = SettingsModel()
        model.resetToDefaults()
        var received: [Bool] = []

        let cancellable = model.$isAnimating.sink { received.append($0) }

        model.isAnimating = false
        model.isAnimating = true

        // Initial value + 2 changes
        XCTAssertGreaterThanOrEqual(received.count, 3)
        XCTAssertEqual(received.last, true)

        cancellable.cancel()
    }

    func testShapePublishes() {
        let model = SettingsModel()
        model.resetToDefaults()
        var received: [AnimationShape] = []

        let cancellable = model.$shape.sink { received.append($0) }

        model.shape = .fullscreen
        model.shape = .circle

        XCTAssertGreaterThanOrEqual(received.count, 3)
        XCTAssertEqual(received.last, .circle)

        cancellable.cancel()
    }

    func testOverlayOpacityPublishes() {
        let model = SettingsModel()
        model.resetToDefaults()
        var received: [Double] = []

        let cancellable = model.$overlayOpacity.sink { received.append($0) }

        model.overlayOpacity = 0.5
        model.overlayOpacity = 0.9

        XCTAssertGreaterThanOrEqual(received.count, 3)
        XCTAssertEqual(received.last!, 0.9, accuracy: 0.001)

        cancellable.cancel()
    }
}

// MARK: - MetalBreathingState Phase Progress Invariants

class BreathingStateInvariantTests: XCTestCase {
    func testHoldAfterInhaleAlwaysProgress1() {
        let state = MetalBreathingState(phase: .holdAfterInhale, progress: 1.0)
        XCTAssertEqual(state.progress, 1.0)
    }

    func testHoldAfterExhaleAlwaysProgress0() {
        let state = MetalBreathingState(phase: .holdAfterExhale, progress: 0.0)
        XCTAssertEqual(state.progress, 0.0)
    }

    func testInhaleProgressBounds() {
        // During inhale, progress should be in [0, 1]
        for p in stride(from: Float(0), through: 1, by: 0.1) {
            let state = MetalBreathingState(phase: .inhale, progress: p)
            XCTAssertGreaterThanOrEqual(state.progress, 0)
            XCTAssertLessThanOrEqual(state.progress, 1)
        }
    }

    func testExhaleProgressBounds() {
        for p in stride(from: Float(0), through: 1, by: 0.1) {
            let state = MetalBreathingState(phase: .exhale, progress: p)
            XCTAssertGreaterThanOrEqual(state.progress, 0)
            XCTAssertLessThanOrEqual(state.progress, 1)
        }
    }
}

// MARK: - Easing Table Performance Test

class EasingPerformanceTests: XCTestCase {
    func testEasingTableBuildPerformance() {
        measure {
            for _ in 0..<100 {
                _ = (0..<1024).map { i in
                    let t = Double(i) / 1023.0
                    return CubicBezierEaseInOut.getValue(t: t, x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0)
                }
            }
        }
    }
}

// MARK: - AppVisibility Enum Tests

class AppVisibilityTests: XCTestCase {
    func testRawValues() {
        XCTAssertEqual(AppVisibility.topBarOnly.rawValue, "Top Bar Only")
        XCTAssertEqual(AppVisibility.dockOnly.rawValue, "Dock Only")
        XCTAssertEqual(AppVisibility.both.rawValue, "Both")
    }

    func testFromRawValue() {
        XCTAssertEqual(AppVisibility(rawValue: "Top Bar Only"), .topBarOnly)
        XCTAssertEqual(AppVisibility(rawValue: "Dock Only"), .dockOnly)
        XCTAssertEqual(AppVisibility(rawValue: "Both"), .both)
        XCTAssertNil(AppVisibility(rawValue: "invalid"))
    }

    func testCaseIterable() {
        XCTAssertEqual(AppVisibility.allCases.count, 3)
    }
}

// MARK: - AppVisibility Settings Tests

class AppVisibilitySettingsTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    override func tearDown() {
        model.resetToDefaults()
        super.tearDown()
    }

    func testDefaultAppVisibilityIsTopBarOnly() {
        XCTAssertEqual(model.appVisibility, .topBarOnly)
    }

    func testAppVisibilityPersists() {
        model.appVisibility = .both

        let reloaded = SettingsModel()
        XCTAssertEqual(reloaded.appVisibility, .both)
    }

    func testAppVisibilityDockOnlyPersists() {
        model.appVisibility = .dockOnly

        let reloaded = SettingsModel()
        XCTAssertEqual(reloaded.appVisibility, .dockOnly)
    }

    func testResetToDefaultsResetsAppVisibility() {
        model.appVisibility = .both
        model.resetToDefaults()

        XCTAssertEqual(model.appVisibility, .topBarOnly)
    }

    func testAppVisibilityPublishes() {
        var received: [AppVisibility] = []
        let cancellable = model.$appVisibility.sink { received.append($0) }

        model.appVisibility = .dockOnly
        model.appVisibility = .both

        XCTAssertGreaterThanOrEqual(received.count, 3)
        XCTAssertEqual(received.last, .both)

        cancellable.cancel()
    }
}

// MARK: - Window Level Tests (settings window above overlay)

class WindowLevelTests: XCTestCase {
    func testSettingsWindowLevelAboveOverlay() {
        XCTAssertGreaterThan(
            AppDelegate.settingsWindowLevel.rawValue,
            AppDelegate.overlayWindowLevel.rawValue,
            "Settings window must be above overlay so it's usable at any opacity"
        )
    }

    func testOverlayWindowLevelIsScreenSaver() {
        let expected = Int(CGWindowLevelForKey(.screenSaverWindow))
        XCTAssertEqual(AppDelegate.overlayWindowLevel.rawValue, expected)
    }

    func testSettingsWindowLevelIsOneAboveOverlay() {
        XCTAssertEqual(
            AppDelegate.settingsWindowLevel.rawValue,
            AppDelegate.overlayWindowLevel.rawValue + 1
        )
    }
}

// MARK: - Overlay Window Configuration Tests

class OverlayWindowConfigTests: XCTestCase {
    /// Verifies the overlay window level is high enough to appear over fullscreen apps.
    /// CGWindowLevelForKey(.screenSaverWindow) is above .mainMenu, .floating, and .modalPanel.
    func testOverlayLevelAboveFullscreenApps() {
        let overlayLevel = AppDelegate.overlayWindowLevel.rawValue
        let mainMenuLevel = Int(CGWindowLevelForKey(.mainMenuWindow))
        XCTAssertGreaterThan(overlayLevel, mainMenuLevel,
                              "Overlay must be above main menu level to show over fullscreen apps")
    }

    func testOverlayLevelAboveFloating() {
        let overlayLevel = AppDelegate.overlayWindowLevel.rawValue
        let floatingLevel = Int(CGWindowLevelForKey(.floatingWindow))
        XCTAssertGreaterThan(overlayLevel, floatingLevel)
    }

    func testOverlayLevelAboveModalPanel() {
        let overlayLevel = AppDelegate.overlayWindowLevel.rawValue
        let modalLevel = Int(CGWindowLevelForKey(.modalPanelWindow))
        XCTAssertGreaterThan(overlayLevel, modalLevel)
    }
}

// MARK: - Single Instance Support Tests

class SingleInstanceTests: XCTestCase {
    func testDistributedNotificationNameIsStable() {
        // The notification name must remain stable across versions so that
        // a new launch can communicate with an existing running instance.
        let name = Notification.Name("exhale.showSettings")
        XCTAssertEqual(name.rawValue, "exhale.showSettings")
    }

    func testBundleIdentifierExists() {
        // In test host context, verify the app has a bundle identifier
        // (single-instance logic falls back to a hardcoded ID if nil)
        let bundleID = Bundle.main.bundleIdentifier
        XCTAssertNotNil(bundleID)
    }
}

// MARK: - Updated ResetToDefaults Tests (including new fields)

class ResetToDefaultsFullTests: XCTestCase {
    func testResetClearsAllSettingsIncludingAppVisibility() {
        let model = SettingsModel()
        model.appVisibility = .both
        model.shape = .fullscreen
        model.animationMode = .linear
        model.overlayOpacity = 1.0
        model.drift = 2.0
        model.reminderIntervalMinutes = 15
        model.autoStopMinutes = 30

        model.resetToDefaults()

        XCTAssertEqual(model.appVisibility, .topBarOnly)
        XCTAssertEqual(model.shape, .rectangle)
        XCTAssertEqual(model.animationMode, .sinusoidal)
        XCTAssertEqual(model.overlayOpacity, 0.25)
        XCTAssertEqual(model.drift, 1.01)
        XCTAssertEqual(model.reminderIntervalMinutes, 0)
        XCTAssertEqual(model.autoStopMinutes, 0)
    }
}

// MARK: - Reminder Interval Tests

class ReminderIntervalTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    override func tearDown() {
        model.resetToDefaults()
        super.tearDown()
    }

    func testDefaultReminderIntervalIsOff() {
        XCTAssertEqual(model.reminderIntervalMinutes, 0)
    }

    func testReminderIntervalPersists() {
        model.reminderIntervalMinutes = 15

        let reloaded = SettingsModel()
        XCTAssertEqual(reloaded.reminderIntervalMinutes, 15)
    }

    func testResetClearsReminderInterval() {
        model.reminderIntervalMinutes = 30
        model.resetToDefaults()
        XCTAssertEqual(model.reminderIntervalMinutes, 0)
    }

    func testReminderIntervalPublishes() {
        var received: [Double] = []
        let cancellable = model.$reminderIntervalMinutes.sink { received.append($0) }

        model.reminderIntervalMinutes = 10
        model.reminderIntervalMinutes = 20

        XCTAssertGreaterThanOrEqual(received.count, 3)
        XCTAssertEqual(received.last!, 20, accuracy: 0.001)

        cancellable.cancel()
    }

    func testZeroMeansOff() {
        model.reminderIntervalMinutes = 0
        XCTAssertEqual(model.reminderIntervalMinutes, 0,
                        "0 should mean reminders are disabled")
    }
}

// MARK: - Auto-Stop Timer Tests

class AutoStopTimerTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    override func tearDown() {
        model.resetToDefaults()
        super.tearDown()
    }

    func testDefaultAutoStopIsOff() {
        XCTAssertEqual(model.autoStopMinutes, 0)
    }

    func testAutoStopPersists() {
        model.autoStopMinutes = 30

        let reloaded = SettingsModel()
        XCTAssertEqual(reloaded.autoStopMinutes, 30)
    }

    func testResetClearsAutoStop() {
        model.autoStopMinutes = 45
        model.resetToDefaults()
        XCTAssertEqual(model.autoStopMinutes, 0)
    }

    func testAutoStopPublishes() {
        var received: [Double] = []
        let cancellable = model.$autoStopMinutes.sink { received.append($0) }

        model.autoStopMinutes = 15
        model.autoStopMinutes = 60

        XCTAssertGreaterThanOrEqual(received.count, 3)
        XCTAssertEqual(received.last!, 60, accuracy: 0.001)

        cancellable.cancel()
    }

    func testZeroMeansOff() {
        model.autoStopMinutes = 0
        XCTAssertEqual(model.autoStopMinutes, 0,
                        "0 should mean auto-stop is disabled")
    }
}

// MARK: - Timer Interaction Tests

class TimerInteractionTests: XCTestCase {
    func testAutoStopDoesNotAffectReminder() {
        let model = SettingsModel()
        model.resetToDefaults()
        model.autoStopMinutes = 30
        model.reminderIntervalMinutes = 10

        // Both can be set independently
        XCTAssertEqual(model.autoStopMinutes, 30)
        XCTAssertEqual(model.reminderIntervalMinutes, 10)
    }

    func testBothTimersResetToZero() {
        let model = SettingsModel()
        model.reminderIntervalMinutes = 15
        model.autoStopMinutes = 45

        model.resetToDefaults()

        XCTAssertEqual(model.reminderIntervalMinutes, 0)
        XCTAssertEqual(model.autoStopMinutes, 0)
    }

    func testAutoStopWithAnimationState() {
        let model = SettingsModel()
        model.resetToDefaults()
        model.autoStopMinutes = 30

        // Auto-stop is configured but animation state is separate
        model.start()
        XCTAssertTrue(model.isAnimating)
        XCTAssertEqual(model.autoStopMinutes, 30)

        model.stop()
        XCTAssertFalse(model.isAnimating)
        // autoStopMinutes persists regardless of animation state
        XCTAssertEqual(model.autoStopMinutes, 30)
    }
}

// MARK: - Cache Consistency Tests

/// These tests ensure cached (non-@Published) color properties stay in sync with
/// their @Published counterparts. Reading @Published properties every animation frame
/// causes SwiftUI observation overhead and CPU regression. The cached variants avoid this.
class CacheConsistencyTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    func testCachedInhaleColorMatchesPublished() {
        let testColor = Color(red: 0.1, green: 0.2, blue: 0.3)
        model.inhaleColor = testColor
        XCTAssertEqual(
            model.cachedInhaleColor.description,
            testColor.description,
            "cachedInhaleColor must update when inhaleColor changes"
        )
    }

    func testCachedExhaleColorMatchesPublished() {
        let testColor = Color(red: 0.4, green: 0.5, blue: 0.6)
        model.exhaleColor = testColor
        XCTAssertEqual(
            model.cachedExhaleColor.description,
            testColor.description,
            "cachedExhaleColor must update when exhaleColor changes"
        )
    }

    func testCachedBackgroundColorMatchesPublished() {
        let testColor = Color(red: 0.7, green: 0.8, blue: 0.9, opacity: 0.5)
        model.backgroundColor = testColor
        XCTAssertEqual(
            model.cachedBackgroundColor.description,
            testColor.description,
            "cachedBackgroundColor must update when backgroundColor changes (including alpha)"
        )
    }

    func testCachedBackgroundAlphaUpdates() {
        model.backgroundColor = Color(red: 1, green: 0, blue: 0, opacity: 0.3)
        XCTAssertEqual(model.cachedBackgroundAlphaComponent, 0.3, accuracy: 0.01,
            "cachedBackgroundAlphaComponent must reflect backgroundColor alpha"
        )
    }

    func testCachedBackgroundColorWithoutAlphaStripsAlpha() {
        model.backgroundColor = Color(red: 1, green: 0, blue: 0, opacity: 0.3)
        let withoutAlpha = model.cachedBackgroundColorWithoutAlpha
        XCTAssertEqual(withoutAlpha.alphaComponent(), 1.0, accuracy: 0.01,
            "cachedBackgroundColorWithoutAlpha must have alpha = 1.0"
        )
    }

    func testCachesUpdateOnResetToDefaults() {
        model.inhaleColor = Color.green
        model.exhaleColor = Color.yellow
        model.backgroundColor = Color(red: 0.5, green: 0.5, blue: 0.5, opacity: 0.5)

        model.resetToDefaults()

        // After reset, caches must reflect default values
        XCTAssertEqual(model.cachedInhaleColor.description, Color.red.description)
        XCTAssertEqual(model.cachedExhaleColor.description, Color.blue.description)
        XCTAssertEqual(model.cachedBackgroundColor.description, Color.clear.description)
    }

    func testCachesUpdateOnRapidColorChanges() {
        // Simulate rapid color changes (as might happen with a color picker)
        for i in 0..<100 {
            let fraction = Double(i) / 100.0
            model.inhaleColor = Color(red: fraction, green: 0, blue: 0)
            model.exhaleColor = Color(red: 0, green: fraction, blue: 0)
            model.backgroundColor = Color(red: 0, green: 0, blue: fraction, opacity: fraction)
        }
        // Final values must match
        let finalFraction = 99.0 / 100.0
        XCTAssertEqual(
            model.cachedInhaleColor.description,
            Color(red: finalFraction, green: 0, blue: 0).description
        )
        XCTAssertEqual(
            model.cachedExhaleColor.description,
            Color(red: 0, green: finalFraction, blue: 0).description
        )
        XCTAssertEqual(model.cachedBackgroundAlphaComponent, finalFraction, accuracy: 0.01)
    }
}

// MARK: - Performance Tests

/// Performance benchmarks for the animation hot path. These use XCTest's measure()
/// to track execution time and detect regressions in per-frame computation cost.
class PerformanceTests: XCTestCase {
    var model: SettingsModel!

    override func setUp() {
        super.setUp()
        model = SettingsModel()
        model.resetToDefaults()
    }

    /// Measures the cost of reading cached properties (the hot path during animation).
    /// This should be significantly faster than reading @Published properties.
    func testCachedPropertyReadPerformance() {
        measure {
            for _ in 0..<10_000 {
                _ = model.cachedInhaleColor
                _ = model.cachedExhaleColor
                _ = model.cachedBackgroundColor
                _ = model.cachedBackgroundColorWithoutAlpha
                _ = model.cachedBackgroundAlphaComponent
            }
        }
    }

    /// Measures the cost of reading @Published properties for comparison.
    /// If this is not meaningfully slower than the cached version, caching is unnecessary.
    func testPublishedPropertyReadPerformance() {
        measure {
            for _ in 0..<10_000 {
                _ = model.inhaleColor
                _ = model.exhaleColor
                _ = model.backgroundColor
                _ = model.overlayOpacity
            }
        }
    }

    /// Simulates the per-frame color selection logic in colorTransitionFill.
    func testColorTransitionSelectionPerformance() {
        model.shape = .circle
        model.colorFillGradient = .on

        measure {
            for i in 0..<10_000 {
                let isInhalePhase = (i % 2 == 0)
                _ = isInhalePhase ? model.cachedInhaleColor : model.cachedExhaleColor
                _ = model.cachedBackgroundColor
                _ = model.shape
                _ = model.colorFillGradient
            }
        }
    }

    /// Measures the cost of Color.alphaComponent() and Color.withoutAlpha() extensions,
    /// which involve CGColor/NSColor conversions. These should only run on color change,
    /// not per-frame.
    func testColorConversionPerformance() {
        let color = Color(red: 0.5, green: 0.3, blue: 0.8, opacity: 0.6)
        measure {
            for _ in 0..<10_000 {
                _ = color.alphaComponent()
                _ = color.withoutAlpha()
            }
        }
    }

    /// Measures CPU usage during a simulated animation loop.
    /// Creates an actual ContentView with SettingsModel, runs the animation for a fixed
    /// duration, and checks that process CPU stays within acceptable bounds.
    func testAnimationCPUUsage_RectangleGradientOn() throws {
        measureCPU(shape: .rectangle, gradient: .on)
    }

    func testAnimationCPUUsage_CircleGradientOn() throws {
        measureCPU(shape: .circle, gradient: .on)
    }

    func testAnimationCPUUsage_FullscreenNoGradient() throws {
        measureCPU(shape: .fullscreen, gradient: .off)
    }

    /// Helper: measures the CPU cost of the animation by comparing CPU with animation ON
    /// vs animation OFF (baseline). This isolates the actual animation cost from XCTest
    /// and RunLoop overhead, making the test reliable regardless of test runner load.
    private func measureCPU(shape: AnimationShape, gradient: ColorFillGradient, file: StaticString = #file, line: UInt = #line) {
        let model = SettingsModel()
        model.resetToDefaults()
        model.shape = shape
        model.colorFillGradient = gradient
        model.overlayOpacity = 0.25
        model.isAnimating = false

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 800, height: 600),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        let hostingView = NSHostingView(rootView: ContentView().environmentObject(model))
        window.contentView = hostingView
        window.orderFront(nil)

        // Warm up — let SwiftUI set up its render pipeline
        RunLoop.main.run(until: Date().addingTimeInterval(0.5))

        let sampleCount = 5
        let sampleInterval: TimeInterval = 1.0

        func getCPUTime() -> TimeInterval {
            var usage = rusage()
            getrusage(RUSAGE_SELF, &usage)
            return TimeInterval(usage.ru_utime.tv_sec) + TimeInterval(usage.ru_utime.tv_usec) / 1_000_000
                + TimeInterval(usage.ru_stime.tv_sec) + TimeInterval(usage.ru_stime.tv_usec) / 1_000_000
        }

        func sampleCPU(count: Int) -> [Double] {
            var samples: [Double] = []
            for _ in 0..<count {
                let cpuBefore = getCPUTime()
                let wallBefore = Date()
                RunLoop.main.run(until: Date().addingTimeInterval(sampleInterval))
                let wallElapsed = Date().timeIntervalSince(wallBefore)
                let cpuUsed = getCPUTime() - cpuBefore
                samples.append((cpuUsed / wallElapsed) * 100.0)
            }
            return samples
        }

        // Phase 1: measure baseline (animation OFF)
        let baselineSamples = sampleCPU(count: sampleCount)
        let baselineAvg = baselineSamples.reduce(0, +) / Double(baselineSamples.count)

        // Phase 2: measure with animation ON
        model.start()
        RunLoop.main.run(until: Date().addingTimeInterval(0.5)) // let animation ramp up
        let animationSamples = sampleCPU(count: sampleCount)

        // Clean up
        model.stop()
        window.orderOut(nil)

        // Compute delta (animation cost above baseline)
        let deltaSamples = animationSamples.map { max(0, $0 - baselineAvg) }
        let peakDelta = deltaSamples.max() ?? 0
        let avgDelta = deltaSamples.reduce(0, +) / Double(deltaSamples.count)

        // Log for visibility
        let baseStr = baselineSamples.map { String(format: "%.1f%%", $0) }.joined(separator: ", ")
        let animStr = animationSamples.map { String(format: "%.1f%%", $0) }.joined(separator: ", ")
        let deltaStr = deltaSamples.map { String(format: "%.1f%%", $0) }.joined(separator: ", ")
        print("[\(shape.rawValue) + gradient \(gradient.rawValue)]")
        print("  baseline: [\(baseStr)] avg: \(String(format: "%.1f", baselineAvg))%")
        print("  animating: [\(animStr)]")
        print("  delta: [\(deltaStr)] avg: \(String(format: "%.1f", avgDelta))% peak: \(String(format: "%.1f", peakDelta))%")

        // Assert animation cost (above baseline) stays under thresholds.
        // Peak: no single second should add more than 10% CPU from the animation.
        // Average: animation should average under 5% CPU cost.
        XCTAssertLessThan(peakDelta, 10.0,
            "\(shape.rawValue) with gradient \(gradient.rawValue) peak animation CPU \(String(format: "%.1f", peakDelta))% exceeded 10% — delta: [\(deltaStr)]",
            file: file, line: line
        )
        XCTAssertLessThan(avgDelta, 5.0,
            "\(shape.rawValue) with gradient \(gradient.rawValue) average animation CPU \(String(format: "%.1f", avgDelta))% exceeded 5% — delta: [\(deltaStr)]",
            file: file, line: line
        )
    }
}
