use mcp_manager_core::adapters::{adapters, workspace_scope_path};
use mcp_manager_core::core::{
    build_import_result, ApplyResult, DetectedServer, ImportResult, MCPConfig, SupportedApp,
};
use mcp_manager_core::platform::PlatformContext;
use mcp_manager_core::storage::{apply_operations, resolve_relative_path, rollback};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REPOSITORY_URL: &str = "https://github.com/xjeway/mcp-manager";
const RELEASES_URL: &str = "https://github.com/xjeway/mcp-manager/releases";

#[tauri::command]
pub fn load_yaml_config(relative_path: String) -> Result<String, String> {
    let path = resolve_relative_path(&relative_path);
    if !path.exists() {
        return Ok("version: 1\nservers: []\n".to_string());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_yaml_config(relative_path: String, content: String) -> Result<(), String> {
    let path = resolve_relative_path(&relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_repository_link() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(REPOSITORY_URL).status();

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", "", REPOSITORY_URL])
        .status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(REPOSITORY_URL).status();

    status.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("failed to open {}", REPOSITORY_URL))
        }
    })
}

/// Returns `path` itself when it exists, otherwise its closest existing
/// ancestor, so "open" on a not-yet-created config file reveals its folder.
fn nearest_existing_path(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| candidate.exists())
        .map(Path::to_path_buf)
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let requested = PathBuf::from(&path);
    // Only absolute paths are accepted: this keeps the argument from being read
    // as a command-line flag by `open`/`xdg-open`.
    if !requested.is_absolute() {
        return Err(format!("refusing to open non-absolute path {}", path));
    }
    let target =
        nearest_existing_path(&requested).ok_or_else(|| format!("failed to open {}", path))?;

    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(&target).status();

    // explorer.exe takes the path as a plain argument (no cmd.exe parsing of
    // `&`, `|`, ...) but reports a non-zero exit code even on success, so only
    // a failure to launch it is treated as an error.
    #[cfg(target_os = "windows")]
    return Command::new("explorer")
        .arg(&target)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string());

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(&target).status();

    #[cfg(not(target_os = "windows"))]
    status.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("failed to open {}", path))
        }
    })
}

#[tauri::command]
pub fn open_releases_link() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(RELEASES_URL).status();

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", "", RELEASES_URL])
        .status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(RELEASES_URL).status();

    status.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("failed to open {}", RELEASES_URL))
        }
    })
}

