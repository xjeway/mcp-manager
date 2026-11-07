# MCP Client Capability Matrix

This document records how MCP Manager should model each supported client before
project-level UI or apply behavior is expanded. The key distinction is that
client enablement is not just a boolean: clients differ by supported scope,
storage path, merge behavior, and precedence.

## Scope Terms

- `user`: Per-user configuration outside the workspace.
- `project`: Shared workspace configuration inside the current project.
- `localProject`: Private current-project configuration stored outside the
  project tree. Claude Code uses this shape in `~/.claude.json`.
- `agent`: Agent-specific configuration. Kiro has a higher-precedence agent
  layer, but MCP Manager does not write it yet.

## Matrix

| Client | User scope | Project scope | Other scopes | Current model status |
| --- | --- | --- | --- | --- |
| VS Code | `Code/User/mcp.json` | `.vscode/mcp.json` | None | Direct project support |
| Cursor | `~/.cursor/mcp.json` | `.cursor/mcp.json` | None | Direct project support |
| Claude Code | `~/.claude.json` | `.mcp.json` | `localProject` in `~/.claude.json.projects[path]` | Direct project support, local project read-only for now |
| Claude Desktop | `claude_desktop_config.json` | Unsupported | None | User only |
| Codex | `~/.codex/config.toml` | `.codex/config.toml` (trusted projects only) | None | Direct project support |
| OpenCode | `~/.config/opencode/opencode.json` | `opencode.json`, `.opencode/opencode.jsonc` | None | Direct project support |
| GitHub Copilot | `~/.copilot/mcp-config.json` | `.mcp.json`, managed through Claude Code (Copilot CLI no longer reads `.vscode/mcp.json`) | None | User only in direct app controls; project via Claude Code |
| Gemini CLI | `~/.gemini/settings.json` | `.gemini/settings.json` | None | Direct project support |
| Antigravity | `~/.gemini/config/mcp_config.json` (2.0, shared by IDE and CLI) or `~/.gemini/antigravity/mcp_config.json` (1.x, read and written only while the new file does not exist) | Unsupported by current model | None | User only |
| iFlow | `~/.iflow/settings.json` | `.iflow/settings.json` | None | Direct project support; the product was shut down on 2026-04-17, kept only for existing configs |
| Qwen Code | `~/.qwen/settings.json` | `.qwen/settings.json` | None | Direct project support |
| Cline | `cline_mcp_settings.json` | Unsupported by current model | None | User only |
| Devin: Desktop (formerly Windsurf) and CLI (id `windsurf`, CLI aliases `devin`, `devin-desktop`, `devin-cli`) | `~/.config/devin/mcp_config.json` (Windows: `%APPDATA%\\devin\\mcp_config.json`) once it exists, otherwise `~/.codeium/windsurf/mcp_config.json` | `.devin/mcp_config.json` (committed) | `.devin/mcp_config.local.json` (gitignored; local > project > user) | Direct project support. New project servers go to `.devin/mcp_config.json`; servers imported from the local file are written back there. Remote servers use `serverUrl`. Only the user file in use is read |
| Qoder | `~/.qoder/settings.json` | `.qoder/settings.json` (`.mcp.json` is also read but shared with Claude Code) | None | Direct project support |
| Kilo Code | `~/.config/kilo/kilo.json` (or `kilo.jsonc`) | `kilo.json(c)`, `.kilo/kilo.json(c)` | None | Direct project support; same `mcp` schema as OpenCode. Kilo also reads `opencode.json`, which is left to the OpenCode adapter |
| Factory Droid | `~/.factory/mcp.json` | `.factory/mcp.json` | Ancestor-folder `.factory/mcp.json` (not read) | Direct project support. Project files are committed, so keep tokens out of them |
| Auggie (Augment CLI) | `~/.augment/settings.json` | `.augment/settings.json` | `.augment/settings.local.json` (private, local > project > user); organization-managed settings (read-only, not imported) | Direct project support. JSONC `mcpServers` edited in place; remote entries use `type: http` / `type: sse` and `headers`. Imported local entries are written back to their private file. New project entries use the shared file |
| CodeBuddy Code | First existing: `~/.codebuddy/.mcp.json`, `~/.codebuddy/mcp.json`, `~/.codebuddy.json`; default to the first path for new installs | `.mcp.json`, managed through Claude Code | Private local entries in `~/.codebuddy.json.projects[path]` and deprecated project `mcp.json` (not managed) | User only in direct controls; project via Claude Code. JSONC `mcpServers` edited in place. Read and write only the active user file; do not merge deprecated files. Remote entries use `type: http` / `type: sse` and `headers` |
| Shared `.agents` (Codebuff / Freebuff) | `~/.agents/mcp.json` | `.agents/mcp.json` | Immediate parent's `.agents/mcp.json` (read-only import) | One shared-file target, with CLI aliases `codebuff` / `freebuff`. Global > parent > project for same-name servers, matching the upstream loader's actual merge order. HTTP/SSE use `type`, `url`, `headers`; `params` are retained. Parent files are never apply targets. Project secrets warn; supported `$VAR` references are retained without local expansion |
| Kimi CLI | `~/.kimi/mcp.json` | Unsupported | None | User only; remote servers are written without `type` |
| Zed | `~/.config/zed/settings.json` (Windows: `%APPDATA%\\Zed\\settings.json`) | `.zed/settings.json` | None | Direct project support. Servers live under `context_servers` in the editor settings file, which is edited in place. No `type` field; older nested `command: {path, args, env}` entries are read and rewritten flat. Extension-provided servers (only `settings`) are left alone |
| Mistral Vibe | `~/.vibe/config.toml` | `.vibe/config.toml` (trusted folders only) | None | Direct project support. Servers are a `[[mcp_servers]]` list keyed by `name`, edited in place. Remote servers are written as `streamable-http` with headers in a static `auth` table; SSE servers are not written (Vibe has no SSE client) and the apply reports it. Vibe rejects unknown fields, so no `type` is written |
| Hermes Agent | `~/.hermes/config.yaml` | Unsupported | Profiles (`~/.hermes/profiles/<name>/config.yaml`, not read) | User only. `mcp_servers` mapping edited line by line in place; SSE servers get `transport: sse`. `enabled: false` is Hermes's own switch: it is not imported as the server being off, and applies keep it |
| Goose | `~/.config/goose/config.yaml` (Windows: `%APPDATA%\\Block\\goose\\config\\config.yaml`) | Unsupported | None | User only. Only `stdio` and `streamable_http` entries of `extensions` are managed; built-in and platform extensions are left alone. Entries use `cmd`/`envs`/`uri`, carry `name` and `enabled`, and Goose's own `enabled` and `env_keys` are kept. SSE servers are not written (Goose has no SSE client) and the apply reports it |
| Grok Build | `~/.grok/config.toml` (`$GROK_HOME/config.toml` when set) | `.grok/config.toml` | `.grok/config.toml` in folders between the project root and the current directory (not read); `.claude.json`, `.cursor/mcp.json`, `.mcp.json` (left to their own clients) | Direct project support. `[mcp_servers.<name>]` tables like Codex, edited in place, with remote headers in `headers`. A project server replaces a user server of the same name. Grok's own `enabled = false` is kept |
| Amp | `~/.config/amp/settings.json` (Windows: `%APPDATA%\\amp\\settings.json`) | `.amp/settings.json` (Amp asks before running servers from it) | Skill-bundled `mcp.json` (not read) | Direct project support. Servers live under the literal key `amp.mcpServers` in the settings file, edited in place; no `type` is written |
| Kiro | `~/.kiro/settings/mcp.json` | `.kiro/settings/mcp.json` | Agent-level scope | Direct project support, agent scope read-only for now |

