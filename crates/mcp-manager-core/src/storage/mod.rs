use crate::core::WriteOperation;
use crate::platform::PlatformContext;
use chrono::Utc;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use toml::Table;

fn base_dir() -> PathBuf {
    PlatformContext::current().app_data_dir()
}

pub fn resolve_path(path: &str) -> PathBuf {
    PlatformContext::current().resolve_path(path)
}

pub fn resolve_relative_path(relative_path: &str) -> Result<PathBuf, String> {
    crate::security::resolve_internal_relative(relative_path)
}

pub fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Mirrors an absolute directory under the backup root. Joining the absolute
/// path directly would replace the root, so the root and drive are turned into
/// plain components: `/a/b` becomes `a/b`, `C:\a\b` becomes `C/a/b`.
fn backup_relative_dir(dir: &Path) -> PathBuf {
    dir.components()
        .filter_map(|component| match component {
            Component::Prefix(prefix) => Some(match prefix.kind() {
                Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                    char::from(letter).to_string().into()
                }
                _ => prefix
                    .as_os_str()
                    .to_string_lossy()
                    .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
                    .into(),
            }),
            Component::Normal(part) => Some(part.to_os_string()),
            _ => None,
        })
        .collect()
}

/// Inverse of [`backup_relative_dir`].
fn restore_dir(relative: &Path) -> PathBuf {
    if cfg!(windows) {
        let mut components = relative.components();
        let drive = components
            .next()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .unwrap_or_default();
        PathBuf::from(format!("{drive}:\\")).join(components.as_path())
    } else {
        Path::new("/").join(relative)
    }
}

pub fn backup_file(target: &Path) -> Result<Option<String>, String> {
    if !target.exists() {
        return Ok(None);
    }

    let backup_parent = base_dir().join("backups").join(backup_relative_dir(
        target.parent().unwrap_or(Path::new("")),
    ));
    fs::create_dir_all(&backup_parent).map_err(|e| e.to_string())?;
    // Backups are MCP Manager files and hold config contents. Keep them
    // private even when the client file itself was group- or world-readable.
    let _ = crate::security::restrict_new_dir(&backup_parent);
    if let Some(root) = backup_parent.parent() {
        let _ = crate::security::restrict_new_dir(root);
    }

    let file_name = target
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| "invalid target file name".to_string())?;

    // Two backups of one file in the same second must not share a name, or
    // the second would overwrite the first. No dots: the stamp is what follows
    // the last dot of the name (see `backup_target`).
    static NEXT_BACKUP_ID: AtomicU64 = AtomicU64::new(0);
    let stamp = format!(
        "{}-{}",
        Utc::now().format("%Y%m%dT%H%M%S%3f"),
        NEXT_BACKUP_ID.fetch_add(1, Ordering::Relaxed)
    );
    let backup = backup_parent.join(format!("{}.{}.bak", file_name, stamp));
    fs::copy(target, &backup).map_err(|e| e.to_string())?;
    crate::security::restrict_new_file(&backup)?;

    Ok(Some(backup.to_string_lossy().to_string()))
}

pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    atomic_write_bytes(path, content.as_bytes())
}

/// Writes to a temporary file and renames it over `path`, so a crash never
/// leaves `path` truncated.
pub fn atomic_write_bytes(path: &Path, content: &[u8]) -> Result<(), String> {
    // Unique per write, so concurrent writes (in this process or another)
    // never share a temporary file, even for `config.json` and `config.toml`.
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
    ensure_parent(path)?;
    crate::security::tighten_app_data_dirs(path);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(
        ".{TEMP_MARKER}{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let tmp = PathBuf::from(tmp);
    remove_stale_temp_files(path);
    let written = (|| {
        crate::security::atomic_replace_file(&tmp, path, content)?;
        fs::rename(&tmp, path).map_err(|e| e.to_string())
    })();
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    written
}

/// Marks a temporary file as this app's own, so cleanup never deletes another
/// program's file that happens to end in `.<digits>.<digits>.tmp`. It is part of
/// the name, so it survives the crash that left the file behind.
const TEMP_MARKER: &str = "mcp-manager-";

/// How old a leftover `<file>.mcp-manager-<pid>-<n>.tmp` must be before a later
/// write deletes it, so a write still in progress in another process is never hit.
const STALE_TEMP_AGE: Duration = Duration::from_secs(60 * 60);

/// Deletes temporary files of `path` that a killed process left behind
/// between writing and renaming. Best effort: failures are ignored.
fn remove_stale_temp_files(path: &Path) {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str()))
    else {
        return;
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    let prefix = format!("{name}.{TEMP_MARKER}");
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let is_temp = file_name
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix(".tmp"))
            .and_then(|ids| ids.split_once('-'))
            .is_some_and(|(pid, n)| {
                !pid.is_empty()
                    && !n.is_empty()
                    && pid.bytes().all(|b| b.is_ascii_digit())
                    && n.bytes().all(|b| b.is_ascii_digit())
            });
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .is_some_and(|age| age >= STALE_TEMP_AGE);
        if is_temp && stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Files as they were at one moment, to put them back after a failed change.
