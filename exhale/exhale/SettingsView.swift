//  SettingsView.swift
import SwiftUI

struct SettingsView: View {
    @Binding var showSettings: Bool
    @Binding var overlayColor: Color
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
        formatter.usesGroupingSeparator = false
        return formatter
    }()
    
    var body: some View {
        VStack {
            Spacer()
            
            HStack {
                Spacer()
                
                VStack {
                    Form {
                        VStack(alignment: .leading, spacing: 10) {
                            ColorPicker("Overlay Color", selection: $overlayColor, supportsOpacity: true)
                            
                            HStack {
                                Text("Inhale Duration (s)")
                                Spacer()
                                TextField("", value: $inhaleDuration, formatter: positiveNumberFormatter)
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Post-Inhale Hold (s)")
                                Spacer()
                                TextField("", value: $postInhaleHoldDuration, formatter: positiveNumberFormatter)
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Exhale Duration (s)")
                                Spacer()
                                TextField("", value: $exhaleDuration, formatter: positiveNumberFormatter)
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Post-Exhale Hold (s)")
                                Spacer()
                                TextField("", value: $postExhaleHoldDuration, formatter: positiveNumberFormatter)
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Drift (s)")
                                Spacer()
                                TextField("", value: $drift, formatter: positiveNumberFormatter)
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                            
                            HStack {
                                Text("Overlay Opacity")
                                Spacer()
                                TextField("", value: $overlayOpacity, formatter: positiveNumberFormatter)
                                    .textFieldStyle(RoundedBorderTextFieldStyle())
                                    .frame(width: 100)
                            }
                        }
                        .foregroundColor(.black)
                        .padding(.horizontal)
                    }
                }
                .padding()
                .background(Color.white.opacity(0.9))
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
