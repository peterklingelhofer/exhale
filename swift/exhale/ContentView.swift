// ContentView.swift
import SwiftUI

struct ContentView: View {
    @EnvironmentObject var settingsModel: SettingsModel
    @State private var animationProgress: CGFloat = 0
    @State private var breathingPhase: BreathingPhase = .inhale
    @State private var backgroundColor = Color.black
    @State private var overlayOpacity: Double = 0.1
    @State private var showSettings = false
    
    var body: some View {
        ZStack {
            GeometryReader { geometry in
                ZStack {
                    backgroundColor.edgesIgnoringSafeArea(.all)
                    Rectangle()
                        .fill(settingsModel.overlayColor) // Use settingsModel.overlayColor instead of overlayColor
                        .frame(height: animationProgress * geometry.size.height)
                        .position(x: geometry.size.width / 2, y: geometry.size.height - (animationProgress * geometry.size.height) / 2)
                }
            }
            .edgesIgnoringSafeArea(.all)

            if showSettings {
                SettingsView(
                    showSettings: $showSettings,
                    overlayColor: $settingsModel.overlayColor,
                    inhaleDuration: $settingsModel.inhaleDuration,
                    postInhaleHoldDuration: $settingsModel.postInhaleHoldDuration,
                    exhaleDuration: $settingsModel.exhaleDuration,
                    postExhaleHoldDuration: $settingsModel.postExhaleHoldDuration,
                    overlayOpacity: $overlayOpacity,
                    drift: $settingsModel.drift
                )
            }
        }
        .onAppear(perform: startBreathingCycle)
    }
    
    func startBreathingCycle() {
        inhale()
    }
    
    func inhale() {
        settingsModel.inhaleDuration += settingsModel.drift
        withAnimation(.linear(duration: settingsModel.inhaleDuration)) {
            breathingPhase = .inhale
            animationProgress = 1.0
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + settingsModel.inhaleDuration) {
            holdAfterInhale()
        }
    }
    
    func holdAfterInhale() {
        breathingPhase = .holdAfterInhale
        DispatchQueue.main.asyncAfter(deadline: .now() + settingsModel.postInhaleHoldDuration) {
            exhale()
        }
    }
    
    func exhale() {
        settingsModel.exhaleDuration += settingsModel.drift
        withAnimation(.linear(duration: settingsModel.exhaleDuration)) {
            breathingPhase = .exhale
            animationProgress = 0.0
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + settingsModel.exhaleDuration) {
            holdAfterExhale()
        }
    }
    
    func holdAfterExhale() {
        breathingPhase = .holdAfterExhale
        DispatchQueue.main.asyncAfter(deadline: .now() + settingsModel.postExhaleHoldDuration) {
            inhale()
        }
    }
}


enum BreathingPhase {
    case inhale, holdAfterInhale, exhale, holdAfterExhale
    
    func duration(settingsModel: SettingsModel) -> TimeInterval {
        switch self {
        case .inhale:
            return settingsModel.inhaleDuration
        case .holdAfterInhale:
            return settingsModel.postInhaleHoldDuration
        case .exhale:
            return settingsModel.exhaleDuration
        case .holdAfterExhale:
            return settingsModel.postExhaleHoldDuration
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
