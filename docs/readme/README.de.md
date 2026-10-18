<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Eine plattformübergreifende Desktop-App zur Verwaltung von MCP-Server-Konfigurationen für Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI und weitere.
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
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <b>Deutsch</b> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>MCP Manager herunterladen</ins></a></h3>

## Inhaltsverzeichnis

- [Funktionen](#funktionen)
- [Installation](#installation)
- [Unterstützte Clients](#unterstützte-clients)
- [Kommandozeile (Vorschau)](#kommandozeile-vorschau)
- [Datenschutz & Netzwerk](#datenschutz--netzwerk)
- [Entwicklung](#entwicklung)
- [Star History](#star-history)
- [Haftungsausschluss](#haftungsausschluss)

## Funktionen

- **Ein Arbeitsbereich** für alle MCP-Server, importiert aus den Konfigurationen, die Ihre Clients bereits haben
- **Online-Marktplatz**: Server in der GitHub MCP Registry, der offiziellen MCP Registry oder einer beliebigen kompatiblen Registry finden und mit wenigen Klicks hinzufügen
- Bearbeitung per **Formular oder Roh-JSON**
- **Auf viele Clients gleichzeitig anwenden**, mit Prüfung riskanter Schreibvorgänge, automatischen Backups und Rollback
- Helles / dunkles Design und eine Oberfläche in 11 Sprachen, die der Systemsprache folgt

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, helles Design" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, dunkles Design" /></td>
  </tr>
</table>

## Installation

Laden Sie das Paket für Ihre Plattform aus dem [neuesten Release](https://github.com/xjeway/mcp-manager/releases/latest) herunter:

| Plattform | Paket |
| --- | --- |
| macOS | `.dmg` oder `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) oder `.msi` (x64) |
| Linux | `.deb`, `.rpm` oder `.AppImage` |

Wählen Sie `arm64` / `aarch64` für Apple Silicon und ARM-Geräte, sonst `x64` / `x86_64` / `amd64`.

<details>
<summary>macOS meldet, dass die App beschädigt ist oder nicht geöffnet werden kann?</summary>

Release-Builds sind ad-hoc signiert, aber noch nicht von Apple notarisiert. Beim ersten Start:

1. Verschieben Sie die App nach `/Applications`, öffnen Sie sie und schließen Sie die Warnung.
2. Öffnen Sie **Systemeinstellungen → Datenschutz & Sicherheit**, klicken Sie unten auf **Dennoch öffnen** und bestätigen Sie.

Wenn macOS weiterhin meldet, dass die App beschädigt ist (ältere, unsignierte Versionen), führen Sie aus:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Unterstützte Clients

Jeder der folgenden Clients unterstützt sowohl **Import** (vorhandene MCP-Server aus seiner Konfiguration lesen) als auch **Anwenden** (die einheitliche Konfiguration zurückschreiben).

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
  <a href="https://windsurf.com"><kbd><img src="../../public/logos/apps/devin.svg" alt="Devin Desktop logo" width="16" valign="middle" /> Devin Desktop</kbd></a> &nbsp;
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
  <a href="https://code.visualstudio.com"><kbd><img src="../../public/logos/apps/vscode.svg" alt="VS Code logo" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## Kommandozeile (Vorschau)

`mcpmgr` teilt sich die Serverliste mit der Desktop-App, sodass Änderungen in einem der beiden auch im anderen erscheinen.

```bash
npx @mcpmgr/cli list                      # oder: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

Im Terminal führt `add` Sie durch Server, Clients und Geltungsbereich und zeigt vor dem Schreiben eine Vorschau jeder Datei. Jeder Schritt hat auch ein Flag (`--yes`, `--dry-run`, `--json`, …); innerhalb eines KI-Coding-Agenten wie Claude Code oder Codex überspringt es die Abfragen und zielt auf den Client dieses Agenten.

## Datenschutz & Netzwerk

MCP Manager geht nur online, um nach Updates zu suchen und, solange die Marktplatz-Seite geöffnet ist, um MCP-Registries abzufragen (6 Stunden zwischengespeichert, offline nutzbar). Deaktivieren Sie den Marktplatz unter **Einstellungen → Online-Marktplatz**, um diese Anfragen vollständig zu unterbinden.

## Entwicklung

Erfordert Node.js 20+, Rust stable und die [Tauri-Voraussetzungen](https://tauri.app/start/prerequisites/) für Ihre Plattform.

```bash
make install      # Abhängigkeiten installieren
make tauri-dev    # Desktop-App starten
make test && make check
```

Die Kernlogik (Client-Adapter, Parsing, sichere Schreibvorgänge) liegt in `crates/mcp-manager-core` und wird von der Tauri-App in `src-tauri/` und der CLI in `crates/mcp-manager-cli/` gemeinsam genutzt. Releases sind in [`docs/releasing.md`](../../docs/releasing.md) beschrieben.

Issues und Pull Requests sind willkommen; bei größeren Änderungen eröffnen Sie bitte zuerst ein Issue. Halten Sie beim Ändern nutzerseitiger Dokumentation [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) und [`docs/readme/`](./) synchron.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Haftungsausschluss

Dieses Projekt wird überwiegend mit KI-Unterstützung entwickelt und von einem menschlichen Maintainer geprüft, kann daher aber dennoch Fehler enthalten. MCP Manager überschreibt die Konfigurationsdateien anderer Tools; Backups und Rollback sind integriert, dennoch sollten Sie wichtige Konfigurationen selbst sichern. Bereitgestellt „wie besehen“ unter der [MIT License](../../LICENSE).
