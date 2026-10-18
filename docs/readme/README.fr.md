<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Une application de bureau multiplateforme pour gérer les configurations de serveurs MCP de Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI et bien d’autres.
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
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <b>Français</b> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>Télécharger MCP Manager</ins></a></h3>

## Table des matières

- [Fonctionnalités](#fonctionnalités)
- [Installation](#installation)
- [Clients pris en charge](#clients-pris-en-charge)
- [Ligne de commande (aperçu)](#ligne-de-commande-aperçu)
- [Confidentialité et réseau](#confidentialité-et-réseau)
- [Développement](#développement)
- [Star History](#star-history)
- [Avertissement](#avertissement)

## Fonctionnalités

- **Un espace de travail unique** pour tous vos serveurs MCP, importés depuis les configurations que vos clients possèdent déjà
- **Catalogue en ligne** : trouvez des serveurs dans le GitHub MCP Registry, le MCP Registry officiel ou tout registre compatible, puis ajoutez-les en quelques clics
- Modification en **mode formulaire ou en JSON brut**
- **Application à plusieurs clients à la fois**, avec vérification des écritures risquées, sauvegardes automatiques et restauration
- Thème clair / sombre, et une interface en 11 langues qui suit la langue du système

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, thème clair" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, thème sombre" /></td>
  </tr>
</table>

## Installation

Téléchargez le paquet correspondant à votre plateforme depuis la [dernière version](https://github.com/xjeway/mcp-manager/releases/latest) :

| Plateforme | Paquet |
| --- | --- |
| macOS | `.dmg`, ou `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) ou `.msi` (x64) |
| Linux | `.deb`, `.rpm` ou `.AppImage` |

Choisissez `arm64` / `aarch64` pour Apple Silicon et les appareils ARM, sinon `x64` / `x86_64` / `amd64`.

<details>
<summary>macOS indique que l’application est endommagée ou ne peut pas être ouverte ?</summary>

Les versions publiées sont signées ad hoc, mais pas encore notarisées par Apple. Au premier lancement :

1. Déplacez l’application dans `/Applications`, ouvrez-la et fermez l’avertissement.
2. Ouvrez **Réglages Système → Confidentialité et sécurité**, cliquez sur **Ouvrir quand même** en bas de la page, puis confirmez.

Si macOS indique toujours que l’application est endommagée (anciennes versions non signées), exécutez :

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Clients pris en charge

Chacun des clients ci-dessous prend en charge à la fois l’**import** (lecture des serveurs MCP existants depuis sa configuration) et l’**application** (réécriture de la configuration unifiée).

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

## Ligne de commande (aperçu)

`mcpmgr` partage la liste des serveurs de l’application de bureau : les modifications faites dans l’un apparaissent dans l’autre.

```bash
npx @mcpmgr/cli list                      # ou : npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

Dans un terminal, `add` vous guide dans le choix des serveurs, des clients et de la portée, et affiche un aperçu de chaque fichier avant d’écrire. Chaque étape dispose aussi d’une option (`--yes`, `--dry-run`, `--json`, …) ; dans un agent de programmation IA comme Claude Code ou Codex, il ignore les questions et cible le client de cet agent.

## Confidentialité et réseau

MCP Manager ne se connecte à Internet que pour rechercher des mises à jour et, lorsque la page du catalogue est ouverte, pour interroger les registres MCP (mis en cache pendant 6 heures, utilisable hors ligne). Désactivez le catalogue dans **Paramètres → Catalogue en ligne** pour supprimer totalement ces requêtes.

## Développement

Nécessite Node.js 20+, Rust stable et les [prérequis de Tauri](https://tauri.app/start/prerequisites/) pour votre plateforme.

```bash
make install      # installer les dépendances
make tauri-dev    # lancer l’application de bureau
make test && make check
```

La logique principale (adaptateurs clients, analyse, écritures sûres) se trouve dans `crates/mcp-manager-core`, partagée par l’application Tauri dans `src-tauri/` et l’outil en ligne de commande dans `crates/mcp-manager-cli/`. Les versions sont décrites dans [`docs/releasing.md`](../../docs/releasing.md).

Les issues et les pull requests sont les bienvenues ; pour les changements importants, ouvrez d’abord une issue. Lorsque vous modifiez la documentation destinée aux utilisateurs, gardez [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) et [`docs/readme/`](./) synchronisés.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Avertissement

Ce projet est principalement développé avec l’aide de l’IA et relu par un mainteneur humain ; il peut donc encore contenir des erreurs. MCP Manager réécrit les fichiers de configuration d’autres outils ; les sauvegardes et la restauration sont intégrées, mais conservez vos propres sauvegardes des configurations importantes. Fourni « en l’état » sous [licence MIT](../../LICENSE).
