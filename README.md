# Open Whisper

Cross-platform voice-to-text desktop app in Rust.

## Current milestone

- Rust workspace with a native desktop shell.
- Shared core crate for settings and provider configuration.
- Local persistence for onboarding and settings.
- UI for startup behavior, hotkey, input device, local Whisper presets, Ollama and LM Studio.
- Tray menu, close-to-tray behavior and global hotkey registration.
- Audio device detection, microphone capture, silence-based stop and local Whisper transcription.
- Clipboard-based insertion into the active app with simulated paste shortcut.
- Model download management with progress, local cleanup and default Whisper download URLs.
- Autostart handling with hidden background launch on supported desktop systems.

## Planned next steps

1. Optional Ollama/LM Studio transcription and post-processing path.
2. Installer/onboarding flow for first-run model setup.
3. Hardening for Linux desktop variants and permission diagnostics.
4. Better permission checks for microphone, accessibility and simulated paste.
5. Packaging/signing for Windows, macOS and Linux releases.
