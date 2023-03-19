//  SettingsModel.swift
import SwiftUI

class SettingsModel: ObservableObject {
    @Published var overlayColor: Color = Color(
        red: 0.9411764705882353,
        green: 0.7803921568627451,
        blue: 0.6784313725490196
    )
    @Published var inhaleDuration: TimeInterval = 5
    @Published var postInhaleHoldDuration: TimeInterval = 0
    @Published var exhaleDuration: TimeInterval = 10
    @Published var postExhaleHoldDuration: TimeInterval = 0
    @Published var drift: Double = 1.0
    @Published var overlayOpacity: Double = 0.1
}
