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

    var onStateChanged: (() -> Void)?

    private let bridge = BridgeClient()
    private var timer: Timer?

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

    func reloadAll() {
        do {
            settings = try bridge.loadSettings()
            devices = try bridge.listInputDevices()
            modelStatus = try bridge.getModelStatus()
            diagnostics = try bridge.runPermissionDiagnostics()
            runtime = try bridge.getRuntimeStatus()
            bridgeError = nil
            isDirty = false
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

    func saveSettings() {
        do {
            _ = try bridge.saveSettings(settings)
            isDirty = false
            reloadAll()
        } catch {
            publish(error)
        }
    }

    func completeOnboarding() {
        settings.onboardingCompleted = true
        saveSettings()
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
        bridgeError = error.localizedDescription
        onStateChanged?()
    }
}