/// A file that did not exist is removed on restore, and so are the
/// directories that had to be created for it (when they are left empty).
#[derive(Debug, Default)]
pub struct Snapshot {
    files: Vec<(PathBuf, Option<Vec<u8>>)>,
    /// Directories that did not exist, parents before children.
    new_dirs: Vec<PathBuf>,
}

impl Snapshot {
    /// Records `path` unless it is already recorded, so the first state wins.
    pub fn capture(&mut self, path: &Path) -> Result<(), String> {
        if self.files.iter().any(|(seen, _)| seen == path) {
            return Ok(());
        }
        let content = if path.exists() {
            Some(fs::read(path).map_err(|e| e.to_string())?)
        } else {
            let mut missing = Vec::new();
            let mut dir = path.parent();
            while let Some(current) = dir.filter(|d| !d.as_os_str().is_empty() && !d.exists()) {
                missing.push(current.to_path_buf());
                dir = current.parent();
            }
            for dir in missing.into_iter().rev() {
                if !self.new_dirs.contains(&dir) {
                    self.new_dirs.push(dir);
                }
            }
            None
        };
        self.files.push((path.to_path_buf(), content));
        Ok(())
    }

    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.files.iter().map(|(path, _)| path.as_path())
    }

    /// Files that did not exist when captured.
    pub fn created_files(&self) -> Vec<PathBuf> {
        self.files
            .iter()
            .filter(|(_, content)| content.is_none())
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Directories that did not exist when a file in them was captured.
    pub fn created_dirs(&self) -> &[PathBuf] {
        &self.new_dirs
    }

    /// Each captured file with the fingerprint (see `store::fingerprint`) it
    /// had when captured.
    pub fn fingerprints(&self) -> Result<Vec<(PathBuf, String)>, String> {
        self.files
            .iter()
            .map(|(path, content)| {
                let fingerprint = match content {
                    Some(bytes) => crate::store::fingerprint_of_bytes(bytes)?,
                    None => crate::store::ABSENT_FINGERPRINT.to_string(),
                };
                Ok((path.clone(), fingerprint))
            })
            .collect()
    }

    /// Puts every recorded file back, newest first, and reports each failure.
    pub fn restore(&self) -> Result<(), String> {
        let mut failures = self
            .files
            .iter()
            .rev()
            .filter_map(|(path, original)| {
                let restored = match original {
                    Some(content) => atomic_write_bytes(path, content),
                    None if path.exists() => fs::remove_file(path).map_err(|e| e.to_string()),
                    None => Ok(()),
                };
                restored
                    .err()
                    .map(|e| format!("{}: {e}", path.to_string_lossy()))
            })
            .collect::<Vec<_>>();
        remove_empty_dirs(&self.new_dirs);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut failures).join("; "))
        }
    }
}

/// Removes each directory (children before parents) that is now empty.
/// Others are left alone: something else put files there.
pub fn remove_empty_dirs(dirs: &[PathBuf]) {
    for dir in dirs.iter().rev() {
        let _ = fs::remove_dir(dir);
    }
}