#[tauri::command]
pub fn import_detected_configs() -> Result<ImportResult, String> {
    let ctx = PlatformContext::current();
    let mut sources = Vec::new();
    let mut detected_servers = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let yaml_path = resolve_relative_path("config/servers.yaml");
    if yaml_path.exists() {
        let content = fs::read_to_string(&yaml_path).map_err(|e| e.to_string())?;
        match mcp_manager_core::parser::parse_yaml_config(&content) {
            Ok(config) => {
                sources.push(mcp_manager_core::core::LocalConfigSource {
                    app: "yaml".to_string(),
                    path: yaml_path.to_string_lossy().to_string(),
                    exists: true,
                    format: "yaml".to_string(),
                    priority: 0,
                    content: Some(content),
                });
                for server in config.servers {
                    detected_servers.push(DetectedServer {
                        server,
                        priority: 0,
                    });
                }
            }
            Err(error) => errors.push(format!("{}: {}", yaml_path.to_string_lossy(), error)),
        }
    } else {
        sources.push(mcp_manager_core::core::LocalConfigSource {
            app: "yaml".to_string(),
            path: yaml_path.to_string_lossy().to_string(),
            exists: false,
            format: "yaml".to_string(),
            priority: 0,
            content: None,
        });
    }

    for adapter in adapters() {
        for (path, priority) in adapter.detect_sources(&ctx) {
            let resolved = ctx.resolve_path(&path);
            if !resolved.exists() {
                sources.push(mcp_manager_core::core::LocalConfigSource {
                    app: adapter.app().as_str().to_string(),
                    path: resolved.to_string_lossy().to_string(),
                    exists: false,
                    format: "json".to_string(),
                    priority,
                    content: None,
                });
                continue;
            }

            let content = fs::read_to_string(&resolved).map_err(|e| e.to_string())?;
            let parsed =
                adapter.parse_source(&ctx, &resolved.to_string_lossy(), priority, &content);
            sources.extend(parsed.sources);
            warnings.extend(parsed.warnings);
            errors.extend(parsed.errors);
            detected_servers.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
    }

    Ok(build_import_result(
        sources,
        detected_servers,
        warnings,
        errors,
    ))
}

#[tauri::command]
pub fn detect_installed_apps() -> Result<Vec<String>, String> {
    let ctx = PlatformContext::current();
    Ok(ctx
        .detect_installed_apps()
        .into_iter()
        .map(|app| app.as_str().to_string())
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    /// Empty when the app was not started from a project directory.
    pub root: String,
    /// Project-level config file per app that supports one.
    pub placement_paths: HashMap<SupportedApp, String>,
}

fn workspace_info(ctx: &PlatformContext) -> WorkspaceInfo {
    if !ctx.has_workspace() {
        return WorkspaceInfo {
            root: String::new(),
            placement_paths: HashMap::new(),
        };
    }

    WorkspaceInfo {
        root: ctx.workspace_root.to_string_lossy().to_string(),
        placement_paths: SupportedApp::ALL
            .into_iter()
            .filter_map(|app| workspace_scope_path(ctx, app).map(|path| (app, path)))
            .collect(),
    }
}

#[tauri::command]
pub fn current_workspace() -> Result<WorkspaceInfo, String> {
    Ok(workspace_info(&PlatformContext::current()))
}

#[tauri::command]
pub fn apply_config(
    config: MCPConfig,
    previous_config: Option<MCPConfig>,
) -> Result<ApplyResult, String> {
    let ctx = PlatformContext::current();
    let operations = adapters()
        .into_iter()
        .flat_map(|adapter| adapter.plan_apply(&ctx, &config, previous_config.as_ref()))
        .collect::<Vec<_>>();

    for operation in &operations {
        let path = ctx.resolve_path(&operation.path);
        if !ctx.can_write(&path) {
            let error = format!("permission denied for {}", path.to_string_lossy());
            if mcp_manager_core::security::is_high_risk_condition(&error) {
                return Err(error);
            }
        }
    }

    let backups = apply_operations(operations)?;
    Ok(ApplyResult { backups })
}

#[tauri::command]
pub fn rollback_from_backups(backups: Vec<String>) -> Result<(), String> {
    rollback(backups)
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    app.request_restart();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{nearest_existing_path, workspace_info};
    use mcp_manager_core::core::SupportedApp;
    use mcp_manager_core::platform::test_paths::{abs, UnixPath};
    use mcp_manager_core::platform::{PlatformContext, PlatformOs};
    use tempfile::tempdir;

    fn ctx(workspace_root: &str) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: abs("/Users/test"),
            workspace_root: abs(workspace_root),
        }
    }

    #[test]
    fn reports_project_paths_from_the_backend_path_table() {
        let info = workspace_info(&ctx("/workspace/project"));

        assert_eq!(info.root.unix(), "/workspace/project");
        assert_eq!(
            info.placement_paths
                .get(&SupportedApp::Vscode)
                .map(UnixPath::unix)
                .as_deref(),
            Some("/workspace/project/.vscode/mcp.json")
        );
        assert!(!info.placement_paths.contains_key(&SupportedApp::Codex));
    }

    #[test]
    fn reports_no_workspace_when_launched_outside_a_project() {
        let info = workspace_info(&ctx("/"));

        assert!(info.root.is_empty());
        assert!(info.placement_paths.is_empty());
    }

    #[test]
    fn falls_back_to_the_closest_existing_folder() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join(".cursor/mcp.json");

        assert_eq!(
            nearest_existing_path(&missing),
            Some(dir.path().to_path_buf())
        );
    }
}
