//! What each applied change did, so the app and the CLI can undo it: the most
//! recent changes, newest last, in `history.json` next to servers.yaml.
//!
//! A change is undone only if nothing touched servers.yaml or the client files
//! since. Undoing is safe to run again after a crash halfway: a file or the
//! server list already back in its earlier state counts as ready.

use crate::core::{ApplyResult, MCPConfig};
use crate::platform::PlatformContext;
use crate::storage::{self, atomic_write, remove_empty_dirs, Snapshot};
use crate::store::{self, ABSENT_FINGERPRINT};
use crate::workflow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// How many changes can be undone one after another.
pub const MAX_CHANGES: usize = 20;

const FILE_NAME: &str = "history.json";

/// A client file a change wrote, by `store::fingerprint` before and after.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub before: String,
    pub after: String,
}

impl FileChange {
    pub fn was_created(&self) -> bool {
        self.before == ABSENT_FINGERPRINT
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRecord {
    /// Copies of the client files that existed before the change.
    pub backups: Vec<String>,
    pub files: Vec<FileChange>,
    /// Directories the change had to create.
    pub created_dirs: Vec<String>,
    /// servers.yaml before the change.
    pub previous_config: MCPConfig,
    /// servers.yaml as the change wrote it.
    pub applied_config: MCPConfig,
}

impl ChangeRecord {
    /// Call after the change was applied. `snapshot` is what
    /// [`workflow::snapshot`] took before it.
    pub fn new(
        snapshot: &Snapshot,
        backups: Vec<String>,
        previous_config: MCPConfig,
        applied_config: MCPConfig,
    ) -> Result<Self, String> {
        let files = snapshot
            .fingerprints()?
            .into_iter()
            .map(|(path, before)| {
                Ok(FileChange {
                    after: store::fingerprint(&path)?,
                    path: path.to_string_lossy().to_string(),
                    before,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(ChangeRecord {
            backups,
            files,
            created_dirs: snapshot
                .created_dirs()
                .iter()
                .map(|dir| dir.to_string_lossy().to_string())
                .collect(),
            previous_config,
            applied_config,
        })
    }

    /// Whether the change altered neither the server list nor any client file.
    pub fn is_noop(&self) -> bool {
        self.files.iter().all(|file| file.before == file.after)
            && same_config(&self.previous_config, &self.applied_config)
    }

    /// Client files this change would put back or remove.
    pub fn file_count(&self) -> usize {
        self.files.iter().filter(|f| f.before != f.after).count()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct History {
    changes: Vec<ChangeRecord>,
}

fn path(ctx: &PlatformContext) -> PathBuf {
    ctx.app_data_dir().join(FILE_NAME)
}

/// Missing or unreadable history is treated as empty.
fn load(path: &Path) -> History {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save(path: &Path, history: &History) -> Result<(), String> {
    let content = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    atomic_write(path, &content)
}

fn same_config(a: &MCPConfig, b: &MCPConfig) -> bool {
    serde_json::to_value(a).ok() == serde_json::to_value(b).ok()
}

/// Adds a change on top of the ones that can be undone.
pub fn record(ctx: &PlatformContext, change: ChangeRecord) -> Result<(), String> {
    let path = path(ctx);
    store::with_lock(&path, || {
        let mut history = load(&path);
        history.changes.push(change);
        let excess = history.changes.len().saturating_sub(MAX_CHANGES);
        history.changes.drain(..excess);
        save(&path, &history)
    })
}

/// How many changes can be undone.
pub fn depth(ctx: &PlatformContext) -> usize {
    load(&path(ctx)).changes.len()
}

/// The change [`rollback_last`] would undo.
pub fn latest(ctx: &PlatformContext) -> Option<ChangeRecord> {
    load(&path(ctx)).changes.pop()
}

/// Applies `config` to every client like [`workflow::apply`] and records it
/// for undo. servers.yaml must already hold `config`.
pub fn apply_recorded(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: &MCPConfig,
) -> Result<ApplyResult, String> {
    let snapshot = workflow::snapshot(ctx, config, Some(previous_config))?;
    let result = workflow::apply(ctx, config, Some(previous_config))?;
    // The change is applied; failing to record it only costs the undo.
    if let Ok(change) = ChangeRecord::new(
        &snapshot,
        result.backups.clone(),
        previous_config.clone(),
        config.clone(),
    ) {
        if !change.is_noop() {
            let _ = record(ctx, change);
        }
    }
    Ok(result)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackReport {
    /// Client files put back as they were.
    pub restored: usize,
    /// Client files the change created, now removed.
    pub removed: usize,
    /// Changes that can still be undone.
    pub remaining: usize,
}

/// Undoes the most recent change: client files and servers.yaml. Fails,
/// changing nothing, if either was edited since the change.
pub fn rollback_last(ctx: &PlatformContext) -> Result<RollbackReport, String> {
    let history_path = path(ctx);
    let Some(change) = latest(ctx) else {
        return Err("Nothing to roll back.".to_string());
    };
    let mut report = RollbackReport {
        restored: 0,
        removed: 0,
        remaining: 0,
    };

    store::update(ctx, |config| {
        // Undoing on top of a later edit would silently discard it.
        if !same_config(config, &change.applied_config)
            && !same_config(config, &change.previous_config)
        {
            return Err(
                "The server list changed after the last change made here (in MCP Manager or \
                 another command), so rolling back would discard that edit. Nothing was changed."
                    .to_string(),
            );
        }
        let edited = change
            .files
            .iter()
            .filter(|file| {
                let current = store::fingerprint(Path::new(&file.path)).ok();
                current.as_deref() != Some(file.after.as_str())
                    && current.as_deref() != Some(file.before.as_str())
            })
            .map(|file| file.path.clone())
            .collect::<Vec<_>>();
        if !edited.is_empty() {
            return Err(format!(
                "These client files were edited after the last change made here, so rolling \
                 back would discard those edits: {}. Nothing was changed.",
                edited.join(", ")
            ));
        }

        // Every backup must be there before the first file is touched.
        let mut to_restore = Vec::new();
        for backup in &change.backups {
            let backup = PathBuf::from(backup);
            let target = storage::backup_target(&backup)?;
            let target_key = target.to_string_lossy();
            let Some(file) = change.files.iter().find(|f| f.path == target_key) else {
                continue;
            };
            let current = store::fingerprint(&target)?;
            if current == file.after && file.before != file.after {
                if !backup.exists() {
                    return Err(format!(
                        "The backup {} is gone, so {target_key} cannot be restored. \
                         Nothing was changed.",
                        backup.to_string_lossy()
                    ));
                }
                to_restore.push(backup);
            }
        }
        for file in change.files.iter().filter(|f| f.was_created()) {
            if Path::new(&file.path).exists() {
                fs::remove_file(&file.path).map_err(|e| format!("{}: {e}", file.path))?;
                report.removed += 1;
            }
        }
        for backup in &to_restore {
            storage::restore_backup(backup)?;
            report.restored += 1;
        }
        let dirs = change
            .created_dirs
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        remove_empty_dirs(&dirs);

        *config = change.previous_config.clone();
        Ok(())
    })?;

    // Only now that everything is back is the change forgotten; if this
    // fails, running the rollback again finishes without touching anything.
    store::with_lock(&history_path, || {
        let mut history = load(&history_path);
        if !history.changes.is_empty() {
            history.changes.pop();
        }
        report.remaining = history.changes.len();
        save(&history_path, &history)
    })?;
    Ok(report)
}
