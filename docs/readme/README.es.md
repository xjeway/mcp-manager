<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [Por qué MCP Manager](#por-qué-mcp-manager)
- [Capturas de pantalla](#capturas-de-pantalla)
- [Descarga e instalación](#descarga-e-instalación)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#usuarios-de-linux) · [Compilar desde el código fuente](#compilar-desde-el-código-fuente)
- [Inicio rápido](#inicio-rápido)
- [Clientes compatibles](#clientes-compatibles)
- [Línea de comandos (vista previa)](#línea-de-comandos-vista-previa)
- [Cómo funciona](#cómo-funciona)
- [Alcance](#alcance)
- [Estructura del proyecto](#estructura-del-proyecto)
- [Desarrollo](#desarrollo)
- [Automatización de versiones](#automatización-de-versiones)
- [Contribuir](#contribuir)
- [Star History](#star-history)
- [Aviso legal](#aviso-legal)
- [Licencia](#licencia)

## Por qué MCP Manager

`MCP Manager` es una aplicación de escritorio multiplataforma para quienes no quieren editar a mano las configuraciones de varios clientes MCP.

- Mantén un espacio de trabajo MCP unificado para todos los servidores configurados
- Importa las entradas existentes desde la configuración local de los clientes
- Encuentra servidores en el GitHub MCP Registry, el MCP Registry oficial o cualquier registro que añadas y que implemente la API de MCP Registry, y añádelos con unos pocos clics
- Edita servidores en modo formulario o en modo JSON sin procesar
- Aplica la configuración generada a varios clientes compatibles
- Revisa las escrituras de riesgo antes de que cambien los archivos
- Conserva copias de seguridad y la posibilidad de revertir durante la aplicación
- Usa el modo claro, el modo oscuro o el tema del sistema
- Usa la interfaz en inglés, chino simplificado, chino tradicional, japonés, coreano, alemán, francés, español, italiano, portugués de Brasil o ruso (sigue el idioma del sistema de forma predeterminada)

## Capturas de pantalla

![Interfaz de MCP Manager en inglés, tema claro](../../docs/page-en.png)

![Interfaz de MCP Manager en inglés, tema oscuro](../../docs/page-en-dark.png)

## Descarga e instalación

### Requisitos del sistema

- Se publican paquetes de escritorio para macOS, Windows y Linux
- Compilar desde el código fuente requiere Node.js 20+, npm 10+, Rust stable y las dependencias de sistema de Tauri para tu plataforma

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### Descarga manual

Descarga desde:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Ejemplos de nombres de archivos para macOS:

- `MCP-Manager-<version>-<arch>.dmg`

Guía de arquitecturas:

- Macs con Apple Silicon: elige los archivos `arm64` o `aarch64`
- Macs con Intel: elige los archivos `x64` o `x86_64`

Después de descargar el archivo de macOS correcto, ábrelo localmente:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Es posible que las versiones públicas actuales para macOS aún no estén firmadas mientras se preparan la firma y la notarización de Apple. Si Gatekeeper bloquea `MCP Manager.app`, sigue este procedimiento alternativo:

1. Arrastra `MCP Manager.app` a `/Applications`.
2. En Finder, haz clic derecho en la aplicación una vez y elige `Open`.
3. Si macOS sigue indicando que la aplicación no se puede abrir o está dañada, elimina el atributo de cuarentena y vuelve a intentarlo:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### Descarga manual

Descarga desde:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Ejemplos de nombres de archivos para Windows:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Guía de arquitecturas:

- Dispositivos Windows on ARM: elige el instalador `arm64`
- PC con Intel / AMD: elige el instalador `x64` o `x86_64`

Guía de paquetes:

- `setup.exe`: disponible tanto para Windows x64 como para Windows arm64
- `.msi`: se publica para las versiones estables de Windows x64

Elige el archivo que corresponda a tu arquitectura y ejecútalo localmente:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# o
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Usuarios de Linux

Descarga la última compilación para Linux desde la página de [Releases](https://github.com/xjeway/mcp-manager/releases/latest):

Ejemplos de nombres de archivos para Linux:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (universal)

Guía ampliada de distribuciones:

- `.deb`: Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS y otras distribuciones basadas en Debian
- `.rpm`: Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE y otras distribuciones basadas en RPM
- `.AppImage`: opción portátil para Linux de escritorio cuando no quieres un paquete del sistema

Guía de arquitecturas:

- `x86_64` o `amd64` para la mayoría de los PC de 64 bits con Intel y AMD
- `arm64` o `aarch64` para dispositivos Linux ARM64, cuando ese archivo se publica en la versión

Instala con el comando correspondiente:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### Compilar desde el código fuente

Instala las dependencias:

```bash
make install
```

Ejecuta la aplicación de escritorio:

```bash
make tauri-dev
```

Ejecuta solo la interfaz web:

```bash
make dev
```

## Inicio rápido

1. Inicia `MCP Manager`.
2. Importa las entradas existentes desde los archivos de configuración locales de los clientes.
3. Edita los servidores en modo formulario o en modo JSON.
4. Aplica la configuración generada a uno o varios clientes compatibles.
5. Revisa las advertencias y luego escribe con protección de copia de seguridad y reversión.

## Clientes compatibles

Todos los clientes siguientes admiten tanto la **importación** (leer los servidores MCP existentes de su configuración) como la **aplicación** (volver a escribir la configuración unificada).

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

## Línea de comandos (vista previa)

Una herramienta de línea de comandos, `mcpmgr`, comparte la lista de servidores de la aplicación de escritorio, de modo que los cambios hechos en una aparecen en la otra. Ejecútala con `npx` o instálala con npm o Homebrew:

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

Desde una copia del repositorio, usa `cargo run --bin mcpmgr -- <command>`.

En una terminal, `add` te guía para elegir los servidores, los clientes y el ámbito de usuario o de proyecto, y después muestra lo que recibirá cada archivo de cliente antes de escribir. Cada paso puede responderse con una opción (`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`); `--dry-run` muestra una vista previa y `--json` imprime una salida legible por máquinas. Si se ejecuta dentro de un agente de programación con IA como Claude Code o Codex, omite las preguntas y apunta al cliente de ese agente.

## Cómo funciona

- La aplicación lee la configuración local de los clientes y la convierte al modelo interno
- La aplicación de cambios escribe una salida específica para cada cliente con actualizaciones atómicas, copia de seguridad y posibilidad de revertir
- El catálogo en línea convierte una entrada del registro (npm, PyPI, imagen de Docker o URL remota) en un borrador de servidor que revisas en el editor; todos los clientes empiezan desactivados

### Acceso a la red

MCP Manager solo se conecta a internet para buscar actualizaciones y, mientras la página del catálogo está abierta, para consultar registros MCP. Los resultados de los registros se almacenan en caché: el catálogo se abre desde la caché y la actualiza en segundo plano pasadas 6 horas, y sigue funcionando sin conexión con los últimos datos. Añadir una fuente en **Ajustes → Fuentes del catálogo** le envía una solicitud de comprobación. Desactiva el catálogo en **Ajustes → Catálogo en línea** para detener por completo esas solicitudes.

## Alcance

El alcance actual se centra en la gestión de la configuración. La gestión del ciclo de vida en tiempo de ejecución, como iniciar y detener procesos, los registros y las comprobaciones de estado, queda intencionadamente fuera del alcance de la v1.

## Estructura del proyecto

```text
mcp-manager/
  src/                aplicación frontend
  src-tauri/          aplicación tauri (comandos expuestos al frontend, catálogo en línea)
  crates/
    mcp-manager-core/ adaptadores de clientes, análisis, escrituras seguras (compartido)
    mcp-manager-cli/  herramienta de línea de comandos (`mcpmgr`)
  public/             recursos estáticos e identidad visual
  docs/               notas de versión y referencias de diseño
  openspec/           seguimiento de cambios y especificaciones
```

### Módulos principales (`crates/mcp-manager-core`)

- `platform`: resolución de rutas según el sistema operativo y contexto del entorno
- `adapters`: lógica de importación y aplicación por cliente
- `core`: modelo de configuración canónico y comportamiento de fusión
- `parser`: análisis y extracción de YAML / JSON / TOML
- `storage`: escritura atómica, copia de seguridad y reversión
- `store`: el `servers.yaml` compartido, con bloqueo y detección de cambios
- `workflow`: importación, vista previa y aplicación en todos los clientes
- `ops`: edición y validación de la lista de servidores, replicadas del frontend

## Desarrollo

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## Automatización de versiones

Este repositorio incluye flujos de trabajo de GitHub Actions para CI y para el empaquetado de versiones de escritorio con la acción oficial de publicación de Tauri.

Consulta [`docs/releasing.md`](../../docs/releasing.md) para:

- publicación de GitHub Releases a partir de etiquetas
- generación de paquetes para macOS, Windows y Linux
- configuración de la firma del actualizador
- firma de código opcional por plataforma
- comandos auxiliares para sincronizar versiones y crear etiquetas de git

## Contribuir

Las incidencias y los pull requests son bienvenidos.

Si planeas contribuir con un cambio no trivial, abre primero una incidencia o una discusión para que el alcance y la dirección queden claros antes de la implementación.

### Desarrollo local

```bash
make install
make tauri-dev
```

### Antes de abrir un PR

```bash
make test
make check
```

Mantén sincronizados [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md) y las traducciones de [`docs/readme/`](./) cuando cambies la documentación del proyecto dirigida a los usuarios.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Aviso legal

Este proyecto se desarrolla principalmente con ayuda de IA (desarrollo impulsado por IA). El código, las pruebas y la documentación se generan en gran parte con IA y los revisa un mantenedor humano, por lo que aún pueden contener errores.

MCP Manager lee y reescribe los archivos de configuración de otras herramientas. Incluye copias de seguridad y reversión, pero aun así deberías hacer tú mismo copias de seguridad de las configuraciones importantes de los clientes, revisar los cambios antes de aplicarlos y usar el software bajo tu propia responsabilidad. Se proporciona "tal cual" bajo la [licencia MIT](../../LICENSE), sin garantía de ningún tipo.

## Licencia

MIT @ xJeway
