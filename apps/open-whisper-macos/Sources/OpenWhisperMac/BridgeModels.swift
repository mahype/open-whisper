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

struct ExternalProviderSettings: Codable {
    var endpoint: String
    var modelName: String
}

struct AppSettings: Codable {
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
    var localModel: ModelPreset
    var localModelPath: String
    var activeProvider: ProviderKind
    var ollama: ExternalProviderSettings
    var lmStudio: ExternalProviderSettings

    static let `default` = AppSettings(
        onboardingCompleted: false,
        startupBehavior: .askOnFirstLaunch,
        inputDeviceName: "System Default",
        hotkey: "Ctrl+Shift+Space",
        triggerMode: .pushToTalk,
        transcriptionLanguage: "de",
        insertTextAutomatically: true,
        insertDelayMs: 120,
        restoreClipboardAfterInsert: true,
        vadEnabled: true,
        vadThreshold: 0.014,
        vadSilenceMs: 900,
        localModel: .standard,
        localModelPath: "",
        activeProvider: .localWhisper,
        ollama: ExternalProviderSettings(endpoint: "http://127.0.0.1:11434", modelName: "whisper"),
        lmStudio: ExternalProviderSettings(endpoint: "http://127.0.0.1:1234", modelName: "openai/whisper-small")
    )
}

struct DeviceDTO: Codable, Identifiable {
    var name: String
    var isSelected: Bool

    var id: String { name }
}

struct ModelStatusDTO: Codable {
    var presetLabel: String
    var backendModelName: String
    var path: String
    var summary: String
    var isDownloaded: Bool
    var isDownloading: Bool
    var progressBasisPoints: UInt16?

    static let empty = ModelStatusDTO(
        presetLabel: "Mittel",
        backendModelName: "small",
        path: "",
        summary: "Noch kein Modellstatus geladen.",
        isDownloaded: false,
        isDownloading: false,
        progressBasisPoints: nil
    )
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

struct RuntimeStatusDTO: Codable {
    var isRecording: Bool
    var isTranscribing: Bool
    var lastStatus: String
    var lastTranscript: String
    var dictationTriggerCount: UInt64
    var hotkeyRegistered: Bool
    var hotkeyText: String
    var startupSummary: String
    var providerSummary: String
    var onboardingCompleted: Bool

    static let empty = RuntimeStatusDTO(
        isRecording: false,
        isTranscribing: false,
        lastStatus: "Open Whisper wird gestartet.",
        lastTranscript: "",
        dictationTriggerCount: 0,
        hotkeyRegistered: false,
        hotkeyText: "Ctrl+Shift+Space",
        startupSummary: "Systemstart noch nicht synchronisiert.",
        providerSummary: "Local Whisper",
        onboardingCompleted: false
    )
}
