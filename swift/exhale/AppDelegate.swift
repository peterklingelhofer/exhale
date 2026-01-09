// AppDelegate.swift
import Cocoa
import Combine
import SwiftUI
import HotKey

class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    var windows: [NSWindow] = []
    var settingsWindow: NSWindow!
    var settingsModel = SettingsModel()
    var inhaleColorSubscription: AnyCancellable?
    var exhaleColorSubscription: AnyCancellable?
    var overlayOpacitySubscription: AnyCancellable?
    var isAnimatingSubscription: AnyCancellable?
    var subscriptions = Set<AnyCancellable>()
    var statusItem: NSStatusItem!
    var startHotKey: HotKey?
    var stopHotKey: HotKey?
    var tintHotKey: HotKey?
    var resetHotKey: HotKey?
    var preferencesHotKey: HotKey?
    
    func setUpStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            button.image = NSImage(named: "StatusBarIcon") // Ensure you have an image named "StatusBarIcon" in Assets
            button.action = #selector(statusBarButtonClicked(sender:))
        }
        
        let menu = NSMenu()
        
        // Preferences
        let preferencesItem = NSMenuItem(title: "Preferences", action: #selector(toggleSettings(_:)), keyEquivalent: "w")
        preferencesItem.keyEquivalentModifierMask = [.control, .shift]
        preferencesItem.target = self
        menu.addItem(preferencesItem)
        
        // Start Animation
        let startMenuItem = NSMenuItem(title: "Start Animation", action: #selector(startAnimating(_:)), keyEquivalent: "a")
        startMenuItem.keyEquivalentModifierMask = [.control, .shift]
        startMenuItem.target = self
        menu.addItem(startMenuItem)
        
        // Stop Animation
        let stopMenuItem = NSMenuItem(title: "Stop Animation", action: #selector(stopAnimating(_:)), keyEquivalent: "s")
        stopMenuItem.keyEquivalentModifierMask = [.control, .shift]
        stopMenuItem.target = self
        menu.addItem(stopMenuItem)
        
        // Tint Screen
        let tintMenuItem = NSMenuItem(title: "Tint Screen", action: #selector(tintScreen(_:)), keyEquivalent: "d")
        tintMenuItem.keyEquivalentModifierMask = [.control, .shift]
        tintMenuItem.target = self
        menu.addItem(tintMenuItem)
        
        // Reset to Defaults
        let resetMenuItem = NSMenuItem(title: "Reset to Defaults", action: #selector(resetToDefaults(_:)), keyEquivalent: "f")
        resetMenuItem.keyEquivalentModifierMask = [.control, .shift]
        resetMenuItem.target = self
        menu.addItem(resetMenuItem)
        
        // Separator
        menu.addItem(NSMenuItem.separator())
        
        // Quit
        let quitMenuItem = NSMenuItem(title: "Quit exhale", action: #selector(terminateApp(_:)), keyEquivalent: "q")
        quitMenuItem.keyEquivalentModifierMask = [.command]
        quitMenuItem.target = self
        menu.addItem(quitMenuItem)
        
        // Bind menu items to model state
        settingsModel.$isAnimating
            .sink { [weak self] isAnimating in
                guard let self = self else { return }
                startMenuItem.title = "Start Animation"
                stopMenuItem.title = "Stop Animation"
                tintMenuItem.title = "Tint Screen"
                resetMenuItem.title = "Reset to Defaults"
                startMenuItem.isEnabled = !isAnimating
                stopMenuItem.isEnabled = isAnimating || self.settingsModel.isPaused
                tintMenuItem.isEnabled = !isAnimating && !self.settingsModel.isPaused
            }
            .store(in: &subscriptions)
        
        settingsModel.$isPaused
            .sink { [weak self] isPaused in
                guard let self = self else { return }
                tintMenuItem.title = isPaused ? "Unpause" : "Tint Screen"
                tintMenuItem.isEnabled = !self.settingsModel.isAnimating && !isPaused
            }
            .store(in: &subscriptions)
        
        statusItem.menu = menu
    }
    
    @objc func startAnimating(_ sender: Any?) {
        settingsModel.start()
    }
    
    @objc func stopAnimating(_ sender: Any?) {
        settingsModel.stop()
    }
    
    @objc func tintScreen(_ sender: Any?) {
        settingsModel.pause()
    }
    
    @objc func statusBarButtonClicked(sender: NSStatusBarButton) {
        statusItem.menu?.popUp(positioning: nil, at: NSEvent.mouseLocation, in: nil)
    }
    
    @objc func terminateApp(_ sender: Any?) {
        NSApp.terminate(nil)
    }
    
    @objc func resetToDefaults(_ sender: Any?) {
        settingsModel.resetToDefaults()
    }
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        settingsModel = SettingsModel()

        // Create overlay windows for each screen.
        for screen in NSScreen.screens {
            let screenSize = screen.frame.size
            let window = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: screenSize.width, height: screenSize.height),
                styleMask: [.borderless, .fullSizeContentView],
                backing: .buffered,
                defer: false
            )

            // Create the hosting view once. SwiftUI will update it via EnvironmentObject.
            let rootView = ContentView().environmentObject(settingsModel)
            window.contentView = NSHostingView(rootView: rootView)

            window.makeKeyAndOrderFront(nil)
            window.level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.mainMenuWindow)) + 1)
            window.alphaValue = CGFloat(settingsModel.overlayOpacity)
            window.isOpaque = false
            window.hasShadow = false
            window.ignoresMouseEvents = true
            window.setFrame(screen.frame, display: true)

            windows.append(window)
        }

        // Subscriptions to update window colors and opacity
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

        overlayOpacitySubscription = settingsModel.$overlayOpacity.sink { [unowned self] _ in
            for window in self.windows {
                window.alphaValue = 1.0
                window.isOpaque = false
                window.backgroundColor = .clear
            }
        }

        // Initialize the Settings Window
        settingsWindow = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 600, height: 600),
            styleMask: [.titled, .closable, .miniaturizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        settingsWindow.setValue("SettingsWindow", forKey: "frameAutosaveName")
        settingsWindow.delegate = self

        settingsWindow.contentView = NSHostingView(rootView: SettingsView(
            showSettings: .constant(false),
            inhaleColor: Binding(get: { self.settingsModel.inhaleColor }, set: { self.settingsModel.inhaleColor = $0 }),
            exhaleColor: Binding(get: { self.settingsModel.exhaleColor }, set: { self.settingsModel.exhaleColor = $0 }),
            backgroundColor: Binding(get: { self.settingsModel.backgroundColor }, set: { self.settingsModel.backgroundColor = $0 }),
            colorFillType: Binding(get: { self.settingsModel.colorFillGradient }, set: { self.settingsModel.colorFillGradient = $0 }),
            inhaleDuration: Binding(get: { self.settingsModel.inhaleDuration }, set: { self.settingsModel.inhaleDuration = $0 }),
            postInhaleHoldDuration: Binding(get: { self.settingsModel.postInhaleHoldDuration }, set: { self.settingsModel.postInhaleHoldDuration = $0 }),
            exhaleDuration: Binding(get: { self.settingsModel.exhaleDuration }, set: { self.settingsModel.exhaleDuration = $0 }),
            postExhaleHoldDuration: Binding(get: { self.settingsModel.postExhaleHoldDuration }, set: { self.settingsModel.postExhaleHoldDuration = $0 }),
            drift: Binding(get: { self.settingsModel.drift }, set: { self.settingsModel.drift = $0 }),
            overlayOpacity: Binding(get: { self.settingsModel.overlayOpacity }, set: { self.settingsModel.overlayOpacity = $0 }),
            shape: Binding<AnimationShape>(
                get: { self.settingsModel.shape },
                set: { self.settingsModel.shape = $0 }
            ),
            animationMode: Binding<AnimationMode>(
                get: { self.settingsModel.animationMode },
                set: { self.settingsModel.animationMode = $0 }
            ),
            randomizedTimingInhale: Binding(get: { self.settingsModel.randomizedTimingInhale }, set: { self.settingsModel.randomizedTimingInhale = $0 }),
            randomizedTimingPostInhaleHold: Binding(get: { self.settingsModel.randomizedTimingPostInhaleHold }, set: { self.settingsModel.randomizedTimingPostInhaleHold = $0 }),
            randomizedTimingExhale: Binding(get: { self.settingsModel.randomizedTimingExhale }, set: { self.settingsModel.randomizedTimingExhale = $0 }),
            randomizedTimingPostExhaleHold: Binding(get: { self.settingsModel.randomizedTimingPostExhaleHold }, set: { self.settingsModel.randomizedTimingPostExhaleHold = $0 }),
            isAnimating: Binding(get: { self.settingsModel.isAnimating }, set: { self.settingsModel.isAnimating = $0 })
        ).environmentObject(settingsModel))

        settingsWindow.title = "exhale"
        toggleSettings(nil)
        setUpStatusItem()

        isAnimatingSubscription = settingsModel.$isAnimating.sink { [unowned self] isAnimating in
            if !isAnimating && !self.settingsModel.isPaused {
                for window in self.windows {
                    window.backgroundColor = NSColor.clear
                }
            }
        }

        setUpGlobalHotKeys()
    }

    func setUpGlobalHotKeys() {
        // Start Animation: Ctrl + Shift + A
        startHotKey = HotKey(key: .a, modifiers: [.control, .shift])
        startHotKey?.keyDownHandler = { [weak self] in
            self?.settingsModel.start()
        }
        
        // Stop Animation: Ctrl + Shift + S
        stopHotKey = HotKey(key: .s, modifiers: [.control, .shift])
        stopHotKey?.keyDownHandler = { [weak self] in
            self?.settingsModel.stop()
        }
        
        // Tint Screen: Ctrl + Shift + D
        tintHotKey = HotKey(key: .d, modifiers: [.control, .shift])
        tintHotKey?.keyDownHandler = { [weak self] in
            self?.settingsModel.pause()
        }
        
        // Reset to Defaults: Ctrl + Shift + F
        resetHotKey = HotKey(key: .f, modifiers: [.control, .shift])
        resetHotKey?.keyDownHandler = { [weak self] in
            self?.showResetConfirmation()
        }
        
        // Preferences: Ctrl + Shift + ,
        preferencesHotKey = HotKey(key: .comma, modifiers: [.control, .shift])
        preferencesHotKey?.keyDownHandler = { [weak self] in
            self?.toggleSettings(nil)
        }
    }
    
    func showResetConfirmation() {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            let alert = NSAlert()
            alert.messageText = "Reset to Defaults"
            alert.informativeText = "Are you sure you want to reset all settings to their default values? This action cannot be undone."
            alert.alertStyle = .warning
            alert.addButton(withTitle: "Reset")
            alert.addButton(withTitle: "Cancel")
            
            if alert.runModal() == .alertFirstButtonReturn {
                // User clicked "Reset"
                self.settingsModel.resetToDefaults()
            }
            // If "Cancel" is clicked, do nothing
        }
    }
    
    func applicationWillTerminate(_ notification: Notification) {
        // Insert code here to tear down your application
    }
    
    @objc func toggleSettings(_ sender: Any?) {
        if settingsWindow.isVisible {
            settingsWindow.orderOut(sender)
        } else {
            if let frameDict    = UserDefaults.standard.dictionary(forKey: "SettingsWindowFrame"),
               let x            = frameDict["x"]      as? CGFloat,
               let y            = frameDict["y"]      as? CGFloat,
               let w            = frameDict["width"]  as? CGFloat,
               let h            = frameDict["height"] as? CGFloat,
               let screenName   = frameDict["screen"] as? String,
               let matchingScreen = NSScreen.screens.first(where: { $0.localizedName == screenName })
            {
                // Offset saved x/y by the screen's origin
                let screenOrigin = matchingScreen.frame.origin
                let restoredFrame = NSRect(
                    x: screenOrigin.x + x,
                    y: screenOrigin.y + y,
                    width: w,
                    height: h
                )
                settingsWindow.setFrame(restoredFrame, display: true)
            }

            settingsWindow.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            settingsWindow.level = .floating
        }
    }
    
    func reloadContentView() {
        // Intentionally left blank.
        // NSHostingView is created once per window; SwiftUI updates via EnvironmentObject.
    }
    
    // Prevent the settings window from closing (just hide it instead)
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        if sender == settingsWindow {
            settingsWindow.orderOut(sender)
            return false
        }
        return true
    }
    
    // MARK: - NSWindowDelegate for Saving Window Frame
    
    func windowDidMove(_ notification: Notification) {
        guard let window = notification.object as? NSWindow,
              window == settingsWindow,
              let screen = window.screen else { return }
        let frame = window.frame
        let frameDict: [String: Any] = [
            "x": frame.origin.x,
            "y": frame.origin.y,
            "width": frame.size.width,
            "height": frame.size.height,
            "screen": screen.localizedName
        ]
        UserDefaults.standard.set(frameDict, forKey: "SettingsWindowFrame")
    }
}