fn json_value_to_toml(value: &Value) -> Result<toml::Value, String> {
    match value {
        Value::Null => Err("null cannot be represented in TOML".to_string()),
        Value::Bool(v) => Ok(toml::Value::Boolean(*v)),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = v.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err("unsupported JSON number".to_string())
            }
        }
        Value::String(v) => Ok(toml::Value::String(v.clone())),
        Value::Array(values) => {
            let mut items = Vec::with_capacity(values.len());
            for value in values {
                items.push(json_value_to_toml(value)?);
            }
            Ok(toml::Value::Array(items))
        }
        Value::Object(map) => {
            let mut table = Table::new();
            for (key, value) in map {
                table.insert(key.clone(), json_value_to_toml(value)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

fn apply_replace_json(path: &Path, content: &str) -> Result<(), String> {
    let parsed: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let pretty = serde_json::to_string_pretty(&parsed).map_err(|e| e.to_string())?;
    atomic_write(path, &pretty)
}

/// Parses an existing client file. Empty files start as `{}`. Malformed JSON
/// is an error so a bad file is not replaced with only the field being merged.
/// Comments are stripped first because several clients store JSONC.
fn read_json_host(path: &Path, existing: &str) -> Result<Value, String> {
    if existing.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let parsed = serde_json::from_str::<Value>(existing)
        .or_else(|_| serde_json::from_str::<Value>(&crate::parser::strip_json_comments(existing)));
    parsed.map_err(|error| {
        format!(
            "{} is not valid JSON ({error}); the file was not changed",
            path.to_string_lossy()
        )
    })
}

fn apply_merge_json_field(path: &Path, field: &str, content: &str) -> Result<(), String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        "{}".to_string()
    };
    let mut host = read_json_host(path, &existing)?;
    let Some(host_map) = host.as_object_mut() else {
        return Err(format!(
            "{} does not contain a JSON object",
            path.to_string_lossy()
        ));
    };
    host_map.insert(field.to_string(), field_value);
    let pretty = serde_json::to_string_pretty(&host).map_err(|e| e.to_string())?;
    atomic_write(path, &pretty)
}

fn apply_merge_json_object_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<(), String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let Some(new_entries) = field_value.as_object() else {
        return Err(format!("{field} merge content must be a JSON object"));
    };

    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        "{}".to_string()
    };
    let mut host = read_json_host(path, &existing)?;
    let Some(host_map) = host.as_object_mut() else {
        return Err(format!(
            "{} does not contain a JSON object",
            path.to_string_lossy()
        ));
    };

    let field_entry = host_map
        .entry(field.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(target_map) = field_entry.as_object_mut() else {
        return Err(format!(
            "{} field {} does not contain a JSON object",
            path.to_string_lossy(),
            field
        ));
    };

    if let Some(keys) = remove_keys {
        for key in keys {
            target_map.remove(key);
        }
    }

    for (key, value) in new_entries {
        target_map.insert(key.clone(), value.clone());
    }

    let pretty = serde_json::to_string_pretty(&host).map_err(|e| e.to_string())?;
    atomic_write(path, &pretty)
}

fn apply_merge_toml_field(path: &Path, field: &str, content: &str) -> Result<(), String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };
    let mut host: Table = if existing.trim().is_empty() {
        Table::new()
    } else {
        toml::from_str(&existing).map_err(|e| e.to_string())?
    };
    host.insert(field.to_string(), json_value_to_toml(&field_value)?);
    let rendered = toml::to_string_pretty(&host).map_err(|e| e.to_string())?;
    atomic_write(path, &rendered)
}

fn apply_merge_toml_table_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<(), String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let Some(new_entries) = field_value.as_object() else {
        return Err(format!("{field} merge content must be a JSON object"));
    };

    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };
    let mut host: Table = if existing.trim().is_empty() {
        Table::new()
    } else {
        toml::from_str(&existing).map_err(|e| e.to_string())?
    };

    let field_entry = host
        .entry(field.to_string())
        .or_insert_with(|| toml::Value::Table(Table::new()));
    let Some(target_table) = field_entry.as_table_mut() else {
        return Err(format!(
            "{} field {} does not contain a TOML table",
            path.to_string_lossy(),
            field
        ));
    };

    if let Some(keys) = remove_keys {
        for key in keys {
            target_table.remove(key);
        }
    }

    for (key, value) in new_entries {
        target_table.insert(key.clone(), json_value_to_toml(value)?);
    }

    let rendered = toml::to_string_pretty(&host).map_err(|e| e.to_string())?;
    atomic_write(path, &rendered)
}

