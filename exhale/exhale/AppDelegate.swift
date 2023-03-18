// AppDelegate.swift
import Cocoa
import Combine
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    var previewWindow: NSWindow!
    var settingsModel: SettingsModel!
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
        previewWindow = NSWindow(
            contentRect: NSRect(x: screenSize.width / 2 - 300, y: screenSize.height / 2 - 200, width: 600, height: 400),
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )

        window.contentView = NSHostingView(rootView: ContentView().environmentObject(settingsModel))
        window.makeKeyAndOrderFront(nil)
        window.level = .floating
        window.alphaValue = CGFloat(settingsModel.overlayOpacity)
        window.isOpaque = false
        window.ignoresMouseEvents = true

        previewWindow.contentView = NSHostingView(rootView: ContentView().environmentObject(settingsModel))
        previewWindow.makeKeyAndOrderFront(nil)
        
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
    }

    func applicationWillTerminate(_ notification: Notification) {
        // Insert code here to tear down your application
    }
    
    func reloadContentView() {
        let contentView = ContentView().environmentObject(settingsModel)
        self.window.contentView = NSHostingView(rootView: contentView)
        self.previewWindow.contentView = NSHostingView(rootView: contentView)
    }
}
