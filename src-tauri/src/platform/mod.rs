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
                PlatformOs::Windows => self
                    .home_dir
                    .join("AppData/Roaming/Code/User/mcp.json"),
                _ => self.home_dir.join(".config/Code/User/mcp.json"),
            },
            SupportedApp::Cursor => match self.os {
                PlatformOs::Windows => self.home_dir.join("AppData/Roaming/Cursor/User/mcp.json"),
                _ => self.home_dir.join(".cursor/mcp.json"),
            },
            SupportedApp::ClaudeCode => self.home_dir.join(".claude.json"),
            SupportedApp::Codex => self.home_dir.join(".codex/config.toml"),
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
}

#[cfg(test)]
mod tests {
    use super::{PlatformContext, PlatformOs};
    use crate::core::SupportedApp;
    use std::path::PathBuf;

    fn ctx(os: PlatformOs) -> PlatformContext {
        PlatformContext {
            os,
            home_dir: PathBuf::from("/Users/test"),
            workspace_root: PathBuf::from("/workspace/project"),
        }
    }

    #[test]
    fn resolves_home_and_workspace_paths() {
        let ctx = ctx(PlatformOs::MacOS);
        assert_eq!(ctx.resolve_path("~/foo").to_string_lossy(), "/Users/test/foo");
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
    }
}
