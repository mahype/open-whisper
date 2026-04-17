import SwiftUI

struct SettingsView: View {
    @ObservedObject var model: AppModel

    var body: some View {
        VStack(spacing: 0) {
            Form {
                Section("Status") {
                    LabeledContent("Hotkey") {
                        Text(model.runtime.hotkeyText)
                    }
                    LabeledContent("Runtime") {
                        Text(runtimeLabel)
                    }
                    LabeledContent("Provider") {
                        Text(model.runtime.providerSummary)
                    }
                    if !model.runtime.lastTranscript.isEmpty {
                        Text(model.runtime.lastTranscript)
                            .font(.body)
                            .textSelection(.enabled)
                            .padding(.vertical, 4)
                    }
                }

                Section("Aufnahme") {
                    Picker("Mikrofon", selection: $model.settings.inputDeviceName) {
                        ForEach(deviceNames, id: \.self) { device in
                            Text(device).tag(device)
                        }
                    }
                    .pickerStyle(.menu)

                    HStack {
                        Button("Geraete aktualisieren") {
                            model.refreshDevices()
                        }
                        Spacer()
                    }

                    TextField("Globaler Hotkey", text: $model.settings.hotkey)
                    Picker("Modus", selection: $model.settings.triggerMode) {
                        ForEach(TriggerMode.allCases) { mode in
                            Text(mode.label).tag(mode)
                        }
                    }
                    .pickerStyle(.segmented)

                    TextField("Sprache", text: $model.settings.transcriptionLanguage)
                    Toggle("Text automatisch in aktive App einfuegen", isOn: $model.settings.insertTextAutomatically)
                    Toggle("Clipboard nach Einfuegen wiederherstellen", isOn: $model.settings.restoreClipboardAfterInsert)
                }

                Section("Sprachmodell") {
                    Picker("Standardmodell", selection: Binding(
                        get: { model.settings.localModel },
                        set: { model.choosePreset($0) }
                    )) {
                        ForEach(ModelPreset.allCases) { preset in
                            Text(preset.label).tag(preset)
                        }
                    }
                    .pickerStyle(.segmented)

                    Text(model.settings.localModel.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    TextField("Modellpfad", text: $model.settings.localModelPath)
                    Text(model.modelStatus.summary)
                        .foregroundStyle(.secondary)

                    if let progress = model.modelDownloadProgress {
                        ProgressView(value: progress)
                    }

                    HStack {
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

                Section("Start & Verhalten") {
                    Picker("Startverhalten", selection: $model.settings.startupBehavior) {
                        ForEach(StartupBehavior.allCases) { behavior in
                            Text(behavior.label).tag(behavior)
                        }
                    }

                    Toggle("Voice Activity Detection aktivieren", isOn: $model.settings.vadEnabled)
                    HStack {
                        Text("Silence-Stop")
                        Spacer()
                        Text("\(model.settings.vadSilenceMs) ms")
                            .foregroundStyle(.secondary)
                    }
                    Slider(
                        value: Binding(
                            get: { Double(model.settings.vadSilenceMs) },
                            set: { model.settings.vadSilenceMs = UInt32($0.rounded()) }
                        ),
                        in: 300...2_500,
                        step: 50
                    )
                }

                Section("Optionale Provider") {
                    Picker("Aktiver Provider", selection: $model.settings.activeProvider) {
                        ForEach(ProviderKind.allCases) { provider in
                            Text(provider.label).tag(provider)
                        }
                    }
                    Text("Ollama und LM Studio bleiben optional. Das produktive Standard-Diktat laeuft lokal ueber Whisper mit Klein, Mittel und Gross.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    TextField("Ollama Endpoint", text: $model.settings.ollama.endpoint)
                    TextField("Ollama Modell", text: $model.settings.ollama.modelName)
                    TextField("LM Studio Endpoint", text: $model.settings.lmStudio.endpoint)
                    TextField("LM Studio Modell", text: $model.settings.lmStudio.modelName)
                }

                Section("Diagnose") {
                    Text(model.diagnostics.summary)
                        .foregroundStyle(.secondary)
                    ForEach(model.diagnostics.items) { item in
                        VStack(alignment: .leading, spacing: 4) {
                            HStack {
                                Text(item.title)
                                    .font(.headline)
                                Spacer()
                                Text(item.status.label)
                                    .font(.caption)
                                    .foregroundStyle(color(for: item.status))
                            }
                            Text(item.problem)
                            Text(item.recommendation)
                                .foregroundStyle(.secondary)
                        }
                        .padding(.vertical, 4)
                    }

                    HStack {
                        Button("Diagnose aktualisieren") {
                            model.refreshDiagnostics()
                        }
                        Button("System Settings oeffnen") {
                            model.openSystemSettings()
                        }
                    }
                }
            }

            Divider()

            HStack(spacing: 12) {
                Text(model.bridgeError ?? model.runtime.lastStatus)
                    .font(.callout)
                    .foregroundColor(model.bridgeError == nil ? .secondary : .red)
                    .lineLimit(2)
                Spacer()
                Button(model.runtime.isRecording ? "Diktat stoppen" : "Diktat starten") {
                    model.toggleDictation()
                }
                Button("Speichern") {
                    model.saveSettings()
                }
                .keyboardShortcut("s", modifiers: [.command])
            }
            .padding(16)
        }
        .padding(.top, 12)
        .frame(minWidth: 760, minHeight: 780)
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

    private func color(for status: DiagnosticStatus) -> Color {
        switch status {
        case .ok:
            return .green
        case .info:
            return .secondary
        case .warning:
            return .orange
        case .error:
            return .red
        }
    }
}
