// MetalOverlayView.swift
import Cocoa
import MetalKit

final class MetalOverlayView: NSView {
    private let metalView: MTKView
    private let renderer: MetalOverlayRenderer

    init(frame: CGRect, settingsModel: SettingsModel) {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not supported on this Mac")
        }

        metalView = MTKView(frame: .zero, device: device)
        metalView.enableSetNeedsDisplay = true
        metalView.isPaused = true
        metalView.preferredFramesPerSecond = 20
        metalView.clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 0)
        metalView.colorPixelFormat = .bgra8Unorm
        metalView.framebufferOnly = true
        metalView.wantsLayer = true
        metalView.layer?.isOpaque = false

        renderer = MetalOverlayRenderer(device: device, metalView: metalView, settingsModel: settingsModel)
        metalView.delegate = renderer

        super.init(frame: frame)

        wantsLayer = true
        layer?.isOpaque = false

        addSubview(metalView)
    }

    required init?(coder: NSCoder) {
        return nil
    }

    override func layout() {
        super.layout()
        metalView.frame = bounds
    }

    func requestDraw() {
        metalView.setNeedsDisplay(metalView.bounds)
    }

    func setFramesPerSecond(_ framesPerSecond: Int) {
        metalView.preferredFramesPerSecond = framesPerSecond
    }
}
