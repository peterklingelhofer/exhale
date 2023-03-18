// exhaleApp.swift
import SwiftUI

@main
struct exhaleApp: App {
    @StateObject private var settingsModel = SettingsModel()
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    init() {
        appDelegate.settingsModel = settingsModel
    }
    
    // Remove the WindowGroup and replace it with an empty body
    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}
