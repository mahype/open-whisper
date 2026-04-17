#!/usr/bin/env bash
# Fast dev loop: build the Rust bridge (debug), then launch the Swift app via SPM.
# For signed release builds or autostart testing, use scripts/build-macos-app.sh instead.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

: "${RUST_LOG:=info}"
export RUST_LOG

cat <<'BANNER'
────────────────────────────────────────────────────────────────
 Open Whisper — dev loop
 This launches outside a .app bundle. Autostart falls back to a
 LaunchAgent plist; SMAppService registration is unavailable.
 For realistic autostart testing, run ./scripts/build-macos-app.sh
────────────────────────────────────────────────────────────────
BANNER

cargo build -p open-whisper-bridge
swift run --package-path apps/open-whisper-macos OpenWhisperMac
