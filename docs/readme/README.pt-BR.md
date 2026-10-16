<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="Logo do MCP Manager" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [Por que o MCP Manager](#por-que-o-mcp-manager)
- [Capturas de tela](#capturas-de-tela)
- [Download e instalação](#download-e-instalação)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#usuários-de-linux) · [Compilar a partir do código-fonte](#compilar-a-partir-do-código-fonte)
- [Início rápido](#início-rápido)
- [Clientes compatíveis](#clientes-compatíveis)
- [Linha de comando (prévia)](#linha-de-comando-prévia)
- [Como funciona](#como-funciona)
- [Escopo](#escopo)
- [Estrutura do projeto](#estrutura-do-projeto)
- [Desenvolvimento](#desenvolvimento)
- [Automação de releases](#automação-de-releases)
- [Contribuindo](#contribuindo)
- [Star History](#star-history)
- [Aviso legal](#aviso-legal)
- [Licença](#licença)

## Por que o MCP Manager

O `MCP Manager` é um aplicativo desktop multiplataforma para quem não quer editar à mão as configurações de vários clientes MCP.

- Mantenha um espaço de trabalho MCP unificado para todos os servidores configurados
- Importe entradas existentes da configuração local dos clientes
- Encontre servidores no GitHub MCP Registry, no MCP Registry oficial ou em qualquer registro que você adicionar e que implemente a API do MCP Registry, e adicione-os com poucos cliques
- Edite servidores no modo formulário ou no modo JSON bruto
- Aplique a configuração gerada a vários clientes compatíveis
- Revise gravações arriscadas antes que os arquivos sejam alterados
- Mantenha backups e suporte a rollback durante a aplicação
- Use o tema claro, o tema escuro ou o tema do sistema
- Use a interface em inglês, chinês simplificado, chinês tradicional, japonês, coreano, alemão, francês, espanhol, italiano, português do Brasil ou russo (por padrão, segue o idioma do sistema)

## Capturas de tela

![Interface do MCP Manager em inglês, tema claro](../../docs/page-en.png)

![Interface do MCP Manager em inglês, tema escuro](../../docs/page-en-dark.png)

## Download e instalação

### Requisitos do sistema

- Os pacotes desktop são publicados para macOS, Windows e Linux
- Compilar a partir do código-fonte requer Node.js 20+, npm 10+, Rust stable e as dependências de sistema do Tauri para a sua plataforma

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### Download manual

Baixe em:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Exemplos de nomes de arquivos para macOS:

- `MCP-Manager-<version>-<arch>.dmg`

Guia de arquitetura:

- Macs com Apple Silicon: escolha os arquivos `arm64` ou `aarch64`
- Macs com Intel: escolha os arquivos `x64` ou `x86_64`

Depois de baixar o arquivo correto para macOS, abra-o localmente:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

As versões públicas atuais para macOS ainda podem não estar assinadas enquanto a assinatura e a notarização da Apple estão sendo preparadas. Se o Gatekeeper bloquear o `MCP Manager.app`, use este procedimento alternativo:

1. Arraste o `MCP Manager.app` para `/Applications`.
2. No Finder, clique com o botão direito no app uma vez e escolha `Open`.
3. Se o macOS ainda disser que o app não pode ser aberto ou está danificado, remova o atributo de quarentena e tente novamente:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### Download manual

Baixe em:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Exemplos de nomes de arquivos para Windows:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

Guia de arquitetura:

- Dispositivos Windows on ARM: escolha o instalador `arm64`
- PCs Intel / AMD: escolha o instalador `x64` ou `x86_64`

Guia de pacotes:

- `setup.exe`: disponível para Windows x64 e Windows arm64
- `.msi`: publicado para as versões estáveis de Windows x64

Escolha o arquivo que corresponde à sua arquitetura e execute-o localmente:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# ou
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Usuários de Linux

Baixe a build mais recente para Linux na página de [Releases](https://github.com/xjeway/mcp-manager/releases/latest):

Exemplos de nomes de arquivos para Linux:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (Universal)

Guia detalhado por distribuição:

- `.deb`: Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS e outras distribuições baseadas em Debian
- `.rpm`: Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE e outras distribuições baseadas em RPM
- `.AppImage`: opção portátil para Linux desktop quando você não quer um pacote do sistema

Guia de arquitetura:

- `x86_64` ou `amd64` para a maioria dos PCs Intel e AMD de 64 bits
- `arm64` ou `aarch64` para dispositivos Linux ARM64, quando esse arquivo for publicado na release

Instale com o comando correspondente:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### Compilar a partir do código-fonte

Instale as dependências:

```bash
make install
```

Execute o aplicativo desktop:

```bash
make tauri-dev
```

Execute apenas a interface web:

```bash
make dev
```

## Início rápido

1. Abra o `MCP Manager`.
2. Importe entradas existentes dos arquivos de configuração locais dos clientes.
3. Edite servidores no modo formulário ou no modo JSON.
4. Aplique a configuração gerada a um ou mais clientes compatíveis.
5. Revise os avisos e grave com proteção de backup e rollback.

## Clientes compatíveis

Todos os clientes abaixo oferecem suporte tanto a **importação** (ler os servidores MCP existentes da sua configuração) quanto a **aplicação** (gravar de volta a configuração unificada).

<p>
  <a href="https://docs.anthropic.com/en/docs/claude-code"><kbd><img src="../../docs/assets/clients/claudecode.svg" alt="Logo do Claude Code" width="16" valign="middle" /> Claude Code</kbd></a> &nbsp;
  <a href="https://claude.ai/download"><kbd><img src="../../docs/assets/clients/claude.svg" alt="Logo do Claude Desktop" width="16" valign="middle" /> Claude Desktop</kbd></a> &nbsp;
  <a href="https://github.com/openai/codex"><kbd><img src="../../docs/assets/clients/codex.svg" alt="Logo do Codex" width="16" valign="middle" /> Codex</kbd></a> &nbsp;
  <a href="https://cursor.com"><kbd><img src="../../docs/assets/clients/cursor.svg" alt="Logo do Cursor" width="16" valign="middle" /> Cursor</kbd></a> &nbsp;
  <a href="https://opencode.ai"><kbd><img src="../../docs/assets/clients/opencode.svg" alt="Logo do OpenCode" width="16" valign="middle" /> OpenCode</kbd></a> &nbsp;
  <a href="https://github.com/features/copilot"><kbd><img src="../../docs/assets/clients/githubcopilot.svg" alt="Logo do GitHub Copilot" width="16" valign="middle" /> GitHub Copilot</kbd></a> &nbsp;
  <a href="https://github.com/google-gemini/gemini-cli"><kbd><img src="../../docs/assets/clients/gemini.svg" alt="Logo do Gemini CLI" width="16" valign="middle" /> Gemini CLI</kbd></a> &nbsp;
  <a href="https://antigravity.google"><kbd><img src="../../docs/assets/clients/antigravity.svg" alt="Logo do Antigravity" width="16" valign="middle" /> Antigravity</kbd></a> &nbsp;
  <a href="https://github.com/iflow-ai/iflow-cli"><kbd><img src="../../docs/assets/clients/iflow.svg" alt="Logo do iFlow" width="16" valign="middle" /> iFlow</kbd></a> &nbsp;
  <a href="https://github.com/QwenLM/qwen-code"><kbd><img src="../../docs/assets/clients/qwen.svg" alt="Logo do Qwen Code" width="16" valign="middle" /> Qwen Code</kbd></a> &nbsp;
  <a href="https://cline.bot"><kbd><img src="../../docs/assets/clients/cline.svg" alt="Logo do Cline" width="16" valign="middle" /> Cline</kbd></a> &nbsp;
  <a href="https://windsurf.com"><kbd><img src="../../docs/assets/clients/windsurf.svg" alt="Logo do Windsurf" width="16" valign="middle" /> Windsurf</kbd></a> &nbsp;
  <a href="https://kiro.dev"><kbd><img src="../../docs/assets/clients/kiro.svg" alt="Logo do Kiro" width="16" valign="middle" /> Kiro</kbd></a> &nbsp;
  <a href="https://qoder.com"><kbd><img src="../../docs/assets/clients/qoder.svg" alt="Logo do Qoder" width="16" valign="middle" /> Qoder</kbd></a> &nbsp;
  <a href="https://code.visualstudio.com"><kbd><img src="../../docs/assets/clients/vscode.svg" alt="Logo do VS Code" width="16" valign="middle" /> VS Code</kbd></a> &nbsp;
</p>

## Linha de comando (prévia)

Uma ferramenta de linha de comando, `mcpmgr`, compartilha a lista de servidores do aplicativo desktop, então as alterações feitas em um aparecem no outro. Execute-a com `npx` ou instale-a com npm ou Homebrew:

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

A partir de um checkout do repositório, use `cargo run --bin mcpmgr -- <command>`.

Em um terminal, `add` guia você na escolha dos servidores, dos clientes e do escopo de usuário ou de projeto, e depois mostra o que cada arquivo de cliente vai receber antes de gravar. Cada etapa pode ser respondida com uma flag (`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`); `--dry-run` mostra uma prévia e `--json` imprime uma saída legível por máquina. Quando executado dentro de um agente de programação com IA, como Claude Code ou Codex, ele pula as perguntas e tem como alvo o cliente desse agente.

## Como funciona

- O aplicativo lê a configuração local dos clientes e a converte para o modelo interno
- A aplicação grava a saída específica de cada cliente com atualizações atômicas, backup e suporte a rollback
- O catálogo online transforma uma entrada de registro (npm, PyPI, imagem Docker ou URL remota) em um rascunho de servidor que você revisa no editor; todos os clientes começam desativados

### Acesso à rede

O MCP Manager só se conecta à internet para verificar atualizações e, enquanto a página do catálogo está aberta, para consultar registros MCP. Os resultados dos registros ficam em cache: o catálogo abre a partir do cache e o atualiza em segundo plano após 6 horas, e continua funcionando offline com os últimos dados. Adicionar uma fonte em **Configurações → Fontes do catálogo** envia uma única solicitação de verificação para ela. Desative o catálogo em **Configurações → Catálogo online** para interromper totalmente essas solicitações.

## Escopo

O escopo atual é focado no gerenciamento de configuração. O gerenciamento do ciclo de vida em tempo de execução, como iniciar e parar processos, logs e verificações de integridade, está intencionalmente fora do escopo da v1.

## Estrutura do projeto

```text
mcp-manager/
  src/                aplicação frontend
  src-tauri/          app tauri (comandos expostos ao frontend, catálogo online)
  crates/
    mcp-manager-core/ adaptadores de clientes, parsing, gravações seguras (compartilhado)
    mcp-manager-cli/  ferramenta de linha de comando (`mcpmgr`)
  public/             assets estáticos e identidade visual
  docs/               notas de release e referências de design
  openspec/           acompanhamento de mudanças e especificações
```

### Módulos principais (`crates/mcp-manager-core`)

- `platform`: resolução de caminhos de acordo com o sistema operacional e contexto do ambiente
- `adapters`: lógica de importação e aplicação por cliente
- `core`: modelo de configuração canônico e comportamento de mesclagem
- `parser`: parsing e extração de YAML / JSON / TOML
- `storage`: gravação atômica, backup e rollback
- `store`: o `servers.yaml` compartilhado, com bloqueio e detecção de alterações
- `workflow`: importação, prévia e aplicação em todos os clientes
- `ops`: edições e validação da lista de servidores, espelhadas do frontend

## Desenvolvimento

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## Automação de releases

Este repositório inclui workflows do GitHub Actions para CI e para o empacotamento das releases desktop com a action oficial de release do Tauri.

Consulte [`docs/releasing.md`](../../docs/releasing.md) para:

- publicação de GitHub Releases a partir de tags
- geração de pacotes para macOS, Windows e Linux
- configuração da assinatura do atualizador
- assinatura de código opcional por plataforma
- comandos auxiliares para sincronizar versões e criar tags git

## Contribuindo

Issues e pull requests são bem-vindos.

Se você pretende contribuir com uma mudança não trivial, abra primeiro uma issue ou discussão para que o escopo e a direção fiquem claros antes da implementação.

### Desenvolvimento local

```bash
make install
make tauri-dev
```

### Antes de abrir um PR

```bash
make test
make check
```

Mantenha o [`README.md`](../../README.md), o [`README.zh-CN.md`](../../README.zh-CN.md) e as traduções em [`docs/readme/`](./) sincronizados ao alterar a documentação do projeto voltada aos usuários.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Gráfico do Star History" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## Aviso legal

Este projeto é desenvolvido principalmente com assistência de IA (desenvolvimento orientado por IA). Código, testes e documentação são em grande parte gerados por IA e revisados por um mantenedor humano, portanto ainda podem conter erros.

O MCP Manager lê e reescreve os arquivos de configuração de outras ferramentas. Backups e rollback já vêm integrados, mas você ainda deve fazer seu próprio backup das configurações importantes dos clientes, revisar as alterações antes de aplicá-las e usar o software por sua conta e risco. Ele é fornecido "no estado em que se encontra" sob a [Licença MIT](../../LICENSE), sem garantia de qualquer tipo.

## Licença

MIT @ xJeway
