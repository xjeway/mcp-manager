# MCP Manager Hardening Report

## Executive Summary

This fork keeps MCP Manager's existing clients and screens, and moves the security decisions into the Rust core. Config files keep the permissions they already had. New secret-bearing files are private. Rollback history no longer stores a second plaintext copy of tokens. Backup files that fall out of the history window are deleted. Writes are limited to modelled client configs and the application-data directory. A marketplace entry shows the command it will run, and an unrecognised or unpinned package needs an explicit confirmation. Automatic update checks can be turned off. There is still no telemetry.

The fork is suitable for storing real MCP credentials on a machine you control, with the limits in Remaining Risks. Client files still contain the literal secrets those clients require. Do not install an upstream update over this build: the updater feed is still the upstream release.

## Upstream Revision Reviewed

| Item | Value |
| --- | --- |
| Upstream | https://github.com/xjeway/mcp-manager |
| Fork | https://github.com/MacHatter1/mcp-manager |
| Branch | `security/harden-mcp-manager` |
| Upstream SHA reviewed | `15e11863ccac22ab7344180a6ea39a8c67f3972f` |
| Tag `v0.2.0` | present; commits after it are release automation only |
| Application version | 0.2.0 |
| Rust | rustc 1.98.1 / cargo 1.98.1 |
| Node | v24.21.0 / npm 11.19.0 |

No commit on `main` since `v0.2.0` fixes the issues below. Nothing here re-applies a patch upstream has already superseded. `upstream/main` was fetched again on 2026-10-01 and was still `15e11863`.

Baseline before edits: `npm test` 225 passed, `npm audit` 0 vulnerabilities, `npm run build` succeeded, `cargo test --workspace --locked` succeeded.

## Threat Model

### Before

An attacker who could influence a path, a marketplace record, or a file MCP Manager replaced could:

- widen a `0600` config to the process umask when the atomic replace created a new inode
- read secrets from `history.json` after the live config was rotated
- collect backup files that outlived the 20-entry history cap
- point a placement path, including `../` or a symlink, at a file outside the client config set
- publish a registry entry whose runtime hint became a client command with no extra confirmation
- ship an unpinned or `latest` package that looked like a normal install
- rely on the React UI alone for header and URL checks

The program already refused marketplace calls when the feature was off, already pinned a registry version when one was present, and already had no telemetry.

### After

Those writes, history records, backup deletes, and apply checks go through `mcp-manager-core`. The GUI and the CLI call that code. Marketplace metadata still cannot be trusted; the user now sees the execution plan and must confirm an unrecognised runtime or an unpinned spec. Network calls are unchanged in destination and still do not include the MCP config.

## Findings

### H-1 — Atomic replace could widen permissions

- ID: H-1
- Severity: P0
- Affected files: `crates/mcp-manager-core/src/storage/mod.rs`, `crates/mcp-manager-core/src/security/perms.rs`
- Attack/precondition: MCP Manager rewrites an existing `0600` or `0640` config. The new temp file is created with the umask, then renamed over the original.
- Impact: a secret file becomes group- or world-readable.
- Fix: copy the existing mode onto the temp file after create, so the umask cannot widen it. New sensitive files use `0600`. Directories under app data only lose group and other bits. Owner bits are never added. Unix owner is copied with `fchown` when the process can. Windows starts from an owner-only DACL and copies the previous DACL when that call succeeds.
- Regression test: `storage::tests::preserves_existing_modes_and_creates_private_files`, `storage::tests::a_failed_replacement_does_not_leave_a_secret_temp_file`

### H-2 — History stored full configs, including secrets

- ID: H-2
- Severity: P0
- Affected files: `crates/mcp-manager-core/src/history.rs`, `crates/mcp-manager-core/src/security/redact.rs`
- Attack/precondition: a later reader of `history.json` (backup, sync, or another user on the machine) sees `command.env` and HTTP headers.
- Impact: tokens, passwords, and Authorization values are duplicated outside the live config.
- Fix: history stores a redacted config plus a SHA-256 fingerprint. The previous `servers.yaml` is written to `backups/internal/` at mode `0600`. Rollback reads that file. Header values are always redacted. Env values are redacted when `secret_env` says so, when the name looks secret, or when the value looks like a token. Legacy files are redacted on load after the backup succeeds.
- Regression test: `history::tests::history_json_does_not_keep_plaintext_secrets_and_rollback_restores_them`, `history::tests::legacy_history_with_plaintext_secrets_is_redacted_on_load`, `security::redact::tests::redacts_headers_marked_env_and_token_shapes_but_keeps_ordinary_env`

### H-3 — Expired history left backup files behind

