// SettingsModel.swift
import SwiftUI

class SettingsModel: ObservableObject {
    @Published var backgroundColor: Color = Color.black
    @Published var inhaleColor: Color = Color(red: 1, green: 0, blue: 0)
    @Published var exhaleColor: Color = Color(red: 0, green: 0, blue: 1)
    @Published var colorFillType: ColorFillType = .constant
    @Published var inhaleDuration: TimeInterval = 5
    @Published var postInhaleHoldDuration: TimeInterval = 0
    @Published var exhaleDuration: TimeInterval = 10
    @Published var postExhaleHoldDuration: TimeInterval = 0
    @Published var drift: Double = 1.0
    @Published var overlayOpacity: Double = 0.1
    @Published var shape: AnimationShape = .fullscreen
    @Published var animationMode: AnimationMode = .sinusoidal
    @Published var colorTransitionEnabled: Bool = true
}
