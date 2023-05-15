// AppDelegate.swift
import Cocoa
import Combine
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    var windows: [NSWindow] = []
    var settingsWindow: NSWindow!
    var settingsModel = SettingsModel()
    var inhaleColorSubscription: AnyCancellable?
    var exhaleColorSubscription: AnyCancellable?
    var overlayOpacitySubscription: AnyCancellable?
    var subscriptions = Set<AnyCancellable>()
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        settingsModel = SettingsModel()
        
        for screen in NSScreen.screens {
            let screenSize = screen.frame.size
            let window = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: screenSize.width, height: screenSize.height),
                styleMask: [.borderless, .fullSizeContentView],
                backing: .buffered,
                defer: false
            )
            
            window.contentView = NSHostingView(rootView: ContentView().environmentObject(settingsModel))
            window.makeKeyAndOrderFront(nil)
            window.level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.mainMenuWindow)) + 1) // Window level in front of the menu bar
            window.alphaValue = CGFloat(settingsModel.overlayOpacity)
            window.isOpaque = false
            window.ignoresMouseEvents = true
            window.setFrame(screen.frame, display: true)
            
            windows.append(window)
        }

        inhaleColorSubscription = settingsModel.$inhaleColor.sink { [unowned self] newColor in
            for window in self.windows {
                window.backgroundColor = NSColor(newColor)
            }
        }
        
        exhaleColorSubscription = settingsModel.$exhaleColor.sink { [unowned self] newColor in
            for window in self.windows {
                window.backgroundColor = NSColor(newColor)
            }
        }
        
        overlayOpacitySubscription = settingsModel.$overlayOpacity.sink { [unowned self] newOpacity in
            for window in self.windows {
                window.alphaValue = CGFloat(newOpacity)
            }
        }
        
        // Reload content view when any setting changes
        settingsModel.objectWillChange.sink { [unowned self] in
            self.reloadContentView()
        }.store(in: &subscriptions)
        
        reloadContentView()
        
        settingsWindow = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 600, height: 200),
            styleMask: [.titled, .closable, .miniaturizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        
        settingsWindow.delegate = self
        settingsWindow.contentView = NSHostingView(rootView: SettingsView(
            showSettings: .constant(false),
            inhaleColor: Binding(get: { self.settingsModel.inhaleColor }, set: { self.settingsModel.inhaleColor = $0 }),
            exhaleColor: Binding(get: { self.settingsModel.exhaleColor }, set: { self.settingsModel.exhaleColor = $0 }),
            backgroundColor: Binding(get: { self.settingsModel.backgroundColor }, set: { self.settingsModel.backgroundColor = $0 }),
            colorFillType: Binding(get: { self.settingsModel.colorFillType }, set: { self.settingsModel.colorFillType = $0 }),
            inhaleDuration: Binding(get: { self.settingsModel.inhaleDuration }, set: { self.settingsModel.inhaleDuration = $0 }),
            postInhaleHoldDuration: Binding(get: { self.settingsModel.postInhaleHoldDuration }, set: { self.settingsModel.postInhaleHoldDuration = $0 }),
            exhaleDuration: Binding(get: { self.settingsModel.exhaleDuration }, set: { self.settingsModel.exhaleDuration = $0 }),
            postExhaleHoldDuration: Binding(get: { self.settingsModel.postExhaleHoldDuration }, set: { self.settingsModel.postExhaleHoldDuration = $0 }),
            drift: Binding(get: { self.settingsModel.drift }, set: { self.settingsModel.drift = $0 }),
            overlayOpacity: Binding(get: { self.settingsModel.overlayOpacity }, set: { self.settingsModel.overlayOpacity = $0 }),
            shape: Binding<AnimationShape>(get: { self.settingsModel.shape }, set: { self.settingsModel.shape = $0 }),
            animationMode: Binding<AnimationMode>(get: { self.settingsModel.animationMode }, set: { self.settingsModel.animationMode = $0 }),
            randomizedTimingInhale: Binding(get: { self.settingsModel.randomizedTimingInhale }, set: { self.settingsModel.randomizedTimingInhale = $0 }),
            randomizedTimingPostInhaleHold: Binding(get: { self.settingsModel.randomizedTimingPostInhaleHold }, set: { self.settingsModel.randomizedTimingPostInhaleHold = $0 }),
            randomizedTimingExhale: Binding(get: { self.settingsModel.randomizedTimingExhale }, set: { self.settingsModel.randomizedTimingExhale = $0 }),
            randomizedTimingPostExhaleHold: Binding(get: { self.settingsModel.randomizedTimingPostExhaleHold }, set: { self.settingsModel.randomizedTimingPostExhaleHold = $0 })
        ).environmentObject(settingsModel))
        
        settingsWindow.title = "exhale"
        showSettings(nil)
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
        for window in windows {
            window.contentView = NSHostingView(rootView: contentView)
        }
    }
    
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        if sender == settingsWindow {
            settingsWindow.orderOut(sender)
            return false
        }
        return true
    }
}