- ID: H-3
- Severity: P0
- Affected files: `crates/mcp-manager-core/src/history.rs`, `crates/mcp-manager-core/src/storage/mod.rs`
- Attack/precondition: history is capped at 20 records, but each record's backup file stayed on disk.
- Impact: unbounded copies of configs that contain secrets.
- Fix: when a record is dropped or rolled back, backup paths that no other record names are unlinked, and only if they sit inside the backup root and are not symlinks. Orphan cleanup is bounded at 200 files a pass. Deletion is a normal unlink.
- Regression test: `history::tests::rolling_history_deletes_backups_owned_only_by_expired_entries`, `storage::tests::backup_deletion_rejects_traversal_and_does_not_follow_symlinks`

### H-4 — Client write paths were not constrained in the backend

- ID: H-4
- Severity: P0
- Affected files: `crates/mcp-manager-core/src/security/paths.rs`, `crates/mcp-manager-core/src/workflow.rs`, `src-tauri/src/commands/mod.rs`
- Attack/precondition: a placement path, a relative internal path, or a symlink under the home or workspace points outside the file MCP Manager should edit.
- Impact: arbitrary file write or read, including of secret material.
- Fix: internal commands accept only `config/servers.yaml`. Client writes must match a modelled user path or a workspace path for a client that has one, lexically and, when both exist, by canonical path. Symlinks at or below the home directory or workspace root are rejected. There is no custom-path feature, so other paths are rejected rather than given a confirmation dialog.
- Regression test: `security::paths::tests::internal_paths_reject_absolutes_and_traversal`, `security::paths::tests::client_writes_accept_modeled_paths_and_reject_escapes`, `security::paths::tests::symlink_inside_the_workspace_cannot_escape_it`, `workflow::tests::apply_rejects_paths_outside_known_client_configs`, `storage::tests::rejects_internal_paths_that_escape_app_data`

### H-5 — Marketplace metadata could choose the executable

- ID: H-5
- Severity: P1
- Affected files: `src-tauri/src/marketplace/install.rs`, `src/components/MarketplacePage.tsx`, `src/view-models/marketplace.ts`
- Attack/precondition: a community publisher sets `runtime_hint` or `PackageType::Other` to a program name the app then writes into a client config.
- Impact: the client later runs that program. Registry inclusion was easy to read as approval.
- Fix: the Rust install option carries `arbitrary_runtime` and a trust label (curated, official, community, custom). The detail pane shows executable, arguments, masked environment, source, repository, and trust, and says inclusion is not a code review. An unrecognised runtime, including every `Other` package, needs a checkbox. A command typed in the server editor is not gated.
- Regression test: `an_unrecognised_runtime_is_flagged_and_latest_is_not_presented_as_pinned`, frontend `acknowledgementRequired` ("asks before an unpinned or unrecognised runtime")

### H-6 — Unpinned packages could look pinned

- ID: H-6
- Severity: P1
- Affected files: `src-tauri/src/marketplace/install.rs`, `src/components/SettingsPage.tsx`
- Attack/precondition: the registry sends no version, `"latest"`, or an OCI tag without a digest.
- Impact: a later install resolves different code. `"latest"` must not be rewritten into a spec that looks pinned.
- Fix: npm uses `name@1.2.3` and PyPI `name==1.2.3` only when the registry sends a real version. OCI uses `image@sha256:…` when a digest is present. A tag without a digest is `mutable-tag`. Missing and `"latest"` stay `unpinned` and are not rewritten. Prefer pinned installs defaults on and requires acknowledgement.
- Regression test: `an_oci_digest_is_pinned_and_a_tag_is_not`, `an_unrecognised_runtime_is_flagged_and_latest_is_not_presented_as_pinned`, `security::privacy::tests::automatic_checks_stop_when_the_preference_is_off_and_manual_checks_do_not` (the same file stores the pin preference, default on)

### H-7 — Some app-data writers bypassed the private atomic write

- ID: H-7
- Severity: P1
- Affected files: `src-tauri/src/marketplace/sources.rs`, `src-tauri/src/marketplace/cache.rs`, `src-tauri/src/commands/mod.rs`
- Attack/precondition: marketplace source list, cache, or a broken-config backup is created with the umask.
- Impact: registry config and a broken `servers.yaml` can be world-readable.
- Fix: those writers use `atomic_write` or `restrict_new_file`.
- Regression test: `storage::tests::preserves_existing_modes_and_creates_private_files` covers the shared writer. Source-store tests still assert a failed save leaves no temp file (`a_failed_save_leaves_the_target_and_no_temporary_file`).

### H-8 — Automatic update checks could not be turned off

