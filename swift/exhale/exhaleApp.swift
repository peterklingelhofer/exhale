//  exhaleApp.swift
import SwiftUI

@main
struct exhaleApp: App {
    @StateObject private var settingsModel = SettingsModel()
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        Settings {
            EmptyView()
        }
        .commands {
            CommandGroup(replacing: .appSettings) {
                Button("Preferences...") {
                    appDelegate.showSettings(())
                }.keyboardShortcut(",", modifiers: .command)
            }
        }
    }
}
