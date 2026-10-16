<p align="center">
  <img src="./src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

<p align="center">
  一个跨平台桌面应用，用于管理 Claude Code、Codex、Cursor、VS Code、OpenCode、Gemini CLI 等客户端的 MCP Server 配置。
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
  <sub><a href="./README.md">English</a> · <b>简体中文</b> · <a href="./docs/readme/README.zh-TW.md">繁體中文</a> · <a href="./docs/readme/README.ja.md">日本語</a> · <a href="./docs/readme/README.ko.md">한국어</a> · <a href="./docs/readme/README.de.md">Deutsch</a> · <a href="./docs/readme/README.fr.md">Français</a> · <a href="./docs/readme/README.es.md">Español</a> · <a href="./docs/readme/README.it.md">Italiano</a> · <a href="./docs/readme/README.pt-BR.md">Português</a> · <a href="./docs/readme/README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>下载 MCP Manager</ins></a></h3>

## 目录

- [为什么选择 MCP Manager](#为什么选择-mcp-manager)
- [界面截图](#界面截图)
- [下载与安装](#下载与安装)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#linux-用户) · [从源码构建](#从源码构建)
- [快速开始](#快速开始)
- [支持的客户端](#支持的客户端)
- [命令行（预览）](#命令行预览)
- [工作方式](#工作方式)
- [当前范围](#当前范围)
- [项目结构](#项目结构)
- [开发命令](#开发命令)
- [自动发版](#自动发版)
- [参与贡献](#参与贡献)
- [Star History](#star-history)
- [免责声明](#免责声明)
- [License](#license)

## 为什么选择 MCP Manager

`MCP Manager` 是一个跨平台桌面应用，适合不想手动维护多份 MCP 客户端配置的用户。

- 统一维护所有 MCP Server 的工作台
- 从本地客户端配置中导入已有条目
- 在 GitHub MCP Registry、官方 MCP Registry，或自行添加的任何兼容 MCP Registry API 的注册表中搜索 server，几步即可添加
- 支持表单模式和原始 JSON 模式编辑
- 向多个受支持客户端生成并应用配置
- 文件写入前展示风险提示
- apply 过程支持备份与回滚
- 支持浅色、深色、跟随系统主题
- 界面支持英文、简体中文、繁体中文、日语、韩语、德语、法语、西班牙语、意大利语、巴西葡萄牙语和俄语（默认跟随系统语言）

## 界面截图

![MCP Manager 中文界面（浅色主题）](./docs/page-cn.png)

![MCP Manager 中文界面（深色主题）](./docs/page-cn-dark.png)

## 下载与安装

### 系统要求

- 提供 macOS、Windows、Linux 的桌面安装包
- 从源码构建需要 Node.js 20+、npm 10+、Rust stable，以及当前平台所需的 Tauri 系统依赖

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### 手动下载

下载地址：

- <https://github.com/xjeway/mcp-manager/releases/latest>

macOS 安装包文件名示例：

- `MCP-Manager-<version>-<arch>.dmg`

Arch 选择说明：

- Apple Silicon Mac：选择 `arm64` 或 `aarch64` 资产
- Intel Mac：选择 `x64` 或 `x86_64` 资产

下载与你的 arch 匹配的 macOS 安装包后，可直接在本地打开：

```bash
open ~/Downloads/MCP-Manager*.dmg
```

当前公开发布的 macOS 安装包在 Apple 签名与公证接入完成前，可能仍会以未签名形式分发。如果 Gatekeeper 阻止打开 `MCP Manager.app`，请按下面的绕过流程处理：

1. 先将 `MCP Manager.app` 拖到 `/Applications`。
2. 在 Finder 中对应用执行一次“右键 -> 打开”。
3. 如果 macOS 仍提示无法打开、已损坏或来源不明，可移除 quarantine 标记后重试：

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### 手动下载

下载地址：

- <https://github.com/xjeway/mcp-manager/releases/latest>

Windows 安装包文件名示例：

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Arch 选择说明：

- Windows on ARM 设备：选择 `arm64` 安装包
- Intel / AMD PC：选择 `x64` 或 `x86_64` 安装包

安装包说明：

- `setup.exe`：同时提供 Windows x64 和 Windows arm64 版本
- `.msi`：在稳定版发布时提供 Windows x64 版本

选择与你的 arch 匹配的安装包后，在本地运行：

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# 或
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Linux 用户

请从 [Releases](https://github.com/xjeway/mcp-manager/releases/latest) 页面下载最新的 Linux 安装包：

Linux 安装包文件名示例：

- `mcp-manager_<version>_<arch>.deb`（Debian/Ubuntu）
- `mcp-manager-<version>.<arch>.rpm`（Fedora/RHEL/openSUSE）
- `MCP-Manager-<version>-<arch>.AppImage`（Universal）

展开说明：

- `.deb`：适用于 Debian、Ubuntu、Linux Mint、Pop!_OS、elementary OS 等 Debian 系发行版
- `.rpm`：适用于 Fedora、RHEL、Rocky Linux、AlmaLinux、openSUSE 等 RPM 系发行版
- `.AppImage`：适用于可移植、通用的桌面 Linux 版本

Arch 选择说明：

- `x86_64` 或 `amd64`：适用于大多数 Intel / AMD 64 位 PC
- `arm64` 或 `aarch64`：适用于 ARM64 Linux 设备，仅在该架构安装包已发布时选择

按对应命令安装：

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>.AppImage
./MCP-Manager-<version>.AppImage
```

### 从源码构建

安装依赖：

```bash
make install
```

启动桌面应用：

```bash
make tauri-dev
```

仅启动 Web UI：

```bash
make dev
```

## 快速开始

1. 启动 `MCP Manager`。
2. 从本地客户端配置文件导入已有条目。
3. 使用表单模式或 JSON 模式编辑 Server。
4. 将生成后的配置应用到一个或多个目标客户端。
5. 确认风险提示后执行写入，并在需要时回滚。

## 支持的客户端

以下所有客户端均支持**导入**（读取其配置中已有的 MCP Server）和**应用**（将统一配置写回）。

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="./docs/assets/clients/claudecode.svg" alt="Claude Code logo" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="./docs/assets/clients/claude.svg" alt="Claude Desktop logo" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="./docs/assets/clients/codex.svg" alt="Codex logo" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="./docs/assets/clients/cursor.svg" alt="Cursor logo" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="./docs/assets/clients/opencode.svg" alt="OpenCode logo" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="./docs/assets/clients/githubcopilot.svg" alt="GitHub Copilot logo" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="./docs/assets/clients/gemini.svg" alt="Gemini CLI logo" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="./docs/assets/clients/antigravity.svg" alt="Antigravity logo" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="./docs/assets/clients/iflow.svg" alt="iFlow logo" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="./docs/assets/clients/qwen.svg" alt="Qwen Code logo" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="./docs/assets/clients/cline.svg" alt="Cline logo" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="./docs/assets/clients/windsurf.svg" alt="Windsurf logo" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="./docs/assets/clients/kiro.svg" alt="Kiro logo" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="./docs/assets/clients/qoder.svg" alt="Qoder logo" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="./docs/assets/clients/vscode.svg" alt="VS Code logo" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## 命令行（预览）

命令行工具 `mcpmgr` 与桌面应用共用同一份 server 列表，在任一端的修改另一端都能看到。可以直接用 `npx` 运行，也可以通过 npm 或 Homebrew 安装：

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

在源码目录中可用 `cargo run --bin mcpmgr -- <命令>` 运行。

在终端中，`add` 会分步引导你选择 server、客户端和用户级或项目级配置，并在写入前列出每个客户端文件将发生的变化。每一步都可以用参数直接回答（`--server`、`--env`、`--app`、`--global`/`--project`、`--yes`）；`--dry-run` 只预览，`--json` 输出机器可读结果。在 Claude Code、Codex 等 AI 编程工具中运行时，会跳过提问并默认写入该工具的配置。

## 工作方式

- 应用会读取本地客户端配置并转换为内部模型
- apply 时会生成客户端对应配置，并通过原子写、备份与回滚保障安全
- 在线市场会把注册表条目（npm、PyPI、Docker 镜像或远程 URL）转换成 server 草稿，在编辑器中确认后保存；所有客户端默认关闭

### 联网说明

MCP Manager 只在检查更新，以及打开在线市场页面时查询 MCP 注册表才会联网。注册表结果会缓存：打开市场时先显示缓存，超过 6 小时后在后台更新，断网时仍可查看上次的数据。在 **设置 → 市场来源** 中添加来源时，会向该地址发一次检查请求。在 **设置 → 在线市场** 中关闭后，不会再发起任何市场请求。

## 当前范围

当前版本主要聚焦配置管理。运行时生命周期管理，例如进程启停、日志查看、健康检查等，不属于 v1 范围。

## 项目结构

```text
mcp-manager/
  src/                前端应用
  src-tauri/          Tauri 应用（对前端暴露的命令、在线市场）
  crates/
    mcp-manager-core/ 客户端适配、解析与安全写入（共用）
    mcp-manager-cli/  命令行工具（`mcpmgr`）
  public/             静态资源与品牌素材
  docs/               发布说明与设计参考
  openspec/           变更与规格记录
```

### 核心模块（`crates/mcp-manager-core`）

- `platform`：平台路径解析与运行上下文
- `adapters`：按客户端实现导入与 apply 逻辑
- `core`：统一配置模型与合并规则
- `parser`：YAML / JSON / TOML 解析与配置提取
- `storage`：原子写、备份与回滚
- `store`：共用的 `servers.yaml`，带文件锁与外部修改检测
- `workflow`：跨所有客户端的导入、预览与应用
- `ops`：server 列表的编辑与校验（与前端逻辑保持一致）

## 开发命令

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## 自动发版

仓库已补充基于 GitHub Actions 和 Tauri 官方 release action 的自动发版方案。

详见 [`docs/releasing.md`](./docs/releasing.md)，包含：

- 基于 tag 的 GitHub Release 自动发布
- macOS、Windows、Linux 安装包构建
- Tauri updater 签名配置
- 可选的平台代码签名说明
- 版本同步与 git tag 辅助命令

## 参与贡献

欢迎提交 Issue 和 Pull Request。

如果是比较大的改动，建议先开一个 issue 或 discussion，对范围和方向先达成一致，再开始实现。

### 本地开发

```bash
make install
make tauri-dev
```

### 提交 PR 前

```bash
make test
make check
```

修改面向用户的项目文档时，请同步维护 [`README.md`](./README.md)、[`README.zh-CN.md`](./README.zh-CN.md) 以及 [`docs/readme/`](./docs/readme/) 中的其他语言版本。

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 免责声明

本项目以 AI 驱动开发为主：代码、测试和文档大多由 AI 生成，并由维护者人工审阅，仍可能存在错误。

MCP Manager 会读取并改写其他工具的配置文件。应用内置了备份与回滚，但你仍应自行备份重要的客户端配置，应用变更前请仔细检查，并自行承担使用风险。本软件按“原样”依据 [MIT License](./LICENSE) 提供，不附带任何形式的担保。

## License

MIT @ xJeway
