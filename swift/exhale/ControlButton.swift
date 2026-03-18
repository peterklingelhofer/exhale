// ControlButton.swift
import SwiftUI

struct ControlButton: View {
    let systemImageName: String
    let title: String
    let action: () -> Void
    let keyboardShortcut: KeyEquivalent
    let modifiers: EventModifiers
    let helpText: String

    @State private var isHovered: Bool = false
    @State private var isPressed: Bool = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                Image(systemName: systemImageName)
                    .imageScale(.medium)
                Text(title)
                    .font(.system(size: 12, weight: .medium))
            }
            .padding(.vertical, 6)
            .padding(.horizontal, 10)
            .frame(maxWidth: .infinity)
            .background(
                RoundedRectangle(cornerRadius: 7, style: .continuous)
                    .fill(Color.primary.opacity(isHovered ? 0.1 : 0.05))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 7, style: .continuous)
                    .strokeBorder(Color.primary.opacity(isHovered ? 0.2 : 0.12), lineWidth: 1)
            )
        }
        .buttonStyle(PlainButtonStyle())
        .opacity(isPressed ? 0.7 : 1.0)
        .scaleEffect(isPressed ? 0.97 : 1.0)
        .animation(.easeInOut(duration: 0.1), value: isPressed)
        .animation(.easeInOut(duration: 0.15), value: isHovered)
        .onHover { hovering in
            isHovered = hovering
        }
        .onLongPressGesture(minimumDuration: 0.0, pressing: { pressing in
            self.isPressed = pressing
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
