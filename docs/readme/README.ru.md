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

- [Почему MCP Manager](#почему-mcp-manager)
- [Скриншоты](#скриншоты)
- [Загрузка и установка](#загрузка-и-установка)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#пользователи-linux) · [Сборка из исходного кода](#сборка-из-исходного-кода)
- [Быстрый старт](#быстрый-старт)
- [Поддерживаемые клиенты](#поддерживаемые-клиенты)
- [Командная строка (предварительная версия)](#командная-строка-предварительная-версия)
- [Как это работает](#как-это-работает)
- [Область применения](#область-применения)
- [Структура проекта](#структура-проекта)
- [Разработка](#разработка)
- [Автоматизация релизов](#автоматизация-релизов)
- [Участие в разработке](#участие-в-разработке)
- [Star History](#star-history)
- [Отказ от ответственности](#отказ-от-ответственности)
- [Лицензия](#лицензия)

## Почему MCP Manager

`MCP Manager` — кроссплатформенное настольное приложение для тех, кто не хочет вручную править конфигурации нескольких MCP-клиентов.

- Единое MCP-пространство для всех настроенных серверов
- Импорт существующих записей из локальных конфигураций клиентов
- Поиск серверов в GitHub MCP Registry, официальном MCP Registry или любом добавленном вами реестре, реализующем MCP Registry API, и добавление их в несколько кликов
- Редактирование серверов в режиме формы или в режиме сырого JSON
- Применение сгенерированной конфигурации к нескольким поддерживаемым клиентам
- Проверка рискованных записей до изменения файлов
- Резервные копии и возможность отката при применении
- Светлая, тёмная или системная тема
- Интерфейс на английском, упрощённом китайском, традиционном китайском, японском, корейском, немецком, французском, испанском, итальянском, бразильском португальском или русском языке (по умолчанию следует языку системы)

## Скриншоты

![Интерфейс MCP Manager на английском, светлая тема](../../docs/page-en.png)

![Интерфейс MCP Manager на английском, тёмная тема](../../docs/page-en-dark.png)

## Загрузка и установка

### Системные требования

- Настольные пакеты публикуются для macOS, Windows и Linux
- Для сборки из исходного кода нужны Node.js 20+, npm 10+, Rust stable и системные зависимости Tauri для вашей платформы

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### Ручная загрузка

Скачать можно здесь:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Пример имён файлов для macOS:

- `MCP-Manager-<version>-<arch>.dmg`

Выбор архитектуры:

- Mac на Apple Silicon: выбирайте файлы `arm64` или `aarch64`
- Mac на Intel: выбирайте файлы `x64` или `x86_64`

Скачав подходящий файл для macOS, откройте его локально:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Текущие публичные релизы для macOS пока могут быть не подписаны, пока идёт подготовка подписи и нотаризации Apple. Если Gatekeeper блокирует `MCP Manager.app`, воспользуйтесь обходным путём:

1. Перетащите `MCP Manager.app` в `/Applications`.
2. В Finder один раз щёлкните приложение правой кнопкой мыши и выберите `Open`.
3. Если macOS всё ещё сообщает, что приложение не может быть открыто или повреждено, снимите флаг карантина и попробуйте снова:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### Ручная загрузка

Скачать можно здесь:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Пример имён файлов для Windows:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Выбор архитектуры:

- Устройства Windows на ARM: выбирайте установщик `arm64`
- ПК на Intel / AMD: выбирайте установщик `x64` или `x86_64`

Выбор пакета:

- `setup.exe`: доступен как для Windows x64, так и для Windows arm64
- `.msi`: публикуется для стабильных релизов Windows x64

Выберите файл, соответствующий вашей архитектуре, и запустите его локально:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# или
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Пользователи Linux

Скачайте последнюю сборку для Linux на странице [Releases](https://github.com/xjeway/mcp-manager/releases/latest):

Пример имён файлов для Linux:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (универсальный)

Подробнее о дистрибутивах:

- `.deb`: Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS и другие дистрибутивы на базе Debian
- `.rpm`: Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE и другие дистрибутивы на базе RPM
- `.AppImage`: портативный вариант для настольного Linux, если вы не хотите ставить системный пакет

Выбор архитектуры:

- `x86_64` или `amd64` для большинства 64-битных ПК на Intel и AMD
- `arm64` или `aarch64` для устройств Linux на ARM64, если такой файл опубликован в релизе

Установите соответствующей командой:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### Сборка из исходного кода

Установите зависимости:

```bash
make install
```

Запустите настольное приложение:

```bash
make tauri-dev
```

Запустите только веб-интерфейс:

```bash
make dev
```

## Быстрый старт

1. Запустите `MCP Manager`.
2. Импортируйте существующие записи из локальных конфигурационных файлов клиентов.
3. Отредактируйте серверы в режиме формы или в режиме JSON.
4. Примените сгенерированную конфигурацию к одному или нескольким поддерживаемым клиентам.
5. Просмотрите предупреждения, затем запишите изменения с защитой резервным копированием и откатом.

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

Инструмент командной строки `mcpmgr` использует тот же список серверов, что и настольное приложение, поэтому изменения, сделанные в одном, видны в другом. Запустите его через `npx` или установите с помощью npm или Homebrew:

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

Из клона репозитория используйте `cargo run --bin mcpmgr -- <command>`.

В терминале `add` пошагово проводит вас через выбор серверов, клиентов и области (пользователь или проект), а затем перед записью показывает, что получит файл каждого клиента. На любой шаг можно ответить флагом (`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`); `--dry-run` показывает предварительный результат, а `--json` выводит машиночитаемые данные. При запуске внутри ИИ-агента для программирования, такого как Claude Code или Codex, вопросы пропускаются, а целью становится клиент этого агента.

## Как это работает

- Приложение читает локальные конфигурации клиентов и преобразует их во внутреннюю модель
- Применение записывает вывод для каждого клиента с атомарными обновлениями, резервным копированием и поддержкой отката
- Онлайн-каталог превращает запись реестра (npm, PyPI, Docker-образ или удалённый URL) в черновик сервера, который вы проверяете в редакторе; все клиенты изначально выключены

### Доступ к сети

MCP Manager выходит в сеть только для проверки обновлений и, пока открыта страница каталога, для запросов к MCP-реестрам. Результаты реестров кэшируются: каталог открывается из кэша и обновляет его в фоне через 6 часов, а без сети продолжает работать с последними данными. Добавление источника в **Настройки → Источники каталога** отправляет к нему один проверочный запрос. Отключите каталог в **Настройки → Онлайн-каталог**, чтобы полностью прекратить эти запросы.

## Область применения

Текущая область применения сосредоточена на управлении конфигурацией. Управление жизненным циклом во время выполнения — запуск и остановка процессов, логи и проверки работоспособности — намеренно не входит в v1.

## Структура проекта

```text
mcp-manager/
  src/                фронтенд-приложение
  src-tauri/          приложение tauri (команды для фронтенда, онлайн-каталог)
  crates/
    mcp-manager-core/ адаптеры клиентов, парсинг, безопасная запись (общие)
    mcp-manager-cli/  инструмент командной строки (`mcpmgr`)
  public/             статические ресурсы и брендинг
  docs/               заметки о релизах и проектные материалы
  openspec/           отслеживание изменений и спецификаций
```

### Основные модули (`crates/mcp-manager-core`)

- `platform`: разрешение путей с учётом ОС и контекст окружения
- `adapters`: логика импорта и применения для каждого клиента
- `core`: каноническая модель конфигурации и поведение слияния
- `parser`: разбор и извлечение YAML / JSON / TOML
- `storage`: атомарная запись, резервное копирование и откат
- `store`: общий `servers.yaml` с блокировкой и обнаружением изменений
- `workflow`: импорт, предпросмотр и применение для всех клиентов
- `ops`: редактирование и проверка списка серверов, зеркально повторяющие фронтенд

## Разработка

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## Автоматизация релизов

Репозиторий содержит workflow GitHub Actions для CI и сборки настольных релизов с помощью официального Tauri release action.

См. [`docs/releasing.md`](../../docs/releasing.md), где описаны:

- публикация GitHub Release по тегам
- сборка пакетов для macOS, Windows и Linux
- настройка подписи для обновлений
- необязательная подпись кода для платформ
- синхронизация версий и вспомогательные команды для git-тегов

## Участие в разработке

Issues и pull requests приветствуются.

Если вы планируете нетривиальное изменение, сначала откройте issue или обсуждение, чтобы до реализации были ясны объём и направление работы.

### Локальная разработка

```bash
make install
make tauri-dev
```

### Перед открытием PR

```bash
make test
make check
```

При изменении пользовательской документации проекта поддерживайте [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) и переводы в [`docs/readme/`](./) в согласованном состоянии.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Отказ от ответственности

Этот проект разрабатывается преимущественно с помощью ИИ (разработка под управлением ИИ). Код, тесты и документация в основном сгенерированы ИИ и проверены сопровождающим-человеком, поэтому в них всё ещё могут быть ошибки.

MCP Manager читает и перезаписывает конфигурационные файлы других инструментов. Резервное копирование и откат встроены, но вам всё равно следует самостоятельно сохранять копии важных конфигураций клиентов, проверять изменения перед применением и использовать программу на свой страх и риск. Она предоставляется «как есть» по [лицензии MIT](../../LICENSE), без каких-либо гарантий.

## Лицензия

MIT @ xJeway
