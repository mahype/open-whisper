# Open Whisper

Tray-first voice-to-text for your desktop. Speak a shortcut, get your words inserted into any app. Runs locally with [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — nothing leaves your machine unless you explicitly configure a remote provider.

> Status: **macOS** is the first supported platform. Windows and Linux UI shells are planned — the Rust core and bridge are already cross-platform.

---

## Install (end users)

| Platform | Status | Download |
| --- | --- | --- |
| macOS 14+ (Apple Silicon & Intel) | Stable | [Latest release](https://github.com/mahype/open-whisper/releases/latest) |
| Windows | Planned | — |
| Linux | Planned | — |

**Quick start (macOS):** Download the `.dmg`, drag `Open Whisper.app` to `/Applications`, launch it, grant microphone + accessibility permissions, and pick your hotkey. See [docs/INSTALL.md](docs/INSTALL.md) for the full walk-through including autostart, permission troubleshooting, and uninstall.

---

## Features

- **Local transcription** via whisper.cpp — models run on your CPU/GPU, no network required.
- **Global hotkey** with push-to-talk or toggle modes.
- **Menu bar only** — no Dock icon, stays out of the way.
- **Automatic text insertion** into the focused app via simulated paste.
- **Onboarding** that guides you through mic setup, model download, and startup preference.
- **Optional remote providers** (Ollama, LM Studio) for post-processing or remote transcription, configured in Settings.
- **Autostart at login** with hidden launch, managed through macOS Login Items.

---

## Documentation

- [docs/INSTALL.md](docs/INSTALL.md) — End-user installation, permissions, autostart, uninstall
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — Dev setup, build the `.app` bundle locally, debugging
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — How the Rust core, FFI bridge, and Swift UI fit together
- [docs/RELEASING.md](docs/RELEASING.md) — Tagging, signing, notarization, publishing a release

---

## Quick start for developers

```bash
git clone git@github.com:mahype/open-whisper.git
cd open-whisper
./scripts/dev.sh       # builds the Rust bridge, runs the Swift app via SPM
```

To build a proper `.app` bundle locally:

```bash
./scripts/build-macos-app.sh
open dist/Open\ Whisper.app
```

Full toolchain and troubleshooting in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

---

## Project layout

```
open-whisper/
├── apps/
│   └── open-whisper-macos/        # SwiftUI/AppKit menu bar app
├── crates/
│   ├── open-whisper-bridge/       # JSON-over-FFI static library (staticlib + rlib)
│   └── open-whisper-core/         # Shared Rust domain types (settings, presets, DTOs)
├── scripts/                       # Dev and build scripts
└── docs/                          # Long-form documentation
```

---

## Roadmap

- [ ] Native UI shells for Windows and Linux on top of the existing Rust bridge
- [ ] Deeper permission probes for accessibility and input monitoring
- [ ] In-app model browser and bootstrap flow
- [ ] Auto-update channel (Sparkle or GitHub-based)
- [ ] Optional cloud transcription providers

---

## License

MIT — see [LICENSE](LICENSE).
