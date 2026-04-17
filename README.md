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

## Planned next steps

1. Autostart handling in setup and settings.
2. Model download management with progress and cleanup.
3. Optional Ollama/LM Studio transcription and post-processing path.
4. Installer/onboarding flow for first-run model setup.
5. Hardening for Linux desktop variants and permission diagnostics.
