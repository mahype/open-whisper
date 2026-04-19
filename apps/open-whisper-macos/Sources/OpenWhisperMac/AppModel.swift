import AppKit
import Foundation
import SwiftUI

@MainActor
final class AppModel: ObservableObject {
    @Published var settings: AppSettings = .default
    @Published var devices: [DeviceDTO] = []
    @Published var modelStatus: ModelStatusDTO = .empty
    @Published var modelStatusList: [ModelStatusDTO] = []
    @Published var llmStatusList: [LlmModelStatusDTO] = []
    @Published var ollamaModels: [RemoteModelDTO] = []
    @Published var lmStudioModels: [RemoteModelDTO] = []
    @Published var ollamaModelsError: String?
    @Published var lmStudioModelsError: String?
    @Published var diagnostics: DiagnosticsDTO = .empty
    @Published var runtime: RuntimeStatusDTO = .empty
    @Published var bridgeError: String?
    @Published var onboardingStep: Int = 0
    @Published var selectedModeID: String = "standard"
    @Published var isCapturingHotkey = false
    @Published var hotkeyCapturePreview = ""
    @Published var hotkeyCaptureError: String?

    var onStateChanged: (() -> Void)?

    private let bridge = BridgeClient()
    private var timer: Timer?
    private var hotkeyBeforeCapture = AppSettings.default.hotkey
    private var persistedSettingsSnapshot: AppSettings = .default
    private var pendingAutoSaveTask: Task<Void, Never>?
    private static let autoSaveDebounceNanoseconds: UInt64 = 500_000_000

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

    var selectedLanguageCode: String {
        TranscriptionLanguageOption.option(for: settings.transcriptionLanguage)?.code ?? "auto"
    }

    var availableLanguageOptions: [TranscriptionLanguageOption] {
        if let current = TranscriptionLanguageOption.option(for: settings.transcriptionLanguage) {
            return TranscriptionLanguageOption.common.contains(current)
                ? TranscriptionLanguageOption.common
                : [current] + TranscriptionLanguageOption.common
        }
        return TranscriptionLanguageOption.common
    }

    var activeProviderLabel: String {
        runtime.providerSummary
    }

    var selectedModelDisplayName: String {
        settings.localModel.displayName
    }

    var selectedModelStatusText: String {
        if modelStatus.isDownloading {
            return "Download laeuft"
        }
        return modelStatus.isDownloaded ? "Bereit" : "Noch nicht geladen"
    }

    var selectedModelSizeText: String {
        if modelStatus.isDownloaded,
           let actual = actualModelFileSize() {
            return "\(Self.formatByteCount(actual)) (geladen)"
        }
        let expected = modelStatus.expectedSizeBytes == 0
            ? settings.localModel.downloadSizeBytes
            : modelStatus.expectedSizeBytes
        return "ca. \(Self.formatByteCount(expected)) (Download)"
    }

    private func actualModelFileSize() -> UInt64? {
        let path = modelStatus.path.isEmpty ? settings.localModelPath : modelStatus.path
        guard !path.isEmpty,
              let attrs = try? FileManager.default.attributesOfItem(atPath: path),
              let size = attrs[.size] as? UInt64 else {
            return nil
        }
        return size
    }

    private static func formatByteCount(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        formatter.allowedUnits = [.useMB, .useGB]
        formatter.includesUnit = true
        formatter.isAdaptive = true
        return formatter.string(fromByteCount: Int64(bytes))
    }

    var trayModelLabel: String {
        let name = modelStatus.presetLabel.isEmpty ? selectedModelDisplayName : modelStatus.presetLabel
        if modelStatus.isDownloading {
            return "Modell: \(name) (Download laeuft)"
        }
        return "Modell: \(name)"
    }

    var hotkeyRiskHint: String? {
        let source = isCapturingHotkey && !hotkeyCapturePreview.isEmpty ? hotkeyCapturePreview : settings.hotkey
        return isSingleKeyHotkey(source)
            ? "Eine einzelne globale Taste kann mit normaler Texteingabe kollidieren. Kombinationen bleiben sicherer."
            : nil
    }

    var availableModes: [ProcessingMode] {
        settings.modes
    }

    var activeMode: ProcessingMode {
        settings.modes.first(where: { $0.id == settings.activeModeId }) ?? settings.modes.first ?? .standard
    }

