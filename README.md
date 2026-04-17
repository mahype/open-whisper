# Open Whisper

Tray-first voice-to-text desktop app with a Rust core and a native macOS menu bar app.

## Current milestone

- Shared Rust core crate for settings, presets and provider configuration.
- Rust bridge crate with JSON-based FFI for native UI shells.
- Local persistence for onboarding and settings.
- Native macOS menu bar app in SwiftUI/AppKit with dedicated onboarding and settings windows.
- UI for startup behavior, hotkey, input device, local Whisper presets, Ollama and LM Studio.
- Tray/menu bar integration, close-to-tray behavior and global hotkey registration.
- Audio device detection, microphone capture, silence-based stop and local Whisper transcription.
- Clipboard-based insertion into the active app with simulated paste shortcut.
- Model download management with progress, local cleanup and default Whisper download URLs.
- Autostart handling with hidden background launch on supported desktop systems.
- First-run onboarding with audio setup, model choice and startup selection.
- Runtime permission diagnostics for microphone, tray, global hotkey and platform-specific hints.
- Structured runtime permission diagnostics exposed as DTOs for native shells.

## Run locally

- Native macOS menu bar app: `./scripts/run-macos-native.sh`

## Planned next steps

1. Optional Ollama/LM Studio transcription and post-processing path.
2. Deeper macOS permission probes for microphone, accessibility and input monitoring.
3. Native UI shells for Windows and Linux on top of the same Rust bridge.
4. Packaging, signing and updater flow for production releases.
5. Distribution-friendly installer and model bootstrap flow.
