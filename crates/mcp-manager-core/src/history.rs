//! What each applied change did, so the app and the CLI can undo it: the most
//! recent changes, newest last, in `history.json` next to servers.yaml.
//!
//! A change is undone only if nothing touched servers.yaml or the client files
//! since. Undoing is safe to run again after a crash halfway: a file or the
//! server list already back in its earlier state counts as ready.

use crate::core::{ApplyResult, MCPConfig};
use crate::platform::PlatformContext;
use crate::security::{config_fingerprint, redact_config};
use crate::storage::{self, atomic_write, remove_empty_dirs, Snapshot};
use crate::store::{self, ABSENT_FINGERPRINT};
use crate::workflow;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    /// servers.yaml before the change. Secret values are redacted. Rollback
    /// restores the real file from [`Self::config_backup`].
    pub previous_config: MCPConfig,
    /// servers.yaml as the change wrote it, with the same redaction.
    pub applied_config: MCPConfig,
    /// SHA-256 of the full previous config, including secrets.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub previous_fingerprint: String,
    /// SHA-256 of the full applied config, including secrets.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub applied_fingerprint: String,
    /// Private copy of the full previous servers.yaml under the backup root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_backup: Option<String>,
}

impl ChangeRecord {
    /// Call after the change was applied. `snapshot` is what
    /// [`workflow::snapshot`] took before it.
    ///
    /// `history.json` stores redacted configs. The full previous config is
    /// written to `config_backup` so rollback does not need the secrets in
    /// the history file.
    pub fn new(
        app_data: &Path,
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
        let previous_fingerprint = config_fingerprint(&previous_config)?;
        let applied_fingerprint = config_fingerprint(&applied_config)?;
        let yaml = store::to_yaml(&previous_config)?;
        let backup = storage::write_internal_backup(app_data, yaml.as_bytes())?;
        Ok(ChangeRecord {
            backups,
            files,
            created_dirs: snapshot
                .created_dirs()
                .iter()
                .map(|dir| dir.to_string_lossy().to_string())
                .collect(),
            previous_config: redact_config(&previous_config),
            applied_config: redact_config(&applied_config),
            previous_fingerprint,
            applied_fingerprint,
            config_backup: Some(backup.to_string_lossy().to_string()),
        })
    }

