<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  一款跨平台桌面應用程式，用於統一管理 Claude Code、Codex、Cursor、VS Code、OpenCode、Gemini CLI 等用戶端的 MCP 伺服器設定。
</p>

<p align="center">
  <a href="https://github.com/xjeway/mcp-manager/stargazers"><img src="https://img.shields.io/github/stars/xjeway/mcp-manager?style=flat&label=%E2%98%85&color=f59e0b" alt="GitHub stars" /></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-0f172a.svg" alt="MIT License" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24c8db.svg" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/React-19-61dafb.svg" alt="React 19" />
  <img src="https://img.shields.io/badge/Rust-stable-f74c00.svg" alt="Rust stable" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-16a34a.svg" alt="Cross-platform" />
</p>

<p align="center">
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <b>繁體中文</b> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>下載 MCP Manager</ins></a></h3>

## 目錄

- [功能特色](#功能特色)
- [安裝](#安裝)
- [支援的用戶端](#支援的用戶端)
- [命令列（預覽）](#命令列預覽)
- [隱私與網路](#隱私與網路)
- [開發](#開發)
- [Star History](#star-history)
- [免責聲明](#免責聲明)

## 功能特色

- **單一工作區**管理所有 MCP 伺服器，可從用戶端既有的設定匯入
- **線上市集**：在 GitHub MCP Registry、官方 MCP Registry 或任何相容的 registry 中尋找伺服器，只需點幾下即可加入
- 以**表單或原始 JSON** 編輯
- **一次套用到多個用戶端**，並在寫入前檢視有風險的寫入操作，自動備份且支援還原
- 淺色／深色主題，介面支援 11 種語言並跟隨系統語言

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager，淺色主題" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager，深色主題" /></td>
  </tr>
</table>

## 安裝

從[最新版本](https://github.com/xjeway/mcp-manager/releases/latest)下載適用於你平台的安裝套件：

| 平台 | 安裝套件 |
| --- | --- |
| macOS | `.dmg`，或 `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe`（x64 / arm64）或 `.msi`（x64） |
| Linux | `.deb`、`.rpm` 或 `.AppImage` |

Apple Silicon 與 ARM 裝置請選擇 `arm64` / `aarch64`，其他情況請選擇 `x64` / `x86_64` / `amd64`。

<details>
<summary>macOS 提示應用程式已損毀或無法開啟？</summary>

發布版本尚未經過公證。請將應用程式移至 `/Applications`，然後執行：

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## 支援的用戶端

以下每個用戶端都同時支援**匯入**（從其設定檔讀取既有的 MCP 伺服器）與**套用**（將統一設定寫回）。

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="../../public/logos/apps/claudecode.svg" alt="Claude Code logo" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="../../public/logos/apps/claude.svg" alt="Claude Desktop logo" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="../../public/logos/apps/codex.svg" alt="Codex logo" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="../../public/logos/apps/cursor.svg" alt="Cursor logo" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="../../public/logos/apps/opencode.svg" alt="OpenCode logo" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="../../public/logos/apps/githubcopilot.svg" alt="GitHub Copilot logo" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="../../public/logos/apps/gemini.svg" alt="Gemini CLI logo" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="../../public/logos/apps/antigravity.svg" alt="Antigravity logo" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="../../public/logos/apps/iflow.svg" alt="iFlow logo" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="../../public/logos/apps/qwen.svg" alt="Qwen Code logo" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="../../public/logos/apps/cline.svg" alt="Cline logo" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="../../public/logos/apps/windsurf.svg" alt="Windsurf logo" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../public/logos/apps/kiro.svg" alt="Kiro logo" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../public/logos/apps/qoder.svg" alt="Qoder logo" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../public/logos/apps/vscode.svg" alt="VS Code logo" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## 命令列（預覽）

`mcpmgr` 與桌面應用程式共用同一份伺服器清單，在任一端所做的變更都會同步顯示在另一端。

```bash
npx @mcpmgr/cli list                      # 或：npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

在終端機中，`add` 會引導你選擇伺服器、用戶端與範圍，並在寫入前預覽每個檔案。每個步驟也都有對應的旗標（`--yes`、`--dry-run`、`--json`……）；在 Claude Code 或 Codex 等 AI 程式設計代理中執行時，它會略過互動提示，直接以該代理的用戶端為目標。

## 隱私與網路

MCP Manager 只會在檢查更新時，以及市集頁面開啟期間查詢 MCP registry 時連線（結果快取 6 小時，離線時仍可使用）。在 **設定 → 線上市集** 中關閉市集，即可完全停止這些請求。

## 開發

需要 Node.js 20+、Rust stable，以及你所用平台的 [Tauri 前置需求](https://tauri.app/start/prerequisites/)。

```bash
make install      # 安裝相依套件
make tauri-dev    # 執行桌面應用程式
make test && make check
```

核心邏輯（用戶端轉接器、解析、安全寫入）位於 `crates/mcp-manager-core`，由 `src-tauri/` 中的 Tauri 應用程式與 `crates/mcp-manager-cli/` 中的命令列工具共用。發布流程說明請參閱 [`docs/releasing.md`](../../docs/releasing.md)。

歡迎提交 Issue 與 Pull Request；較大的變更請先開立 Issue。修改面向使用者的文件時，請保持 [`README.md`](../../README.md)、[`README.zh-CN.md`](../../README.zh-CN.md) 與 [`docs/readme/`](./) 同步。

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 免責聲明

本專案主要在 AI 協助下開發，並經人工維護者審閱，因此仍可能包含錯誤。MCP Manager 會改寫其他工具的設定檔；雖然內建備份與還原功能，你仍應自行備份重要的設定。本軟體依 [MIT 授權條款](../../LICENSE)以「現狀」提供。