    var selectedMode: ProcessingMode {
        settings.modes.first(where: { $0.id == selectedModeID }) ?? activeMode
    }

    var activeModeName: String {
        runtime.activeModeName.isEmpty ? activeMode.name : runtime.activeModeName
    }

    var trayModeLabel: String {
        "Modus: \(activeModeName)"
    }

    var canDeleteSelectedMode: Bool {
        selectedMode.id != "standard"
    }

    var persistedModes: [ProcessingMode] {
        persistedSettingsSnapshot.modes
    }

    var persistedActiveModeID: String {
        persistedSettingsSnapshot.activeModeId
    }

    func binding<Value>(for keyPath: WritableKeyPath<AppSettings, Value>) -> Binding<Value> {
        Binding(
            get: { self.settings[keyPath: keyPath] },
            set: { newValue in
                self.settings[keyPath: keyPath] = newValue
                self.requestAutoSave()
            }
        )
    }

    func modeBinding<Value>(for keyPath: WritableKeyPath<ProcessingMode, Value>) -> Binding<Value> {
        Binding(
            get: {
                self.settings.modes.first(where: { $0.id == self.selectedModeID })?[keyPath: keyPath]
                    ?? self.activeMode[keyPath: keyPath]
            },
            set: { newValue in
                guard let index = self.settings.modes.firstIndex(where: { $0.id == self.selectedModeID }) else {
                    return
                }
                self.settings.modes[index][keyPath: keyPath] = newValue
                self.requestAutoSave()
            }
        )
    }

    func languageBinding() -> Binding<String> {
        Binding(
            get: { self.selectedLanguageCode },
            set: { newValue in
                self.settings.transcriptionLanguage = newValue == "auto" ? "auto" : newValue
                self.requestAutoSave()
            }
        )
    }

    var postProcessingChoiceBinding: Binding<PostProcessingChoice> {
        Binding(
            get: {
                switch self.settings.activePostProcessingBackend {
                case .local:
                    if !self.settings.activeCustomLlmId.isEmpty,
                       let entry = self.settings.customLlmModels.first(where: { $0.id == self.settings.activeCustomLlmId }) {
                        return .localCustom(id: entry.id, name: entry.name)
                    }
                    return .localPreset(self.settings.localLlm)
                case .ollama:
                    return .ollamaModel(self.settings.ollama.modelName)
                case .lmStudio:
                    return .lmStudioModel(self.settings.lmStudio.modelName)
                }
            },
            set: { newValue in
                switch newValue {
                case .localPreset(let preset):
                    self.settings.activePostProcessingBackend = .local
                    self.settings.activeCustomLlmId = ""
                    self.settings.localLlm = preset
                case .localCustom(let id, _):
                    self.settings.activePostProcessingBackend = .local
                    self.settings.activeCustomLlmId = id
                case .ollamaModel(let name):
                    self.settings.activePostProcessingBackend = .ollama
                    self.settings.activeCustomLlmId = ""
                    self.settings.ollama.modelName = name
                case .lmStudioModel(let name):
                    self.settings.activePostProcessingBackend = .lmStudio
                    self.settings.activeCustomLlmId = ""
                    self.settings.lmStudio.modelName = name
                }
                self.requestAutoSave()
            }
        )
    }

    var postProcessingChoices: [PostProcessingChoice] {
        var choices: [PostProcessingChoice] = LlmPreset.allCases.map { .localPreset($0) }

        choices.append(
            contentsOf: settings.customLlmModels.map { .localCustom(id: $0.id, name: $0.name) }
        )

        var ollamaNames = ollamaModels.map(\.name)
        let currentOllama = settings.ollama.modelName
        if !currentOllama.isEmpty && !ollamaNames.contains(currentOllama) {
            ollamaNames.insert(currentOllama, at: 0)
        }
        choices.append(contentsOf: ollamaNames.map { .ollamaModel($0) })

        var lmNames = lmStudioModels.map(\.name)
        let currentLmStudio = settings.lmStudio.modelName
        if !currentLmStudio.isEmpty && !lmNames.contains(currentLmStudio) {
            lmNames.insert(currentLmStudio, at: 0)
        }
        choices.append(contentsOf: lmNames.map { .lmStudioModel($0) })

        return choices
    }

