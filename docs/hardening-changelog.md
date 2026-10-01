# Hardening changelog

Work landed on branch `security/harden-mcp-manager` against upstream `main` (`15e11863ccac22ab7344180a6ea39a8c67f3972f` at review time). Commits after `v0.2.0` on that branch are release automation. None of them fix the issues below, so the changes are new rather than a replay of an upstream patch.

## Behaviour that can surprise an existing user

- Apply, from the GUI and the CLI, refuses a config that fails core validation. That includes secret headers on a non-loopback `http://` URL, duplicate header names, an empty program, a bad URL, and a server id containing `/`, `\`, NUL, or a `..` segment. Previously the GUI could strip headers and continue.
- A placement path that is not a modelled client config is rejected. There was no supported custom-path feature to preserve.
- A client file that is not valid JSON or TOML is left unchanged. A broken JSON file is no longer replaced by an empty object plus the merge.
- `history.json` no longer contains secret values. Rollback restores `servers.yaml` from `backups/internal/`. Old history files are redacted on load, after a backup of the previous config is written.
- New app-data files are mode `0600`. Replacing a file keeps its mode. App-data directories lose group and other bits and do not gain owner bits, so a directory left at `0555` stays unwritable.
- Backups of client configs are mode `0600` even when the source file was wider, because the backup is MCP Manager's copy and may contain secrets.
- Marketplace installs that are unpinned, tagged without a digest, or use an unrecognised runtime need a confirmation when the matching preference is on. `"latest"` is not rewritten into a pinned spec.
- Automatic update checks can be turned off. The check talks to the upstream GitHub release, and installing it replaces this build.
- macOS release text no longer tells people to clear the quarantine attribute.

## What stayed the same

- Per-client serializers. Codex stays TOML, Claude Code stays JSON, and the other clients keep their own writers.
- Unrelated keys in client files.
- Marketplace off means the Rust commands refuse registry calls.
- A package version supplied by the registry is still pinned (`name@1.2.3`, `name==1.2.3`). A digest is pinned as `image@sha256:…`.
- No telemetry was added.
- Tests still use temporary directories. They do not edit the developer's real client configs.

## Limits

- POSIX ACLs beyond the mode bits are not copied.
- Windows DACL copy is implemented with `windows-sys` and is exercised by CI on `windows-latest`, not by the macOS machine that prepared this branch.
- Unlink does not scrub SSD pages.
- A modified webview can still call the updater plugin directly. The supported check path does not.
- A missing binary on `PATH` does not block apply.
- JSON comments are dropped when a valid JSONC file is reserialised.
