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
| Windsurf (now Devin Desktop) | `~/.codeium/windsurf/mcp_config.json` | Unsupported by current model | None | User only; remote servers use `serverUrl` |
| Qoder | `~/.qoder/settings.json` | `.qoder/settings.json` (`.mcp.json` is also read but shared with Claude Code) | None | Direct project support |
| Kilo Code | `~/.config/kilo/kilo.json` (or `kilo.jsonc`) | `kilo.json(c)`, `.kilo/kilo.json(c)` | None | Direct project support; same `mcp` schema as OpenCode. Kilo also reads `opencode.json`, which is left to the OpenCode adapter |
| Factory Droid | `~/.factory/mcp.json` | `.factory/mcp.json` | Ancestor-folder `.factory/mcp.json` (not read) | Direct project support. Project files are committed, so keep tokens out of them |
| Kimi CLI | `~/.kimi/mcp.json` | Unsupported | None | User only; remote servers are written without `type` |
| Zed | `~/.config/zed/settings.json` (Windows: `%APPDATA%\\Zed\\settings.json`) | `.zed/settings.json` | None | Direct project support. Servers live under `context_servers` in the editor settings file, which is edited in place. No `type` field; older nested `command: {path, args, env}` entries are read and rewritten flat. Extension-provided servers (only `settings`) are left alone |
| Mistral Vibe | `~/.vibe/config.toml` | `.vibe/config.toml` (trusted folders only) | None | Direct project support. Servers are a `[[mcp_servers]]` list keyed by `name`, edited in place. Remote servers are written as `streamable-http` with headers in a static `auth` table; SSE servers are not written (Vibe has no SSE client) and the apply reports it. Vibe rejects unknown fields, so no `type` is written |
| Hermes Agent | `~/.hermes/config.yaml` | Unsupported | Profiles (`~/.hermes/profiles/<name>/config.yaml`, not read) | User only. `mcp_servers` mapping edited line by line in place; SSE servers get `transport: sse`. `enabled: false` is Hermes's own switch: it is not imported as the server being off, and applies keep it |
| Goose | `~/.config/goose/config.yaml` (Windows: `%APPDATA%\\Block\\goose\\config\\config.yaml`) | Unsupported | None | User only. Only `stdio` and `streamable_http` entries of `extensions` are managed; built-in and platform extensions are left alone. Entries use `cmd`/`envs`/`uri`, carry `name` and `enabled`, and Goose's own `enabled` and `env_keys` are kept. SSE servers are not written (Goose has no SSE client) and the apply reports it |
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
- Antigravity 2.0's shared config path and Windsurf's `serverUrl` key come from
  third-party sources; confirm them on a real install.
