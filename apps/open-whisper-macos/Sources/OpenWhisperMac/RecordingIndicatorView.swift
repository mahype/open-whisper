import SwiftUI

enum IndicatorPhase: Equatable {
    case recording
    case transcribing
    case postProcessing
    case modelNotReady(label: String, progress: Double?, isDownloading: Bool)
}

@MainActor
final class RecordingLevelFeed: ObservableObject {
    static let barCount = 28
    static let pollingInterval: TimeInterval = 1.0 / 30.0
    static let levelGain: Float = 2.8
    static let noiseFloor: Float = 0.002

    @Published private(set) var bars: [Float] = Array(repeating: 0, count: RecordingLevelFeed.barCount)

    private let bridge = BridgeClient()
    private var timer: Timer?

    func start() {
        stop()
        let newTimer = Timer(timeInterval: Self.pollingInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.tick()
            }
        }
        RunLoop.main.add(newTimer, forMode: .common)
        timer = newTimer
    }

    func stop() {
        timer?.invalidate()
        timer = nil
        bars = Array(repeating: 0, count: Self.barCount)
    }

    private func tick() {
        guard let levels = try? bridge.getRecordingLevels().levels else {
            return
        }

        let slice = Array(levels.suffix(Self.barCount))
        if slice.count == Self.barCount {
            bars = slice
        } else {
            var padded = Array(repeating: Float(0), count: Self.barCount - slice.count)
            padded.append(contentsOf: slice)
            bars = padded
        }
    }
}

struct RecordingIndicatorView: View {
    let phase: IndicatorPhase
    var style: WaveformStyle = .centeredBars
    var color: WaveformColor = .accent
    var modeName: String = ""
    @StateObject private var feed = RecordingLevelFeed()

