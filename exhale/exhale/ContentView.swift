// ContentView.swift
import SwiftUI

struct ContentView: View {
    @State private var animationProgress: CGFloat = 0
    @State private var breathingPhase: BreathingPhase = .inhale

    let timer = Timer.publish(every: 0.1, on: .main, in: .common).autoconnect()

    var body: some View {
        GeometryReader { geometry in
            Rectangle()
                .fill(Color(red: 0.658823529411765, green: 0.196078431372549, blue: 0.588235294117647))
                .frame(height: animationProgress * geometry.size.height)
                .position(x: geometry.size.width / 2, y: geometry.size.height - (animationProgress * geometry.size.height) / 2)
                .onReceive(timer) { _ in
                    updateAnimation()
                }
        }
        .edgesIgnoringSafeArea(.all)
    }

    func updateAnimation() {
        let increment = CGFloat(0.1 / breathingPhase.duration)

        switch breathingPhase {
        case .inhale:
            animationProgress += increment
            if animationProgress >= 1.0 {
                breathingPhase = .holdAfterInhale
                animationProgress = 1.0
            }
        case .holdAfterInhale:
            // Implement a hold after inhale if needed
            breathingPhase = .exhale
        case .exhale:
            animationProgress -= increment
            if animationProgress <= 0.0 {
                breathingPhase = .holdAfterExhale
                animationProgress = 0.0
            }
        case .holdAfterExhale:
            // Implement a hold after exhale if needed
            breathingPhase = .inhale
        }
    }
}

enum BreathingPhase {
    case inhale, holdAfterInhale, exhale, holdAfterExhale

    var duration: TimeInterval {
        switch self {
        case .inhale:
            return 5
        case .exhale:
            return 10
        default:
            return 0
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
