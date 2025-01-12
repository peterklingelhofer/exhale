// CombinedStepperTextField.swift
import SwiftUI

struct CombinedStepperTextField: View {
    var title: String
    @Binding var value: Double
    var formatter: NumberFormatter
    var minimumValue: Double = 0.0 // Default min
    var maximumValue: Double?
    var step: Double = 1.0 // Default step value
    
    var body: some View {
        HStack {
            Text(title)
                .frame(width: 224, alignment: .leading)
            
            Spacer()
            
            // TextField for manual input
            TextField("", value: $value, formatter: formatter)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .frame(width: 60)
                .onChange(of: value) { newValue in
                    value = validateValue(value: newValue, minimumValue: minimumValue, maximumValue: maximumValue, formatter: formatter)
                }
            
            // Stepper for increment/decrement
            Stepper("", value: $value, in: minimumValue...(maximumValue ?? Double.infinity), step: step)
                .labelsHidden()
                .frame(width: 0)
                .onChange(of: value) { _ in
                    // Optional: Additional actions on value change
                }
        }
    }
}

// Validation Function Updated to Include Maximum Value
func validateValue(value: Double, minimumValue: Double, maximumValue: Double?, formatter: NumberFormatter) -> Double {
    var updatedValue = value
    if updatedValue < minimumValue {
        updatedValue = minimumValue
    }
    if let max = maximumValue, updatedValue > max {
        updatedValue = max
    }
    return updatedValue
}
