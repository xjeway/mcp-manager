# MCP Manager

<p align="center">
  <img src="./src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<p align="center">
  一个用统一工作台管理 MCP Server 配置的桌面应用。
</p>

<p align="center">
  <a href="./README.md">English</a> | 简体中文
</p>

## 项目简介

`MCP Manager` 的目标是把不同客户端里的 MCP 配置集中到一个界面中维护，再按目标客户端生成并应用配置。

当前项目采用：

- Tauri 2 + Rust 作为桌面端后端
- React + TypeScript 作为前端
- YAML 作为统一配置源
- 多客户端导入 / 应用流程

## 功能特性

- 统一 MCP 工作台
- 支持表单模式和 JSON 模式新增、编辑 Server
- 支持从本地客户端配置导入现有条目
- 支持向多个客户端应用配置
- apply 过程支持备份与回滚
- 写入前展示风险警告
- 支持浅色、深色、跟随系统主题
- 支持英文与简体中文界面

## 支持的客户端

| 客户端 | 导入 | 应用 |
| --- | --- | --- |
| Claude Code | ✅ | ✅ |
| Claude Desktop | ✅ | ✅ |
| Codex | ✅ | ✅ |
| Cursor | ✅ | ✅ |
| OpenCode | ✅ | ✅ |
| GitHub Copilot | ✅ | ✅ |
| Gemini CLI | ✅ | ✅ |
| Antigravity | ✅ | ✅ |
| iFlow | ✅ | ✅ |
| VS Code | ✅ | ✅ |
| Windsurf | 规划中 | 规划中 |
| Cline | 规划中 | 规划中 |
| RooCode | 规划中 | 规划中 |
| Kilo Code | 规划中 | 规划中 |
| Amazon Q | 规划中 | 规划中 |
| Qoder | 规划中 | 规划中 |
| Auggie CLI | 规划中 | 规划中 |
| Qwen Code | 规划中 | 规划中 |
| CodeBuddy | 规划中 | 规划中 |
| CoStrict | 规划中 | 规划中 |
| Crush | 规划中 | 规划中 |
| Factory Droid | 规划中 | 规划中 |

当前实现已落地 VS Code、Cursor、Claude Code、Claude Desktop、Codex、OpenCode、GitHub Copilot CLI 配置、Gemini CLI、Antigravity、iFlow 的导入 / 应用适配器。其余客户端已加入路线图，但尚未接入 Rust 侧适配层。

## 界面预览

界面截图和设计参考可见 [`docs/`](./docs/)。

## 快速开始

### 环境要求

- Node.js 20+
- npm 10+
- Rust stable toolchain
- 当前平台所需的 Tauri 依赖

### 安装依赖

```bash
make install
```

### 启动 Web 前端

```bash
make dev
```

### 启动桌面应用

```bash
make tauri-dev
```

### 构建

```bash
make build
```

### 测试

```bash
make test
```

### 全量检查

```bash
make check
```

## 常用命令

```bash
npm run dev
npm run build
npm test
npm run tauri -- dev
npm run tauri -- build
```

## 配置模型

- 统一配置源存放在 `config/servers.yaml`
- 应用会读取本地客户端配置并转换为内部模型
- apply 时会生成客户端对应配置，并带备份与回滚支持

当前版本主要聚焦配置管理。运行时生命周期管理，例如进程启停、日志查看、健康检查等，不属于 v1 范围。

## 项目结构

```text
mcp-manager/
  src/                前端应用
  src-tauri/          Tauri 应用与 Rust 后端
  public/             静态资源
  docs/               文档与设计参考
  openspec/           变更与规格记录
```

### 后端模块

- `platform`：平台路径解析与运行上下文
- `adapters`：按客户端实现导入与 apply 逻辑
- `core`：统一配置模型与合并规则
- `parser`：YAML / JSON / TOML 解析与配置提取
- `storage`：原子写、备份与回滚
- `commands`：对前端暴露的 Tauri 命令

## 参与贡献

欢迎提交 Issue 和 Pull Request。

如果是比较大的改动，建议先开一个 issue 或 discussion，对范围和方向先达成一致，再开始实现。

## License

MIT
