<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [為什麼選擇 MCP Manager](#為什麼選擇-mcp-manager)
- [截圖](#截圖)
- [下載與安裝](#下載與安裝)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#linux-使用者) · [從原始碼建置](#從原始碼建置)
- [快速開始](#快速開始)
- [支援的用戶端](#支援的用戶端)
- [命令列（預覽）](#命令列預覽)
- [運作方式](#運作方式)
- [適用範圍](#適用範圍)
- [專案結構](#專案結構)
- [開發](#開發)
- [發布自動化](#發布自動化)
- [參與貢獻](#參與貢獻)
- [Star History](#star-history)
- [免責聲明](#免責聲明)
- [授權條款](#授權條款)

## 為什麼選擇 MCP Manager

`MCP Manager` 是一款跨平台桌面應用程式，專為不想手動編輯多個 MCP 用戶端設定檔的使用者打造。

- 為所有已設定的伺服器維護一個統一的 MCP 工作區
- 從本機用戶端設定匯入既有項目
- 在 GitHub MCP Registry、官方 MCP Registry，或任何你新增且實作了 MCP Registry API 的 registry 中尋找伺服器，只需點幾下即可加入
- 以表單模式或原始 JSON 模式編輯伺服器
- 將產生的設定套用到多個支援的用戶端
- 在檔案變更前檢視有風險的寫入操作
- 套用過程中保留備份並支援還原
- 支援淺色模式、深色模式或跟隨系統主題
- 介面支援英文、簡體中文、繁體中文、日文、韓文、德文、法文、西班牙文、義大利文、巴西葡萄牙文與俄文（預設跟隨系統語言）

## 截圖

![MCP Manager 英文介面，淺色主題](../../docs/page-en.png)

![MCP Manager 英文介面，深色主題](../../docs/page-en-dark.png)

## 下載與安裝

### 系統需求

- 提供 macOS、Windows 與 Linux 的桌面安裝套件
- 從原始碼建置需要 Node.js 20+、npm 10+、Rust stable，以及你所用平台的 Tauri 系統相依套件

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### 手動下載

下載位置：

- <https://github.com/xjeway/mcp-manager/releases/latest>

macOS 安裝檔名稱範例：

- `MCP-Manager-<version>-<arch>.dmg`

架構說明：

- Apple Silicon Mac：選擇 `arm64` 或 `aarch64` 安裝檔
- Intel Mac：選擇 `x64` 或 `x86_64` 安裝檔

下載正確的 macOS 安裝檔後，在本機開啟：

```bash
open ~/Downloads/MCP-Manager*.dmg
```

在 Apple 簽署與公證準備完成之前，目前公開的 macOS 版本可能仍未簽署。如果 Gatekeeper 阻擋了 `MCP Manager.app`，請使用以下替代流程：

1. 將 `MCP Manager.app` 拖曳到 `/Applications`。
2. 在 Finder 中對該應用程式按一下右鍵，選擇 `Open`。
3. 如果 macOS 仍提示無法開啟應用程式或應用程式已損毀，請移除隔離標記後再試一次：

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### 手動下載

下載位置：

- <https://github.com/xjeway/mcp-manager/releases/latest>

Windows 安裝檔名稱範例：

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

架構說明：

- Windows on ARM 裝置：選擇 `arm64` 安裝檔
- Intel / AMD 電腦：選擇 `x64` 或 `x86_64` 安裝檔

套件說明：

- `setup.exe`：Windows x64 與 Windows arm64 皆有提供
- `.msi`：隨 Windows x64 穩定版發布

選擇與你的架構相符的安裝檔，然後在本機執行：

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# 或
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Linux 使用者

請從 [Releases](https://github.com/xjeway/mcp-manager/releases/latest) 頁面下載最新的 Linux 版本：

Linux 安裝檔名稱範例：

- `mcp-manager_<version>_<arch>.deb`（Debian/Ubuntu）
- `mcp-manager-<version>.<arch>.rpm`（Fedora/RHEL/openSUSE）
- `MCP-Manager-<version>-<arch>.AppImage`（通用）

發行版詳細說明：

- `.deb`：Debian、Ubuntu、Linux Mint、Pop!_OS、elementary OS 及其他基於 Debian 的發行版
- `.rpm`：Fedora、RHEL、Rocky Linux、AlmaLinux、openSUSE 及其他基於 RPM 的發行版
- `.AppImage`：適用於桌面 Linux 的可攜式選項，適合不想安裝系統套件的情況

架構說明：

- `x86_64` 或 `amd64`：適用於大多數 Intel 與 AMD 64 位元電腦
- `arm64` 或 `aarch64`：適用於 ARM64 Linux 裝置（需該版本有發布對應安裝檔）

使用對應的指令安裝：

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### 從原始碼建置

安裝相依套件：

```bash
make install
```

執行桌面應用程式：

```bash
make tauri-dev
```

僅執行 Web 介面：

```bash
make dev
```

## 快速開始

1. 啟動 `MCP Manager`。
2. 從本機用戶端設定檔匯入既有項目。
3. 以表單模式或 JSON 模式編輯伺服器。
4. 將產生的設定套用到一個或多個支援的用戶端。
5. 檢視警告後，在備份與還原保護下寫入。

## 支援的用戶端

以下每個用戶端都同時支援**匯入**（從其設定檔讀取既有的 MCP 伺服器）與**套用**（將統一設定寫回）。

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="../../docs/assets/clients/claudecode.svg" alt="Claude Code logo" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="../../docs/assets/clients/claude.svg" alt="Claude Desktop logo" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="../../docs/assets/clients/codex.svg" alt="Codex logo" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="../../docs/assets/clients/cursor.svg" alt="Cursor logo" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="../../docs/assets/clients/opencode.svg" alt="OpenCode logo" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="../../docs/assets/clients/githubcopilot.svg" alt="GitHub Copilot logo" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="../../docs/assets/clients/gemini.svg" alt="Gemini CLI logo" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="../../docs/assets/clients/antigravity.svg" alt="Antigravity logo" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="../../docs/assets/clients/iflow.svg" alt="iFlow logo" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="../../docs/assets/clients/qwen.svg" alt="Qwen Code logo" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="../../docs/assets/clients/cline.svg" alt="Cline logo" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="../../docs/assets/clients/windsurf.svg" alt="Windsurf logo" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../docs/assets/clients/kiro.svg" alt="Kiro logo" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../docs/assets/clients/qoder.svg" alt="Qoder logo" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../docs/assets/clients/vscode.svg" alt="VS Code logo" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## 命令列（預覽）

命令列工具 `mcpmgr` 與桌面應用程式共用同一份伺服器清單，在任一端所做的變更都會同步顯示在另一端。可以透過 `npx` 執行，或使用 npm 或 Homebrew 安裝：

```bash
npx @mcpmgr/cli list
npm install -g @mcpmgr/cli
brew install xjeway/mcp-manager/mcpmgr
```

```bash
mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr add --from mcp.json --project
mcpmgr list
mcpmgr remove context7 -a codex
mcpmgr rollback
```

在原始碼目錄中，可使用 `cargo run --bin mcpmgr -- <command>`。

在終端機中，`add` 會引導你選擇伺服器、用戶端，以及使用者或專案範圍，並在寫入前顯示每個用戶端檔案將寫入的內容。每個步驟都可以用旗標直接指定（`--server`、`--env`、`--app`、`--global`/`--project`、`--yes`）；`--dry-run` 用於預覽，`--json` 則輸出機器可讀的結果。在 Claude Code 或 Codex 等 AI 程式設計代理中執行時，它會略過互動提示，直接以該代理的用戶端為目標。

## 運作方式

- 應用程式讀取本機用戶端設定，並將其轉換為內部模型
- 套用時以原子更新、備份與還原支援寫入各用戶端專屬的輸出
- 線上市集會將 registry 項目（npm、PyPI、Docker 映像檔或遠端 URL）轉換為伺服器草稿，供你在編輯器中檢視；所有用戶端預設皆為關閉

### 網路存取

MCP Manager 只會在檢查更新時，以及市集頁面開啟期間查詢 MCP registry 時連線。Registry 結果會被快取：市集會從快取開啟，並在超過 6 小時後於背景更新，離線時仍可使用最後一次取得的資料。在 **設定 → 市集來源** 中新增來源時，會向該來源發送一次檢查請求。在 **設定 → 線上市集** 中關閉市集，即可完全停止這些請求。

## 適用範圍

目前的範圍專注於設定管理。程序啟動、停止、日誌與健康檢查等執行階段生命週期管理，刻意不納入 v1 的範圍。

## 專案結構

```text
mcp-manager/
  src/                前端應用程式
  src-tauri/          tauri 應用程式（提供給前端的命令、線上市集）
  crates/
    mcp-manager-core/ 用戶端轉接器、解析、安全寫入（共用）
    mcp-manager-cli/  命令列工具（`mcpmgr`）
  public/             靜態資源與品牌素材
  docs/               版本說明與設計參考
  openspec/           變更與規格追蹤
```

### 核心模組（`crates/mcp-manager-core`）

- `platform`：依作業系統解析路徑與環境上下文
- `adapters`：各用戶端的匯入與套用邏輯
- `core`：標準設定模型與合併行為
- `parser`：YAML / JSON / TOML 解析與擷取
- `storage`：原子寫入、備份與還原
- `store`：共用的 `servers.yaml`，具備鎖定與變更偵測
- `workflow`：跨所有用戶端的匯入、預覽與套用
- `ops`：伺服器清單編輯與驗證，與前端邏輯一致

## 開發

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## 發布自動化

本儲存庫包含 GitHub Actions 工作流程，用於 CI，以及透過官方 Tauri release action 打包桌面版本。

請參閱 [`docs/releasing.md`](../../docs/releasing.md)，了解：

- 由標籤驅動的 GitHub Release 發布
- macOS、Windows 與 Linux 套件產生
- 更新程式簽署設定
- 選用的平台程式碼簽署
- 版本同步與 git 標籤輔助指令

## 參與貢獻

歡迎提交 Issue 與 Pull Request。

如果你打算貢獻較大的變更，請先開立 Issue 或討論，在實作前先釐清範圍與方向。

### 本機開發

```bash
make install
make tauri-dev
```

### 提交 PR 之前

```bash
make test
make check
```

修改面向使用者的專案文件時，請保持 [`README.md`](../../README.md)、[`README.zh-CN.md`](../../README.zh-CN.md) 與 [`docs/readme/`](./) 中的各語言翻譯同步。

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 免責聲明

本專案主要在 AI 協助下開發（AI 驅動開發）。程式碼、測試與文件大多由 AI 產生，並經人工維護者審閱，因此仍可能包含錯誤。

MCP Manager 會讀取並改寫其他工具的設定檔。雖然內建備份與還原功能，你仍應自行備份重要的用戶端設定，在套用前檢視變更，並自行承擔使用本軟體的風險。本軟體依 [MIT 授權條款](../../LICENSE)以「現狀」提供，不附帶任何形式的保證。

## 授權條款

MIT @ xJeway
