import Foundation

enum StartupBehavior: String, Codable, CaseIterable, Identifiable {
    case askOnFirstLaunch = "ask_on_first_launch"
    case launchAtLogin = "launch_at_login"
    case manualLaunch = "manual_launch"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .askOnFirstLaunch:
            return "Beim ersten Start fragen"
        case .launchAtLogin:
            return "Mit dem System starten"
        case .manualLaunch:
            return "Nur manuell starten"
        }
    }
}

enum TriggerMode: String, Codable, CaseIterable, Identifiable {
    case pushToTalk = "push_to_talk"
    case toggle

    var id: String { rawValue }

    var label: String {
        switch self {
        case .pushToTalk:
            return "Push-to-talk"
        case .toggle:
            return "Toggle"
        }
    }
}

enum WaveformStyle: String, CaseIterable, Identifiable {
    case centeredBars = "centered_bars"
    case line
    case envelope

    var id: String { rawValue }

    var label: String {
        switch self {
        case .centeredBars:
            return "Zentrierte Balken"
        case .line:
            return "Linie"
        case .envelope:
            return "Welle"
        }
    }
}

extension WaveformStyle: Codable {
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(String.self)
        self = WaveformStyle(rawValue: raw) ?? .centeredBars
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

enum WaveformColor: String, CaseIterable, Identifiable {
    case accent
    case blue
    case green
    case teal
    case orange
    case red
    case pink
    case purple

    var id: String { rawValue }

    var label: String {
        switch self {
        case .accent: return "Systemfarbe"
        case .blue: return "Blau"
        case .green: return "Gruen"
        case .teal: return "Tuerkis"
        case .orange: return "Orange"
        case .red: return "Rot"
        case .pink: return "Pink"
        case .purple: return "Violett"
        }
    }
}

extension WaveformColor: Codable {
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(String.self)
        self = WaveformColor(rawValue: raw) ?? .accent
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

enum ModelPreset: String, Codable, CaseIterable, Identifiable {
    case light
    case standard
    case quality

    var id: String { rawValue }

    var label: String {
        switch self {
        case .light:
            return "Klein"
        case .standard:
            return "Mittel"
        case .quality:
            return "Gross"
        }
    }

    var displayName: String {
        switch self {
        case .light:
            return "Whisper Base (klein)"
        case .standard:
            return "Whisper Small (mittel)"
        case .quality:
            return "Whisper Medium (gross)"
        }
    }

    var whisperModel: String {
        switch self {
        case .light:
            return "base"
        case .standard:
            return "small"
        case .quality:
            return "medium"
        }
    }

    var defaultFilename: String {
        switch self {
        case .light:
            return "ggml-base.bin"
        case .standard:
            return "ggml-small.bin"
        case .quality:
            return "ggml-medium.bin"
        }
    }

    var description: String {
        switch self {
        case .light:
            return "Kleines lokales Modell fuer schnelle Reaktion auf schwachen Rechnern."
        case .standard:
            return "Guter Standard fuer Alltag und Genauigkeit."
        case .quality:
            return "Groesseres Modell mit hoeherer Genauigkeit und mehr CPU-/RAM-Bedarf."
        }
    }

    var downloadSizeBytes: UInt64 {
        switch self {
        case .light:
            return 147_951_465
        case .standard:
            return 487_601_967
        case .quality:
            return 1_533_763_059
        }
    }

    var downloadSizeText: String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        formatter.allowedUnits = [.useMB, .useGB]
        formatter.includesUnit = true
        formatter.isAdaptive = true
        return formatter.string(fromByteCount: Int64(downloadSizeBytes))
    }
}

enum LlmPreset: String, Codable, CaseIterable, Identifiable {
    case small
    case medium
    case large

    var id: String { rawValue }

    var label: String {
        switch self {
        case .small: return "Klein"
        case .medium: return "Mittel"
        case .large: return "Gross"
        }
    }

    var displayName: String {
        switch self {
        case .small: return "Gemma 4 E2B (klein)"
        case .medium: return "Gemma 4 E4B (mittel)"
        case .large: return "Gemma 4 26B (gross)"
        }
    }

    var description: String {
        switch self {
        case .small:
            return "Kleines Sprachmodell (Gemma 4 E2B). Schnell und sparsam, laeuft auch auf 8 GB RAM."
        case .medium:
            return "Mittleres Sprachmodell (Gemma 4 E4B) als guter Standard fuer 16 GB RAM und mehr."
        case .large:
            return "Grosses Sprachmodell (Gemma 4 26B A4B, Mixture-of-Experts) mit bester Qualitaet, braucht 32 GB RAM oder mehr."
        }
    }

    var approxSizeLabel: String {
        switch self {
        case .small: return "ca. 3.5 GB"
        case .medium: return "ca. 5.4 GB"
        case .large: return "ca. 17 GB"
        }
    }

