// MetalOverlayRenderer.swift
import Combine
import Foundation
import Metal
import MetalKit
import SwiftUI

final class MetalOverlayRenderer: NSObject, MTKViewDelegate {
    private let device: MTLDevice
    private let commandQueue: MTLCommandQueue
    private let pipelineState: MTLRenderPipelineState
    private let settingsModel: SettingsModel

    private var viewportSize: SIMD2<Float> = .zero
    private var maxCircleScale: Float = 1

    private var uniforms = OverlayUniforms()

    private let breathingController: MetalBreathingController
    private var subscriptions = Set<AnyCancellable>()

    private var cachedBackgroundColorFloat4: SIMD4<Float> = SIMD4<Float>(0, 0, 0, 0)
    private var cachedInhaleColorFloat4: SIMD4<Float> = SIMD4<Float>(1, 0, 0, 1)
    private var cachedExhaleColorFloat4: SIMD4<Float> = SIMD4<Float>(0, 0, 1, 1)

    private var uniformBuffers: [MTLBuffer] = []
    private var uniformBufferIndex: Int = 0
    private let uniformBufferCount: Int = 3

    init(device: MTLDevice, metalView: MTKView, settingsModel: SettingsModel) {
        self.device = device
        self.settingsModel = settingsModel

        guard let commandQueue = device.makeCommandQueue() else {
            fatalError("Failed to create command queue")
        }
        self.commandQueue = commandQueue

        let library = try! device.makeDefaultLibrary(bundle: .main)
        let vertexFunction = library.makeFunction(name: "overlayVertex")!
        let fragmentFunction = library.makeFunction(name: "overlayFragment")!

        let pipelineDescriptor = MTLRenderPipelineDescriptor()
        pipelineDescriptor.vertexFunction = vertexFunction
        pipelineDescriptor.fragmentFunction = fragmentFunction
        pipelineDescriptor.colorAttachments[0].pixelFormat = metalView.colorPixelFormat
        pipelineDescriptor.colorAttachments[0].isBlendingEnabled = true

        pipelineDescriptor.colorAttachments[0].rgbBlendOperation = .add
        pipelineDescriptor.colorAttachments[0].alphaBlendOperation = .add
        pipelineDescriptor.colorAttachments[0].sourceRGBBlendFactor = .one
        pipelineDescriptor.colorAttachments[0].sourceAlphaBlendFactor = .one
        pipelineDescriptor.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha
        pipelineDescriptor.colorAttachments[0].destinationAlphaBlendFactor = .oneMinusSourceAlpha

        self.pipelineState = try! device.makeRenderPipelineState(descriptor: pipelineDescriptor)

        self.breathingController = MetalBreathingController(settingsModel: settingsModel)

        super.init()

        cachedBackgroundColorFloat4 = settingsModel.cachedBackgroundColorWithoutAlpha.toFloat4Cached()
        cachedInhaleColorFloat4 = settingsModel.inhaleColor.toFloat4Cached()
        cachedExhaleColorFloat4 = settingsModel.exhaleColor.toFloat4Cached()

        breathingController.requestDraw = { [weak metalView] in
            guard let metalView else { return }

            DispatchQueue.main.async {
                guard let window = metalView.window else { return }
                guard window.isVisible else { return }
                guard window.occlusionState.contains(.visible) else { return }
                metalView.setNeedsDisplay(metalView.bounds)
            }
        }

        settingsModel.$backgroundColor
            .receive(on: RunLoop.main)
            .sink { [weak self, weak metalView] _ in
                guard let self else { return }
                self.cachedBackgroundColorFloat4 = self.settingsModel.cachedBackgroundColorWithoutAlpha.toFloat4Cached()
                metalView?.setNeedsDisplay(metalView?.bounds ?? .zero)
            }
            .store(in: &subscriptions)

        settingsModel.$inhaleColor
            .receive(on: RunLoop.main)
            .sink { [weak self, weak metalView] newColor in
                guard let self else { return }
                self.cachedInhaleColorFloat4 = newColor.toFloat4Cached()
                metalView?.setNeedsDisplay(metalView?.bounds ?? .zero)
            }
            .store(in: &subscriptions)

        settingsModel.$exhaleColor
            .receive(on: RunLoop.main)
            .sink { [weak self, weak metalView] newColor in
                guard let self else { return }
                self.cachedExhaleColorFloat4 = newColor.toFloat4Cached()
                metalView?.setNeedsDisplay(metalView?.bounds ?? .zero)
            }
            .store(in: &subscriptions)

        Publishers.CombineLatest(settingsModel.$isAnimating, settingsModel.$isPaused)
            .receive(on: RunLoop.main)
            .sink { [weak self, weak metalView] _, _ in
                self?.breathingController.startIfNeeded()
                metalView?.setNeedsDisplay(metalView?.bounds ?? .zero)
            }
            .store(in: &subscriptions)

        breathingController.startIfNeeded()
        metalView.setNeedsDisplay(metalView.bounds)
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
        viewportSize = SIMD2<Float>(Float(size.width), Float(size.height))
        maxCircleScale = Self.getMaxCircleScaleForCurrentScreen()
    }

