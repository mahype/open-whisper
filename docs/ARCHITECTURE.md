# Architecture

Open Whisper is split into a **shared Rust core**, an **FFI bridge** that exposes JSON-over-C to native UIs, and **platform-specific UI shells** (currently just macOS). The UI layers are kept thin: everything stateful — settings, dictation, model management, hotkeys — lives in the bridge.

## High-level diagram

```
┌─────────────────────────────────────────────────────────┐
│  apps/open-whisper-macos  (SwiftUI + AppKit menu bar)   │
│                                                         │
│  OpenWhisperMacApp → AppDelegate → SettingsView, …      │
│       │                                                 │
│       ▼                                                 │
│  BridgeClient  (Swift wrapper around the C functions)   │
└───────────┬─────────────────────────────────────────────┘
            │   C FFI: JSON-in, JSON-out strings
            │   Header: Bridge/OpenWhisperBridgeFFI.h
            ▼
┌─────────────────────────────────────────────────────────┐
│  crates/open-whisper-bridge  (staticlib + rlib)         │
│                                                         │
│  lib.rs          FFI entry points, BridgeRuntime        │
│  dictation.rs    cpal capture + whisper-rs transcription│
│  model_manager   Whisper model download & cleanup       │
│  autostart.rs    auto-launch wrapper (Dev fallback)     │
│  settings_store  settings.json read/write               │
│  text_inserter   arboard + enigo (clipboard + paste)    │
│  post_processing Ollama / LM Studio HTTP clients        │
│  permission_…    TCC / platform-specific probes         │
└───────────┬─────────────────────────────────────────────┘
            │ pure-Rust types & enums
            ▼
┌─────────────────────────────────────────────────────────┐
│  crates/open-whisper-core  (no_std-friendly domain)     │
│                                                         │
│  AppSettings, ModelPreset, StartupBehavior,             │
│  TriggerMode, DeviceDto, RuntimeStatusDto, …            │
└─────────────────────────────────────────────────────────┘
```

## Crates

### `open-whisper-core`

Pure-Rust domain types shared between the bridge and any future shells or tools. No I/O, no OS calls — just `serde`-friendly DTOs, enums, and configuration structs.

Key types:

- `AppSettings` — the root of everything the user configures.
- `StartupBehavior` — `AskOnFirstLaunch`, `LaunchAtLogin`, `ManualLaunch`.
- `TriggerMode` — `PushToTalk`, `Toggle`.
- `ModelPreset`, `DeviceDto`, `RuntimeStatusDto`, `RecordingLevelsDto`, `DiagnosticsDto`.

### `open-whisper-bridge`

Built as both `staticlib` (for Swift linkage) and `rlib` (for Rust-side testing). All public surface goes through `extern "C"` functions in `src/lib.rs` that take and return UTF-8 JSON strings.

Module responsibilities:

| Module | Responsibility |
| --- | --- |
| [lib.rs](../crates/open-whisper-bridge/src/lib.rs) | FFI entry points, the `BridgeRuntime` aggregate that owns all subsystems |
| [dictation.rs](../crates/open-whisper-bridge/src/dictation.rs) | Mic capture via `cpal`, silence detection, whisper.cpp transcription |
| [model_manager.rs](../crates/open-whisper-bridge/src/model_manager.rs) | Download, list, and delete whisper models |
| [autostart.rs](../crates/open-whisper-bridge/src/autostart.rs) | `auto-launch` crate wrapper; writes a `LaunchAgent` plist on macOS, XDG autostart on Linux, registry on Windows. Used as a fallback when the app is **not** running from a `.app` bundle. |
| [settings_store.rs](../crates/open-whisper-bridge/src/settings_store.rs) | Reads/writes `~/Library/Application Support/open-whisper/settings.json` |
| [text_inserter.rs](../crates/open-whisper-bridge/src/text_inserter.rs) | `arboard` (clipboard) + `enigo` (simulated paste) |
| [post_processing.rs](../crates/open-whisper-bridge/src/post_processing.rs) | Ollama/LM Studio HTTP clients for post-processing the raw transcript |
| [permission_diagnostics.rs](../crates/open-whisper-bridge/src/permission_diagnostics.rs) | Platform-specific permission probing (TCC on macOS) |

