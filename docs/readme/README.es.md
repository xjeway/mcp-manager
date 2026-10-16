<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Una aplicación de escritorio multiplataforma para gestionar las configuraciones de servidores MCP en Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI y más.
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
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <b>Español</b> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>Descargar MCP Manager</ins></a></h3>

## Índice

- [Funciones](#funciones)
- [Instalación](#instalación)
- [Clientes compatibles](#clientes-compatibles)
- [Línea de comandos (vista previa)](#línea-de-comandos-vista-previa)
- [Privacidad y red](#privacidad-y-red)
- [Desarrollo](#desarrollo)
- [Star History](#star-history)
- [Aviso legal](#aviso-legal)

## Funciones

- **Un único espacio de trabajo** para todos los servidores MCP, importados desde las configuraciones que ya tienen tus clientes
- **Catálogo en línea**: encuentra servidores en el GitHub MCP Registry, el MCP Registry oficial o cualquier registro compatible, y añádelos con unos pocos clics
- Edición en **modo formulario o JSON sin procesar**
- **Aplica a varios clientes a la vez**, con revisión de las escrituras de riesgo, copias de seguridad automáticas y reversión
- Tema claro / oscuro, y una interfaz en 11 idiomas que sigue el idioma del sistema

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, tema claro" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, tema oscuro" /></td>
  </tr>
</table>

## Instalación

Descarga el paquete para tu plataforma desde la [última versión](https://github.com/xjeway/mcp-manager/releases/latest):

| Plataforma | Paquete |
| --- | --- |
| macOS | `.dmg`, o `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) o `.msi` (x64) |
| Linux | `.deb`, `.rpm` o `.AppImage` |

Elige `arm64` / `aarch64` para Apple Silicon y dispositivos ARM, y `x64` / `x86_64` / `amd64` en los demás casos.

<details>
<summary>¿macOS dice que la aplicación está dañada o que no se puede abrir?</summary>

Las compilaciones de las versiones aún no están notarizadas. Mueve la aplicación a `/Applications` y ejecuta:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Clientes compatibles

Todos los clientes siguientes admiten tanto la **importación** (leer los servidores MCP existentes de su configuración) como la **aplicación** (volver a escribir la configuración unificada).

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

## Línea de comandos (vista previa)

`mcpmgr` comparte la lista de servidores de la aplicación de escritorio, de modo que los cambios hechos en una aparecen en la otra.

```bash
npx @mcpmgr/cli list                      # o: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

En una terminal, `add` te guía para elegir los servidores, los clientes y el ámbito, y muestra una vista previa de cada archivo antes de escribir. Cada paso tiene también una opción (`--yes`, `--dry-run`, `--json`, …); dentro de un agente de programación con IA como Claude Code o Codex, omite las preguntas y apunta al cliente de ese agente.

## Privacidad y red

MCP Manager solo se conecta a internet para buscar actualizaciones y, mientras la página del catálogo está abierta, para consultar registros MCP (en caché durante 6 horas, utilizable sin conexión). Desactiva el catálogo en **Ajustes → Catálogo en línea** para detener por completo esas solicitudes.

## Desarrollo

Requiere Node.js 20+, Rust stable y los [requisitos previos de Tauri](https://tauri.app/start/prerequisites/) para tu plataforma.

```bash
make install      # instalar dependencias
make tauri-dev    # ejecutar la aplicación de escritorio
make test && make check
```

La lógica principal (adaptadores de clientes, análisis, escrituras seguras) está en `crates/mcp-manager-core`, compartida por la aplicación Tauri en `src-tauri/` y la CLI en `crates/mcp-manager-cli/`. El proceso de publicación de versiones se describe en [`docs/releasing.md`](../../docs/releasing.md).

Las incidencias y los pull requests son bienvenidos; para cambios de mayor envergadura, abre primero una incidencia. Cuando cambies la documentación dirigida a los usuarios, mantén sincronizados [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) y [`docs/readme/`](./).

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Aviso legal

Este proyecto se desarrolla principalmente con ayuda de IA y lo revisa un mantenedor humano, por lo que aún puede contener errores. MCP Manager reescribe los archivos de configuración de otras herramientas; incluye copias de seguridad y reversión, pero conserva tus propias copias de seguridad de las configuraciones importantes. Se proporciona "tal cual" bajo la [licencia MIT](../../LICENSE).
