import AppKit
import SwiftUI

struct UpdatesSettingsView: View {
    let updaterController: UpdaterController
    @State private var autoCheck: Bool

    init(updaterController: UpdaterController) {
        self.updaterController = updaterController
        _autoCheck = State(initialValue: updaterController.automaticallyChecksForUpdates)
    }

    var body: some View {
        Section("Automatische Updates") {
            Toggle("Automatisch nach Updates suchen", isOn: Binding(
                get: { autoCheck },
                set: { newValue in
                    autoCheck = newValue
                    updaterController.automaticallyChecksForUpdates = newValue
                }
            ))

            HStack {
                Text("Manuell prüfen:")
                Spacer()
                Button("Jetzt nach Updates suchen") {
                    updaterController.checkForUpdates()
                }
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
