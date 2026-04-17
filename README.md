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
- First-run onboarding with audio setup, model choice and startup selection.
- Runtime permission diagnostics for microphone, tray, global hotkey and platform-specific hints.
- Modernized desktop UI with a cleaner onboarding flow, dashboard cards and a custom light theme.

## Planned next steps

1. Optional Ollama/LM Studio transcription and post-processing path.
2. Hardening for Linux desktop variants and deeper OS permission probes.
3. Better permission checks for microphone, accessibility and simulated paste.
4. Packaging/signing for Windows, macOS and Linux releases.
5. Distribution-friendly installer/update flow.
