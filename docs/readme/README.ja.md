<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [MCP Manager を選ぶ理由](#mcp-manager-を選ぶ理由)
- [スクリーンショット](#スクリーンショット)
- [ダウンロードとインストール](#ダウンロードとインストール)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#linux-ユーザー) · [ソースからビルド](#ソースからビルド)
- [クイックスタート](#クイックスタート)
- [対応クライアント](#対応クライアント)
- [コマンドライン (プレビュー)](#コマンドライン-プレビュー)
- [仕組み](#仕組み)
- [スコープ](#スコープ)
- [プロジェクト構成](#プロジェクト構成)
- [開発](#開発)
- [リリースの自動化](#リリースの自動化)
- [コントリビューション](#コントリビューション)
- [Star History](#star-history)
- [免責事項](#免責事項)
- [ライセンス](#ライセンス)

## MCP Manager を選ぶ理由

`MCP Manager` は、複数の MCP クライアント設定を手作業で編集したくない人のためのクロスプラットフォームのデスクトップアプリです。

- 設定済みのすべてのサーバーを 1 つの統合 MCP ワークスペースで管理
- ローカルのクライアント設定から既存のエントリをインポート
- GitHub MCP Registry、公式 MCP Registry、または MCP Registry API を実装した任意のレジストリを追加してサーバーを探し、数クリックで追加
- フォームモードまたは生の JSON モードでサーバーを編集
- 生成した設定を複数の対応クライアントに適用
- ファイルが変更される前に、リスクのある書き込みを確認
- 適用時にバックアップを保持し、ロールバックにも対応
- ライトモード、ダークモード、またはシステムテーマを利用可能
- UI は英語、簡体字中国語、繁体字中国語、日本語、韓国語、ドイツ語、フランス語、スペイン語、イタリア語、ブラジルポルトガル語、ロシア語に対応 (デフォルトではシステムの言語に従います)

## スクリーンショット

![MCP Manager の英語インターフェース (ライトテーマ)](../../docs/page-en.png)

![MCP Manager の英語インターフェース (ダークテーマ)](../../docs/page-en-dark.png)

## ダウンロードとインストール

### システム要件

- デスクトップ版パッケージは macOS、Windows、Linux 向けに公開されています
- ソースからビルドするには Node.js 20 以上、npm 10 以上、Rust stable、およびお使いのプラットフォーム向けの Tauri システム依存関係が必要です

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### 手動ダウンロード

ダウンロード先:

- <https://github.com/xjeway/mcp-manager/releases/latest>

macOS 向けアセット名の例:

- `MCP-Manager-<version>-<arch>.dmg`

アーキテクチャの選び方:

- Apple Silicon Mac: `arm64` または `aarch64` のアセットを選択
- Intel Mac: `x64` または `x86_64` のアセットを選択

適切な macOS 向けアセットをダウンロードしたら、ローカルで開きます:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Apple の署名と公証を準備中のため、現在公開されている macOS リリースはまだ署名されていない場合があります。Gatekeeper によって `MCP Manager.app` がブロックされた場合は、次の代替手順を使用してください:

1. `MCP Manager.app` を `/Applications` にドラッグします。
2. Finder でアプリを一度右クリックし、`Open` を選択します。
3. それでも macOS がアプリを開けない、または破損していると表示する場合は、隔離フラグを削除してから再試行します:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### 手動ダウンロード

ダウンロード先:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Windows 向けアセット名の例:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

アーキテクチャの選び方:

- ARM 版 Windows デバイス: `arm64` のインストーラーアセットを選択
- Intel / AMD PC: `x64` または `x86_64` のインストーラーアセットを選択

パッケージの選び方:

- `setup.exe`: Windows x64 と Windows arm64 の両方で利用可能
- `.msi`: Windows x64 の安定版リリース向けに公開

お使いのアーキテクチャに合ったアセットを選び、ローカルで実行します:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# または
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Linux ユーザー

最新の Linux ビルドは [Releases](https://github.com/xjeway/mcp-manager/releases/latest) ページからダウンロードしてください:

Linux 向けアセット名の例:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (汎用)

ディストリビューション別の詳しいガイド:

- `.deb`: Debian、Ubuntu、Linux Mint、Pop!_OS、elementary OS、およびその他の Debian 系ディストリビューション
- `.rpm`: Fedora、RHEL、Rocky Linux、AlmaLinux、openSUSE、およびその他の RPM 系ディストリビューション
- `.AppImage`: システムパッケージを使いたくない場合の、デスクトップ Linux 向けポータブル版

アーキテクチャの選び方:

- ほとんどの Intel / AMD 64 ビット PC では `x86_64` または `amd64`
- ARM64 Linux デバイスでは `arm64` または `aarch64` (リリースでそのアセットが公開されている場合)

対応するコマンドでインストールします:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### ソースからビルド

依存関係をインストールします:

```bash
make install
```

デスクトップアプリを起動します:

```bash
make tauri-dev
```

Web UI のみを起動します:

```bash
make dev
```

## クイックスタート

1. `MCP Manager` を起動します。
2. ローカルのクライアント設定ファイルから既存のエントリをインポートします。
3. フォームモードまたは JSON モードでサーバーを編集します。
4. 生成された設定を 1 つ以上の対応クライアントに適用します。
5. 警告を確認し、バックアップとロールバックによる保護のもとで書き込みます。

## 対応クライアント

以下のすべてのクライアントが、**インポート** (設定ファイルから既存の MCP サーバーを読み込む) と **適用** (統合された設定を書き戻す) の両方に対応しています。

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

## コマンドライン (プレビュー)

コマンドラインツール `mcpmgr` はデスクトップアプリとサーバーリストを共有しているため、どちらで行った変更ももう一方に反映されます。`npx` で実行するか、npm または Homebrew でインストールしてください:

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

リポジトリをチェックアウトしている場合は `cargo run --bin mcpmgr -- <command>` を使用します。

ターミナルでは、`add` がサーバー、クライアント、ユーザーまたはプロジェクトのスコープの選択を順に案内し、書き込む前に各クライアントのファイルに何が書き込まれるかを表示します。すべてのステップはフラグ (`--server`、`--env`、`--app`、`--global`/`--project`、`--yes`) で指定することもできます。`--dry-run` でプレビューし、`--json` で機械可読な出力を表示します。Claude Code や Codex などの AI コーディングエージェント内で実行した場合は、プロンプトを省略し、そのエージェントのクライアントを対象にします。

## 仕組み

- アプリはローカルのクライアント設定を読み込み、内部モデルに変換します
- 適用時は、アトミックな更新、バックアップ、ロールバックに対応したうえで、クライアントごとの出力を書き込みます
- オンラインマーケットプレイスは、レジストリのエントリ (npm、PyPI、Docker イメージ、またはリモート URL) を、エディターで確認できるサーバーの下書きに変換します。すべてのクライアントは無効の状態から始まります

### ネットワークアクセス

MCP Manager がオンラインにアクセスするのは、アップデートの確認時と、マーケットプレイスのページを開いている間に MCP レジストリを照会するときだけです。レジストリの結果はキャッシュされます。マーケットプレイスはキャッシュから開き、6 時間経過後にバックグラウンドで更新します。オフラインでも最後に取得したデータで動作します。**設定 → マーケットプレイスのソース** でソースを追加すると、そのソースに確認リクエストが 1 回送信されます。**設定 → オンラインマーケットプレイス** でマーケットプレイスをオフにすると、これらのリクエストは一切送信されなくなります。

## スコープ

現在のスコープは設定管理に重点を置いています。プロセスの起動、停止、ログ、ヘルスチェックといったランタイムのライフサイクル管理は、v1 では意図的に対象外としています。

## プロジェクト構成

```text
mcp-manager/
  src/                フロントエンドアプリケーション
  src-tauri/          Tauri アプリ (フロントエンドに公開するコマンド、オンラインマーケットプレイス)
  crates/
    mcp-manager-core/ クライアントアダプター、パース、安全な書き込み (共有)
    mcp-manager-cli/  コマンドラインツール (`mcpmgr`)
  public/             静的アセットとブランディング
  docs/               リリースノートと設計資料
  openspec/           変更と仕様の追跡
```

### コアモジュール (`crates/mcp-manager-core`)

- `platform`: OS を考慮したパス解決と環境コンテキスト
- `adapters`: クライアントごとのインポートと適用のロジック
- `core`: 正規の設定モデルとマージ動作
- `parser`: YAML / JSON / TOML のパースと抽出
- `storage`: アトミックな書き込み、バックアップ、ロールバック
- `store`: 共有の `servers.yaml` (ロックと変更検出付き)
- `workflow`: すべてのクライアントにわたるインポート、プレビュー、適用
- `ops`: サーバーリストの編集と検証 (フロントエンドと同じ処理)

## 開発

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## リリースの自動化

このリポジトリには、CI と、公式の Tauri リリースアクションを使ったデスクトップ版リリースのパッケージングのための GitHub Actions ワークフローが含まれています。

以下については [`docs/releasing.md`](../../docs/releasing.md) を参照してください:

- タグをトリガーとする GitHub Release の公開
- macOS、Windows、Linux 向けパッケージの生成
- アップデーターの署名設定
- 任意のプラットフォーム向けコード署名
- バージョン同期と Git タグ用のヘルパーコマンド

## コントリビューション

Issue やプルリクエストを歓迎します。

大きな変更を提案する予定の場合は、実装の前にスコープと方向性を明確にするため、まず Issue または Discussion を作成してください。

### ローカル開発

```bash
make install
make tauri-dev
```

### PR を作成する前に

```bash
make test
make check
```

ユーザー向けのプロジェクトドキュメントを変更する際は、[`README.md`](../../README.md)、[`README.zh-CN.md`](../../README.zh-CN.md)、および [`docs/readme/`](./) 内の翻訳を同期させてください。

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 免責事項

このプロジェクトは主に AI の支援 (AI 駆動開発) によって開発されています。コード、テスト、ドキュメントの大部分は AI によって生成され、人間のメンテナーがレビューしていますが、誤りが含まれている可能性があります。

MCP Manager は他のツールの設定ファイルを読み込み、書き換えます。バックアップとロールバックの機能は組み込まれていますが、重要なクライアント設定はご自身でもバックアップし、適用前に変更内容を確認したうえで、自己責任でご使用ください。本ソフトウェアは [MIT License](../../LICENSE) のもと「現状のまま」提供され、いかなる保証もありません。

## ライセンス

MIT @ xJeway
