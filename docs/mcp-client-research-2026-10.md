# MCP 客户端配置调研与新客户端评估（2026-10）

调研日期：2026-10-16。对象：`SupportedApp` 里已有的 15 个客户端，以及近几个月新出现、流行的 agent / harness / assistant。

## 方法与可信度

官网（cursor.com、developers.openai.com、opencode.ai、code.visualstudio.com 等）在本次环境里被网络策略拦截，所以能读到的一手资料有：

- Claude Code 官方文档 `code.claude.com/docs/en/mcp`
- 各开源项目仓库里的文档和源码：codex 的 `config.schema.json`、opencode、gemini-cli（含 `mcp-client.ts`）、qwen-code、copilot-cli changelog、crush、goose、hermes-agent、openclaw、Factory Droid、Kilo、Cline、Mistral Vibe、Kimi CLI、Zed、iflow-cli README

闭源客户端（Cursor、Windsurf、Kiro、Antigravity、Qoder、Amp、Claude Desktop）只有第三方文章和搜索摘要。下面每条结论带可信度标记：

- **[已证实]**：读到了官方文档或源码。
- **[二手]**：只来自第三方文章，落地前需要本机验证。

## 一、总结

1. **iFlow CLI 已停服**：README 顶部写明 2026-04-17（UTC+8）关闭。建议标记为 legacy，保留读取，写入默认关闭。[已证实]
2. **Copilot CLI 的项目级配置变了**：1.0.22（2026-04）起不再读 `.vscode/mcp.json` 和 devcontainer，只读工作区 `.mcp.json`，也自动加载 `.github/mcp.json`。能力矩阵里“项目配置通过 VS Code 管理”已经过时，而且 `.mcp.json` 和 Claude Code 是同一个文件。[已证实]
3. **Codex 现在支持项目级配置**：官方文档说明可在受信任项目中使用 `.codex/config.toml`，矩阵里还写着“无权威文档，保持禁用”，这个前提已经不成立。[搜索摘要，官方页面未能直接读取；字段来自 schema，已证实]
4. **Antigravity 2.0 可能迁移了配置路径**：Google codelab 称 2.0 / IDE / CLI 共用 `~/.gemini/config/mcp_config.json`，另一篇指南列出 CLI 专用的 `~/.gemini/antigravity-cli/mcp_config.json`，当前代码用的是旧路径 `~/.gemini/antigravity/mcp_config.json`。需要在真机上确认。[二手，冲突]
5. **Windsurf 已更名 Devin Desktop**（二手来源称 2026-06-02）。配置路径据称没变，仍是 `~/.codeium/windsurf/mcp_config.json`。更重要的是：多个来源说远程服务器必须用 `serverUrl`，写成 `url` 会被静默忽略，而当前通用写入器对所有客户端都写 `url`。[二手，但风险高，优先验证]
6. **Claude Code 的字段变多**：新增 `headersHelper`、`oauth`、`alwaysLoad`、`timeout`、`ws` 传输、`${VAR:-default}`。SSE 已弃用。路径和结构没变。[已证实]
7. **新客户端里最值得做的**：Kilo Code、Factory Droid、Kimi CLI 几乎可以直接复用现有解析/写入逻辑；Zed、Mistral Vibe、Hermes、Goose 需要新的格式处理。

## 二、已支持客户端核对

