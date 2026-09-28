use mcp_manager_core::core::{ApplyResult, ImportResult, MCPConfig};
use mcp_manager_core::platform::PlatformContext;
use mcp_manager_core::storage::{resolve_relative_path, rollback};
use mcp_manager_core::store;
use mcp_manager_core::workflow::{self, WorkspaceInfo};
use std::path::{Path, PathBuf};
use std::process::Command;

const REPOSITORY_URL: &str = "https://github.com/xjeway/mcp-manager";
const RELEASES_URL: &str = "https://github.com/xjeway/mcp-manager/releases";

#[tauri::command]
pub fn load_yaml_config(relative_path: String) -> Result<String, String> {
    store::read_text(&resolve_relative_path(&relative_path))
}

#[tauri::command]
pub fn save_yaml_config(relative_path: String, content: String) -> Result<(), String> {
    store::write_text(&resolve_relative_path(&relative_path), &content)
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
    workflow::import_detected(&PlatformContext::current())
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

#[tauri::command]
pub fn current_workspace() -> Result<WorkspaceInfo, String> {
    Ok(workflow::workspace_info(&PlatformContext::current()))
}

#[tauri::command]
pub fn apply_config(
    config: MCPConfig,
    previous_config: Option<MCPConfig>,
) -> Result<ApplyResult, String> {
    workflow::apply(
        &PlatformContext::current(),
        &config,
        previous_config.as_ref(),
    )
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
    use super::nearest_existing_path;
    use tempfile::tempdir;

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
