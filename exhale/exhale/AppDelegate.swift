// AppDelegate.swift
import Cocoa
import Combine
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    var settingsWindow: NSWindow!
    var settingsModel = SettingsModel()
    var overlayColorSubscription: AnyCancellable?
    var overlayOpacitySubscription: AnyCancellable?
    var subscriptions = Set<AnyCancellable>()

    func applicationDidFinishLaunching(_ notification: Notification) {
        let screenSize = NSScreen.main?.frame.size ?? CGSize(width: 800, height: 600)
        settingsModel = SettingsModel()
        
        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: screenSize.width, height: screenSize.height),
            styleMask: [.borderless, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )

        window.contentView = NSHostingView(rootView: ContentView().environmentObject(settingsModel))
        window.makeKeyAndOrderFront(nil)
        window.level = .floating
        window.alphaValue = CGFloat(settingsModel.overlayOpacity)
        window.isOpaque = false
        window.ignoresMouseEvents = true

        overlayColorSubscription = settingsModel.$overlayColor.sink { newColor in
            self.window.backgroundColor = NSColor(newColor)
        }
        
        overlayOpacitySubscription = settingsModel.$overlayOpacity.sink { newOpacity in
            self.window.alphaValue = CGFloat(newOpacity)
        }
        
        // Reload content view when any setting changes
        settingsModel.objectWillChange.sink {
            self.reloadContentView()
        }.store(in: &subscriptions)

        // Call reloadContentView() after initializing window and previewWindow
        reloadContentView()
        
        settingsWindow = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 300, height: 300),
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )

        settingsWindow.contentView = NSHostingView(rootView: SettingsView(
            showSettings: .constant(false),
            overlayColor: Binding(get: { self.settingsModel.overlayColor }, set: { self.settingsModel.overlayColor = $0 }),
            backgroundColor: Binding(get: { self.settingsModel.backgroundColor }, set: { self.settingsModel.backgroundColor = $0 }),
            inhaleDuration: Binding(get: { self.settingsModel.inhaleDuration }, set: { self.settingsModel.inhaleDuration = $0 }),
            postInhaleHoldDuration: Binding(get: { self.settingsModel.postInhaleHoldDuration }, set: { self.settingsModel.postInhaleHoldDuration = $0 }),
            exhaleDuration: Binding(get: { self.settingsModel.exhaleDuration }, set: { self.settingsModel.exhaleDuration = $0 }),
            postExhaleHoldDuration: Binding(get: { self.settingsModel.postExhaleHoldDuration }, set: { self.settingsModel.postExhaleHoldDuration = $0 }),
            drift: Binding(get: { self.settingsModel.drift }, set: { self.settingsModel.drift = $0 }),
            overlayOpacity: Binding(get: { self.settingsModel.overlayOpacity }, set: { self.settingsModel.overlayOpacity = $0 })
        ).environmentObject(settingsModel))

        settingsWindow.title = "Preferences"
    }

    func applicationWillTerminate(_ notification: Notification) {
        // Insert code here to tear down your application
    }
    
    func showSettings(_ sender: Any?) {
        settingsWindow.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow.level = .floating
    }
    
    func reloadContentView() {
        let contentView = ContentView().environmentObject(settingsModel)
        self.window.contentView = NSHostingView(rootView: contentView)
    }
}
