import AppKit
import SwiftUI

enum LanguageModelsManagerTab: String, CaseIterable, Identifiable {
    case transcription
    case postProcessing

    var id: String { rawValue }

    var title: String {
        switch self {
        case .transcription: return "Transkription"
        case .postProcessing: return "Nachbearbeitung"
        }
    }
}

struct LanguageModelsManagerSheet: View {
    @ObservedObject var model: AppModel
    @Binding var selectedTab: LanguageModelsManagerTab
    let onDone: () -> Void

    @State private var isShowingUrlDialog: Bool = false
    @State private var urlDialogName: String = ""
    @State private var urlDialogUrl: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text("Sprachmodelle verwalten")
                    .font(.title3.weight(.semibold))
                Spacer()
            }

            Picker("", selection: $selectedTab) {
                ForEach(LanguageModelsManagerTab.allCases) { tab in
                    Text(tab.title).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .labelsHidden()

            Form {
                switch selectedTab {
                case .transcription:
                    transcriptionContent
                case .postProcessing:
                    postProcessingContent
                }
            }
            .formStyle(.grouped)
            .scrollContentBackground(.hidden)

            HStack {
                Spacer()
                Button("Fertig", action: onDone)
                    .keyboardShortcut(.defaultAction)
            }
        }
        .padding(20)
        .frame(minWidth: 640, idealWidth: 700, minHeight: 480, idealHeight: 560)
        .sheet(isPresented: $isShowingUrlDialog) {
            urlAddDialog
        }
    }

    private var urlAddDialog: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Sprachmodell per URL hinzuf\u{FC}gen")
                .font(.headline)

            Form {
                TextField("Anzeigename", text: $urlDialogName)
                TextField("Download-URL (.gguf)", text: $urlDialogUrl)
            }
            .formStyle(.grouped)

            Text("Die Datei wird nach dem Hinzuf\u{FC}gen \u{FC}ber den 'Herunterladen'-Button geladen. Quellen wie Hugging Face 'resolve/main'-Links werden empfohlen.")
                .font(.caption)
                .foregroundStyle(.secondary)

            HStack {
                Spacer()
                Button("Abbrechen") {
                    isShowingUrlDialog = false
                }
                Button("Hinzuf\u{FC}gen") {
                    let trimmedName = urlDialogName.trimmingCharacters(in: .whitespacesAndNewlines)
                    let trimmedUrl = urlDialogUrl.trimmingCharacters(in: .whitespacesAndNewlines)
                    guard !trimmedName.isEmpty, !trimmedUrl.isEmpty else { return }
                    model.addCustomUrlLlm(name: trimmedName, url: trimmedUrl)
                    urlDialogName = ""
                    urlDialogUrl = ""
                    isShowingUrlDialog = false
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
                .disabled(
                    urlDialogName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                        || urlDialogUrl.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                )
            }
        }
        .padding(20)
        .frame(minWidth: 420, idealWidth: 480, minHeight: 240)
    }

    @ViewBuilder
    private var transcriptionContent: some View {
        Section("Whisper-Presets") {
            ForEach(ModelPreset.allCases) { preset in
                let status = model.modelStatusList.first(where: { $0.backendModelName == preset.whisperModel })
                whisperTile(preset: preset, status: status)
            }
        }
    }

    @ViewBuilder
    private var postProcessingContent: some View {
        Section("Lokale Sprachmodelle") {
            ForEach(LlmPreset.allCases) { preset in
                let status = model.llmStatusList.first(where: { $0.displayLabel == preset.displayName })
                llmTile(preset: preset, status: status)
            }
        }

        Section("Eigene Modelle") {
            if model.settings.customLlmModels.isEmpty {
                Text("Noch keine eigenen Sprachmodelle hinzugef\u{FC}gt.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(model.settings.customLlmModels) { entry in
                    customLlmTile(entry: entry)
                }
            }

            HStack(spacing: 10) {
                Button("+ Datei ausw\u{E4}hlen") {
                    presentCustomLlmFilePicker()
                }
                Button("+ Von URL laden") {
                    urlDialogName = ""
                    urlDialogUrl = ""
                    isShowingUrlDialog = true
                }
            }
        }

        Section("Ollama") {
            TextField("Endpoint", text: model.binding(for: \.ollama.endpoint))
            HStack(spacing: 10) {
                Button("Modelle abrufen") {
                    model.refreshRemoteModels(backend: .ollama)
                }
                if let err = model.ollamaModelsError {
                    Text(err)
                        .font(.caption)
                        .foregroundStyle(.red)
                        .lineLimit(2)
                }
            }
            if model.ollamaModels.isEmpty && model.ollamaModelsError == nil {
                Text("Noch keine Modellliste abgerufen. Laufender Ollama-Server ben\u{F6}tigt.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(model.ollamaModels) { entry in
                    remoteModelTile(entry: entry, isActive: isActiveOllama(entry))
                }
            }
        }

        Section("LM Studio") {
            TextField("Endpoint", text: model.binding(for: \.lmStudio.endpoint))
            HStack(spacing: 10) {
                Button("Modelle abrufen") {
                    model.refreshRemoteModels(backend: .lmStudio)
                }
                if let err = model.lmStudioModelsError {
                    Text(err)
                        .font(.caption)
                        .foregroundStyle(.red)
                        .lineLimit(2)
                }
            }
            if model.lmStudioModels.isEmpty && model.lmStudioModelsError == nil {
                Text("Noch keine Modellliste abgerufen. Laufender LM-Studio-Server ben\u{F6}tigt.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(model.lmStudioModels) { entry in
                    remoteModelTile(entry: entry, isActive: isActiveLmStudio(entry))
                }
            }
        }
    }

    @ViewBuilder
    private func customLlmTile(entry: CustomLlmModel) -> some View {
        let isActive = model.settings.activePostProcessingBackend == .local
            && model.settings.activeCustomLlmId == entry.id
        let status = model.customLlmStatusList.first(where: { $0.id == entry.id })
        let needsDownload = status?.needsDownload ?? false
        let isDownloading = status?.isDownloading ?? false
        let isDownloaded = status?.isDownloaded ?? false

        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(entry.name)
                            .font(.body.weight(.medium))
                        if isActive {
                            Text("Aktiv")
                                .font(.caption2.weight(.semibold))
                                .padding(.vertical, 2)
                                .padding(.horizontal, 6)
                                .background(Color.accentColor.opacity(0.14), in: Capsule())
                                .foregroundStyle(Color.accentColor)
                        }
                    }
                    Text(status?.sourceLabel ?? entry.source.summaryText)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }

                Spacer()

                Button(isActive ? "Aktiv" : "Ausw\u{E4}hlen") {
                    model.postProcessingChoiceBinding.wrappedValue = .localCustom(id: entry.id)
                }
                .disabled(isActive)

                if needsDownload {
                    if isDownloaded {
                        Button("Datei l\u{F6}schen") {
                            model.deleteCustomLlmFile(id: entry.id)
                        }
                        .disabled(isDownloading)
                    } else {
                        Button(isDownloading ? "Lade..." : "Herunterladen") {
                            model.startCustomLlmDownload(id: entry.id)
                        }
                        .disabled(isDownloading)
                    }
                }

                Button("Entfernen") {
                    model.removeCustomLlm(id: entry.id)
                }
            }

            if isDownloading, let basisPoints = status?.progressBasisPoints {
                ProgressView(value: Double(basisPoints) / 10_000.0)
            }
        }
        .padding(.vertical, 2)
    }

    private func presentCustomLlmFilePicker() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = []
        panel.allowsOtherFileTypes = true
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.prompt = "Hinzuf\u{FC}gen"
        panel.title = "Eigenes Sprachmodell ausw\u{E4}hlen"
        panel.message = "GGUF- oder GGML-Modelldatei ausw\u{E4}hlen."

        guard panel.runModal() == .OK, let url = panel.url else {
            return
        }

        let name = url.deletingPathExtension().lastPathComponent
        model.addCustomLocalLlm(name: name, path: url.path)
    }

    private func isActiveOllama(_ entry: RemoteModelDTO) -> Bool {
        model.settings.activePostProcessingBackend == .ollama
            && model.settings.ollama.modelName == entry.name
    }

    private func isActiveLmStudio(_ entry: RemoteModelDTO) -> Bool {
        model.settings.activePostProcessingBackend == .lmStudio
            && model.settings.lmStudio.modelName == entry.name
    }

    @ViewBuilder
    private func remoteModelTile(entry: RemoteModelDTO, isActive: Bool) -> some View {
        HStack(spacing: 10) {
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(entry.name)
                        .font(.body.weight(.medium))
                    if isActive {
                        Text("Aktiv")
                            .font(.caption2.weight(.semibold))
                            .padding(.vertical, 2)
                            .padding(.horizontal, 6)
                            .background(Color.accentColor.opacity(0.14), in: Capsule())
                            .foregroundStyle(Color.accentColor)
                    }
                }
                Text(entry.summary)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Button(isActive ? "Aktiv" : "Ausw\u{E4}hlen") {
                switch entry.backend {
                case .ollama:
                    model.postProcessingChoiceBinding.wrappedValue = .ollamaModel(entry.name)
                case .lmStudio:
                    model.postProcessingChoiceBinding.wrappedValue = .lmStudioModel(entry.name)
                }
            }
            .disabled(isActive)
        }
        .padding(.vertical, 2)
    }

    @ViewBuilder
    private func whisperTile(preset: ModelPreset, status: ModelStatusDTO?) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(preset.displayName)
                        .font(.body.weight(.medium))
                    Text(preset.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }

                Spacer(minLength: 8)

                Text(preset.downloadSizeText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }

            if let status, status.isDownloading, let basisPoints = status.progressBasisPoints {
                ProgressView(value: Double(basisPoints) / 10_000.0)
            }

            HStack(spacing: 10) {
                Text(status?.summary ?? "Status unbekannt.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)

                Spacer()

                if status?.isDownloaded == true {
                    Button("Loeschen") {
                        model.deleteModel(preset: preset)
                    }
                    .disabled(status?.isDownloading == true)
                } else {
                    Button(status?.isDownloading == true ? "Lade..." : "Herunterladen") {
                        model.startModelDownload(preset: preset)
                    }
                    .disabled(status?.isDownloading == true)
                }
            }
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private func llmTile(preset: LlmPreset, status: LlmModelStatusDTO?) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(preset.displayName)
                            .font(.body.weight(.medium))
                        if status?.isLoaded == true {
                            Text("Geladen")
                                .font(.caption2.weight(.semibold))
                                .padding(.vertical, 2)
                                .padding(.horizontal, 6)
                                .background(Color.accentColor.opacity(0.14), in: Capsule())
                                .foregroundStyle(Color.accentColor)
                        }
                    }
                    Text(preset.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }

                Spacer(minLength: 8)

                Text(preset.approxSizeLabel)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }

            if let status, status.isDownloading, let basisPoints = status.progressBasisPoints {
                ProgressView(value: Double(basisPoints) / 10_000.0)
            }

            HStack(spacing: 10) {
                Text(status?.summary ?? "Status unbekannt.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)

                Spacer()

                if status?.isDownloaded == true {
                    Button("Loeschen") {
                        model.deleteLlmModel(preset: preset)
                    }
                    .disabled(status?.isDownloading == true)
                } else {
                    Button(status?.isDownloading == true ? "Lade..." : "Herunterladen") {
                        model.startLlmDownload(preset: preset)
                    }
                    .disabled(status?.isDownloading == true)
                }
            }
        }
        .padding(.vertical, 4)
    }
}

