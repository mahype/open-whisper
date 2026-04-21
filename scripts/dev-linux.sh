#!/usr/bin/env bash
set -euo pipefail

BREW_PREFIX="/home/linuxbrew/.linuxbrew"

# System GTK4/libadwaita via pkg-config default path. We *only* pull LLVM's
# libclang from Linuxbrew — brew-GTK4 on Linux produces ABI-mismatched
# widgets (icons report i32::MIN baselines, GTK tries to allocate 225561px
# Cairo surfaces, the compositor marks the window as not responding).
#
# If you see "Package gtk4 was not found in the pkg-config search path",
# install the dev packages: `sudo apt install libgtk-4-dev libadwaita-1-dev`.

# Brew's libclang (bindgen dependency for llama-cpp-sys) stays in the env,
# but we keep it path-targeted so its GTK libraries don't hijack the
# system ones at link time.
export LIBCLANG_PATH="${LIBCLANG_PATH:-${BREW_PREFIX}/lib}"
export PATH="${HOME}/.cargo/bin:${PATH}"

# Force software rendering. brew-GTK4 + NVIDIA + Wayland triggered SIGKILL
# via a GPU hang previously; even on system GTK4 we prefer llvmpipe until
# the Flatpak/AppImage packaging bundles proper GL drivers.
export GSK_RENDERER="${GSK_RENDERER:-cairo}"
export GDK_DISABLE="${GDK_DISABLE:-gl,vulkan}"
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"

export RUST_LOG="${RUST_LOG:-open_whisper_bridge=info,open_whisper_linux=debug}"

exec cargo run -p open-whisper-linux "$@"
