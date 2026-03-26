use crate::core::SupportedApp;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformOs {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PlatformContext {
    pub os: PlatformOs,
    pub home_dir: PathBuf,
    pub workspace_root: PathBuf,
}

impl PlatformContext {
    pub fn current() -> Self {
        let os = if cfg!(target_os = "macos") {
            PlatformOs::MacOS
        } else if cfg!(target_os = "linux") {
            PlatformOs::Linux
        } else if cfg!(target_os = "windows") {
            PlatformOs::Windows
        } else {
            PlatformOs::Unknown
        };

        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let home_dir = dirs::home_dir().unwrap_or_else(|| workspace_root.clone());

        Self {
            os,
            home_dir,
            workspace_root,
        }
    }

    pub fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            return self.home_dir.join(stripped);
        }

        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            return candidate;
        }

        self.workspace_root.join(candidate)
    }

    pub fn workspace_file(&self, relative: &str) -> PathBuf {
        self.workspace_root.join(relative)
    }

    pub fn user_app_config_path(&self, app: SupportedApp) -> PathBuf {
        match app {
            SupportedApp::Vscode => match self.os {
                PlatformOs::MacOS => self
                    .home_dir
                    .join("Library/Application Support/Code/User/mcp.json"),
                PlatformOs::Windows => self.home_dir.join("AppData/Roaming/Code/User/mcp.json"),
                _ => self.home_dir.join(".config/Code/User/mcp.json"),
            },
            SupportedApp::Cursor => match self.os {
                PlatformOs::Windows => self.home_dir.join("AppData/Roaming/Cursor/User/mcp.json"),
                _ => self.home_dir.join(".cursor/mcp.json"),
            },
            SupportedApp::ClaudeCode => self.home_dir.join(".claude.json"),
            SupportedApp::ClaudeDesktop => match self.os {
                PlatformOs::MacOS => self
                    .home_dir
                    .join("Library/Application Support/Claude/claude_desktop_config.json"),
                PlatformOs::Windows => self
                    .home_dir
                    .join("AppData/Roaming/Claude/claude_desktop_config.json"),
                _ => self
                    .home_dir
                    .join(".config/Claude/claude_desktop_config.json"),
            },
            SupportedApp::Codex => self.home_dir.join(".codex/config.toml"),
            SupportedApp::OpenCode => self.home_dir.join(".config/opencode/opencode.json"),
            SupportedApp::GithubCopilot => self.home_dir.join(".copilot/mcp-config.json"),
            SupportedApp::GeminiCli => self.home_dir.join(".gemini/settings.json"),
            SupportedApp::Antigravity => self.home_dir.join(".gemini/antigravity/mcp_config.json"),
            SupportedApp::IFlow => self.home_dir.join(".iflow/settings.json"),
            SupportedApp::QwenCode => self.home_dir.join(".qwen/settings.json"),
            SupportedApp::Cline => self
                .home_dir
                .join(".cline/data/settings/cline_mcp_settings.json"),
            SupportedApp::Windsurf => self.home_dir.join(".codeium/windsurf/mcp_config.json"),
            SupportedApp::Kiro => self.home_dir.join(".kiro/settings/mcp.json"),
        }
    }

    pub fn can_write(&self, path: &Path) -> bool {
        if path.exists() {
            std::fs::OpenOptions::new().write(true).open(path).is_ok()
        } else {
            path.parent()
                .map(|parent| std::fs::create_dir_all(parent).is_ok())
                .unwrap_or(false)
        }
    }

    pub fn detect_installed_apps(&self) -> Vec<SupportedApp> {
        SupportedApp::ALL
            .into_iter()
            .filter(|app| self.is_app_installed(*app))
            .collect()
    }

    fn is_app_installed(&self, app: SupportedApp) -> bool {
        let (commands, macos_bundles, windows_paths) = match app {
            SupportedApp::Vscode => (
                &["code"][..],
                &["Visual Studio Code.app"][..],
                &[
                    "AppData/Local/Programs/Microsoft VS Code/Code.exe",
                    "AppData/Local/Programs/Code/Code.exe",
                ][..],
            ),
            SupportedApp::Cursor => (
                &["cursor"][..],
                &["Cursor.app"][..],
                &["AppData/Local/Programs/Cursor/Cursor.exe"][..],
            ),
            SupportedApp::ClaudeCode => (&["claude", "claude-code"][..], &[][..], &[][..]),
            SupportedApp::ClaudeDesktop => (
                &[][..],
                &["Claude.app"][..],
                &[
                    "AppData/Local/AnthropicClaude/Claude.exe",
                    "AppData/Local/Programs/Claude/Claude.exe",
                ][..],
            ),
            SupportedApp::Codex => (&["codex"][..], &[][..], &[][..]),
            SupportedApp::OpenCode => (&["opencode"][..], &[][..], &[][..]),
            SupportedApp::GithubCopilot => (
                &["github-copilot", "copilot", "gh-copilot"][..],
                &[][..],
                &[][..],
            ),
            SupportedApp::GeminiCli => (&["gemini"][..], &[][..], &[][..]),
            SupportedApp::Antigravity => (&["antigravity"][..], &[][..], &[][..]),
            SupportedApp::IFlow => (&["iflow"][..], &["iFlow.app"][..], &[][..]),
            SupportedApp::QwenCode => (&["qwen", "qwen-code"][..], &[][..], &[][..]),
            SupportedApp::Cline => (&["cline"][..], &[][..], &[][..]),
            SupportedApp::Windsurf => (
                &["windsurf"][..],
                &["Windsurf.app"][..],
                &["AppData/Local/Programs/Windsurf/Windsurf.exe"][..],
            ),
            SupportedApp::Kiro => (
                &["kiro"][..],
                &["Kiro.app"][..],
                &["AppData/Local/Programs/Kiro/Kiro.exe"][..],
            ),
        };

        self.command_exists(commands)
            || self.macos_app_bundle_exists(macos_bundles)
            || self.windows_install_exists(windows_paths)
    }

    fn command_exists(&self, commands: &[&str]) -> bool {
        let Some(path_var) = std::env::var_os("PATH") else {
            return false;
        };

        std::env::split_paths(&path_var).any(|dir| {
            commands.iter().any(|command| {
                let candidate = dir.join(command);
                if candidate.is_file() {
                    return true;
                }

                if self.os == PlatformOs::Windows {
                    return dir.join(format!("{command}.exe")).is_file();
                }

                false
            })
        })
    }

    fn macos_app_bundle_exists(&self, bundles: &[&str]) -> bool {
        if self.os != PlatformOs::MacOS {
            return false;
        }

        bundles.iter().any(|bundle| {
            self.home_dir.join("Applications").join(bundle).exists()
                || PathBuf::from("/Applications").join(bundle).exists()
        })
    }

    fn windows_install_exists(&self, candidates: &[&str]) -> bool {
        if self.os != PlatformOs::Windows {
            return false;
        }

        candidates
            .iter()
            .any(|relative| self.home_dir.join(relative).is_file())
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformContext, PlatformOs};
    use crate::core::SupportedApp;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn ctx(os: PlatformOs) -> PlatformContext {
        PlatformContext {
            os,
            home_dir: PathBuf::from("/Users/test"),
            workspace_root: PathBuf::from("/workspace/project"),
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn resolves_home_and_workspace_paths() {
        let ctx = ctx(PlatformOs::MacOS);
        assert_eq!(
            ctx.resolve_path("~/foo").to_string_lossy(),
            "/Users/test/foo"
        );
        assert_eq!(
            ctx.resolve_path(".vscode/mcp.json").to_string_lossy(),
            "/workspace/project/.vscode/mcp.json"
        );
    }

    #[test]
    fn resolves_macos_and_linux_vscode_paths() {
        assert_eq!(
            ctx(PlatformOs::MacOS)
                .user_app_config_path(SupportedApp::Vscode)
                .to_string_lossy(),
            "/Users/test/Library/Application Support/Code/User/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Vscode)
                .to_string_lossy(),
            "/Users/test/.config/Code/User/mcp.json"
        );
    }

    #[test]
    fn defines_windows_path_placeholders() {
        assert_eq!(
            ctx(PlatformOs::Windows)
                .user_app_config_path(SupportedApp::Vscode)
                .to_string_lossy(),
            "/Users/test/AppData/Roaming/Code/User/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::Windows)
                .user_app_config_path(SupportedApp::Cursor)
                .to_string_lossy(),
            "/Users/test/AppData/Roaming/Cursor/User/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::MacOS)
                .user_app_config_path(SupportedApp::ClaudeDesktop)
                .to_string_lossy(),
            "/Users/test/Library/Application Support/Claude/claude_desktop_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Windows)
                .user_app_config_path(SupportedApp::ClaudeDesktop)
                .to_string_lossy(),
            "/Users/test/AppData/Roaming/Claude/claude_desktop_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::GeminiCli)
                .to_string_lossy(),
            "/Users/test/.gemini/settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Antigravity)
                .to_string_lossy(),
            "/Users/test/.gemini/antigravity/mcp_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::IFlow)
                .to_string_lossy(),
            "/Users/test/.iflow/settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::QwenCode)
                .to_string_lossy(),
            "/Users/test/.qwen/settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Cline)
                .to_string_lossy(),
            "/Users/test/.cline/data/settings/cline_mcp_settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Windsurf)
                .to_string_lossy(),
            "/Users/test/.codeium/windsurf/mcp_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Kiro)
                .to_string_lossy(),
            "/Users/test/.kiro/settings/mcp.json"
        );
    }

    #[test]
    fn detect_installed_apps_ignores_leftover_config_files() {
        let _guard = env_lock().lock().expect("lock env");
        let temp = tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(home.join(".cursor")).expect("create cursor config dir");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        std::fs::write(home.join(".cursor/mcp.json"), "{}").expect("write leftover config");
        std::env::set_var("PATH", "");

        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(!installed.contains(&SupportedApp::Cursor));
    }

    #[test]
    fn detect_installed_apps_finds_cli_tools_on_path() {
        let _guard = env_lock().lock().expect("lock env");
        let temp = tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        let bin = temp.path().join("bin");
        std::fs::create_dir_all(&home).expect("create home");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        std::fs::create_dir_all(&bin).expect("create bin");
        std::fs::write(bin.join("codex"), "").expect("write codex stub");
        std::env::set_var("PATH", &bin);

        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(installed.contains(&SupportedApp::Codex));
    }

    #[test]
    fn detect_installed_apps_finds_macos_app_bundles() {
        let _guard = env_lock().lock().expect("lock env");
        let temp = tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(home.join("Applications/Cursor.app"))
            .expect("create cursor bundle");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        std::env::set_var("PATH", "");

        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(installed.contains(&SupportedApp::Cursor));
    }
}
