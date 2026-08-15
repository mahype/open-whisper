import XCTest

@testable import TorroWhisper

/// Deleting post-processing modes: any entry may be deleted — including the
/// last one. The settings must stay consistent with the bridge's
/// `AppSettings::normalize()`, which never allows an empty mode list: deleting
/// the last entry restores the default and turns post-processing off.
final class ProcessingModeDeletionTests: XCTestCase {
    private let secondMode = ProcessingMode(
        id: "email",
        name: "E-Mail",
        prompt: "Rewrite as a friendly e-mail."
    )

    private func makeSettings(
        modes: [ProcessingMode],
        activeModeId: String,
        postProcessingEnabled: Bool = true
    ) -> AppSettings {
        var settings = AppSettings.default
        settings.modes = modes
        settings.activeModeId = activeModeId
        settings.postProcessingEnabled = postProcessingEnabled
        return settings
    }

    func testDeleteInactiveModeKeepsActiveSelection() {
        var settings = makeSettings(modes: [.cleanup, secondMode], activeModeId: "cleanup")

        settings.deleteMode("email")

        XCTAssertEqual(settings.modes, [.cleanup])
        XCTAssertEqual(settings.activeModeId, "cleanup")
        XCTAssertTrue(settings.postProcessingEnabled)
    }

    func testDeleteActiveModeFallsBackToFirstRemaining() {
        var settings = makeSettings(modes: [.cleanup, secondMode], activeModeId: "email")

        settings.deleteMode("email")

        XCTAssertEqual(settings.modes, [.cleanup])
        XCTAssertEqual(settings.activeModeId, "cleanup")
        XCTAssertTrue(settings.postProcessingEnabled)
    }

    func testDeleteLastModeRestoresDefaultAndDisablesPostProcessing() {
        var renamedDefault = ProcessingMode.cleanup
        renamedDefault.name = "Accidentally dictated name"
        var settings = makeSettings(modes: [renamedDefault], activeModeId: "cleanup")

        settings.deleteMode("cleanup")

        XCTAssertEqual(settings.modes, [.cleanup])
        XCTAssertEqual(settings.activeModeId, "cleanup")
        XCTAssertFalse(settings.postProcessingEnabled)
    }

    func testDeleteLastNonDefaultModeRestoresDefault() {
        var settings = makeSettings(modes: [secondMode], activeModeId: "email")

        settings.deleteMode("email")

        XCTAssertEqual(settings.modes, [.cleanup])
        XCTAssertEqual(settings.activeModeId, "cleanup")
        XCTAssertFalse(settings.postProcessingEnabled)
    }

    func testDeleteUnknownModeIsNoOp() {
        let original = makeSettings(modes: [.cleanup, secondMode], activeModeId: "email")
        var settings = original

        settings.deleteMode("does-not-exist")

        XCTAssertEqual(settings, original)
    }
}
