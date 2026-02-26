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
        let lastColor = isInhalePhase ? settingsModel.inhaleColor : settingsModel.exhaleColor
        let backgroundColor = settingsModel.cachedBackgroundColorWithoutAlpha

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

struct ContentView: View {
    @EnvironmentObject var settingsModel: SettingsModel
    @State private var animationProgress: CGFloat = 0
    @State private var breathingPhase: BreathingPhase = .inhale
    @State private var overlayOpacity: Double = 0.1
    @State private var showSettings = false
    @State private var cycleCount: Int = 0
    @State private var cachedMaxCircleScale: CGFloat = 1
    @State private var animationSessionIdentifier: Int = 0
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
                                            ? settingsModel.inhaleColor
                                            : settingsModel.exhaleColor
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
                    }
                }
            }

            if showSettings {
                SettingsView(
                    showSettings: $showSettings,
                    inhaleColor: $settingsModel.inhaleColor,
                    exhaleColor: $settingsModel.exhaleColor,
                    backgroundColor: $settingsModel.backgroundColor,
                    colorFillType: $settingsModel.colorFillGradient,
                    inhaleDuration: $settingsModel.inhaleDuration,
                    postInhaleHoldDuration: $settingsModel.postInhaleHoldDuration,
                    exhaleDuration: $settingsModel.exhaleDuration,
                    postExhaleHoldDuration: $settingsModel.postExhaleHoldDuration,
                    drift: $settingsModel.drift,
                    overlayOpacity: $overlayOpacity,
                    shape: Binding<AnimationShape>(
                        get: { self.settingsModel.shape },
                        set: { self.settingsModel.shape = $0 }
                    ),
                    animationMode: Binding<AnimationMode>(
                        get: { self.settingsModel.animationMode },
                        set: { self.settingsModel.animationMode = $0 }
                    ),
                    randomizedTimingInhale: $settingsModel.randomizedTimingInhale,
                    randomizedTimingPostInhaleHold: $settingsModel.randomizedTimingPostInhaleHold,
                    randomizedTimingExhale: $settingsModel.randomizedTimingExhale,
                    randomizedTimingPostExhaleHold: $settingsModel.randomizedTimingPostExhaleHold,
                    isAnimating: $settingsModel.isAnimating
                )
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
        .onChange(of: settingsModel.resetAnimation) { newValue in
            if newValue {
                resetAnimation()
                startBreathingCycle()
            }
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
        breathingPhase = .inhale
    }

    func stopCurrentAnimation() {
        // Stop the current animation
        animationSessionIdentifier += 1
        cycleCount = 0
        animationProgress = 0.0
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
