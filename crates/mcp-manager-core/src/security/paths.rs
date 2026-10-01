//! Which paths MCP Manager is willing to read or write.
//!
//! The UI is not the authority. Every client-config write is checked against
//! the paths the adapters already model, and every internal config command is
//! limited to `config/servers.yaml` under the application-data directory.
//!
//! Symlinks are refused when they sit inside the home directory or the active
//! workspace and point the write somewhere else. Symlinks above those roots
//! (for example `/var` -> `/private/var` on macOS) are not an escape: both the
//! expected path and the candidate live under the same root.

use crate::adapters::workspace_scope_path;
use crate::core::SupportedApp;
use crate::platform::PlatformContext;
use crate::store::CONFIG_RELATIVE_PATH;
use std::path::{Component, Path, PathBuf};

/// Resolves a frontend-supplied internal config path. Absolute paths and `..`
/// are rejected. Only the unified server list is addressable this way.
pub fn resolve_internal_relative(relative: &str) -> Result<PathBuf, String> {
    if relative != CONFIG_RELATIVE_PATH {
        return Err(format!(
            "refusing unexpected internal config path {relative}"
        ));
    }
    let path = Path::new(relative);
    if path.is_absolute() || has_parent_dir(path) {
        return Err(format!(
            "refusing internal config path that escapes the application data directory: {relative}"
        ));
    }
    let root = PlatformContext::current().app_data_dir();
    let joined = root.join(path);
    if !joined.starts_with(&root) {
        return Err(format!(
            "refusing internal config path that escapes the application data directory: {relative}"
        ));
    }
    Ok(joined)
}

/// Accepts `raw` only when it is one of the modeled client config files and
/// does not hop out of that file's root through a symlink.
pub fn validate_client_config_path(ctx: &PlatformContext, raw: &str) -> Result<PathBuf, String> {
    if raw.contains('\0') {
        return Err("refusing a config path that contains a NUL byte".to_string());
    }
    let resolved = lexical_normalize(&ctx.resolve_path(raw));
    if has_parent_dir(Path::new(raw)) {
        return Err(format!("refusing a config path that contains '..': {raw}"));
    }
    let expected = allowed_client_paths(ctx)
        .into_iter()
        .find(|candidate| {
            paths_equal(candidate, &resolved) || same_existing_file(candidate, &resolved)
        })
        .ok_or_else(|| {
            format!(
                "refusing to write {} — it is not a known MCP client config path",
                resolved.display()
            )
        })?;
    // Symlinks above the home or workspace (macOS `/var` -> `/private/var`)
    // are ignored. A symlink inside either root can point the write at a
    // different file, so both the caller path and the allowlisted path are
    // checked.
    if symlink_inside(&ctx.home_dir, &expected)
        || symlink_inside(&ctx.workspace_root, &expected)
        || symlink_inside(&ctx.home_dir, &resolved)
        || symlink_inside(&ctx.workspace_root, &resolved)
    {
        return Err(format!(
            "refusing to follow a symlink in {}",
            expected.display()
        ));
    }
    Ok(expected)
}

/// `/var/...` and `/private/var/...` name one file on macOS. Only used when
/// both paths exist, so a missing file cannot be matched through a dangling link.
fn same_existing_file(left: &Path, right: &Path) -> bool {
    matches!(
        (left.canonicalize(), right.canonicalize()),
        (Ok(left), Ok(right)) if paths_equal(&left, &right)
    )
}

pub fn allowed_client_paths(ctx: &PlatformContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for app in SupportedApp::ALL {
        paths.push(lexical_normalize(&ctx.user_app_config_path(app)));
        if let Some(workspace) = workspace_scope_path(ctx, app) {
            paths.push(lexical_normalize(&ctx.resolve_path(&workspace)));
        }
    }
    paths
}

fn has_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| component == Component::ParentDir)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    if cfg!(windows) {
        let fold = |path: &Path| {
            path.components()
                .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
                .collect::<Vec<_>>()
        };
        fold(left) == fold(right)
    } else {
        left == right
    }
}

/// True when any component of `full` strictly inside `root` is a symlink.
fn symlink_inside(root: &Path, full: &Path) -> bool {
    let Ok(relative) = full.strip_prefix(root) else {
        return false;
    };
    let mut acc = root.to_path_buf();
    for component in relative.components() {
        acc.push(component);
        if fs_is_symlink(&acc) {
            return true;
        }
    }
    false
}

fn fs_is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformOs;
    use std::fs;

    fn ctx(root: &std::path::Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::Linux,
            home_dir: root.join("home"),
            workspace_root: root.join("workspace"),
        }
    }

    #[test]
    fn internal_paths_reject_absolutes_and_traversal() {
        for bad in [
            "../servers.yaml",
            "/etc/passwd",
            "config/../../secrets",
            "history.json",
        ] {
            assert!(
                resolve_internal_relative(bad).is_err(),
                "{bad} should be rejected"
            );
        }
        let ok = resolve_internal_relative(CONFIG_RELATIVE_PATH).expect("servers.yaml");
        assert!(ok.ends_with(CONFIG_RELATIVE_PATH));
    }

    #[test]
    fn client_writes_accept_modeled_paths_and_reject_escapes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        fs::create_dir_all(ctx.home_dir.join(".codex")).expect("codex dir");
        fs::create_dir_all(&ctx.workspace_root).expect("workspace");

        let codex = ctx.user_app_config_path(SupportedApp::Codex);
        validate_client_config_path(&ctx, &codex.to_string_lossy()).expect("codex path");

        for bad in [
            "../etc/passwd",
            "/etc/passwd",
            dir.path().join("evil.json").to_string_lossy().as_ref(),
        ] {
            assert!(
                validate_client_config_path(&ctx, bad).is_err(),
                "{bad} should be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlink_inside_the_workspace_cannot_escape_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).expect("outside");
        fs::create_dir_all(&ctx.workspace_root).expect("workspace");
        std::os::unix::fs::symlink(&outside, ctx.workspace_root.join(".cursor")).expect("symlink");

        let escaped = ctx.workspace_root.join(".cursor/mcp.json");
        let error = validate_client_config_path(&ctx, &escaped.to_string_lossy())
            .expect_err("symlink must be rejected");
        assert!(error.contains("symlink"), "{error}");
        assert!(!outside.join("mcp.json").exists());
    }
}
