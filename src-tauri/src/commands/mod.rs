use mcp_manager_core::core::{ApplyResult, ImportResult, MCPConfig};
use mcp_manager_core::history::{self, RollbackReport};
use mcp_manager_core::platform::PlatformContext;
use mcp_manager_core::storage::resolve_relative_path;
use mcp_manager_core::store::{self, StoredText};
use mcp_manager_core::workflow::{self, WorkspaceInfo};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

mod marketplace;
pub use marketplace::{
    marketplace_add_source, marketplace_open_url, marketplace_refresh, marketplace_remove_source,
    marketplace_search, marketplace_set_enabled, marketplace_sources,
};

const REPOSITORY_URL: &str = "https://github.com/xjeway/mcp-manager";
const RELEASES_URL: &str = "https://github.com/xjeway/mcp-manager/releases";

/// Resolve one explicit desktop selection without changing process-global cwd.
fn project_context(root: Option<&str>) -> Result<PlatformContext, String> {
    project_context_from(PlatformContext::current(), root, false)
}

/// Shared-store compensation may restore its prior revision even if the project was removed.
fn project_context_from(
    mut ctx: PlatformContext,
    root: Option<&str>,
    allow_missing: bool,
) -> Result<PlatformContext, String> {
    let Some(root) = root else {
        return Ok(ctx);
    };
    if root.is_empty() {
        ctx.workspace_root = PathBuf::new();
        return Ok(ctx);
    }
    let path = Path::new(root);
    if !path.is_absolute() {
        return Err("Choose an absolute project directory.".into());
    }
    let path = match path.canonicalize() {
        Ok(path) => path,
        Err(error)
            if allow_missing
                && error.kind() == ErrorKind::NotFound
                && !path.components().any(|c| {
                    matches!(
                        c,
                        std::path::Component::ParentDir | std::path::Component::CurDir
                    )
                }) =>
        {
            path.to_path_buf()
        }
        Err(error) => return Err(format!("Project directory is unavailable: {error}")),
    };
    if (path.exists() || !allow_missing) && !path.is_dir() {
        return Err("Choose a project directory, not a file.".into());
    }
    ctx.workspace_root = path;
    let home = ctx.home_dir.canonicalize().unwrap_or(ctx.home_dir.clone());
    if home.starts_with(&ctx.workspace_root) {
        return Err("Choose a project folder inside your home directory or another project location, not the home directory or its parent.".into());
    }
    Ok(ctx)
}

#[tauri::command]
pub fn load_yaml_config(relative_path: String) -> Result<StoredText, String> {
    store::read_text(&resolve_relative_path(&relative_path))
}

/// Saves unless the file changed since `expected_fingerprint` was read, in
/// which case the error starts with `store::CONFLICT_ERROR`. Returns the new
/// fingerprint.
#[tauri::command]
pub fn save_yaml_config(
    relative_path: String,
    content: String,
    expected_fingerprint: Option<String>,
    workspace_root: Option<String>,
) -> Result<String, String> {
    let ctx = project_context_from(PlatformContext::current(), workspace_root.as_deref(), true)?;
    if resolve_relative_path(&relative_path) == store::config_path(&ctx) {
        let next: MCPConfig = mcp_manager_core::parser::parse_yaml_config(&content)?;
        let previous = store::load(&ctx)?;
        mcp_manager_core::project::validate_foreign_placements(&ctx, &previous, &next)?;
        let scoped = mcp_manager_core::project::scoped_config(&ctx, &next);
        workflow::validate_inherited_agents(&ctx, &scoped, None)?;
    }
    store::write_text_checked(
        &resolve_relative_path(&relative_path),
        &content,
        expected_fingerprint.as_deref(),
    )
}

#[tauri::command]
pub fn yaml_config_fingerprint(relative_path: String) -> Result<String, String> {
    store::fingerprint(&resolve_relative_path(&relative_path))
}

