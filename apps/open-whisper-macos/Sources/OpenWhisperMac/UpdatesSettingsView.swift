import SwiftUI

struct UpdatesSettingsView: View {
    let updaterController: UpdaterController

    var body: some View {
        Section {
            Toggle(isOn: Binding(
                get: { updaterController.automaticallyChecksForUpdates },
                set: { updaterController.automaticallyChecksForUpdates = $0 }
            )) {
                Text("Automatically check for updates", bundle: .module)
            }
            .disabled(!updaterController.isAvailable)

            Button {
                updaterController.checkForUpdates()
            } label: {
                Text("Check for updates now", bundle: .module)
            }
            .disabled(!updaterController.isAvailable)
        } header: {
            Text("Automatic updates", bundle: .module)
        }

        Section {
            if updaterController.isAvailable {
                Text("Open Whisper checks for new versions at launch and then every 24 hours. Updates download in the background and install the next time you restart.", bundle: .module)
                    .font(.callout)
                    .foregroundStyle(.secondary)
            } else {
                Text("Updates are only available in the installed .app (dev build).", bundle: .module)
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
        }
    }
}
