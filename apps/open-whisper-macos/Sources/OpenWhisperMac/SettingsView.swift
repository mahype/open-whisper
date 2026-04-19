import SwiftUI

struct SettingsView: View {
    @ObservedObject var model: AppModel
    let updaterController: UpdaterController
    let onReopenOnboarding: () -> Void
    @State private var selectedSection: SettingsSection? = .recording
    @State private var isEditingMode: Bool = false
    @State private var isManagingLanguageModels: Bool = false
    @State private var managerTab: LanguageModelsManagerTab = .transcription

    var body: some View {
        NavigationSplitView {
            List(SettingsSection.allCases, selection: $selectedSection) { section in
                Label(section.title, systemImage: section.symbolName)
                    .tag(section)
            }
            .listStyle(.sidebar)
            .navigationSplitViewColumnWidth(min: 190, ideal: 200, max: 220)
        } detail: {
            Form {
                detailContent(for: detailSection)
            }
            .formStyle(.grouped)
            .navigationTitle(detailSection.title)
            .sheet(isPresented: $isEditingMode) {
                ModeEditorSheet(model: model) {
                    isEditingMode = false
                }
            }
            .sheet(isPresented: $isManagingLanguageModels) {
                LanguageModelsManagerSheet(
                    model: model,
                    selectedTab: $managerTab
                ) {
                    isManagingLanguageModels = false
                }
            }
        }
        .navigationSplitViewStyle(.balanced)
        .safeAreaInset(edge: .bottom) {
            bottomBar
        }
        .frame(width: 820, height: 720)
    }

    private var detailSection: SettingsSection {
        selectedSection ?? .recording
    }

    @ViewBuilder
    private func detailContent(for section: SettingsSection) -> some View {
        switch section {
        case .recording:
            recordingContent
        case .modes:
            modesContent
        case .languageModels:
            languageModelsContent
        case .startup:
            startupContent
        case .updates:
            UpdatesSettingsView(updaterController: updaterController)
        case .diagnostics:
            diagnosticsContent
        case .help:
            helpContent
        }
    }