/// Applies one write to `path` without taking a backup.
pub fn apply_operation(path: &Path, item: &WriteOperation) -> Result<(), String> {
    let field = |kind: &str| {
        item.field
            .as_deref()
            .ok_or_else(|| format!("missing {kind} merge field"))
    };
    match item.mode.as_str() {
        "replace_json" => apply_replace_json(path, &item.content),
        "merge_json_field" => apply_merge_json_field(path, field("JSON")?, &item.content),
        "merge_json_object_entries" => apply_merge_json_object_entries(
            path,
            field("JSON")?,
            &item.content,
            item.remove_keys.as_deref(),
        ),
        "merge_toml_field" => apply_merge_toml_field(path, field("TOML")?, &item.content),
        "merge_toml_table_entries" => apply_merge_toml_table_entries(
            path,
            field("TOML")?,
            &item.content,
            item.remove_keys.as_deref(),
        ),
        other => Err(format!("unsupported artifact mode: {other}")),
    }
}

/// Applies every operation, backing up each existing file first. If one
/// fails, every file already written by this batch is put back as it was
/// (and files it created are removed), so clients never end up half-applied.
pub fn apply_operations(artifacts: Vec<WriteOperation>) -> Result<Vec<String>, String> {
    let mut backups = Vec::new();
    let mut originals = Snapshot::default();

    for item in artifacts {
        let path = resolve_path(&item.path);
        let result = (|| {
            originals.capture(&path)?;
            if let Some(backup) = backup_file(&path)? {
                backups.push(backup);
            }
            apply_operation(&path, &item)
        })();

        if let Err(error) = result {
            return Err(match originals.restore() {
                Ok(()) => error,
                Err(restore_error) => format!(
                    "{error}; restoring the files written before it also failed: {restore_error}"
                ),
            });
        }
    }

    Ok(backups)
}

/// The file a backup made by [`backup_file`] was taken of.
pub fn backup_target(backup: &Path) -> Result<PathBuf, String> {
    let backup_root = base_dir().join("backups");
    let relative = backup
        .strip_prefix(&backup_root)
        .map_err(|_| "backup is outside backup root".to_string())?;

    let file_name = relative
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| "invalid backup file name".to_string())?;

    // filename format: <original>.<stamp>.bak
    let original_name = file_name
        .strip_suffix(".bak")
        .and_then(|name| name.rsplit_once('.'))
        .map(|(original, _stamp)| original)
        .ok_or_else(|| "invalid backup name format".to_string())?;

    Ok(restore_dir(relative.parent().unwrap_or(Path::new(""))).join(original_name))
}

/// Copies one backup over the file it was taken of.
///
/// The backup path has to stay inside the backup store after symlink
/// resolution, and the restored file has to be a modeled client config path.
pub fn restore_backup(backup: &Path) -> Result<(), String> {
    let target = backup_target(backup)?;
    let ctx = PlatformContext::current();
    crate::security::validate_client_config_path(&ctx, &target.to_string_lossy())?;
    let content = read_backup_bytes(&ctx.app_data_dir(), backup)?;
    atomic_write_bytes(&target, &content)
}

/// Writes `bytes` under `app_data/backups/internal` as a private file.
/// History uses this for the previous `servers.yaml`, so rollback does not
/// need the secrets to live in `history.json`.
pub fn write_internal_backup(app_data: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let dir = app_data.join("backups").join("internal");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let _ = crate::security::restrict_new_dir(app_data);
    let _ = crate::security::restrict_new_dir(&app_data.join("backups"));
    crate::security::restrict_new_dir(&dir)?;
    static NEXT_INTERNAL_ID: AtomicU64 = AtomicU64::new(0);
    let name = format!(
        "servers.yaml.{}-{}.bak",
        Utc::now().format("%Y%m%dT%H%M%S%3f"),
        NEXT_INTERNAL_ID.fetch_add(1, Ordering::Relaxed)
    );
    let path = dir.join(name);
    atomic_write_bytes(&path, bytes)?;
    Ok(path)
}

/// Reads a backup file. Symlinks are refused so a planted link cannot make a
/// restore or a history migration copy an arbitrary file.
pub fn read_backup_bytes(app_data: &Path, candidate: &Path) -> Result<Vec<u8>, String> {
    let path = contained_backup(app_data, candidate, false)?;
    fs::read(&path).map_err(|e| e.to_string())
}