- ID: H-8
- Severity: P1
- Affected files: `crates/mcp-manager-core/src/security/privacy.rs`, `src/services/updater.ts`, `src/components/SettingsPage.tsx`
- Attack/precondition: the app contacts GitHub on every launch. A user who does not want that has no switch. The feed is the upstream release, so an install replaces this fork.
- Impact: unwanted network request. Installing the result drops the hardening.
- Fix: `Check for MCP Manager updates automatically` defaults on and can be turned off. The automatic path returns before `check()`. Manual check still runs. Rust `authorize_update_check` rejects an automatic check when the preference is off. The settings copy says the check does not send the configuration, and that the feed is the upstream release.
- Regression test: `makes no update request when automatic checks are disabled`, `security::privacy::tests::automatic_checks_stop_when_the_preference_is_off_and_manual_checks_do_not`

### H-9 — UI validation was not authoritative

- ID: H-9
- Severity: P1
- Affected files: `crates/mcp-manager-core/src/ops.rs`, `crates/mcp-manager-core/src/workflow.rs`
- Attack/precondition: the CLI, or a GUI that skips the React checks, applies secret headers on `http://example.com`, a duplicate header, a bad id, or an empty program.
- Impact: secrets on the wire, or a config the UI would have blocked.
- Fix: `workflow::apply` refuses blocking `ops::validate` issues before any write. The CLI and the GUI both use that function. A missing executable on `PATH` does not block apply.
- Regression test: `workflow::tests::apply_refuses_secret_headers_on_insecure_remote_http`, `ops` tests for duplicate header casing and insecure headers (existing, still passing)

### H-10 — Release workflows tracked moving tags and recommended clearing quarantine

- ID: H-10
- Severity: P1
- Affected files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/workflows/homebrew.yml`, `.github/workflows/issue-triage.yml`, `README.md`, `README.zh-CN.md`, `docs/releasing.md`, `scripts/github-release.mjs`
- Attack/precondition: a tag such as `actions/checkout@v7` is moved, or a user follows the `xattr` install note and runs a downloaded unsigned app as if Gatekeeper had accepted it.
- Impact: a changed action runs with the workflow's token. The quarantine note hides that the build is unsigned.
- Fix: third-party actions are pinned to full commit SHAs with the tag in a comment. `contents: write` is limited to jobs that publish a release. Signing secrets stay on the sign steps. Each target uploads its own `SHA256SUMS-*.txt`. Release text says an unsigned macOS build is unsigned and that clearing quarantine is not the install method.
- Regression test: `scripts/workflow-actions.test.mjs`, `scripts/release-workflow.test.mjs` ("stays on the pinned tauri-action v0.6.2 commit"), `scripts/github-release.test.mjs` ("says an unsigned macOS build is unsigned and does not clear quarantine")

### H-11 — Secret handling was not recorded per client

- ID: H-11
- Severity: P2
- Affected files: `crates/mcp-manager-core/src/security/capabilities.rs`
- Attack/precondition: a future change rewrites every secret as `${GITHUB_TOKEN}` or a keychain id.
- Impact: clients that store literals would send the reference text, or fail open.
- Fix: a capability row per supported app. VS Code can take input references; every other client is literal-only; none take an OS keychain id. `materialize_env` still returns the literal for every app, including VS Code.
- Regression test: `security::capabilities::tests::every_supported_app_has_one_capability_row_and_literals_are_not_rewritten`

## Filesystem Security

See H-1, H-3, and H-4. `apply_operations` itself stays permissive so low-level adapter tests can use temporary paths. `workflow::apply`, `workflow::snapshot`, and backup restore validate before they touch a client file. A failed multi-file apply still restores files already written in that attempt (`storage::tests::a_failed_batch_puts_every_written_file_back`). Malformed JSON and TOML are left unchanged.

## Secret Handling

See H-2 and H-11. No OS keychain and no custom encryption. SHA-256 fingerprints are for change detection. Redaction covers Authorization and Proxy-Authorization by treating every header value as secret, plus API keys, bearer tokens, passwords, client secrets, private tokens, and env vars marked secret.

## Marketplace Supply Chain

See H-5 and H-6. Marketplace disabled still means the Rust commands do not call a registry (`commands_refuse_to_run_while_disabled`, pre-existing and still passing). Registry URLs stay HTTPS-only with no userinfo (`rejects_unusable_urls`, pre-existing).

## Network Behaviour

No analytics, crash reporting, or usage telemetry was added. Settings shows `Telemetry: None`.

Update checks request `https://github.com/xjeway/mcp-manager/releases/latest/download/latest.json` and do not upload the MCP config. Automatic checks stop when the preference is off. Manual checks still run.

Marketplace requests go only to the configured HTTPS registries, and only while the Rust flag is on.

## Updater / Release Chain

