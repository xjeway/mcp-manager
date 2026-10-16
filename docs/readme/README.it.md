<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="Logo di MCP Manager" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [Perché MCP Manager](#perché-mcp-manager)
- [Screenshot](#screenshot)
- [Download e installazione](#download-e-installazione)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#utenti-linux) · [Compilare dai sorgenti](#compilare-dai-sorgenti)
- [Avvio rapido](#avvio-rapido)
- [Client supportati](#client-supportati)
- [Riga di comando (anteprima)](#riga-di-comando-anteprima)
- [Come funziona](#come-funziona)
- [Ambito](#ambito)
- [Struttura del progetto](#struttura-del-progetto)
- [Sviluppo](#sviluppo)
- [Automazione dei rilasci](#automazione-dei-rilasci)
- [Contribuire](#contribuire)
- [Star History](#star-history)
- [Esclusione di responsabilità](#esclusione-di-responsabilità)
- [Licenza](#licenza)

## Perché MCP Manager

`MCP Manager` è un'app desktop multipiattaforma pensata per chi non vuole modificare a mano le configurazioni di più client MCP.

- Mantieni un workspace MCP unificato per tutti i server configurati
- Importa le voci esistenti dalla configurazione locale dei client
- Trova server nel GitHub MCP Registry, nel MCP Registry ufficiale o in qualsiasi registro che aggiungi e che implementi l'API MCP Registry, e aggiungili in pochi clic
- Modifica i server in modalità modulo o in modalità JSON grezzo
- Applica la configurazione generata a più client supportati
- Controlla le scritture rischiose prima che i file vengano modificati
- Conserva backup e supporto al rollback durante l'applicazione
- Usa il tema chiaro, il tema scuro o quello di sistema
- Usa l'interfaccia in inglese, cinese semplificato, cinese tradizionale, giapponese, coreano, tedesco, francese, spagnolo, italiano, portoghese brasiliano o russo (per impostazione predefinita segue la lingua di sistema)

## Screenshot

![Interfaccia di MCP Manager in inglese, tema chiaro](../../docs/page-en.png)

![Interfaccia di MCP Manager in inglese, tema scuro](../../docs/page-en-dark.png)

## Download e installazione

### Requisiti di sistema

- I pacchetti desktop sono pubblicati per macOS, Windows e Linux
- La compilazione dai sorgenti richiede Node.js 20+, npm 10+, Rust stable e le dipendenze di sistema di Tauri per la tua piattaforma

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### Download manuale

Scarica da:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Esempi di nomi degli asset per macOS:

- `MCP-Manager-<version>-<arch>.dmg`

Guida all'architettura:

- Mac con Apple Silicon: scegli gli asset `arm64` o `aarch64`
- Mac Intel: scegli gli asset `x64` o `x86_64`

Dopo aver scaricato l'asset macOS corretto, aprilo in locale:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Le release pubbliche attuali per macOS potrebbero non essere ancora firmate, mentre la firma e la notarizzazione Apple sono in preparazione. Se Gatekeeper blocca `MCP Manager.app`, segui questa procedura alternativa:

1. Trascina `MCP Manager.app` in `/Applications`.
2. Nel Finder, fai clic destro sull'app una volta e scegli `Open`.
3. Se macOS indica ancora che l'app non può essere aperta o è danneggiata, rimuovi l'attributo di quarantena e riprova:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### Download manuale

Scarica da:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Esempi di nomi degli asset per Windows:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Guida all'architettura:

- Dispositivi Windows on ARM: scegli l'asset di installazione `arm64`
- PC Intel / AMD: scegli l'asset di installazione `x64` o `x86_64`

Guida ai pacchetti:

- `setup.exe`: disponibile sia per Windows x64 sia per Windows arm64
- `.msi`: pubblicato per le release stabili di Windows x64

Scegli l'asset adatto alla tua architettura, quindi eseguilo in locale:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# oppure
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Utenti Linux

Scarica l'ultima build per Linux dalla pagina [Releases](https://github.com/xjeway/mcp-manager/releases/latest):

Esempi di nomi degli asset per Linux:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (universale)

Guida dettagliata alle distribuzioni:

- `.deb`: Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS e altre distribuzioni basate su Debian
- `.rpm`: Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE e altre distribuzioni basate su RPM
- `.AppImage`: opzione portatile per Linux desktop quando non vuoi un pacchetto di sistema

Guida all'architettura:

- `x86_64` o `amd64` per la maggior parte dei PC Intel e AMD a 64 bit
- `arm64` o `aarch64` per i dispositivi Linux ARM64, quando quell'asset è pubblicato nella release

Installa con il comando corrispondente:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### Compilare dai sorgenti

Installa le dipendenze:

```bash
make install
```

Avvia l'app desktop:

```bash
make tauri-dev
```

Avvia solo l'interfaccia web:

```bash
make dev
```

## Avvio rapido

1. Avvia `MCP Manager`.
2. Importa le voci esistenti dai file di configurazione locali dei client.
3. Modifica i server in modalità modulo o in modalità JSON.
4. Applica la configurazione generata a uno o più client supportati.
5. Controlla gli avvisi, quindi scrivi con la protezione di backup e rollback.

## Client supportati

Ogni client qui sotto supporta sia l'**importazione** (lettura dei server MCP esistenti dalla sua configurazione) sia l'**applicazione** (riscrittura della configurazione unificata).

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="../../docs/assets/clients/claudecode.svg" alt="Logo di Claude Code" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="../../docs/assets/clients/claude.svg" alt="Logo di Claude Desktop" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="../../docs/assets/clients/codex.svg" alt="Logo di Codex" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="../../docs/assets/clients/cursor.svg" alt="Logo di Cursor" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="../../docs/assets/clients/opencode.svg" alt="Logo di OpenCode" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="../../docs/assets/clients/githubcopilot.svg" alt="Logo di GitHub Copilot" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="../../docs/assets/clients/gemini.svg" alt="Logo di Gemini CLI" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="../../docs/assets/clients/antigravity.svg" alt="Logo di Antigravity" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="../../docs/assets/clients/iflow.svg" alt="Logo di iFlow" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="../../docs/assets/clients/qwen.svg" alt="Logo di Qwen Code" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="../../docs/assets/clients/cline.svg" alt="Logo di Cline" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="../../docs/assets/clients/windsurf.svg" alt="Logo di Windsurf" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../docs/assets/clients/kiro.svg" alt="Logo di Kiro" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../docs/assets/clients/qoder.svg" alt="Logo di Qoder" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../docs/assets/clients/vscode.svg" alt="Logo di VS Code" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## Riga di comando (anteprima)

Uno strumento da riga di comando, `mcpmgr`, condivide l'elenco dei server dell'app desktop, quindi le modifiche fatte in uno compaiono anche nell'altro. Eseguilo con `npx`, oppure installalo con npm o Homebrew:

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

Da un checkout del repository, usa `cargo run --bin mcpmgr -- <command>`.

In un terminale, `add` ti guida nella scelta di server, client e ambito utente o progetto, poi mostra cosa riceverà il file di ciascun client prima di scrivere. Ogni passaggio può essere risolto con un flag (`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`); `--dry-run` mostra un'anteprima e `--json` stampa un output leggibile dalle macchine. Se eseguito all'interno di un agente di programmazione AI come Claude Code o Codex, salta le domande e prende come destinazione il client di quell'agente.

## Come funziona

- L'app legge la configurazione locale dei client e la converte nel modello interno
- L'applicazione scrive un output specifico per ciascun client con aggiornamenti atomici, backup e supporto al rollback
- Il catalogo online trasforma una voce di registro (npm, PyPI, immagine Docker o URL remoto) in una bozza di server da controllare nell'editor; ogni client parte disattivato

### Accesso alla rete

MCP Manager si connette a Internet solo per verificare gli aggiornamenti e, mentre la pagina del catalogo è aperta, per interrogare i registri MCP. I risultati dei registri vengono memorizzati nella cache: il catalogo si apre dalla cache e la aggiorna in background dopo 6 ore, e funziona anche offline con gli ultimi dati disponibili. Aggiungere un'origine in **Impostazioni → Origini del catalogo** invia a quest'ultima una richiesta di verifica. Disattiva il catalogo in **Impostazioni → Catalogo online** per interrompere completamente queste richieste.

## Ambito

L'ambito attuale è incentrato sulla gestione della configurazione. La gestione del ciclo di vita a runtime, come avvio e arresto dei processi, log e controlli di integrità, è volutamente esclusa dalla v1.

## Struttura del progetto

```text
mcp-manager/
  src/                applicazione frontend
  src-tauri/          app tauri (comandi esposti al frontend, catalogo online)
  crates/
    mcp-manager-core/ adattatori dei client, parsing, scritture sicure (condiviso)
    mcp-manager-cli/  strumento da riga di comando (`mcpmgr`)
  public/             asset statici e branding
  docs/               note di rilascio e riferimenti di design
  openspec/           tracciamento di modifiche e specifiche
```

### Moduli principali (`crates/mcp-manager-core`)

- `platform`: risoluzione dei percorsi in base al sistema operativo e contesto dell'ambiente
- `adapters`: logica di importazione e applicazione per ciascun client
- `core`: modello di configurazione canonico e comportamento di unione
- `parser`: parsing ed estrazione di YAML / JSON / TOML
- `storage`: scrittura atomica, backup e rollback
- `store`: il file condiviso `servers.yaml`, con locking e rilevamento delle modifiche
- `workflow`: importazione, anteprima e applicazione su tutti i client
- `ops`: modifiche e validazione dell'elenco dei server, speculari al frontend

## Sviluppo

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## Automazione dei rilasci

Questo repository include workflow di GitHub Actions per la CI e per il packaging delle release desktop con l'azione di rilascio ufficiale di Tauri.

Consulta [`docs/releasing.md`](../../docs/releasing.md) per:

- pubblicazione di GitHub Release basata sui tag
- generazione dei pacchetti per macOS, Windows e Linux
- configurazione della firma per l'updater
- firma del codice opzionale per piattaforma
- comandi di supporto per la sincronizzazione della versione e i tag git

## Contribuire

Issue e pull request sono benvenute.

Se intendi contribuire con una modifica non banale, apri prima una issue o una discussione, così che ambito e direzione siano chiari prima dell'implementazione.

### Sviluppo locale

```bash
make install
make tauri-dev
```

### Prima di aprire una PR

```bash
make test
make check
```

Mantieni sincronizzati [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) e le traduzioni in [`docs/readme/`](./) quando modifichi la documentazione del progetto rivolta agli utenti.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Grafico della Star History" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Esclusione di responsabilità

Questo progetto è sviluppato principalmente con l'assistenza dell'AI (sviluppo guidato dall'AI). Codice, test e documentazione sono in gran parte generati dall'AI e revisionati da un maintainer umano, quindi potrebbero comunque contenere errori.

MCP Manager legge e riscrive i file di configurazione di altri strumenti. Backup e rollback sono integrati, ma dovresti comunque eseguire tu stesso il backup delle configurazioni importanti dei client, controllare le modifiche prima di applicarle e usare il software a tuo rischio. Viene fornito "così com'è" secondo la [licenza MIT](../../LICENSE), senza alcuna garanzia.

## Licenza

MIT @ xJeway