    @ViewBuilder
    private var recordingContent: some View {
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

            Button("Geraete aktualisieren") {
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

        Section("Textausgabe") {
            Toggle("Text automatisch einfuegen", isOn: model.binding(for: \.insertTextAutomatically))
            Toggle("Clipboard wiederherstellen", isOn: model.binding(for: \.restoreClipboardAfterInsert))
        }

        Section("Aufnahme-Anzeige") {
            Toggle(
                "Wellenform-Fenster waehrend Aufnahme anzeigen",
                isOn: model.binding(for: \.showRecordingIndicator)
            )

            Picker("Stil", selection: model.binding(for: \.waveformStyle)) {
                ForEach(WaveformStyle.allCases) { style in
                    Text(style.label).tag(style)
                }
            }
            .disabled(!model.settings.showRecordingIndicator)

            Picker("Farbe", selection: model.binding(for: \.waveformColor)) {
                ForEach(WaveformColor.allCases) { color in
                    Text(color.label)
                        .foregroundStyle(color.swiftUIColor)
                        .tag(color)
                }
            }
            .disabled(!model.settings.showRecordingIndicator)
        }
    }

    @ViewBuilder
    private var modesContent: some View {
        Section("Nachbearbeitung") {
            Toggle("Nachbearbeitung aktivieren", isOn: model.binding(for: \.postProcessingEnabled))

            ForEach(model.availableModes) { mode in
                ModeListTile(
                    mode: mode,
                    isSelected: model.selectedModeID == mode.id,
                    isActive: model.settings.postProcessingEnabled && model.settings.activeModeId == mode.id,
                    action: { model.setSelectedMode(mode.id) },
                    onEdit: {
                        model.setSelectedMode(mode.id)
                        isEditingMode = true
                    }
                )
                .listRowInsets(EdgeInsets(top: 4, leading: 0, bottom: 4, trailing: 0))
            }

            HStack(spacing: 10) {
                Button("Neue Nachbearbeitung") {
                    model.createMode()
                    isEditingMode = true
                }
                Button("Loeschen") { model.deleteSelectedMode() }
                    .disabled(!model.canDeleteSelectedMode)
                Spacer()
                Button(model.settings.activeModeId == model.selectedMode.id ? "Aktiv" : "Als aktiv setzen") {
                    model.setActiveMode(model.selectedMode.id)
                }
                .disabled(model.settings.activeModeId == model.selectedMode.id)
            }
        }
    }

    @ViewBuilder
    private var languageModelsContent: some View {
        Section("Transkription") {
            Picker("Modell", selection: model.binding(for: \.localModel)) {
                ForEach(model.availableModelPresets) { preset in
                    Text(model.whisperPresetPickerLabel(preset)).tag(preset)
                }
            }

            Text(model.selectedModelStatusText)
                .font(.caption)
                .foregroundStyle(.secondary)

            if let progress = model.modelDownloadProgress {
                ProgressView(value: progress)
            }

            Button("Sprachmodelle verwalten...") {
                managerTab = .transcription
                isManagingLanguageModels = true
            }
        }

        Section("Nachbearbeitung") {
            Picker("Modell", selection: model.postProcessingChoiceBinding) {
                ForEach(model.availablePostProcessingChoices) { choice in
                    Text(model.postProcessingChoicePickerLabel(choice)).tag(choice)
                }
            }

            Text(postProcessingSummaryText)
                .font(.caption)
                .foregroundStyle(.secondary)

            Button("Sprachmodelle verwalten...") {
                managerTab = .postProcessing
                isManagingLanguageModels = true
            }
        }
    }

    private var postProcessingSummaryText: String {
        switch model.settings.activePostProcessingBackend {
        case .local:
            return "Lokal \u{2013} \(model.settings.localLlm.approxSizeLabel)"
        case .ollama:
            return "Ollama \u{2013} \(model.settings.ollama.endpoint) / \(model.settings.ollama.modelName)"
        case .lmStudio:
            return "LM Studio \u{2013} \(model.settings.lmStudio.endpoint) / \(model.settings.lmStudio.modelName)"
        }
    }

    @ViewBuilder
    private var startupContent: some View {
        Section("Systemstart") {
            Picker("Verhalten", selection: model.binding(for: \.startupBehavior)) {
                ForEach(StartupBehavior.allCases) { behavior in
                    Text(behavior.label).tag(behavior)
                }
            }
        }

        Section("Diktat-Stopp") {
            Toggle("Voice Activity Detection", isOn: model.binding(for: \.vadEnabled))

            LabeledContent("Silence-Stop") {
                HStack(spacing: 10) {
                    Slider(
                        value: Binding(
                            get: { Double(model.settings.vadSilenceMs) },
                            set: {
                                model.settings.vadSilenceMs = UInt32($0.rounded())
                                model.requestAutoSave()
                            }
                        ),
                        in: 300...2_500,
                        step: 50
                    )
                    .frame(width: 200)
                    Text("\(model.settings.vadSilenceMs) ms")
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                        .frame(width: 70, alignment: .trailing)
                }
            }
        }

        Section("Aktuell registriert") {
            LabeledContent("Systemstart", value: model.runtime.startupSummary)
            LabeledContent("Hotkey", value: model.runtime.hotkeyText)
            LabeledContent("Nachbearbeitung", value: model.activeModeName)
        }
    }

    @ViewBuilder
    private var diagnosticsContent: some View {
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

    @ViewBuilder
    private var helpContent: some View {
        Section("Setup") {
            Text("Du kannst den Einrichtungs-Assistenten jederzeit erneut starten, um Mikrofon, Hotkey und Sprachmodelle neu zu konfigurieren.")
                .font(.callout)
                .foregroundStyle(.secondary)

            Button("Onboarding erneut starten") {
                onReopenOnboarding()
            }
        }
    }

    private var bottomBar: some View {
        HStack(spacing: 12) {
            HStack(spacing: 6) {
                Circle()
                    .fill(runtimeAccent)
                    .frame(width: 8, height: 8)
                Text(model.bridgeError ?? runtimeLabel)
                    .font(.callout)
                    .foregroundStyle(model.bridgeError == nil ? Color.primary : Color.red)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }

            Spacer()

            Button(model.runtime.isRecording ? "Stoppen" : "Diktat starten") {
                model.toggleDictation()
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 10)
        .background(.regularMaterial)
    }

    private var deviceNames: [String] {
        let names = model.devices.map(\.name)
        if names.isEmpty {
            return [model.settings.inputDeviceName]
        }
        return names
    }

    private var runtimeLabel: String {
        if model.runtime.isRecording {
            return "Aufnahme aktiv"
        }
        if model.runtime.isPostProcessing {
            return "Nachverarbeitung laeuft"
        }
        if model.runtime.isTranscribing {
            return "Transkription laeuft"
        }
        return model.runtime.lastStatus.isEmpty ? "Bereit" : model.runtime.lastStatus
    }

    private var runtimeAccent: Color {
        if model.bridgeError != nil {
            return .red
        }
        if model.runtime.isRecording {
            return .red
        }
        if model.runtime.isPostProcessing {
            return .purple
        }
        if model.runtime.isTranscribing {
            return .orange
        }
        return .green
    }
}
