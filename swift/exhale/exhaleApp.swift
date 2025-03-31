// exhaleApp.swift
import SwiftUI

@main
struct exhaleApp: App {
    @ObservedObject private var settingsModel = SettingsModel()
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    init() {
        appDelegate.settingsModel = settingsModel
    }

    var body: some Scene {
        WindowGroup {
            EmptyClosingView()
        }
        .commands {
            CommandGroup(replacing: .appSettings) {
                Button("Preferences...") {
                    appDelegate.toggleSettings(nil)
                }
                .environmentObject(settingsModel)
                .keyboardShortcut("w", modifiers: [.control, .shift])
                .keyboardShortcut(",", modifiers: [.control, .shift])
                
                Button(settingsModel.isAnimating ? "Stop" : "Start") {
                    settingsModel.isAnimating.toggle()
                }
                .environmentObject(settingsModel)
                .keyboardShortcut("s", modifiers: [.control, .shift])
                
                Button("Reset to Defaults") {
                    appDelegate.resetToDefaults(nil)
                }
                .keyboardShortcut("f", modifiers: [.control, .shift])
                .help("Reset all settings to their default values.")
                
                Button("Quit exhale") {
                    NSApp.terminate(nil)
                }
                .keyboardShortcut("q", modifiers: .command)
            }
        }
    }
}
