import SwiftUI

struct SettingsView: View {
    @ObservedObject var model: AppModel
    @State private var selectedSection: SettingsSection? = .recording

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
        }
        .navigationSplitViewStyle(.balanced)
        .safeAreaInset(edge: .bottom) {
            bottomBar
        }
        .frame(width: 820, height: 560)
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
        case .model:
            modelContent
        case .startup:
            startupContent
        case .providers:
            providersContent
        case .diagnostics:
            diagnosticsContent
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
        }
    }

    @ViewBuilder
    private var modesContent: some View {
        Section("Modi") {
            ScrollView {
                VStack(spacing: 4) {
                    ForEach(model.availableModes) { mode in
                        ModeListTile(
                            mode: mode,
                            isSelected: model.selectedModeID == mode.id,
                            isActive: model.settings.activeModeId == mode.id
                        ) {
                            model.setSelectedMode(mode.id)
                        }
                    }
                }
                .padding(.vertical, 2)
            }
            .frame(maxHeight: 220)
            .listRowInsets(EdgeInsets())

            HStack(spacing: 10) {
                Button("Neuer Modus") { model.createMode() }
                Button("Loeschen") { model.deleteSelectedMode() }
                    .disabled(!model.canDeleteSelectedMode)
                Spacer()
                Button(model.settings.activeModeId == model.selectedMode.id ? "Aktiv" : "Als aktiv setzen") {
                    model.setActiveMode(model.selectedMode.id)
                }
                .disabled(model.settings.activeModeId == model.selectedMode.id)
            }
        }

        Section("Details") {
            TextField("Name", text: model.modeBinding(for: \.name))

            Picker("Nachverarbeitung", selection: model.modeBinding(for: \.postProcessingProvider)) {
                ForEach(PostProcessingProvider.allCases) { provider in
                    Text(provider.label).tag(provider)
                }
            }

            Text(model.selectedMode.postProcessingSummary)
                .font(.caption)
                .foregroundStyle(.secondary)

            VStack(alignment: .leading, spacing: 4) {
                Text("Prompt")
                    .font(.subheadline)
                TextEditor(text: model.modeBinding(for: \.prompt))
                    .font(.body)
                    .frame(height: 90)
                    .scrollContentBackground(.hidden)
                    .padding(6)
                    .background(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .fill(Color(nsColor: .textBackgroundColor))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .stroke(Color.primary.opacity(0.08), lineWidth: 1)
                    )
            }
        }
    }

    @ViewBuilder
    private var modelContent: some View {
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
            LabeledContent("Modus", value: model.activeModeName)
        }
    }

    @ViewBuilder
    private var providersContent: some View {
        Section("Ollama") {
            TextField("Endpoint", text: model.binding(for: \.ollama.endpoint))
            TextField("Modellname", text: model.binding(for: \.ollama.modelName))
        }

        Section("LM Studio") {
            TextField("Endpoint", text: model.binding(for: \.lmStudio.endpoint))
            TextField("Modellname", text: model.binding(for: \.lmStudio.modelName))
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
