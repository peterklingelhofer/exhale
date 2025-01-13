// SettingsView.swift
import SwiftUI

func getAppVersion() -> String {
    if let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String,
       let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String {
        return "Version \(version) (Build \(build))"
    }
    return "Unknown"
}

struct SettingsView: View {
    @EnvironmentObject var settingsModel: SettingsModel
    @Binding var showSettings: Bool
    @Binding var inhaleColor: Color
    @Binding var exhaleColor: Color
    @Binding var backgroundColor: Color
    @Binding var colorFillType: ColorFillGradient
    @Binding var inhaleDuration: Double
    @Binding var postInhaleHoldDuration: Double
    @Binding var exhaleDuration: Double
    @Binding var postExhaleHoldDuration: Double
    @Binding var drift: Double
    @Binding var overlayOpacity: Double
    @Binding var shape: AnimationShape
    @Binding var animationMode: AnimationMode
    @Binding var randomizedTimingInhale: Double
    @Binding var randomizedTimingPostInhaleHold: Double
    @Binding var randomizedTimingExhale: Double
    @Binding var randomizedTimingPostExhaleHold: Double
    @Binding var isAnimating: Bool

    private let labelWidth: CGFloat = 130
    private let controlWidth: CGFloat = 90

    // State variables for managing opacity change
    @State private var tempOverlayOpacity: Double = 0.0
    @State private var showOpacityWarning: Bool = false
    @State private var previousOverlayOpacity: Double = 0.0

    // UserDefaults key for tracking if the alert has been shown
    private let opacityAlertShownKey = "hasShownOverlayOpacityAlert"