/// Deletes one MCP Manager backup. A symlink is removed as a link and its
/// target is left in place. Paths outside the backup root are rejected.
pub fn delete_owned_backup(app_data: &Path, candidate: &Path) -> Result<(), String> {
    let path = contained_backup(app_data, candidate, true)?;
    if path.exists() || path.symlink_metadata().is_ok() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Removes `*.bak` files under the backup root that `referenced` does not name.
/// At most `limit` files are removed. This is ordinary deletion: it does not
/// promise that an SSD has forgotten the bytes.
pub fn cleanup_orphan_backups(
    app_data: &Path,
    referenced: &HashSet<PathBuf>,
    limit: usize,
) -> Result<usize, String> {
    let root = app_data.join("backups");
    if !root.exists() {
        return Ok(0);
    }
    let mut removed = 0usize;
    let mut dirs = vec![root];
    while let Some(dir) = dirs.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => return Err(error.to_string()),
        };
        for entry in entries.flatten() {
            if removed >= limit {
                return Ok(removed);
            }
            let path = entry.path();
            let meta = match fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.file_type().is_symlink() {
                if is_backup_name(&path) && !referenced.contains(&path) {
                    // `remove_file` on a symlink deletes the link, not the target.
                    fs::remove_file(&path).map_err(|e| e.to_string())?;
                    removed += 1;
                }
                continue;
            }
            if meta.is_dir() {
                dirs.push(path);
                continue;
            }
            if meta.is_file() && is_backup_name(&path) && !referenced.contains(&path) {
                delete_owned_backup(app_data, &path)?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}

fn is_backup_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".bak"))
}

/// `allow_symlink` is for deletion, which unlinks the symlink itself.
/// Reads pass `false` and refuse the link.
fn contained_backup(
    app_data: &Path,
    candidate: &Path,
    allow_symlink: bool,
) -> Result<PathBuf, String> {
    if candidate
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err("backup path traversal rejected".to_string());
    }
    let root = app_data.join("backups");
    if !candidate.starts_with(&root) {
        return Err(format!(
            "backup path is outside the backup store: {}",
            candidate.display()
        ));
    }
    let meta = fs::symlink_metadata(candidate).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        if allow_symlink {
            return Ok(candidate.to_path_buf());
        }
        return Err("refusing to follow a symlink in the backup store".to_string());
    }
    if root.exists() {
        if let Ok(root_canon) = root.canonicalize() {
            if let Ok(canon) = candidate.canonicalize() {
                if !canon.starts_with(&root_canon) {
                    return Err(
                        "backup path escapes the backup store after canonicalization".to_string(),
                    );
                }
            }
        }
    }
    Ok(candidate.to_path_buf())
}

