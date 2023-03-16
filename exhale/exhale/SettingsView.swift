//  SettingsView.swift
import SwiftUI

struct SettingsView: View {
    @Binding var showSettings: Bool
    @Binding var overlayColor: Color
    @Binding var inhaleDuration: Double
    @Binding var postInhaleHoldDuration: Double
    @Binding var exhaleDuration: Double
    @Binding var postExhaleHoldDuration: Double
    @Binding var overlayOpacity: Double
    
    var body: some View {
        VStack {
            Spacer()
            
            HStack {
                Spacer()
                
                VStack {
                    Button(action: {
                        withAnimation {
                            showSettings.toggle()
                        }
                    }) {
                        Image(systemName: "gear")
                            .font(.system(size: 24))
                            .foregroundColor(.black)
                    }
                    .padding(.bottom)
                    
                    VStack(alignment: .leading, spacing: 10) {
                        ColorPicker("Overlay Color", selection: $overlayColor, supportsOpacity: true)
                        
                        Text("Inhale Duration (s)")
                        Slider(value: $inhaleDuration, in: 1...10)
                        
                        Text("Post-Inhale Hold Duration (s)")
                        Slider(value: $postInhaleHoldDuration, in: 1...10)
                        
                        Text("Exhale Duration (s)")
                        Slider(value: $exhaleDuration, in: 1...10)
                        
                        Text("Post-Exhale Hold Duration (s)")
                        Slider(value: $postExhaleHoldDuration, in: 1...10)
                        

                        Slider(value: $overlayOpacity, in: 0...1)
                    }
                    .padding(.horizontal)
                }
                .padding()
                .background(Color.white.opacity(0.9))
                .cornerRadius(10)
                .shadow(radius: 10)
                
                Spacer()
            }
            
            Spacer()
        }
        .padding(.top)
    }
}
