<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [Warum MCP Manager](#warum-mcp-manager)
- [Screenshots](#screenshots)
- [Download und Installation](#download-und-installation)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#linux-nutzer) · [Aus dem Quellcode bauen](#aus-dem-quellcode-bauen)
- [Schnellstart](#schnellstart)
- [Unterstützte Clients](#unterstützte-clients)
- [Kommandozeile (Vorschau)](#kommandozeile-vorschau)
- [Funktionsweise](#funktionsweise)
- [Umfang](#umfang)
- [Projektstruktur](#projektstruktur)
- [Entwicklung](#entwicklung)
- [Release-Automatisierung](#release-automatisierung)
- [Mitwirken](#mitwirken)
- [Star History](#star-history)
- [Haftungsausschluss](#haftungsausschluss)
- [Lizenz](#lizenz)

## Warum MCP Manager

`MCP Manager` ist eine plattformübergreifende Desktop-App für alle, die mehrere MCP-Client-Konfigurationen nicht von Hand bearbeiten möchten.

- Einen einheitlichen MCP-Arbeitsbereich für alle konfigurierten Server pflegen
- Vorhandene Einträge aus der lokalen Client-Konfiguration importieren
- Server in der GitHub MCP Registry, der offiziellen MCP Registry oder einer beliebigen selbst hinzugefügten Registry finden, die die MCP Registry API implementiert, und mit wenigen Klicks hinzufügen
- Server im Formularmodus oder im Roh-JSON-Modus bearbeiten
- Die generierte Konfiguration auf mehrere unterstützte Clients anwenden
- Riskante Schreibvorgänge prüfen, bevor Dateien geändert werden
- Backups und Rollback-Unterstützung beim Anwenden nutzen
- Hellen Modus, dunklen Modus oder das Systemdesign verwenden
- Die Oberfläche auf Englisch, vereinfachtem Chinesisch, traditionellem Chinesisch, Japanisch, Koreanisch, Deutsch, Französisch, Spanisch, Italienisch, brasilianischem Portugiesisch oder Russisch nutzen (standardmäßig wird die Systemsprache verwendet)

## Screenshots

![MCP Manager-Oberfläche auf Englisch, helles Design](../../docs/page-en.png)

![MCP Manager-Oberfläche auf Englisch, dunkles Design](../../docs/page-en-dark.png)

## Download und Installation

### Systemanforderungen

- Desktop-Pakete werden für macOS, Windows und Linux veröffentlicht
- Zum Bauen aus dem Quellcode werden Node.js 20+, npm 10+, Rust stable und die Tauri-Systemabhängigkeiten für Ihre Plattform benötigt

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### Manueller Download

Download unter:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Beispiele für macOS-Dateinamen:

- `MCP-Manager-<version>-<arch>.dmg`

Architektur-Leitfaden:

- Macs mit Apple Silicon: `arm64`- oder `aarch64`-Dateien wählen
- Intel-Macs: `x64`- oder `x86_64`-Dateien wählen

Öffnen Sie die passende macOS-Datei nach dem Herunterladen lokal:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Aktuelle öffentliche macOS-Releases sind möglicherweise noch unsigniert, während Apple-Signierung und Notarisierung vorbereitet werden. Falls Gatekeeper `MCP Manager.app` blockiert, gehen Sie wie folgt vor:

1. Ziehen Sie `MCP Manager.app` nach `/Applications`.
2. Klicken Sie im Finder einmal mit der rechten Maustaste auf die App und wählen Sie `Open`.
3. Falls macOS weiterhin meldet, dass die App nicht geöffnet werden kann oder beschädigt ist, entfernen Sie das Quarantäne-Attribut und versuchen Sie es erneut:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### Manueller Download

Download unter:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Beispiele für Windows-Dateinamen:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Architektur-Leitfaden:

- Windows-Geräte mit ARM: die `arm64`-Installationsdatei wählen
- Intel-/AMD-PCs: die `x64`- oder `x86_64`-Installationsdatei wählen

Paket-Leitfaden:

- `setup.exe`: verfügbar für Windows x64 und Windows arm64
- `.msi`: wird für stabile Windows-x64-Releases veröffentlicht

Wählen Sie die Datei, die zu Ihrer Architektur passt, und führen Sie sie lokal aus:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# oder
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Linux-Nutzer

Laden Sie den neuesten Linux-Build von der Seite [Releases](https://github.com/xjeway/mcp-manager/releases/latest) herunter:

Beispiele für Linux-Dateinamen:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (universell)

Ausführlicher Distributions-Leitfaden:

- `.deb`: Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS und andere Debian-basierte Distributionen
- `.rpm`: Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE und andere RPM-basierte Distributionen
- `.AppImage`: portable Option für Desktop-Linux, wenn Sie kein Systempaket möchten

Architektur-Leitfaden:

- `x86_64` oder `amd64` für die meisten 64-Bit-PCs mit Intel oder AMD
- `arm64` oder `aarch64` für ARM64-Linux-Geräte, sofern diese Datei im Release veröffentlicht wird

Mit dem passenden Befehl installieren:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### Aus dem Quellcode bauen

Abhängigkeiten installieren:

```bash
make install
```

Desktop-App starten:

```bash
make tauri-dev
```

Nur die Web-Oberfläche starten:

```bash
make dev
```

## Schnellstart

1. Starten Sie `MCP Manager`.
2. Importieren Sie vorhandene Einträge aus lokalen Client-Konfigurationsdateien.
3. Bearbeiten Sie Server im Formularmodus oder im JSON-Modus.
4. Wenden Sie die generierte Konfiguration auf einen oder mehrere unterstützte Clients an.
5. Prüfen Sie die Warnungen und schreiben Sie dann mit Backup- und Rollback-Schutz.

## Unterstützte Clients

Jeder der folgenden Clients unterstützt sowohl **Import** (vorhandene MCP-Server aus seiner Konfiguration lesen) als auch **Anwenden** (die einheitliche Konfiguration zurückschreiben).

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

## Kommandozeile (Vorschau)

Ein Kommandozeilen-Tool, `mcpmgr`, teilt sich die Serverliste mit der Desktop-App, sodass Änderungen in einem der beiden auch im anderen erscheinen. Führen Sie es mit `npx` aus oder installieren Sie es mit npm oder Homebrew:

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

Aus einem ausgecheckten Repository verwenden Sie `cargo run --bin mcpmgr -- <command>`.

Im Terminal führt `add` Sie durch die Auswahl von Servern, Clients und dem Geltungsbereich (Benutzer oder Projekt) und zeigt vor dem Schreiben, was jede Client-Datei erhalten wird. Jeder Schritt lässt sich auch per Flag beantworten (`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`); `--dry-run` zeigt eine Vorschau, und `--json` gibt maschinenlesbare Ausgaben aus. Innerhalb eines KI-Coding-Agenten wie Claude Code oder Codex ausgeführt, überspringt es die Abfragen und zielt auf den Client dieses Agenten.

## Funktionsweise

- Die App liest die lokale Client-Konfiguration und wandelt sie in das interne Modell um
- Beim Anwenden wird clientspezifische Ausgabe mit atomaren Updates, Backup und Rollback-Unterstützung geschrieben
- Der Online-Marktplatz macht aus einem Registry-Eintrag (npm, PyPI, Docker-Image oder Remote-URL) einen Serverentwurf, den Sie im Editor prüfen; dabei sind zunächst alle Clients deaktiviert

### Netzwerkzugriff

MCP Manager geht nur online, um nach Updates zu suchen und, solange die Marktplatz-Seite geöffnet ist, um MCP-Registries abzufragen. Registry-Ergebnisse werden zwischengespeichert: Der Marktplatz öffnet sich aus dem Cache, aktualisiert ihn nach 6 Stunden im Hintergrund und funktioniert offline weiterhin mit den zuletzt abgerufenen Daten. Das Hinzufügen einer Quelle unter **Einstellungen → Marktplatz-Quellen** sendet eine einzelne Prüfanfrage an diese. Deaktivieren Sie den Marktplatz unter **Einstellungen → Online-Marktplatz**, um diese Anfragen vollständig zu unterbinden.

## Umfang

Der aktuelle Umfang konzentriert sich auf die Konfigurationsverwaltung. Laufzeit-Lebenszyklusverwaltung wie Starten und Stoppen von Prozessen, Logs und Health-Checks ist für v1 bewusst nicht vorgesehen.

## Projektstruktur

```text
mcp-manager/
  src/                Frontend-Anwendung
  src-tauri/          Tauri-App (Befehle für das Frontend, Online-Marktplatz)
  crates/
    mcp-manager-core/ Client-Adapter, Parsing, sichere Schreibvorgänge (gemeinsam genutzt)
    mcp-manager-cli/  Kommandozeilen-Tool (`mcpmgr`)
  public/             statische Assets und Branding
  docs/               Release-Notes und Design-Referenzen
  openspec/           Änderungs- und Spezifikationsverfolgung
```

### Kernmodule (`crates/mcp-manager-core`)

- `platform`: betriebssystemabhängige Pfadauflösung und Umgebungskontext
- `adapters`: Import- und Anwende-Logik pro Client
- `core`: kanonisches Konfigurationsmodell und Zusammenführungsverhalten
- `parser`: Parsen und Extrahieren von YAML / JSON / TOML
- `storage`: atomares Schreiben, Backup und Rollback
- `store`: die gemeinsame `servers.yaml` mit Sperren und Änderungserkennung
- `workflow`: Import, Vorschau und Anwenden über alle Clients hinweg
- `ops`: Bearbeitung und Validierung der Serverliste, gespiegelt vom Frontend

## Entwicklung

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## Release-Automatisierung

Dieses Repository enthält GitHub-Actions-Workflows für CI und das Paketieren von Desktop-Releases mit der offiziellen Tauri-Release-Action.

Siehe [`docs/releasing.md`](../../docs/releasing.md) für:

- tag-gesteuerte Veröffentlichung von GitHub Releases
- Paketerstellung für macOS, Windows und Linux
- Einrichtung der Updater-Signierung
- optionale Code-Signierung je Plattform
- Hilfsbefehle für Versionsabgleich und Git-Tags

## Mitwirken

Issues und Pull Requests sind willkommen.

Wenn Sie eine größere Änderung beisteuern möchten, eröffnen Sie bitte zuerst ein Issue oder eine Diskussion, damit Umfang und Richtung vor der Umsetzung geklärt sind.

### Lokale Entwicklung

```bash
make install
make tauri-dev
```

### Vor dem Öffnen eines PR

```bash
make test
make check
```

Halten Sie [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) und die Übersetzungen in [`docs/readme/`](./) synchron, wenn Sie nutzerseitige Projektdokumentation ändern.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Haftungsausschluss

Dieses Projekt wird überwiegend mit KI-Unterstützung entwickelt (KI-gesteuerte Entwicklung). Code, Tests und Dokumentation sind größtenteils KI-generiert und werden von einem menschlichen Maintainer geprüft, können aber dennoch Fehler enthalten.

MCP Manager liest und überschreibt die Konfigurationsdateien anderer Tools. Backups und Rollback sind integriert, dennoch sollten Sie wichtige Client-Konfigurationen selbst sichern, Änderungen vor dem Anwenden prüfen und die Software auf eigenes Risiko verwenden. Sie wird „wie besehen“ unter der [MIT License](../../LICENSE) bereitgestellt, ohne jegliche Gewährleistung.

## Lizenz

MIT @ xJeway
