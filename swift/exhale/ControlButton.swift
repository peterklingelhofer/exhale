// ControlButton.swift
import SwiftUI

struct ControlButton: View {
    let systemImageName: String
    let title: String
    let action: () -> Void
    let keyboardShortcut: KeyEquivalent
    let modifiers: EventModifiers
    let helpText: String

    @State private var isPressed: Bool = false

    var body: some View {
        Button(action: action) {
            VStack {
                Image(systemName: systemImageName)
                    .imageScale(.large)
                    .frame(width: 24, height: 24)
                Text(title)
                    .font(.caption)
            }
        }
        .buttonStyle(PlainButtonStyle())
        .opacity(isPressed ? 0.6 : 1.0)
        .onLongPressGesture(minimumDuration: 0.0, pressing: { pressing in
            withAnimation(.easeInOut(duration: 0.1)) {
                self.isPressed = pressing
            }
        }, perform: {})
        .keyboardShortcut(keyboardShortcut, modifiers: modifiers)
        .help(helpText)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(Text(title))
        .accessibilityHint(Text(helpText))
    }
}

struct ControlButton_Previews: PreviewProvider {
    static var previews: some View {
        ControlButton(
            systemImageName: "play.circle.fill",
            title: "Start",
            action: { print("Start tapped") },
            keyboardShortcut: "a",
            modifiers: [.control, .shift],
            helpText: "Start the app and re-initialize animation."
        )
        .previewLayout(.sizeThatFits)
        .padding()
    }
}