pub fn rollback(backups: Vec<String>) -> Result<(), String> {
    for backup in backups {
        let src = PathBuf::from(&backup);
        if !src.exists() {
            continue;
        }
        restore_backup(&src)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{apply_operations, backup_file, resolve_relative_path, rollback};
    use crate::core::WriteOperation;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[cfg(target_os = "windows")]
    const HOME_ENV_VAR: &str = "USERPROFILE";

    #[cfg(not(target_os = "windows"))]
    const HOME_ENV_VAR: &str = "HOME";

    fn expected_app_data_dir(home: &Path) -> PathBuf {
        if cfg!(target_os = "macos") {
            home.join("Library/Application Support/mcp-manager")
        } else if cfg!(target_os = "windows") {
            home.join("AppData/Roaming/mcp-manager")
        } else {
            home.join(".config/mcp-manager")
        }
    }

    fn set_test_runtime(home: &Path, current_dir: &Path) -> (Option<OsString>, PathBuf) {
        let previous_home = std::env::var_os(HOME_ENV_VAR);
        let previous_dir = std::env::current_dir().expect("current dir");
        std::env::set_var(HOME_ENV_VAR, home);
        std::env::set_current_dir(current_dir).expect("set current dir");
        (previous_home, previous_dir)
    }

    fn restore_test_runtime(previous_home: Option<OsString>, previous_dir: PathBuf) {
        std::env::set_current_dir(previous_dir).expect("restore current dir");
        match previous_home {
            Some(value) => std::env::set_var(HOME_ENV_VAR, value),
            None => std::env::remove_var(HOME_ENV_VAR),
        }
    }

    #[test]
    fn a_failed_batch_puts_every_written_file_back() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let existing = dir.path().join("existing.json");
        let created = dir.path().join("created/mcp.json");
        fs::write(&existing, r#"{"mcpServers":{"old":{"command":"old"}}}"#).expect("seed");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());

        let merge = |path: &Path| WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_json_object_entries".to_string(),
            field: Some("mcpServers".to_string()),
            remove_keys: Some(vec!["old".to_string()]),
            content: r#"{"new":{"command":"npx"}}"#.to_string(),
        };
        let result = apply_operations(vec![
            merge(&existing),
            merge(&created),
            WriteOperation {
                mode: "not_a_mode".to_string(),
                ..merge(&existing)
            },
        ]);
        restore_test_runtime(previous_home, previous_dir);

        assert!(result.unwrap_err().contains("not_a_mode"));
        assert_eq!(
            fs::read_to_string(&existing).expect("read"),
            r#"{"mcpServers":{"old":{"command":"old"}}}"#
        );
        assert!(!created.exists());
        assert!(
            !created.parent().unwrap().exists(),
            "the folder made for it is removed too"
        );
    }

    #[test]
    fn backups_taken_in_the_same_second_keep_their_own_content() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let target = dir.path().join("mcp.json");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());

        fs::write(&target, "first").expect("seed");
        let first = backup_file(&target).expect("backup").expect("exists");
        fs::write(&target, "second").expect("seed");
        let second = backup_file(&target).expect("backup").expect("exists");
        restore_test_runtime(previous_home, previous_dir);

        assert_ne!(first, second);
        assert_eq!(fs::read_to_string(first).expect("read"), "first");
        assert_eq!(fs::read_to_string(second).expect("read"), "second");
    }

    #[test]
    fn a_write_removes_only_old_leftover_temp_files() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let target = dir.path().join("mcp.json");
        let old = dir.path().join("mcp.json.mcp-manager-4242-0.tmp");
        let recent = dir.path().join("mcp.json.mcp-manager-4242-1.tmp");
        let unrelated = dir.path().join("mcp.json.notes.tmp");
        // Same digits pattern but written by some other program: not ours to delete.
        let foreign = dir.path().join("mcp.json.4242.0.tmp");
        for file in [&old, &recent, &unrelated, &foreign] {
            fs::write(file, "x").expect("seed");
        }
        let two_hours_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
        for file in [&old, &unrelated, &foreign] {
            fs::File::options()
                .write(true)
                .open(file)
                .and_then(|f| f.set_modified(two_hours_ago))
                .expect("age file");
        }

        super::atomic_write(&target, "{}").expect("write");

        assert!(!old.exists());
        assert!(recent.exists(), "might belong to a write in progress");
        assert!(unrelated.exists(), "not one of our temp file names");
        assert!(foreign.exists(), "lacks the mcp-manager marker");
        assert_eq!(fs::read_to_string(&target).expect("read"), "{}");
    }

    #[test]
    fn preserves_unrelated_json_fields_when_merging() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let path = dir.path().join("claude.json");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());
        fs::write(
            &path,
            r#"{"theme":"dark","mcpServers":{"old":{"command":"old"}}}"#,
        )
        .expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_json_field".to_string(),
            field: Some("mcpServers".to_string()),
            remove_keys: None,
            content: r#"{"new":{"command":"npx"}}"#.to_string(),
        }])
        .expect("apply");
        restore_test_runtime(previous_home, previous_dir);

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("\"theme\": \"dark\""));
        assert!(next.contains("\"new\""));
    }

    #[test]
    fn merges_json_object_entries_without_removing_unknown_servers() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let path = dir.path().join("claude.json");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());
        fs::write(
            &path,
            r#"{"theme":"dark","mcpServers":{"unknown":{"command":"keep"},"managed":{"command":"old"}}}"#,
        )
        .expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_json_object_entries".to_string(),
            field: Some("mcpServers".to_string()),
            remove_keys: None,
            content: r#"{"managed":{"command":"new"},"fresh":{"command":"npx"}}"#.to_string(),
        }])
        .expect("apply");
        restore_test_runtime(previous_home, previous_dir);

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("\"theme\": \"dark\""));
        assert!(next.contains("\"unknown\""));
        assert!(next.contains("\"managed\""));
        assert!(next.contains("\"fresh\""));
        assert!(!next.contains("\"command\": \"old\""));
    }

    #[test]
    fn removes_selected_json_object_entries_before_merging() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let path = dir.path().join("claude.json");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());
        fs::write(
            &path,
            r#"{"mcpServers":{"unknown":{"command":"keep"},"legacy":{"command":"legacy"}}}"#,
        )
        .expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_json_object_entries".to_string(),
            field: Some("mcpServers".to_string()),
            remove_keys: Some(vec!["legacy".to_string(), "managed".to_string()]),
            content: r#"{"managed":{"command":"new"}}"#.to_string(),
        }])
        .expect("apply");
        restore_test_runtime(previous_home, previous_dir);

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("\"unknown\""));
        assert!(next.contains("\"managed\""));
        assert!(!next.contains("\"legacy\""));
    }

    #[test]
    fn preserves_unrelated_toml_fields_when_merging() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let path = dir.path().join("config.toml");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());
        fs::write(&path, "model = \"gpt-5.4\"\n").expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_toml_field".to_string(),
            field: Some("mcp_servers".to_string()),
            remove_keys: None,
            content: r#"{"playwright":{"command":"npx","args":["@playwright/mcp@latest"]}}"#
                .to_string(),
        }])
        .expect("apply");
        restore_test_runtime(previous_home, previous_dir);

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("model = \"gpt-5.4\""));
        assert!(next.contains("[mcp_servers.playwright]"));
    }

    #[test]
    fn merges_toml_table_entries_without_removing_unknown_servers() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = tempfile::tempdir().expect("tmpdir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let path = dir.path().join("config.toml");
        let (previous_home, previous_dir) = set_test_runtime(&home, dir.path());
        fs::write(
            &path,
            "[mcp_servers.unknown]\ncommand = \"keep\"\n\n[mcp_servers.managed]\ncommand = \"old\"\n",
        )
        .expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_toml_table_entries".to_string(),
            field: Some("mcp_servers".to_string()),
            remove_keys: Some(vec!["managed".to_string()]),
            content: r#"{"managed":{"command":"new"},"fresh":{"command":"npx"}}"#.to_string(),
        }])
        .expect("apply");
        restore_test_runtime(previous_home, previous_dir);

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("[mcp_servers.unknown]"));
        assert!(next.contains("[mcp_servers.managed]"));
        assert!(next.contains("[mcp_servers.fresh]"));
        assert!(!next.contains("command = \"old\""));
    }

    #[test]
    fn resolves_relative_paths_inside_app_data_dir() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().expect("tmpdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(&workspace).expect("create workspace");

        let (previous_home, previous_dir) = set_test_runtime(&home, &workspace);

        let resolved = resolve_relative_path("config/servers.yaml").expect("relative path");

        restore_test_runtime(previous_home, previous_dir);

        assert_eq!(
            resolved,
            expected_app_data_dir(&home).join("config/servers.yaml")
        );
    }

    #[test]
    fn rejects_internal_paths_that_escape_app_data() {
        for bad in ["../servers.yaml", "/etc/passwd", "config/../../.ssh/id_rsa"] {
            assert!(resolve_relative_path(bad).is_err(), "{bad}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn preserves_existing_modes_and_creates_private_files() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tmpdir");
        let mode_of = |path: &Path| fs::metadata(path).expect("meta").permissions().mode() & 0o777;
        let set_mode = |path: &Path, mode: u32| {
            let mut permissions = fs::metadata(path).expect("meta").permissions();
            permissions.set_mode(mode);
            fs::set_permissions(path, permissions).expect("chmod");
        };

        let private = dir.path().join("private.toml");
        fs::write(&private, "before").expect("seed");
        set_mode(&private, 0o600);
        super::atomic_write(&private, "after").expect("rewrite");
        assert_eq!(mode_of(&private), 0o600);
        assert_eq!(fs::read_to_string(&private).expect("read"), "after");

        let group = dir.path().join("group.toml");
        fs::write(&group, "before").expect("seed");
        set_mode(&group, 0o640);
        super::atomic_write(&group, "after").expect("rewrite");
        assert_eq!(mode_of(&group), 0o640, "0640 must not widen to 0644");

        let created = dir.path().join("history.json");
        super::atomic_write(&created, "{\"token\":\"x\"}").expect("create");
        assert_eq!(
            mode_of(&created) & 0o077,
            0,
            "new secret file is world-readable"
        );
    }

    #[test]
    fn a_failed_replacement_does_not_leave_a_secret_temp_file() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dest = dir.path().join("servers.yaml");
        fs::create_dir(&dest).expect("dir");
        let error = super::atomic_write(&dest, "super-secret-token").expect_err("rename onto dir");
        assert!(!error.is_empty());
        let names = fs::read_dir(dir.path())
            .expect("list")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(names, ["servers.yaml"]);
        assert!(!dir.path().join("servers.yaml").is_file());
    }

    #[test]
    fn writes_backups_inside_app_data_dir() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().expect("tmpdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(&workspace).expect("create workspace");

        let target = workspace.join("client.json");
        fs::write(&target, "{}").expect("seed file");

        let (previous_home, previous_dir) = set_test_runtime(&home, &workspace);

        let backup = backup_file(&target)
            .expect("backup result")
            .expect("backup path should exist");

        restore_test_runtime(previous_home, previous_dir);

        let expected_root = expected_app_data_dir(&home).join("backups");
        assert!(PathBuf::from(backup).starts_with(&expected_root));
    }

    #[test]
    fn rollback_restores_the_original_file() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().expect("tmpdir");
        let home = temp.path().join("home");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(&workspace).expect("create workspace");

        let target = workspace.join(".cursor").join("mcp.json");
        fs::create_dir_all(target.parent().expect("parent")).expect("create config dir");
        fs::write(&target, "original").expect("seed file");

        let (previous_home, previous_dir) = set_test_runtime(&home, &workspace);

        let backup = backup_file(&target)
            .expect("backup result")
            .expect("backup path should exist");
        fs::write(&target, "changed").expect("change file");
        let restored = rollback(vec![backup]);

        restore_test_runtime(previous_home, previous_dir);

        restored.expect("rollback");
        assert_eq!(fs::read_to_string(&target).expect("read"), "original");
        assert!(!target.with_file_name("mcp").exists());
    }

    #[test]
    fn malformed_json_and_toml_are_left_unchanged() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let json_path = dir.path().join("client.json");
        let toml_path = dir.path().join("config.toml");
        let json_body = "{ this is not json, \"theme\": \"dark\" ";
        let toml_body = "model = \"gpt\"\n[mcp_servers\n";
        fs::write(&json_path, json_body).expect("json");
        fs::write(&toml_path, toml_body).expect("toml");

        let json_error = super::apply_operation(
            &json_path,
            &WriteOperation {
                path: json_path.to_string_lossy().to_string(),
                mode: "merge_json_object_entries".to_string(),
                field: Some("mcpServers".to_string()),
                remove_keys: None,
                content: r#"{"demo":{"command":"npx"}}"#.to_string(),
            },
        )
        .expect_err("json");
        let toml_error = super::apply_operation(
            &toml_path,
            &WriteOperation {
                path: toml_path.to_string_lossy().to_string(),
                mode: "merge_toml_table_entries".to_string(),
                field: Some("mcp_servers".to_string()),
                remove_keys: None,
                content: r#"{"demo":{"command":"npx"}}"#.to_string(),
            },
        )
        .expect_err("toml");

        assert!(json_error.contains("not valid JSON"), "{json_error}");
        assert!(
            toml_error.contains("TOML") || toml_error.contains("toml"),
            "{toml_error}"
        );
        assert_eq!(fs::read_to_string(&json_path).expect("json"), json_body);
        assert_eq!(fs::read_to_string(&toml_path).expect("toml"), toml_body);
    }

    #[test]
    fn backup_deletion_rejects_traversal_and_does_not_follow_symlinks() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let app_data = dir.path().join("app");
        let backups = app_data.join("backups");
        fs::create_dir_all(&backups).expect("backups");
        let outside = dir.path().join("outside-secret.txt");
        fs::write(&outside, "keep-me").expect("outside");

        let traversal = app_data
            .join("backups")
            .join("..")
            .join("outside-secret.txt");
        let error = super::delete_owned_backup(&app_data, &traversal).expect_err("traversal");
        assert!(
            error.contains("traversal") || error.contains("outside"),
            "{error}"
        );
        assert_eq!(fs::read_to_string(&outside).expect("kept"), "keep-me");

        let absolute = dir.path().join("not-a-backup.bak");
        fs::write(&absolute, "nope").expect("absolute");
        assert!(super::delete_owned_backup(&app_data, &absolute).is_err());
        assert!(absolute.exists());

        #[cfg(unix)]
        {
            let link = backups.join("linked.bak");
            std::os::unix::fs::symlink(&outside, &link).expect("symlink");
            super::delete_owned_backup(&app_data, &link).expect("unlink");
            assert!(!link.exists(), "the symlink itself is removed");
            assert_eq!(fs::read_to_string(&outside).expect("target"), "keep-me");
            std::os::unix::fs::symlink(&outside, &link).expect("symlink again");
            let error = super::read_backup_bytes(&app_data, &link).expect_err("read link");
            assert!(error.contains("symlink"), "{error}");
            assert_eq!(fs::read_to_string(&outside).expect("target"), "keep-me");
        }
    }
}
