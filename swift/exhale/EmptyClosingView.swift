// EmptyClosingView.swift
import SwiftUI

struct EmptyClosingView: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = SelfClosingView()
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) {
    }

    class SelfClosingView: NSView {
        override func viewWillMove(toWindow newWindow: NSWindow?) {
            super.viewWillMove(toWindow: newWindow)
            
            // Ensure the closure happens before the window is displayed
            DispatchQueue.main.async {
                guard let window = newWindow else { return }
                
                // 1. Make the window fully transparent
                window.alphaValue = 0
                
                // 2. Move the window offscreen
                let offscreenOrigin = NSPoint(x: -2000, y: -2000)
                window.setFrameOrigin(offscreenOrigin)
                
                // 3. Close the window immediately
                window.close()
            }
        }
    }
}
