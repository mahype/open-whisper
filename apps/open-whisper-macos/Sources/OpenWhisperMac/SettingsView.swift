import SwiftUI

struct SettingsView: View {
    @ObservedObject var model: AppModel
    @State private var selectedSection: SettingsSection? = .recording

    var body: some View {
        NavigationSplitView {
            List(SettingsSection.allCases, selection: $selectedSection) { section in
                Label {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(section.title)
                        Text(section.subtitle)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                } icon: {
                    Image(systemName: section.symbolName)
                }
                .tag(section)
            }
            .listStyle(.sidebar)
            .navigationSplitViewColumnWidth(min: 210, ideal: 230, max: 260)
        } detail: {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    DetailHeader(title: detailSection.title, subtitle: detailSection.subtitle)
                    detailContent(for: detailSection)
                }
                .padding(24)
                .frame(maxWidth: .infinity, alignment: .topLeading)
            }
            .background(Color(nsColor: .windowBackgroundColor))
        }
        .navigationSplitViewStyle(.balanced)
        .safeAreaInset(edge: .bottom) {
            bottomBar
        }
        .frame(minWidth: 920, minHeight: 660)
    }

    private var detailSection: SettingsSection {
        selectedSection ?? .recording
    }

    @ViewBuilder
    private func detailContent(for section: SettingsSection) -> some View {
        switch section {
        case .recording:
            recordingContent
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

    private var recordingContent: some View {
        VStack(alignment: .leading, spacing: 18) {
            AppCard(title: "Audioquelle", subtitle: "Mikrofon und Aufnahmemodus") {
                Picker("Mikrofon", selection: model.binding(for: \.inputDeviceName)) {
                    ForEach(deviceNames, id: \.self) { device in
                        Text(device).tag(device)
                    }
                }

                Picker("Modus", selection: model.binding(for: \.triggerMode)) {
                    ForEach(TriggerMode.allCases) { mode in
                        Text(mode.label).tag(mode)
                    }
                }
                .pickerStyle(.segmented)

                Picker("Sprache", selection: model.languageBinding()) {
                    ForEach(model.availableLanguageOptions) { option in
                        Text(option.label).tag(option.code)
                    }
                }

                HStack {
                    Button("Geraete aktualisieren") {
                        model.refreshDevices()
                    }
                    Spacer()
                }
            }

            AppCard(title: "Globaler Hotkey", subtitle: "Wird erst nach dem Speichern neu registriert") {
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

            AppCard(title: "Textausgabe", subtitle: "Wie das Transkript in die aktive App gelangt") {
                Toggle("Text automatisch in aktive App einfuegen", isOn: model.binding(for: \.insertTextAutomatically))
                Toggle("Clipboard nach Einfuegen wiederherstellen", isOn: model.binding(for: \.restoreClipboardAfterInsert))
            }
        }
    }

    private var modelContent: some View {
        VStack(alignment: .leading, spacing: 18) {
            AppCard(title: "Standardmodell", subtitle: "Waehl das lokale Whisper-Modell fuer deinen Rechner") {
                VStack(alignment: .leading, spacing: 12) {
                    ForEach(ModelPreset.allCases) { preset in
                        ModelPresetTile(preset: preset, isSelected: model.settings.localModel == preset) {
                            model.choosePreset(preset)
                        }
                    }
                }
            }

            AppCard(title: "Downloadstatus", subtitle: "Nur nutzerrelevante Infos zum aktuell gewaehlten Modell") {
                MetricRow(label: "Auswahl", value: model.selectedModelDisplayName)
                MetricRow(label: "Status", value: model.selectedModelStatusText)
                if let progress = model.modelDownloadProgress {
                    ProgressView(value: progress) {
                        Text("Download")
                    }
                }

                HStack(spacing: 12) {
                    Button(model.modelStatus.isDownloading ? "Download laeuft..." : "Modell herunterladen") {
                        model.startModelDownload()
                    }
                    .disabled(model.modelStatus.isDownloading)

                    Button("Lokales Modell loeschen") {
                        model.deleteModel()
                    }
                    .disabled(model.modelStatus.isDownloading)
                }
            }
        }
    }

    private var startupContent: some View {
        VStack(alignment: .leading, spacing: 18) {
            AppCard(title: "Systemstart", subtitle: "Wie Open Whisper beim Login startet") {
                Picker("Startverhalten", selection: model.binding(for: \.startupBehavior)) {
                    ForEach(StartupBehavior.allCases) { behavior in
                        Text(behavior.label).tag(behavior)
                    }
                }
            }

            AppCard(title: "Diktatverhalten", subtitle: "Stopp bei Stille und Recorder-Verhalten") {
                Toggle("Voice Activity Detection aktivieren", isOn: model.binding(for: \.vadEnabled))

                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text("Silence-Stop")
                        Spacer()
                        Text("\(model.settings.vadSilenceMs) ms")
                            .foregroundStyle(.secondary)
                    }
                    Slider(
                        value: Binding(
                            get: { Double(model.settings.vadSilenceMs) },
                            set: {
                                model.settings.vadSilenceMs = UInt32($0.rounded())
                                model.isDirty = true
                            }
                        ),
                        in: 300...2_500,
                        step: 50
                    )
                }
            }

            AppCard(title: "Aktive Registrierung", subtitle: "Was im laufenden Prozess aktuell gilt") {
                MetricRow(label: "Systemstart", value: model.runtime.startupSummary)
                MetricRow(label: "Registrierter Hotkey", value: model.runtime.hotkeyText)
                MetricRow(label: "Ausloesungen", value: "\(model.runtime.dictationTriggerCount)")
            }
        }
    }

    private var providersContent: some View {
        VStack(alignment: .leading, spacing: 18) {
            AppCard(title: "Provider-Auswahl", subtitle: "Optional fuer spaetere Erweiterungen") {
                Picker("Aktiver Provider", selection: model.binding(for: \.activeProvider)) {
                    ForEach(ProviderKind.allCases) { provider in
                        Text(provider.label).tag(provider)
                    }
                }

                Text("Das produktive Standard-Diktat bleibt lokal auf Whisper Base, Whisper Small und Whisper Medium. Ollama und LM Studio bleiben optional.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            AppCard(title: "Ollama", subtitle: "Optionaler lokaler Zusatzprovider") {
                TextField("Endpoint", text: model.binding(for: \.ollama.endpoint))
                    .textFieldStyle(.roundedBorder)
                TextField("Modellname", text: model.binding(for: \.ollama.modelName))
                    .textFieldStyle(.roundedBorder)
            }

            AppCard(title: "LM Studio", subtitle: "Optionaler lokaler Zusatzprovider") {
                TextField("Endpoint", text: model.binding(for: \.lmStudio.endpoint))
                    .textFieldStyle(.roundedBorder)
                TextField("Modellname", text: model.binding(for: \.lmStudio.modelName))
                    .textFieldStyle(.roundedBorder)
            }
        }
    }

    private var diagnosticsContent: some View {
        VStack(alignment: .leading, spacing: 18) {
            AppCard(title: "Diagnose", subtitle: "Kompakte Uebersicht ueber Rechte, Tray und Systemstatus") {
                Text(model.diagnostics.summary)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                HStack(spacing: 12) {
                    Button("Diagnose aktualisieren") {
                        model.refreshDiagnostics()
                    }
                    Button("System Settings oeffnen") {
                        model.openSystemSettings()
                    }
                }
            }

            VStack(alignment: .leading, spacing: 12) {
                ForEach(model.diagnostics.items) { item in
                    DiagnosticDisclosureCard(item: item)
                }
            }
        }
    }

    private var bottomBar: some View {
        HStack(spacing: 14) {
            VStack(alignment: .leading, spacing: 6) {
                Text(model.bridgeError ?? model.runtime.lastStatus)
                    .font(.callout)
                    .foregroundStyle(model.bridgeError == nil ? Color.secondary : Color.red)
                    .lineLimit(2)

                HStack(spacing: 8) {
                    InlineStatusPill(title: "Hotkey", value: hotkeyDisplayString(model.hotkeyDisplayText), accent: .blue)
                    InlineStatusPill(title: "Runtime", value: runtimeLabel, accent: runtimeAccent)
                    InlineStatusPill(title: "Provider", value: model.activeProviderLabel, accent: .green)
                }
            }

            Spacer()

            if model.isDirty {
                Text("Ungespeicherte Aenderungen")
                    .font(.caption.weight(.semibold))
                    .padding(.vertical, 6)
                    .padding(.horizontal, 10)
                    .background(Color.orange.opacity(0.14), in: Capsule())
                    .foregroundStyle(.orange)
            }

            Button(model.runtime.isRecording ? "Diktat stoppen" : "Diktat starten") {
                model.toggleDictation()
            }

            Button("Speichern") {
                model.saveSettings()
            }
            .buttonStyle(.borderedProminent)
            .keyboardShortcut("s", modifiers: [.command])
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 14)
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
        if model.runtime.isTranscribing {
            return "Transkription laeuft"
        }
        return "Bereit"
    }

    private var runtimeAccent: Color {
        if model.runtime.isRecording {
            return .red
        }
        if model.runtime.isTranscribing {
            return .orange
        }
        return .green
    }
}
