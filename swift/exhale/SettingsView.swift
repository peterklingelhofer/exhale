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
                .frame(width: 100)
        }
    }
}

func createNumberFormatter(limits: (min: Double, max: Double?)) -> NumberFormatter {
    let formatter = NumberFormatter()
    formatter.numberStyle = .decimal
    formatter.maximumFractionDigits = 2
    formatter.minimum = NSNumber(value: limits.min)
    if let max = limits.max {
        formatter.maximum = NSNumber(value: max)
    }
    formatter.usesGroupingSeparator = false
    return formatter
}

struct SettingsView: View {
    @Binding var showSettings: Bool
    @Binding var inhaleColor: Color
    @Binding var exhaleColor: Color
    @Binding var backgroundColor: Color
    @Binding var colorFillType: ColorFillType
    @Binding var inhaleDuration: Double
    @Binding var postInhaleHoldDuration: Double
    @Binding var exhaleDuration: Double
    @Binding var postExhaleHoldDuration: Double
    @Binding var drift: Double
    @Binding var overlayOpacity: Double
    @Binding var shape: AnimationShape
    @Binding var animationMode: AnimationMode
    
    var body: some View {
        VStack {
            Spacer()
            
            HStack {
                Spacer()
                
                VStack {
                    Form {
                        HStack {
                            VStack {
                                ColorPicker("Inhale Color", selection: $inhaleColor, supportsOpacity: false)
                                
                                ColorPicker("Exhale Color", selection: $exhaleColor, supportsOpacity: false)
                                
                                ColorPicker("Background Color", selection: $backgroundColor, supportsOpacity: false)
                                
                                Picker("Gradient Type", selection: $colorFillType) {
                                    ForEach(ColorFillType.allCases) { type in
                                        Text(type.rawValue).tag(type)
                                    }
                                }
                                .pickerStyle(MenuPickerStyle())
                                
                                Picker("Shape", selection: $shape) {
                                    ForEach(AnimationShape.allCases, id: \.self) { shape in
                                        Text(shape.rawValue).tag(shape)
                                    }
                                }
                                .pickerStyle(MenuPickerStyle())
                                
                                Picker("Animation Mode", selection: $animationMode) {
                                    ForEach(AnimationMode.allCases) { mode in
                                        Text(mode.rawValue).tag(mode)
                                    }
                                }
                                .pickerStyle(MenuPickerStyle())
                            }
                            
                            VStack {
                                TextFieldWithValidation(title: "Inhale Duration (s)", value: $inhaleDuration, formatter: createNumberFormatter(limits: (min: 0.5, max: nil)), minimumValue: 0.5)
                                
                                TextFieldWithValidation(title: "Post-Inhale Hold (s)", value: $postInhaleHoldDuration, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0)
                                
                                TextFieldWithValidation(title: "Exhale Duration (s)", value: $exhaleDuration, formatter: createNumberFormatter(limits: (min: 0.5, max: nil)), minimumValue: 0.5)
                                
                                TextFieldWithValidation(title: "Post-Exhale Hold (s)", value: $postExhaleHoldDuration, formatter: createNumberFormatter(limits: (min: 0, max: nil)), minimumValue: 0)
                                
                                TextFieldWithValidation(title: "Drift", value: $drift, formatter: createNumberFormatter(limits: (min: 0.5, max: nil)), minimumValue: 0.5)
                                
                                TextFieldWithValidation(title: "Overlay Opacity", value: $overlayOpacity, formatter: createNumberFormatter(limits: (min: 0, max: 1)), minimumValue: 0.0)
                            }
                        }
                    }
                }
                .foregroundColor(.white)
                .padding()
                .background(Color.black.opacity(0.9))
                .cornerRadius(10)
                .shadow(radius: 10)
                .frame(width: 600)
                
                Spacer()
            }
            
            Spacer()
        }
        .padding(.top)
    }
}