    func addCustomLocalLlm(name: String, path: String) {
        let id = UUID().uuidString.lowercased()
        let entry = CustomLlmModel(
            id: id,
            name: name.trimmingCharacters(in: .whitespacesAndNewlines),
            source: .localPath(path: path)
        )
        settings.customLlmModels.append(entry)
        requestAutoSave()
    }

    func removeCustomLlm(id: String) {
        settings.customLlmModels.removeAll(where: { $0.id == id })
        if settings.activeCustomLlmId == id {
            settings.activeCustomLlmId = ""
            settings.activePostProcessingBackend = .local
        }
        requestAutoSave()
    }

    func reloadAll() {
        do {
            settings = try bridge.loadSettings()
            persistedSettingsSnapshot = settings
            devices = try bridge.listInputDevices()
            modelStatus = try bridge.getModelStatus()
            modelStatusList = (try? bridge.getModelStatusList()) ?? []
            llmStatusList = (try? bridge.getLlmStatusList()) ?? []
            diagnostics = try bridge.runPermissionDiagnostics()
            runtime = try bridge.getRuntimeStatus()
            bridgeError = nil
            isCapturingHotkey = false
            hotkeyCapturePreview = ""
            hotkeyCaptureError = nil
            hotkeyBeforeCapture = settings.hotkey
            ensureSelectedMode()
            onStateChanged?()
        } catch {
            publish(error)
        }
    }