### `apps/open-whisper-macos`

Swift Package (SPM) producing a single executable, `OpenWhisperMac`. Uses SwiftUI for onboarding/settings windows, AppKit for the menu bar integration. Links to `libopen_whisper_bridge.a` from the target directory via unsafe linker flags in `Package.swift`.

Key files:

| File | Responsibility |
| --- | --- |
| [OpenWhisperMacApp.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/OpenWhisperMacApp.swift) | `@main` entry point |
| [AppDelegate.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/AppDelegate.swift) | Menu bar icon, window lifecycle, login-item bootstrap |
| [BridgeClient.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/BridgeClient.swift) | Typed wrapper around every `ow_*` FFI call |
| [BridgeModels.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/BridgeModels.swift) | `Codable` DTOs matching the Rust side |
| [AppModel.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/AppModel.swift) | Observable state for the UI |
| [SettingsView.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/SettingsView.swift), [OnboardingView.swift](../apps/open-whisper-macos/Sources/OpenWhisperMac/OnboardingView.swift) | SwiftUI windows |

## FFI contract

All functions in [OpenWhisperBridgeFFI.h](../apps/open-whisper-macos/Bridge/OpenWhisperBridgeFFI.h) follow the same shape:

```c
char *ow_do_something(const char *input_json);   // or no args
void  ow_string_free(char *raw);
```

- **Input**: a UTF-8 `char *` to JSON, or `NULL` if the function takes no args.
- **Output**: a newly allocated UTF-8 `char *` to JSON. Ownership transfers to the caller.
- **Freeing**: the caller **must** call `ow_string_free` with the returned pointer. Don't use `free(3)` — the allocator may differ.
- **Errors**: errors are encoded inside the JSON response (typically a top-level `error` or `status` field), not via a separate error channel.

### Thread-safety

The `BridgeRuntime` lives in a `thread_local! RefCell` on the bridge side. All FFI calls from Swift must happen from the **same thread** (today: the Swift main thread / AppKit event loop). If you need concurrent access from Swift, serialize the calls on a single queue.

## Runtime state and persistence

| Location | Content |
| --- | --- |
| `~/Library/Application Support/open-whisper/settings.json` | `AppSettings` — hotkey, input device, model, post-processing, startup behavior |
| `~/Library/Application Support/open-whisper/models/` | Downloaded whisper `.bin` files |
| `~/Library/LaunchAgents/open-whisper.plist` | *Dev fallback* — written by the `auto-launch` crate when the app runs outside a `.app` bundle |
| macOS Login Items database (`sfltool dumpbtm`) | *Production* — registered via `SMAppService` from the Swift side when the app runs from a signed bundle |

The bridge is the source of truth for settings; the Swift UI reads via `ow_load_settings` and writes via `ow_save_settings`. The UI does not keep its own copy.

## Autostart: two paths, one user-facing switch

macOS offers two autostart mechanisms, and Open Whisper uses both depending on how it was launched:

1. **`SMAppService.mainApp`** (production, macOS 13+) — used when the executable lives inside `Open Whisper.app/Contents/MacOS/`. The Swift layer calls `register()`/`unregister()` directly; the user-facing switch in Settings routes to this path.
2. **`LaunchAgent` plist** (dev fallback) — used when running via `swift run` during development. The Rust `auto-launch` crate writes `~/Library/LaunchAgents/open-whisper.plist` pointing at the raw executable.

The Swift UI picks the mechanism based on the `autostart_mechanism` field returned by the bridge. Users never see the distinction — they just see *Launch at login* in Settings.

## Cross-platform outlook

The bridge already has `#[cfg(target_os = …)]` blocks for Linux and Windows in `autostart.rs` and `permission_diagnostics.rs`. What's missing is a UI shell for each. The same `libopen_whisper_bridge` static library and the same FFI contract will back those shells — only the UI layer changes.