| 客户端 | 结论 | 说明 |
| --- | --- | --- |
| Claude Code | 路径无变化，字段增加 | user/local 在 `~/.claude.json`（local 为 `projects[path].mcpServers`），project 在 `.mcp.json`。优先级 local > project > user > plugin > claude.ai connectors，`managedMcpServers` 最高。带 `url` 但没 `type` 的条目会报错并被跳过，现有写入器总是写 `type`，没问题。`streamable-http` 是 `http` 的别名。 |
| Claude Desktop | 无变化 | 远程服务器走 Connectors，本地 stdio 才写 JSON，现有“不写 headers”的处理正确。`.dxt` 已改名 `.mcpb`。Windows 的 MSIX/商店版安装路径会被重定向到 LocalCache，检测路径时要考虑。[二手] |
| Codex | 需更新 | 新增项目级 `.codex/config.toml`（仅受信任项目）。schema 里的 MCP 字段：`command/args/env/env_vars/cwd`、`url`、`bearer_token_env_var`、`http_headers`、`env_http_headers`、`http_headers_helper`、`auth`、`oauth`、`scopes`、`enabled`、`required`、`enabled_tools/disabled_tools`、`startup_timeout_sec`、`tool_timeout_sec` 等。 |
| Cursor | 无变化 | `~/.cursor/mcp.json`、`.cursor/mcp.json`。变量插值 `${env:NAME}`、`${userHome}`、`${workspaceFolder}`；远程可用 `auth`（CLIENT_ID/CLIENT_SECRET/scopes）。[二手] |
| VS Code | 未能核实 | 官方文档被拦截，没发现变化证据。 |
| OpenCode | 无结构变化 | `mcp` 键，`type: local/remote`，`command` 是数组，`environment`。新增 `cwd`、`timeout`、`oauth`，企业可通过 `.well-known/opencode` 下发默认服务器。[已证实] |
| GitHub Copilot | 需更新 | 用户级仍是 `~/.copilot/mcp-config.json`，支持 `COPILOT_HOME`；用户设置在 `~/.copilot/settings.json`。项目级见上文第 2 条。[已证实] |
| Gemini CLI | 无需改 | 流式 HTTP 推荐写法已是 `url` + `type: "http"`，`httpUrl` 标注弃用（源码里有迁移提示），现有写入器的输出是对的。新字段：`trust`、`includeTools/excludeTools`、`cwd`、`oauth`。[已证实] |
| Antigravity | 需验证路径 | 见上文第 4 条。 |
| iFlow | 已停服 | 见上文第 1 条。 |
| Qwen Code | 无需改 | `httpUrl`（HTTP）/`url`（SSE）两种写法在文档里都有。[已证实，仅读了文档] |
| Cline | 路径需分场景 | CLI：`~/.cline/data/settings/cline_mcp_settings.json`，可由 `CLINE_MCP_SETTINGS_PATH` 覆盖，与当前代码一致。VS Code 扩展用的是另一个文件，要在 UI 里打开才能确定位置，当前代码没有覆盖。[已证实] |
| Windsurf | 需验证 | 见上文第 5 条。 |
| Kiro | 无变化 | user `~/.kiro/settings/mcp.json`，workspace `.kiro/settings/mcp.json`，合并且 workspace 优先。字段 `disabled`、`autoApprove`、`disabledTools`；`${VAR}` 只展开白名单里的变量。[二手] |
| Qoder | 可补项目级 | 官方文档：user `~/.qoder/settings.json`，project `.qoder/settings.json`（需批准）、`.qoder/settings.local.json`，另外读项目根 `.mcp.json`。当前适配器注释写“只管 user 级”，可以补上 project。[二手] |

### 一个需要先确认的风险

通用写入器（`crates/mcp-manager-core/src/adapters/mod.rs` 里的 `standard_mcp_servers_at`）为每个受管服务器从零重建条目，只写 `command/args/env` 或 `url`、`type`、`headers`。`storage/mod.rs` 的 `apply_merge_json_object_entries` 会先删掉受管 key，再整条写入新值，不合并旧字段。所以对 MCP Manager 管理的服务器，用户手写的 `oauth`、`headersHelper`、`timeout`、`alwaysLoad`、`autoApprove`、`disabledTools`、`trust` 等未建模字段，会在 apply 时丢失（未受管的服务器不受影响）。这是读代码得出的结论，没有跑测试复现。随着各客户端字段增多（见上表），建议对未建模字段做透传，或至少在风险审查里提示。

另外，该写入路径用 `serde_json` 整体重新序列化文件，JSONC 里的注释（OpenCode、Kilo、Zed 常见）也会丢，新增 Zed 这类把 MCP 配置和其他设置混在一个文件里的客户端之前，需要先解决这点。

## 三、新客户端评估

