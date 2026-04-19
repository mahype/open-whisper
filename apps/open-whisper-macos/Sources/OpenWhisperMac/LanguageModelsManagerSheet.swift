import SwiftUI

enum PostProcessingChoice: Hashable, Identifiable {
    case localPreset(LlmPreset)
    case ollama
    case lmStudio

    var id: String {
        switch self {
        case .localPreset(let preset):
            return "local.\(preset.rawValue)"
        case .ollama:
            return "ollama"
        case .lmStudio:
            return "lmStudio"
        }
    }

    var label: String {
        switch self {
        case .localPreset(let preset):
            return "\(preset.displayName) (lokal)"
        case .ollama:
            return "Ollama"
        case .lmStudio:
            return "LM Studio"
        }
    }

    static var allChoices: [PostProcessingChoice] {
        LlmPreset.allCases.map { .localPreset($0) } + [.ollama, .lmStudio]
    }
}

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
    }

    @ViewBuilder
    private var transcriptionContent: some View {
        Section("Whisper-Presets") {
            if model.modelStatusList.isEmpty {
                ForEach(ModelPreset.allCases) { preset in
                    whisperTile(preset: preset, status: nil)
                }
            } else {
                ForEach(model.modelStatusList) { status in
                    let preset = ModelPreset(whisperModel: status.backendModelName)
                    whisperTile(preset: preset, status: status)
                }
            }
        }
    }

    @ViewBuilder
    private var postProcessingContent: some View {
        Section("Lokale Sprachmodelle") {
            if model.llmStatusList.isEmpty {
                ForEach(LlmPreset.allCases) { preset in
                    llmTile(preset: preset, status: nil)
                }
            } else {
                ForEach(model.llmStatusList) { status in
                    let preset = LlmPreset(displayLabel: status.displayLabel)
                    llmTile(preset: preset, status: status)
                }
            }
        }

        Section("Ollama") {
            TextField("Endpoint", text: model.binding(for: \.ollama.endpoint))
            TextField("Modellname", text: model.binding(for: \.ollama.modelName))
            Text("Hinweis: Remote-Modellliste wird in einem spaeteren Schritt direkt vom Endpoint abgefragt.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }

        Section("LM Studio") {
            TextField("Endpoint", text: model.binding(for: \.lmStudio.endpoint))
            TextField("Modellname", text: model.binding(for: \.lmStudio.modelName))
        }
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

private extension ModelPreset {
    init(whisperModel: String) {
        switch whisperModel {
        case "base": self = .light
        case "medium": self = .quality
        default: self = .standard
        }
    }
}

private extension LlmPreset {
    init(displayLabel: String) {
        if displayLabel.contains("E2B") || displayLabel.contains("1B") {
            self = .small
        } else if displayLabel.contains("26B") || displayLabel.contains("12B") {
            self = .large
        } else {
            self = .medium
        }
    }
}
