<h1 align="center">
  <img src="./src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  A cross-platform desktop app for managing MCP server configs across Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI, and more.
</p>

<p align="center">
  <a href="https://github.com/xjeway/mcp-manager/stargazers"><img src="https://img.shields.io/github/stars/xjeway/mcp-manager?style=flat&label=%E2%98%85&color=f59e0b" alt="GitHub stars" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-0f172a.svg" alt="MIT License" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24c8db.svg" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/React-19-61dafb.svg" alt="React 19" />
  <img src="https://img.shields.io/badge/Rust-stable-f74c00.svg" alt="Rust stable" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-16a34a.svg" alt="Cross-platform" />
</p>

<p align="center">
  <sub><b>English</b> · <a href="./README.zh-CN.md">简体中文</a> · <a href="./docs/readme/README.zh-TW.md">繁體中文</a> · <a href="./docs/readme/README.ja.md">日本語</a> · <a href="./docs/readme/README.ko.md">한국어</a> · <a href="./docs/readme/README.de.md">Deutsch</a> · <a href="./docs/readme/README.fr.md">Français</a> · <a href="./docs/readme/README.es.md">Español</a> · <a href="./docs/readme/README.it.md">Italiano</a> · <a href="./docs/readme/README.pt-BR.md">Português</a> · <a href="./docs/readme/README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>Download MCP Manager</ins></a></h3>

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Supported Clients](#supported-clients)
- [Command Line (Preview)](#command-line-preview)
- [Privacy & Network](#privacy--network)
- [Development](#development)
- [Star History](#star-history)
- [Disclaimer](#disclaimer)

## Features

- **One workspace** for every MCP server, imported from the configs your clients already have
- **Online marketplace**: find servers in the GitHub MCP Registry, the official MCP Registry, or any compatible registry, and add them in a few clicks
- **Form or raw JSON** editing
- **Apply to many clients at once**, with a review of risky writes, automatic backups, and rollback
- Light / dark theme, and a UI in 11 languages that follows your system language

<table>
  <tr>
    <td><img src="./docs/page-en.png" alt="MCP Manager, light theme" /></td>
    <td><img src="./docs/page-en-dark.png" alt="MCP Manager, dark theme" /></td>
  </tr>
</table>

## Installation

Download the package for your platform from the [latest release](https://github.com/xjeway/mcp-manager/releases/latest):

| Platform | Package |
| --- | --- |
| macOS | `.dmg`, or `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) or `.msi` (x64) |
| Linux | `.deb`, `.rpm`, or `.AppImage` |

Pick `arm64` / `aarch64` for Apple Silicon and ARM devices, `x64` / `x86_64` / `amd64` otherwise.

<details>
<summary>macOS says the app is damaged or can't be opened?</summary>

Release builds are not notarized yet. Move the app to `/Applications`, then run:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Supported Clients

Every client below supports both **import** (read existing MCP servers from its config) and **apply** (write the unified config back).

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="./public/logos/apps/claudecode.svg" alt="Claude Code logo" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="./public/logos/apps/claude.svg" alt="Claude Desktop logo" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="./public/logos/apps/codex.svg" alt="Codex logo" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="./public/logos/apps/cursor.svg" alt="Cursor logo" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="./public/logos/apps/opencode.svg" alt="OpenCode logo" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="./public/logos/apps/githubcopilot.svg" alt="GitHub Copilot logo" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="./public/logos/apps/gemini.svg" alt="Gemini CLI logo" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="./public/logos/apps/antigravity.svg" alt="Antigravity logo" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="./public/logos/apps/iflow.svg" alt="iFlow logo" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="./public/logos/apps/qwen.svg" alt="Qwen Code logo" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="./public/logos/apps/cline.svg" alt="Cline logo" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="./public/logos/apps/windsurf.svg" alt="Windsurf logo" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="./public/logos/apps/kiro.svg" alt="Kiro logo" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="./public/logos/apps/qoder.svg" alt="Qoder logo" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="./public/logos/apps/vscode.svg" alt="VS Code logo" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## Command Line (Preview)

`mcpmgr` shares the desktop app's server list, so changes made in either show up in the other.

```bash
npx @mcpmgr/cli list                      # or: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

In a terminal, `add` walks you through servers, clients and scope, and previews every file before writing. Each step also has a flag (`--yes`, `--dry-run`, `--json`, …); inside an AI coding agent such as Claude Code or Codex, it skips the prompts and targets that agent's client.

## Privacy & Network

MCP Manager only goes online to check for updates and, while the marketplace page is open, to query MCP registries (cached for 6 hours, usable offline). Turn the marketplace off under **Settings → Online marketplace** to stop those requests entirely.

## Development

Requires Node.js 20+, Rust stable, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
make install      # install dependencies
make tauri-dev    # run the desktop app
make test && make check
```

The core logic (client adapters, parsing, safe writes) lives in `crates/mcp-manager-core`, shared by the Tauri app in `src-tauri/` and the CLI in `crates/mcp-manager-cli/`. Releases are described in [`docs/releasing.md`](./docs/releasing.md).

Issues and pull requests are welcome; for larger changes, please open an issue first. When changing user-facing docs, keep [`README.md`](./README.md), [`README.zh-CN.md`](./README.zh-CN.md), and [`docs/readme/`](./docs/readme/) in sync.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Disclaimer

This project is developed primarily with AI assistance and reviewed by a human maintainer, so it may still contain mistakes. MCP Manager rewrites other tools' config files; backups and rollback are built in, but keep your own backups of important configs. Provided "as is" under the [MIT License](./LICENSE).