    var downloadSizeBytes: UInt64 {
        switch self {
        case .small: return 3_462_677_760
        case .medium: return 5_405_167_904
        case .large: return 17_035_037_632
        }
    }
}

enum ProviderKind: String, Codable, CaseIterable, Identifiable {
    case localWhisper = "local_whisper"
    case ollama
    case lmStudio = "lm_studio"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .localWhisper:
            return "Local Whisper"
        case .ollama:
            return "Ollama"
        case .lmStudio:
            return "LM Studio"
        }
    }
}

enum PostProcessingBackend: String, Codable, CaseIterable, Identifiable {
    case local
    case ollama
    case lmStudio = "lm_studio"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .local:
            return "Lokales Modell"
        case .ollama:
            return "Ollama"
        case .lmStudio:
            return "LM Studio"
        }
    }
}

enum RemoteModelBackend: String, Codable, CaseIterable, Identifiable {
    case ollama
    case lmStudio = "lm_studio"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .ollama: return "Ollama"
        case .lmStudio: return "LM Studio"
        }
    }
}

struct RemoteModelDTO: Codable, Identifiable, Hashable {
    var backend: RemoteModelBackend
    var name: String
    var summary: String

    var id: String { "\(backend.rawValue).\(name)" }
}

enum DiagnosticStatus: String, Codable {
    case ok
    case info
    case warning
    case error

    var label: String {
        switch self {
        case .ok:
            return "OK"
        case .info:
            return "Hinweis"
        case .warning:
            return "Warnung"
        case .error:
            return "Fehler"
        }
    }
}

struct ExternalProviderSettings: Codable, Equatable {
    var endpoint: String
    var modelName: String
}

enum CustomLlmSource: Codable, Equatable, Hashable {
    case localPath(path: String)
    case downloadUrl(url: String, filename: String)

    private enum CodingKeys: String, CodingKey {
        case kind
        case path
        case url
        case filename
    }

    private enum Kind: String, Codable {
        case localPath = "local_path"
        case downloadUrl = "download_url"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try container.decode(Kind.self, forKey: .kind)
        switch kind {
        case .localPath:
            let path = try container.decode(String.self, forKey: .path)
            self = .localPath(path: path)
        case .downloadUrl:
            let url = try container.decode(String.self, forKey: .url)
            let filename = try container.decode(String.self, forKey: .filename)
            self = .downloadUrl(url: url, filename: filename)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .localPath(let path):
            try container.encode(Kind.localPath, forKey: .kind)
            try container.encode(path, forKey: .path)
        case .downloadUrl(let url, let filename):
            try container.encode(Kind.downloadUrl, forKey: .kind)
            try container.encode(url, forKey: .url)
            try container.encode(filename, forKey: .filename)
        }
    }

    var summaryText: String {
        switch self {
        case .localPath(let path):
            return path
        case .downloadUrl(let url, _):
            return url
        }
    }
}

struct CustomLlmModel: Codable, Identifiable, Hashable, Equatable {
    var id: String
    var name: String
    var source: CustomLlmSource
}

struct ProcessingMode: Codable, Identifiable, Hashable {
    var id: String
    var name: String
    var prompt: String
    var postProcessingEnabled: Bool

    static let standard = ProcessingMode(
        id: "standard",
        name: "Standard",
        prompt: "",
        postProcessingEnabled: false
    )

    var postProcessingSummary: String {
        postProcessingEnabled
            ? "Nachverarbeitung aktiv"
            : "Direktes Diktat ohne Nachverarbeitung"
    }
}

struct TranscriptionLanguageOption: Identifiable, Hashable {
    let code: String
    let label: String

    var id: String { code }

    static let automatic = TranscriptionLanguageOption(code: "auto", label: "Automatisch")

    static let common: [TranscriptionLanguageOption] = [
        .automatic,
        TranscriptionLanguageOption(code: "de", label: "Deutsch"),
        TranscriptionLanguageOption(code: "en", label: "Englisch"),
        TranscriptionLanguageOption(code: "fr", label: "Franzoesisch"),
        TranscriptionLanguageOption(code: "es", label: "Spanisch"),
        TranscriptionLanguageOption(code: "it", label: "Italienisch"),
        TranscriptionLanguageOption(code: "nl", label: "Niederlaendisch"),
        TranscriptionLanguageOption(code: "pt", label: "Portugiesisch"),
        TranscriptionLanguageOption(code: "tr", label: "Tuerkisch"),
    ]

    static func option(for storedValue: String) -> TranscriptionLanguageOption? {
        let normalized = storedValue.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        if normalized.isEmpty || normalized == "auto" {
            return automatic
        }
        return common.first(where: { $0.code == normalized })
    }
}

