// ContentView.swift
import SwiftUI

extension Shape {
    @ViewBuilder
    func colorTransitionFill(settingsModel: SettingsModel, animationProgress: CGFloat, breathingPhase: BreathingPhase, endRadius: CGFloat = 0) -> some View {
        let isInhalePhase = breathingPhase == .inhale || breathingPhase == .holdAfterInhale
        let lastColor = isInhalePhase ? settingsModel.inhaleColor : settingsModel.exhaleColor
        
        let colorSequence: [Color] = [settingsModel.backgroundColor, lastColor, settingsModel.backgroundColor]
        
        switch settingsModel.colorFillGradient {
        case .off:
            self.fill(lastColor)
        case .inner:
            if settingsModel.shape == .rectangle {
                let gradient = LinearGradient(
                    gradient: Gradient(colors: [lastColor, settingsModel.backgroundColor]),
                    startPoint: .top,
                    endPoint: .bottom
                )
                self.fill(gradient)
            } else {
                let gradient = RadialGradient(
                    gradient: Gradient(colors: [settingsModel.backgroundColor, lastColor]),
                    center: .center,
                    startRadius: 0,
                    endRadius: endRadius
                )
                self.fill(gradient)
            }
        case .on:
            if settingsModel.shape == .rectangle {
                let gradient = LinearGradient(
                    gradient: Gradient(colors: colorSequence),
                    startPoint: .bottom,
                    endPoint: .top
                )
                self.fill(gradient)
            } else {
                let gradient = RadialGradient(
                    gradient: Gradient(colors: colorSequence),
                    center: .center,
                    startRadius: 0,
                    endRadius: endRadius
                )
                self.fill(gradient)
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
    
    var maxCircleScale: CGFloat {
        guard let screen = NSScreen.main else { return settingsModel.colorFillGradient == .on ? 2 : 1 }
        let screenWidth = screen.frame.width
        let screenHeight = screen.frame.height
        let maxDimension = max(screenWidth, screenHeight)
        return maxDimension / min(screenWidth, screenHeight)
    }
    
    var body: some View {
        ZStack {
            GeometryReader { geometry in
                if !settingsModel.isAnimating && !settingsModel.isPaused {
                    Color.clear.edgesIgnoringSafeArea(.all)
                } else {
                    // FULLSCREEN MODE
                    if settingsModel.shape == .fullscreen {
                        Rectangle()
                            .fill(
                                (breathingPhase == .inhale || breathingPhase == .holdAfterInhale)
                                ? settingsModel.inhaleColor
                                : settingsModel.exhaleColor
                            )
                            .edgesIgnoringSafeArea(.all)
                            .opacity(settingsModel.overlayOpacity)   // global slider
                    }
                    // RECTANGLE & CIRCLE: we draw
                    //    The _outside_ region tinted with backgroundColor’s own alpha
                    //    The breathing shape tinted with inhale/exhale color & overlayOpacity
                    else {
                        // “donut” or “inverted mask” path
                        Path { path in
                            let size = geometry.size
                            path.addRect(CGRect(origin: .zero, size: size))
                            
                            switch settingsModel.shape {
                            case .rectangle:
                                let fullHeight = size.height
                                let effectiveHeight = fullHeight * animationProgress
                                * (settingsModel.colorFillGradient == .on ? 2 : 1)
                                let yOffset = (fullHeight - effectiveHeight)/2
                                let rect = CGRect(
                                    x: 0,
                                    y: yOffset,
                                    width: size.width,
                                    height: effectiveHeight
                                )
                                path.addRect(rect)
                                
                            case .circle:
                                let diameter = min(size.width, size.height)
                                * animationProgress * maxCircleScale
                                let circleRect = CGRect(
                                    x: (size.width - diameter)/2,
                                    y: (size.height - diameter)/2,
                                    width: diameter,
                                    height: diameter
                                )
                                path.addEllipse(in: circleRect)
                                
                            default: break
                            }
                        }
                        .fill(style: FillStyle(eoFill: true))              // even-odd rule
                        .foregroundColor(settingsModel.backgroundColor)     // respects the color’s own alpha
                        
                        Group {
                            if settingsModel.shape == .rectangle {
                                Rectangle()
                                    .colorTransitionFill(
                                        settingsModel: settingsModel,
                                        animationProgress: animationProgress,
                                        breathingPhase: breathingPhase
                                    )
                                    .frame(height: geometry.size.height)
                                    .scaleEffect(
                                        x: 1,
                                        y: animationProgress
                                        * (settingsModel.colorFillGradient == .on ? 2 : 1),
                                        anchor: .bottom
                                    )
                                    .position(
                                        x: geometry.size.width/2,
                                        y: geometry.size.height/2
                                    )
                            } else { // .circle
                                Circle()
                                    .colorTransitionFill(
                                        settingsModel: settingsModel,
                                        animationProgress: animationProgress,
                                        breathingPhase: breathingPhase,
                                        endRadius: (min(geometry.size.width, geometry.size.height)
                                                    * animationProgress * maxCircleScale)/2
                                    )
                                    .frame(
                                        width: min(geometry.size.width, geometry.size.height)
                                        * animationProgress * maxCircleScale,
                                        height: min(geometry.size.width, geometry.size.height)
                                        * animationProgress * maxCircleScale
                                    )
                                    .scaleEffect(
                                        x: animationProgress
                                        * (settingsModel.colorFillGradient == .on ? 2 : 1),
                                        y: animationProgress
                                        * (settingsModel.colorFillGradient == .on ? 2 : 1),
                                        anchor: .center
                                    )
                                    .position(x: geometry.size.width/2,
                                              y: geometry.size.height/2)
                            }
                        }
                        .opacity(settingsModel.overlayOpacity)  // only shapes respect the slider
                    }
                }
            }
            .edgesIgnoringSafeArea(.all)
            
            
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
        .onAppear(perform: startBreathingCycle)
        .onChange(of: settingsModel.isAnimating) { newValue in
            if !newValue {
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
    }
    
    func startBreathingCycle() {
        cycleCount = 0
        inhale()
    }
    
    func inhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        var duration = settingsModel.inhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingInhale > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingInhale...settingsModel.randomizedTimingInhale)
        }
        duration = max(duration, 0.1)
        
        let animation: Animation = settingsModel.animationMode == .linear ? .linear(duration: duration) : .timingCurve(0.42, 0, 0.58, 1, duration: duration)
        
        withAnimation(animation) {
            breathingPhase = .inhale
            animationProgress = 1.0
            if settingsModel.shape == .circle {
                animationProgress = 1
            }
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            holdAfterInhale()
        }
    }
    
    func holdAfterInhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        var duration = settingsModel.postInhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingPostInhaleHold > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingPostInhaleHold...settingsModel.randomizedTimingPostInhaleHold)
        }
        duration = max(duration, 0.1)
        breathingPhase = .holdAfterInhale
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            exhale()
        }
    }
    
    func exhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        var duration = settingsModel.exhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingExhale > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingExhale...settingsModel.randomizedTimingExhale)
        }
        duration = max(duration, 0.1)
        
        let animation: Animation = settingsModel.animationMode == .linear ? .linear(duration: duration) : .timingCurve(0.42, 0, 0.58, 1, duration: duration)
        
        withAnimation(animation) {
            breathingPhase = .exhale
            animationProgress = 0.0
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            holdAfterExhale()
        }
    }
    
    func holdAfterExhale() {
        guard settingsModel.isAnimating && !settingsModel.isPaused else { return }
        var duration = settingsModel.postExhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingPostExhaleHold > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingPostExhaleHold...settingsModel.randomizedTimingPostExhaleHold)
        }
        duration = max(duration, 0.1)
        breathingPhase = .holdAfterExhale
        
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            guard self.settingsModel.isAnimating else { return self.resetAnimation() }
            self.cycleCount += 1
            self.inhale()
        }
    }
    
    func resetAnimation() {
        cycleCount = 0
        animationProgress = 0.0
        breathingPhase = .inhale
    }
    
    func stopCurrentAnimation() {
        // Stop the current animation
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