    var body: some View {
        VStack {
            // App Version Display
            HStack {
                Spacer()
                Text("\(getAppVersion())")
                    .font(.footnote)
                    .padding(.top, 4)
            }

            // Control Buttons
            HStack {
                // Start Button
                ControlButton(
                    systemImageName: "play.circle.fill",
                    title: "Start",
                    action: { settingsModel.start() },
                    keyboardShortcut: "s",
                    modifiers: .command,
                    helpText: "Start the app and re-initialize animation."
                )

                Spacer().frame(width: 16)

                // Stop Button
                ControlButton(
                    systemImageName: "stop.circle.fill",
                    title: "Stop",
                    action: { settingsModel.stop() },
                    keyboardShortcut: "x",
                    modifiers: .command,
                    helpText: "Stop the animation and remove all screen tints."
                )

                Spacer().frame(width: 16)

                // Tint Button
                ControlButton(
                    systemImageName: "paintbrush.fill",
                    title: "Tint",
                    action: { settingsModel.pause() },
                    keyboardShortcut: "p",
                    modifiers: .command,
                    helpText: "Tint the screen with the background color."
                )

                Spacer().frame(width: 16)

                // Reset Button
                ControlButton(
                    systemImageName: "arrow.counterclockwise.circle.fill",
                    title: "Reset",
                    action: { settingsModel.resetToDefaults() },
                    keyboardShortcut: "r",
                    modifiers: .command,
                    helpText: "Reset all settings to their default values."
                )

                Spacer()
            }
            .padding(.leading, 25)

            // Settings Form
            HStack {
                Spacer()

                VStack {
                    Form {
                        HStack {
                            VStack(alignment: .leading) {
                                // Inhale Color Picker
                                HStack {
                                    Text("Inhale Color")
                                        .frame(width: labelWidth, alignment: .leading)

                                    ColorPicker("", selection: $inhaleColor)
                                        .labelsHidden()
                                        .frame(alignment: .trailing)
                                        .onChange(of: inhaleColor) { _ in
                                            settingsModel.triggerAnimationReset()
                                        }
                                }
                                .help("Choose the color for the inhale phase.")

                                // Exhale Color Picker
                                HStack {
                                    Text("Exhale Color")
                                        .frame(width: labelWidth, alignment: .leading)

                                    ColorPicker("", selection: $exhaleColor)
                                        .labelsHidden()
                                        .frame(alignment: .trailing)
                                        .onChange(of: exhaleColor) { _ in
                                            settingsModel.triggerAnimationReset()
                                        }
                                }
                                .help("Choose the color for the exhale phase.")

                                // Background Color Picker
                                HStack {
                                    Text("Background Color")
                                        .frame(width: labelWidth, alignment: .leading)

                                    ColorPicker("", selection: $backgroundColor)
                                        .labelsHidden()
                                        .frame(alignment: .trailing)
                                        .disabled(shape == .fullscreen)
                                        .onChange(of: backgroundColor) { _ in
                                            settingsModel.triggerAnimationReset()
                                        }
                                }
                                .help("Choose the background color, or the color outside of the animation shape. This parameter has no effect if the Shape parameter is set to Fullscreen.")

                                // Inhale Duration
                                CombinedStepperTextField(
                                    title: "Inhale Duration (s)",
                                    value: $inhaleDuration,
                                    limits: (min: 0, max: nil)
                                )
                                .help("Choose the duration of the inhale phase, in seconds.")
                                .onChange(of: inhaleDuration) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Post-Inhale Hold Duration
                                CombinedStepperTextField(
                                    title: "Post-Inhale Hold (s)",
                                    value: $postInhaleHoldDuration,
                                    limits: (min: 0, max: nil)
                                )
                                .help("Choose the duration of the hold/pause that occurs at the end of the inhale phase, in seconds.")
                                .onChange(of: postInhaleHoldDuration) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Exhale Duration
                                CombinedStepperTextField(
                                    title: "Exhale Duration (s)",
                                    value: $exhaleDuration,
                                    limits: (min: 0, max: nil)
                                )
                                .help("Choose the duration of the exhale phase, in seconds.")
                                .onChange(of: exhaleDuration) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Post-Exhale Hold Duration
                                CombinedStepperTextField(
                                    title: "Post-Exhale Hold (s)",
                                    value: $postExhaleHoldDuration,
                                    limits: (min: 0, max: nil)
                                )
                                .help("Choose the duration of the hold/pause that occurs at the end of the exhale phase, in seconds.")
                                .onChange(of: postExhaleHoldDuration) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Overlay Opacity Control
                                CombinedStepperTextField(
                                    title: "Overlay Opacity (%)",
                                    value: Binding(
                                        get: { self.tempOverlayOpacity * 100 },
                                        set: { self.tempOverlayOpacity = $0 / 100 }
                                    ),
                                    limits: (min: 0, max: 100)
                                )
                                .onAppear {
                                    tempOverlayOpacity = overlayOpacity
                                    previousOverlayOpacity = overlayOpacity
                                }
                                .onChange(of: tempOverlayOpacity) { newValue in
                                    let validatedValue = validateValue(
                                        value: newValue,
                                        minimumValue: 0.0,
                                        maximumValue: 1.0
                                    )

                                    if validatedValue > 0.6 && !UserDefaults.standard.bool(forKey: opacityAlertShownKey) {
                                        showOpacityWarning = true
                                    } else {
                                        overlayOpacity = validatedValue
                                        previousOverlayOpacity = validatedValue
                                        settingsModel.triggerAnimationReset()
                                    }
                                }
                                .onChange(of: overlayOpacity) { newValue in
                                    // Synchronize tempOverlayOpacity and previousOverlayOpacity with overlayOpacity
                                    tempOverlayOpacity = newValue
                                    previousOverlayOpacity = newValue
                                }
                                .help("Choose the transparency of the overlay colors, with lower values being more transparent and higher values being more visible.")
                            }
                            .alert(isPresented: $showOpacityWarning) {
                                Alert(
                                    title: Text("High Opacity Warning"),
                                    message: Text("""
                                        You've attempted to set the overlay opacity to a very high value (>60%).

                                        To change this value back:
                                        1. Swipe left or right with four fingers on your trackpad to switch to a different workspace, or four finger swipe up and select an alternate workspace at the top.
                                        2. From the top bar menu, click Preferences to close the Preferences pane in the previous workspace.
                                        3. Access the top bar menu again, click Preferences to open the Preferences pane in the current workspace, and adjust your Opacity value accordingly.
                                        4. Switch back to the original workspace.

                                        **Note:** A high opacity value can obscure the Preferences pane in the current workspace.
                                        """),
                                    primaryButton: .default(Text("OK")) {
                                        // Commit the new opacity value
                                        overlayOpacity = tempOverlayOpacity
                                        previousOverlayOpacity = tempOverlayOpacity
                                        settingsModel.triggerAnimationReset()

                                        // Set the flag to true to indicate the alert has been shown
                                        UserDefaults.standard.set(true, forKey: opacityAlertShownKey)
                                    },
                                    secondaryButton: .cancel() {
                                        // Revert to the previous opacity value
                                        tempOverlayOpacity = previousOverlayOpacity
                                    }
                                )
                            }
                            .padding()

                            VStack(alignment: .leading) {
                                // Shape Picker
                                HStack {
                                    Text("Shape")
                                        .frame(width: labelWidth, alignment: .leading)

                                    Picker("", selection: $shape) {
                                        ForEach(AnimationShape.allCases, id: \.self) { shape in
                                            Text(shape.rawValue).tag(shape)
                                        }
                                    }
                                    .pickerStyle(SegmentedPickerStyle())
                                    .frame(width: controlWidth)
                                    .labelsHidden()
                                    .onChange(of: shape) { _ in
                                        settingsModel.triggerAnimationReset()
                                    }
                                }
                                .help("Choose the Shape of the animation. Fullscreen changes the color of every pixel on the screen, starting with the Inhale Color at the beginning of the inhale phase and transitioning to the Exhale Color, then for the exhale phase transitioning back from the Exhale Color to the Inhale Color (Fullscreen uses Gradient Type Constant, setting it to Linear Gradient has no effect). Rectangle rises vertically from the bottom of the screen to the top for the inhale phase, and then lowers back down from the top to the bottom for the exhale phase. Circle grows outwards starting from a single point in the center of the screen to the outer edges of the screen for the inhale phase, and then shrinks back to the center again for the exhale phase.")

                                // Gradient Picker
                                HStack {
                                    Text("Gradient")
                                        .frame(width: labelWidth, alignment: .leading)

                                    Picker("", selection: $colorFillType) {
                                        ForEach(ColorFillGradient.allCases) { type in
                                            Text(type.rawValue).tag(type)
                                        }
                                    }
                                    .pickerStyle(SegmentedPickerStyle())
                                    .frame(width: controlWidth)
                                    .disabled(shape == .fullscreen)
                                    .labelsHidden()
                                    .onChange(of: colorFillType) { _ in
                                        settingsModel.triggerAnimationReset()
                                    }
                                }
                                .help("Choose the gradient color effect. Off allows the change the color to transition over time between the Inhale Color and Exhale Color and back. Inner and On causes abrupt color transitions at the end of the inhale and exhale phases (which can make it easier to notice when it is time to reverse the direction of your breathing), and enables a color gradient from the Background Color to the Inhale Color or Exhale Color (depending on the current phase). When the Shape is Circle the Inner gradient color transition is from the innermost center point of the Circle to the diameter, whereas with the Rectangle shape the Inner gradient color transition is from the bottom of the Rectangle to the top. On has similar behavior to Inner, but includes a gradient on the exterior of shape in addition to the interior. This parameter has no effect if the Shape parameter is set to Fullscreen.")

                                // Animation Mode Picker
                                HStack {
                                    Text("Animation Mode")
                                        .frame(width: labelWidth, alignment: .leading)

                                    Picker("", selection: $animationMode) {
                                        ForEach(AnimationMode.allCases) { mode in
                                            Text(mode.rawValue).tag(mode)
                                        }
                                    }
                                    .pickerStyle(SegmentedPickerStyle())
                                    .frame(width: controlWidth)
                                    .labelsHidden()
                                    .onChange(of: animationMode) { _ in
                                        settingsModel.triggerAnimationReset()
                                    }
                                }
                                .help("Choose the animation speed's acceleration curve. Sinusoidal begins slowly, speeds up during the middle point, and slows down again near the end, creating a natural and organic feel to the transition. Linear provides a constant animation speed and acceleration rate throughout the duration of the animation.")

                                // Inhale Randomization
                                CombinedStepperTextField(
                                    title: "Inhale Randomization (%)",
                                    value: Binding(
                                        get: { self.randomizedTimingInhale * 100 },
                                        set: { self.randomizedTimingInhale = $0 / 100 }
                                    )
                                )
                                .help("Choose the extent to which the duration of the inhale phase should be randomized, in seconds.")
                                .onChange(of: randomizedTimingInhale) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Post-Inhale Hold Randomization
                                CombinedStepperTextField(
                                    title: "Post-Inhale Hold Randomization (%)",
                                    value: Binding(
                                        get: { self.randomizedTimingPostInhaleHold * 100 },
                                        set: { self.randomizedTimingPostInhaleHold = $0 / 100 }
                                    )
                                )
                                .help("Choose the extent to which the duration of the hold/pause that occurs at the end of the inhale phase should be randomized, in seconds.")
                                .onChange(of: randomizedTimingPostInhaleHold) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Exhale Randomization
                                CombinedStepperTextField(
                                    title: "Exhale Randomization (%)",
                                    value: Binding(
                                        get: { self.randomizedTimingExhale * 100 },
                                        set: { self.randomizedTimingExhale = $0 / 100 }
                                    )
                                )
                                .help("Choose the extent to which the duration of the exhale phase should be randomized, in seconds.")
                                .onChange(of: randomizedTimingExhale) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Post-Exhale Hold Randomization
                                CombinedStepperTextField(
                                    title: "Post-Exhale Hold Randomization (%)",
                                    value: Binding(
                                        get: { self.randomizedTimingPostExhaleHold * 100 },
                                        set: { self.randomizedTimingPostExhaleHold = $0 / 100 }
                                    )
                                )
                                .help("Choose the extent to which the duration of the hold/pause that occurs at the end of the exhale phase should be randomized, in seconds.")
                                .onChange(of: randomizedTimingPostExhaleHold) { _ in
                                    settingsModel.triggerAnimationReset()
                                }

                                // Drift
                                CombinedStepperTextField(
                                    title: "Drift (%)",
                                    value: Binding(
                                        get: { self.drift * 100 - 100 },
                                        set: { self.drift = ($0 + 100) / 100 }
                                    )
                                )
                                .help("Choose the extent to which the duration of every inhale and exhale phase (as well as the end-of-phase hold if Post-Inhale Hold or Post-Exhale Hold are set to non-zero values) lengthens or shortens in duration over time. Drift is multiplicative, so a value of 1% will gradually lengthen the duration (by 1% each cycle), allowing you to extend the duration of your breath over time, whereas a value of -25% would shorten the duration of each phase (by 25%) each cycle. Values of 1% - 5% are recommended for working on slowly elongating one's breath cycle.")
                                .onChange(of: drift) { _ in
                                    settingsModel.triggerAnimationReset()
                                }
                            }

                            Spacer()
                        }
                    }
                }
            }
        }
        .alert(isPresented: $showOpacityWarning) {
            Alert(
                title: Text("High Opacity Warning"),
                message: Text("""
                    You've attempted to set the overlay opacity to a very high value (>60%).

                    To change this value back:
                    1. Swipe left or right with four fingers on your trackpad to switch to a different workspace, or four finger swipe up and select an alternate workspace at the top.
                    2. From the top bar menu, click Preferences to close the Preferences pane in the previous workspace.
                    3. Access the top bar menu again, click Preferences to open the Preferences pane in the current workspace, and adjust your Opacity value accordingly.
                    4. Switch back to the original workspace.

                    **Note:** A high opacity value can obscure the Preferences pane in the current workspace.
                    """),
                primaryButton: .default(Text("OK")) {
                    // Commit the new opacity value
                    overlayOpacity = tempOverlayOpacity
                    previousOverlayOpacity = tempOverlayOpacity
                    settingsModel.triggerAnimationReset()

                    // Set the flag to true to indicate the alert has been shown
                    UserDefaults.standard.set(true, forKey: opacityAlertShownKey)
                },
                secondaryButton: .cancel() {
                    // Revert to the previous opacity value
                    tempOverlayOpacity = previousOverlayOpacity
                }
            )
        }
        .padding(.trailing, 20)
        .padding(.bottom, 8)
    }
}
