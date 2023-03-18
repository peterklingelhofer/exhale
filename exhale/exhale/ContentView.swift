// ContentView.swift
import SwiftUI

struct ContentView: View {
    @EnvironmentObject var settingsModel: SettingsModel
    @State private var animationProgress: CGFloat = 0
    @State private var breathingPhase: BreathingPhase = .inhale
    @State private var overlayColor = Color(red: 0.658823529411765, green: 0.196078431372549, blue: 0.588235294117647)
    @State private var backgroundColor = Color.white
    @State private var overlayOpacity: Double = 0.1
    @State private var showSettings = false
    
    var body: some View {
        ZStack {
            GeometryReader { geometry in
                ZStack {
                    backgroundColor.edgesIgnoringSafeArea(.all)
                    Rectangle()
                        .fill(overlayColor)
                        .frame(height: animationProgress * geometry.size.height)
                        .position(x: geometry.size.width / 2, y: geometry.size.height - (animationProgress * geometry.size.height) / 2)
                }
            }
            .edgesIgnoringSafeArea(.all)
            
            VStack {
                HStack {
                    Spacer()
                    Button(action: {
                        showSettings.toggle()
                    }) {
                        Image(systemName: "gearshape")
                            .resizable()
                            .frame(width: 30, height: 30)
                            .foregroundColor(.gray)
                    }
                    .padding()
                }
                Spacer()
            }
            
            if showSettings {
                SettingsView(
                    showSettings: $showSettings,
                    overlayColor: $overlayColor, inhaleDuration: $settingsModel.inhaleDuration,
                    postInhaleHoldDuration: $settingsModel.postInhaleHoldDuration,
                    exhaleDuration: $settingsModel.exhaleDuration,
                    postExhaleHoldDuration: $settingsModel.postExhaleHoldDuration,
                    overlayOpacity: $overlayOpacity
                )
            }
        }
        .onAppear(perform: startBreathingCycle)
    }
    
    func startBreathingCycle() {
        inhale()
    }
    
    func inhale() {
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
