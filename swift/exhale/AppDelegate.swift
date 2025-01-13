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
    var isAnimatingSubscription: AnyCancellable?
    var subscriptions = Set<AnyCancellable>()
    var statusItem: NSStatusItem!

    func setUpStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            button.image = NSImage(named: "StatusBarIcon")
            button.action = #selector(statusBarButtonClicked(sender:))
        }

        let menu = NSMenu()
        
        // Preferences
        menu.addItem(NSMenuItem(title: "Preferences...", action: #selector(toggleSettings(_:)), keyEquivalent: ","))
        
        // Start/Stop
        let startMenuItem = NSMenuItem(title: "Start", action: #selector(startAnimating(_:)), keyEquivalent: "s")
        let stopMenuItem = NSMenuItem(title: "Stop", action: #selector(stopAnimating(_:)), keyEquivalent: "x")
        menu.addItem(startMenuItem)
        menu.addItem(stopMenuItem)
        
        // Tint
        let tintMenuItem = NSMenuItem(title: "Tint", action: #selector(pauseAnimating(_:)), keyEquivalent: "p")
        menu.addItem(tintMenuItem)
        
        // Reset to Defaults
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Reset to Defaults", action: #selector(resetToDefaults(_:)), keyEquivalent: "r"))
        
        // Quit
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Quit exhale", action: #selector(terminateApp(_:)), keyEquivalent: "q"))

        // Bind menu items to model state
        settingsModel.$isAnimating
            .sink { [weak self] isAnimating in
                guard let self = self else { return }
                startMenuItem.title = isAnimating ? "Start" : "Start"
                stopMenuItem.title = isAnimating ? "Stop" : "Stop"
                tintMenuItem.title = isAnimating ? "Tint" : "Tint"
                startMenuItem.isEnabled = !isAnimating
                stopMenuItem.isEnabled = isAnimating || self.settingsModel.isPaused
                tintMenuItem.isEnabled = !isAnimating && !self.settingsModel.isPaused
            }
            .store(in: &subscriptions)

        settingsModel.$isPaused
            .sink { [weak self] isPaused in
                guard let self = self else { return }
                tintMenuItem.title = isPaused ? "Unpause" : "Tint"
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

    @objc func pauseAnimating(_ sender: Any?) {
        if settingsModel.isPaused {
            settingsModel.unpause()
        } else {
            settingsModel.pause()
        }
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
            // window.collectionBehavior = [.canJoinAllSpaces]  // Ensures window appears in all spaces

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

        // Initialize the Settings Window
        settingsWindow = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 600, height: 600),
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
    }

    func applicationWillTerminate(_ notification: Notification) {
        // Insert code here to tear down your application
    }

    @objc func toggleSettings(_ sender: Any?) {
        if settingsWindow.isVisible {
            settingsWindow.orderOut(sender)
        } else {
            settingsWindow.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            settingsWindow.level = .floating
        }
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
