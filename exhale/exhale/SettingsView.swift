//  SettingsView.swift
import SwiftUI

struct SettingsView: View {
    @Binding var showSettings: Bool
    @Binding var overlayColor: Color
    @Binding var backgroundColor: Color
    @Binding var inhaleDuration: Double
    @Binding var postInhaleHoldDuration: Double
    @Binding var exhaleDuration: Double
    @Binding var postExhaleHoldDuration: Double
    @Binding var drift: Double
    @Binding var overlayOpacity: Double
    
    let positiveNumberFormatter: NumberFormatter = {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.maximumFractionDigits = 2
        formatter.minimum = 0
        formatter.maximum = 1
        formatter.usesGroupingSeparator = false
        return formatter
    }()
    
    func createNumberFormatter(minimumValue: Double, maximumValue: Double? = nil) -> NumberFormatter {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.maximumFractionDigits = 2
        formatter.minimum = NSNumber(value: minimumValue)
        if let max = maximumValue {
            formatter.maximum = NSNumber(value: max)
        }
        formatter.usesGroupingSeparator = false
        return formatter
    }
    
    private func validate(value: inout Double, formatter: NumberFormatter, minimumValue: Double) {
        if value < minimumValue {
            value = minimumValue
        }
        if let maximum = formatter.maximum?.doubleValue,
           value > maximum {
            value = maximum
        }
    }
    
    var body: some View {
        VStack {
            Spacer()
            
            HStack {
                Spacer()
                
                VStack {
                    Form {
                        VStack(alignment: .leading, spacing: 10) {
                            ColorPicker("Overlay Color", selection: $overlayColor, supportsOpacity: true)
                            
                            ColorPicker("Background Color", selection: $backgroundColor, supportsOpacity: true)
                            
                            HStack {
                                Text("Inhale Duration (s)")
                                Spacer()
                                TextField("", value: $inhaleDuration, formatter: createNumberFormatter(minimumValue: 0.5))
                                    .onChange(of: inhaleDuration) { newValue in
                                        validate(value: &inhaleDuration, formatter: createNumberFormatter(minimumValue: 0.5), minimumValue: 0.5)
                                    }
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Post-Inhale Hold (s)")
                                Spacer()
                                TextField("", value: $postInhaleHoldDuration, formatter: positiveNumberFormatter)
                                    .onChange(of: postInhaleHoldDuration) { newValue in
                                        validate(value: &postInhaleHoldDuration, formatter: positiveNumberFormatter, minimumValue: 0)
                                    }
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Exhale Duration (s)")
                                Spacer()
                                TextField("", value: $exhaleDuration, formatter: createNumberFormatter(minimumValue: 0.5))
                                    .onChange(of: exhaleDuration) { newValue in
                                        validate(value: &exhaleDuration, formatter: createNumberFormatter(minimumValue: 0.5), minimumValue: 0.5)
                                    }
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Post-Exhale Hold (s)")
                                Spacer()
                                TextField("", value: $postExhaleHoldDuration, formatter: positiveNumberFormatter)
                                    .onChange(of: postExhaleHoldDuration) { newValue in
                                        validate(value: &postExhaleHoldDuration, formatter: positiveNumberFormatter, minimumValue: 0)
                                    }
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            
                            HStack {
                                Text("Drift")
                                Spacer()
                                TextField("", value: $drift, formatter: createNumberFormatter(minimumValue: 0.5))
                                    .onChange(of: drift) { newValue in
                                        validate(value: &drift, formatter: createNumberFormatter(minimumValue: 0.5), minimumValue: 0.5)
                                    }
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Overlay Opacity")
                                Spacer()
                                TextField("", value: $overlayOpacity, formatter: createNumberFormatter(minimumValue: 0, maximumValue: 1))
                                    .onChange(of: overlayOpacity) { newValue in
                                        validate(value: &overlayOpacity, formatter: createNumberFormatter(minimumValue: 0, maximumValue: 1), minimumValue: 0)
                                    }
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                        }
                        .foregroundColor(.white)
                        .padding(.horizontal)
                    }
                }
                .padding()
                .background(Color.black.opacity(0.9))
                .cornerRadius(10)
                .shadow(radius: 10)
                .frame(width: 300)
                
                Spacer()
            }
            
            Spacer()
        }
        .padding(.top)
    }
}