struct AppSettings: Codable, Equatable {
    var onboardingCompleted: Bool
    var startupBehavior: StartupBehavior
    var inputDeviceName: String
    var hotkey: String
    var triggerMode: TriggerMode
    var transcriptionLanguage: String
    var insertTextAutomatically: Bool
    var insertDelayMs: UInt32
    var restoreClipboardAfterInsert: Bool
    var vadEnabled: Bool
    var vadThreshold: Float
    var vadSilenceMs: UInt32
    var showRecordingIndicator: Bool
    var waveformStyle: WaveformStyle
    var waveformColor: WaveformColor
    var localModel: ModelPreset
    var localModelPath: String
    var localLlm: LlmPreset
    var localLlmPath: String
    var localLlmAutoUnloadSecs: UInt32
    var activeProvider: ProviderKind
    var activePostProcessingBackend: PostProcessingBackend
    var activeCustomLlmId: String
    var customLlmModels: [CustomLlmModel]
    var ollama: ExternalProviderSettings
    var lmStudio: ExternalProviderSettings
    var modes: [ProcessingMode]
    var activeModeId: String

    static let `default` = AppSettings(
        onboardingCompleted: false,
        startupBehavior: .askOnFirstLaunch,
        inputDeviceName: "System Default",
        hotkey: "Ctrl+Shift+Space",
        triggerMode: .toggle,
        transcriptionLanguage: "auto",
        insertTextAutomatically: true,
        insertDelayMs: 120,
        restoreClipboardAfterInsert: true,
        vadEnabled: false,
        vadThreshold: 0.014,
        vadSilenceMs: 900,
        showRecordingIndicator: true,
        waveformStyle: .centeredBars,
        waveformColor: .accent,
        localModel: .standard,
        localModelPath: "",
        localLlm: .medium,
        localLlmPath: "",
        localLlmAutoUnloadSecs: 180,
        activeProvider: .localWhisper,
        activePostProcessingBackend: .local,
        activeCustomLlmId: "",
        customLlmModels: [],
        ollama: ExternalProviderSettings(endpoint: "http://127.0.0.1:11434", modelName: "whisper"),
        lmStudio: ExternalProviderSettings(endpoint: "http://127.0.0.1:1234", modelName: "openai/whisper-small"),
        modes: [.standard],
        activeModeId: "standard"
    )
}

struct DeviceDTO: Codable, Identifiable {
    var name: String
    var isSelected: Bool

    var id: String { name }
}

struct ModelStatusDTO: Codable, Identifiable {
    var presetLabel: String
    var backendModelName: String
    var path: String
    var summary: String
    var isDownloaded: Bool
    var isDownloading: Bool
    var progressBasisPoints: UInt16?
    var expectedSizeBytes: UInt64

    var id: String { backendModelName }

    static let empty = ModelStatusDTO(
        presetLabel: "Whisper Small (mittel)",
        backendModelName: "small",
        path: "",
        summary: "Noch kein Modellstatus geladen.",
        isDownloaded: false,
        isDownloading: false,
        progressBasisPoints: nil,
        expectedSizeBytes: ModelPreset.standard.downloadSizeBytes
    )
}

struct CustomLlmStatusDTO: Codable, Identifiable, Hashable {
    var id: String
    var name: String
    var sourceLabel: String
    var path: String
    var isDownloaded: Bool
    var isDownloading: Bool
    var isLoaded: Bool
    var needsDownload: Bool
    var progressBasisPoints: UInt16?
}

struct LlmModelStatusDTO: Codable, Identifiable {
    var presetLabel: String
    var displayLabel: String
    var path: String
    var summary: String
    var isDownloaded: Bool
    var isDownloading: Bool
    var isLoaded: Bool
    var progressBasisPoints: UInt16?
    var expectedSizeBytes: UInt64

    var id: String { presetLabel }
}

struct DiagnosticItemDTO: Codable, Identifiable {
    var title: String
    var status: DiagnosticStatus
    var problem: String
    var recommendation: String

    var id: String { title + problem }
}

struct DiagnosticsDTO: Codable {
    var summary: String
    var items: [DiagnosticItemDTO]

    static let empty = DiagnosticsDTO(summary: "Diagnose wird geladen.", items: [])
}

struct RecordingLevelsDTO: Codable {
    var levels: [Float]

    static let empty = RecordingLevelsDTO(levels: [])
}

struct RuntimeStatusDTO: Codable {
    var isRecording: Bool
    var isTranscribing: Bool
    var isPostProcessing: Bool
    var lastStatus: String
    var lastTranscript: String
    var dictationTriggerCount: UInt64
    var hotkeyRegistered: Bool
    var hotkeyText: String
    var startupSummary: String
    var providerSummary: String
    var activeModeName: String
    var onboardingCompleted: Bool

    static let empty = RuntimeStatusDTO(
        isRecording: false,
        isTranscribing: false,
        isPostProcessing: false,
        lastStatus: "Open Whisper wird gestartet.",
        lastTranscript: "",
        dictationTriggerCount: 0,
        hotkeyRegistered: false,
        hotkeyText: "Ctrl+Shift+Space",
        startupSummary: "Systemstart noch nicht synchronisiert.",
        providerSummary: "Local Whisper",
        activeModeName: "Standard",
        onboardingCompleted: false
    )
}
