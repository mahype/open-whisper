# Open Whisper

Cross-platform voice-to-text desktop app in Rust.

## Current milestone

- Rust workspace with a native desktop shell.
- Shared core crate for settings and provider configuration.
- Local persistence for onboarding and settings.
- UI for startup behavior, hotkey, input device, local Whisper presets, Ollama and LM Studio.
- Tray menu, close-to-tray behavior and global hotkey registration.
- Audio device detection, microphone capture, silence-based stop and local Whisper transcription.

## Planned next steps

1. Text insertion into the active application per platform.
2. Autostart handling in setup and settings.
3. Model download management with progress and cleanup.
4. Optional Ollama/LM Studio transcription and post-processing path.
5. Installer/onboarding flow for first-run model setup.
