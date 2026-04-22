# Linux

The Linux shell ([apps/open-whisper-linux/](../apps/open-whisper-linux/)) is GTK4 + libadwaita and is being built out in stages to reach feature parity with the macOS app. Today it ships:

- Main window with a derived status badge, three info cards (mode / model / hotkey), a dictate toggle, a settings button, and a hamburger menu.
- Settings window skeleton with 7 navigation pages (Recording, Post-processing, Language models, Start & behavior, Updates, Diagnostics, Help). Only *Updates* has real content today; the rest are placeholders for the upcoming stages.
- StatusNotifierItem tray (KDE/Xfce/Cinnamon/Budgie/MATE; GNOME requires the *AppIndicator and KStatusNotifierItem Support* extension).

Not yet implemented: hotkey recorder, mode editor, model download UI, recording HUD, onboarding wizard, system integration for auto-updates. These live in the staged roadmap maintained in the internal plan file.

---

## For users

Binary releases for Linux are **not published yet** — the UI is still filling in and we don't want to ship something half-working. Users who want to try the current state should follow the *For developers* section below.

When we do ship, the first target is a Flatpak bundle (see [Packaging](#packaging)).

### Runtime permissions

Same three capability areas as macOS, but flavored for Linux desktops:

| Capability | Required for | Notes |
| --- | --- | --- |
| **Microphone** | Recording | ALSA/PipeWire/PulseAudio — whatever the system exposes to `cpal`. No explicit permission prompt on most desktops. |
| **Global hotkey** | Trigger dictation from any focused app | Wayland uses the XDG `org.freedesktop.portal.GlobalShortcuts` portal (via `ashpd`); X11 uses direct key grabs. |
| **Text insertion** | Paste the transcript into the focused app | `enigo` simulates a paste keystroke; falls back to clipboard when the compositor blocks synthetic input. |

### GNOME: enable the tray

Vanilla GNOME ships without a StatusNotifierWatcher, so the tray icon won't appear. Install the [AppIndicator and KStatusNotifierItem Support](https://extensions.gnome.org/extension/615/appindicator-support/) extension, or rely on the main window alone. The tray code probes the session bus at startup and skips itself with a log warning if no watcher is registered — so missing tray ≠ missing window.

---

## For developers

### System packages

Two dev packages beyond a standard build toolchain:

| Distro | Packages |
| --- | --- |
| **Debian / Ubuntu / AnduinOS** | `libgtk-4-dev libadwaita-1-dev libasound2-dev libdbus-1-dev libxkbcommon-dev` |
| **Fedora / RHEL** | `gtk4-devel libadwaita-devel alsa-lib-devel dbus-devel libxkbcommon-devel` |
| **Arch** | `gtk4 libadwaita alsa-lib dbus libxkbcommon` |

Also required: **Rust ≥ 1.88** (edition 2024 + transitive sys crates), **CMake** (whisper.cpp + llama.cpp build scripts), and **libclang** from LLVM (bindgen dependency for `llama-cpp-sys-2`). On systems where the distro doesn't ship a recent enough libclang, Linuxbrew's `brew install llvm` is the simplest fallback — that's the path [scripts/dev-linux.sh](../scripts/dev-linux.sh) assumes.

> **Do not substitute brew-GTK4 for system GTK4.** Homebrew's GTK4/libadwaita on Linux has an ABI mismatch that corrupts widget measurements (`GtkImage` reports `i32::MIN` baselines, GTK then requests a 225561-pixel-tall Cairo surface, the allocation fails, the window never renders). Use the distro's packages. Linuxbrew is fine for LLVM only. See [Known issues](#known-issues) for the log fingerprint.

### Build & run

```bash
cd open-whisper
./scripts/dev-linux.sh
```

The script is thin; it just sets the envs listed below and calls `cargo run -p open-whisper-linux`. The first build compiles `whisper.cpp` and `llama.cpp` via CMake and takes several minutes. Subsequent runs reuse the cache.

### Environment variables set by the dev script

| Variable | Why |
| --- | --- |
| `LIBCLANG_PATH=/home/linuxbrew/.linuxbrew/lib` | `llama-cpp-sys-2` uses bindgen; bindgen needs `libclang.so`. Linuxbrew's `llvm` formula provides it. If your distro libclang is new enough, unset this. |
| `GSK_RENDERER=cairo` | GTK's default GL scene-graph renderer triggers a GPU hang + SIGKILL on NVIDIA + Wayland without the proprietary driver in the loader path. Cairo is software-only and safe. |
| `GDK_DISABLE=gl,vulkan` | Belt-and-braces for the renderer switch; prevents GDK from falling back to GL behind GSK's back. |
| `LIBGL_ALWAYS_SOFTWARE=1` | Force Mesa's llvmpipe for any remaining GL probes. |
| `RUST_LOG=open_whisper_bridge=info,open_whisper_linux=debug` | Debug level surfaces the bridge-call timing traces — any bridge call > 100 ms logs a `WARN slow bridge call` so regressions are visible. |

Override any of them by exporting before running: `GSK_RENDERER=gl ./scripts/dev-linux.sh` uses hardware GL on a well-configured host with Mesa or open-source NVIDIA drivers.

---

## Known issues

Each entry: **symptom → root cause → fix**. These are the traps we hit while bringing the shell up on AnduinOS 1.4.2 (Ubuntu 25.10) with NVIDIA + Wayland + GNOME.

### Link error: `rust-lld: unable to find library -lasound`

**Cause.** The runtime `libasound.so.2` is present on most desktops (any audio works), but the link-time symlink `libasound.so` only comes with the dev package.

**Fix.** `sudo apt install libasound2-dev` (or distro equivalent).

### SIGKILL during window creation (`Getötet` / `Killed`)

**Symptom.** App logs `libEGL warning: egl: failed to create dri2 screen` shortly after startup, then the process dies with SIGKILL. No panic, no Rust trace.

**Cause.** GTK's default GL renderer tries to initialise EGL via the NVIDIA driver; in Flatpak-style environments or hosts where Mesa can't locate the proprietary driver, the context creation hangs the GPU. The kernel terminates the hung client.

**Fix.** Force software rendering — `GSK_RENDERER=cairo`, `GDK_DISABLE=gl,vulkan`, `LIBGL_ALWAYS_SOFTWARE=1` (already in [scripts/dev-linux.sh](../scripts/dev-linux.sh)). For distribution, bundle the appropriate `org.freedesktop.Platform.GL.nvidia-*` Flatpak extension or ship a renderer-selection switch.

### Window appears but is unresponsive

**Symptom.** Main window renders, but dragging or clicking is ignored; compositor eventually shows *App not responding*. Log contains:

```
Gtk-WARNING: GtkImage reported baselines of minimum -2147483648 and natural -2147483648, but sizes of minimum 16 and natural 16.
Gtk-WARNING: GtkBox reported min height 225561 and natural height 50 in measure() — natural size must be >= min size
Gdk-CRITICAL: Unable to create Cairo image surface: invalid value (typically too big) for the size of the input
```

**Cause.** Linking against Homebrew's libadwaita + GTK4 on Linux. The brew builds are ABI-mismatched for the Linux runtime; icon widgets return `i32::MIN` as their baseline, which poisons the surrounding widget measurements. GTK tries to allocate a Cairo surface with an impossible height, fails, and the window is never paintable.

**Fix.** Use the distro packages (`libgtk-4-dev`, `libadwaita-1-dev`). Remove `/home/linuxbrew/.linuxbrew/lib/pkgconfig` from `PKG_CONFIG_PATH` — brew stays in scope only for LLVM/libclang. If you previously built against brew, force a re-link of the GTK family:

```bash
rm -rf target/debug/deps/gtk4* target/debug/deps/adwaita* \
       target/debug/deps/libadwaita* target/debug/deps/gdk* \
       target/debug/deps/gsk* target/debug/deps/graphene* \
       target/debug/deps/pango* target/debug/deps/cairo* \
       target/debug/deps/gio* target/debug/deps/gobject* target/debug/deps/glib* \
       target/debug/build/gtk4-sys-* target/debug/build/libadwaita-sys-* \
       target/debug/deps/open_whisper_linux* target/debug/open-whisper-linux*
```

That keeps the expensive whisper.cpp / llama.cpp caches. Full `cargo clean` works too but rebuilds everything.

### `Failed to register: org.freedesktop.DBus.Error.ServiceUnknown` at startup

**Cause.** Running inside a Flatpak proxy bus (e.g. the VS Code Flatpak), where the sandbox rejects arbitrary well-known bus names. `adw::Application::register` fails.

**Fix.** Use `ApplicationFlags::NON_UNIQUE` (already done in [main.rs](../apps/open-whisper-linux/src/main.rs)) so GApplication skips the name registration. For normal host-terminal runs this is a no-op.

### ksni panic: `called Result::unwrap() on an Err value: org.freedesktop.DBus.Error.ServiceUnknown`

**Cause.** The `ksni` 0.2 tray service unwraps the StatusNotifierWatcher registration result. On vanilla GNOME (no watcher) or in sandboxed sessions the unwrap panics.

**Fix.** [tray.rs](../apps/open-whisper-linux/src/tray.rs) probes for the watcher via `org.freedesktop.DBus.NameHasOwner` before spawning the service and skips with a log warning if absent. No user action needed; it self-heals when the AppIndicator extension is installed.

### Running from the VS Code Flatpak terminal fails to render

**Symptom.** Window never appears; log ends with `Unable to create shared memory pool` or `Truncating shared memory file failed: Invalid argument`.

**Cause.** The VS Code Flatpak sandbox denies Wayland SHM pool allocation.

**Fix.** Run the dev script from a **host** terminal (e.g. `gnome-terminal`, `ptyxis`, `kgx`). Build and compile work fine inside the Flatpak; only window creation is blocked. If you must launch from the Flatpak-internal sandbox, set `PKG_CONFIG_SYSROOT_DIR=/run/host` and `LD_LIBRARY_PATH=/run/host/usr/lib/x86_64-linux-gnu:/home/linuxbrew/.linuxbrew/lib` to link against the host libraries — useful for smoke-testing bridge init without a visible window.

### Main loop feels slow / UI stutters

**Cause.** Bridge polling too aggressive.

**Fix.** Already tuned: `RUNTIME_POLL = 1 s`, `MODEL_POLL = 3 s` in [app.rs](../apps/open-whisper-linux/src/app.rs). Any bridge call > 100 ms triggers a `WARN slow bridge call` log entry — that's the signal to investigate a regression (usually ALSA device enumeration or the hotkey portal registration).

### Window is visible but unresponsive (no SIGKILL, no crash) on GNOME

**Symptom.** The window paints once, then the compositor marks it as *not
responding*. Bridge-poll traces stop after the first tick even though the
process is still alive. Killing the process requires `SIGKILL`. No
`Gtk-CRITICAL`, no `Gdk-WARNING` in the log.

**Cause.** The `ksni` 0.2 StatusNotifierItem integration deadlocks the GTK
main thread on certain GNOME sessions (observed on AnduinOS / Ubuntu
25.10 + NVIDIA). The tray's D-Bus wiring re-enters the main context
during its well-known-name negotiation and never yields again, even
though our probe correctly reported a watcher on the session bus.

**Fix.** The tray is **opt-in** — unset by default, enabled with
`OW_ENABLE_TRAY=1`. KDE / Xfce / Cinnamon / Budgie / MATE sessions can
set the env var safely. On GNOME, leave it unset until Stage 5 replaces
the integration with a portal-aware implementation.

### Pressing the global hotkey does nothing on Wayland + GNOME

**Symptom.** Log reports
`GlobalShortcuts portal rejected bind` and
`Method BindShortcuts is not implemented on interface
org.gnome.Settings.GlobalShortcutsProvider` (visible via
`dbus-monitor --session`). The main window works, but the hotkey
doesn't toggle recording.

**Cause.** `global-hotkey` 0.7 has no functional path on Wayland (it
only knows X11 grabs), so the Linux shell routes through the
`org.freedesktop.portal.GlobalShortcuts` portal instead. KDE/Plasma
implements that portal fully; on GNOME 49 `CreateSession` works but
`BindShortcuts` is still a stub — it's an upstream GNOME / gnome-shell
limitation, tracked in gnome-shell's bug tracker.

**Status / workaround.** Nothing in the app to fix. Until upstream
lands the portal backend:

- Use the main-window dashboard button to start/stop dictation.
- Or bind the app's command to a GNOME *Custom Shortcut* yourself
  (Settings → Keyboard → View and Customise Shortcuts → Custom
  Shortcuts) pointing at the binary. An in-app D-Bus-service hook for
  that is on the roadmap.

On KDE/Plasma the portal accepts the binding and the app reads
`Activated` signals normally — no user action required beyond the
one-time confirmation dialog.

### Tons of `Theme parser warning: gtk.css:…: Expected ';' at end of block`

**Cause.** GTK's default Adwaita stylesheet in some distribution packagings uses CSS that the parser flags (older GTK parsing a newer Adwaita stylesheet, or vice versa). The warnings are cosmetic — the theme renders anyway.

**Fix.** Ignore. Filter with `./scripts/dev-linux.sh 2>&1 | grep -v 'gtk.css:'` when you need a readable log.

---

## Packaging

The Linux shell is not released yet, but here are the constraints a packaging story will need to cover.

### Runtime dependencies to bundle or require

| Library | Runtime package (Debian/Ubuntu) | Notes |
| --- | --- | --- |
| GTK4 ≥ 4.12 | `libgtk-4-1` | Matches the features gate in [apps/open-whisper-linux/Cargo.toml](../apps/open-whisper-linux/Cargo.toml) (`v4_12`). |
| libadwaita ≥ 1.5 | `libadwaita-1-0` | Matches the `v1_5` features gate. |
| ALSA | `libasound2` | `cpal` audio input. |
| libxkbcommon | `libxkbcommon0` | `enigo` Wayland paste. |
| D-Bus | `libdbus-1-3` | Tray (`ksni`) and portal access (`ashpd`). |
| Wayland client libs | `libwayland-client0` | Wayland backends. |
| Optional: CUDA/Vulkan drivers | — | Only when we ship a GL-accelerated build. First release stays CPU-only. |

### Flatpak (recommended first target)

- Base: `org.gnome.Platform` ≥ 46 (includes GTK 4 + libadwaita + portals).
- Finish args:
  - `--socket=wayland`, `--socket=fallback-x11`
  - `--device=all` (microphone via `/dev/snd`; PipeWire portal is the modern alternative)
  - `--socket=pulseaudio` for legacy setups
  - `--talk-name=org.kde.StatusNotifierWatcher` for the tray
  - `--talk-name=org.freedesktop.portal.GlobalShortcuts` for Wayland hotkey
- Decide on a GL extension story before first release — Flatpak apps ship their own Mesa, so NVIDIA proprietary users need the matching `org.freedesktop.Platform.GL.nvidia-*` extension. Until that's wired, ship a `GSK_RENDERER=cairo` wrapper so the app still works on unsupported GPUs.

### AppImage / .deb

Less preferred because the app dynamically depends on GTK4/libadwaita — pinning them inside an AppImage bloats the image by ~200 MB, and a `.deb` inherits the distro's GTK version, which on older Ubuntu LTS may not meet the minimum. Revisit once Flatpak is flowing.

### Auto-updates

macOS uses Sparkle. On Linux we **do not** ship a Sparkle-equivalent — the user's package manager (Flatpak, AppImage Hub, APT/DNF/pacman for distro packages) is the right update channel. The in-app Updates settings tab says so explicitly; no release automation is needed beyond publishing the new artifact to the chosen channel.
