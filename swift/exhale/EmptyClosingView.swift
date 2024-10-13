// EmptyClosingView.swift
import SwiftUI

struct EmptyClosingView: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = SelfClosingView()
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) {}

    class SelfClosingView: NSView {
        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            DispatchQueue.main.async {
                if let window = self.window {
                    window.close()
                }
            }
        }
    }
}
