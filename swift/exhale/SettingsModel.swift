import SwiftUI
import Combine

struct SettingsModelTypes {
    enum ColorFillGradient: String, Codable {
        case on, off
    }
    
    enum AnimationShape: String, Codable {
        case fullscreen, circle, square
    }
    
    enum AnimationMode: String, Codable {
        case sinusoidal, linear
    }
}

class SettingsModel: ObservableObject {
    private var cancellables = Set<AnyCancellable>()
    private let defaults = UserDefaults.standard
    
    @Published var backgroundColor: Color {
        didSet {
            saveColor(backgroundColor, forKey: "backgroundColor")
        }
    }
    
    @Published var inhaleColor: Color {
        didSet {
            saveColor(inhaleColor, forKey: "inhaleColor")
        }
    }
    
    @Published var exhaleColor: Color {
        didSet {
            saveColor(exhaleColor, forKey: "exhaleColor")
        }
    }
    
    @Published var inhaleDuration: TimeInterval {
        didSet {
            defaults.set(inhaleDuration, forKey: "inhaleDuration")
        }
    }
    
    @Published var postInhaleHoldDuration: TimeInterval {
        didSet {
            defaults.set(postInhaleHoldDuration, forKey: "postInhaleHoldDuration")
        }
    }
    
    @Published var exhaleDuration: TimeInterval {
        didSet {
            defaults.set(exhaleDuration, forKey: "exhaleDuration")
        }
    }
    
    @Published var postExhaleHoldDuration: TimeInterval {
        didSet {
            defaults.set(postExhaleHoldDuration, forKey: "postExhaleHoldDuration")
        }
    }
    
    @Published var drift: Double {
        didSet {
            defaults.set(drift, forKey: "drift")
        }
    }
    
    @Published var overlayOpacity: Double {
        didSet {
            defaults.set(overlayOpacity, forKey: "overlayOpacity")
        }
    }

    @Published var colorFillGradient: ColorFillGradient {
        didSet {
            defaults.set(colorFillGradient.rawValue, forKey: "colorFillGradient")
        }
    }

    @Published var shape: AnimationShape {
        didSet {
            defaults.set(shape.rawValue, forKey: "shape")
        }
    }
    
    @Published var animationMode: AnimationMode {
        didSet {
            defaults.set(animationMode.rawValue, forKey: "animationMode")
        }
    }
    
    @Published var randomizedTimingInhale: Double {
        didSet {
            defaults.set(randomizedTimingInhale, forKey: "randomizedTimingInhale")
        }
    }
    
    @Published var randomizedTimingPostInhaleHold: Double {
        didSet {
            defaults.set(randomizedTimingPostInhaleHold, forKey: "randomizedTimingPostInhaleHold")
        }
    }
    
    @Published var randomizedTimingExhale: Double {
        didSet {
            defaults.set(randomizedTimingExhale, forKey: "randomizedTimingExhale")
        }
    }
    
    @Published var randomizedTimingPostExhaleHold: Double {
        didSet {
            defaults.set(randomizedTimingPostExhaleHold, forKey: "randomizedTimingPostExhaleHold")
        }
    }
    
    init() {
        self.backgroundColor = Color.black
        self.inhaleColor = Color.red
        self.exhaleColor = Color.blue
        
        self.colorFillGradient = .on
        self.inhaleDuration = 5
        self.postInhaleHoldDuration = 0
        self.exhaleDuration = 10
        self.postExhaleHoldDuration = 0
        self.drift = 1.01
        self.overlayOpacity = 0.25
        self.shape = .fullscreen
        self.animationMode = .sinusoidal
        self.randomizedTimingInhale = 0
        self.randomizedTimingPostInhaleHold = 0
        self.randomizedTimingExhale = 0
        self.randomizedTimingPostExhaleHold = 0
        
        self.backgroundColor = loadColor(forKey: "backgroundColor") ?? Color.black
        self.inhaleColor = loadColor(forKey: "inhaleColor") ?? Color(red: 1, green: 0, blue: 0)
        self.exhaleColor = loadColor(forKey: "exhaleColor") ?? Color(red: 0, green: 0, blue: 1)
        
        self.inhaleDuration = defaults.double(forKey: "inhaleDuration")
        self.postInhaleHoldDuration = defaults.double(forKey: "postInhaleHoldDuration")
        self.exhaleDuration = defaults.double(forKey: "exhaleDuration")
        self.postExhaleHoldDuration = defaults.double(forKey: "postExhaleHoldDuration")
        self.drift = defaults.double(forKey: "drift")
        self.overlayOpacity = defaults.double(forKey: "overlayOpacity")
        
        if let savedGradient = defaults.string(forKey: "colorFillGradient"),
           let gradient = ColorFillGradient(rawValue: savedGradient) {
            self.colorFillGradient = gradient
        } else {
            self.colorFillGradient = .on
        }
        
        if let savedShape = defaults.string(forKey: "shape"),
          let shape = AnimationShape(rawValue: savedShape) {
           self.shape = shape
       } else {
           self.shape = .fullscreen
       }
       
       if let savedMode = defaults.string(forKey: "animationMode"),
          let mode = AnimationMode(rawValue: savedMode) {
           self.animationMode = mode
       } else {
           self.animationMode = .sinusoidal
       }

        self.randomizedTimingInhale = defaults.double(forKey: "randomizedTimingInhale")
        self.randomizedTimingPostInhaleHold = defaults.double(forKey: "randomizedTimingPostInhaleHold")
        self.randomizedTimingExhale = defaults.double(forKey: "randomizedTimingExhale")
        self.randomizedTimingPostExhaleHold = defaults.double(forKey: "randomizedTimingPostExhaleHold")
    }
    
