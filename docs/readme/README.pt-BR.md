<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Um aplicativo desktop multiplataforma para gerenciar configurações de servidores MCP no Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI e outros.
</p>

<p align="center">
  <a href="https://github.com/xjeway/mcp-manager/stargazers"><img src="https://img.shields.io/github/stars/xjeway/mcp-manager?style=flat&label=%E2%98%85&color=f59e0b" alt="Estrelas no GitHub" /></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-0f172a.svg" alt="Licença MIT" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24c8db.svg" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/React-19-61dafb.svg" alt="React 19" />
  <img src="https://img.shields.io/badge/Rust-stable-f74c00.svg" alt="Rust stable" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-16a34a.svg" alt="Multiplataforma" />
</p>

<p align="center">
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <a href="./README.ko.md">한국어</a> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <b>Português</b> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>Baixar o MCP Manager</ins></a></h3>

## Sumário

- [Recursos](#recursos)
- [Instalação](#instalação)
- [Clientes compatíveis](#clientes-compatíveis)
- [Linha de comando (prévia)](#linha-de-comando-prévia)
- [Privacidade e rede](#privacidade-e-rede)
- [Desenvolvimento](#desenvolvimento)
- [Star History](#star-history)
- [Aviso legal](#aviso-legal)

## Recursos

- **Um único espaço de trabalho** para todos os servidores MCP, importados das configurações que seus clientes já têm
- **Catálogo online**: encontre servidores no GitHub MCP Registry, no MCP Registry oficial ou em qualquer registro compatível, e adicione-os com poucos cliques
- Edição no **modo formulário ou JSON bruto**
- **Aplique a vários clientes de uma vez**, com revisão de gravações arriscadas, backups automáticos e rollback
- Tema claro / escuro e interface em 11 idiomas que segue o idioma do sistema

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, tema claro" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, tema escuro" /></td>
  </tr>
</table>

## Instalação

Baixe o pacote para a sua plataforma na [release mais recente](https://github.com/xjeway/mcp-manager/releases/latest):

| Plataforma | Pacote |
| --- | --- |
| macOS | `.dmg` ou `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) ou `.msi` (x64) |
| Linux | `.deb`, `.rpm` ou `.AppImage` |

Escolha `arm64` / `aarch64` para Apple Silicon e dispositivos ARM; caso contrário, `x64` / `x86_64` / `amd64`.

<details>
<summary>O macOS diz que o app está danificado ou não pode ser aberto?</summary>

As builds de release ainda não são notarizadas. Mova o app para `/Applications` e execute:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## Clientes compatíveis

Todos os clientes abaixo oferecem suporte tanto a **importação** (ler os servidores MCP existentes da sua configuração) quanto a **aplicação** (gravar de volta a configuração unificada).

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="../../public/logos/apps/claudecode.svg" alt="Logo do Claude Code" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="../../public/logos/apps/claude.svg" alt="Logo do Claude Desktop" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="../../public/logos/apps/codex.svg" alt="Logo do Codex" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="../../public/logos/apps/cursor.svg" alt="Logo do Cursor" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="../../public/logos/apps/opencode.svg" alt="Logo do OpenCode" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="../../public/logos/apps/githubcopilot.svg" alt="Logo do GitHub Copilot" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="../../public/logos/apps/gemini.svg" alt="Logo do Gemini CLI" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="../../public/logos/apps/antigravity.svg" alt="Logo do Antigravity" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="../../public/logos/apps/iflow.svg" alt="Logo do iFlow" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="../../public/logos/apps/qwen.svg" alt="Logo do Qwen Code" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="../../public/logos/apps/cline.svg" alt="Logo do Cline" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="../../public/logos/apps/windsurf.svg" alt="Logo do Windsurf" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../public/logos/apps/kiro.svg" alt="Logo do Kiro" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../public/logos/apps/qoder.svg" alt="Logo do Qoder" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../public/logos/apps/vscode.svg" alt="Logo do VS Code" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## Linha de comando (prévia)

O `mcpmgr` compartilha a lista de servidores do aplicativo desktop, então as alterações feitas em um aparecem no outro.

```bash
npx @mcpmgr/cli list                      # ou: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

Em um terminal, `add` guia você na escolha dos servidores, dos clientes e do escopo, e mostra uma prévia de cada arquivo antes de gravar. Cada etapa também tem uma flag (`--yes`, `--dry-run`, `--json`, …); dentro de um agente de programação com IA, como Claude Code ou Codex, ele pula as perguntas e tem como alvo o cliente desse agente.

## Privacidade e rede

O MCP Manager só se conecta à internet para verificar atualizações e, enquanto a página do catálogo está aberta, para consultar registros MCP (em cache por 6 horas, utilizável offline). Desative o catálogo em **Configurações → Catálogo online** para interromper totalmente essas solicitações.

## Desenvolvimento

Requer Node.js 20+, Rust stable e os [pré-requisitos do Tauri](https://tauri.app/start/prerequisites/) para a sua plataforma.

```bash
make install      # instala as dependências
make tauri-dev    # executa o aplicativo desktop
make test && make check
```

A lógica central (adaptadores de clientes, parsing, gravações seguras) fica em `crates/mcp-manager-core`, compartilhada pelo app Tauri em `src-tauri/` e pela CLI em `crates/mcp-manager-cli/`. As releases são descritas em [`docs/releasing.md`](../../docs/releasing.md).

Issues e pull requests são bem-vindos; para mudanças maiores, abra primeiro uma issue. Ao alterar a documentação voltada aos usuários, mantenha o [`README.md`](../../README.md), o [`README.zh-CN.md`](../../README.zh-CN.md) e [`docs/readme/`](./) sincronizados.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Gráfico do Star History" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Aviso legal

Este projeto é desenvolvido principalmente com assistência de IA e revisado por um mantenedor humano, portanto ainda pode conter erros. O MCP Manager reescreve os arquivos de configuração de outras ferramentas; backups e rollback já vêm integrados, mas mantenha seus próprios backups das configurações importantes. Fornecido "no estado em que se encontra" sob a [Licença MIT](../../LICENSE).