See H-10. Updater artefacts stay minisign-signed with the public key already in `tauri.conf.json`. `npm ci` and `cargo --locked` were already required and remain so. CI also runs `cargo fmt --check`, `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`, `cargo test --locked --workspace`, `npm audit --audit-level=moderate`, `cargo audit` 0.22.2, `cargo deny` 0.20.2, gitleaks 8.30.1, and dependency review on pull requests.

If Apple signing secrets are absent, the workflow still builds an unsigned macOS app and the release notes say Gatekeeper will block it.

## Tests Added

Rust coverage is in the modules named above. Frontend coverage is the updater test, the acknowledgement test, and the workflow/release script tests. The full suites after the change:

- `cargo test --workspace --locked`: mcp-manager 58 passed (1 ignored), mcp-manager-cli 17 + 17 passed, mcp-manager-core 137 passed. Exit 0.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`: exit 0.
- `cargo fmt --all --check`: clean after `cargo fmt --all`.
- `npm test`: 38 files, 228 passed.
- `npm run build` (`tsc` and Vite): exit 0.
- `npm audit --audit-level=moderate`: 0 vulnerabilities.

Tests use temporary directories. They do not edit the developer machine's Codex, Claude, or other client configs.

## Dependency Audit

| Scanner | Version | Result |
| --- | --- | --- |
| npm audit | npm 11.19.0, `--audit-level=moderate` | 0 vulnerabilities |
| cargo deny | 0.20.2, `--all-features check` | advisories ok, bans ok, licenses ok, sources ok |
| gitleaks | 8.30.1, default rules, git history | no leaks |
| cargo audit | 0.22.2, advisory db `3461c0d8` (2026-10-01) | exit 0. 509 crates. 0 vulnerabilities. 3 informational warnings |

`cargo audit` reported no vulnerabilities. The warnings, all pre-existing transitive crates, are:

- `proc-macro-error` 1.0.4, RUSTSEC-2024-0370, unmaintained. Pulled in by gtk/glib. No fixed release.
- `glib` 0.18.5, RUSTSEC-2024-0429, unsound `VariantStrIter` (fixed in glib 0.20). Linux UI only. Not on the credential path.
- `yoke-derive` 0.8.3, yanked. Transitive idna/icu crate, not a vulnerability in this program.

`deny.toml` allows the licences this lockfile uses (MIT, Apache-2.0, BSD-3-Clause, ISC, MPL-2.0, Unicode-3.0, Zlib, and the other SPDX ids listed there). Duplicate crate versions are warnings. Unmaintained advisories are checked for workspace crates only, so RUSTSEC-2024-0370 does not fail `cargo deny`. Yanked crates warn and do not fail the job. No advisory is ignored.

Gitleaks did not flag the test fixtures (`ghp_SUPERSECRETVALUE1234567890` and similar). Those strings exist only in tests that prove redaction. They are not credentials.

## Remaining Risks

- Client config files still store literal secrets. That is what Codex, Claude Code, Gemini CLI, OpenCode, and the other non-VS-Code clients read. VS Code could take `${input:id}`; this fork does not rewrite values into that form.
- POSIX ACLs beyond the mode bits are not copied. A mode of `0600` is preserved; an ACL that granted extra access on the old inode is not copied onto the new one, and an ACL that was tighter than the mode is not reconstructed.
- Windows DACL code is implemented and CI runs the tests on `windows-latest`. It was not executed on the macOS machine that prepared the branch.
- Unlink does not scrub SSD pages.
- A modified webview can call the updater plugin directly. The supported automatic path does not.
- A missing binary on `PATH` does not block apply.
- The updater feed and minisign key are the upstream project's. Installing an update replaces this fork.
- `glib` 0.18.5 (RUSTSEC-2024-0429) is an informational unsoundness warning in the Linux UI stack. `cargo audit` does not count it as a vulnerability. It is not on the path that stores credentials.
- JSON comments are dropped when a valid JSONC file is reserialised. Invalid JSON is left untouched.
- There is no custom-path confirmation. Paths outside the modelled set are rejected.

## Recommended Future Work

- Point the updater at a feed signed by this fork, or refuse upstream payloads, so an update cannot silently replace the hardened build.
- Teach the VS Code adapter to offer `${input:id}` when the user asks, using the capability table, without changing the other clients.
- Copy POSIX ACLs where the platform allows it without a shell.
- Move the updater `check()` call into Rust so a modified webview cannot bypass the preference.
- Replace `proc-macro-error` when gtk/glib publishes a fixed release, then set `unmaintained` back to `all`.
- Move the Linux UI off `glib` 0.18 (RUSTSEC-2024-0429) when the toolkit dependency allows 0.20 or newer.