    var body: some View {
        VStack(spacing: 4) {
            HStack(spacing: 10) {
                statusDot
                content
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            if shouldShowModeLabel {
                modeLabel
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .frame(width: 260, height: 86)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(Color.primary.opacity(0.08), lineWidth: 1)
        )
        .onAppear { syncFeed() }
        .onChange(of: phase) { _ in syncFeed() }
        .onDisappear { feed.stop() }
    }

    private var statusDot: some View {
        Circle()
            .fill(statusDotColor)
            .frame(width: 8, height: 8)
            .shadow(color: phase == .recording ? Color.red.opacity(0.6) : .clear, radius: 3)
    }

    private var statusDotColor: Color {
        switch phase {
        case .recording: return .red
        case .transcribing: return .secondary
        case .postProcessing: return .purple
        case .modelNotReady: return .orange
        }
    }

    private var shouldShowModeLabel: Bool {
        guard !modeName.isEmpty else { return false }
        switch phase {
        case .recording, .postProcessing: return true
        case .transcribing, .modelNotReady: return false
        }
    }

    @ViewBuilder
    private var content: some View {
        switch phase {
        case .recording:
            waveform
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .transcribing:
            processingRow(text: "Transkribiere...")
        case .postProcessing:
            processingRow(text: "Nachbearbeitung...")
        case let .modelNotReady(label, progress, isDownloading):
            modelNotReadyRow(label: label, progress: progress, isDownloading: isDownloading)
        }
    }

    @ViewBuilder
    private func modelNotReadyRow(label: String, progress: Double?, isDownloading: Bool) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Aufnahme nicht m\u{F6}glich")
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(.primary)
            if let progress, isDownloading {
                let percent = Int((progress * 100.0).rounded())
                Text("Modell l\u{E4}dt: \(label) (\(percent)%)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                ProgressView(value: progress)
            } else if isDownloading {
                Text("Modell l\u{E4}dt: \(label)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                ProgressView()
                    .progressViewStyle(.linear)
            } else {
                Text("Modell \(label) fehlt. Bitte in den Einstellungen laden.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var modeLabel: some View {
        Text(modeName)
            .font(.system(size: 10, weight: .medium))
            .foregroundStyle(.secondary)
            .lineLimit(1)
            .truncationMode(.tail)
            .frame(maxWidth: .infinity, alignment: .center)
    }

    @ViewBuilder
    private var waveform: some View {
        switch style {
        case .centeredBars:
            centeredBars
        case .line:
            lineWave
        case .envelope:
            envelopeWave
        }
    }

    private var tint: Color { color.swiftUIColor }

    private var centeredBars: some View {
        HStack(spacing: 3) {
            ForEach(Array(feed.bars.enumerated()), id: \.offset) { _, level in
                Capsule()
                    .fill(tint)
                    .frame(width: 3, height: barHeight(for: level))
                    .animation(.linear(duration: RecordingLevelFeed.pollingInterval), value: level)
            }
        }
        .frame(maxWidth: .infinity)
    }

    private var lineWave: some View {
        GeometryReader { geo in
            ZStack(alignment: .center) {
                Rectangle()
                    .fill(tint.opacity(0.18))
                    .frame(height: 1)

                envelopePath(in: geo.size, direction: .up)
                    .stroke(tint,
                            style: StrokeStyle(lineWidth: 1.5, lineCap: .round, lineJoin: .round))
                envelopePath(in: geo.size, direction: .down)
                    .stroke(tint.opacity(0.85),
                            style: StrokeStyle(lineWidth: 1.5, lineCap: .round, lineJoin: .round))
            }
            .frame(width: geo.size.width, height: geo.size.height)
            .animation(.linear(duration: RecordingLevelFeed.pollingInterval), value: feed.bars)
            .drawingGroup()
        }
    }

    private var envelopeWave: some View {
        GeometryReader { geo in
            filledEnvelopePath(in: geo.size)
                .fill(tint)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .animation(.linear(duration: RecordingLevelFeed.pollingInterval), value: feed.bars)
                .drawingGroup()
        }
    }

    private enum EnvelopeDirection { case up, down }

    private func envelopePath(in size: CGSize, direction: EnvelopeDirection) -> Path {
        let width = size.width
        let height = size.height
        let mid = height / 2
        let bars = feed.bars
        let count = bars.count
        guard count > 0 else { return Path() }
        let step = count > 1 ? width / CGFloat(count - 1) : 0

        return Path { path in
            path.move(to: CGPoint(x: 0, y: mid))
            for (i, level) in bars.enumerated() {
                let amp = normalizedLevel(level) * mid
                let x = CGFloat(i) * step
                let y: CGFloat = direction == .up ? (mid - amp) : (mid + amp)
                path.addLine(to: CGPoint(x: x, y: y))
            }
            path.addLine(to: CGPoint(x: width, y: mid))
        }
    }

    private func filledEnvelopePath(in size: CGSize) -> Path {
        let width = size.width
        let height = size.height
        let mid = height / 2
        let bars = feed.bars
        let count = bars.count
        guard count > 0 else { return Path() }
        let step = count > 1 ? width / CGFloat(count - 1) : 0

        return Path { path in
            path.move(to: CGPoint(x: 0, y: mid))
            for (i, level) in bars.enumerated() {
                let amp = normalizedLevel(level) * mid
                path.addLine(to: CGPoint(x: CGFloat(i) * step, y: mid - amp))
            }
            path.addLine(to: CGPoint(x: width, y: mid))
            for (i, level) in bars.enumerated().reversed() {
                let amp = normalizedLevel(level) * mid
                path.addLine(to: CGPoint(x: CGFloat(i) * step, y: mid + amp))
            }
            path.closeSubpath()
        }
    }

    private func processingRow(text: String) -> some View {
        HStack(spacing: 10) {
            ProgressView()
                .controlSize(.small)
            Text(text)
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(.primary)
            Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func syncFeed() {
        if phase == .recording {
            feed.start()
        } else {
            feed.stop()
        }
    }

    private func barHeight(for level: Float) -> CGFloat {
        return max(2.0, normalizedLevel(level) * 36.0)
    }

    private func normalizedLevel(_ level: Float) -> CGFloat {
        guard level.isFinite else { return 0 }
        let cleaned = max(0.0, level - RecordingLevelFeed.noiseFloor)
        let curved = sqrt(cleaned) * RecordingLevelFeed.levelGain
        return min(1.0, max(0.0, CGFloat(curved)))
    }
}

extension WaveformColor {
    var swiftUIColor: Color {
        switch self {
        case .accent: return .accentColor
        case .blue: return .blue
        case .green: return .green
        case .teal: return .teal
        case .orange: return .orange
        case .red: return .red
        case .pink: return .pink
        case .purple: return .purple
        }
    }
}