    /// Whether the change altered neither the server list nor any client file.
    ///
    /// Fingerprints cover secret values, so a secret-only edit is not a no-op
    /// even though the redacted copies look the same.
    pub fn is_noop(&self) -> bool {
        let configs_same =
            if self.previous_fingerprint.is_empty() && self.applied_fingerprint.is_empty() {
                same_config(&self.previous_config, &self.applied_config)
            } else {
                self.previous_fingerprint == self.applied_fingerprint
            };
        self.files.iter().all(|file| file.before == file.after) && configs_same
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

/// Missing or unreadable history is treated as empty. Does not rewrite the file.
fn load_raw(path: &Path) -> History {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// Loads history and, when an older record still holds plaintext secrets,
/// moves the previous config into a protected backup and redacts the record.
fn load(ctx: &PlatformContext) -> History {
    let path = path(ctx);
    store::with_lock(&path, || {
        let mut history = load_raw(&path);
        match sanitize(ctx, &mut history) {
            Ok(true) => save(&path, &history)?,
            Ok(false) => {}
            // Leave the file as it is. Redacting without a backup would drop
            // the only copy of the previous config.
            Err(_) => return Ok(load_raw(&path)),
        }
        Ok(history)
    })
    .unwrap_or_else(|_| load_raw(&path))
}

fn save(path: &Path, history: &History) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        // history.json sits in the application-data directory.
        let _ = crate::security::restrict_new_dir(parent);
    }
    let content = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    atomic_write(path, &content)
}

fn contains_plaintext_secret(config: &MCPConfig) -> bool {
    serde_json::to_value(config).ok() != serde_json::to_value(redact_config(config)).ok()
}

/// Spills legacy plaintext secrets into a backup, then redacts the record.
/// Returns whether `history` changed. A backup that cannot be written leaves
/// the record untouched so rollback data is not discarded.
fn sanitize(ctx: &PlatformContext, history: &mut History) -> Result<bool, String> {
    let mut changed = false;
    for change in &mut history.changes {
        let previous_secret = contains_plaintext_secret(&change.previous_config);
        let applied_secret = contains_plaintext_secret(&change.applied_config);
        if !previous_secret && !applied_secret {
            continue;
        }
        if change.config_backup.is_none() {
            if change.previous_fingerprint.is_empty() {
                change.previous_fingerprint = config_fingerprint(&change.previous_config)?;
            }
            if change.applied_fingerprint.is_empty() {
                change.applied_fingerprint = config_fingerprint(&change.applied_config)?;
            }
            let yaml = store::to_yaml(&change.previous_config)?;
            let backup = storage::write_internal_backup(&ctx.app_data_dir(), yaml.as_bytes())?;
            change.config_backup = Some(backup.to_string_lossy().to_string());
        }
        change.previous_config = redact_config(&change.previous_config);
        change.applied_config = redact_config(&change.applied_config);
        changed = true;
    }
    Ok(changed)
}

fn owned_backups(change: &ChangeRecord) -> Vec<PathBuf> {
    change
        .backups
        .iter()
        .chain(change.config_backup.iter())
        .map(PathBuf::from)
        .collect()
}

fn referenced_backups(history: &History) -> HashSet<PathBuf> {
    history.changes.iter().flat_map(owned_backups).collect()
}

/// Deletes backup files that `history` no longer names. Ordinary unlink only.
fn retire_backups(ctx: &PlatformContext, history: &History, retired: &[ChangeRecord]) {
    let app_data = ctx.app_data_dir();
    let referenced = referenced_backups(history);
    for change in retired {
        for backup in owned_backups(change) {
            if !referenced.contains(&backup) {
                let _ = storage::delete_owned_backup(&app_data, &backup);
            }
        }
    }
    let _ = storage::cleanup_orphan_backups(&app_data, &referenced, 200);
}

fn same_config(a: &MCPConfig, b: &MCPConfig) -> bool {
    serde_json::to_value(a).ok() == serde_json::to_value(b).ok()
}

/// `fingerprint` is of the full config. The stored config may be redacted, so
/// a missing fingerprint falls back to comparing the stored values.
fn matches_recorded(current: &MCPConfig, recorded: &MCPConfig, fingerprint: &str) -> bool {
    if !fingerprint.is_empty() {
        return config_fingerprint(current).ok().as_deref() == Some(fingerprint);
    }
    same_config(current, recorded)
}

fn full_previous(ctx: &PlatformContext, change: &ChangeRecord) -> Result<MCPConfig, String> {
    let Some(backup) = change.config_backup.as_deref() else {
        return Ok(change.previous_config.clone());
    };
    let bytes = storage::read_backup_bytes(&ctx.app_data_dir(), Path::new(backup))?;
    let text = String::from_utf8(bytes)
        .map_err(|error| format!("config backup is not valid UTF-8: {error}"))?;
    crate::parser::parse_yaml_config(&text)
}

/// Adds a change on top of the ones that can be undone. Call it while holding
/// the servers.yaml lock (inside `store::update_guarded`, or [`apply_recorded`]),
/// so no other change or rollback can slip in between saving and recording.
pub fn record(ctx: &PlatformContext, change: ChangeRecord) -> Result<(), String> {
    let path = path(ctx);
    store::with_lock(&path, || {
        let mut history = load_raw(&path);
        sanitize(ctx, &mut history)?;
        history.changes.push(change);
        let excess = history.changes.len().saturating_sub(MAX_CHANGES);
        let retired = history.changes.drain(..excess).collect::<Vec<_>>();
        save(&path, &history)?;
        retire_backups(ctx, &history, &retired);
        Ok(())
    })
}

/// How many changes can be undone.
pub fn depth(ctx: &PlatformContext) -> usize {
    load(ctx).changes.len()
}

/// Drops the newest record again, for a change that could not be saved after
/// it was recorded. Same locking as [`record`].
pub fn forget_latest(ctx: &PlatformContext) -> Result<(), String> {
    let path = path(ctx);
    store::with_lock(&path, || {
        let mut history = load_raw(&path);
        // Do not delete backups here. The caller may put this record back if
        // saving servers.yaml failed, and it still names those files.
        sanitize(ctx, &mut history)?;
        history.changes.pop();
        save(&path, &history)
    })
}

/// The change [`rollback_last`] would undo.
pub fn latest(ctx: &PlatformContext) -> Option<ChangeRecord> {
    load(ctx).changes.pop()
}

/// Applies `config` to every client like [`workflow::apply`] and records it
/// for undo. servers.yaml must already hold `config`: the caller saved it and
/// passes the fingerprint it got back as `expected_fingerprint`.
///
/// Runs under the servers.yaml lock and first checks that fingerprint, so a
/// change another program (such as the CLI) saved after the caller's is not
/// overwritten in the client files with the caller's older list. On a mismatch
/// nothing is applied and the error starts with [`store::CONFLICT_ERROR`].
pub fn apply_recorded(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: &MCPConfig,
    expected_fingerprint: Option<&str>,
) -> Result<ApplyResult, String> {
    let config_path = store::config_path(ctx);
    store::with_lock(&config_path, || {
        if let Some(expected) = expected_fingerprint {
            if store::fingerprint(&config_path)? != expected {
                return Err(format!(
                    "{}: {} was changed by another program before the change was applied",
                    store::CONFLICT_ERROR,
                    config_path.to_string_lossy()
                ));
            }
        }
        let snapshot = workflow::snapshot(ctx, config, Some(previous_config))?;
        let mut result = workflow::apply(ctx, config, Some(previous_config))?;
        // The change is applied, so failing to record it is not an error: it
        // only costs the undo, which the caller is told about through
        // `history_error`.
        let recorded = ChangeRecord::new(
            &ctx.app_data_dir(),
            &snapshot,
            result.backups.clone(),
            previous_config.clone(),
            config.clone(),
        )
        .and_then(|change| {
            if change.is_noop() {
                return Ok(());
            }
            record(ctx, change)
        });
        result.history_error = recorded.err();
        Ok(result)
    })
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

/// Whether two paths name the same file: the app records them as it built
/// them and backups map back with the platform's separators, so on Windows
/// `/` versus `\\` and letter case must not matter.
fn same_path(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        let normalize = |p: &str| p.replace('\\', "/").to_ascii_lowercase();
        normalize(a) == normalize(b)
    } else {
        a == b
    }
}

/// Undoes the most recent change: client files and servers.yaml. Fails,
/// changing nothing, if either was edited since the change.
pub fn rollback_last(ctx: &PlatformContext) -> Result<RollbackReport, String> {
    let mut report = RollbackReport {
        restored: 0,
        removed: 0,
        remaining: 0,
    };
    // Read and removed under the servers.yaml lock, like every record, so it
    // is the change that was undone and not one recorded meanwhile.
    let undone = std::cell::RefCell::new(None);
    let retired_backups = std::cell::RefCell::new(Vec::new());

    store::update_guarded(
        ctx,
        |config| {
            let change = latest(ctx).ok_or_else(|| "Nothing to roll back.".to_string())?;
            // Undoing on top of a later edit would silently discard it.
            // Fingerprints include secret values; the stored configs do not.
            if !matches_recorded(config, &change.applied_config, &change.applied_fingerprint)
                && !matches_recorded(
                    config,
                    &change.previous_config,
                    &change.previous_fingerprint,
                )
            {
                return Err(
                    "The server list changed after the last change made here (in MCP Manager or \
                     another command), so rolling back would discard that edit. Nothing was \
                     changed."
                        .to_string(),
                );
            }
            let previous = full_previous(ctx, &change)?;
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

            // Every backup must be there, and be what the file was, before the
            // first file is touched.
            let mut to_restore = Vec::new();
            for backup in &change.backups {
                let backup = PathBuf::from(backup);
                let target = storage::backup_target(&backup)?;
                let target_key = target.to_string_lossy().to_string();
                let file = change
                    .files
                    .iter()
                    .find(|f| same_path(&f.path, &target_key))
                    .ok_or_else(|| {
                        format!(
                            "The backup {} belongs to no file of this change. Nothing was changed.",
                            backup.to_string_lossy()
                        )
                    })?;
                let current = store::fingerprint(&target)?;
                if current == file.after && file.before != file.after {
                    let bytes =
                        storage::read_backup_bytes(&ctx.app_data_dir(), &backup).map_err(|e| {
                            format!(
                                "The backup {} cannot be read ({e}), so {target_key} cannot be \
                             restored. Nothing was changed.",
                                backup.to_string_lossy()
                            )
                        })?;
                    if store::fingerprint_of_bytes(&bytes).ok().as_deref()
                        != Some(file.before.as_str())
                    {
                        return Err(format!(
                            "The backup {} is not the file as it was before this change, so \
                             {target_key} cannot be restored. Nothing was changed.",
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

            *config = previous;
            // Forgotten only now that everything is back; if servers.yaml
            // cannot be saved after all, the record is put back below and
            // running the rollback again finishes without touching anything.
            // Backups are deleted only after that save succeeds.
            forget_latest(ctx)?;
            report.remaining = depth(ctx);
            *retired_backups.borrow_mut() = owned_backups(&change);
            *undone.borrow_mut() = Some(change);
            Ok(())
        },
        |error| match undone.borrow_mut().take() {
            Some(change) => match record(ctx, change) {
                Ok(()) => error,
                Err(record_error) => format!(
                    "{error}. The change could not be put back in the history either \
                     ({record_error}); run the rollback again to finish it."
                ),
            },
            None => error,
        },
    )?;
    let history = load(ctx);
    let referenced = referenced_backups(&history);
    let app_data = ctx.app_data_dir();
    for backup in retired_backups.into_inner() {
        if !referenced.contains(&backup) {
            let _ = storage::delete_owned_backup(&app_data, &backup);
        }
    }
    let _ = storage::cleanup_orphan_backups(&app_data, &referenced, 200);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{apply_recorded, depth, path};
    use crate::core::MCPConfig;
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::store;
    use std::fs;
    use tempfile::tempdir;

    fn ctx(root: &std::path::Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::Linux,
            home_dir: root.join("home"),
            workspace_root: root.join("workspace"),
        }
    }

    fn config(version: u32) -> MCPConfig {
        MCPConfig {
            version,
            servers: vec![],
        }
    }

    #[test]
    fn records_an_applied_change_for_undo() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());

        let result = apply_recorded(&ctx, &config(2), &config(1), None).expect("apply");
        assert_eq!(result.history_error, None);
        assert_eq!(depth(&ctx), 1);
    }

    #[test]
    fn applies_when_servers_yaml_is_still_what_the_caller_saved() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        let saved =
            store::write_text_checked(&store::config_path(&ctx), "version: 1\nservers: []\n", None)
                .expect("save");

        apply_recorded(&ctx, &config(2), &config(1), Some(&saved)).expect("apply");
        assert_eq!(depth(&ctx), 1);
    }

    #[test]
    fn refuses_when_another_program_saved_servers_yaml_since() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        let path = store::config_path(&ctx);
        let saved =
            store::write_text_checked(&path, "version: 1\nservers: []\n", None).expect("save");
        // The CLI saves its own change before the app applies.
        store::write_text_checked(
            &path,
            "# edited by the CLI\nversion: 1\nservers: []\n",
            None,
        )
        .expect("save");

        let error =
            apply_recorded(&ctx, &config(2), &config(1), Some(&saved)).expect_err("conflict");
        assert!(error.starts_with(store::CONFLICT_ERROR), "{error}");
        assert_eq!(depth(&ctx), 0, "nothing was applied or recorded");
    }

    #[test]
    fn reports_a_change_it_could_not_record_without_failing_the_apply() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        // A directory in place of history.json makes saving the record fail.
        fs::create_dir_all(path(&ctx).join("blocked")).expect("dir");

        let result = apply_recorded(&ctx, &config(2), &config(1), None).expect("apply");
        assert!(result.history_error.is_some());
        assert_eq!(depth(&ctx), 0);
    }

