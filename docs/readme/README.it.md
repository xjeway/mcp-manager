<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Un'app desktop multipiattaforma per gestire le configurazioni dei server MCP in Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI e altri ancora.
</p>

<p align="center">
  <a href="https://github.com/xjeway/mcp-manager/stargazers"><img src="https://img.shields.io/github/stars/xjeway/mcp-manager?style=flat&label=%E2%98%85&color=f59e0b" alt="Stelle su GitHub" /></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-0f172a.svg" alt="Licenza MIT" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24c8db.svg" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/React-19-61dafb.svg" alt="React 19" />
  <img src="https://img.shields.io/badge/Rust-stable-f74c00.svg" alt="Rust stable" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-16a34a.svg" alt="Multipiattaforma" />
</p>

<p align="center">
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <b>Italiano</b> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>Scarica MCP Manager</ins></a></h3>

## Indice

- [Funzionalità](#funzionalità)
- [Installazione](#installazione)
- [Client supportati](#client-supportati)
- [Riga di comando (anteprima)](#riga-di-comando-anteprima)
- [Privacy e rete](#privacy-e-rete)
- [Sviluppo](#sviluppo)
- [Star History](#star-history)
- [Esclusione di responsabilità](#esclusione-di-responsabilità)

## Funzionalità

- **Un unico workspace** per tutti i server MCP, importati dalle configurazioni che i tuoi client hanno già
- **Catalogo online**: trova server nel GitHub MCP Registry, nel MCP Registry ufficiale o in qualsiasi registro compatibile, e aggiungili in pochi clic
- Modifica in **modalità modulo o JSON grezzo**
- **Applica a più client contemporaneamente**, con controllo delle scritture rischiose, backup automatici e rollback
- Tema chiaro / scuro e un'interfaccia in 11 lingue che segue la lingua di sistema

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, tema chiaro" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, tema scuro" /></td>
  </tr>
</table>

## Installazione

Scarica il pacchetto per la tua piattaforma dall'[ultima release](https://github.com/xjeway/mcp-manager/releases/latest):

| Piattaforma | Pacchetto |
| --- | --- |
| macOS | `.dmg`, oppure `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) o `.msi` (x64) |
| Linux | `.deb`, `.rpm` o `.AppImage` |

Scegli `arm64` / `aarch64` per Apple Silicon e i dispositivi ARM, altrimenti `x64` / `x86_64` / `amd64`.

<details>
<summary>macOS dice che l'app è danneggiata o non può essere aperta?</summary>

Le build di rilascio non sono ancora notarizzate. Sposta l'app in `/Applications`, quindi esegui:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Client supportati

Ogni client qui sotto supporta sia l'**importazione** (lettura dei server MCP esistenti dalla sua configurazione) sia l'**applicazione** (riscrittura della configurazione unificata).

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="../../public/logos/apps/claudecode.svg" alt="Logo di Claude Code" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="../../public/logos/apps/claude.svg" alt="Logo di Claude Desktop" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="../../public/logos/apps/codex.svg" alt="Logo di Codex" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="../../public/logos/apps/cursor.svg" alt="Logo di Cursor" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="../../public/logos/apps/opencode.svg" alt="Logo di OpenCode" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="../../public/logos/apps/githubcopilot.svg" alt="Logo di GitHub Copilot" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="../../public/logos/apps/gemini.svg" alt="Logo di Gemini CLI" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="../../public/logos/apps/antigravity.svg" alt="Logo di Antigravity" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="../../public/logos/apps/iflow.svg" alt="Logo di iFlow" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="../../public/logos/apps/qwen.svg" alt="Logo di Qwen Code" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="../../public/logos/apps/cline.svg" alt="Logo di Cline" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="../../public/logos/apps/devin.svg" alt="Devin Desktop logo" width="16" valign="middle" /> Devin Desktop</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../public/logos/apps/kiro.svg" alt="Logo di Kiro" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../public/logos/apps/qoder.svg" alt="Logo di Qoder" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://kilo.ai"><kbd><img src="../../public/logos/apps/kilocode.svg" alt="Kilo Code logo" width="16" valign="middle" /> Kilo Code</kbd></a> &nbsp;
  <a href="https://factory.ai"><kbd><img src="../../public/logos/apps/factorydroid.svg" alt="Factory Droid logo" width="16" valign="middle" /> Factory Droid</kbd></a> &nbsp;
  <a href="https://github.com/MoonshotAI/kimi-cli"><kbd><img src="../../public/logos/apps/kimi.svg" alt="Kimi CLI logo" width="16" valign="middle" /> Kimi CLI</kbd></a> &nbsp;
  <a href="https://zed.dev"><kbd><img src="../../public/logos/apps/zed.svg" alt="Zed logo" width="16" valign="middle" /> Zed</kbd></a> &nbsp;
  <a href="https://github.com/mistralai/mistral-vibe"><kbd><img src="../../public/logos/apps/mistral.svg" alt="Mistral Vibe logo" width="16" valign="middle" /> Mistral Vibe</kbd></a> &nbsp;
  <a href="https://github.com/NousResearch/hermes-agent"><kbd><img src="../../public/logos/apps/hermes.svg" alt="Hermes Agent logo" width="16" valign="middle" /> Hermes Agent</kbd></a> &nbsp;
  <a href="https://github.com/block/goose"><kbd><img src="../../public/logos/apps/goose.svg" alt="Goose logo" width="16" valign="middle" /> Goose</kbd></a> &nbsp;
  <a href="https://ampcode.com"><kbd><img src="../../public/logos/apps/amp.svg" alt="Amp logo" width="16" valign="middle" /> Amp</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../public/logos/apps/vscode.svg" alt="Logo di VS Code" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## Riga di comando (anteprima)

`mcpmgr` condivide l'elenco dei server dell'app desktop, quindi le modifiche fatte in uno compaiono anche nell'altro.

```bash
npx @mcpmgr/cli list                      # oppure: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

In un terminale, `add` ti guida nella scelta di server, client e ambito, e mostra un'anteprima di ogni file prima di scrivere. Ogni passaggio ha anche un flag (`--yes`, `--dry-run`, `--json`, …); all'interno di un agente di programmazione AI come Claude Code o Codex, salta le domande e prende come destinazione il client di quell'agente.

## Privacy e rete

MCP Manager si connette a Internet solo per verificare gli aggiornamenti e, mentre la pagina del catalogo è aperta, per interrogare i registri MCP (risultati nella cache per 6 ore, utilizzabile offline). Disattiva il catalogo in **Impostazioni → Catalogo online** per interrompere completamente queste richieste.

## Sviluppo

Richiede Node.js 20+, Rust stable e i [prerequisiti di Tauri](https://tauri.app/start/prerequisites/) per la tua piattaforma.

```bash
make install      # installa le dipendenze
make tauri-dev    # avvia l'app desktop
make test && make check
```

La logica principale (adattatori dei client, parsing, scritture sicure) si trova in `crates/mcp-manager-core`, condivisa dall'app Tauri in `src-tauri/` e dalla CLI in `crates/mcp-manager-cli/`. I rilasci sono descritti in [`docs/releasing.md`](../../docs/releasing.md).

Issue e pull request sono benvenute; per modifiche più ampie, apri prima una issue. Quando modifichi la documentazione rivolta agli utenti, mantieni sincronizzati [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) e [`docs/readme/`](./).

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Grafico della Star History" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Esclusione di responsabilità

Questo progetto è sviluppato principalmente con l'assistenza dell'AI e revisionato da un maintainer umano, quindi potrebbe comunque contenere errori. MCP Manager riscrive i file di configurazione di altri strumenti; backup e rollback sono integrati, ma conserva comunque i tuoi backup delle configurazioni importanti. Viene fornito "così com'è" secondo la [licenza MIT](../../LICENSE).
