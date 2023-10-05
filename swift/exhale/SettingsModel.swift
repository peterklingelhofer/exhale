// SettingsModel.swift
import SwiftUI

class SettingsModel: ObservableObject {
    @Published var backgroundColor: Color = Color.black
    @Published var inhaleColor: Color = Color(red: 1, green: 0, blue: 0)
    @Published var exhaleColor: Color = Color(red: 0, green: 0, blue: 1)
    @Published var colorFillGradient: ColorFillGradient = .off
    @Published var inhaleDuration: TimeInterval = 5
    @Published var postInhaleHoldDuration: TimeInterval = 0
    @Published var exhaleDuration: TimeInterval = 10
    @Published var postExhaleHoldDuration: TimeInterval = 0
    @Published var drift: Double = 1.01
    @Published var overlayOpacity: Double = 0.25
    @Published var shape: AnimationShape = .fullscreen
    @Published var animationMode: AnimationMode = .sinusoidal
    @Published var randomizedTimingInhale: Double = 0
    @Published var randomizedTimingPostInhaleHold: Double = 0
    @Published var randomizedTimingExhale: Double = 0
    @Published var randomizedTimingPostExhaleHold: Double = 0
}
