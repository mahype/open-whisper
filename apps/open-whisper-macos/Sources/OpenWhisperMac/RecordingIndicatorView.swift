import SwiftUI

@MainActor
final class RecordingLevelFeed: ObservableObject {
    static let barCount = 28
    static let pollingInterval: TimeInterval = 1.0 / 30.0
    static let levelGain: Float = 8.0

    @Published private(set) var bars: [Float] = Array(repeating: 0, count: RecordingLevelFeed.barCount)

    private let bridge = BridgeClient()
    private var timer: Timer?

    func start() {
        stop()
        timer = Timer.scheduledTimer(withTimeInterval: Self.pollingInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.tick()
            }
        }
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
    @StateObject private var feed = RecordingLevelFeed()

    var body: some View {
        HStack(spacing: 10) {
            Circle()
                .fill(Color.red)
                .frame(width: 8, height: 8)
                .shadow(color: Color.red.opacity(0.6), radius: 3)

            HStack(spacing: 3) {
                ForEach(Array(feed.bars.enumerated()), id: \.offset) { _, level in
                    Capsule()
                        .fill(Color.accentColor)
                        .frame(width: 3, height: barHeight(for: level))
                        .animation(.linear(duration: RecordingLevelFeed.pollingInterval), value: level)
                }
            }
            .frame(maxWidth: .infinity)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .frame(width: 220, height: 64)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(Color.primary.opacity(0.08), lineWidth: 1)
        )
        .onAppear { feed.start() }
        .onDisappear { feed.stop() }
    }

    private func barHeight(for level: Float) -> CGFloat {
        let normalized = min(1.0, max(0.0, CGFloat(level * RecordingLevelFeed.levelGain)))
        return max(3.0, normalized * 32.0)
    }
}
