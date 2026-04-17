#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo build -p open-whisper-bridge
swift run --package-path apps/open-whisper-macos OpenWhisperMac
