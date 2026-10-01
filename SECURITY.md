# Security policy

MCP Manager reads and writes the MCP configuration of local coding agents. Those files can contain API keys, bearer tokens, passwords, and HTTP headers. This document describes what the program is trusted with, what it refuses to do, and how to report a vulnerability.

Installing an MCP server means trusting code that another agent or client may run later. Registry inclusion is not a code review.

## Threat model

### Assets

- Unified config at `config/servers.yaml`, including env values and HTTP headers.
- Rollback state in `history.json` and the backup files it names.
- Client config files MCP Manager is allowed to edit (Codex, Claude Code, Claude Desktop, Gemini CLI, OpenCode, Cursor, VS Code, and the other modelled clients).
- Marketplace source list and cache under the application-data directory.
- The user's decision to run a marketplace package.

### Trusted

- The person using the app on their own machine.
- The operating system, including file permissions on the home directory.
- Client programs the user already runs (Codex, Claude Code, and the others). Those programs read the files MCP Manager writes and may execute the commands stored there.

### Untrusted

- Marketplace registry metadata: runtime, arguments, package name, environment, and URLs.
- Paths supplied by the UI, a hand-edited `servers.yaml`, or another process.
- Existing client files that another program can change while MCP Manager is writing.
- The network, including update metadata and registry responses.

### Out of scope

- Malware the user chooses to install after seeing the execution plan.
- A compromised operating system or a user account that can already read the config files.
- Remnant data on an SSD after a normal file delete. MCP Manager unlinks expired backups. It does not claim to erase flash cells.
- Secrets that a client stores outside the files MCP Manager edits.

## What MCP Manager reads and writes

Application data, mode `0700` for new directories and `0600` for new secret-bearing files on Unix:

| Path | Contents |
| --- | --- |
| macOS `~/Library/Application Support/mcp-manager/` | application data root |
| Linux `~/.config/mcp-manager/` | application data root |
| Windows `%APPDATA%\mcp-manager\` | application data root |
| `config/servers.yaml` | unified server list, including secrets |
| `history.json` | redacted rollback records and backup paths |
| `config/privacy.json` | update-check and pin preferences |
| `backups/` | client-file backups and `backups/internal/` config backups |
| `marketplace-sources.json`, `marketplace-cache/` | registry list and cached catalogue pages |

Client files are the modelled user and workspace MCP configs only. Examples: `~/.codex/config.toml`, `~/.claude.json`, `~/.cursor/mcp.json`, VS Code `mcp.json`, Gemini CLI settings, OpenCode `opencode.json`. A path that is not one of those targets is rejected. There is no custom-path override.

Unrelated keys in a client file (models, theme, other client options) are left in place. A client file that is not valid JSON or TOML is left unchanged.

## Where secrets may be stored

- `servers.yaml` stores the working copy, including secret env values and header values. On Unix a new file is created mode `0600`. Replacing an existing file keeps that file's mode and, on a best-effort basis, its owner. MCP Manager does not make an existing file more permissive.
- Client config files receive literal secret values. See `docs/security-architecture.md` for which clients can expand an environment reference. MCP Manager does not rewrite a token into `${GITHUB_TOKEN}` because most clients would treat that text as the secret.
- `history.json` stores a redacted config and a SHA-256 fingerprint. The previous `servers.yaml`, including secrets, is kept only in `backups/internal/`, mode `0600`. Rollback reads that file. It does not read the redacted history copy.
- Expired history entries delete the backup files that only they referenced. Deletion is an ordinary unlink.
- MCP Manager does not put secrets in an OS keychain. Protected backups are enough to roll back.
- MCP Manager does not send configs or credentials to its developer, to GitHub, or to a registry.

## Backups

Backups live only under the application-data `backups/` directory. Restore and delete refuse paths that escape that directory, paths that contain `..`, and symlinks inside the backup store. History keeps 20 records. On startup and after a record is retired, orphan files under the backup root are removed, up to 200 per pass. Files outside that root are never deleted by this cleanup.

## Network

There is no analytics, crash reporting, or usage telemetry. The Settings screen says `Telemetry: None` for that reason.

| Destination | When | What is sent |
| --- | --- | --- |
| `https://github.com/xjeway/mcp-manager/releases/latest/download/latest.json` | App launch, if automatic checks are on. Also when the user presses Check for updates. | The updater request only. Not the MCP config. |
| Built-in and user-added MCP registries (HTTPS only, no userinfo) | Marketplace page, and only while marketplace networking is enabled | Search queries and registry URLs. Not the local MCP config. |

Automatic update checks are on by default and can be turned off. A manual check still runs. The Rust command `authorize_update_check` rejects an automatic check when the preference is off. The desktop updater plugin can still be called by a modified webview; the supported UI path does not do that.

Marketplace networking is off until the user enables it. Every marketplace command checks that flag in Rust.

The update feed and its minisign public key are the upstream xjeway release. Installing an update replaces this build with that upstream build.

## Marketplace trust

Each source is labelled curated, official, community, or custom. Those words describe who published the catalogue. They do not mean the package is safe.

Before a marketplace server is added, the app shows the executable, arguments, environment (secrets masked), source, repository, and trust label. An unrecognised runtime, including any `PackageType::Other` entry, needs an explicit confirmation. When "Prefer pinned marketplace installs" is on (the default), an unpinned package or a moving OCI tag also needs confirmation. A missing version is not rewritten to look pinned. `"latest"` stays unpinned. An OCI digest is pinned as `image@sha256:…` when the registry supplies one. MCP Manager does not invent a version.

A command typed in the server editor is the user's own command. That path is not gated as a marketplace runtime.

## Updates and release artefacts

Release workflows pin third-party GitHub Actions to a full commit SHA. Cargo builds use `--locked`. Frontend installs use `npm ci`. Updater artefacts are minisign-signed with the key in `tauri.conf.json` (a public key). Each platform job uploads `SHA256SUMS-<target>.txt`. CLI archives upload `SHA256SUMS-mcpmgr-<target>.txt`.

If Apple signing and notarisation secrets are not configured, the macOS build is unsigned. Gatekeeper will block it. The release notes say so. Clearing the quarantine attribute is not the install method.

## What MCP Manager does not do

- It does not phone home with configuration or credentials.
- It does not add telemetry.
- It does not follow a UI path onto an arbitrary file.
- It does not widen the mode of an existing config file.
- It does not keep a second plaintext copy of secrets in `history.json`.
- It does not treat a registry listing as a security review.
- It does not silently turn an unrecognised marketplace runtime into a client command.
- It does not send secret HTTP headers to a non-loopback `http://` URL. Apply is refused in the shared core, so the CLI and the GUI follow the same rule.

## Reporting a vulnerability

Report privately through GitHub Security Advisories on the repository you installed from. Do not open a public issue that includes config files, tokens, logs, or screenshots of headers.

Please include the version, the operating system, and what an attacker who does not already have your user account could do. You should hear back within 7 days. If you are running the hardened fork, use that fork's advisory form. Upstream remains <https://github.com/xjeway/mcp-manager>.
