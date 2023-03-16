// exhaleApp.swift
import SwiftUI

@main
struct exhaleApp: App {
    @StateObject private var settingsModel = SettingsModel()
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    init() {
        appDelegate.settingsModel = settingsModel
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(settingsModel)
        }
    }
}