“适配成本”以现有模型（stdio/http/sse + command/args/env + url/headers）为基准。

### 建议优先做（A）

| 客户端 | 配置位置 | 格式 | 适配成本 | 备注 |
| --- | --- | --- | --- | --- |
| **Kilo Code**（CLI + VS Code 扩展） | user `~/.config/kilo/kilo.json(c)`；project `kilo.json(c)` 或 `.kilo/kilo.json(c)` | `mcp` 键，和 OpenCode 完全同构（`type: local/remote`，`command` 数组） | 低：复用 OpenCode 适配器 | Kilo 也会读 `opencode.json(c)` 作为备选，同一项目文件会被两个客户端同时当作配置，需要去重。[已证实] |
| **Factory Droid** | user `~/.factory/mcp.json`；project 和上级目录的 `.factory/mcp.json` | `mcpServers`，`type: http/sse/stdio`，`disabled`、`disabledTools/enabledTools`，支持 `${VAR}` | 低 | 官方明确警告项目级文件会入库，不要放 token，正好可以接入现有“写入前风险审查”。[已证实] |
| **Kimi CLI** | user `~/.kimi/mcp.json` | 标准 `mcpServers`（`url`/`headers`、`command`/`args`） | 低 | README 提到新版首次启动会迁移 `~/.kimi/` 的数据，新版实际目录要在真机上确认。[已证实，路径需确认] |

### 值得做，但要新增格式处理（B）

| 客户端 | 配置位置 | 格式 | 备注 |
| --- | --- | --- | --- |
| **Zed** | `settings.json` 的 `context_servers`（user；项目可用 `.zed/settings.json`） | JSON（带注释），本地 `command/args/env`，远程 `url` + `headers`，无 `type` | 必须按 JSONC 保留注释和其他设置。没有 `Authorization` 头时走 OAuth。已有 JSONC 解析器可复用。[已证实，路径未逐项核对] |
| **Mistral Vibe** | `~/.vibe/config.toml`，项目 `.vibe/` | TOML 数组表 `[[mcp_servers]]`，`name`、`transport`（`http`/`streamable-http`/`stdio`）、`url`、`command`、`args`、`env`、`[mcp_servers.auth]` | 与 Codex 的 `[mcp_servers.<name>]` 表结构不同，需要新的 TOML 写入分支。[已证实] |
| **Hermes Agent**（Nous Research） | `~/.hermes/config.yaml` 的 `mcp_servers` | YAML，`command/args/env` 或 `url/headers`，`tools.include/exclude`，`enabled` | 增长快，官方提供从 Claude Code 导入。YAML 写入要保留注释，当前依赖 `serde_yaml` 会丢注释，需要评估。[已证实] |
| **Goose** | `~/.config/goose/config.yaml` 的 `extensions` | YAML，`type: stdio/streamable_http/builtin`，`cmd`、`args`、`envs`、`uri`、`timeout`、`enabled` | 字段名和 MCP 惯例不同（`cmd`、`envs`、`uri`），且 `extensions` 里混有 builtin 非 MCP 项，只能管理其中的 MCP 部分。同样有 YAML 注释问题。[已证实] |
| **Amp** | `amp.mcpServers`（settings.json），项目 `.amp/settings.json` 需手动批准 | `command/args/env`、`url/headers` | 用户级路径没能确认；官方更推荐把 MCP 随 skill 打包（`mcp.json`）。[二手] |
| **OpenClaw**（个人 assistant 网关） | OpenClaw 配置里的 `mcp.servers` | JSON，`command/args/env/cwd` 或 `url/headers/auth: "oauth"`，另有 `connectionTimeoutMs/requestTimeoutMs` | 它更像 assistant 而非 coding agent，并自带 `openclaw mcp add/set/list` 命令。建议通过 CLI 间接管理，不直接改配置文件。配置文件位置未核实。[已证实，路径未确认] |

### 暂缓（C）

