// SettingsView.swift
import SwiftUI

// MARK: - Section card
private struct SectionCard<Content: View>: View {
    var header: String? = nil
    @ViewBuilder var content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let header = header {
                Text(header.uppercased())
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundColor(.secondary)
                    .tracking(0.8)
            }
            content()
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(NSColor.controlBackgroundColor).opacity(0.55))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .strokeBorder(Color.primary.opacity(0.06), lineWidth: 1)
        )
    }
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
    @Binding var holdRippleMode: HoldRippleMode
    @Binding var isAnimating: Bool
    @Binding var appVisibility: AppVisibility
    @Binding var reminderIntervalMinutes: Double
    @Binding var autoStopMinutes: Double

    @State private var tempOverlayOpacity: Double = 0.0
    @State private var previousOverlayOpacity: Double = 0.0

    private let rowSpacing: CGFloat = 8
    private let sectionSpacing: CGFloat = 10

    var body: some View {
        VStack(spacing: sectionSpacing) {
            // Controls — pinned at top
            SectionCard {
                HStack(spacing: 8) {
                    ControlButton(
                        systemImageName: "play.circle.fill",
                        title: "Start",
                        action: { settingsModel.start() },
                        keyboardShortcut: "a",
                        modifiers: [.control, .shift],
                        helpText: "Start the app and re-initialize animation."
                    )
                    ControlButton(
                        systemImageName: "stop.circle.fill",
                        title: "Stop",
                        action: { settingsModel.stop() },
                        keyboardShortcut: "s",
                        modifiers: [.control, .shift],
                        helpText: "Stop the animation and remove all screen tints."
                    )
                    ControlButton(
                        systemImageName: "arrow.counterclockwise.circle.fill",
                        title: "Reset",
                        action: { settingsModel.resetToDefaults() },
                        keyboardShortcut: "f",
                        modifiers: [.control, .shift],
                        helpText: "Reset all settings to their default values."
                    )
                }
            }
            .padding(.horizontal, 14)
            .padding(.top, 14)

            // Scrollable settings
            ScrollView {
                VStack(spacing: sectionSpacing) {
                // Colors & Appearance
                SectionCard(header: "Appearance") {
                    VStack(alignment: .leading, spacing: rowSpacing) {
                        settingRow("Inhale Color") {
                            ColorPicker("", selection: $inhaleColor, supportsOpacity: false)
                                .labelsHidden()
                                .onChange(of: inhaleColor) { _ in
                                    settingsModel.triggerAnimationReset()
                                }
                        }
                        .help("Choose the color for the inhale phase.")

                        settingRow("Exhale Color") {
                            ColorPicker("", selection: $exhaleColor, supportsOpacity: false)
                                .labelsHidden()
                                .onChange(of: exhaleColor) { _ in
                                    settingsModel.triggerAnimationReset()
                                }
                        }
                        .help("Choose the color for the exhale phase.")

                        settingRow("Background Color") {
                            ColorPicker("", selection: $backgroundColor, supportsOpacity: true)
                                .labelsHidden()
                                .disabled(shape == .fullscreen)
                                .onChange(of: backgroundColor) { _ in
                                    settingsModel.triggerAnimationReset()
                                }
                        }
                        .help("Choose the background color. No effect when Shape is Fullscreen.")

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
                            overlayOpacity = validatedValue
                            previousOverlayOpacity = validatedValue
                            settingsModel.triggerAnimationReset()
                        }
                        .onChange(of: overlayOpacity) { newValue in
                            tempOverlayOpacity = newValue
                            previousOverlayOpacity = newValue
                        }
                        .help("Transparency of the overlay. Lower = more transparent.")

                        settingRow("Shape") {
                            Picker("", selection: $shape) {
                                ForEach(AnimationShape.allCases, id: \.self) { shape in
                                    Text(shape.shortLabel).tag(shape)
                                }
                            }
                            .pickerStyle(SegmentedPickerStyle())

                            .labelsHidden()
                        }
                        .help("Shape of the animation: Fullscreen, Rectangle, or Circle.")

                        settingRow("Gradient") {
                            Picker("", selection: $colorFillType) {
                                ForEach(ColorFillGradient.allCases) { type in
                                    Text(type.rawValue).tag(type)
                                }
                            }
                            .pickerStyle(SegmentedPickerStyle())

                            .disabled(shape == .fullscreen)
                            .labelsHidden()
                            .onChange(of: colorFillType) { _ in
                                settingsModel.triggerAnimationReset()
                            }
                        }
                        .help("Gradient color effect. No effect when Shape is Fullscreen.")

                        settingRow("Animation") {
                            Picker("", selection: $animationMode) {
                                ForEach(AnimationMode.allCases) { mode in
                                    Text(mode.rawValue).tag(mode)
                                }
                            }
                            .pickerStyle(SegmentedPickerStyle())

                            .labelsHidden()
                            .onChange(of: animationMode) { _ in
                                settingsModel.triggerAnimationReset()
                            }
                        }
                        .help("Sinusoidal eases in/out naturally. Linear is constant speed.")

                        settingRow("Hold Ripple") {
                            Picker("", selection: $holdRippleMode) {
                                ForEach(HoldRippleMode.allCases) { mode in
                                    Text(mode.rawValue).tag(mode)
                                }
                            }
                            .pickerStyle(SegmentedPickerStyle())
                            .labelsHidden()
                            .onChange(of: holdRippleMode) { _ in
                                settingsModel.triggerAnimationReset()
                            }
                        }
                        .help("Hold phase ripple: Gradient (smooth glow), Stark (solid edge), or Off.")

                        settingRow("Show In") {
                            Picker("", selection: $appVisibility) {
                                ForEach(AppVisibility.allCases) { vis in
                                    Text(vis.shortLabel).tag(vis)
                                }
                            }
                            .pickerStyle(SegmentedPickerStyle())

                            .labelsHidden()
                        }
                        .help("Where exhale appears: Top Bar, Dock, or Both.")
                    }
                }

                // Timing
                SectionCard(header: "Timing") {
                    VStack(alignment: .leading, spacing: rowSpacing) {
                        CombinedStepperTextField(
                            title: "Inhale Duration (s)",
                            value: $inhaleDuration,
                            limits: (min: 0, max: nil)
                        )
                        .help("Duration of the inhale phase, in seconds.")
                        .onChange(of: inhaleDuration) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Post-Inhale Hold (s)",
                            value: $postInhaleHoldDuration,
                            limits: (min: 0, max: nil)
                        )
                        .help("Hold/pause duration at the end of inhale, in seconds.")
                        .onChange(of: postInhaleHoldDuration) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Exhale Duration (s)",
                            value: $exhaleDuration,
                            limits: (min: 0, max: nil)
                        )
                        .help("Duration of the exhale phase, in seconds.")
                        .onChange(of: exhaleDuration) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Post-Exhale Hold (s)",
                            value: $postExhaleHoldDuration,
                            limits: (min: 0, max: nil)
                        )
                        .help("Hold/pause duration at the end of exhale, in seconds.")
                        .onChange(of: postExhaleHoldDuration) { _ in
                            settingsModel.triggerAnimationReset()
                        }
                    }
                }

                // Randomization
                SectionCard(header: "Randomization") {
                    VStack(alignment: .leading, spacing: rowSpacing) {
                        CombinedStepperTextField(
                            title: "Inhale (%)",
                            value: Binding(
                                get: { self.randomizedTimingInhale * 100 },
                                set: { self.randomizedTimingInhale = $0 / 100 }
                            )
                        )
                        .help("Randomize inhale duration by this percentage.")
                        .onChange(of: randomizedTimingInhale) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Post-Inhale Hold (%)",
                            value: Binding(
                                get: { self.randomizedTimingPostInhaleHold * 100 },
                                set: { self.randomizedTimingPostInhaleHold = $0 / 100 }
                            )
                        )
                        .help("Randomize post-inhale hold duration by this percentage.")
                        .onChange(of: randomizedTimingPostInhaleHold) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Exhale (%)",
                            value: Binding(
                                get: { self.randomizedTimingExhale * 100 },
                                set: { self.randomizedTimingExhale = $0 / 100 }
                            )
                        )
                        .help("Randomize exhale duration by this percentage.")
                        .onChange(of: randomizedTimingExhale) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Post-Exhale Hold (%)",
                            value: Binding(
                                get: { self.randomizedTimingPostExhaleHold * 100 },
                                set: { self.randomizedTimingPostExhaleHold = $0 / 100 }
                            )
                        )
                        .help("Randomize post-exhale hold duration by this percentage.")
                        .onChange(of: randomizedTimingPostExhaleHold) { _ in
                            settingsModel.triggerAnimationReset()
                        }

                        CombinedStepperTextField(
                            title: "Drift (%)",
                            value: Binding(
                                get: { self.drift * 100 - 100 },
                                set: { self.drift = ($0 + 100) / 100 }
                            )
                        )
                        .help("Multiplicative drift per cycle. 1-5% recommended for gradually lengthening breath.")
                        .onChange(of: drift) { _ in
                            settingsModel.triggerAnimationReset()
                        }
                    }
                }

                // Timers
                SectionCard(header: "Timers") {
                    VStack(alignment: .leading, spacing: rowSpacing) {
                        CombinedStepperTextField(
                            title: "Reminder (mins)",
                            value: $reminderIntervalMinutes,
                            limits: (min: 0, max: nil),
                            hint: "0 = off"
                        )
                        .help("Notification reminder every N minutes. 0 to disable.")

                        CombinedStepperTextField(
                            title: "End After (mins)",
                            value: $autoStopMinutes,
                            limits: (min: 0, max: nil),
                            hint: "0 = off"
                        )
                        .help("Auto-stop after N minutes. 0 to disable.")
                    }
                }
                }
                .padding(.horizontal, 14)
                .padding(.bottom, 14)
            }
        }
    }

    private let settingLabelWidth: CGFloat = 115

    private func settingRow<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        HStack {
            Text(title)
                .lineLimit(1)
                .frame(width: settingLabelWidth, alignment: .leading)
            Spacer()
            content()
        }
    }
}
