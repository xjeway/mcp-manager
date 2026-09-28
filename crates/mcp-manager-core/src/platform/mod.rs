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
        // Unlike `dirs`, std honors USERPROFILE on Windows, as it does HOME elsewhere.
        let home_dir = std::env::home_dir().unwrap_or_else(|| workspace_root.clone());

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

    /// The process cwd only identifies a project when the app is started from a
    /// project directory. GUI launches (Finder, Dock, Start menu) start in the
    /// filesystem root or the home directory; neither they nor any other ancestor
    /// of the home directory may be treated as a project, or every user-level
    /// config file would be classified as project-level.
    pub fn has_workspace(&self) -> bool {
        self.workspace_root.is_absolute() && !self.home_dir.starts_with(&self.workspace_root)
    }

    pub fn workspace_file(&self, relative: &str) -> PathBuf {
        self.workspace_root.join(relative)
    }

    pub fn app_data_dir(&self) -> PathBuf {
        match self.os {
            PlatformOs::MacOS => self
                .home_dir
                .join("Library/Application Support/mcp-manager"),
            PlatformOs::Windows => self.home_dir.join("AppData/Roaming/mcp-manager"),
            PlatformOs::Linux | PlatformOs::Unknown => self.home_dir.join(".config/mcp-manager"),
        }
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
            // Cursor keeps its global MCP config under the home directory on every OS.
            SupportedApp::Cursor => self.home_dir.join(".cursor/mcp.json"),
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
            SupportedApp::Qoder => self.home_dir.join(".qoder/settings.json"),
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
                &[
                    "AppData/Local/Programs/Cursor/Cursor.exe",
                    "AppData/Local/Programs/cursor/Cursor.exe",
                ][..],
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
            SupportedApp::Qoder => (
                &["qoder", "qodercli"][..],
                &["Qoder.app"][..],
                &["AppData/Local/Programs/Qoder/Qoder.exe"][..],
            ),
        };

        self.command_exists(commands)
            || self.macos_app_bundle_exists(macos_bundles)
            || self.windows_install_exists(windows_paths)
            || self.config_marker_exists(app)
    }

    fn command_exists(&self, commands: &[&str]) -> bool {
        self.command_search_dirs().iter().any(|dir| {
            commands.iter().any(|command| {
                let candidate = dir.join(command);
                if candidate.is_file() {
                    return true;
                }

                if self.os == PlatformOs::Windows {
                    return [".exe", ".cmd", ".bat"]
                        .iter()
                        .any(|extension| dir.join(format!("{command}{extension}")).is_file());
                }

                false
            })
        })
    }

    fn command_search_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        let mut push_unique = |path: PathBuf| {
            if !dirs.contains(&path) {
                dirs.push(path);
            }
        };

        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                push_unique(dir);
            }
        }

        for relative in [
            "bin",
            ".local/bin",
            ".cargo/bin",
            ".npm-global/bin",
            ".bun/bin",
            ".yarn/bin",
            ".local/share/pnpm",
            "Library/pnpm",
        ] {
            push_unique(self.home_dir.join(relative));
        }

        if std::env::var_os("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS").is_none() {
            match self.os {
                PlatformOs::MacOS => {
                    push_unique(PathBuf::from("/opt/homebrew/bin"));
                    push_unique(PathBuf::from("/usr/local/bin"));
                    push_unique(PathBuf::from("/opt/local/bin"));
                    push_unique(PathBuf::from("/usr/bin"));
                    push_unique(PathBuf::from("/bin"));
                }
                PlatformOs::Linux => {
                    push_unique(PathBuf::from("/usr/local/bin"));
                    push_unique(PathBuf::from("/usr/bin"));
                    push_unique(PathBuf::from("/bin"));
                    push_unique(PathBuf::from("/snap/bin"));
                }
                PlatformOs::Windows | PlatformOs::Unknown => {}
            }
        }

        dirs
    }

    fn config_marker_exists(&self, app: SupportedApp) -> bool {
        self.config_markers(app)
            .into_iter()
            .any(|path| self.resolve_path(&path).exists())
    }

    fn config_markers(&self, app: SupportedApp) -> Vec<String> {
        match app {
            SupportedApp::Vscode => vec![
                self.workspace_file(".vscode/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                self.user_app_config_path(SupportedApp::Vscode)
                    .to_string_lossy()
                    .to_string(),
            ],
            SupportedApp::ClaudeCode
            | SupportedApp::Codex
            | SupportedApp::OpenCode
            | SupportedApp::GeminiCli
            | SupportedApp::Antigravity
            | SupportedApp::QwenCode
            | SupportedApp::Cline
            | SupportedApp::Qoder => {
                vec![self.user_app_config_path(app).to_string_lossy().to_string()]
            }
            SupportedApp::GithubCopilot => vec![
                self.workspace_file(".vscode/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                self.user_app_config_path(SupportedApp::Vscode)
                    .to_string_lossy()
                    .to_string(),
                self.user_app_config_path(SupportedApp::GithubCopilot)
                    .to_string_lossy()
                    .to_string(),
            ],
            SupportedApp::Cursor
            | SupportedApp::ClaudeDesktop
            | SupportedApp::IFlow
            | SupportedApp::Windsurf
            | SupportedApp::Kiro => Vec::new(),
        }
    }

    fn macos_app_bundle_exists(&self, bundles: &[&str]) -> bool {
        if self.os != PlatformOs::MacOS {
            return false;
        }

        // Tests set this to keep what the machine has installed out of detection.
        let include_global = std::env::var_os("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS").is_none();
        bundles.iter().any(|bundle| {
            self.home_dir.join("Applications").join(bundle).exists()
                || (include_global && PathBuf::from("/Applications").join(bundle).exists())
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

/// Fixture helpers so tests written with Unix paths also run on Windows.
#[cfg(test)]
pub(crate) mod test_paths {
    use std::path::{Path, PathBuf};

    /// A Unix-style absolute path, given a drive on Windows so it stays absolute.
    pub fn abs(unix: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:{unix}"))
        } else {
            PathBuf::from(unix)
        }
    }

    pub trait UnixPath {
        /// The path with `/` separators and no drive, to compare with fixtures.
        fn unix(&self) -> String;
    }

    impl<T: AsRef<Path> + ?Sized> UnixPath for T {
        fn unix(&self) -> String {
            let path = self.as_ref().to_string_lossy().replace('\\', "/");
            path.strip_prefix("C:").map(str::to_string).unwrap_or(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_paths::{abs, UnixPath};
    use super::{PlatformContext, PlatformOs};
    use crate::core::SupportedApp;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn ctx(os: PlatformOs) -> PlatformContext {
        PlatformContext {
            os,
            home_dir: abs("/Users/test"),
            workspace_root: abs("/workspace/project"),
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn resolves_home_and_workspace_paths() {
        let ctx = ctx(PlatformOs::MacOS);
        assert_eq!(ctx.resolve_path("~/foo").unix(), "/Users/test/foo");
        assert_eq!(
            ctx.resolve_path(".vscode/mcp.json").unix(),
            "/workspace/project/.vscode/mcp.json"
        );
    }

    #[test]
    fn treats_only_real_project_directories_as_workspaces() {
        assert!(ctx(PlatformOs::MacOS).has_workspace());

        let mut launched_from_finder = ctx(PlatformOs::MacOS);
        launched_from_finder.workspace_root = abs("/");
        assert!(!launched_from_finder.has_workspace());

        let mut launched_from_home = ctx(PlatformOs::MacOS);
        launched_from_home.workspace_root = abs("/Users/test");
        assert!(!launched_from_home.has_workspace());

        let mut launched_from_users = ctx(PlatformOs::MacOS);
        launched_from_users.workspace_root = abs("/Users");
        assert!(!launched_from_users.has_workspace());
    }

    #[test]
    fn resolves_macos_and_linux_vscode_paths() {
        assert_eq!(
            ctx(PlatformOs::MacOS)
                .user_app_config_path(SupportedApp::Vscode)
                .unix(),
            "/Users/test/Library/Application Support/Code/User/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Vscode)
                .unix(),
            "/Users/test/.config/Code/User/mcp.json"
        );
    }

    #[test]
    fn defines_windows_path_placeholders() {
        assert_eq!(
            ctx(PlatformOs::Windows)
                .user_app_config_path(SupportedApp::Vscode)
                .unix(),
            "/Users/test/AppData/Roaming/Code/User/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::Windows)
                .user_app_config_path(SupportedApp::Cursor)
                .unix(),
            "/Users/test/.cursor/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::MacOS)
                .user_app_config_path(SupportedApp::ClaudeDesktop)
                .unix(),
            "/Users/test/Library/Application Support/Claude/claude_desktop_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Windows)
                .user_app_config_path(SupportedApp::ClaudeDesktop)
                .unix(),
            "/Users/test/AppData/Roaming/Claude/claude_desktop_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::GeminiCli)
                .unix(),
            "/Users/test/.gemini/settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Antigravity)
                .unix(),
            "/Users/test/.gemini/antigravity/mcp_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::IFlow)
                .unix(),
            "/Users/test/.iflow/settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::QwenCode)
                .unix(),
            "/Users/test/.qwen/settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Cline)
                .unix(),
            "/Users/test/.cline/data/settings/cline_mcp_settings.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Windsurf)
                .unix(),
            "/Users/test/.codeium/windsurf/mcp_config.json"
        );
        assert_eq!(
            ctx(PlatformOs::Linux)
                .user_app_config_path(SupportedApp::Kiro)
                .unix(),
            "/Users/test/.kiro/settings/mcp.json"
        );
        assert_eq!(
            ctx(PlatformOs::MacOS).app_data_dir().unix(),
            "/Users/test/Library/Application Support/mcp-manager"
        );
        assert_eq!(
            ctx(PlatformOs::Windows).app_data_dir().unix(),
            "/Users/test/AppData/Roaming/mcp-manager"
        );
        assert_eq!(
            ctx(PlatformOs::Linux).app_data_dir().unix(),
            "/Users/test/.config/mcp-manager"
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
        std::env::set_var("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS", "1");

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
        std::env::set_var("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS", "1");

        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(installed.contains(&SupportedApp::Codex));
    }

    #[test]
    fn detect_installed_apps_finds_cli_tools_in_common_user_bin_without_path() {
        let _guard = env_lock().lock().expect("lock env");
        let temp = tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        let local_bin = home.join(".local/bin");
        std::fs::create_dir_all(&local_bin).expect("create local bin");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        std::fs::write(local_bin.join("codex"), "").expect("write codex stub");
        std::env::set_var("PATH", "");
        std::env::set_var("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS", "1");

        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(installed.contains(&SupportedApp::Codex));
    }

    #[test]
    fn detect_installed_apps_finds_plugin_clients_from_existing_config_sources() {
        let _guard = env_lock().lock().expect("lock env");
        let temp = tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(home.join(".cline/data/settings"))
            .expect("create cline settings dir");
        std::fs::create_dir_all(home.join(".copilot")).expect("create copilot dir");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        std::fs::write(
            home.join(".cline/data/settings/cline_mcp_settings.json"),
            r#"{"mcpServers":{}}"#,
        )
        .expect("write cline config");
        std::fs::write(
            home.join(".copilot/mcp-config.json"),
            r#"{"mcpServers":{}}"#,
        )
        .expect("write copilot config");
        std::env::set_var("PATH", "");
        std::env::set_var("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS", "1");

        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(installed.contains(&SupportedApp::Cline));
        assert!(installed.contains(&SupportedApp::GithubCopilot));
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
        std::env::set_var("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS", "1");

        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home,
            workspace_root: workspace,
        };

        let installed = ctx.detect_installed_apps();

        assert!(installed.contains(&SupportedApp::Cursor));
    }
}
