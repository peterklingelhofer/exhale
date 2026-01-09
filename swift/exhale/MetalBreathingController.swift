// MetalBreathingController.swift
import CoreVideo
import Foundation
import QuartzCore

struct MetalBreathingState {
    var phase: BreathingPhase
    var progress: Float
}

final class MetalBreathingController {
    private let settingsModel: SettingsModel

    private var displayLink: CVDisplayLink?
    private let stateQueue = DispatchQueue(label: "exhale.metalBreathingController.stateQueue")

    private var cycleCount: Int = 0
    private var currentPhase: BreathingPhase = .inhale
    private var phaseStartTime: CFTimeInterval = 0
    private var phaseDuration: CFTimeInterval = 1

    private var didRenderThisHold: Bool = false
    private var easingTable: [Float] = []

    private var lastDrawRequestTime: CFTimeInterval = 0
    private var lastDrawnPhase: BreathingPhase = .inhale
    private var lastDrawnProgress: Float = -1

    // Cap at 24fps to reduce CPU spikes but remain visually smooth
    private let maximumDrawIntervalFast: CFTimeInterval = 1.0 / 24.0
    private let maximumDrawIntervalSlow: CFTimeInterval = 1.0 / 14.0

    // Adaptive redraw thresholds (hysteresis)
    // - If delta exceeds enterFastThreshold, go "fast" (up to 24fps)
    // - If delta drops below exitFastThreshold, return to "slow"
    private let enterFastThreshold: Float = 0.006
    private let exitFastThreshold: Float = 0.0035

    // Minimum delta required to draw at all (prevents noise)
    private let minimumProgressDelta: Float = 0.0025

    private var isFastCadenceEnabled: Bool = false

    // Called from the CVDisplayLink thread
    var requestDraw: (() -> Void)?

    init(settingsModel: SettingsModel) {
        self.settingsModel = settingsModel
        easingTable = Self.buildEasingTable(sampleCount: 1024, x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0)
    }

    func startIfNeeded() {
        let shouldRun = settingsModel.isAnimating || settingsModel.isPaused
        if shouldRun {
            start()
        } else {
            stop()
        }
    }

    func start() {
        stateQueue.sync {
            cycleCount = 0
            currentPhase = .inhale
            phaseStartTime = CACurrentMediaTime()
            phaseDuration = getDurationForInhale()

            didRenderThisHold = false

            lastDrawRequestTime = 0
            lastDrawnPhase = currentPhase
            lastDrawnProgress = -1

            isFastCadenceEnabled = false
        }

        stop()

        var newDisplayLink: CVDisplayLink?
        CVDisplayLinkCreateWithActiveCGDisplays(&newDisplayLink)
        guard let createdDisplayLink = newDisplayLink else {
            return
        }

        let callback: CVDisplayLinkOutputCallback = { _, _, _, _, _, userInfo in
            guard let userInfo else { return kCVReturnError }
            let controller = Unmanaged<MetalBreathingController>.fromOpaque(userInfo).takeUnretainedValue()
            controller.step()
            return kCVReturnSuccess
        }

        CVDisplayLinkSetOutputCallback(
            createdDisplayLink,
            callback,
            Unmanaged.passUnretained(self).toOpaque()
        )

        displayLink = createdDisplayLink
        CVDisplayLinkStart(createdDisplayLink)
    }

    func stop() {
        if let displayLink {
            CVDisplayLinkStop(displayLink)
            self.displayLink = nil
        }
    }

    func getCurrentState() -> MetalBreathingState {
        stateQueue.sync {
            computeCurrentState(now: CACurrentMediaTime())
        }
    }

    // MARK: - Private

    private func step() {
        let now = CACurrentMediaTime()

        if !settingsModel.isAnimating && !settingsModel.isPaused {
            stop()
            return
        }

        let shouldRequestDraw: Bool = stateQueue.sync {
            // Pause mode: tint is static, but redraw occasionally to reflect settings changes
            if settingsModel.isPaused {
                if now - lastDrawRequestTime < 1.0 {
                    return false
                }
                lastDrawRequestTime = now
                let state = computeCurrentState(now: now)
                lastDrawnPhase = state.phase
                lastDrawnProgress = state.progress
                isFastCadenceEnabled = false
                return true
            }

            let isHoldPhase = (currentPhase == .holdAfterInhale) || (currentPhase == .holdAfterExhale)

            if isHoldPhase {
                let elapsed = now - phaseStartTime
                if elapsed >= phaseDuration {
                    advancePhase(now: now)
                    didRenderThisHold = false
                    isFastCadenceEnabled = false
                    return true
                }

                if didRenderThisHold {
                    return false
                }

                didRenderThisHold = true
                lastDrawRequestTime = now
                let state = computeCurrentState(now: now)
                lastDrawnPhase = state.phase
                lastDrawnProgress = state.progress
                return true
            }

            // Advance phase timing
            let elapsed = now - phaseStartTime
            if elapsed >= phaseDuration {
                advancePhase(now: now)
                didRenderThisHold = false
                isFastCadenceEnabled = false
            }

            let currentState = computeCurrentState(now: now)

            // Always draw on phase change
            if currentState.phase != lastDrawnPhase {
                if now - lastDrawRequestTime < maximumDrawIntervalFast {
                    return false
                }
                lastDrawRequestTime = now
                lastDrawnPhase = currentState.phase
                lastDrawnProgress = currentState.progress
                return true
            }

            if lastDrawnProgress < 0 {
                if now - lastDrawRequestTime < maximumDrawIntervalFast {
                    return false
                }
                lastDrawRequestTime = now
                lastDrawnPhase = currentState.phase
                lastDrawnProgress = currentState.progress
                return true
            }

            let delta = abs(currentState.progress - lastDrawnProgress)

            if delta < minimumProgressDelta {
                return false
            }

            // Hysteresis for cadence mode
            if isFastCadenceEnabled {
                if delta < exitFastThreshold {
                    isFastCadenceEnabled = false
                }
            } else {
                if delta > enterFastThreshold {
                    isFastCadenceEnabled = true
                }
            }

            let cadenceInterval = isFastCadenceEnabled ? maximumDrawIntervalFast : maximumDrawIntervalSlow

            if now - lastDrawRequestTime < cadenceInterval {
                return false
            }

            lastDrawRequestTime = now
            lastDrawnPhase = currentState.phase
            lastDrawnProgress = currentState.progress
            return true
        }

        if shouldRequestDraw {
            requestDraw?()
        }
    }

