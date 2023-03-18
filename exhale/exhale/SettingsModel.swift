//  SettingsModel.swift
import SwiftUI

class SettingsModel: ObservableObject {
    @Published var overlayColor: Color = Color(
        red: 0.658823529411765,
        green: 0.196078431372549,
        blue: 0.588235294117647
    )
    @Published var inhaleDuration: TimeInterval = 5
    @Published var postInhaleHoldDuration: TimeInterval = 0
    @Published var exhaleDuration: TimeInterval = 10
    @Published var postExhaleHoldDuration: TimeInterval = 0
    @Published var overlayOpacity: Double = 0.1
}
