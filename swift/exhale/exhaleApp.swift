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
        Settings {
            EmptyClosingView()
        }
        .commands {
            CommandGroup(replacing: .appSettings) {
                Button("Preferences...") {
                    appDelegate.toggleSettings(nil)
                }
                .environmentObject(settingsModel)
                .keyboardShortcut(",", modifiers: .command)
                
                Button(settingsModel.isAnimating ? "Stop" : "Start") {
                    settingsModel.isAnimating.toggle()
                }
                .environmentObject(settingsModel)
                .keyboardShortcut("s", modifiers: .command)
                
                Button("Reset to Defaults") {
                    settingsModel.resetToDefaults()
                }
                .keyboardShortcut("r", modifiers: .command)
                .help("Reset all settings to their default values.")
                
                Button("Quit exhale") {
                    NSApp.terminate(nil)
                }
                .keyboardShortcut("q", modifiers: .command)
            }
        }
    }
}