## Implementation Rules

- The dashboard should stay server-centric and compact.
- Project-oriented UI should use `CLIENT_CAPABILITIES`, not
  `workspacePlacementPath()` as a source of truth.
- `apps[client] = true` means legacy/default user-level enablement.
- `placements[]` is the scoped enablement model and should become the primary
  representation for project-level state.
- Clients with `projectManagement: "unsupported"` must not render current
  project toggles.
- Clients with `projectManagement: "viaOtherClient"` must not render direct
  project toggles. GitHub Copilot project config is the Claude Code `.mcp.json`
  and is only written through Claude Code.
- Read-only higher-precedence scopes, such as Claude Code `localProject` and
  Kiro `agent`, can be shown later as status rows, but should not be writable
  until the adapter can round-trip them safely.

## Evaluated, not managed yet

- OpenClaw keeps servers under `mcp.servers` in `~/.openclaw/openclaw.json`
  (or `OPENCLAW_CONFIG_PATH`). The file is JSON5, usually with unquoted keys,
  which the in-place JSONC editor refuses, and OpenClaw ships
  `openclaw mcp set`/`unset` for these edits. Supporting it means either a
  JSON5 editor or driving that CLI.
- Crush moved its configuration to `crushrc`, a Bash script it executes; the
  older `crush.json` is deprecated.

## Writes keep fields MCP Manager does not model

When MCP Manager rewrites a server it manages, fields it does not own
(`oauth`, `timeout`, `autoApprove`, `disabledTools`, `trust`, ...) are carried
over from the existing entry. Only the transport fields (`command`, `args`,
`env`, `url`, `serverUrl`, `httpUrl`, `type`, `headers`, `http_headers`) are
replaced.

