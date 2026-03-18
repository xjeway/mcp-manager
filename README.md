# MCP Manager

<p align="center">
  <img src="./src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<p align="center">
  A desktop app for managing MCP server configuration with one unified workspace.
</p>

<p align="center">
  English | <a href="./README.zh-CN.md">简体中文</a>
</p>

## Overview

`MCP Manager` helps you maintain MCP server definitions in one place, then render and apply them to different clients such as VS Code, Cursor, Claude Code, and Codex.

The project is built as a desktop application with:

- Tauri 2 + Rust backend
- React + TypeScript frontend
- YAML as the source of truth
- Multi-client import / apply workflow

## Features

- Unified MCP workspace for all configured servers
- Create and edit servers in form mode or JSON mode
- Import existing entries from local client configuration
- Apply generated configuration to multiple supported clients
- Backup and rollback support during apply
- Warnings for risky changes before write
- Light mode, dark mode, and system theme support
- English and Simplified Chinese UI

## Supported Clients

| Client | Import | Apply |
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
| Windsurf | Planned | Planned |
| Cline | Planned | Planned |
| RooCode | Planned | Planned |
| Kilo Code | Planned | Planned |
| Amazon Q | Planned | Planned |
| Qoder | Planned | Planned |
| Auggie CLI | Planned | Planned |
| Qwen Code | Planned | Planned |
| CodeBuddy | Planned | Planned |
| CoStrict | Planned | Planned |
| Crush | Planned | Planned |
| Factory Droid | Planned | Planned |

The current implementation ships import/apply adapters for VS Code, Cursor, Claude Code, Claude Desktop, Codex, OpenCode, GitHub Copilot CLI config, Gemini CLI, Antigravity, and iFlow. The remaining clients above are tracked as roadmap targets and are not yet wired into the Rust adapter layer.

## Screenshots

Desktop UI screenshots and references are available in [`docs/`](./docs/) and the historical UI reference project [`mcp-server-manager/`](./mcp-server-manager/).

## Getting Started

### Prerequisites

- Node.js 20+
- npm 10+
- Rust stable toolchain
- Tauri system dependencies for your platform

### Install

```bash
make install
```

### Run in Web Mode

```bash
make dev
```

### Run as Desktop App

```bash
make tauri-dev
```

### Build

```bash
make build
```

### Test

```bash
make test
```

### Full Check

```bash
make check
```

## Common Commands

```bash
npm run dev
npm run build
npm test
npm run tauri -- dev
npm run tauri -- build
```

## Configuration Model

- Canonical data is stored in `config/servers.yaml`
- The app reads local client configuration and converts it into the internal model
- Apply writes client-specific output with backup and rollback support

Current scope is focused on configuration management. Runtime lifecycle management such as process start, stop, logs, and health checks is intentionally out of scope for v1.

## Project Structure

```text
mcp-manager/
  src/                frontend application
  src-tauri/          tauri app + rust backend
  public/             static assets
  docs/               notes and design references
  openspec/           change and spec tracking
```

### Backend Modules

- `platform`: OS-aware path resolution and environment context
- `adapters`: per-client import and apply logic
- `core`: canonical config model and merge behavior
- `parser`: YAML / JSON / TOML parsing and extraction
- `storage`: atomic write, backup, and rollback
- `commands`: Tauri commands exposed to the frontend

## Contributing

Issues and pull requests are welcome.

If you plan to contribute a non-trivial change, open an issue or discussion first so the scope and direction are clear before implementation.

## License

MIT
