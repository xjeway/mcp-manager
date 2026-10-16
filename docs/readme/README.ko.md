<h1 align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="80" valign="middle" /> MCP Manager
</h1>

<p align="center">
  Claude Code, Codex, Cursor, VS Code, OpenCode, Gemini CLI 등 여러 클라이언트의 MCP 서버 설정을 한곳에서 관리하는 크로스 플랫폼 데스크톱 앱입니다.
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
  <sub><a href="../../README.md">English</a> · <a href="../../README.zh-CN.md">简体中文</a> · <a href="./README.zh-TW.md">繁體中文</a> · <a href="./README.ja.md">日本語</a> · <b>한국어</b> · <a href="./README.de.md">Deutsch</a> · <a href="./README.fr.md">Français</a> · <a href="./README.es.md">Español</a> · <a href="./README.it.md">Italiano</a> · <a href="./README.pt-BR.md">Português</a> · <a href="./README.ru.md">Русский</a></sub>
</p>

<h3 align="center"><a href="https://github.com/xjeway/mcp-manager/releases/latest"><ins>MCP Manager 다운로드</ins></a></h3>

## 목차

- [기능](#기능)
- [설치](#설치)
- [지원 클라이언트](#지원-클라이언트)
- [명령줄 (미리 보기)](#명령줄-미리-보기)
- [개인정보 및 네트워크](#개인정보-및-네트워크)
- [개발](#개발)
- [Star History](#star-history)
- [면책 조항](#면책-조항)

## 기능

- 모든 MCP 서버를 위한 **하나의 워크스페이스**, 클라이언트에 이미 있는 설정에서 가져오기
- **온라인 마켓플레이스**: GitHub MCP Registry, 공식 MCP Registry 또는 호환되는 모든 레지스트리에서 서버를 찾아 몇 번의 클릭으로 추가
- **폼 또는 원본 JSON**으로 편집
- 위험한 쓰기 작업 검토, 자동 백업, 롤백과 함께 **여러 클라이언트에 한 번에 적용**
- 라이트 / 다크 테마, 그리고 시스템 언어를 따르는 11개 언어 UI

<table>
  <tr>
    <td><img src="../../docs/page-en.png" alt="MCP Manager, light theme" /></td>
    <td><img src="../../docs/page-en-dark.png" alt="MCP Manager, dark theme" /></td>
  </tr>
</table>

## 설치

[최신 릴리스](https://github.com/xjeway/mcp-manager/releases/latest)에서 사용 중인 플랫폼에 맞는 패키지를 다운로드하세요:

| 플랫폼 | 패키지 |
| --- | --- |
| macOS | `.dmg` 또는 `brew install --cask xjeway/mcp-manager/mcp-manager` |
| Windows | `setup.exe` (x64 / arm64) 또는 `.msi` (x64) |
| Linux | `.deb`, `.rpm` 또는 `.AppImage` |

Apple Silicon 및 ARM 기기에서는 `arm64` / `aarch64`를, 그 외에는 `x64` / `x86_64` / `amd64`를 선택하세요.

<details>
<summary>macOS에서 앱이 손상되었거나 열 수 없다고 표시되나요?</summary>

릴리스 빌드는 아직 공증되지 않았습니다. 앱을 `/Applications`로 옮긴 뒤 다음을 실행하세요:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
```

</details>

## 지원 클라이언트

아래의 모든 클라이언트는 **가져오기**(설정에서 기존 MCP 서버 읽기)와 **적용**(통합 설정을 다시 기록)을 모두 지원합니다.

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

## 명령줄 (미리 보기)

`mcpmgr`는 데스크톱 앱과 서버 목록을 공유하므로, 어느 한쪽에서 변경한 내용이 다른 쪽에도 반영됩니다.

```bash
npx @mcpmgr/cli list                      # 또는: npm i -g @mcpmgr/cli / brew install xjeway/mcp-manager/mcpmgr

mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
mcpmgr remove context7 -a codex
mcpmgr rollback
```

터미널에서 `add`를 실행하면 서버, 클라이언트, 범위를 차례로 선택하도록 안내하고, 기록하기 전에 모든 파일을 미리 보여 줍니다. 각 단계에는 플래그(`--yes`, `--dry-run`, `--json`, …)도 있으며, Claude Code나 Codex 같은 AI 코딩 에이전트 안에서 실행하면 프롬프트를 건너뛰고 해당 에이전트의 클라이언트를 대상으로 합니다.

## 개인정보 및 네트워크

MCP Manager는 업데이트를 확인할 때와, 마켓플레이스 페이지가 열려 있는 동안 MCP 레지스트리를 조회할 때(6시간 동안 캐시되며 오프라인에서도 사용 가능)만 온라인에 접속합니다. 이러한 요청을 완전히 중단하려면 **설정 → 온라인 마켓플레이스**에서 마켓플레이스를 끄세요.

## 개발

Node.js 20+, Rust stable 및 사용 중인 플랫폼에 맞는 [Tauri 사전 요구 사항](https://tauri.app/start/prerequisites/)이 필요합니다.

```bash
make install      # 의존성 설치
make tauri-dev    # 데스크톱 앱 실행
make test && make check
```

핵심 로직(클라이언트 어댑터, 파싱, 안전한 쓰기)은 `crates/mcp-manager-core`에 있으며, `src-tauri/`의 Tauri 앱과 `crates/mcp-manager-cli/`의 CLI가 이를 공유합니다. 릴리스 절차는 [`docs/releasing.md`](../../docs/releasing.md)에 설명되어 있습니다.

이슈와 풀 리퀘스트를 환영합니다. 규모가 큰 변경은 먼저 이슈를 열어 주세요. 사용자에게 보이는 문서를 변경할 때는 [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md), [`docs/readme/`](./)를 서로 동기화된 상태로 유지하세요.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 면책 조항

이 프로젝트는 주로 AI의 도움을 받아 개발되고 사람 메인테이너가 검토하므로 여전히 오류가 있을 수 있습니다. MCP Manager는 다른 도구의 설정 파일을 다시 씁니다. 백업과 롤백 기능이 내장되어 있지만, 중요한 설정은 직접 백업해 두세요. [MIT License](../../LICENSE)에 따라 "있는 그대로" 제공됩니다.
