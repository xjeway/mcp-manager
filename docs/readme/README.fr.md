<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [Pourquoi MCP Manager](#pourquoi-mcp-manager)
- [Captures d’écran](#captures-décran)
- [Téléchargement et installation](#téléchargement-et-installation)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#utilisateurs-linux) · [Compiler depuis les sources](#compiler-depuis-les-sources)
- [Démarrage rapide](#démarrage-rapide)
- [Clients pris en charge](#clients-pris-en-charge)
- [Ligne de commande (aperçu)](#ligne-de-commande-aperçu)
- [Fonctionnement](#fonctionnement)
- [Périmètre](#périmètre)
- [Structure du projet](#structure-du-projet)
- [Développement](#développement)
- [Automatisation des versions](#automatisation-des-versions)
- [Contribuer](#contribuer)
- [Star History](#star-history)
- [Avertissement](#avertissement)
- [Licence](#licence)

## Pourquoi MCP Manager

`MCP Manager` est une application de bureau multiplateforme destinée à celles et ceux qui ne veulent pas modifier à la main les configurations de plusieurs clients MCP.

- Conservez un espace de travail MCP unifié pour tous les serveurs configurés
- Importez les entrées existantes depuis la configuration locale des clients
- Trouvez des serveurs dans le GitHub MCP Registry, le MCP Registry officiel ou tout registre que vous ajoutez et qui implémente l’API MCP Registry, puis ajoutez-les en quelques clics
- Modifiez les serveurs en mode formulaire ou en JSON brut
- Appliquez la configuration générée à plusieurs clients pris en charge
- Vérifiez les écritures risquées avant que les fichiers ne soient modifiés
- Bénéficiez de sauvegardes et d’une restauration possible lors de l’application
- Utilisez le thème clair, le thème sombre ou celui du système
- Utilisez l’interface en anglais, chinois simplifié, chinois traditionnel, japonais, coréen, allemand, français, espagnol, italien, portugais du Brésil ou russe (elle suit par défaut la langue du système)

## Captures d’écran

![Interface de MCP Manager en anglais, thème clair](../../docs/page-en.png)

![Interface de MCP Manager en anglais, thème sombre](../../docs/page-en-dark.png)

## Téléchargement et installation

### Configuration requise

- Des paquets de bureau sont publiés pour macOS, Windows et Linux
- La compilation depuis les sources nécessite Node.js 20+, npm 10+, Rust stable et les dépendances système de Tauri pour votre plateforme

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### Téléchargement manuel

Téléchargez depuis :

- <https://github.com/xjeway/mcp-manager/releases/latest>

Exemples de noms de fichiers macOS :

- `MCP-Manager-<version>-<arch>.dmg`

Choix de l’architecture :

- Mac Apple Silicon : choisissez les fichiers `arm64` ou `aarch64`
- Mac Intel : choisissez les fichiers `x64` ou `x86_64`

Après avoir téléchargé le bon fichier macOS, ouvrez-le localement :

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Les versions macOS publiques actuelles peuvent encore être non signées, le temps que la signature et la notarisation Apple soient mises en place. Si Gatekeeper bloque `MCP Manager.app`, procédez ainsi :

1. Faites glisser `MCP Manager.app` dans `/Applications`.
2. Dans le Finder, faites un clic droit sur l’application une fois et choisissez `Open`.
3. Si macOS indique toujours que l’application ne peut pas être ouverte ou qu’elle est endommagée, retirez l’attribut de quarantaine et réessayez :

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### Téléchargement manuel

Téléchargez depuis :

- <https://github.com/xjeway/mcp-manager/releases/latest>

Exemples de noms de fichiers Windows :

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Choix de l’architecture :

- Appareils Windows sur ARM : choisissez l’installateur `arm64`
- PC Intel / AMD : choisissez l’installateur `x64` ou `x86_64`

Choix du paquet :

- `setup.exe` : disponible pour Windows x64 et Windows arm64
- `.msi` : publié pour les versions stables Windows x64

Choisissez le fichier correspondant à votre architecture, puis exécutez-le localement :

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# ou
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Utilisateurs Linux

Téléchargez la dernière version Linux depuis la page [Releases](https://github.com/xjeway/mcp-manager/releases/latest) :

Exemples de noms de fichiers Linux :

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (universel)

Guide détaillé par distribution :

- `.deb` : Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS et autres distributions basées sur Debian
- `.rpm` : Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE et autres distributions basées sur RPM
- `.AppImage` : option portable pour Linux de bureau lorsque vous ne voulez pas de paquet système

Choix de l’architecture :

- `x86_64` ou `amd64` pour la plupart des PC 64 bits Intel et AMD
- `arm64` ou `aarch64` pour les appareils Linux ARM64, lorsque ce fichier est publié dans la version

Installez avec la commande correspondante :

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### Compiler depuis les sources

Installez les dépendances :

```bash
make install
```

Lancez l’application de bureau :

```bash
make tauri-dev
```

Lancez uniquement l’interface web :

```bash
make dev
```

## Démarrage rapide

1. Lancez `MCP Manager`.
2. Importez les entrées existantes depuis les fichiers de configuration locaux des clients.
3. Modifiez les serveurs en mode formulaire ou en mode JSON.
4. Appliquez la configuration générée à un ou plusieurs clients pris en charge.
5. Vérifiez les avertissements, puis écrivez avec la protection par sauvegarde et restauration.

## Clients pris en charge

Chacun des clients ci-dessous prend en charge à la fois l’**import** (lecture des serveurs MCP existants depuis sa configuration) et l’**application** (réécriture de la configuration unifiée).

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

## Ligne de commande (aperçu)

Un outil en ligne de commande, `mcpmgr`, partage la liste des serveurs de l’application de bureau : les modifications faites dans l’un apparaissent dans l’autre. Lancez-le avec `npx`, ou installez-le avec npm ou Homebrew :

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

Depuis une copie du dépôt, utilisez `cargo run --bin mcpmgr -- <command>`.

Dans un terminal, `add` vous guide dans le choix des serveurs, des clients et de la portée (utilisateur ou projet), puis montre ce que recevra chaque fichier client avant d’écrire. Chaque étape peut être renseignée par une option (`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`) ; `--dry-run` affiche un aperçu et `--json` produit une sortie lisible par une machine. Lancé dans un agent de programmation IA comme Claude Code ou Codex, il ignore les questions et cible le client de cet agent.

## Fonctionnement

- L’application lit la configuration locale des clients et la convertit dans le modèle interne
- L’application écrit une sortie propre à chaque client, avec des mises à jour atomiques, des sauvegardes et une restauration possible
- Le catalogue en ligne transforme une entrée de registre (npm, PyPI, image Docker ou URL distante) en brouillon de serveur que vous vérifiez dans l’éditeur ; tous les clients sont désactivés au départ

### Accès réseau

MCP Manager ne se connecte à Internet que pour rechercher des mises à jour et, lorsque la page du catalogue est ouverte, pour interroger les registres MCP. Les résultats des registres sont mis en cache : le catalogue s’ouvre depuis le cache et le met à jour en arrière-plan au bout de 6 heures, et il fonctionne toujours hors ligne avec les dernières données. Ajouter une source dans **Paramètres → Sources du catalogue** lui envoie une requête de vérification. Désactivez le catalogue dans **Paramètres → Catalogue en ligne** pour supprimer totalement ces requêtes.

## Périmètre

Le périmètre actuel se concentre sur la gestion de la configuration. La gestion du cycle de vie à l’exécution, comme le démarrage et l’arrêt des processus, les journaux et les vérifications d’état, est volontairement exclue de la v1.

## Structure du projet

```text
mcp-manager/
  src/                application frontend
  src-tauri/          application tauri (commandes exposées au frontend, catalogue en ligne)
  crates/
    mcp-manager-core/ adaptateurs clients, analyse, écritures sûres (partagé)
    mcp-manager-cli/  outil en ligne de commande (`mcpmgr`)
  public/             ressources statiques et identité visuelle
  docs/               notes de version et références de conception
  openspec/           suivi des changements et des spécifications
```

### Modules principaux (`crates/mcp-manager-core`)

- `platform` : résolution des chemins selon l’OS et contexte d’environnement
- `adapters` : logique d’import et d’application propre à chaque client
- `core` : modèle de configuration canonique et comportement de fusion
- `parser` : analyse et extraction YAML / JSON / TOML
- `storage` : écriture atomique, sauvegarde et restauration
- `store` : le fichier partagé `servers.yaml`, avec verrouillage et détection des changements
- `workflow` : import, aperçu et application pour tous les clients
- `ops` : modifications et validation de la liste des serveurs, en miroir du frontend

## Développement

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## Automatisation des versions

Ce dépôt contient des workflows GitHub Actions pour la CI et la création des paquets de bureau à l’aide de l’action de publication officielle de Tauri.

Consultez [`docs/releasing.md`](../../docs/releasing.md) pour :

- la publication de GitHub Releases déclenchée par les tags
- la génération des paquets macOS, Windows et Linux
- la configuration de la signature du programme de mise à jour
- la signature de code facultative par plateforme
- les commandes d’aide à la synchronisation des versions et aux tags git

## Contribuer

Les issues et les pull requests sont les bienvenues.

Si vous prévoyez de proposer un changement important, ouvrez d’abord une issue ou une discussion afin que le périmètre et l’orientation soient clairs avant l’implémentation.

### Développement local

```bash
make install
make tauri-dev
```

### Avant d’ouvrir une PR

```bash
make test
make check
```

Gardez [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) et les traductions de [`docs/readme/`](./) synchronisés lorsque vous modifiez la documentation du projet destinée aux utilisateurs.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Avertissement

Ce projet est principalement développé avec l’aide de l’IA (développement piloté par l’IA). Le code, les tests et la documentation sont en grande partie générés par l’IA et relus par un mainteneur humain ; ils peuvent donc encore contenir des erreurs.

MCP Manager lit et réécrit les fichiers de configuration d’autres outils. Les sauvegardes et la restauration sont intégrées, mais vous devriez tout de même sauvegarder vous-même les configurations importantes de vos clients, vérifier les changements avant de les appliquer et utiliser le logiciel à vos propres risques. Il est fourni « en l’état » sous [licence MIT](../../LICENSE), sans aucune garantie.

## Licence

MIT @ xJeway
