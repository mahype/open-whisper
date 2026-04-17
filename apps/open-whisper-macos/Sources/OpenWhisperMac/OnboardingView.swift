import SwiftUI

struct OnboardingView: View {
    @ObservedObject var model: AppModel
    let onFinish: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Open Whisper einrichten")
                    .font(.largeTitle)
                Text("Tray-first, lokal, nativer macOS-Workflow. Das produktive Standard-Diktat bleibt auf den integrierten Whisper-Presets Klein, Mittel und Gross.")
                    .foregroundStyle(.secondary)
            }

            Text("Schritt \(model.onboardingStep + 1) von 4")
                .font(.headline)

            GroupBox {
                currentStep
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            HStack {
                Button("Zurueck") {
                    model.onboardingStep = max(0, model.onboardingStep - 1)
                }
                .disabled(model.onboardingStep == 0)

                Spacer()

                if model.onboardingStep == 3 {
                    Button("Setup abschliessen") {
                        model.completeOnboarding()
                        onFinish()
                    }
                    .keyboardShortcut(.defaultAction)
                } else {
                    Button("Weiter") {
                        model.onboardingStep = min(3, model.onboardingStep + 1)
                    }
                    .keyboardShortcut(.defaultAction)
                }
            }
        }
        .padding(24)
        .frame(minWidth: 680, minHeight: 560)
    }

    @ViewBuilder
    private var currentStep: some View {
        switch model.onboardingStep {
        case 0:
            VStack(alignment: .leading, spacing: 12) {
                Text("Willkommen")
                    .font(.title2)
                Text("Open Whisper lebt in der Menueleiste, reagiert auf einen globalen Hotkey und fuegt den diktierten Text direkt in die aktive App ein.")
                Text("Ollama und LM Studio bleiben optional. Standard ist lokales Whisper mit drei eingebauten Modellstufen.")
                    .foregroundStyle(.secondary)
            }
        case 1:
            Form {
                Picker("Mikrofon", selection: $model.settings.inputDeviceName) {
                    ForEach(deviceNames, id: \.self) { device in
                        Text(device).tag(device)
                    }
                }
                TextField("Globaler Hotkey", text: $model.settings.hotkey)
                Picker("Aufnahmemodus", selection: $model.settings.triggerMode) {
                    ForEach(TriggerMode.allCases) { mode in
                        Text(mode.label).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
                TextField("Sprache", text: $model.settings.transcriptionLanguage)
                HStack {
                    Button("Mikrofone neu laden") {
                        model.refreshDevices()
                    }
                    Spacer()
                }
            }
            .formStyle(.grouped)
        case 2:
            Form {
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
                    .foregroundStyle(.secondary)
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
                Picker("Systemstart", selection: $model.settings.startupBehavior) {
                    ForEach(StartupBehavior.allCases) { behavior in
                        Text(behavior.label).tag(behavior)
                    }
                }
            }
            .formStyle(.grouped)
        default:
            VStack(alignment: .leading, spacing: 12) {
                Text("Diagnose und Rechte")
                    .font(.title2)
                Text(model.diagnostics.summary)
                    .foregroundStyle(.secondary)
                ForEach(model.diagnostics.items) { item in
                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text(item.title)
                                .font(.headline)
                            Spacer()
                            Text(item.status.label)
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
    }

    private var deviceNames: [String] {
        let names = model.devices.map(\.name)
        if names.isEmpty {
            return [model.settings.inputDeviceName]
        }
        return names
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
