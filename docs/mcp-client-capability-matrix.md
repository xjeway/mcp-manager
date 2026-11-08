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
| Command Code | `~/.commandcode/mcp.json` | `.mcp.json`, managed through Claude Code | Private `~/.commandcode/projects/<slug>/mcp.json` (not managed) | User only in direct controls; project via Claude Code. Strict JSON `mcpServers`; canonical `transport: stdio` / `transport: http` with `type` accepted on import. Preserves environment references, enabled flags and same-transport OAuth metadata |
| CodeBuddy Code | First existing: `~/.codebuddy/.mcp.json`, `~/.codebuddy/mcp.json`, `~/.codebuddy.json`; default to the first path for new installs | `.mcp.json`, managed through Claude Code | Private local entries in `~/.codebuddy.json.projects[path]` and deprecated project `mcp.json` (not managed) | User only in direct controls; project via Claude Code. JSONC `mcpServers` edited in place. Read and write only the active user file; do not merge deprecated files. Remote entries use `type: http` / `type: sse` and `headers` |
| Codebuff / Freebuff | `~/.agents/mcp.json` | `.agents/mcp.json` | Immediate parent's `.agents/mcp.json` (read-only import) | One shared-file target, with CLI aliases `codebuff` / `freebuff`. Global > parent > project for same-name servers, matching the upstream loader's actual merge order. HTTP/SSE use `type`, `url`, `headers`; `params` are retained. Parent files are never apply targets. Project secrets warn; supported `$VAR` references are retained without local expansion |
| ZCode | `~/.zcode/cli/config.json` → `mcp.servers` | `.zcode/config.json` → `mcp.servers` | Same-scope `.agents/mcp.json` fallback, only when native definitions are empty | Direct project support with user > workspace precedence. Native files are edited only under the nested `mcp.servers` path. Fallbacks are imported but never written by this adapter; the first native write carries forward fallback entries so they are not silently shadowed. Client-owned `enable: false`, timeouts and OAuth settings survive rewrites |
| MiMo Code | `$MIMOCODE_HOME/config` or `$XDG_CONFIG_HOME/mimocode` (default `~/.config/mimocode`): `config.json`, `mimocode.json`, `mimocode.jsonc`; `~/.mimocode/mimocode.json(c)` overlay | `mimocode.json(c)` and `.mimocode/mimocode.json(c)` | Ancestor, managed/remote and per-run injected sources are not managed | Direct project support. OpenCode-style `mcp` entries (`local` command array / `environment`, `remote` URL / headers). JSONC overrides JSON; hidden-directory files override root files. Import deep-merges supported layers before decoding servers and retains references. New user installs select the highest loaded user file; imported user/project layers are pinned for in-place edits and complete removal |
| Muse Code | `~/.config/muse/settings.json` | — | — | User only. Strict JSON, `schema_version: 1`, root `mcp_servers`; `transport: stdio` / `streamable_http`. New settings are initialized atomically. Legacy `mcpServers` or an unsupported schema blocks apply. Retains client enabled flags, mode, same-transport framing and unrelated settings |
| Rovo Dev CLI | `~/.rovodev/mcp.json`, overridden by `mcp.mcpConfigPath` in `~/.rovodev/config.yml` | — | Per-run config overrides not managed | User only. JSON `mcpServers`, canonical `transport: stdio` / `http` / `sse`. Read-only YAML resolution accepts absolute paths or `~/`; malformed YAML or ambiguous relative paths block apply. The YAML disabled-server list is preserved and never becomes a global disable |
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
Parent entries are imported as unmanaged read-only placements. Their paths and
priority are not displayed in the dashboard; edit the parent file to change them. Applied writes are restricted
to the current project's and user's file. ZCode's fallback behavior is described below.
Remote query parameters and other unmodeled fields survive managed rewrites.

## ZCode verification (2026-11-08)

