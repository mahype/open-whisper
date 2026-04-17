import AppKit
import Foundation
import SwiftUI

@MainActor
final class AppModel: ObservableObject {
    @Published var settings: AppSettings = .default
    @Published var devices: [DeviceDTO] = []
    @Published var modelStatus: ModelStatusDTO = .empty
    @Published var diagnostics: DiagnosticsDTO = .empty
    @Published var runtime: RuntimeStatusDTO = .empty
    @Published var bridgeError: String?
    @Published var onboardingStep: Int = 0
    @Published var isDirty = false
    @Published var isCapturingHotkey = false
    @Published var hotkeyCapturePreview = ""
    @Published var hotkeyCaptureError: String?

    var onStateChanged: (() -> Void)?

    private let bridge = BridgeClient()
    private var timer: Timer?
    private var hotkeyBeforeCapture = AppSettings.default.hotkey

    init() {
        reloadAll()
        startPolling()
    }

    var modelDownloadProgress: Double? {
        guard let basisPoints = modelStatus.progressBasisPoints else {
            return nil
        }
        return Double(basisPoints) / 10_000.0
    }

    var hotkeyDisplayText: String {
        runtime.hotkeyRegistered ? runtime.hotkeyText : settings.hotkey
    }

    var hotkeyFieldTitle: String {
        isCapturingHotkey ? "Jetzt Tastenkombination druecken" : "Globaler Hotkey"
    }

    func binding<Value>(for keyPath: WritableKeyPath<AppSettings, Value>) -> Binding<Value> {
        Binding(
            get: { self.settings[keyPath: keyPath] },
            set: { newValue in
                self.settings[keyPath: keyPath] = newValue
                self.isDirty = true
            }
        )
    }

    func reloadAll() {
        do {
            settings = try bridge.loadSettings()
            devices = try bridge.listInputDevices()
            modelStatus = try bridge.getModelStatus()
            diagnostics = try bridge.runPermissionDiagnostics()
            runtime = try bridge.getRuntimeStatus()
            bridgeError = nil
            isDirty = false
            isCapturingHotkey = false
            hotkeyCapturePreview = ""
            hotkeyCaptureError = nil
            hotkeyBeforeCapture = settings.hotkey
            onStateChanged?()
        } catch {
            publish(error)
        }
    }

    func poll() {
        do {
            runtime = try bridge.getRuntimeStatus()
            modelStatus = try bridge.getModelStatus()
            bridgeError = nil
            onStateChanged?()
        } catch {
            publish(error)
        }
    }

    func refreshDevices() {
        do {
            devices = try bridge.listInputDevices()
            if !devices.contains(where: { $0.name == settings.inputDeviceName }) && !devices.isEmpty {
                settings.inputDeviceName = devices[0].name
                isDirty = true
            }
            bridgeError = nil
        } catch {
            publish(error)
        }
    }

    func refreshDiagnostics() {
        do {
            diagnostics = try bridge.runPermissionDiagnostics()
            bridgeError = nil
        } catch {
            publish(error)
        }
    }

    @discardableResult
    func saveSettings() -> Bool {
        do {
            let normalizedHotkey = try bridge.validateHotkey(settings.hotkey)
            settings.hotkey = normalizedHotkey
            _ = try bridge.saveSettings(settings)
            isDirty = false
            reloadAll()
            return true
        } catch {
            publish(error)
            return false
        }
    }

    func completeOnboarding() -> Bool {
        settings.onboardingCompleted = true
        return saveSettings()
    }

    func reopenOnboarding() {
        onboardingStep = 0
    }

    func choosePreset(_ preset: ModelPreset) {
        let previousFilename = URL(fileURLWithPath: settings.localModelPath).lastPathComponent
        let previousDefaults = Set(ModelPreset.allCases.map(\.defaultFilename))

        settings.localModel = preset
        if settings.localModelPath.isEmpty || previousDefaults.contains(previousFilename) {
            let basePath = modelStatus.path.isEmpty ? settings.localModelPath : modelStatus.path
            if !basePath.isEmpty {
                let newURL = URL(fileURLWithPath: basePath).deletingLastPathComponent().appendingPathComponent(preset.defaultFilename)
                settings.localModelPath = newURL.path
            } else {
                settings.localModelPath = ""
            }
        }

        isDirty = true
    }

    func startHotkeyCapture() {
        hotkeyBeforeCapture = settings.hotkey
        hotkeyCapturePreview = settings.hotkey
        hotkeyCaptureError = nil
        isCapturingHotkey = true
    }

    func updateHotkeyCapturePreview(_ value: String) {
        hotkeyCapturePreview = value
        hotkeyCaptureError = nil
    }

    func commitCapturedHotkey(_ hotkey: String) {
        do {
            let normalized = try bridge.validateHotkey(hotkey)
            settings.hotkey = normalized
            hotkeyCapturePreview = normalized
            hotkeyCaptureError = nil
            isCapturingHotkey = false
            isDirty = true
        } catch {
            failHotkeyCapture(error.localizedDescription)
        }
    }

    func cancelHotkeyCapture() {
        settings.hotkey = hotkeyBeforeCapture
        hotkeyCapturePreview = ""
        hotkeyCaptureError = nil
        isCapturingHotkey = false
    }

    func clearHotkeyCapture() {
        settings.hotkey = hotkeyBeforeCapture
        hotkeyCapturePreview = ""
        hotkeyCaptureError = "Open Whisper braucht einen globalen Hotkey mit Zusatztaste."
        isCapturingHotkey = false
    }

    func failHotkeyCapture(_ message: String) {
        hotkeyCaptureError = message
    }

    func startModelDownload() {
        do {
            _ = try bridge.startModelDownload(preset: settings.localModel)
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func deleteModel() {
        do {
            _ = try bridge.deleteModel(preset: settings.localModel)
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func toggleDictation() {
        do {
            if runtime.isRecording {
                _ = try bridge.stopDictation()
            } else {
                _ = try bridge.startDictation()
            }
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func openSystemSettings() {
        let candidates = [
            "/System/Applications/System Settings.app",
            "/System/Applications/System Preferences.app",
        ]

        for candidate in candidates where FileManager.default.fileExists(atPath: candidate) {
            NSWorkspace.shared.open(URL(fileURLWithPath: candidate))
            return
        }
    }

    private func startPolling() {
        timer?.invalidate()
        timer = Timer.scheduledTimer(withTimeInterval: 0.35, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.poll()
            }
        }
    }

    private func publish(_ error: Error) {
        isCapturingHotkey = false
        bridgeError = error.localizedDescription
        onStateChanged?()
    }
}