    const SECRET: &str = "ghp_SUPERSECRETVALUE1234567890";

    fn secret_config() -> MCPConfig {
        use crate::core::{CommandSpec, MCPServer, TransportSpec};
        use std::collections::{BTreeSet, HashMap};
        MCPConfig {
            version: 1,
            servers: vec![MCPServer {
                description: None,
                homepage: None,
                id: "demo".to_string(),
                name: "Demo".to_string(),
                enabled: true,
                transport: TransportSpec {
                    kind: "stdio".to_string(),
                    url: None,
                    headers: Default::default(),
                },
                command: Some(CommandSpec {
                    program: "npx".to_string(),
                    args: vec![],
                    env: HashMap::from([("API_TOKEN".to_string(), SECRET.to_string())]),
                    secret_env: BTreeSet::new(),
                }),
                apps: HashMap::new(),
                placements: vec![],
            }],
        }
    }

    fn write_config(ctx: &PlatformContext, config: &MCPConfig) {
        let yaml = store::to_yaml(config).expect("yaml");
        store::write_text_checked(&store::config_path(ctx), &yaml, None).expect("save");
    }

    fn bak_files(ctx: &PlatformContext) -> Vec<std::path::PathBuf> {
        let root = ctx.app_data_dir().join("backups");
        let mut found = Vec::new();
        let mut dirs = vec![root];
        while let Some(dir) = dirs.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("bak") {
                    found.push(path);
                }
            }
        }
        found
    }

    #[test]
    fn history_json_does_not_keep_plaintext_secrets_and_rollback_restores_them() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        let previous = secret_config();
        let applied = config(1);
        write_config(&ctx, &applied);
        apply_recorded(&ctx, &applied, &previous, None).expect("apply");

        let history_path = path(&ctx);
        let history_text = fs::read_to_string(&history_path).expect("history");
        assert!(
            !history_text.contains(SECRET),
            "history.json retained a secret: {history_text}"
        );
        let history: serde_json::Value = serde_json::from_str(&history_text).expect("json");
        let backup = history["changes"][0]["configBackup"]
            .as_str()
            .expect("config backup")
            .to_string();
        assert!(std::path::Path::new(&backup).is_file());
        let backup_text = fs::read_to_string(&backup).expect("backup");
        assert!(
            backup_text.contains(SECRET),
            "rollback backup must keep the secret"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |path: &std::path::Path| {
                fs::metadata(path).expect("meta").permissions().mode() & 0o777
            };
            assert_eq!(mode(std::path::Path::new(&backup)), 0o600);
            assert_eq!(mode(&history_path), 0o600);
        }

        super::rollback_last(&ctx).expect("rollback");
        let restored = fs::read_to_string(store::config_path(&ctx)).expect("servers.yaml");
        assert!(restored.contains(SECRET), "{restored}");
        assert!(!std::path::Path::new(&backup).exists());
        let after = fs::read_to_string(&history_path).unwrap_or_default();
        assert!(!after.contains(SECRET), "{after}");
    }

    #[test]
    fn rolling_history_deletes_backups_owned_only_by_expired_entries() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        for version in 1..=super::MAX_CHANGES + 1 {
            apply_recorded(
                &ctx,
                &config(version as u32 + 1),
                &config(version as u32),
                None,
            )
            .expect("apply");
        }
        assert_eq!(depth(&ctx), super::MAX_CHANGES);
        assert_eq!(bak_files(&ctx).len(), super::MAX_CHANGES);
    }

    #[test]
    fn legacy_history_with_plaintext_secrets_is_redacted_on_load() {
        let dir = tempdir().expect("tempdir");
        let ctx = ctx(dir.path());
        let previous = secret_config();
        let applied = config(1);
        write_config(&ctx, &applied);
        let legacy = super::History {
            changes: vec![super::ChangeRecord {
                backups: vec![],
                files: vec![],
                created_dirs: vec![],
                previous_config: previous,
                applied_config: applied,
                previous_fingerprint: String::new(),
                applied_fingerprint: String::new(),
                config_backup: None,
            }],
        };
        super::save(&path(&ctx), &legacy).expect("seed history");
        assert_eq!(depth(&ctx), 1);
        let history_text = fs::read_to_string(path(&ctx)).expect("history");
        assert!(!history_text.contains(SECRET), "{history_text}");

        super::rollback_last(&ctx).expect("rollback");
        let restored = fs::read_to_string(store::config_path(&ctx)).expect("servers.yaml");
        assert!(restored.contains(SECRET), "{restored}");
    }
}
