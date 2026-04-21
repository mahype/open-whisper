#!/usr/bin/env bash
set -euo pipefail

BREW_PREFIX="/home/linuxbrew/.linuxbrew"
SYSTEM_LIB="/usr/lib/x86_64-linux-gnu"

# We need brew *only* for LLVM's libclang (bindgen dependency of
# llama-cpp-sys-2). GTK4 / libadwaita come from the distro (apt install
# libgtk-4-dev libadwaita-1-dev) — see docs/LINUX.md for why brew-GTK4 on
# Linux is toxic: it returns i32::MIN baselines from GtkImage which
# poisons widget measurements and the window never renders.
export LIBCLANG_PATH="${LIBCLANG_PATH:-${BREW_PREFIX}/lib}"

# Point PATH at the cargo toolchain but *avoid* sourcing brew shellenv —
# it would otherwise put brew lib dirs into LD_LIBRARY_PATH and hijack
# the system GTK libraries at runtime even when we linked against system.
export PATH="${HOME}/.cargo/bin:${PATH}"

# Build the runtime library path explicitly. System libs come first; if
# the user had brew lib dirs in LD_LIBRARY_PATH from a `brew shellenv`
# evaluation, they're deliberately stripped for the duration of this
# process. The llama/whisper static libs are already baked in at link
# time, so we don't need brew on the runtime path at all.
export LD_LIBRARY_PATH="${SYSTEM_LIB}"

# Same defense for pkg-config so a follow-up rebuild doesn't pick up
# brew .pc files. cargo-run will still rebuild sys crates when the env
# changes, which is what we want.
export PKG_CONFIG_PATH="${SYSTEM_LIB}/pkgconfig:/usr/share/pkgconfig"

# Force software rendering. brew-GTK4 + NVIDIA + Wayland triggered SIGKILL
# via a GPU hang previously; we default to Cairo until a packaging story
# bundles proper GL drivers. Override with `GSK_RENDERER=gl ./scripts/dev-linux.sh`
# on a well-configured host with open-source drivers.
export GSK_RENDERER="${GSK_RENDERER:-cairo}"
export GDK_DISABLE="${GDK_DISABLE:-gl,vulkan}"
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"

export RUST_LOG="${RUST_LOG:-open_whisper_bridge=info,open_whisper_linux=debug}"

# Quick guard: if system GTK4 is missing, the user needs the dev packages.
if ! pkg-config --exists gtk4 2>/dev/null; then
    cat >&2 <<'EOF'
System GTK4 not found via pkg-config. Install the dev packages first:

  Debian/Ubuntu/AnduinOS:  sudo apt install libgtk-4-dev libadwaita-1-dev
  Fedora/RHEL:             sudo dnf install gtk4-devel libadwaita-devel
  Arch:                    sudo pacman -S gtk4 libadwaita

EOF
    exit 1
fi

exec cargo run -p open-whisper-linux "$@"
