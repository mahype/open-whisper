# Open Whisper

Cross-platform voice-to-text desktop app in Rust.

## Current milestone

- Rust workspace with a native desktop shell.
- Shared core crate for settings and provider configuration.
- Local persistence for onboarding and settings.
- UI for startup behavior, hotkey, input device, local Whisper presets, Ollama and LM Studio.
- Tray menu, close-to-tray behavior and global hotkey registration.

## Planned next steps

1. Microphone capture and voice activity detection.
2. Local Whisper execution and model download flow.
3. Text insertion into the active application per platform.
4. Autostart handling in setup and settings.
5. Model download management with progress and cleanup.
