// ContentView.swift
import SwiftUI

extension Color {
  func alphaComponent() -> Double {
    guard let cg = self.cgColor,
          let ns = NSColor(cgColor: cg) else { return 1 }
    return Double(ns.alphaComponent)
  }
  func withoutAlpha() -> Color {
    guard let cg = self.cgColor,
          let ns = NSColor(cgColor: cg) else { return self }
    return Color(ns.withAlphaComponent(1))
  }
}

extension Shape {
    @ViewBuilder
    func colorTransitionFill(
        settingsModel: SettingsModel,
        animationProgress: CGFloat,
        breathingPhase: BreathingPhase,
        endRadius: CGFloat = 0
    ) -> some View {
        let isInhalePhase = breathingPhase == .inhale || breathingPhase == .holdAfterInhale
        let lastColor = isInhalePhase ? settingsModel.cachedInhaleColor : settingsModel.cachedExhaleColor
        let backgroundColor = settingsModel.cachedBackgroundColor

        switch settingsModel.colorFillGradient {
        case .off:
            self.fill(lastColor)

        case .inner:
            if settingsModel.shape == .rectangle {
                self.fill(
                    LinearGradient(
                        gradient: Gradient(colors: [lastColor, backgroundColor]),
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
            } else {
                self.fill(
                    RadialGradient(
                        gradient: Gradient(colors: [backgroundColor, lastColor]),
                        center: .center,
                        startRadius: 0,
                        endRadius: endRadius
                    )
                )
            }

        case .on:
            if settingsModel.shape == .rectangle {
                self.fill(
                    LinearGradient(
                        gradient: Gradient(colors: [backgroundColor, lastColor, backgroundColor]),
                        startPoint: .bottom,
                        endPoint: .top
                    )
                )
            } else {
                self.fill(
                    RadialGradient(
                        gradient: Gradient(colors: [backgroundColor, lastColor, backgroundColor]),
                        center: .center,
                        startRadius: 0,
                        endRadius: endRadius
                    )
                )
            }
        }
    }
}

// Traces half the screen perimeter: bottom-center → corner → side → top-center.
// Used for the hold-phase ripple effect. `rightSide: true` goes clockwise (right),
// `rightSide: false` goes counter-clockwise (left).
struct HalfPerimeterShape: Shape {
    let rightSide: Bool

    func path(in rect: CGRect) -> Path {
        var path = Path()
        let w = rect.width
        let h = rect.height

        if rightSide {
            path.move(to: CGPoint(x: w / 2, y: h))
            path.addLine(to: CGPoint(x: w, y: h))
            path.addLine(to: CGPoint(x: w, y: 0))
            path.addLine(to: CGPoint(x: w / 2, y: 0))
        } else {
            path.move(to: CGPoint(x: w / 2, y: h))
            path.addLine(to: CGPoint(x: 0, y: h))
            path.addLine(to: CGPoint(x: 0, y: 0))
            path.addLine(to: CGPoint(x: w / 2, y: 0))
        }

        return path
    }
}

struct ContentView: View {
    @EnvironmentObject var settingsModel: SettingsModel
    @State private var animationProgress: CGFloat = 0
    @State private var breathingPhase: BreathingPhase = .inhale
    @State private var cycleCount: Int = 0
    @State private var cachedMaxCircleScale: CGFloat = 1
    @State private var animationSessionIdentifier: Int = 0
    @State private var holdProgress: CGFloat = 0
    @State private var rippleOpacity: Double = 0
    var body: some View {
        ZStack {
            GeometryReader { geometry in
                let centerX = geometry.size.width / 2
                let centerY = geometry.size.height / 2

                if !settingsModel.isAnimating && !settingsModel.isPaused {
                    Color.clear.edgesIgnoringSafeArea(.all)
                } else {
                    if settingsModel.isPaused {
                        // Tint mode: keep the screen tinted using ONLY the configured overlay opacity
                        settingsModel.cachedBackgroundColorWithoutAlpha
                            .edgesIgnoringSafeArea(.all)
                            .opacity(settingsModel.overlayOpacity)
                    } else {
                        if settingsModel.shape != .fullscreen {
                            settingsModel.cachedBackgroundColorWithoutAlpha
                                .edgesIgnoringSafeArea(.all)
                                .opacity(min(settingsModel.cachedBackgroundAlphaComponent, settingsModel.overlayOpacity))
                        }

                        Group {
                            switch settingsModel.shape {
                            case .fullscreen:
                                Rectangle()
                                    .fill(
                                        (breathingPhase == .inhale || breathingPhase == .holdAfterInhale)
                                            ? settingsModel.cachedInhaleColor
                                            : settingsModel.cachedExhaleColor
                                    )
                                    .edgesIgnoringSafeArea(.all)

                            case .rectangle:
                                Rectangle()
                                    .colorTransitionFill(
                                        settingsModel: settingsModel,
                                        animationProgress: animationProgress,
                                        breathingPhase: breathingPhase
                                    )
                                    .frame(height: geometry.size.height)
                                    .scaleEffect(
                                        x: 1,
                                        y: animationProgress * (settingsModel.colorFillGradient == .on ? 2 : 1),
                                        anchor: .bottom
                                    )
                                    .position(x: centerX, y: centerY)

                            case .circle:
                                let minDimension = min(geometry.size.width, geometry.size.height)
                                let gradientScale: CGFloat = settingsModel.colorFillGradient == .on ? 2 : 1

                                let bakedSize = minDimension
                                    * animationProgress
                                    * cachedMaxCircleScale
                                    * animationProgress
                                    * gradientScale

                                Circle()
                                    .colorTransitionFill(
                                        settingsModel: settingsModel,
                                        animationProgress: animationProgress,
                                        breathingPhase: breathingPhase,
                                        endRadius: bakedSize / 2
                                    )
                                    .frame(width: bakedSize, height: bakedSize)
                                    .position(x: centerX, y: centerY)
                            }
                        }
                        .opacity(settingsModel.overlayOpacity)

                        // Screen-edge ripple during hold phases (driven by rippleOpacity)
                        if settingsModel.holdRippleEnabled && rippleOpacity > 0 {
                            let isExhale = breathingPhase == .holdAfterExhale
                                || (breathingPhase == .inhale && holdProgress > 0)
                            let phaseColor = isExhale
                                ? settingsModel.cachedExhaleColor
                                : settingsModel.cachedInhaleColor
                            let borderUnit = min(geometry.size.width, geometry.size.height) * 0.04
                            let useGradient = settingsModel.holdRippleMode == .gradient

                            let trailFrom = isExhale ? holdProgress : 0 as CGFloat
                            let trailTo   = isExhale ? 1 as CGFloat : holdProgress
                            let bandFrom  = isExhale ? holdProgress : max(0, holdProgress - 0.12)
                            let bandTo    = isExhale ? min(1, holdProgress + 0.12) : holdProgress

                            // When blurred, use a wider stroke so the glow is visible after
                            // the blur spreads it. The blur softens ALL edges: inner (toward
                            // center), and leading/trailing (along the perimeter).
                            let strokeWidth = useGradient ? borderUnit * 3 : borderUnit * 2
                            let blurRadius = useGradient ? borderUnit * 2 : 0 as CGFloat

                            Group {
                                // Trail glow (fills behind the sweep front)
                                HalfPerimeterShape(rightSide: true)
                                    .trim(from: trailFrom, to: trailTo)
                                    .stroke(phaseColor, style: StrokeStyle(lineWidth: strokeWidth, lineCap: .butt))
                                    .opacity(0.25)
                                HalfPerimeterShape(rightSide: false)
                                    .trim(from: trailFrom, to: trailTo)
                                    .stroke(phaseColor, style: StrokeStyle(lineWidth: strokeWidth, lineCap: .butt))
                                    .opacity(0.25)
                                // Leading band (bright sweep at the front)
                                HalfPerimeterShape(rightSide: true)
                                    .trim(from: bandFrom, to: bandTo)
                                    .stroke(phaseColor, style: StrokeStyle(lineWidth: strokeWidth, lineCap: .butt))
                                    .opacity(0.8)
                                HalfPerimeterShape(rightSide: false)
                                    .trim(from: bandFrom, to: bandTo)
                                    .stroke(phaseColor, style: StrokeStyle(lineWidth: strokeWidth, lineCap: .butt))
                                    .opacity(0.8)
                            }
                            .blur(radius: blurRadius)
                            .opacity(rippleOpacity)
                        }
                    }
                }
            }

        }
        .onAppear {
            cachedMaxCircleScale = Self.getMaxCircleScale()
            startBreathingCycle()
        }
        .onChange(of: settingsModel.isAnimating) { newValue in
            if newValue {
                guard !settingsModel.isPaused else { return }
                resetAnimation()
                startBreathingCycle()
            } else {
                resetAnimation()
            }
        }
        .onChange(of: settingsModel.isPaused) { newValue in
            if newValue {
                stopCurrentAnimation()
            } else if settingsModel.isAnimating {
                resumeBreathingCycle()
            }
        }
        .onReceive(settingsModel.resetAnimationSignal) { _ in
            resetAnimation()
            startBreathingCycle()
        }
        .onChange(of: settingsModel.shape) { _ in
            guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
            resetAnimation()
            startBreathingCycle()
        }
    }

    static func getMaxCircleScale() -> CGFloat {
        guard let screen = NSScreen.main else { return 1 }
        let screenWidth = screen.frame.width
        let screenHeight = screen.frame.height
        let maxDimension = max(screenWidth, screenHeight)
        return maxDimension / min(screenWidth, screenHeight)
    }

    func startBreathingCycle() {
        cycleCount = 0
        animationSessionIdentifier += 1
        inhale()
    }

    func inhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        let currentAnimationSessionIdentifier = animationSessionIdentifier
        var duration = settingsModel.inhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingInhale > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingInhale...settingsModel.randomizedTimingInhale)
        }
        duration = max(duration, 0.1)

