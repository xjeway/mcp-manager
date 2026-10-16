<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

<p align="center">
  Кроссплатформенное настольное приложение для управления конфигурациями MCP-серверов в Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI и других клиентах.
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
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <b>Русский</b></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>Скачать MCP Manager</ins></a></h3>

## Содержание

- [Возможности](#возможности)
- [Установка](#установка)
- [Поддерживаемые клиенты](#поддерживаемые-клиенты)
- [Командная строка (предварительная версия)](#командная-строка-предварительная-версия)
- [Конфиденциальность и сеть](#конфиденциальность-и-сеть)
- [Разработка](#разработка)
- [Star History](#star-history)
- [Отказ от ответственности](#отказ-от-ответственности)

## Возможности

- **Единое пространство** для всех MCP-серверов, импортированных из конфигураций, которые уже есть у ваших клиентов
- **Онлайн-каталог**: ищите серверы в GitHub MCP Registry, официальном MCP Registry или любом совместимом реестре и добавляйте их в несколько кликов
- Редактирование **в форме или в сыром JSON**
- **Применение сразу к нескольким клиентам** с проверкой рискованных записей, автоматическим резервным копированием и откатом
- Светлая / тёмная тема и интерфейс на 11 языках, который следует языку системы

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, светлая тема" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, тёмная тема" /></td>
  </tr>
</table>

## Установка

Скачайте пакет для своей платформы из [последнего релиза](https://github.com/xjeway/mcp-manager/releases/latest):

| Платформа | Пакет |
| --- | --- |
| macOS | `.dmg` или `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) или `.msi` (x64) |
| Linux | `.deb`, `.rpm` или `.AppImage` |

Выбирайте `arm64` / `aarch64` для Apple Silicon и устройств на ARM, в остальных случаях — `x64` / `x86_64` / `amd64`.

<details>
<summary>macOS сообщает, что приложение повреждено или не может быть открыто?</summary>

Релизные сборки пока не нотаризованы. Переместите приложение в `/Applications`, затем выполните:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Поддерживаемые клиенты

Каждый клиент ниже поддерживает и **импорт** (чтение существующих MCP-серверов из его конфигурации), и **применение** (запись единой конфигурации обратно).

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

## Командная строка (предварительная версия)

`mcpmgr` использует тот же список серверов, что и настольное приложение, поэтому изменения, сделанные в одном, видны в другом.

```bash
npx @mcpmgr/cli list                      # или: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

В терминале `add` пошагово проводит вас через выбор серверов, клиентов и области, а перед записью показывает предварительный просмотр каждого файла. Для каждого шага есть и флаг (`--yes`, `--dry-run`, `--json`, …); внутри ИИ-агента для программирования, такого как Claude Code или Codex, вопросы пропускаются, а целью становится клиент этого агента.

## Конфиденциальность и сеть

MCP Manager выходит в сеть только для проверки обновлений и, пока открыта страница каталога, для запросов к MCP-реестрам (результаты кэшируются на 6 часов, работает и без сети). Отключите каталог в **Настройки → Онлайн-каталог**, чтобы полностью прекратить эти запросы.

## Разработка

Требуются Node.js 20+, Rust stable и [системные зависимости Tauri](https://tauri.app/start/prerequisites/) для вашей платформы.

```bash
make install      # установить зависимости
make tauri-dev    # запустить настольное приложение
make test && make check
```

Основная логика (адаптеры клиентов, парсинг, безопасная запись) находится в `crates/mcp-manager-core` и используется как приложением Tauri в `src-tauri/`, так и CLI в `crates/mcp-manager-cli/`. Процесс выпуска релизов описан в [`docs/releasing.md`](../../docs/releasing.md).

Issues и pull requests приветствуются; для крупных изменений сначала откройте issue. При изменении пользовательской документации поддерживайте [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) и [`docs/readme/`](./) в согласованном состоянии.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Отказ от ответственности

Этот проект разрабатывается преимущественно с помощью ИИ и проверяется сопровождающим-человеком, поэтому в нём всё ещё могут быть ошибки. MCP Manager перезаписывает конфигурационные файлы других инструментов; резервное копирование и откат встроены, но всё равно самостоятельно сохраняйте копии важных конфигураций. Предоставляется «как есть» по [лицензии MIT](../../LICENSE).
