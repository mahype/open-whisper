import SwiftUI

struct OnboardingView: View {
    @ObservedObject var model: AppModel
    let onFinish: () -> Void

    var body: some View {
        HStack(spacing: 0) {
            StepRail(currentStep: model.onboardingStep)
                .frame(width: 200)

            VStack(spacing: 0) {
                Form {
                    currentStep
                }
                .formStyle(.grouped)
                .scrollDisabled(true)
                .navigationTitle(stepTitle)

                footer
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(width: 760, height: 520)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private var stepTitle: String {
        switch model.onboardingStep {
        case 0: return "Willkommen"
        case 1: return "Audio & Hotkey"
        case 2: return "Sprachmodelle"
        case 3: return "Start & Verhalten"
        default: return "Diagnose"
        }
    }

    @ViewBuilder
    private var currentStep: some View {
        switch model.onboardingStep {
        case 0:
            Section("Open Whisper") {
                Text("Tray-first, lokal und fuer den Alltag. Standard ist lokales Whisper; Ollama und LM Studio bleiben optional.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            Section("Aktuelle Auswahl") {
                LabeledContent("Mikrofon", value: model.settings.inputDeviceName)
                LabeledContent("Hotkey", value: hotkeyDisplayString(model.settings.hotkey))
                LabeledContent("Modell", value: model.selectedModelDisplayName)
                LabeledContent("Systemstart", value: model.settings.startupBehavior.label)
            }
        case 1:
            Section("Audioquelle") {
                Picker("Mikrofon", selection: model.binding(for: \.inputDeviceName)) {
                    ForEach(deviceNames, id: \.self) { device in
                        Text(device).tag(device)
                    }
                }

                Picker("Sprache", selection: model.languageBinding()) {
                    ForEach(model.availableLanguageOptions) { option in
                        Text(option.label).tag(option.code)
                    }
                }

                Button("Mikrofone neu laden") {
                    model.refreshDevices()
                }
            }

            Section("Trigger") {
                Picker("Modus", selection: model.binding(for: \.triggerMode)) {
                    ForEach(TriggerMode.allCases) { mode in
                        Text(mode.label).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
            }

            Section("Globaler Hotkey") {
                HotkeyRecorderField(
                    title: model.hotkeyFieldTitle,
                    currentHotkey: model.settings.hotkey,
                    isCapturing: model.isCapturingHotkey,
                    preview: model.hotkeyCapturePreview,
                    errorText: model.hotkeyCaptureError,
                    warningText: model.hotkeyRiskHint,
                    onStartCapture: { model.startHotkeyCapture() },
                    onCommit: { model.commitCapturedHotkey($0) },
                    onCancel: { model.cancelHotkeyCapture() },
                    onClear: { model.clearHotkeyCapture() },
                    onPreview: { model.updateHotkeyCapturePreview($0) },
                    onInvalid: { model.failHotkeyCapture($0) }
                )
            }
        case 2:
            Section("Transkription") {
                Picker("Whisper-Modell", selection: model.binding(for: \.localModel)) {
                    ForEach(ModelPreset.allCases) { preset in
                        Text(preset.displayName).tag(preset)
                    }
                }

                Text("\(model.settings.localModel.description) (\(model.settings.localModel.downloadSizeText))")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let status = currentWhisperStatus {
                    if status.isDownloading, let basisPoints = status.progressBasisPoints {
                        ProgressView(value: Double(basisPoints) / 10_000.0)
                    }
                    LabeledContent("Status", value: status.summary)
                }
            }

            Section("Nachbearbeitung") {
                Picker("Sprachmodell", selection: model.binding(for: \.localLlm)) {
                    ForEach(LlmPreset.allCases) { preset in
                        Text(preset.displayName).tag(preset)
                    }
                }

                Text("\(model.settings.localLlm.description) (\(model.settings.localLlm.approxSizeLabel))")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let status = currentLlmStatus {
                    if status.isDownloading, let basisPoints = status.progressBasisPoints {
                        ProgressView(value: Double(basisPoints) / 10_000.0)
                    }
                    LabeledContent("Status", value: status.summary)
                }
            }

            Section {
                Text("Beide Modelle starten den Download im Hintergrund, sobald du auf 'Weiter' klickst. Du kannst sie spaeter in den Einstellungen unter 'Sprachmodelle' aendern oder verwalten.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        case 3:
            Section("Systemstart") {
                Picker("Systemstart", selection: model.binding(for: \.startupBehavior)) {
                    ForEach(StartupBehavior.allCases) { behavior in
                        Text(behavior.label).tag(behavior)
                    }
                }
            }

            Section("Textausgabe") {
                Toggle("Text automatisch einfuegen", isOn: model.binding(for: \.insertTextAutomatically))
                Toggle("Clipboard wiederherstellen", isOn: model.binding(for: \.restoreClipboardAfterInsert))
            }

            Section("Diktat-Stopp") {
                Toggle("Silence-Stop aktivieren", isOn: model.binding(for: \.vadEnabled))
            }
        default:
            Section("Uebersicht") {
                Text(model.diagnostics.summary)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                HStack(spacing: 10) {
                    Button("Aktualisieren") { model.refreshDiagnostics() }
                    Button("System Settings oeffnen") { model.openSystemSettings() }
                }
            }

            Section("Details") {
                ForEach(model.diagnostics.items) { item in
                    DiagnosticDisclosureCard(item: item)
                        .listRowInsets(EdgeInsets(top: 4, leading: 0, bottom: 4, trailing: 0))
                }
            }
        }
    }

    private var footer: some View {
        HStack {
            Button("Zurueck") {
                model.onboardingStep = max(0, model.onboardingStep - 1)
            }
            .disabled(model.onboardingStep == 0)

            Spacer()

            if model.onboardingStep == 4 {
                Button("Abschliessen") {
                    if model.completeOnboarding() {
                        onFinish()
                    }
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
            } else {
                Button("Weiter") {
                    let current = model.onboardingStep
                    if current == 2 {
                        triggerModelDownloadsIfNeeded()
                    }
                    model.onboardingStep = min(4, current + 1)
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
        .background(.regularMaterial)
    }

    private var deviceNames: [String] {
        let names = model.devices.map(\.name)
        if names.isEmpty {
            return [model.settings.inputDeviceName]
        }
        return names
    }

    private var currentWhisperStatus: ModelStatusDTO? {
        if model.modelStatusList.isEmpty {
            return model.modelStatus
        }
        return model.modelStatusList.first { $0.backendModelName == model.settings.localModel.whisperModel }
    }

    private var currentLlmStatus: LlmModelStatusDTO? {
        model.llmStatusList.first { $0.displayLabel == model.settings.localLlm.displayName }
    }

    private func triggerModelDownloadsIfNeeded() {
        let whisperDownloaded = currentWhisperStatus?.isDownloaded ?? false
        let whisperDownloading = currentWhisperStatus?.isDownloading ?? false
        if !whisperDownloaded && !whisperDownloading {
            model.startModelDownload(preset: model.settings.localModel)
        }

        let llmDownloaded = currentLlmStatus?.isDownloaded ?? false
        let llmDownloading = currentLlmStatus?.isDownloading ?? false
        if !llmDownloaded && !llmDownloading {
            model.startLlmDownload(preset: model.settings.localLlm)
        }
    }
}