The [official MCP guide](https://zcode.z.ai/en/docs/mcp-services) confirms native
user/workspace paths and `mcp.servers`, HTTP/SSE and request headers, and the
client-owned `enable` switch. User definitions override project definitions.
Within one scope, a **nonempty native server map** suppresses `.agents`; an
empty map or a config containing only unrelated settings does not.

Import reads native definitions when present, otherwise same-scope `.agents`
definitions. Apply always writes the native file, retaining fallback settings
when it first creates native server definitions. The shared file remains owned
by the `.agents` adapter. Native JSON path edits preserve hooks, permission
settings, plugin toggles, sibling `mcp` fields and comments. Removing every
native definition permits the client's fallback to load again; manage the
shared target too when removing a service from every consumer.

Project fallback migration requires importing all of its servers first (CLI:
`--from <project>/.agents/mcp.json -a zcode --project`). This prevents raw,
unreviewed fallback credentials from being copied into a new shared native file
outside the normal server-level secret review. User-level migration remains
private. The nested-write mode uses JSON Pointer `/mcp/servers`, so a literal
key named `mcp.servers` or `amp.mcpServers` is never mistaken for nesting.

## MiMo Code verification (2026-11-08)

Verified against [official configuration docs](https://mimo.xiaomi.com/mimocode/config-overrides)
and Xiaomi's source at `3a12665a9e27789b04dc7529d4ce0c10128e9ad8`:
[profile resolver](https://github.com/XiaomiMiMo/MiMo-Code/blob/3a12665a9e27789b04dc7529d4ce0c10128e9ad8/packages/shared/src/global.ts),
[loader and general writer](https://github.com/XiaomiMiMo/MiMo-Code/blob/3a12665a9e27789b04dc7529d4ce0c10128e9ad8/packages/cli/src/config/config.ts),
and [MCP schema](https://github.com/XiaomiMiMo/MiMo-Code/blob/3a12665a9e27789b04dc7529d4ce0c10128e9ad8/packages/cli/src/config/mcp.ts).

Supported files are deep-merged without expanding environment/file references,
then each source's declared IDs are attributed to that source's scope. Project
files override the base user files; the home `.mimocode` overlay is loaded later
and overrides project settings. New user writes select the highest-priority existing
file (JSONC before JSON); new files use `mimocode.jsonc`. Imported user sources
retain explicit managed placements so removal covers every imported layer.
Project imports exclude the private home overlay. Writes preserve each layer's
own raw values and apply only edited modeled fields; unchanged private values
are never materialized into project files. Editor saves refresh a changed project
alias while retaining managed lower layers. The bound alias is checked throughout
the transaction, including removal/disable of the last project placement; canceled
saves retain the old baseline, and already-disabled placements retain their state.
A removed managed project entry is not recreated from private home configuration. MiMo's MCP-add command
currently checks JSON before JSONC despite its comment; the manager follows the
actual load precedence so a write cannot be hidden by an existing JSONC file.

Only the current project and local user layers above are managed. Ancestor
configs, managed organization configs and `MIMOCODE_CONFIG*` per-run injected
layers can still override them and are left untouched. Invalid relative
`MIMOCODE_HOME` values or malformed supported config files abort apply instead
of writing a guessed path. This is Xiaomi's MiMo Code, not KoinaAI/MiMo-CLI.

## Command Code format verification (2026-11-08)

The [official MCP documentation](https://commandcode.ai/docs/mcp) now publishes
user, project and local paths together with the `mcpServers` schema. The
canonical transport key is `transport`; `type` is a documented alias. Only
stdio and HTTP are supported by this adapter, matching the documented CLI
transport choices. An incidental reference to SSE in the OAuth guidance is not
treated as confirmation of an SSE configuration schema.

The user file has one Command Code writer. Shared project `.mcp.json` remains
owned by Claude Code, as for CodeBuddy, and is available through that client's
project controls. Private local files are not discovered or modified: the slug
algorithm is not specified. Local > project > user precedence can therefore
make a private local definition override a managed definition.

Raw `${VAR}`, `${VAR:-default}` and `$${VAR}` values are preserved without local
expansion. Existing `enabled: false` is retained, including transport switches;
manager enablement and the client's own switch remain separate. Windows uses
configuration markers for detection because the OS `cmd.exe` is not evidence
that Command Code is installed. The official favicon is used for the client icon.

## Muse Code verification (2026-11-08)

The current [official settings documentation](https://dev.meta.ai/docs/muse-code/configuration)
requires `schema_version: 1`, and the [MCP documentation](https://dev.meta.ai/docs/muse-code/extending)
explicitly uses `mcp_servers` with `transport: stdio` (command, args, env) or
`transport: streamable_http` (url, headers). This is the settings schema, not
the SDK's separate protocol representation.

Only the user file is managed. Raw `${VAR}` strings remain unexpanded. Client
`enabled` and `mode` survive writes, including transport changes; framing is
retained only for the same transport. A new file receives its schema and MCP
entries in one atomic write. Existing missing/unknown schema versions and any
legacy `mcpServers` key fail import/apply before writes, requiring the user to
resolve the version conflict instead of creating two conflicting server maps.
Unrelated hooks, runtime capabilities and settings remain intact. SSE is not
written to Muse because only streamable HTTP is documented for remote servers.

## Rovo Dev CLI verification (2026-11-08)

Verified the official standalone Rovo plugin `202611.3.1` obtained from the
[public Atlassian plugin endpoint](https://api.atlassian.com/acli/api/v1/plugin/rovodev)
(with OS/architecture headers). Its official
[Darwin arm64 archive](https://acli.atlassian.com/plugins/rovodev/darwin/arm64/202611.3.1/rovodev.zip)
matched published SHA-256 `fb785065c0bd58fe64d2714da9bc70b96eaecef0666d9a383b9f404f8e42831a`.
Running `--version` and `mcp` in an isolated HOME with a recording editor confirmed
that this version creates/opens `~/.rovodev/mcp.json`, initialized as
`{"mcpServers": {}}`. No login, model invocation or real user configuration was used.
The client icon comes from this archive's bundled web GUI favicon.

The [MCP guide](https://support.atlassian.com/rovo/docs/connect-to-an-mcp-server-in-rovo-dev-cli/)
confirms the transport/command/args/url schema. The
[settings guide](https://support.atlassian.com/rovo/docs/manage-rovo-dev-cli-settings/)
still shows the older `mcp_config.json` default. The adapter uses the executable's
observed default, and honors the guide's explicit `mcp.mcpConfigPath` override.
Only absolute or home-relative overrides are managed; per-run `--config-file`
and `--config-override` are left untouched. The current `mcp` editor command
always opened the default even with YAML overrides in the probe, so that command
is not evidence of the effective runtime override path. No authenticated server
connection was tested.

The YAML file is read-only: allowed/disabled server lists and all other settings
remain unchanged. Client-local disabled entries are imported without globally
disabling them. Prepared writes reject a changed target path or invalid YAML
before writing; JSON updates retain unrelated entries and same-transport metadata.

Prepared writes fingerprint read-only source dependencies separately from rollback
files, including only the ZCode fallback scopes actually used and the MiMo layers
needed for reconciliation. Unrelated project configs are not read for global-only
writes. Same-batch Agents updates are projected into first-write ZCode migration.
These inputs are rechecked
at validation and before each write. Batch writes to one target compare against
the exact bytes committed by the batch, never a reread that could bless an external
revision. Rollback restores only still-owned versions and preserves external edits detected at the ownership check
even when failure occurred in a different dependency. Retained project parameters/environment values are included
in redacted secret warnings before CLI or desktop confirmation.

MiMo workspace placements are disabled with the native `enabled: false` switch, retaining their project-only definition for safe re-enabling. Deleting a placement/server removes the managed entry. Private user overlays are never used to reconstruct a missing managed project entry.

When an apply omits the previous configuration, MiMo uses saved placement provenance to distinguish a new project assignment from a removed imported entry. If neither baseline is available and the same id exists only in user layers, the ambiguous assignment is rejected with guidance to provide the previous configuration or re-import. GUI and CLI edits provide their previous configuration explicitly.

Failed-batch and failed-store-save compensation restore only payloads still owned by the batch and report skipped external revisions. Dependency aliases and active-path markers are rechecked at write boundaries. These checks are optimistic; a noncooperating editor can still race the final check and atomic replacement.
