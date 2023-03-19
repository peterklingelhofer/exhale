// ContentView.swift
import SwiftUI

struct ContentView: View {
    @EnvironmentObject var settingsModel: SettingsModel
    @State private var animationProgress: CGFloat = 0
    @State private var breathingPhase: BreathingPhase = .inhale
    @State private var overlayOpacity: Double = 0.1
    @State private var showSettings = false
    @State private var cycleCount: Int = 0
    
    var body: some View {
        ZStack {
            GeometryReader { geometry in
                ZStack {
                    settingsModel.backgroundColor.edgesIgnoringSafeArea(.all)
                    Rectangle()
                        .fill(settingsModel.overlayColor)
                        .frame(height: animationProgress * geometry.size.height)
                        .position(x: geometry.size.width / 2, y: geometry.size.height - (animationProgress * geometry.size.height) / 2)
                }
            }
            .edgesIgnoringSafeArea(.all)
            
            if showSettings {
                SettingsView(
                    showSettings: $showSettings,
                    overlayColor: $settingsModel.overlayColor,
                    backgroundColor: $settingsModel.backgroundColor,
                    inhaleDuration: $settingsModel.inhaleDuration,
                    postInhaleHoldDuration: $settingsModel.postInhaleHoldDuration,
                    exhaleDuration: $settingsModel.exhaleDuration,
                    postExhaleHoldDuration: $settingsModel.postExhaleHoldDuration,
                    drift: $settingsModel.drift,
                    overlayOpacity: $overlayOpacity
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
        
        withAnimation(.linear(duration: duration)) {
            breathingPhase = .inhale
            animationProgress = 1.0
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
        
        withAnimation(.linear(duration: duration)) {
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
