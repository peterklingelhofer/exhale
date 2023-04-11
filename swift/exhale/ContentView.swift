// ContentView.swift
import SwiftUI

extension Color {
    func interpolate(to color: Color, fraction: Double) -> Color {
        let fromComponents = self.cgColor?.components ?? [0, 0, 0, 0]
        let toComponents = color.cgColor?.components ?? [0, 0, 0, 0]
        
        let red = CGFloat(fromComponents[0] + (toComponents[0] - fromComponents[0]) * CGFloat(fraction))
        let green = CGFloat(fromComponents[1] + (toComponents[1] - fromComponents[1]) * CGFloat(fraction))
        let blue = CGFloat(fromComponents[2] + (toComponents[2] - fromComponents[2]) * CGFloat(fraction))
        let alpha = CGFloat(fromComponents[3] + (toComponents[3] - fromComponents[3]) * CGFloat(fraction))
        
        return Color(red: red, green: green, blue: blue, opacity: alpha)
    }
}

extension Shape {
    @ViewBuilder
    func colorTransitionFill(settingsModel: SettingsModel, animationProgress: CGFloat, breathingPhase: BreathingPhase, endRadius: CGFloat = 0) -> some View {
        let isInhalePhase = breathingPhase == .inhale || breathingPhase == .holdAfterInhale
        let lastColor = isInhalePhase ? settingsModel.inhaleColor : settingsModel.exhaleColor
        let nextColor = isInhalePhase ? settingsModel.exhaleColor : settingsModel.inhaleColor
        let startingColor = isInhalePhase ? settingsModel.exhaleColor : settingsModel.inhaleColor
        let transitionFraction = breathingPhase == .exhale ? Double(1 - animationProgress) : Double(animationProgress)
        let finalColor = settingsModel.colorTransitionEnabled ? startingColor.interpolate(to: nextColor, fraction: transitionFraction) : lastColor
        
        if settingsModel.colorFillType != .constant {
            if settingsModel.shape == .rectangle {
                let gradient = LinearGradient(
                    gradient: Gradient(colors: [finalColor, settingsModel.backgroundColor]),
                    startPoint: .top,
                    endPoint: .bottom
                )
                self.fill(gradient)
            } else {
                let gradient = RadialGradient(
                    gradient: Gradient(colors: [settingsModel.backgroundColor, finalColor]),
                    center: .center,
                    startRadius: 0,
                    endRadius: endRadius
                )
                self.fill(gradient)
            }
        } else {
            self.fill(finalColor)
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
                        Rectangle()
                            .colorTransitionFill(settingsModel: settingsModel, animationProgress: animationProgress, breathingPhase: breathingPhase)
                            .frame(height: geometry.size.height)
                            .scaleEffect(y: animationProgress, anchor: .bottom)
                            .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
                    } else if settingsModel.shape == .circle {
                        Circle()
                            .colorTransitionFill(settingsModel: settingsModel, animationProgress: animationProgress, breathingPhase: breathingPhase, endRadius: (min(geometry.size.width, geometry.size.height) * animationProgress * maxCircleScale) / 2)
                            .frame(width: min(geometry.size.width, geometry.size.height) * animationProgress * maxCircleScale, height: min(geometry.size.width, geometry.size.height) * animationProgress * maxCircleScale)
                            .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
                    } else if settingsModel.shape == .fullscreen {
                        Rectangle()
                            .fill(settingsModel.colorTransitionEnabled ? settingsModel.inhaleColor.interpolate(to: settingsModel.exhaleColor, fraction: Double(animationProgress)) : (breathingPhase == .inhale || breathingPhase == .holdAfterInhale) ? settingsModel.inhaleColor : settingsModel.exhaleColor)
                            .edgesIgnoringSafeArea(.all)
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
