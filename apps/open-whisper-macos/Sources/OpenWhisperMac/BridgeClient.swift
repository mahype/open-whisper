import Foundation
import OpenWhisperBridgeFFI

struct BridgeError: LocalizedError {
    let message: String

    var errorDescription: String? { message }
}

final class BridgeClient {
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder

    init() {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        self.decoder = decoder

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        self.encoder = encoder
    }

    func loadSettings() throws -> AppSettings {
        try decodeResponse(from: ow_load_settings())
    }

    func saveSettings(_ settings: AppSettings) throws -> String {
        try encodeAndCall(settings, function: ow_save_settings)
    }

    func listInputDevices() throws -> [DeviceDTO] {
        try decodeResponse(from: ow_list_input_devices())
    }

    func getModelStatus() throws -> ModelStatusDTO {
        try decodeResponse(from: ow_get_model_status())
    }

    func startModelDownload(preset: ModelPreset?) throws -> String {
        try encodeAndCall(["preset": preset?.rawValue], function: ow_start_model_download)
    }

    func deleteModel(preset: ModelPreset?) throws -> String {
        try encodeAndCall(["preset": preset?.rawValue], function: ow_delete_model)
    }

    func runPermissionDiagnostics() throws -> DiagnosticsDTO {
        try decodeResponse(from: ow_run_permission_diagnostics())
    }

    func startDictation() throws -> String {
        try decodeResponse(from: ow_start_dictation())
    }

    func stopDictation() throws -> String {
        try decodeResponse(from: ow_stop_dictation())
    }

    func getRuntimeStatus() throws -> RuntimeStatusDTO {
        try decodeResponse(from: ow_get_runtime_status())
    }

    func getRecordingLevels() throws -> RecordingLevelsDTO {
        try decodeResponse(from: ow_get_recording_levels())
    }

    func validateHotkey(_ hotkey: String) throws -> String {
        try encodeAndCall(["hotkey": hotkey], function: ow_validate_hotkey)
    }

    private func encodeAndCall<Input: Encodable, Output: Decodable>(
        _ input: Input,
        function: (UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?
    ) throws -> Output {
        let payload = try encoder.encode(input)
        guard let json = String(data: payload, encoding: .utf8) else {
            throw BridgeError(message: "JSON-Payload konnte nicht erzeugt werden.")
        }

        return try json.withCString { pointer in
            try decodeResponse(from: function(pointer))
        }
    }

    private func decodeResponse<T: Decodable>(from rawPointer: UnsafeMutablePointer<CChar>?) throws -> T {
        guard let rawPointer else {
            throw BridgeError(message: "Bridge hat keinen Rueckgabewert geliefert.")
        }
        defer { ow_string_free(rawPointer) }

        let json = String(cString: rawPointer)
        guard let data = json.data(using: .utf8) else {
            throw BridgeError(message: "Bridge lieferte kein gueltiges UTF-8.")
        }

        let envelope = try decoder.decode(Envelope<T>.self, from: data)
        if envelope.ok, let value = envelope.value {
            return value
        }

        throw BridgeError(message: envelope.error ?? "Unbekannter Bridge-Fehler.")
    }
}

private struct Envelope<Value: Decodable>: Decodable {
    let ok: Bool
    let value: Value?
    let error: String?
}
