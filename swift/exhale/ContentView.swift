// ContentView.swift
import SwiftUI

extension Shape {
    func conditionalFill<S1: ShapeStyle, S2: ShapeStyle>(_ condition: Bool, ifTrue: S1, ifFalse: S2) -> some View {
        Group {
            if condition {
                self.fill(ifTrue)
            } else {
                self.fill(ifFalse)
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
        guard let screen = NSScreen.main else { return 1.0 }
        let screenWidth = screen.frame.width
        let screenHeight = screen.frame.height
        let maxDimension = max(screenWidth, screenHeight)
        return maxDimension / min(screenWidth, screenHeight)
    }
    
    var body: some View {
        ZStack {
            GeometryReader { geometry in
                ZStack {
                    settingsModel.backgroundColor.edgesIgnoringSafeArea(.all)
                    if settingsModel.shape == .rectangle {
                        let fillColor = breathingPhase == .inhale || breathingPhase == .holdAfterInhale ? settingsModel.inhaleColor : settingsModel.exhaleColor
                        let gradient = LinearGradient(gradient: Gradient(colors: [fillColor, settingsModel.backgroundColor]), startPoint: .top, endPoint: .bottom)
                        Rectangle()
                            .conditionalFill(settingsModel.colorFillType == .linear, ifTrue: gradient, ifFalse: fillColor)
                            .frame(height: animationProgress * geometry.size.height)
                            .position(x: geometry.size.width / 2, y: geometry.size.height - (animationProgress * geometry.size.height) / 2)
                    } else {
                        let fillColor = breathingPhase == .inhale || breathingPhase == .holdAfterInhale ? settingsModel.inhaleColor : settingsModel.exhaleColor
                        let gradient = RadialGradient(gradient: Gradient(colors: [settingsModel.backgroundColor, fillColor]), center: .center, startRadius: 0, endRadius: (min(geometry.size.width, geometry.size.height) * animationProgress * maxCircleScale) / 2)
                        Circle()
                            .conditionalFill(settingsModel.colorFillType == .linear, ifTrue: gradient, ifFalse: fillColor)
                            .frame(width: min(geometry.size.width, geometry.size.height) * animationProgress * maxCircleScale, height: min(geometry.size.width, geometry.size.height) * animationProgress * maxCircleScale)
                            .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
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
                    colorFillType: $settingsModel.colorFillType,
                    inhaleDuration: $settingsModel.inhaleDuration,
                    postInhaleHoldDuration: $settingsModel.postInhaleHoldDuration,
                    exhaleDuration: $settingsModel.exhaleDuration,
                    postExhaleHoldDuration: $settingsModel.postExhaleHoldDuration,
                    drift: $settingsModel.drift,
                    overlayOpacity: $overlayOpacity,
                    shape: Binding<AnimationShape>(get: { self.settingsModel.shape }, set: { self.settingsModel.shape = $0 }), animationMode: Binding<AnimationMode>(get: { self.settingsModel.animationMode }, set: { self.settingsModel.animationMode = $0 })
                )
            }
        }
        .onAppear(perform: startBreathingCycle)
    }
    
    func startBreathingCycle() {
        cycleCount = 0
        inhale()
    }
    
    func inhale() {
        var duration = settingsModel.inhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        duration = max(duration, 0.5)

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
        let duration = settingsModel.postInhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        breathingPhase = .holdAfterInhale
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            exhale()
        }
    }

    func exhale() {
        var duration = settingsModel.exhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        duration = max(duration, 0.5)

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
        let duration = settingsModel.postExhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        breathingPhase = .holdAfterExhale
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            cycleCount += 1
            inhale()
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