- **Crush**：新版把配置改成 `crushrc`（一份被真实 Bash 解释器执行的脚本），旧 `crush.json` 标记为 deprecated 但仍支持。`mcp add --command/--url` 是 `crushrc` 的内置命令。往可执行脚本里自动写入既难可靠地回读，也有安全顾虑；如果要支持，只读写旧的 JSON（`mcp` 键，`type: stdio/http/sse`、`oauth`、`disabled_tools`），并提示已弃用。[已证实]
- **Continue**：每个服务器一个 YAML 文件（`.continue/mcpServers/*.yaml`），格式和其他客户端差别较大，收益有限。[已证实]
- **Roo Code / Trae / Warp / Junie / Augment**：本次没有读到可靠的 MCP 配置文档，不建议在没验证前实现。
- **Pi（badlogic/pi-mono）**：README 里没有 MCP 相关内容，不适用。[已证实]

## 四、建议的落地顺序

1. **修正已有适配器**（影响现有用户，优先级最高）：
   - 验证并修复 Windsurf 的 `serverUrl`；
   - 在真机上确认 Antigravity 2.0 路径，必要时同时检测新旧两个路径；
   - iFlow 标记为 legacy；
   - 确认写入时是否丢失未建模字段（见上文风险）。
2. **更新能力矩阵**：Copilot CLI 项目级改为 `.mcp.json`（与 Claude Code 共享，需要处理同一文件被两个客户端管理的情况，Qoder 适配器里已有类似处理）；Codex 增加项目级；Qoder 增加项目级。
3. **新增 A 类客户端**：Kilo Code、Factory Droid、Kimi CLI。
4. **再做 B 类**，顺序建议：Zed → Mistral Vibe → Hermes → Goose；Amp 和 OpenClaw 待路径验证后再定。

## 五、没能验证的项目

- Cursor、VS Code、Windsurf/Devin Desktop、Kiro、Antigravity、Qoder、Amp 的官方页面
- Codex 官方 MCP 文档页（只读到了 schema 和搜索摘要）
- Zed 项目级 `.zed/settings.json`、Hermes/OpenClaw/Kimi 的新版目录结构

## 来源

- [Claude Code MCP 文档](https://code.claude.com/docs/en/mcp)
- [Codex config schema](https://github.com/openai/codex/blob/main/codex-rs/core/config.schema.json)
- [Copilot CLI changelog](https://github.com/github/copilot-cli/blob/main/changelog.md)
- [OpenCode MCP 文档](https://github.com/sst/opencode/blob/dev/packages/web/src/content/docs/mcp-servers.mdx)
- [Gemini CLI MCP 文档](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md)、[mcp-client.ts](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/tools/mcp-client.ts)
- [iFlow CLI README](https://github.com/iflow-ai/iflow-cli)
- [Kilo MCP 文档](https://github.com/Kilo-Org/kilocode)
- [Factory Droid MCP 文档](https://github.com/Factory-AI/factory)
- [Kimi CLI MCP 文档](https://github.com/MoonshotAI/kimi-cli/blob/main/docs/en/customization/mcp.md)
- [Mistral Vibe README](https://github.com/mistralai/mistral-vibe)
- [Hermes Agent MCP 文档](https://github.com/NousResearch/hermes-agent)
- [Goose extensions 文档](https://github.com/block/goose)
- [Crush README](https://github.com/charmbracelet/crush)
- [Zed MCP 文档](https://github.com/zed-industries/zed/blob/main/docs/src/ai/mcp.md)
- [OpenClaw MCP 文档](https://github.com/openclaw/openclaw/blob/main/docs/cli/mcp.md)
- [Cline MCP 文档](https://github.com/cline/cline)
- 二手来源（搜索摘要）：[Cursor MCP](https://cursor.com/docs/mcp)、[Windsurf MCP 指南](https://designrevision.com/blog/add-mcp-server-to-windsurf)、[Antigravity codelab](https://codelabs.developers.google.com/google-workspace-mcp-antigravity)、[Qoder MCP 文档](https://docs.qoder.com/cli/mcp-reference)、[Amp MCP 手册](https://ampcode.com/manual/mcp.md)、[Kiro MCP 配置](https://docs.ionos.com/cloud/ai/mcp-server/connect-to-an-ai-client/kiro)
