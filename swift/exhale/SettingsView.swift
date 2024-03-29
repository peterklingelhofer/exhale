//  SettingsView.swift
import SwiftUI

func validateValue(value: Double, minimumValue: Double, formatter: NumberFormatter) -> Double {
    var updatedValue = value
    if updatedValue < minimumValue {
        updatedValue = minimumValue
    }
    if let maximum = formatter.maximum?.doubleValue,
       updatedValue > maximum {
        updatedValue = maximum
    }
    return updatedValue
}

struct TextFieldWithValidation: View {
    var title: String
    @Binding var value: Double
    var formatter: NumberFormatter
    var minimumValue: Double
    
    var body: some View {
        HStack {
            Text(title)
            Spacer()
            TextField("", value: $value, formatter: formatter)
                .onChange(of: value) { newValue in
                    value = validateValue(value: newValue, minimumValue: minimumValue, formatter: formatter)
                }
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .frame(width: 65)
        }
    }
}

func createNumberFormatter(limits: (min: Double, max: Double?)) -> NumberFormatter {
    let formatter = NumberFormatter()
    formatter.numberStyle = .decimal
    formatter.maximumFractionDigits = 3
    formatter.minimum = NSNumber(value: limits.min)
    if let max = limits.max {
        formatter.maximum = NSNumber(value: max)
    }
    formatter.usesGroupingSeparator = false
    return formatter
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
    
    var body: some View {
        VStack {
            HStack {
                VStack {
                    Image(systemName: isAnimating ? "stop.circle.fill" : "play.circle.fill")
                    Text(isAnimating ? "Stop" : "Start")
                }
                .onTapGesture {
                    isAnimating.toggle()
                }
                .keyboardShortcut("s", modifiers: .command)
                .help(isAnimating ? "Stop the animation, remove all screen tints, and put the app into idle mode." : "Start the app and re-initialize animation.")
                
                Spacer().frame(width: 16)
                
                VStack {
                    Spacer().frame(height: 2)
                    Image(systemName: "eraser")
                    Text("Reset")
                }
                .onTapGesture {
                    settingsModel.resetToDefaults()
                }
                .help("Reset all settings to their default values.")
            }
            .padding(.top)
            
            HStack {
                Spacer()
                
                VStack {
                    Form {
                        HStack {
                            VStack(alignment: .leading) {
                                HStack {
                                    Text("Inhale Color")
                                        .frame(width: labelWidth, alignment: .leading)
                                    
                                    ColorPicker("", selection: $inhaleColor, supportsOpacity: false)
                                        .labelsHidden()
                                        .frame(alignment: .trailing)
                                }
                                .help("Choose the color for the inhale phase.")
                                
                                HStack {
                                    Text("Exhale Color")
                                        .frame(width: labelWidth, alignment: .leading)
                                    
                                    ColorPicker("", selection: $exhaleColor, supportsOpacity: false)
                                        .labelsHidden()
                                        .frame(alignment: .trailing)
                                }
                                .help("Choose the color for the exhale phase.")
                                
                                HStack {
                                    Text("Background Color")
                                        .frame(width: labelWidth, alignment: .leading)
                                    
                                    ColorPicker("", selection: $backgroundColor, supportsOpacity: false)
                                        .labelsHidden()
                                        .frame(alignment: .trailing)
                                        .disabled(shape == .fullscreen)
                                }
                                .help("Choose the background color, or the color outside of the animation shape. This parameter has no effect if the Shape parameter is set to Fullscreen.")
                                
                                TextFieldWithValidation(title: "Inhale Duration (s)", value: $inhaleDuration, formatter: createNumberFormatter(limits: (min: 0.1, max: nil)), minimumValue: 0.1)
                                    .help("Choose the duration of the inhale phase, in seconds.")
                                
                                TextFieldWithValidation(title: "Post-Inhale Hold (s)", value: $postInhaleHoldDuration, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0)
                                    .help("Choose the duration of the hold/pause that occurs at the end of the inhale phase, in seconds.")
                                
                                TextFieldWithValidation(title: "Exhale Duration (s)", value: $exhaleDuration, formatter: createNumberFormatter(limits: (min: 0.1, max: nil)), minimumValue: 0.1)
                                    .help("Choose the duration of the exhale phase, in seconds.")
                                
                                TextFieldWithValidation(title: "Post-Exhale Hold (s)", value: $postExhaleHoldDuration, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0)
                                    .help("Choose the duration of the hold/pause that occurs at the end of the exhale phase, in seconds.")
                                
                                TextFieldWithValidation(title: "Overlay Opacity", value: $overlayOpacity, formatter: createNumberFormatter(limits: (min: 0, max: 1)), minimumValue: 0.0)
                                    .help("Choose the transparency of the overlay colors, with lower values being more transparent and higher values being more visible.")
                            }.padding()
                            
                            VStack(alignment: .leading) {
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
                                }
                                .help("Choose the Shape of the animation. Fullscreen changes the color of every pixel on the screen, starting with the Inhale Color at the beginning of the inhale phase and transitioning to the Exhale Color, then for the exhale phase transitioning back from the Exhale Color to the Inhale Color (Fullscreen uses Gradient Type Constant, setting it to Linear Gradient has no effect). Rectangle rises vertically from the bottom of the screen to the top for the inhale phase, and then lowers back down from the top to the bottom for the exhale phase. Circle grows outwards starting from a single point in the center of the screen to the outer edges of the screen for the inhale phase, and then shrinks back to the center again for the exhale phase.")
                                
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
                                }
                                .help("Choose the gradient color effect. Off allows the change the color to transition over time between the Inhale Color and Exhale Color and back. Inner and On causes abrupt color transitions at the end of the inhale and exhale phases (which can make it easier to notice when it is time to reverse the direction of your breathing), and enables a color gradient from the Background Color to the Inhale Color or Exhale Color (depending on the current phase). When the Shape is Circle the Inner gradient color transition is from the innermost center point of the Circle to the diameter, whereas with the Rectangle shape the Inner gradient color transition is from the bottom of the Rectangle to the top. On has similar behavior to Inner, but includes a gradient on the exterior of shape in addition to the interior. This parameter has no effect if the Shape parameter is set to Fullscreen.")
                                
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
                                }
                                .help("Choose the animation speed's acceleration curve. Sinusoidal begins slowly, speeds up during the middle point, and slows down again near the end, creating a natural and organic feel to the transition. Linear provides a constant animation speed and acceleration rate throughout the duration of the animation.")
                                TextFieldWithValidation(title: "Inhale Randomization", value: $randomizedTimingInhale, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0.0)
                                    .help("Choose the extent to which the duration of the inhale phase should be randomized, in seconds.")
                                
                                TextFieldWithValidation(title: "Post-Inhale Hold Randomization", value: $randomizedTimingPostInhaleHold, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0.0)
                                    .help("Choose the extent to which the duration of the of the hold/pause that occurs at the end of the inhale phase should be randomized, in seconds.")
                                
                                TextFieldWithValidation(title: "Exhale Randomization", value: $randomizedTimingExhale, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0.0)
                                    .help("Choose the extent to which the duration of the exhale phase should be randomized, in seconds.")
                                
                                TextFieldWithValidation(title: "Post-Exhale Hold Randomization", value: $randomizedTimingPostExhaleHold, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0.0)
                                    .help("Choose the extent to which the duration of the of the hold/pause that occurs at the end of the exhale phase should be randomized, in seconds.")
                                
                                TextFieldWithValidation(title: "Drift", value: $drift, formatter: createNumberFormatter(limits: (min: 0.0, max: nil)), minimumValue: 0.0)
                                    .help("Choose the extent to which the duration of the every inhale and exhale phase (as well as the end-of-phase hold if Post-Inhale Hold or Post-Exhale Hold are set to non-zero values) lengthens or shortens in duration over time. Drift is multiplicative, so a value of 1.01 will gradually lengthen the duration (by 1% each cycle), allowing you to extend the duration of your breath over time, whereas a value of 0.75 would shorten the duration of each phase (by 25%) each cycle. Values of 1.01 - 1.05 are recommended for working on slowly elongating one's breath cycle.")
                            }
                        }.lineLimit(1)
                    }.frame(width: 724)
                    
                    Spacer()
                }
                
                Spacer()
            }
        }
    }
}
