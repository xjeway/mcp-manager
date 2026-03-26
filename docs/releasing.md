# Release Automation

This project uses the mainstream Tauri release flow for GitHub-hosted open source desktop apps:

- GitHub Actions for CI and release orchestration
- `tauri-apps/tauri-action` for cross-platform desktop packaging
- GitHub Releases for artifact hosting
- Tauri updater signing keys for in-app update metadata

## What Gets Built

When you push a tag like `v0.1.0`, GitHub Actions builds release artifacts on:

- macOS arm64: `.app`, `.dmg`, and updater artifacts
- macOS x64: `.app`, `.dmg`, and updater artifacts
- Windows x64: `.msi`, NSIS `setup.exe`, and updater artifacts
- Windows arm64: NSIS `setup.exe` and updater artifacts
- Linux x64: `.AppImage`, `.deb`, `.rpm`, and updater artifacts
- Linux arm64: `.AppImage`, `.deb`, `.rpm`, and updater artifacts

The generated assets are uploaded to a draft GitHub Release automatically. Tauri also generates updater artifacts because `bundle.createUpdaterArtifacts` is enabled in `src-tauri/tauri.conf.json`.

## Workflows

- `.github/workflows/ci.yml`
  Runs frontend tests, frontend build, and `cargo check` on pull requests and pushes to `main`.
- `.github/workflows/release.yml`
  Runs on tags matching `v*` and publishes the desktop bundles to GitHub Releases.
  The workflow explicitly builds `app,dmg` on macOS, `nsis,msi` for Windows x64, `nsis` for Windows arm64, and `appimage,deb,rpm` on Linux.

## Required Repository Secrets

These are required if you want Tauri updater metadata and signed update bundles:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Generate the updater key pair locally with:

```bash
npm run tauri signer generate
```

Put the private key into `TAURI_SIGNING_PRIVATE_KEY` and keep the generated public key for `src-tauri/tauri.conf.json`.

## Recommended Optional Secrets

### macOS signing and notarization

Add these when you want a smoother installation experience on macOS:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

Notes:

- `APPLE_CERTIFICATE` is typically the base64-encoded `.p12` signing certificate.
- Without these, macOS artifacts can still be built, but users will see stronger security warnings.

### Windows code signing

Add these when you want to reduce SmartScreen friction:

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

Notes:

- `WINDOWS_CERTIFICATE` is typically the base64-encoded `.pfx` certificate.
- Without code signing, Windows packages still build, but reputation warnings are more likely.

### Windows ARM runners

The workflow includes a native Windows ARM64 build on `windows-11-arm`, which is currently a GitHub-hosted arm64 runner in public preview.

The release job publishes an ARM64 NSIS installer by targeting `aarch64-pc-windows-msvc`. This matches Tauri's documented ARM64 Windows build path; the installed app is native ARM64, while the NSIS installer itself runs under emulation on ARM machines.

### Linux ARM runners

The workflow includes a native Linux ARM64 build on `ubuntu-24.04-arm` so the release can publish ARM64 `.AppImage`, `.deb`, and `.rpm` assets alongside x64 builds.

This matches Tauri's recommendation to build ARM AppImages on ARM hardware instead of relying on cross-compilation.

## Updater Configuration

`src-tauri/tauri.conf.json` already points the updater endpoint at:

```text
https://github.com/xjeway/mcp-manager/releases/latest/download/latest.json
```

Before shipping updater-enabled releases, set the updater public key:

1. Generate the signer key pair locally.
2. Copy the public key into `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
3. Store the private key and password in GitHub repository secrets.

If the GitHub owner or repository name changes, update the endpoint URL accordingly.

## Release Process

1. Bump the app version consistently in:
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
2. Commit and push to `main`.
3. Create and push a Git tag such as `v0.1.0`.
4. GitHub Actions creates a draft release and uploads the platform installers.
5. Verify artifacts, publish the draft release, and test updater behavior from an installed app.

## Version Sync And Tagging

This repository includes a small release helper script so you do not need to edit multiple version files by hand.

Show current versions:

```bash
npm run release:current
```

Sync a new version across `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`:

```bash
npm run release:sync -- 0.1.1
```

Or:

```bash
make release-sync VERSION=0.1.1
```

Verify alignment before tagging:

```bash
npm run release:verify -- 0.1.1
```

Create the annotated git tag after committing:

```bash
npm run release:tag -- 0.1.1
git push origin v0.1.1
```

Notes:

- `release:tag` requires a clean git working tree.
- The script validates semver and refuses to tag if versions are not aligned.
- Prerelease tags such as `v0.1.0-rc.1` are supported and will become GitHub prereleases in the workflow.

## First Production Rollout Checklist

- Set Tauri updater signing secrets
- Fill `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`
- Decide whether macOS notarization is required before public release
- Decide whether Windows code signing is required before public release
- Push a test tag like `v0.1.0-rc.1` first
