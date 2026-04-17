import SwiftUI

enum SettingsSection: String, CaseIterable, Identifiable {
    case recording
    case modes
    case model
    case startup
    case providers
    case diagnostics

    var id: String { rawValue }

    var title: String {
        switch self {
        case .recording:
            return "Aufnahme"
        case .modes:
            return "Modi"
        case .model:
            return "Sprachmodell"
        case .startup:
            return "Start & Verhalten"
        case .providers:
            return "Optionale Provider"
        case .diagnostics:
            return "Diagnose"
        }
    }

    var symbolName: String {
        switch self {
        case .recording:
            return "mic.fill"
        case .modes:
            return "square.text.square"
        case .model:
            return "square.stack.3d.up.fill"
        case .startup:
            return "power.circle.fill"
        case .providers:
            return "server.rack"
        case .diagnostics:
            return "checklist"
        }
    }
}

struct ModeListTile: View {
    let mode: ProcessingMode
    let isSelected: Bool
    let isActive: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .foregroundStyle(isSelected ? Color.accentColor : Color.secondary.opacity(0.7))

                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(mode.name)
                            .font(.body.weight(.medium))
                            .foregroundStyle(.primary)
                        if isActive {
                            Text("Aktiv")
                                .font(.caption2.weight(.semibold))
                                .padding(.vertical, 2)
                                .padding(.horizontal, 6)
                                .background(Color.accentColor.opacity(0.14), in: Capsule())
                                .foregroundStyle(Color.accentColor)
                        }
                    }
                    Text(mode.postProcessingSummary)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer(minLength: 8)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
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
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
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
        "Modell & Start",
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
