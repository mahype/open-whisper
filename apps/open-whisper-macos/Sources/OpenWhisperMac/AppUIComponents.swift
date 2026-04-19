import AppKit
import SwiftUI

enum SettingsSection: String, CaseIterable, Identifiable {
    case recording
    case modes
    case languageModels = "language_models"
    case startup
    case updates
    case diagnostics
    case help

    var id: String { rawValue }

    var title: String {
        switch self {
        case .recording:
            return "Aufnahme"
        case .modes:
            return "Nachbearbeitung"
        case .languageModels:
            return "Sprachmodelle"
        case .startup:
            return "Start & Verhalten"
        case .updates:
            return "Updates"
        case .diagnostics:
            return "Diagnose"
        case .help:
            return "Hilfe"
        }
    }

    var symbolName: String {
        switch self {
        case .recording:
            return "mic.fill"
        case .modes:
            return "square.text.square"
        case .languageModels:
            return "brain.head.profile"
        case .startup:
            return "power.circle.fill"
        case .updates:
            return "arrow.triangle.2.circlepath"
        case .diagnostics:
            return "checklist"
        case .help:
            return "questionmark.circle"
        }
    }
}

struct ModeListTile: View {
    let mode: ProcessingMode
    let isActive: Bool
    let canDelete: Bool
    let onActivate: () -> Void
    let onEdit: () -> Void
    let onDelete: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: isActive ? "largecircle.fill.circle" : "circle")
                .font(.body)
                .foregroundStyle(isActive ? Color.accentColor : Color.secondary.opacity(0.7))
                .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 2) {
                Text(mode.name)
                    .font(.body.weight(.medium))
                    .foregroundStyle(.primary)
                Text(mode.prompt.isEmpty ? "Kein Prompt hinterlegt" : mode.prompt)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 8)

            Button(action: onEdit) {
                Image(systemName: "pencil")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.borderless)
            .help("Nachbearbeitung bearbeiten")

            Button(action: onDelete) {
                Image(systemName: "trash")
                    .foregroundStyle(canDelete ? .secondary : Color.secondary.opacity(0.35))
            }
            .buttonStyle(.borderless)
            .disabled(!canDelete)
            .help(canDelete ? "Nachbearbeitung loeschen" : "Mindestens eine Nachbearbeitung muss bestehen bleiben")
        }
        .contentShape(Rectangle())
        .onTapGesture { onActivate() }
        .onHover { hovering in
            if hovering {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
        }
    }
}

struct PostProcessingOffTile: View {
    let isActive: Bool
    let onActivate: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: isActive ? "largecircle.fill.circle" : "circle")
                .font(.body)
                .foregroundStyle(isActive ? Color.accentColor : Color.secondary.opacity(0.7))
                .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 2) {
                Text("Aus")
                    .font(.body.weight(.medium))
                    .foregroundStyle(.primary)
                Text("Transkription wird unverändert übernommen")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 8)
        }
        .contentShape(Rectangle())
        .onTapGesture { onActivate() }
        .onHover { hovering in
            if hovering {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
        }
    }
}

struct ModelPresetTile: View {
    let preset: ModelPreset
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .foregroundStyle(isSelected ? Color.accentColor : Color.secondary.opacity(0.7))

                VStack(alignment: .leading, spacing: 2) {
                    Text(preset.displayName)
                        .font(.body.weight(.medium))
                        .foregroundStyle(.primary)
                    Text(preset.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer(minLength: 8)

                Text("ca. \(preset.downloadSizeText)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

struct ModeEditorSheet: View {
    @ObservedObject var model: AppModel
    let onDone: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text("Nachbearbeitung bearbeiten")
                    .font(.title3.weight(.semibold))
                Spacer()
            }

            Form {
                Section {
                    TextField("Name", text: model.modeBinding(for: \.name))
                }

                Section("Prompt") {
                    TextEditor(text: model.modeBinding(for: \.prompt))
                        .font(.body)
                        .frame(minHeight: 180)
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
                        .listRowInsets(EdgeInsets(top: 4, leading: 0, bottom: 4, trailing: 0))
                }

                Section("Sprachmodell") {
                    Picker("Modell", selection: model.modeChoiceBinding()) {
                        Text("Standard (global)")
                            .tag(Optional<PostProcessingChoice>.none)
                        ForEach(model.availablePostProcessingChoices) { choice in
                            Text(model.postProcessingChoicePickerLabel(choice))
                                .tag(Optional(choice))
                        }
                    }

                    Text(modelHintText)
                        .font(.caption)
                        .foregroundStyle(.secondary)
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
        .frame(minWidth: 460, idealWidth: 520, minHeight: 380, idealHeight: 440)
    }

    private var modelHintText: String {
        if let choice = model.modeChoiceBinding().wrappedValue {
            return "Dieses Profil nutzt: \(model.postProcessingChoiceLabel(choice))."
        }
        let global = model.postProcessingChoiceBinding.wrappedValue
        return "Nutzt globales Modell: \(model.postProcessingChoiceLabel(global))."
    }
}

struct DiagnosticStatusBadge: View {
    let status: DiagnosticStatus

    var body: some View {
        Text(status.label)
            .font(.caption.weight(.semibold))
            .padding(.vertical, 3)
            .padding(.horizontal, 7)
            .background(backgroundColor.opacity(0.14), in: Capsule())
            .foregroundStyle(backgroundColor)
    }

    private var backgroundColor: Color {
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

struct DiagnosticDisclosureCard: View {
    let item: DiagnosticItemDTO

    var body: some View {
        DisclosureGroup {
            VStack(alignment: .leading, spacing: 6) {
                Text(item.problem)
                    .font(.caption)
                Text(item.recommendation)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .padding(.top, 6)
        } label: {
            HStack(spacing: 10) {
                Text(item.title)
                    .font(.body.weight(.medium))
                Spacer()
                DiagnosticStatusBadge(status: item.status)
            }
        }
    }
}

struct StepRail: View {
    let currentStep: Int

    private let steps = [
        "Willkommen",
        "Audio & Hotkey",
        "Sprachmodelle",
        "Start & Verhalten",
        "Diagnose",
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("Einrichtung")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .padding(.horizontal, 16)
                .padding(.top, 16)
                .padding(.bottom, 8)

            ForEach(Array(steps.enumerated()), id: \.offset) { index, title in
                HStack(spacing: 10) {
                    ZStack {
                        Circle()
                            .fill(index == currentStep ? Color.accentColor : Color.secondary.opacity(0.18))
                            .frame(width: 20, height: 20)
                        if index < currentStep {
                            Image(systemName: "checkmark")
                                .font(.system(size: 10, weight: .bold))
                                .foregroundStyle(.white)
                        } else {
                            Text("\(index + 1)")
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(index == currentStep ? Color.white : Color.secondary)
                        }
                    }

                    Text(title)
                        .font(.subheadline)
                        .fontWeight(index == currentStep ? .semibold : .regular)
                        .foregroundStyle(index == currentStep ? Color.primary : Color.secondary)

                    Spacer()
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 6)
            }

            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(Color(nsColor: .underPageBackgroundColor))
    }
}