        let animation: Animation = settingsModel.animationMode == .linear
            ? .linear(duration: duration)
            : .timingCurve(0.42, 0, 0.58, 1, duration: duration)

        // Fade ripple out over the first 10% of the inhale
        withAnimation(.linear(duration: duration * 0.1)) {
            rippleOpacity = 0
        }

        withAnimation(animation) {
            breathingPhase = .inhale
            animationProgress = 1.0
            if settingsModel.shape == .circle {
                animationProgress = 1
            }
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            guard currentAnimationSessionIdentifier == self.animationSessionIdentifier else { return }
            self.holdAfterInhale()
        }
    }

    func holdAfterInhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        let currentAnimationSessionIdentifier = animationSessionIdentifier
        var duration = settingsModel.postInhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingPostInhaleHold > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingPostInhaleHold...settingsModel.randomizedTimingPostInhaleHold)
        }
        duration = max(duration, 0.1)
        breathingPhase = .holdAfterInhale
        if settingsModel.holdRippleEnabled && settingsModel.postInhaleHoldDuration > 0 {
            holdProgress = 0
            rippleOpacity = 1
            withAnimation(.linear(duration: duration)) {
                holdProgress = 1
            }
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            guard currentAnimationSessionIdentifier == self.animationSessionIdentifier else { return }
            self.exhale()
        }
    }

    func exhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        let currentAnimationSessionIdentifier = animationSessionIdentifier
        var duration = settingsModel.exhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingExhale > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingExhale...settingsModel.randomizedTimingExhale)
        }
        duration = max(duration, 0.1)

        let animation: Animation = settingsModel.animationMode == .linear
            ? .linear(duration: duration)
            : .timingCurve(0.42, 0, 0.58, 1, duration: duration)

        // Fade ripple out over the first 10% of the exhale
        withAnimation(.linear(duration: duration * 0.1)) {
            rippleOpacity = 0
        }

        withAnimation(animation) {
            breathingPhase = .exhale
            animationProgress = 0.0
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            guard currentAnimationSessionIdentifier == self.animationSessionIdentifier else { return }
            self.holdAfterExhale()
        }
    }

    func holdAfterExhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        let currentAnimationSessionIdentifier = animationSessionIdentifier
        var duration = settingsModel.postExhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingPostExhaleHold > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingPostExhaleHold...settingsModel.randomizedTimingPostExhaleHold)
        }
        duration = max(duration, 0.1)
        breathingPhase = .holdAfterExhale
        if settingsModel.holdRippleEnabled && settingsModel.postExhaleHoldDuration > 0 {
            holdProgress = 1
            rippleOpacity = 1
            withAnimation(.linear(duration: duration)) {
                holdProgress = 0
            }
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            guard currentAnimationSessionIdentifier == self.animationSessionIdentifier else { return }
            guard self.settingsModel.isAnimating else { return self.resetAnimation() }
            self.cycleCount += 1
            self.inhale()
        }
    }

    func resetAnimation() {
        animationSessionIdentifier += 1
        cycleCount = 0
        animationProgress = 0.0
        holdProgress = 0
        rippleOpacity = 0
        breathingPhase = .inhale
    }

    func stopCurrentAnimation() {
        animationSessionIdentifier += 1
        cycleCount = 0
        animationProgress = 0.0
        holdProgress = 0
        rippleOpacity = 0
    }

    func resumeBreathingCycle() {
        // Resume the breathing cycle
        switch breathingPhase {
        case .inhale:
            inhale()
        case .holdAfterInhale:
            holdAfterInhale()
        case .exhale:
            exhale()
        case .holdAfterExhale:
            holdAfterExhale()
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
