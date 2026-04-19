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
        case 2: return "Modell & Start"
        case 3: return "Nachbearbeitung"
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
            Section("Modell") {
                ForEach(ModelPreset.allCases) { preset in
                    ModelPresetTile(preset: preset, isSelected: model.settings.localModel == preset) {
                        model.choosePreset(preset)
                    }
                    .listRowInsets(EdgeInsets(top: 4, leading: 0, bottom: 4, trailing: 0))
                }
            }

            Section("Status") {
                LabeledContent("Auswahl", value: model.selectedModelDisplayName)
                LabeledContent("Status", value: model.selectedModelStatusText)

                if let progress = model.modelDownloadProgress {
                    ProgressView(value: progress)
                }

                HStack(spacing: 10) {
                    Button(model.modelStatus.isDownloading ? "Download laeuft..." : "Herunterladen") {
                        model.startModelDownload()
                    }
                    .disabled(model.modelStatus.isDownloading)

                    Button("Loeschen") {
                        model.deleteModel()
                    }
                    .disabled(model.modelStatus.isDownloading)
                }
            }

            Section("Start & Verhalten") {
                Picker("Systemstart", selection: model.binding(for: \.startupBehavior)) {
                    ForEach(StartupBehavior.allCases) { behavior in
                        Text(behavior.label).tag(behavior)
                    }
                }

                Toggle("Text automatisch einfuegen", isOn: model.binding(for: \.insertTextAutomatically))
                Toggle("Clipboard wiederherstellen", isOn: model.binding(for: \.restoreClipboardAfterInsert))
                Toggle("Silence-Stop aktivieren", isOn: model.binding(for: \.vadEnabled))
            }
        case 3:
            Section("Sprachmodell fuer Nachbearbeitung") {
                Text("Waehle ein Gemma-4-Modell, das deine Diktate automatisch aufraeumen oder in einem bestimmten Stil umschreiben kann. Der Download laeuft im Hintergrund. Du kannst das Modell pro Modus aendern oder den Schritt ueberspringen.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            Section("Modellgroesse") {
                Picker("Default", selection: model.binding(for: \.localLlm)) {
                    ForEach(LlmPreset.allCases) { preset in
                        Text(preset.displayName).tag(preset)
                    }
                }
                .pickerStyle(.radioGroup)

                Text(model.settings.localLlm.description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("Download-Groesse: \(model.settings.localLlm.approxSizeLabel)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section("Download") {
                let defaultStatus = model.llmStatusList.first { status in
                    status.displayLabel == model.settings.localLlm.displayName
                }

                LabeledContent("Status", value: defaultStatus?.summary ?? "Noch nicht geprueft.")

                if let status = defaultStatus,
                   status.isDownloading,
                   let basisPoints = status.progressBasisPoints {
                    ProgressView(value: Double(basisPoints) / 10_000.0)
                }

                HStack(spacing: 10) {
                    Button(defaultStatus?.isDownloading == true ? "Lade..." : "Herunterladen") {
                        model.startLlmDownload(preset: model.settings.localLlm)
                    }
                    .disabled(defaultStatus?.isDownloading == true || defaultStatus?.isDownloaded == true)

                    if defaultStatus?.isDownloaded == true {
                        Text("Bereit.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
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
                    model.onboardingStep = min(4, model.onboardingStep + 1)
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
}
