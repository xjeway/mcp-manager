<p align="center">
  <img src="../../src-tauri/icons/app-icon-rounded.png" alt="MCP Manager logo" width="160" height="160" />
</p>

<h1 align="center">MCP Manager</h1>

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

- [MCP Manager를 쓰는 이유](#mcp-manager를-쓰는-이유)
- [스크린샷](#스크린샷)
- [다운로드 및 설치](#다운로드-및-설치)
  - [macOS](#macos) · [Windows](#windows) · [Linux](#linux-사용자) · [소스에서 빌드](#소스에서-빌드)
- [빠른 시작](#빠른-시작)
- [지원 클라이언트](#지원-클라이언트)
- [명령줄 (미리 보기)](#명령줄-미리-보기)
- [작동 방식](#작동-방식)
- [범위](#범위)
- [프로젝트 구조](#프로젝트-구조)
- [개발](#개발)
- [릴리스 자동화](#릴리스-자동화)
- [기여하기](#기여하기)
- [Star History](#star-history)
- [면책 조항](#면책-조항)
- [라이선스](#라이선스)

## MCP Manager를 쓰는 이유

`MCP Manager`는 여러 MCP 클라이언트 설정을 손으로 직접 편집하고 싶지 않은 사람을 위한 크로스 플랫폼 데스크톱 앱입니다.

- 설정된 모든 서버를 하나의 통합 MCP 워크스페이스에서 관리
- 로컬 클라이언트 설정에서 기존 항목 가져오기
- GitHub MCP Registry, 공식 MCP Registry, 또는 MCP Registry API를 구현한 원하는 레지스트리에서 서버를 찾아 몇 번의 클릭으로 추가
- 폼 모드 또는 원본 JSON 모드로 서버 편집
- 생성된 설정을 지원되는 여러 클라이언트에 적용
- 파일이 변경되기 전에 위험한 쓰기 작업 검토
- 적용 중 백업 및 롤백 지원
- 라이트 모드, 다크 모드 또는 시스템 테마 사용
- 영어, 중국어 간체, 중국어 번체, 일본어, 한국어, 독일어, 프랑스어, 스페인어, 이탈리아어, 브라질 포르투갈어, 러시아어 UI 지원 (기본적으로 시스템 언어를 따름)

## 스크린샷

![MCP Manager interface in English, light theme](../../docs/page-en.png)

![MCP Manager interface in English, dark theme](../../docs/page-en-dark.png)

## 다운로드 및 설치

### 시스템 요구 사항

- 데스크톱 패키지는 macOS, Windows, Linux용으로 배포됩니다
- 소스에서 빌드하려면 Node.js 20+, npm 10+, Rust stable 및 사용 중인 플랫폼에 맞는 Tauri 시스템 의존성이 필요합니다

### macOS

#### Homebrew

```bash
brew tap xjeway/mcp-manager
brew install --cask mcp-manager
brew update
brew upgrade --cask mcp-manager
```

#### 수동 다운로드

다운로드 위치:

- <https://github.com/xjeway/mcp-manager/releases/latest>

macOS 에셋 이름 예시:

- `MCP-Manager-<version>-<arch>.dmg`

아키텍처 안내:

- Apple Silicon Mac: `arm64` 또는 `aarch64` 에셋을 선택하세요
- Intel Mac: `x64` 또는 `x86_64` 에셋을 선택하세요

올바른 macOS 에셋을 다운로드한 뒤 로컬에서 엽니다:

```bash
open ~/Downloads/MCP-Manager*.dmg
```

Apple 서명 및 공증을 준비하는 동안 현재 공개된 macOS 릴리스는 아직 서명되지 않았을 수 있습니다. Gatekeeper가 `MCP Manager.app`을 차단하면 다음 대체 절차를 따르세요:

1. `MCP Manager.app`을 `/Applications`로 드래그합니다.
2. Finder에서 앱을 한 번 마우스 오른쪽 버튼으로 클릭하고 `Open`을 선택합니다.
3. 그래도 macOS가 앱을 열 수 없거나 손상되었다고 표시하면 격리(quarantine) 플래그를 제거한 뒤 다시 시도합니다:

```bash
xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"
open "/Applications/MCP Manager.app"
```

### Windows

#### 수동 다운로드

다운로드 위치:

- <https://github.com/xjeway/mcp-manager/releases/latest>

Windows 에셋 이름 예시:

- `MCP-Manager-<version>-<arch>-setup.exe`
- `MCP-Manager-<version>-x64.msi`

아키텍처 안내:

- Windows on ARM 기기: `arm64` 설치 프로그램 에셋을 선택하세요
- Intel / AMD PC: `x64` 또는 `x86_64` 설치 프로그램 에셋을 선택하세요

패키지 안내:

- `setup.exe`: Windows x64와 Windows arm64 모두 제공
- `.msi`: Windows x64 정식 릴리스용으로 배포

아키텍처에 맞는 에셋을 선택한 뒤 로컬에서 실행합니다:

```powershell
Start-Process "C:\Path\To\MCP-Manager-Setup.exe"
# 또는
msiexec /i "C:\Path\To\MCP-Manager.msi"
```

### Linux 사용자

[Releases](https://github.com/xjeway/mcp-manager/releases/latest) 페이지에서 최신 Linux 빌드를 다운로드하세요:

Linux 에셋 이름 예시:

- `mcp-manager_<version>_<arch>.deb` (Debian/Ubuntu)
- `mcp-manager-<version>.<arch>.rpm` (Fedora/RHEL/openSUSE)
- `MCP-Manager-<version>-<arch>.AppImage` (범용)

배포판별 상세 안내:

- `.deb`: Debian, Ubuntu, Linux Mint, Pop!_OS, elementary OS 및 기타 Debian 기반 배포판
- `.rpm`: Fedora, RHEL, Rocky Linux, AlmaLinux, openSUSE 및 기타 RPM 기반 배포판
- `.AppImage`: 시스템 패키지를 원하지 않을 때 데스크톱 Linux에서 쓸 수 있는 포터블 옵션

아키텍처 안내:

- 대부분의 Intel 및 AMD 64비트 PC는 `x86_64` 또는 `amd64`
- ARM64 Linux 기기는 `arm64` 또는 `aarch64` (해당 릴리스에 이 에셋이 배포된 경우)

맞는 명령으로 설치합니다:

```bash
# Debian / Ubuntu
sudo dpkg -i ./mcp-manager_<version>_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i ./mcp-manager-<version>.x86_64.rpm

# AppImage
chmod +x ./MCP-Manager-<version>-<arch>.AppImage
./MCP-Manager-<version>-<arch>.AppImage
```

### 소스에서 빌드

의존성 설치:

```bash
make install
```

데스크톱 앱 실행:

```bash
make tauri-dev
```

웹 UI만 실행:

```bash
make dev
```

## 빠른 시작

1. `MCP Manager`를 실행합니다.
2. 로컬 클라이언트 설정 파일에서 기존 항목을 가져옵니다.
3. 폼 모드 또는 JSON 모드로 서버를 편집합니다.
4. 생성된 설정을 지원되는 하나 이상의 클라이언트에 적용합니다.
5. 경고를 검토한 뒤 백업 및 롤백 보호와 함께 기록합니다.

## 지원 클라이언트

아래의 모든 클라이언트는 **가져오기**(설정에서 기존 MCP 서버 읽기)와 **적용**(통합 설정을 다시 기록)을 모두 지원합니다.

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

## 명령줄 (미리 보기)

명령줄 도구 `mcpmgr`는 데스크톱 앱과 서버 목록을 공유하므로, 어느 한쪽에서 변경한 내용이 다른 쪽에도 반영됩니다. `npx`로 실행하거나 npm 또는 Homebrew로 설치하세요:

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

체크아웃한 소스에서는 `cargo run --bin mcpmgr -- <command>`를 사용하세요.

터미널에서 `add`를 실행하면 서버, 클라이언트, 사용자 또는 프로젝트 범위를 차례로 선택하도록 안내하고, 기록하기 전에 각 클라이언트 파일에 어떤 내용이 들어갈지 보여 줍니다. 모든 단계는 플래그(`--server`, `--env`, `--app`, `--global`/`--project`, `--yes`)로 답할 수 있으며, `--dry-run`은 미리 보기를, `--json`은 기계가 읽을 수 있는 출력을 제공합니다. Claude Code나 Codex 같은 AI 코딩 에이전트 안에서 실행하면 프롬프트를 건너뛰고 해당 에이전트의 클라이언트를 대상으로 합니다.

## 작동 방식

- 앱은 로컬 클라이언트 설정을 읽어 내부 모델로 변환합니다
- 적용 시 원자적 업데이트, 백업, 롤백 지원과 함께 클라이언트별 출력을 기록합니다
- 온라인 마켓플레이스는 레지스트리 항목(npm, PyPI, Docker 이미지 또는 원격 URL)을 편집기에서 검토할 수 있는 서버 초안으로 변환하며, 모든 클라이언트는 꺼진 상태로 시작합니다

### 네트워크 접근

MCP Manager는 업데이트를 확인할 때와, 마켓플레이스 페이지가 열려 있는 동안 MCP 레지스트리를 조회할 때만 온라인에 접속합니다. 레지스트리 결과는 캐시됩니다. 마켓플레이스는 캐시에서 열리고 6시간이 지나면 백그라운드에서 캐시를 업데이트하며, 오프라인에서도 마지막 데이터로 계속 동작합니다. **설정 → 마켓플레이스 소스**에서 소스를 추가하면 해당 소스로 확인 요청을 한 번 보냅니다. 이러한 요청을 완전히 중단하려면 **설정 → 온라인 마켓플레이스**에서 마켓플레이스를 끄세요.

## 범위

현재 범위는 설정 관리에 집중되어 있습니다. 프로세스 시작, 중지, 로그, 상태 확인과 같은 런타임 생명주기 관리는 v1에서 의도적으로 범위에서 제외했습니다.

## 프로젝트 구조

```text
mcp-manager/
  src/                프런트엔드 애플리케이션
  src-tauri/          tauri 앱 (프런트엔드에 노출되는 명령, 온라인 마켓플레이스)
  crates/
    mcp-manager-core/ 클라이언트 어댑터, 파싱, 안전한 쓰기 (공유)
    mcp-manager-cli/  명령줄 도구 (`mcpmgr`)
  public/             정적 에셋 및 브랜딩
  docs/               릴리스 노트 및 디자인 참고 자료
  openspec/           변경 및 사양 추적
```

### 핵심 모듈 (`crates/mcp-manager-core`)

- `platform`: OS를 인식하는 경로 해석 및 환경 컨텍스트
- `adapters`: 클라이언트별 가져오기 및 적용 로직
- `core`: 표준 설정 모델 및 병합 동작
- `parser`: YAML / JSON / TOML 파싱 및 추출
- `storage`: 원자적 쓰기, 백업, 롤백
- `store`: 잠금 및 변경 감지를 갖춘 공유 `servers.yaml`
- `workflow`: 모든 클라이언트에 걸친 가져오기, 미리 보기, 적용
- `ops`: 프런트엔드와 동일하게 구현된 서버 목록 편집 및 검증

## 개발

```bash
make install
make dev
make tauri-dev
make build
make test
make check
make tauri-build
```

## 릴리스 자동화

이 저장소에는 CI와, 공식 Tauri 릴리스 액션을 이용한 데스크톱 릴리스 패키징을 위한 GitHub Actions 워크플로가 포함되어 있습니다.

다음 내용은 [`docs/releasing.md`](../../docs/releasing.md)를 참고하세요:

- 태그 기반 GitHub Release 게시
- macOS, Windows, Linux 패키지 생성
- 업데이터 서명 설정
- 선택적 플랫폼 코드 서명
- 버전 동기화 및 git 태그 헬퍼 명령

## 기여하기

이슈와 풀 리퀘스트를 환영합니다.

규모가 큰 변경을 기여하려는 경우, 구현 전에 범위와 방향을 명확히 할 수 있도록 먼저 이슈나 토론을 열어 주세요.

### 로컬 개발

```bash
make install
make tauri-dev
```

### PR을 열기 전에

```bash
make test
make check
```

사용자에게 보이는 프로젝트 문서를 변경할 때는 [`README.md`](../../README.md), [`README.zh-CN.md`](../../README.zh-CN.md), 그리고 [`docs/readme/`](./)의 번역본을 서로 동기화된 상태로 유지하세요.

## Star History

<a href="https://star-history.com/#xjeway/mcp-manager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=xjeway/mcp-manager&type=Date" />
  </picture>
</a>

## 면책 조항

이 프로젝트는 주로 AI의 도움을 받아(AI 주도 개발) 개발됩니다. 코드, 테스트, 문서는 대부분 AI가 생성하고 사람 메인테이너가 검토하므로 여전히 오류가 있을 수 있습니다.

MCP Manager는 다른 도구의 설정 파일을 읽고 다시 씁니다. 백업과 롤백 기능이 내장되어 있지만, 중요한 클라이언트 설정은 직접 백업하고, 적용하기 전에 변경 사항을 검토하며, 본인의 책임하에 소프트웨어를 사용하세요. 이 소프트웨어는 [MIT License](../../LICENSE)에 따라 어떠한 보증도 없이 "있는 그대로" 제공됩니다.

## 라이선스

MIT @ xJeway