    private func computeCurrentState(now: CFTimeInterval) -> MetalBreathingState {
        let elapsed = now - phaseStartTime
        let rawT = phaseDuration > 0 ? min(max(elapsed / phaseDuration, 0), 1) : 1
        let easedT = getEasedT(rawT: rawT)

        switch currentPhase {
        case .inhale:
            return MetalBreathingState(phase: .inhale, progress: Float(easedT))
        case .holdAfterInhale:
            return MetalBreathingState(phase: .holdAfterInhale, progress: 1)
        case .exhale:
            return MetalBreathingState(phase: .exhale, progress: Float(1 - easedT))
        case .holdAfterExhale:
            return MetalBreathingState(phase: .holdAfterExhale, progress: 0)
        }
    }

    private func advancePhase(now: CFTimeInterval) {
        switch currentPhase {
        case .inhale:
            currentPhase = .holdAfterInhale
            phaseStartTime = now
            phaseDuration = getDurationForHoldAfterInhale()

        case .holdAfterInhale:
            currentPhase = .exhale
            phaseStartTime = now
            phaseDuration = getDurationForExhale()

        case .exhale:
            currentPhase = .holdAfterExhale
            phaseStartTime = now
            phaseDuration = getDurationForHoldAfterInhale()

        case .holdAfterExhale:
            cycleCount += 1
            currentPhase = .inhale
            phaseStartTime = now
            phaseDuration = getDurationForInhale()
        }
    }

    private func getDurationForInhale() -> CFTimeInterval {
        var duration = settingsModel.inhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingInhale > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingInhale...settingsModel.randomizedTimingInhale)
        }
        return max(duration, 0.1)
    }

    private func getDurationForHoldAfterInhale() -> CFTimeInterval {
        var duration = settingsModel.postInhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingPostInhaleHold > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingPostInhaleHold...settingsModel.randomizedTimingPostInhaleHold)
        }
        return max(duration, 0.1)
    }

    private func getDurationForExhale() -> CFTimeInterval {
        var duration = settingsModel.exhaleDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingExhale > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingExhale...settingsModel.randomizedTimingExhale)
        }
        return max(duration, 0.1)
    }

    private func getDurationForHoldAfterExhale() -> CFTimeInterval {
        var duration = settingsModel.postExhaleHoldDuration * pow(settingsModel.drift, Double(cycleCount))
        if settingsModel.randomizedTimingPostExhaleHold > 0 {
            duration += Double.random(in: -settingsModel.randomizedTimingPostExhaleHold...settingsModel.randomizedTimingPostExhaleHold)
        }
        return max(duration, 0.1)
    }

    private func getEasedT(rawT: Double) -> Double {
        if settingsModel.animationMode == .linear {
            return rawT
        }

        let indexFloat = rawT * Double(easingTable.count - 1)
        let lowerIndex = max(0, min(easingTable.count - 2, Int(indexFloat)))
        let fraction = Float(indexFloat - Double(lowerIndex))

        let a = easingTable[lowerIndex]
        let b = easingTable[lowerIndex + 1]
        return Double(a + (b - a) * fraction)
    }

    private static func buildEasingTable(sampleCount: Int, x1: Double, y1: Double, x2: Double, y2: Double) -> [Float] {
        var table: [Float] = []
        table.reserveCapacity(sampleCount)

        for i in 0..<sampleCount {
            let t = Double(i) / Double(sampleCount - 1)
            let value = CubicBezierEaseInOut.getValue(t: t, x1: x1, y1: y1, x2: x2, y2: y2)
            table.append(Float(value))
        }

        return table
    }
}

enum CubicBezierEaseInOut {
    static func getValue(t: Double, x1: Double, y1: Double, x2: Double, y2: Double) -> Double {
        let epsilon = 1e-6
        var tPrime = t

        for _ in 0..<8 {
            let x = cubic(t: tPrime, a1: x1, a2: x2) - t
            if abs(x) < epsilon {
                break
            }
            let dx = cubicDerivative(t: tPrime, a1: x1, a2: x2)
            if abs(dx) < 1e-6 {
                break
            }
            tPrime -= x / dx
            tPrime = min(max(tPrime, 0), 1)
        }

        return cubic(t: tPrime, a1: y1, a2: y2)
    }

    private static func cubic(t: Double, a1: Double, a2: Double) -> Double {
        let c = 3.0 * a1
        let b = 3.0 * (a2 - a1) - c
        let a = 1.0 - c - b
        return ((a * t + b) * t + c) * t
    }

    private static func cubicDerivative(t: Double, a1: Double, a2: Double) -> Double {
        let c = 3.0 * a1
        let b = 3.0 * (a2 - a1) - c
        let a = 1.0 - c - b
        return (3.0 * a * t + 2.0 * b) * t + c
    }
}
