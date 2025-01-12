// CombinedStepperTextField.swift
import SwiftUI

var defaultMin = 0.0

struct CombinedStepperTextField: View {
    var title: String
    @Binding var value: Double
    var limits: (min: Double?, max: Double?)
    var step: Double = 1.0

    // MARK: - Number Formatter
    private var formatter: NumberFormatter {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.maximumFractionDigits = 3
        formatter.minimum = NSNumber(value: limits.min ?? defaultMin)
        if let max = limits.max {
            formatter.maximum = NSNumber(value: max)
        }
        formatter.usesGroupingSeparator = false
        return formatter
    }

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
                    value = validateValue(
                        value: newValue,
                        minimumValue: limits.min ?? defaultMin,
                        maximumValue: limits.max
                    )
                }

            // Stepper for increment/decrement
            Stepper("", value: $value, in: (limits.min ?? defaultMin)...(limits.max ?? Double.infinity), step: step)
                .labelsHidden()
                .frame(width: 16, alignment: .leading)
        }
    }
}

func validateValue(value: Double, minimumValue: Double = defaultMin, maximumValue: Double?) -> Double {
    var updatedValue = value
    if updatedValue < minimumValue {
        updatedValue = minimumValue
    }
    if let max = maximumValue, updatedValue > max {
        updatedValue = max
    }
    return updatedValue
}