#[tauri::command]
pub fn yaml_config_path(relative_path: String) -> String {
    resolve_relative_path(&relative_path)
        .to_string_lossy()
        .into_owned()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetResult {
    backup_path: String,
    fingerprint: String,
}

/// Starts over from `content` when the stored file cannot be used: copies it
/// to `<name>.broken-<timestamp>` first, and refuses (with
/// `store::CONFLICT_ERROR`) unless it is still the file read as
/// `expected_fingerprint`.
#[tauri::command]
pub fn reset_yaml_config(
    relative_path: String,
    content: String,
    expected_fingerprint: String,
) -> Result<ResetResult, String> {
    reset_with_backup(
        &resolve_relative_path(&relative_path),
        &content,
        &expected_fingerprint,
        &chrono::Local::now().format("%Y%m%d-%H%M%S").to_string(),
    )
}

fn reset_with_backup(
    path: &Path,
    content: &str,
    expected: &str,
    stamp: &str,
) -> Result<ResetResult, String> {
    let conflict = || {
        format!(
            "{}: {} was changed by another program",
            store::CONFLICT_ERROR,
            path.to_string_lossy()
        )
    };
    if !path.exists() {
        return Err(conflict());
    }
    let current = store::read_text(path)?;
    if current.fingerprint != expected {
        return Err(conflict());
    }

    let backup_path = write_new_backup(path, &current.content, stamp)?;
    // Checks the fingerprint again under the write lock.
    let fingerprint = store::write_text_checked(path, content, Some(expected))?;
    Ok(ResetResult {
        backup_path: backup_path.to_string_lossy().into_owned(),
        fingerprint,
    })
}

/// Writes `content` next to `path` under a name no existing file has.
fn write_new_backup(path: &Path, content: &str, stamp: &str) -> Result<PathBuf, String> {
    for attempt in 1.. {
        let mut name = path.as_os_str().to_owned();
        name.push(format!(".broken-{stamp}"));
        if attempt > 1 {
            name.push(format!("-{attempt}"));
        }
        let candidate = PathBuf::from(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                file.write_all(content.as_bytes())
                    .and_then(|()| file.sync_all())
                    .map_err(|e| e.to_string())?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    unreachable!()
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
pub fn import_detected_configs(workspace_root: Option<String>) -> Result<ImportResult, String> {
    workflow::import_detected(&project_context(workspace_root.as_deref())?)
}

#[tauri::command]
pub fn detect_installed_apps(workspace_root: Option<String>) -> Result<Vec<String>, String> {
    let ctx = project_context(workspace_root.as_deref())?;
    Ok(ctx
        .detect_installed_apps()
        .into_iter()
        .map(|app| app.as_str().to_string())
        .collect())
}

#[tauri::command]
pub fn current_workspace(workspace_root: Option<String>) -> Result<WorkspaceInfo, String> {
    let ctx = project_context(workspace_root.as_deref())?;
    let mut info = workflow::workspace_info(&ctx);
    // An unreadable store is handled by load_yaml_config; context discovery itself is read-only.
    if let Ok(config) = store::load(&ctx) {
        info.context_paths = config
            .servers
            .iter()
            .flat_map(|s| &s.placements)
            .filter(|p| {
                p.scope == mcp_manager_core::core::PlacementScope::Workspace
                    && mcp_manager_core::project::placement_in_context(&ctx, p)
            })
            .filter_map(|p| p.path.clone())
            .collect();
        info.context_paths.sort();
        info.context_paths.dedup();
    }
    Ok(info)
}

/// Preflight includes retained client fields absent from the editable server model.
#[tauri::command]
pub fn project_secret_warnings(
    config: MCPConfig,
    workspace_root: Option<String>,
) -> Result<Vec<mcp_manager_core::core::ApplyWarning>, String> {
    let ctx = project_context(workspace_root.as_deref())?;
    let config = mcp_manager_core::project::scoped_config(&ctx, &config);
    Ok(mcp_manager_core::adapters::shared_project_secret_warnings(
        &ctx, &config,
    ))
}

#[tauri::command]
/// Apply the approved configuration under the shared-store lock with a bound MiMo project alias.
pub fn apply_config(
    config: MCPConfig,
    previous_config: Option<MCPConfig>,
    expected_fingerprint: Option<String>,
    expected_mimo_project_path: Option<String>,
    workspace_root: Option<String>,
) -> Result<ApplyResult, String> {
    let ctx = project_context(workspace_root.as_deref())?;
    match previous_config {
        // Recorded, so it can be undone with `rollback_last_change`. Refused
        // if servers.yaml changed since the caller saved `config`.
        Some(previous) => history::apply_recorded_in_project(
            &ctx,
            &config,
            &previous,
            expected_fingerprint.as_deref(),
            expected_mimo_project_path.as_deref(),
        ),
        None => workflow::prepare_with_mimo_project_path(
            &ctx,
            &mcp_manager_core::project::scoped_config(&ctx, &config),
            None,
            expected_mimo_project_path.as_deref(),
        )?
        .bind_project_root(&ctx)?
        .apply(&ctx),
    }
}

/// Undoes the most recent change, in servers.yaml and the client files. The
/// caller reloads the server list afterwards.
#[tauri::command]
pub fn rollback_last_change() -> Result<RollbackReport, String> {
    history::rollback_last(&PlatformContext::current())
}

/// How many changes `rollback_last_change` can undo, one after another.
#[tauri::command]
pub fn rollback_depth() -> usize {
    history::depth(&PlatformContext::current())
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    app.request_restart();
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn explicit_project_selection_is_canonical_and_never_changes_process_cwd() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        let project = home.join("repo");
        std::fs::create_dir_all(&project).unwrap();
        let ctx = mcp_manager_core::platform::PlatformContext {
            os: mcp_manager_core::platform::PlatformOs::MacOS,
            home_dir: home.clone(),
            workspace_root: temp.path().join("startup"),
        };
        let cwd = std::env::current_dir().unwrap();
        let selected = super::project_context_from(ctx.clone(), project.to_str(), false).unwrap();
        assert_eq!(selected.workspace_root, project.canonicalize().unwrap());
        assert_eq!(std::env::current_dir().unwrap(), cwd);
        assert!(super::project_context_from(ctx.clone(), home.to_str(), false).is_err());
        assert!(super::project_context_from(ctx.clone(), Some("relative/path"), false).is_err());
        let file = project.join("file");
        std::fs::write(&file, "").unwrap();
        assert!(super::project_context_from(ctx.clone(), file.to_str(), false).is_err());
        assert!(!super::project_context_from(ctx.clone(), Some(""), false)
            .unwrap()
            .has_workspace());
        std::fs::remove_dir_all(&project).unwrap();
        assert!(super::project_context_from(ctx.clone(), project.to_str(), false).is_err());
        assert!(
            super::project_context_from(ctx, project.to_str(), true).is_ok(),
            "a failed apply can still compensate the shared store"
        );
    }

    use super::{nearest_existing_path, reset_with_backup};
    use mcp_manager_core::store;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn reset_backs_up_the_unreadable_file_then_overwrites_it() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("servers.yaml");
        fs::write(&path, "servers: [oops\n").expect("write");
        let broken = store::read_text(&path).expect("read").fingerprint;

        let first =
            reset_with_backup(&path, "version: 1\nservers: []\n", &broken, "stamp").expect("reset");
        assert_eq!(
            fs::read_to_string(&first.backup_path).expect("backup"),
            "servers: [oops\n"
        );
        assert!(first.backup_path.ends_with("servers.yaml.broken-stamp"));
        assert_eq!(
            fs::read_to_string(&path).expect("config"),
            "version: 1\nservers: []\n"
        );
        assert_eq!(
            first.fingerprint,
            store::read_text(&path).expect("read").fingerprint
        );

        // A second reset in the same second keeps the first backup.
        fs::write(&path, "again: [\n").expect("write");
        let broken = store::read_text(&path).expect("read").fingerprint;
        let second =
            reset_with_backup(&path, "version: 1\nservers: []\n", &broken, "stamp").expect("reset");
        assert!(second.backup_path.ends_with("servers.yaml.broken-stamp-2"));
        assert_eq!(
            fs::read_to_string(&first.backup_path).expect("backup"),
            "servers: [oops\n"
        );
    }

    #[test]
    fn reset_refuses_when_the_file_changed_or_is_gone() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("servers.yaml");
        fs::write(&path, "fixed: true\n").expect("write");

        let error = reset_with_backup(&path, "x", "stale", "stamp").expect_err("conflict");
        assert!(error.starts_with(store::CONFLICT_ERROR));
        assert_eq!(fs::read_to_string(&path).expect("config"), "fixed: true\n");
        assert_eq!(fs::read_dir(dir.path()).expect("dir").count(), 1);

        let missing = dir.path().join("missing.yaml");
        let error = reset_with_backup(&missing, "x", "absent", "stamp").expect_err("conflict");
        assert!(error.starts_with(store::CONFLICT_ERROR));
        assert!(!missing.exists());
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
