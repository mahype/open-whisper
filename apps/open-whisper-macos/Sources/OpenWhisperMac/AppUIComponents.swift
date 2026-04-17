import SwiftUI

enum SettingsSection: String, CaseIterable, Identifiable {
    case recording
    case model
    case startup
    case providers
    case diagnostics

    var id: String { rawValue }

    var title: String {
        switch self {
        case .recording:
            return "Aufnahme"
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

    var subtitle: String {
        switch self {
        case .recording:
            return "Mikrofon, Hotkey und Sprache"
        case .model:
            return "Whisper Base, Small und Medium"
        case .startup:
            return "Autostart und Diktatverhalten"
        case .providers:
            return "Ollama und LM Studio"
        case .diagnostics:
            return "Rechte, Status und Hinweise"
        }
    }

    var symbolName: String {
        switch self {
        case .recording:
            return "mic.fill"
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

struct AppCard<Content: View>: View {
    let title: String
    let subtitle: String?
    @ViewBuilder var content: Content

    init(title: String, subtitle: String? = nil, @ViewBuilder content: () -> Content) {
        self.title = title
        self.subtitle = subtitle
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.headline)
                if let subtitle, !subtitle.isEmpty {
                    Text(subtitle)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
            }

            content
        }
        .padding(16)
        .background(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color.primary.opacity(0.06), lineWidth: 1)
        )
    }
}

struct DetailHeader: View {
    let title: String
    let subtitle: String

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.system(size: 28, weight: .semibold))
            Text(subtitle)
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
    }
}

struct StatusBadge: View {
    let title: String
    let value: String
    var accent: Color = .accentColor

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title.uppercased())
                .font(.caption2)
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.primary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, 10)
        .padding(.horizontal, 12)
        .background(accent.opacity(0.10), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}

struct MetricRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer(minLength: 12)
            Text(value)
                .multilineTextAlignment(.trailing)
        }
        .font(.subheadline)
    }
}

struct InlineStatusPill: View {
    let title: String
    let value: String
    var accent: Color = .secondary

    var body: some View {
        HStack(spacing: 6) {
            Text(title)
                .foregroundStyle(.secondary)
            Text(value)
                .fontWeight(.semibold)
        }
        .font(.caption)
        .padding(.vertical, 6)
        .padding(.horizontal, 10)
        .background(accent.opacity(0.10), in: Capsule())
    }
}

struct ModelPresetTile: View {
    let preset: ModelPreset
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(preset.displayName)
                        .font(.headline)
                        .foregroundStyle(.primary)
                    Text(preset.description)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.leading)
                }

                Spacer(minLength: 12)

                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .font(.title3)
                    .foregroundStyle(isSelected ? Color.accentColor : Color.secondary.opacity(0.7))
            }
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 14, style: .continuous)
                    .fill(isSelected ? Color.accentColor.opacity(0.10) : Color(nsColor: .textBackgroundColor))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 14, style: .continuous)
                    .stroke(isSelected ? Color.accentColor.opacity(0.45) : Color.primary.opacity(0.06), lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
    }
}

struct DiagnosticStatusBadge: View {
    let status: DiagnosticStatus

    var body: some View {
        Text(status.label)
            .font(.caption.weight(.semibold))
            .padding(.vertical, 5)
            .padding(.horizontal, 8)
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
            VStack(alignment: .leading, spacing: 10) {
                Text(item.problem)
                Text(item.recommendation)
                    .foregroundStyle(.secondary)
            }
            .padding(.top, 8)
        } label: {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 3) {
                    Text(item.title)
                        .font(.headline)
                    Text(item.problem)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }
                Spacer()
                DiagnosticStatusBadge(status: item.status)
            }
        }
        .padding(16)
        .background(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
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
        VStack(alignment: .leading, spacing: 18) {
            Text("Einrichtung")
                .font(.headline)

            ForEach(Array(steps.enumerated()), id: \.offset) { index, title in
                HStack(alignment: .top, spacing: 12) {
                    ZStack {
                        Circle()
                            .fill(index == currentStep ? Color.accentColor : Color.secondary.opacity(0.18))
                            .frame(width: 28, height: 28)
                        Text("\(index + 1)")
                            .font(.subheadline.weight(.semibold))
                            .foregroundStyle(index == currentStep ? Color.white : Color.secondary)
                    }

                    VStack(alignment: .leading, spacing: 2) {
                        Text(title)
                            .font(.subheadline.weight(index == currentStep ? .semibold : .regular))
                        Text(index < currentStep ? "Fertig" : index == currentStep ? "Aktiver Schritt" : "Ausstehend")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Spacer()
        }
        .padding(20)
        .frame(maxHeight: .infinity, alignment: .top)
        .background(
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .fill(Color(nsColor: .underPageBackgroundColor))
        )
    }
}
