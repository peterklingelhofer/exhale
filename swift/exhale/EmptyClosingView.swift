// EmptyClosingView.swift
import SwiftUI
import SwiftUI

struct EmptyClosingView: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = NSView(frame: .zero)
        // Dispatch asynchronously so that the view is attached to a window
        DispatchQueue.main.async {
            if let window = view.window {
                window.orderOut(nil)
                window.close()
            }
        }
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) { }
}
