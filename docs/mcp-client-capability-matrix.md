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
| Codex | `~/.codex/config.toml` | Unsupported by current model | None | User only |
| OpenCode | `~/.config/opencode/opencode.json` | `opencode.json`, `.opencode/opencode.jsonc` | None | Direct project support |
| GitHub Copilot | `~/.copilot/mcp-config.json` | Managed through VS Code workspace config | None | User only in direct app controls; project via VS Code |
| Gemini CLI | `~/.gemini/settings.json` | `.gemini/settings.json` | None | Direct project support |
| Antigravity | `~/.gemini/antigravity/mcp_config.json` | Unsupported by current model | None | User only |
| iFlow | `~/.iflow/settings.json` | `.iflow/settings.json` | None | Direct project support |
| Qwen Code | `~/.qwen/settings.json` | `.qwen/settings.json` | None | Direct project support |
| Cline | `cline_mcp_settings.json` | Unsupported by current model | None | User only |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` | Unsupported by current model | None | User only |
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
  project toggles. GitHub Copilot project config should be represented through
  VS Code workspace config until a separate authoritative model is added.
- Read-only higher-precedence scopes, such as Claude Code `localProject` and
  Kiro `agent`, can be shown later as status rows, but should not be writable
  until the adapter can round-trip them safely.

## Open Verification Items

- VS Code and Cursor same-server ID conflict behavior between user and
  workspace configs should be validated in a fixture or manual integration
  test before showing conflict resolution text.
- OpenCode merge precedence across `opencode.json`, `.opencode/opencode.jsonc`,
  and user config should be verified before exposing priority badges.
- Codex project-level MCP support should remain disabled until there is
  authoritative documentation and adapter support.