    func poll() {
        do {
            runtime = try bridge.getRuntimeStatus()
            modelStatus = try bridge.getModelStatus()
            if let list = try? bridge.getModelStatusList() {
                modelStatusList = list
            }
            if let list = try? bridge.getLlmStatusList() {
                llmStatusList = list
            }
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
                requestAutoSave()
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
        pendingAutoSaveTask?.cancel()
        pendingAutoSaveTask = nil
        do {
            try persistSettings()
            reloadAll()
            return true
        } catch let error as InlineHotkeyValidationError {
            failHotkeyCapture(error.message)
            return false
        } catch {
            publish(error)
            return false
        }
    }

    func requestAutoSave() {
        pendingAutoSaveTask?.cancel()
        pendingAutoSaveTask = Task { [weak self] in
            try? await Task.sleep(nanoseconds: AppModel.autoSaveDebounceNanoseconds)
            guard !Task.isCancelled else { return }
            await MainActor.run {
                self?.flushAutoSave()
            }
        }
    }

    func flushAutoSave() {
        pendingAutoSaveTask?.cancel()
        pendingAutoSaveTask = nil
        guard settings != persistedSettingsSnapshot else { return }
        do {
            try persistSettings()
            runtime = try bridge.getRuntimeStatus()
            onStateChanged?()
        } catch let error as InlineHotkeyValidationError {
            failHotkeyCapture(error.message)
        } catch {
            publish(error)
        }
    }

    private func persistSettings() throws {
        let normalizedHotkey = try prepareHotkeyForAssignment(
            settings.hotkey,
            allowNoOpHotkeys: [persistedSettingsSnapshot.hotkey, runtime.hotkeyRegistered ? runtime.hotkeyText : nil]
        )
        settings.hotkey = normalizedHotkey
        hotkeyCaptureError = nil
        bridgeError = nil
        _ = try bridge.saveSettings(settings)
        persistedSettingsSnapshot = settings
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

        requestAutoSave()
    }

    func setSelectedMode(_ modeID: String) {
        selectedModeID = modeID
    }

    func setActiveMode(_ modeID: String) {
        settings.activeModeId = modeID
        selectedModeID = modeID
        flushAutoSave()
    }

    func persistActiveModeImmediately(_ modeID: String) {
        do {
            var freshSettings = try bridge.loadSettings()
            if !freshSettings.modes.contains(where: { $0.id == modeID }) {
                return
            }
            freshSettings.activeModeId = modeID
            _ = try bridge.saveSettings(freshSettings)
            reloadAll()
        } catch {
            publish(error)
        }
    }

    func createMode() {
        let existingNames = Set(settings.modes.map(\.name))
        let baseName = "Neuer Modus"
        var suffix = 1
        var candidate = baseName
        while existingNames.contains(candidate) {
            suffix += 1
            candidate = "\(baseName) \(suffix)"
        }

        let mode = ProcessingMode(
            id: UUID().uuidString.lowercased(),
            name: candidate,
            prompt: "",
            postProcessingEnabled: true
        )
        settings.modes.append(mode)
        selectedModeID = mode.id
        flushAutoSave()
    }

    func deleteSelectedMode() {
        guard canDeleteSelectedMode,
              let index = settings.modes.firstIndex(where: { $0.id == selectedModeID }) else {
            return
        }

        let deletedModeID = settings.modes[index].id
        settings.modes.remove(at: index)
        if settings.activeModeId == deletedModeID {
            settings.activeModeId = settings.modes.first?.id ?? ProcessingMode.standard.id
        }
        ensureSelectedMode()
        flushAutoSave()
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
            let normalized = try prepareHotkeyForAssignment(
                hotkey,
                allowNoOpHotkeys: [hotkeyBeforeCapture, runtime.hotkeyRegistered ? runtime.hotkeyText : nil]
            )
            settings.hotkey = normalized
            hotkeyCapturePreview = normalized
            hotkeyCaptureError = nil
            bridgeError = nil
            isCapturingHotkey = false
            flushAutoSave()
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
        hotkeyCaptureError = "Open Whisper braucht einen globalen Hotkey. Leere Eingaben sind nicht erlaubt."
        isCapturingHotkey = false
    }

    func failHotkeyCapture(_ message: String) {
        hotkeyCaptureError = message
        bridgeError = nil
    }

    func startModelDownload() {
        startModelDownload(preset: settings.localModel)
    }

    func startModelDownload(preset: ModelPreset) {
        do {
            _ = try bridge.startModelDownload(preset: preset)
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func deleteModel() {
        deleteModel(preset: settings.localModel)
    }

    func deleteModel(preset: ModelPreset) {
        do {
            _ = try bridge.deleteModel(preset: preset)
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func startLlmDownload(preset: LlmPreset) {
        do {
            _ = try bridge.startLlmDownload(preset: preset)
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func deleteLlmModel(preset: LlmPreset) {
        do {
            _ = try bridge.deleteLlmModel(preset: preset)
            bridgeError = nil
            poll()
        } catch {
            publish(error)
        }
    }

    func refreshRemoteModels(backend: RemoteModelBackend) {
        do {
            let list = try bridge.listRemoteModels(backend: backend)
            switch backend {
            case .ollama:
                ollamaModels = list
                ollamaModelsError = nil
            case .lmStudio:
                lmStudioModels = list
                lmStudioModelsError = nil
            }
        } catch {
            let message = (error as? BridgeError)?.message ?? error.localizedDescription
            switch backend {
            case .ollama:
                ollamaModels = []
                ollamaModelsError = message
            case .lmStudio:
                lmStudioModels = []
                lmStudioModelsError = message
            }
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

    private func prepareHotkeyForAssignment(
        _ hotkey: String,
        allowNoOpHotkeys: [String?]
    ) throws -> String {
        let normalized: String
        do {
            normalized = try bridge.validateHotkey(hotkey)
        } catch {
            throw InlineHotkeyValidationError(message: error.localizedDescription)
        }

        do {
            try HotkeyAssignmentAdvisor.assertCanAssign(
                normalized,
                allowNoOpHotkeys: allowNoOpHotkeys.compactMap { $0 }
            )
        } catch {
            throw InlineHotkeyValidationError(message: error.localizedDescription)
        }

        return normalized
    }

    private func ensureSelectedMode() {
        if settings.modes.isEmpty {
            settings.modes = [.standard]
        }

        if !settings.modes.contains(where: { $0.id == settings.activeModeId }) {
            settings.activeModeId = settings.modes.first?.id ?? ProcessingMode.standard.id
        }

        if !settings.modes.contains(where: { $0.id == selectedModeID }) {
            selectedModeID = settings.activeModeId
        }
    }
}

private func isSingleKeyHotkey(_ hotkey: String) -> Bool {
    let normalized = hotkey.trimmingCharacters(in: .whitespacesAndNewlines)
    return !normalized.isEmpty && !normalized.contains("+")
}

private struct InlineHotkeyValidationError: LocalizedError {
    let message: String

    var errorDescription: String? { message }
}
