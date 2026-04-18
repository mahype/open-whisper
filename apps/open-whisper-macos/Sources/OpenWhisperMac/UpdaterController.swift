import Foundation
import Sparkle

/// Thin wrapper around Sparkle's standard updater controller.
/// Holds the Sparkle instance for the lifetime of the app; discarding the
/// controller silently stops background update checks.
@MainActor
final class UpdaterController {
    private let controller: SPUStandardUpdaterController

    init() {
        self.controller = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: nil,
            userDriverDelegate: nil
        )
    }

    func checkForUpdates() {
        controller.checkForUpdates(nil)
    }

    var automaticallyChecksForUpdates: Bool {
        get { controller.updater.automaticallyChecksForUpdates }
        set { controller.updater.automaticallyChecksForUpdates = newValue }
    }
}
