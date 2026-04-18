# Open Whisper

**Dictate anywhere on your Mac — 100% local.**

Press a hotkey, speak, and your words land in whatever app has focus: mail, chat, your editor, the browser. Transcription runs on your machine with [whisper.cpp](https://github.com/ggerganov/whisper.cpp). Nothing leaves your Mac unless you deliberately configure a remote provider.

> **Status:** macOS 14+ is stable. Windows and Linux UI shells are on the roadmap — the Rust core and bridge already compile cross-platform.

---

## How it works

1. **Press** your global hotkey (push-to-talk or toggle).
2. **Speak** — Open Whisper records from your chosen mic.
3. **Done** — the transcription is pasted into the focused app.

Open Whisper lives in your menu bar. No Dock icon, no window clutter.

---

## Install (Users)

**Requires macOS 14+ on Apple Silicon or Intel.**

1. Download the [latest DMG](https://github.com/mahype/open-whisper/releases/latest).
2. Drag **Open Whisper.app** into **Applications** and launch it.
3. Follow the onboarding — mic, model download, hotkey, autostart.

Need permissions help, autostart setup, or uninstall steps? → [docs/INSTALL.md](docs/INSTALL.md)

| Platform | Status |
| --- | --- |
| macOS 14+ (Apple Silicon & Intel) | Stable — [download](https://github.com/mahype/open-whisper/releases/latest) |
| Windows | Planned |
| Linux | Planned |

---

## Features

- **Fully local transcription** — your voice never leaves the machine.
- **Global hotkey** with push-to-talk or toggle mode.
- **Menu-bar-only** UI — stays out of the way.
- **Automatic paste** into the focused app via simulated keystroke.
- **Guided onboarding** for mic, model, and startup preferences.
- **Autostart at login** via native macOS Login Items.
- **Optional remote providers** (Ollama, LM Studio) for post-processing or remote transcription — off by default.

---

## Run it locally (Developers)

Prereqs: **Rust 1.88+**, **Swift 6 / Xcode 16+**, **Xcode Command Line Tools**, and **CMake** (`brew install cmake`).

```bash
git clone git@github.com:mahype/open-whisper.git
cd open-whisper
./scripts/dev.sh
```

`dev.sh` is the fast inner loop: it builds the Rust bridge (`cargo build -p open-whisper-bridge`) and launches the Swift app via SwiftPM. No bundle, no signing — ideal for iterating.

### Build a real `.app` bundle

```bash
./scripts/build-macos-app.sh
open "dist/Open Whisper.app"
```

Universal (Apple Silicon + Intel), release build, ad-hoc signed — good for running on your own Mac. For signed + notarized releases, see [docs/RELEASING.md](docs/RELEASING.md).

Full toolchain, debugging tips, and project walk-through: → [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)

---

## Project layout

```
open-whisper/
├── apps/open-whisper-macos/       # SwiftUI + AppKit menu bar app
├── crates/
│   ├── open-whisper-bridge/       # JSON-over-FFI static library (staticlib + rlib)
│   └── open-whisper-core/         # Shared Rust domain types (settings, presets, DTOs)
├── scripts/                       # Dev, build, sign, DMG packaging
└── docs/                          # Long-form documentation
```

How the Rust core, FFI bridge, and Swift UI fit together: → [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

---

## Documentation

| Doc | What's inside |
| --- | --- |
| [INSTALL.md](docs/INSTALL.md) | Install, permissions, autostart, uninstall |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | Dev setup, build scripts, debugging |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Rust core ↔ FFI bridge ↔ Swift UI |
| [RELEASING.md](docs/RELEASING.md) | Tagging, signing, notarization, publishing |

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
