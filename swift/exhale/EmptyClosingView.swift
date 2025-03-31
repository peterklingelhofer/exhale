// EmptyClosingView.swift
import SwiftUI

struct EmptyClosingView: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = NSView(frame: .zero)
        DispatchQueue.main.async {
            if let window = view.window {
                window.level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.desktopIconWindow)))
                window.orderBack(nil)
                window.isRestorable = false
                window.close()
            }
        }
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) { }
}