    private func saveColor(_ color: Color, forKey key: String) {
        let nsColor = NSColor(cgColor: color.cgColor!)
        guard let unwrappedColor = nsColor else { return }
        let data = try? NSKeyedArchiver.archivedData(withRootObject: unwrappedColor, requiringSecureCoding: false)
        defaults.set(data, forKey: key)
    }
        
    private func loadColor(forKey key: String) -> Color? {
        guard let data = defaults.object(forKey: key) as? Data else {
            return nil
        }
        do {
            if let nsColor = try NSKeyedUnarchiver.unarchivedObject(ofClasses: [NSColor.self], from: data) as? NSColor {
                return Color(nsColor)
            }
        } catch {
            print("Couldn't read file.")
        }
        return nil
    }

    func resetToDefaults() {
        self.backgroundColor = Color.black
        self.inhaleColor = Color.red
        self.exhaleColor = Color.blue
        self.inhaleDuration = 5
        self.postInhaleHoldDuration = 0
        self.exhaleDuration = 10
        self.postExhaleHoldDuration = 0
        self.drift = 1.01
        self.overlayOpacity = 0.25
        self.colorFillGradient = .on
        self.shape = .fullscreen
        self.animationMode = .sinusoidal
        self.randomizedTimingInhale = 0
        self.randomizedTimingPostInhaleHold = 0
        self.randomizedTimingExhale = 0
        self.randomizedTimingPostExhaleHold = 0
        
        let keys = ["backgroundColor", "inhaleColor", "exhaleColor", "inhaleDuration", "postInhaleHoldDuration", "exhaleDuration", "postExhaleHoldDuration", "drift", "overlayOpacity", "colorFillGradient", "shape", "animationMode", "randomizedTimingInhale", "randomizedTimingPostInhaleHold", "randomizedTimingExhale", "randomizedTimingPostExhaleHold"]
        for key in keys {
            defaults.removeObject(forKey: key)
        }
    }

}

extension SettingsModel {
    func convertToAppType(_ type: SettingsModelTypes.ColorFillGradient) -> ColorFillGradient {
        return ColorFillGradient(rawValue: type.rawValue) ?? .on
    }
    
    func convertToSettingsModelType(_ type: ColorFillGradient) -> SettingsModelTypes.ColorFillGradient {
        return SettingsModelTypes.ColorFillGradient(rawValue: type.rawValue) ?? .on
    }
    
    func convertToAppType(_ type: SettingsModelTypes.AnimationShape) -> AnimationShape {
        return AnimationShape(rawValue: type.rawValue) ?? .fullscreen
    }
    
    func convertToSettingsModelType(_ type: AnimationShape) -> SettingsModelTypes.AnimationShape {
        return SettingsModelTypes.AnimationShape(rawValue: type.rawValue) ?? .fullscreen
    }
    
    func convertToAppType(_ type: SettingsModelTypes.AnimationMode) -> AnimationMode {
        return AnimationMode(rawValue: type.rawValue) ?? .sinusoidal
    }
    
    func convertToSettingsModelType(_ type: AnimationMode) -> SettingsModelTypes.AnimationMode {
        return SettingsModelTypes.AnimationMode(rawValue: type.rawValue) ?? .sinusoidal
    }
}