    func draw(in view: MTKView) {
        if let window = view.window, window.occlusionState.contains(.visible) == false {
            return
        }

        guard let drawable = view.currentDrawable,
              let renderPassDescriptor = view.currentRenderPassDescriptor
        else {
            return
        }

        let breathingState = breathingController.getCurrentState()

        uniforms.viewportSize = viewportSize
        uniforms.overlayOpacity = Float(settingsModel.overlayOpacity)
        uniforms.backgroundOpacity = Float(min(settingsModel.cachedBackgroundAlphaComponent, settingsModel.overlayOpacity))
        uniforms.maxCircleScale = maxCircleScale

        uniforms.shape = settingsModel.shape.metalValue
        uniforms.gradientMode = settingsModel.colorFillGradient.metalValue

        uniforms.phase = breathingState.phase.metalValue
        uniforms.progress = breathingState.progress

        uniforms.rectangleScale = settingsModel.colorFillGradient == .on ? 2.0 : 1.0
        uniforms.circleGradientScale = settingsModel.colorFillGradient == .on ? 2.0 : 1.0

        uniforms.backgroundColor = cachedBackgroundColorFloat4
        uniforms.inhaleColor = cachedInhaleColorFloat4
        uniforms.exhaleColor = cachedExhaleColorFloat4

        let bufferLength = MemoryLayout<OverlayUniforms>.stride

        if uniformBuffers.isEmpty {
            uniformBuffers = (0..<uniformBufferCount).compactMap { _ in
                device.makeBuffer(
                    length: bufferLength,
                    options: [.storageModeShared, .cpuCacheModeWriteCombined]
                )
            }

            if uniformBuffers.count != uniformBufferCount {
                return
            }

            uniformBufferIndex = 0
        }

        uniformBufferIndex = (uniformBufferIndex + 1) % uniformBufferCount
        let uniformBuffer = uniformBuffers[uniformBufferIndex]
        memcpy(uniformBuffer.contents(), &uniforms, bufferLength)

        guard let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: renderPassDescriptor)
        else {
            return
        }

        encoder.setRenderPipelineState(pipelineState)

        // Vertex shader does not require uniforms
        encoder.setFragmentBuffer(uniformBuffer, offset: 0, index: 0)
        encoder.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 3)

        encoder.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

    private static func getMaxCircleScaleForCurrentScreen() -> Float {
        guard let screen = NSScreen.main else { return 1 }
        let w = Float(screen.frame.width)
        let h = Float(screen.frame.height)
        return max(w, h) / min(w, h)
    }
}

private extension AnimationShape {
    var metalValue: UInt32 {
        switch self {
        case .fullscreen: return 0
        case .rectangle: return 1
        case .circle: return 2
        }
    }
}

private extension ColorFillGradient {
    var metalValue: UInt32 {
        switch self {
        case .off: return 0
        case .inner: return 1
        case .on: return 2
        }
    }
}

private extension BreathingPhase {
    var metalValue: UInt32 {
        switch self {
        case .inhale: return 0
        case .holdAfterInhale: return 1
        case .exhale: return 2
        case .holdAfterExhale: return 3
        }
    }
}

private extension Color {
    func toFloat4Cached() -> SIMD4<Float> {
        guard let cg = self.cgColor,
              let ns = NSColor(cgColor: cg) else {
            return SIMD4<Float>(0, 0, 0, 0)
        }

        let rgb = ns.usingColorSpace(.deviceRGB) ?? ns
        return SIMD4<Float>(
            Float(rgb.redComponent),
            Float(rgb.greenComponent),
            Float(rgb.blueComponent),
            Float(rgb.alphaComponent)
        )
    }
}
