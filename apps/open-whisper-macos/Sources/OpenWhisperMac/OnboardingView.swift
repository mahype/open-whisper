import SwiftUI

struct OnboardingView: View {
    @ObservedObject var model: AppModel
    let onFinish: () -> Void

    var body: some View {
        HStack(spacing: 20) {
            StepRail(currentStep: model.onboardingStep)
                .frame(width: 220)

            VStack(alignment: .leading, spacing: 18) {
                DetailHeader(
                    title: "Open Whisper einrichten",
                    subtitle: "Tray-first, lokal und fuer den Alltag vorbereitet. Standard bleibt lokales Whisper mit Whisper Base, Whisper Small und Whisper Medium."
                )

                ScrollView {
                    VStack(alignment: .leading, spacing: 18) {
                        currentStep
                    }
                    .padding(.trailing, 4)
                }

                footer
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .padding(24)
        .frame(minWidth: 840, minHeight: 560)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    @ViewBuilder
    private var currentStep: some View {
        switch model.onboardingStep {
        case 0:
            VStack(alignment: .leading, spacing: 18) {
                AppCard(title: "Willkommen", subtitle: "Wie Open Whisper im Alltag arbeitet") {
                    Text("Open Whisper lebt in der Menueleiste, reagiert auf einen globalen Hotkey und fuegt den diktierten Text direkt in die aktive App ein.")
                    Text("Ollama und LM Studio bleiben optional. Das Standard-Diktat nutzt immer lokales Whisper mit Whisper Base, Whisper Small und Whisper Medium.")
                        .foregroundStyle(.secondary)
                }

                AppCard(title: "Was du gleich festlegst", subtitle: "Die produktiven Basis-Einstellungen fuer den ersten Start") {
                    MetricRow(label: "Mikrofon", value: model.settings.inputDeviceName)
                    MetricRow(label: "Hotkey", value: hotkeyDisplayString(model.settings.hotkey))
                    MetricRow(label: "Modell", value: model.selectedModelDisplayName)
                    MetricRow(label: "Systemstart", value: model.settings.startupBehavior.label)
                }
            }
        case 1:
            VStack(alignment: .leading, spacing: 18) {
                AppCard(title: "Audioquelle", subtitle: "Mikrofon, Sprache und Aufnahmemodus") {
                    Picker("Mikrofon", selection: model.binding(for: \.inputDeviceName)) {
                        ForEach(deviceNames, id: \.self) { device in
                            Text(device).tag(device)
                        }
                    }

                    Picker("Aufnahmemodus", selection: model.binding(for: \.triggerMode)) {
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
                        Button("Mikrofone neu laden") {
                            model.refreshDevices()
                        }
                        Spacer()
                    }
                }

                AppCard(title: "Globaler Hotkey", subtitle: "Wird nach dem Setup in den Settings gespeichert und registriert") {
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
            }
        case 2:
            VStack(alignment: .leading, spacing: 18) {
                AppCard(title: "Lokales Standardmodell", subtitle: "Die drei festen Whisper-Modelle fuer produktives Diktat") {
                    VStack(alignment: .leading, spacing: 12) {
                        ForEach(ModelPreset.allCases) { preset in
                            ModelPresetTile(preset: preset, isSelected: model.settings.localModel == preset) {
                                model.choosePreset(preset)
                            }
                        }
                    }
                }

                AppCard(title: "Downloadstatus", subtitle: "Nur Status und Aktionen, keine Dateipfade") {
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

                AppCard(title: "Startverhalten", subtitle: "Wie sich die App nach dem Login verhalten soll") {
                    Picker("Systemstart", selection: model.binding(for: \.startupBehavior)) {
                        ForEach(StartupBehavior.allCases) { behavior in
                            Text(behavior.label).tag(behavior)
                        }
                    }

                    Toggle("Text automatisch in aktive App einfuegen", isOn: model.binding(for: \.insertTextAutomatically))
                    Toggle("Clipboard nach Einfuegen wiederherstellen", isOn: model.binding(for: \.restoreClipboardAfterInsert))
                }
            }
        default:
            VStack(alignment: .leading, spacing: 18) {
                AppCard(title: "Diagnose und Rechte", subtitle: "Kompakte Hinweise fuer Mikrofon, Hotkey und Systemintegration") {
                    Text(model.diagnostics.summary)
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
    }

    private var footer: some View {
        HStack {
            Button("Zurueck") {
                model.onboardingStep = max(0, model.onboardingStep - 1)
            }
            .disabled(model.onboardingStep == 0)

            Spacer()

            if model.onboardingStep == 3 {
                Button("Setup abschliessen") {
                    if model.completeOnboarding() {
                        onFinish()
                    }
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
            } else {
                Button("Weiter") {
                    model.onboardingStep = min(3, model.onboardingStep + 1)
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(.top, 8)
    }

    private var deviceNames: [String] {
        let names = model.devices.map(\.name)
        if names.isEmpty {
            return [model.settings.inputDeviceName]
        }
        return names
    }
}
