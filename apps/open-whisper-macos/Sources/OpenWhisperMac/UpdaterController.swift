import Foundation
import Sparkle

/// Thin wrapper around Sparkle's standard updater controller.
/// Holds the Sparkle instance for the lifetime of the app; discarding the
/// controller silently stops background update checks.
///
/// Sparkle is only started when the app is launched from a real `.app` bundle
/// with a populated `Info.plist`. During `swift run` / `scripts/dev.sh` there
/// is no bundle identity or `SUFeedURL`, so starting Sparkle would surface the
/// "Unable to Check For Updates" alert on every launch.
@MainActor
final class UpdaterController {
    private let controller: SPUStandardUpdaterController?

    init() {
        if UpdaterController.isPackagedApp {
            self.controller = SPUStandardUpdaterController(
                startingUpdater: true,
                updaterDelegate: nil,
                userDriverDelegate: nil
            )
        } else {
            self.controller = nil
            print("Sparkle disabled: running outside a packaged .app bundle")
        }
    }

    var isAvailable: Bool { controller != nil }

    func checkForUpdates() {
        controller?.checkForUpdates(nil)
    }

    var automaticallyChecksForUpdates: Bool {
        get { controller?.updater.automaticallyChecksForUpdates ?? false }
        set { controller?.updater.automaticallyChecksForUpdates = newValue }
    }

    private static var isPackagedApp: Bool {
        guard Bundle.main.bundleIdentifier != nil else { return false }
        guard Bundle.main.bundlePath.hasSuffix(".app") else { return false }
        return Bundle.main.object(forInfoDictionaryKey: "SUFeedURL") != nil
    }
}
