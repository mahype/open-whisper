#!/usr/bin/env bash
# Kept for backwards compatibility. Prefer ./scripts/dev.sh going forward.

set -euo pipefail
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
echo "[run-macos-native.sh] This script has been renamed to dev.sh. Forwarding…" >&2
exec "$repo_root/scripts/dev.sh" "$@"
