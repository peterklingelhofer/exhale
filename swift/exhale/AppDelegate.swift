// AppDelegate.swift
import Cocoa
import Combine
import SwiftUI
import HotKey
import UserNotifications

class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    static let overlayWindowLevel = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.screenSaverWindow)))
    static let settingsWindowLevel = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.screenSaverWindow)) + 1)
    static let tooltipWindowLevel = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.screenSaverWindow)) + 2)

    var windows: [NSWindow] = []
    var settingsWindow: NSWindow!
    var settingsModel = SettingsModel()
    var tooltipCheckTimer: Timer?
    var inhaleColorSubscription: AnyCancellable?
    var exhaleColorSubscription: AnyCancellable?
    var overlayOpacitySubscription: AnyCancellable?
    var isAnimatingSubscription: AnyCancellable?
    var subscriptions = Set<AnyCancellable>()
    var statusItem: NSStatusItem?
    var startHotKey: HotKey?
    var stopHotKey: HotKey?
    var resetHotKey: HotKey?
    var preferencesHotKey: HotKey?
    var reminderTimer: Timer?
    var autoStopTimer: Timer?
    
    func setUpStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem?.button {
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
                resetMenuItem.title = "Reset to Defaults"
                startMenuItem.isEnabled = !isAnimating
                stopMenuItem.isEnabled = isAnimating || self.settingsModel.isPaused
            }
            .store(in: &subscriptions)
        
        statusItem?.menu = menu
    }
    
    @objc func startAnimating(_ sender: Any?) {
        settingsModel.start()
    }
    
    @objc func stopAnimating(_ sender: Any?) {
        settingsModel.stop()
    }
    
    @objc func statusBarButtonClicked(sender: NSStatusBarButton) {
        statusItem?.menu?.popUp(positioning: nil, at: NSEvent.mouseLocation, in: nil)
    }
    
    @objc func terminateApp(_ sender: Any?) {
        NSApp.terminate(nil)
    }
    
    @objc func resetToDefaults(_ sender: Any?) {
        settingsModel.resetToDefaults()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Single-instance enforcement: if another instance is already running,
        // ask it to show settings and terminate this one.
        let bundleID = Bundle.main.bundleIdentifier ?? "com.peterklingelhofer.exhale"
        let runningInstances = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
        if runningInstances.count > 1 {
            DistributedNotificationCenter.default().post(
                name: .init("exhale.showSettings"), object: nil
            )
            // Small delay to ensure notification is delivered before we exit
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                NSApp.terminate(nil)
            }
            return
        }

        // Listen for "show settings" from duplicate launch attempts
        DistributedNotificationCenter.default().addObserver(
            self,
            selector: #selector(showSettingsFromNotification(_:)),
            name: .init("exhale.showSettings"),
            object: nil
        )

        // Tooltip timer is started/stopped with the settings window visibility

        settingsModel = SettingsModel()
        applyAppVisibility(settingsModel.appVisibility)

        for screen in NSScreen.screens {
            let screenSize = screen.frame.size
            let window = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: screenSize.width, height: screenSize.height),
                styleMask: [.borderless, .fullSizeContentView],
                backing: .buffered,
                defer: false,
                screen: screen
            )

            window.contentView = NSHostingView(rootView: ContentView().environmentObject(settingsModel))
            window.setFrame(screen.frame, display: true)

            window.isOpaque = false
            window.backgroundColor = .clear
            window.alphaValue = 1.0
            window.ignoresMouseEvents = true
            window.isReleasedWhenClosed = false

            // Make the overlay participate in fullscreen spaces
            window.collectionBehavior = [
                .canJoinAllSpaces,
                .fullScreenAuxiliary,
                .ignoresCycle
            ]

            // Ensure overlay can appear above fullscreen content
            window.level = Self.overlayWindowLevel

            window.makeKeyAndOrderFront(nil)

            windows.append(window)
        }

        // Subscriptions to update window colors and opacity
        overlayOpacitySubscription = settingsModel.$overlayOpacity.sink { [unowned self] _ in
            for window in self.windows {
                window.alphaValue = 1.0
                window.isOpaque = false
                window.backgroundColor = .clear
            }
        }

        // Initialize the Settings Window
        settingsWindow = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 246, height: 870),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        settingsWindow.setValue("SettingsWindow3", forKey: "frameAutosaveName")
        settingsWindow.delegate = self
        settingsWindow.minSize = NSSize(width: 246, height: 300)
        settingsWindow.maxSize = NSSize(width: 246, height: 870)
        let visualEffect = NSVisualEffectView()
        visualEffect.material = .hudWindow
        visualEffect.blendingMode = .behindWindow
        visualEffect.state = .active
        settingsWindow.contentView = visualEffect

        let hostingView = NSHostingView(rootView: SettingsView(
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
            isAnimating: Binding(get: { self.settingsModel.isAnimating }, set: { self.settingsModel.isAnimating = $0 }),
            appVisibility: Binding(get: { self.settingsModel.appVisibility }, set: { self.settingsModel.appVisibility = $0 }),
            reminderIntervalMinutes: Binding(get: { self.settingsModel.reminderIntervalMinutes }, set: { self.settingsModel.reminderIntervalMinutes = $0 }),
            autoStopMinutes: Binding(get: { self.settingsModel.autoStopMinutes }, set: { self.settingsModel.autoStopMinutes = $0 })
        ).environmentObject(settingsModel))
        hostingView.translatesAutoresizingMaskIntoConstraints = false
        visualEffect.addSubview(hostingView)
        NSLayoutConstraint.activate([
            hostingView.topAnchor.constraint(equalTo: visualEffect.topAnchor),
            hostingView.bottomAnchor.constraint(equalTo: visualEffect.bottomAnchor),
            hostingView.leadingAnchor.constraint(equalTo: visualEffect.leadingAnchor),
            hostingView.trailingAnchor.constraint(equalTo: visualEffect.trailingAnchor),
        ])

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

        // React to app visibility changes
        settingsModel.$appVisibility
            .dropFirst()
            .receive(on: RunLoop.main)
            .sink { [weak self] visibility in
                self?.applyAppVisibility(visibility)
            }
            .store(in: &subscriptions)

        // Reminder notification timer
        if settingsModel.reminderIntervalMinutes > 0 {
            requestNotificationPermission()
        }
        settingsModel.$reminderIntervalMinutes
            .dropFirst()
            .receive(on: RunLoop.main)
            .sink { [weak self] minutes in
                if minutes > 0 { self?.requestNotificationPermission() }
                self?.rescheduleReminderTimer()
            }
            .store(in: &subscriptions)
        rescheduleReminderTimer()

        // Auto-stop timer: restart when interval changes or animation starts/stops
        settingsModel.$autoStopMinutes
            .dropFirst()
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in self?.rescheduleAutoStopTimer() }
            .store(in: &subscriptions)

        settingsModel.$isAnimating
            .dropFirst()
            .receive(on: RunLoop.main)
            .sink { [weak self] isAnimating in
                if isAnimating {
                    self?.rescheduleAutoStopTimer()
                } else {
                    self?.autoStopTimer?.invalidate()
                    self?.autoStopTimer = nil
                }
            }
            .store(in: &subscriptions)

        setUpGlobalHotKeys()
    }

    @objc func showSettingsFromNotification(_ notification: Notification) {
        DispatchQueue.main.async { [weak self] in
            self?.showSettings()
        }
    }

    @objc func raiseTooltipWindows(_ notification: Notification?) {
        guard settingsWindow.isVisible else { return }
        for window in NSApp.windows where String(describing: type(of: window)).contains("ToolTip") {
            if window.level != Self.tooltipWindowLevel {
                window.level = Self.tooltipWindowLevel
            }
        }
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        showSettings()
        return false
    }

    func showSettings() {
        if !settingsWindow.isVisible {
            toggleSettings(nil)
        } else {
            settingsWindow.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    func applyAppVisibility(_ visibility: AppVisibility) {
        switch visibility {
        case .topBarOnly:
            NSApp.setActivationPolicy(.accessory)
            if statusItem == nil { setUpStatusItem() }
        case .dockOnly:
            NSApp.setActivationPolicy(.regular)
            if let item = statusItem {
                NSStatusBar.system.removeStatusItem(item)
                statusItem = nil
            }
        case .both:
            NSApp.setActivationPolicy(.regular)
            if statusItem == nil { setUpStatusItem() }
        }
    }

    // MARK: - Reminder Notifications

    func requestNotificationPermission() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
    }

    func rescheduleReminderTimer() {
        reminderTimer?.invalidate()
        reminderTimer = nil

        let minutes = settingsModel.reminderIntervalMinutes
        guard minutes > 0 else { return }

        let interval = minutes * 60
        reminderTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { [weak self] _ in
            self?.postReminderNotification()
        }
    }

    func postReminderNotification() {
        let content = UNMutableNotificationContent()
        content.title = "exhale"
        content.body = "Remember to breathe"
        content.sound = .default

        let request = UNNotificationRequest(
            identifier: "exhale.reminder.\(UUID().uuidString)",
            content: content,
            trigger: nil
        )
        UNUserNotificationCenter.current().add(request)
    }

    // MARK: - Auto-Stop Timer

    func rescheduleAutoStopTimer() {
        autoStopTimer?.invalidate()
        autoStopTimer = nil

        let minutes = settingsModel.autoStopMinutes
        guard minutes > 0, settingsModel.isAnimating else { return }

        let interval = minutes * 60
        autoStopTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: false) { [weak self] _ in
            DispatchQueue.main.async {
                self?.settingsModel.stop()
            }
        }
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
            tooltipCheckTimer?.invalidate()
            tooltipCheckTimer = nil
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
                    width: min(w, 246),
                    height: min(h, 870)
                )
                settingsWindow.setFrame(restoredFrame, display: true)
            }

            settingsWindow.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            // Settings window must be above the overlay so it's usable at any opacity
            settingsWindow.level = Self.settingsWindowLevel

            // Start tooltip elevation timer while settings is visible
            if tooltipCheckTimer == nil {
                tooltipCheckTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { [weak self] _ in
                    self?.raiseTooltipWindows(nil)
                }
            }
        }
    }
    
    func reloadContentView() {
        // Intentionally left blank.
        // NSHostingView is created once per window; SwiftUI updates via EnvironmentObject.
    }
    
    // Prevent the settings window from closing (just hide it instead)
    func windowWillResize(_ sender: NSWindow, to frameSize: NSSize) -> NSSize {
        if sender == settingsWindow {
            return NSSize(width: sender.frame.width, height: frameSize.height)
        }
        return frameSize
    }

    func windowShouldClose(_ sender: NSWindow) -> Bool {
        if sender == settingsWindow {
            settingsWindow.orderOut(sender)
            tooltipCheckTimer?.invalidate()
            tooltipCheckTimer = nil
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