## Open Verification Items

- VS Code and Cursor same-server ID conflict behavior between user and
  workspace configs should be validated in a fixture or manual integration
  test before showing conflict resolution text.
- OpenCode merge precedence across `opencode.json`, `.opencode/opencode.jsonc`,
  and user config should be verified before exposing priority badges.
- Codex only loads `.codex/config.toml` for trusted projects; MCP Manager
  cannot tell whether a project is trusted, so a project write may have no
  effect until the user trusts it in Codex.
- Copilot CLI and Claude Code both read `.mcp.json`, so it is managed only
  through Claude Code. Writing it from two adapters would let one delete the
  other's entries.
- Devin Desktop (renamed from Windsurf on 2026-06-02) keeps its servers in
  `~/.config/devin/mcp_config.json`. On first start with only the Windsurf-era
  file it asks once whether to copy that file there (confirmed on a macOS
  install), and then reads only the new file. MCP Manager therefore uses the
  new file once it exists and the Windsurf-era file until then, and reads only
  the file in use, so a server left in the old file does not come back. The
  Devin CLI reads the same user file (`%APPDATA%\devin\mcp_config.json` on
  Windows, per its docs) plus the project files `.devin/mcp_config.json` and
  `.devin/mcp_config.local.json`, so Desktop and CLI are one client here.
- Grok Build also loads servers from Claude Code, Cursor and `.mcp.json`.
  Those stay with their own clients; importing them again under Grok would
  list each server twice.
- Antigravity 2.0's shared config path comes from third-party sources; confirm
  it on a real install.

## Auggie format verification (2026-11-07)

The current [MCP guide](https://docs.augmentcode.com/cli/integrations) uses `type`
(not the `transport` key described in issue #56) and supports HTTP/SSE headers.
The [settings guide](https://docs.augmentcode.com/cli/settings) documents JSONC,
user/project/local files and their precedence. This adapter follows those current
examples. Managed organization settings and per-run `--mcp-config` overrides are
not imported or modified.

## CodeBuddy format verification (2026-11-08)

The official [MCP guide](https://www.codebuddy.cn/docs/cli/mcp) confirms user file
precedence, matching read/write resolution, JSONC, stdio/HTTP/SSE and request
headers. A preferred file is selected by existence, not by whether it contains
servers or parses successfully. Unrelated settings, private `projects` entries,
comments and client-specific server fields are retained when the active file is
edited. Project `.mcp.json` has one writer, the Claude Code adapter; CodeBuddy
users select Claude Code for shared project configuration. Deprecated project
`mcp.json` and private local-scope entries are not imported by this adapter.

### Concurrent client edits

CLI confirmation binds a prepared apply to the previewed file paths and desired
entries. Active-path and content changes detected before apply require a fresh
preview. Approved file bytes are checked again before each batch write; a
conflict aborts and rolls back earlier batch writes. This is optimistic
concurrency, not a lock shared with CodeBuddy: an unrelated client can still
race between the last byte check and the atomic replacement. Frozen operations
never redirect a write to a newly preferred file after validation. Configuration
writes by external clients must be serialized with an apply for strict isolation.

## Shared `.agents` verification (2026-11-08)

The [Codebuff MCP guide](https://www.codebuff.com/docs/tips/mcp-servers) and the
[SDK loader](https://github.com/CodebuffAI/freebuff/blob/e9ee7f66dd4a4e73116a7433aa050442aa3c9a98/sdk/src/agents/load-mcp-config.ts)
confirm global, immediate-parent and current-project files. The loop reads
project, parent, then home and replaces same-name definitions, so home wins
(despite an inconsistent explanatory source comment). The loader uses strict
JSON, not JSONC. This adapter rejects comments/trailing commas on import.

Freebuff builds the same CLI via
[its build script](https://github.com/CodebuffAI/freebuff/blob/e9ee7f66dd4a4e73116a7433aa050442aa3c9a98/freebuff/cli/build.ts),
and the shared [agent registry](https://github.com/CodebuffAI/freebuff/blob/e9ee7f66dd4a4e73116a7433aa050442aa3c9a98/cli/src/utils/local-agent-registry.ts)
loads MCP through that SDK. Repository files still require the client's own
trust consent. MCP Manager reads data only and does not grant that consent.

There is one `.agents` target rather than competing adapters for each reader.
Parent entries are imported as unmanaged read-only placements so their priority
is visible without modifying another repository. Applied writes are restricted
to the current project's and user's file. ZCode fallback handling remains #59.
Remote query parameters and other unmodeled fields survive managed rewrites.
