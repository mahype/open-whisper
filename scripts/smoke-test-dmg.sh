#!/usr/bin/env bash
# Mounts a built DMG and verifies the enclosed .app passes Gatekeeper,
# has a valid deep codesign signature, and a valid stapled notarization
# ticket. Used after build-dmg.sh in the release workflow to catch broken
# artifacts before they are published.
#
# Usage: scripts/smoke-test-dmg.sh <path-to-dmg>

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <path-to-dmg>" >&2
    exit 2
fi

dmg_path="$1"

if [[ ! -f "$dmg_path" ]]; then
    echo "error: DMG not found at $dmg_path" >&2
    exit 1
fi

mount_root="$(mktemp -d)"
mount_point="$mount_root/dmg"
mkdir -p "$mount_point"

cleanup() {
    hdiutil detach "$mount_point" -quiet || true
    rm -rf "$mount_root"
}
trap cleanup EXIT

echo "Mounting $dmg_path"
hdiutil attach "$dmg_path" -mountpoint "$mount_point" -nobrowse -quiet

app_path="$(find "$mount_point" -maxdepth 2 -type d -name '*.app' | head -1)"
if [[ -z "$app_path" ]]; then
    echo "error: no .app bundle found inside DMG" >&2
    exit 1
fi
echo "Found app bundle: $app_path"

echo "-> codesign --verify --deep --strict"
codesign --verify --deep --strict --verbose=2 "$app_path"

echo "-> spctl --assess (Gatekeeper)"
spctl --assess --type execute --verbose "$app_path"

echo "-> stapler validate (notarization ticket)"
xcrun stapler validate "$app_path"

echo "Smoke test passed: $app_path"
