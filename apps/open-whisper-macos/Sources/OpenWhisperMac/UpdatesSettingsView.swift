import SwiftUI

struct UpdatesSettingsView: View {
    let updaterController: UpdaterController

    var body: some View {
        Section("Automatische Updates") {
            Toggle("Automatisch nach Updates suchen", isOn: Binding(
                get: { updaterController.automaticallyChecksForUpdates },
                set: { updaterController.automaticallyChecksForUpdates = $0 }
            ))

            Button("Jetzt nach Updates suchen") {
                updaterController.checkForUpdates()
            }
        }

        Section {
            Text("""
            Open Whisper prüft beim Start und danach alle 24 Stunden auf neue \
            Versionen. Updates werden im Hintergrund heruntergeladen und installiert, \
            sobald du neu startest.
            """)
            .font(.callout)
            .foregroundStyle(.secondary)
        }
    }
}
