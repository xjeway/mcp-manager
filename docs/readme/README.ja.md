<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Claude Code、Codex、Cursor、VS Code、OpenCode、Gemini CLI などにまたがる MCP サーバー設定を管理するための、クロスプラットフォームのデスクトップアプリです。
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
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <b>日本語</b> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>MCP Manager をダウンロード</ins></a></h3>

## 目次

- [機能](#機能)
- [インストール](#インストール)
- [対応クライアント](#対応クライアント)
- [コマンドライン (プレビュー)](#コマンドライン-プレビュー)
- [プライバシーとネットワーク](#プライバシーとネットワーク)
- [開発](#開発)
- [Star History](#star-history)
- [免責事項](#免責事項)

## 機能

- クライアントにすでにある設定からインポートした、すべての MCP サーバーを管理する **1 つのワークスペース**
- **オンラインマーケットプレイス**: GitHub MCP Registry、公式 MCP Registry、または互換性のある任意のレジストリでサーバーを探し、数クリックで追加
- **フォームまたは生の JSON** で編集
- **複数のクライアントに一括適用**。リスクのある書き込みの確認、自動バックアップ、ロールバックに対応
- ライト / ダークテーマ、およびシステムの言語に従う 11 言語対応の UI

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager (ライトテーマ)" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager (ダークテーマ)" /></td>
  </tr>
</table>

## インストール

[最新リリース](https://github.com/xjeway/mcp-manager/releases/latest) から、お使いのプラットフォーム向けのパッケージをダウンロードしてください:

| プラットフォーム | パッケージ |
| --- | --- |
| macOS | `.dmg`、または `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) または `.msi` (x64) |
| Linux | `.deb`、`.rpm`、または `.AppImage` |

Apple Silicon および ARM デバイスでは `arm64` / `aarch64` を、それ以外では `x64` / `x86_64` / `amd64` を選択してください。

<details>
<summary>macOS でアプリが破損している、または開けないと表示される場合</summary>

リリースビルドは ad-hoc 署名されていますが、Apple の公証はまだ受けていません。初回起動時:

1. アプリを `/Applications` に移動して開き、警告を閉じます。
2. **システム設定 → プライバシーとセキュリティ** を開き、下部の **このまま開く** をクリックして確認します。

それでも「破損している」と表示される場合（署名のない古いバージョン）は、次を実行してください:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## 対応クライアント

以下のすべてのクライアントが、**インポート** (設定ファイルから既存の MCP サーバーを読み込む) と **適用** (統合された設定を書き戻す) の両方に対応しています。

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
  <a href="https://devin.ai"><kbd><img src="../../public/logos/apps/devin.svg" alt="Devin logo" width="16" valign="middle" /> Devin (Desktop / CLI)</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../public/logos/apps/kiro.svg" alt="Kiro logo" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../public/logos/apps/qoder.svg" alt="Qoder logo" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://kilo.ai"><kbd><img src="../../public/logos/apps/kilocode.svg" alt="Kilo Code logo" width="16" valign="middle" /> Kilo Code</kbd></a> &nbsp;
  <a href="https://factory.ai"><kbd><img src="../../public/logos/apps/factorydroid.svg" alt="Factory Droid logo" width="16" valign="middle" /> Factory Droid</kbd></a> &nbsp;
  <a href="https://github.com/MoonshotAI/kimi-cli"><kbd><img src="../../public/logos/apps/kimi.svg" alt="Kimi CLI logo" width="16" valign="middle" /> Kimi CLI</kbd></a> &nbsp;
  <a href="https://zed.dev"><kbd><img src="../../public/logos/apps/zed.svg" alt="Zed logo" width="16" valign="middle" /> Zed</kbd></a> &nbsp;
  <a href="https://github.com/mistralai/mistral-vibe"><kbd><img src="../../public/logos/apps/mistral.svg" alt="Mistral Vibe logo" width="16" valign="middle" /> Mistral Vibe</kbd></a> &nbsp;
  <a href="https://github.com/NousResearch/hermes-agent"><kbd><img src="../../public/logos/apps/hermes.svg" alt="Hermes Agent logo" width="16" valign="middle" /> Hermes Agent</kbd></a> &nbsp;
  <a href="https://github.com/block/goose"><kbd><img src="../../public/logos/apps/goose.svg" alt="Goose logo" width="16" valign="middle" /> Goose</kbd></a> &nbsp;
  <a href="https://ampcode.com"><kbd><img src="../../public/logos/apps/amp.svg" alt="Amp logo" width="16" valign="middle" /> Amp</kbd></a> &nbsp;
  <a href="https://x.ai/cli"><kbd><img src="../../public/logos/apps/grok.svg" alt="Grok Build logo" width="16" valign="middle" /> Grok Build</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../public/logos/apps/vscode.svg" alt="VS Code logo" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## コマンドライン (プレビュー)

`mcpmgr` はデスクトップアプリとサーバーリストを共有しているため、どちらで行った変更ももう一方に反映されます。

```bash
npx @mcpmgr/cli list                      # または: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

ターミナルでは、`add` がサーバー、クライアント、スコープの選択を順に案内し、書き込む前にすべてのファイルをプレビューします。各ステップはフラグ (`--yes`、`--dry-run`、`--json` など) でも指定できます。Claude Code や Codex などの AI コーディングエージェント内で実行した場合は、プロンプトを省略し、そのエージェントのクライアントを対象にします。

## プライバシーとネットワーク

MCP Manager がオンラインにアクセスするのは、アップデートの確認時と、マーケットプレイスのページを開いている間に MCP レジストリを照会するとき (6 時間キャッシュされ、オフラインでも利用可能) だけです。**設定 → オンラインマーケットプレイス** でマーケットプレイスをオフにすると、これらのリクエストは一切送信されなくなります。

## 開発

Node.js 20 以上、Rust stable、およびお使いのプラットフォーム向けの [Tauri の前提条件](https://tauri.app/start/prerequisites/) が必要です。

```bash
make install      # 依存関係をインストール
make tauri-dev    # デスクトップアプリを起動
make test && make check
```

コアロジック (クライアントアダプター、パース、安全な書き込み) は `crates/mcp-manager-core` にあり、`src-tauri/` の Tauri アプリと `crates/mcp-manager-cli/` の CLI で共有されています。リリースについては [`docs/releasing.md`](../../docs/releasing.md) を参照してください。

Issue やプルリクエストを歓迎します。大きな変更の場合は、まず Issue を作成してください。ユーザー向けのドキュメントを変更する際は、[`README.md`](../../README.md)、[`README.zh-CN.md`](../../README.zh-CN.md)、および [`docs/readme/`](./) を同期させてください。

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 免責事項

このプロジェクトは主に AI の支援によって開発され、人間のメンテナーがレビューしていますが、誤りが含まれている可能性があります。MCP Manager は他のツールの設定ファイルを書き換えます。バックアップとロールバックの機能は組み込まれていますが、重要な設定はご自身でもバックアップしてください。本ソフトウェアは [MIT License](../../LICENSE) のもと「現状のまま」提供されます。
