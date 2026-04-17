# Releasing

This document is for maintainers. It describes how Open Whisper is versioned, built, signed, and published.

## TL;DR

```bash
# 1. Update versions
# 2. Tag and push
git tag v0.1.0
git push origin v0.1.0
# 3. Watch the release workflow on GitHub
# 4. Review the draft release, write release notes, publish
```

## Versioning

Open Whisper follows **SemVer**: `MAJOR.MINOR.PATCH`, optionally `-rc.N` for pre-releases.

The **Git tag is the source of truth** for the version. Before tagging:

1. Update `version` in the root [Cargo.toml](../Cargo.toml) under `[workspace.package]`.
2. The build script injects the version into the macOS bundle's `Info.plist` via `git describe --tags`.
3. Commit the Cargo.toml bump, then create the tag from that commit.

## Release tag format

- Stable: `v0.1.0`, `v0.2.0`, `v1.0.0`
- Release candidate: `v0.1.0-rc.1`
- Any tag matching `v*` triggers [.github/workflows/release.yml](../.github/workflows/release.yml).

## What the release workflow does

On push of a `v*` tag, the GitHub Actions release workflow:

1. Runs the full CI suite (Rust fmt/clippy/test, macOS app build).
2. Builds a universal (arm64 + x86_64) Rust static library.
3. Builds a universal Swift executable and assembles `Open Whisper.app`.
4. Injects the version from the tag into `Info.plist`.
5. Signs the bundle with the **Developer ID Application** certificate.
6. Submits to Apple's notary service via `notarytool` and staples the ticket.
7. Packages the result into `OpenWhisper-<version>.dmg`.
8. Uploads the DMG and a `SHA256SUMS.txt` to a **draft** GitHub Release.

You then review the draft, add release notes, and publish it.

## Secrets required in the GitHub repository

Configure these under **Settings → Secrets and variables → Actions**:

| Secret | Purpose |
| --- | --- |
| `MACOS_CERTIFICATE_P12` | Base64-encoded `.p12` export of your *Developer ID Application* cert |
| `MACOS_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12` |
| `APPLE_ID` | Apple ID email tied to your developer account |
| `APPLE_TEAM_ID` | 10-character Team ID from [developer.apple.com → Membership](https://developer.apple.com/account/#MembershipDetailsCard) |
| `APPLE_APP_SPECIFIC_PASSWORD` | App-specific password generated at [appleid.apple.com](https://appleid.apple.com/account/manage) → *Sign-In and Security* → *App-Specific Passwords* |

### Generating `MACOS_CERTIFICATE_P12`

1. In **Keychain Access**, find your *Developer ID Application: Your Name (TEAMID)* certificate.
2. Right-click → **Export…** → save as `DeveloperID.p12`, set a strong password.
3. Base64-encode it:
   ```bash
   base64 -i DeveloperID.p12 | pbcopy
   ```
4. Paste into the `MACOS_CERTIFICATE_P12` secret. Paste the export password into `MACOS_CERTIFICATE_PASSWORD`.

### Generating `APPLE_APP_SPECIFIC_PASSWORD`

`notarytool` does not accept your main Apple ID password. Go to [appleid.apple.com](https://appleid.apple.com/) → *Sign-In and Security* → *App-Specific Passwords* → generate one labeled e.g. `open-whisper-notarization`.

## Test a release build locally

Before cutting a tag, verify the full pipeline on your machine:

```bash
# 1. Build the universal .app
./scripts/build-macos-app.sh

# 2. Sign + notarize (needs the env vars below set locally)
export MACOS_SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_ID="you@example.com"
export APPLE_TEAM_ID="XXXXXXXXXX"
export APPLE_APP_SPECIFIC_PASSWORD="xxxx-xxxx-xxxx-xxxx"
./scripts/codesign-macos.sh

# 3. Package the DMG
./scripts/build-dmg.sh

# 4. Verify Gatekeeper accepts the result
spctl --assess --type open --context context:primary-signature "dist/Open Whisper.app"
# Expected: "accepted, source=Notarized Developer ID"
```

## Rollback

If a release has a critical bug:

1. **Mark the GitHub release as pre-release** (or delete it). This removes it from the "Latest release" link users follow.
2. Re-tag with a patch bump (`v0.1.1`) after pushing the fix. Do **not** re-use or re-point the broken tag — users and the notary service have already seen it.
3. If the DMG is already notarized and in the wild, you cannot revoke it remotely. The only signal users get is that a newer release exists; make the release notes clear about the issue.

## Troubleshooting a failed release workflow

**Signing step fails with "no identity found".**
`MACOS_CERTIFICATE_P12` was not imported correctly. Re-run the export + base64 process and double-check you're exporting the *Developer ID Application* cert, not *Developer ID Installer* or *Apple Development*.

**Notarization returns "Invalid".**
Run `xcrun notarytool log <submission-id> --apple-id … --team-id … --password …` to see the detailed rejection. The most common causes are missing hardened runtime (`--options=runtime`) or an unsigned nested binary inside the bundle.

**DMG upload step fails.**
Check that the workflow has `permissions: contents: write`. GitHub Actions requires explicit write permission on the repo to create releases.
