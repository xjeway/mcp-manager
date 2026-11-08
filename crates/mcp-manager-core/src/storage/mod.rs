use crate::core::WriteOperation;
use crate::platform::PlatformContext;
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use toml::Table;

mod jsonc;
mod toml_doc;
mod yaml_doc;

fn base_dir() -> PathBuf {
    PlatformContext::current().app_data_dir()
}

/// Resolve a stored path using the current runtime home and application data directory.
pub fn resolve_path(path: &str) -> PathBuf {
    PlatformContext::current().resolve_path(path)
}

/// Canonicalize existing prefixes so aliases share one batch ownership key.
pub(crate) fn target_identity(path: &Path) -> PathBuf {
    if let Ok(path) = path.canonicalize() {
        return path;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) if parent != path && !parent.as_os_str().is_empty() => {
            target_identity(parent).join(name)
        }
        _ => path.to_path_buf(),
    }
}

#[derive(Debug)]
pub struct BatchCommit {
    pub backups: Vec<String>,
    pub files: std::collections::BTreeMap<PathBuf, Vec<u8>>,
}

pub fn resolve_relative_path(relative_path: &str) -> PathBuf {
    let candidate = PathBuf::from(relative_path);
    if candidate.is_absolute() {
        return candidate;
    }

    base_dir().join(candidate)
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
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(
        ".{TEMP_MARKER}{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let tmp = PathBuf::from(tmp);
    remove_stale_temp_files(path);
    fs::write(&tmp, content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        e.to_string()
    })
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
    aliases: std::collections::BTreeMap<PathBuf, PathBuf>,
    presence: std::collections::BTreeMap<PathBuf, bool>,
    files: Vec<(PathBuf, Option<Vec<u8>>)>,
    /// Directories that did not exist, parents before children.
    new_dirs: Vec<PathBuf>,
}

impl Snapshot {
    /// Records `path` unless it is already recorded, so the first state wins.
    pub fn capture(&mut self, path: &Path) -> Result<(), String> {
        let identity = target_identity(path);
        self.aliases.insert(path.to_path_buf(), identity.clone());
        let path = identity.as_path();
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

    /// Watch path selection without reading inactive client files.
    pub(crate) fn capture_presence(&mut self, path: &Path) {
        self.presence.insert(path.to_path_buf(), path.exists());
    }

    /// Recheck original dependency aliases and precedence markers, not just canonical contents.
    pub(crate) fn verify_bindings(&self) -> Result<(), String> {
        self.verify_bindings_except(|_| false)
    }

    /// Presence changes to approved write targets are checked against batch-owned bytes instead.
    fn verify_bindings_except(
        &self,
        ignore_presence: impl Fn(&Path) -> bool,
    ) -> Result<(), String> {
        for (alias, identity) in &self.aliases {
            if target_identity(alias) != *identity {
                return Err(format!(
                    "Client configuration alias changed before writing {}",
                    alias.display()
                ));
            }
        }
        for (path, existed) in &self.presence {
            if !ignore_presence(path) && path.exists() != *existed {
                return Err(format!(
                    "Client configuration active path changed before writing {}",
                    path.display()
                ));
            }
        }
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

    /// Checks a planned target against its captured bytes just before a batch write.
    pub fn verify_path(&self, path: &Path) -> Result<(), String> {
        let identity = target_identity(path);
        let path = identity.as_path();
        let expected = self
            .files
            .iter()
            .find(|(target, _)| target == path)
            .ok_or_else(|| format!("unapproved target: {}", path.display()))?;
        let current = match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.to_string()),
        };
        if current != expected.1 {
            return Err(format!("Client configuration changed before writing {}. Run the command again to review it.", path.display()));
        }
        Ok(())
    }

    /// Compare ownership immediately before restoring each individual file.
    /// As with forward writes, uncooperative writers may still race the final atomic replacement.
    pub fn restore_owned(
        &self,
        written: &std::collections::BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<Vec<PathBuf>, String> {
        self.restore_owned_with(written, |_| {})
    }

    /// Check ownership separately at each restoration boundary, allowing deterministic race injection.
    fn restore_owned_with(
        &self,
        written: &std::collections::BTreeMap<PathBuf, Vec<u8>>,
        mut before: impl FnMut(&Path),
    ) -> Result<Vec<PathBuf>, String> {
        let mut failures = Vec::new();
        let mut skipped = Vec::new();
        for (path, original) in self.files.iter().rev() {
            before(path);
            let Some(bytes) = written.get(path) else {
                continue;
            };
            if !fs::read(path).is_ok_and(|current| current == *bytes) {
                skipped.push(path.clone());
                continue;
            }
            let result = match original {
                Some(content) => atomic_write_bytes(path, content),
                None => fs::remove_file(path).map_err(|e| e.to_string()),
            };
            if let Err(error) = result {
                failures.push(format!("{}: {error}", path.display()));
            }
        }
        remove_empty_dirs(&self.new_dirs);
        if failures.is_empty() {
            Ok(skipped)
        } else {
            Err(failures.join("; "))
        }
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

/// Convert modeled JSON values into the corresponding TOML representation.
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

/// Return the exact committed payload, without rereading a possibly externally edited target.
fn commit_text(path: &Path, content: &str) -> Result<Vec<u8>, String> {
    commit_text_with(path, content, || {})
}

/// Explicit post-commit seam for deterministic concurrency tests.
fn commit_text_with(
    path: &Path,
    content: &str,
    after_commit: impl FnOnce(),
) -> Result<Vec<u8>, String> {
    atomic_write(path, content)?;
    after_commit();
    Ok(content.as_bytes().to_vec())
}

/// Validate and render replacement JSON, returning the exact bytes committed.
fn apply_replace_json(path: &Path, content: &str) -> Result<Vec<u8>, String> {
    let parsed: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let pretty = serde_json::to_string_pretty(&parsed).map_err(|e| e.to_string())?;
    commit_text(path, &pretty)
}

fn read_existing(path: &Path) -> Result<String, String> {
    if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

/// A client file that cannot be parsed is reported instead of replaced: it is
/// usually hand-edited, and writing over it would lose everything in it.
fn unreadable(path: &Path, error: String) -> String {
    format!(
        "{} could not be read as JSON: {error}",
        path.to_string_lossy()
    )
}

/// Replace one JSON field while preserving unrelated configuration and returning committed bytes.
fn apply_merge_json_field(path: &Path, field: &str, content: &str) -> Result<Vec<u8>, String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let existing = read_existing(path)?;
    let updated = jsonc::set_root_member(&existing, field, &field_value)
        .map_err(|error| unreadable(path, error))?;
    commit_text(path, &updated)
}

/// Fields MCP Manager models and rewrites on every apply. Anything else on an
/// existing entry (`oauth`, `timeout`, `autoApprove`, `disabledTools`, ...) was
/// set by the user or the client and is carried over to the rewritten entry.
const TRANSPORT_FIELDS: &[&str] = &[
    "command",
    "args",
    "env",
    "environment",
    "url",
    "serverUrl",
    "httpUrl",
    "type",
    "headers",
    "http_headers",
    "tools",
    // Older Zed marker for a custom server, next to a nested `command`.
    "source",
    // Mistral Vibe and Hermes name the transport; Goose spells the command,
    // URL and environment `cmd`, `uri` and `envs`.
    "transport",
    "cmd",
    "uri",
    "envs",
];

fn is_transport_field(field: &str) -> bool {
    TRANSPORT_FIELDS.contains(&field)
}

/// Whether an entry runs a local command, as opposed to a remote URL. Fields are
/// only carried over within the same kind: a stdio-only `cwd` on a remote
/// entry, or a remote-only `bearer_token_env_var` on a stdio one, is rejected
/// by some clients (Codex fails to load its config).
fn is_local_entry<'a>(mut keys: impl Iterator<Item = &'a String>) -> bool {
    keys.any(|key| key == "command" || key == "cmd")
}

/// Copies the fields of `old` that an apply keeps into `new`.
pub(crate) fn carry_unmanaged_fields(old: &Value, new: &mut Value) {
    let (Some(old), Some(new)) = (old.as_object(), new.as_object_mut()) else {
        return;
    };
    if is_local_entry(old.keys()) != is_local_entry(new.keys()) {
        return;
    }
    // Mistral Vibe keeps API-key settings next to static headers, in `auth`
    // (or, in older files, at the top of the entry; it rejects both at once).
    // When the new entry brings its own static `auth`, those settings move
    // into it instead of being lost.
    const API_KEY_FIELDS: [&str; 3] = ["api_key_env", "api_key_header", "api_key_format"];
    if let Some(Value::Object(auth)) = new.get_mut("auth") {
        if auth.get("type").and_then(Value::as_str) == Some("static") {
            let old_auth = old
                .get("auth")
                .and_then(Value::as_object)
                .filter(|auth| auth.get("type").and_then(Value::as_str) == Some("static"));
            for field in API_KEY_FIELDS {
                let kept = old_auth
                    .and_then(|auth| auth.get(field))
                    .or_else(|| old.get(field));
                if let (Some(value), false) = (kept, auth.contains_key(field)) {
                    auth.insert(field.to_string(), value.clone());
                }
            }
        }
    }
    // Whether the client turned the entry off is the client's setting: an
    // apply writes `enabled: true` only for a new entry.
    if let (Some(enabled), Some(slot)) = (old.get("enabled"), new.get_mut("enabled")) {
        *slot = enabled.clone();
    }
    let has_auth = new.contains_key("auth");
    for (field, value) in old {
        let moved_into_auth = has_auth && API_KEY_FIELDS.contains(&field.as_str());
        if !is_transport_field(field) && !moved_into_auth && !new.contains_key(field) {
            new.insert(field.clone(), value.clone());
        }
    }
}

/// Carries JSON client fields, including CodeBuddy's unmodeled per-tool map.
/// TOML/YAML clients keep their existing transport-field rules.
pub(crate) fn carry_unmanaged_json_fields(old: &Value, new: &mut Value) {
    // ZCode's client-owned switch is independent of the transport kind.
    if let (Some(enable), Some(new)) = (
        old.get("enable").filter(|value| value.is_boolean()),
        new.as_object_mut(),
    ) {
        new.entry("enable".to_string())
            .or_insert_with(|| enable.clone());
    }
    carry_unmanaged_fields(old, new);
    let (Some(old), Some(new)) = (old.as_object(), new.as_object_mut()) else {
        return;
    };
    if is_local_entry(old.keys()) == is_local_entry(new.keys()) && !new.contains_key("tools") {
        if let Some(tools) = old.get("tools").filter(|value| value.is_object()) {
            new.insert("tools".into(), tools.clone());
        }
    }
}

/// Merge managed JSON entries, optionally honoring MiMo native enable transitions.
fn apply_merge_json_object_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
    native_enabled: bool,
) -> Result<Vec<u8>, String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let Some(new_entries) = field_value.as_object() else {
        return Err(format!("{field} merge content must be a JSON object"));
    };
    let existing = read_existing(path)?;
    let updated = jsonc::merge_object_entries(
        &existing,
        field,
        new_entries,
        remove_keys.unwrap_or_default(),
        |old, new| {
            let enabled = native_enabled
                .then(|| new.get("enabled").cloned())
                .flatten();
            carry_unmanaged_json_fields(old, new);
            if let Some(enabled) = enabled {
                new["enabled"] = enabled;
            }
        },
    )
    .map_err(|error| unreadable(path, error))?;
    commit_text(path, &updated)
}

/// Uses a JSON Pointer to distinguish nested paths from literal dotted client keys.
fn apply_merge_json_object_path_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    let entries = merge_content(field, content)?;
    let pointer = field
        .strip_prefix('/')
        .ok_or_else(|| "nested JSON field must be a JSON Pointer".to_string())?;
    let keys = pointer
        .split('/')
        .map(|key| key.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>();
    if keys.iter().any(String::is_empty) {
        return Err("nested JSON field contains an empty segment".into());
    }
    let existing = read_existing(path)?;
    let updated = jsonc::merge_object_entries_at_path(
        &existing,
        &keys,
        &entries,
        remove_keys.unwrap_or_default(),
        carry_unmanaged_json_fields,
    )
    .map_err(|error| unreadable(path, error))?;
    commit_text(path, &updated)
}

/// Initializes Muse's required schema only for a new file, then commits one atomic merge.
fn apply_merge_muse_entries(
    path: &Path,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    let existing = if path.exists() {
        let existing = fs::read_to_string(path).map_err(|error| error.to_string())?;
        crate::adapters::validate_muse_document(&existing)
            .map_err(|error| unreadable(path, error))?;
        existing
    } else {
        "{\n  \"schema_version\": 1\n}\n".into()
    };
    let entries: Value = serde_json::from_str(content).map_err(|error| error.to_string())?;
    let entries = entries
        .as_object()
        .ok_or("Muse MCP entries must be an object")?;
    let updated = jsonc::merge_object_entries(
        &existing,
        "mcp_servers",
        entries,
        remove_keys.unwrap_or_default(),
        carry_unmanaged_json_fields,
    )
    .map_err(|error| unreadable(path, error))?;
    commit_text(path, &updated)
}

/// Replace one TOML field while preserving unrelated document content.
fn apply_merge_toml_field(path: &Path, field: &str, content: &str) -> Result<Vec<u8>, String> {
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
    commit_text(path, &rendered)
}

/// Update managed TOML tables while carrying forward client-owned fields.
fn apply_merge_toml_table_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    let new_entries = merge_content(field, content)?;
    let existing = read_existing(path)?;
    let updated = toml_doc::merge_table_entries(
        &existing,
        field,
        &new_entries,
        remove_keys.unwrap_or_default(),
        carry_unmanaged_fields,
    )
    .map_err(|error| unreadable_toml(path, error))?;
    commit_text(path, &updated)
}

/// Update managed YAML entries and return their committed document payload.
fn apply_merge_yaml_mapping_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    let new_entries = merge_content(field, content)?;
    let existing = read_existing(path)?;
    let updated = yaml_doc::merge_mapping_entries(
        &existing,
        field,
        &new_entries,
        remove_keys.unwrap_or_default(),
        carry_unmanaged_fields,
    )
    .map_err(|error| {
        format!(
            "{} could not be edited as YAML: {error}",
            path.to_string_lossy()
        )
    })?;
    commit_text(path, &updated)
}

/// Key that identifies an entry of a `merge_toml_array_entries` list.
const TOML_ARRAY_ENTRY_KEY: &str = "name";

/// Update managed array entries without deleting unrelated TOML settings.
fn apply_merge_toml_array_entries(
    path: &Path,
    field: &str,
    content: &str,
    remove_keys: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    let new_entries = merge_content(field, content)?;
    let existing = read_existing(path)?;
    let updated = toml_doc::merge_array_entries(
        &existing,
        field,
        TOML_ARRAY_ENTRY_KEY,
        &new_entries,
        remove_keys.unwrap_or_default(),
        carry_unmanaged_fields,
    )
    .map_err(|error| unreadable_toml(path, error))?;
    commit_text(path, &updated)
}

fn merge_content(field: &str, content: &str) -> Result<serde_json::Map<String, Value>, String> {
    match serde_json::from_str(content).map_err(|e| e.to_string())? {
        Value::Object(entries) => Ok(entries),
        _ => Err(format!("{field} merge content must be a JSON object")),
    }
}

fn unreadable_toml(path: &Path, error: String) -> String {
    format!(
        "{} could not be read as TOML: {error}",
        path.to_string_lossy()
    )
}

/// Applies one write to `path` without taking a backup.
pub fn apply_operation(path: &Path, item: &WriteOperation) -> Result<(), String> {
    apply_operation_committed(path, item).map(|_| ())
}

/// Render and commit one operation, retaining ownership of the exact written bytes.
fn apply_operation_committed(path: &Path, item: &WriteOperation) -> Result<Vec<u8>, String> {
    let field = |kind: &str| {
        item.field
            .as_deref()
            .ok_or_else(|| format!("missing {kind} merge field"))
    };
    match item.mode.as_str() {
        "replace_json" => apply_replace_json(path, &item.content),
        "merge_muse_mcp_entries" => {
            apply_merge_muse_entries(path, &item.content, item.remove_keys.as_deref())
        }
        "merge_json_field" => apply_merge_json_field(path, field("JSON")?, &item.content),
        "merge_json_object_path_entries" => apply_merge_json_object_path_entries(
            path,
            field("JSON path")?,
            &item.content,
            item.remove_keys.as_deref(),
        ),
        "merge_json_object_entries" | "merge_mimo_entries" => apply_merge_json_object_entries(
            path,
            field("JSON")?,
            &item.content,
            item.remove_keys.as_deref(),
            item.mode == "merge_mimo_entries",
        ),
        "merge_toml_field" => apply_merge_toml_field(path, field("TOML")?, &item.content),
        "merge_toml_table_entries" => apply_merge_toml_table_entries(
            path,
            field("TOML")?,
            &item.content,
            item.remove_keys.as_deref(),
        ),
        "merge_toml_array_entries" => apply_merge_toml_array_entries(
            path,
            field("TOML")?,
            &item.content,
            item.remove_keys.as_deref(),
        ),
        "merge_yaml_mapping_entries" => apply_merge_yaml_mapping_entries(
            path,
            field("YAML")?,
            &item.content,
            item.remove_keys.as_deref(),
        ),
        other => Err(format!("unsupported artifact mode: {other}")),
    }
}

/// Apply operations with backups and compensating restoration on failure.
/// Restore only revisions still owned by the batch; report and preserve observed
/// external edits. Checks are optimistic against uncooperative external writers.
pub fn apply_operations(artifacts: Vec<WriteOperation>) -> Result<Vec<String>, String> {
    apply_operations_guarded(artifacts, None)
}

/// Rechecks the approved bytes before each write. This is optimistic concurrency;
/// external clients do not participate in our lock and can still race after a check.
pub fn apply_operations_checked(
    artifacts: Vec<WriteOperation>,
    expected: &Snapshot,
) -> Result<Vec<String>, String> {
    apply_operations_guarded(artifacts, Some(expected))
}

/// Read-only planning inputs are checked before every write and never enter rollback state.
pub fn apply_operations_checked_with_inputs(
    artifacts: Vec<WriteOperation>,
    expected: &Snapshot,
    inputs: &Snapshot,
    mut check_context: impl FnMut() -> Result<(), String>,
) -> Result<BatchCommit, String> {
    apply_operations_guarded_with(artifacts, Some(expected), |_, _| {
        check_context()?;
        inputs.verify_bindings_except(|path| {
            expected
                .paths()
                .any(|target| target == target_identity(path))
        })?;
        for path in inputs
            .paths()
            .filter(|path| !expected.paths().any(|target| target == *path))
        {
            inputs.verify_path(path)?;
        }
        Ok(())
    })
}

/// Shared batch rollback implementation with an optional per-target precondition.
fn apply_operations_guarded(
    artifacts: Vec<WriteOperation>,
    expected: Option<&Snapshot>,
) -> Result<Vec<String>, String> {
    apply_operations_guarded_with(artifacts, expected, |_, _| Ok(())).map(|commit| commit.backups)
}

/// A batch owns its prior writes, while an edit between operations remains external.
fn apply_operations_guarded_with(
    artifacts: Vec<WriteOperation>,
    expected: Option<&Snapshot>,
    mut before: impl FnMut(usize, &Path) -> Result<(), String>,
) -> Result<BatchCommit, String> {
    let mut backups = Vec::new();
    let mut originals = Snapshot::default();
    let mut written = std::collections::BTreeMap::<PathBuf, Vec<u8>>::new();
    for (index, item) in artifacts.into_iter().enumerate() {
        let path = target_identity(&resolve_path(&item.path));
        let result = (|| {
            before(index, &path)?;
            if let Some(expected) = expected {
                expected.verify_bindings()?;
                if !expected.paths().any(|target| target == path) {
                    return Err(format!("unapproved target: {}", path.display()));
                }
                // Dependencies may themselves be later write targets. Validate every
                // approved target against either its initial or this batch's version.
                for target in expected.paths() {
                    if let Some(bytes) = written.get(target) {
                        if !fs::read(target).is_ok_and(|current| current == *bytes) {
                            return Err(format!("Client configuration changed between batch writes to {}. Nothing further was written.",target.display()));
                        }
                    } else {
                        expected.verify_path(target)?;
                    }
                }
            }
            originals.capture(&path)?;
            if !written.contains_key(&path) {
                if let Some(backup) = backup_file(&path)? {
                    backups.push(backup);
                }
            }
            let committed = apply_operation_committed(&path, &item)?;
            written.insert(path.clone(), committed);
            Ok(())
        })();
        if let Err(error) = result {
            // Restore only versions still owned by this batch, even if failure was elsewhere.
            return Err(match originals.restore_owned(&written) {
                Ok(skipped) if skipped.is_empty() => error,
                Ok(skipped) => format!(
                    "{error}; external edits were preserved during rollback: {}",
                    skipped
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Err(restore_error) => format!(
                    "{error}; restoring the files written before it also failed: {restore_error}"
                ),
            });
        }
    }
    Ok(BatchCommit {
        backups,
        files: written,
    })
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
pub fn restore_backup(backup: &Path) -> Result<(), String> {
    let target = backup_target(backup)?;
    let content = fs::read(backup).map_err(|e| e.to_string())?;
    atomic_write_bytes(&target, &content)
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
    use super::{apply_operation, apply_operations, backup_file, resolve_relative_path, rollback};
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
    fn preserves_tool_configuration_objects_without_preserving_managed_tool_lists() {
        use serde_json::json;
        let mut new = json!({"url":"https://new/mcp"});
        super::carry_unmanaged_json_fields(
            &json!({"url":"https://old/mcp","tools":{"search":{"defer_loading":true}}}),
            &mut new,
        );
        assert_eq!(new["tools"], json!({"search":{"defer_loading":true}}));
        let mut new = json!({"url":"https://new/mcp"});
        super::carry_unmanaged_json_fields(
            &json!({"url":"https://old/mcp","tools":["old"]}),
            &mut new,
        );
        assert!(new.get("tools").is_none());
        let mut new = json!({"url":"https://new/mcp","tools":["*"]});
        super::carry_unmanaged_json_fields(
            &json!({"url":"https://old/mcp","tools":{"search":{}}}),
            &mut new,
        );
        assert_eq!(new["tools"], json!(["*"]));
    }

    #[test]
    fn json_tool_maps_do_not_change_codex_toml_rewrites() {
        use serde_json::json;
        let old = json!({"url":"https://old/mcp", "tools":{"search":"old"}});
        let mut written = json!({"url":"https://new/mcp"});
        super::carry_unmanaged_fields(&old, &mut written);
        assert!(written.get("tools").is_none());
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(
            &path,
            "[mcp_servers.api]\nurl = 'https://old/mcp'\ntools = { search = 'old' }\n",
        )
        .unwrap();
        super::apply_merge_toml_table_entries(
            &path,
            "mcp_servers",
            &json!({"api":{"url":"https://new/mcp"}}).to_string(),
            Some(&["api".into()]),
        )
        .unwrap();
        let parsed: toml::Value = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert!(parsed["mcp_servers"]["api"].get("tools").is_none());
    }

    #[test]
    fn rewriting_a_json_entry_keeps_fields_it_does_not_model() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("mcp.json");
        fs::write(
            &path,
            r#"{"mcpServers":{"linear":{"type":"http","url":"https://old/mcp","headers":{"A":"1"},"timeout":9000,"oauth":{"clientId":"abc"}},"gone":{"command":"x"}}}"#,
        )
        .expect("seed");
        let operation = WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_json_object_entries".to_string(),
            field: Some("mcpServers".to_string()),
            remove_keys: Some(vec!["linear".to_string(), "gone".to_string()]),
            content: r#"{"linear":{"type":"http","url":"https://new/mcp"}}"#.to_string(),
        };
        apply_operation(&path, &operation).expect("apply");

        let written: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let linear = &written["mcpServers"]["linear"];
        assert_eq!(linear["url"], "https://new/mcp");
        assert_eq!(linear["timeout"], 9000);
        assert_eq!(linear["oauth"]["clientId"], "abc");
        // Transport fields the manager owns are not resurrected.
        assert!(linear.get("headers").is_none());
        assert!(written["mcpServers"].get("gone").is_none());
    }

    #[test]
    fn switching_between_local_and_remote_drops_the_old_entry_fields() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            "[mcp_servers.docs]\nurl = \"https://old/mcp\"\nbearer_token_env_var = \"TOKEN\"\n",
        )
        .expect("seed");
        let operation = WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_toml_table_entries".to_string(),
            field: Some("mcp_servers".to_string()),
            remove_keys: Some(vec!["docs".to_string()]),
            content: r#"{"docs":{"command":"npx","args":["docs-mcp"]}}"#.to_string(),
        };
        apply_operation(&path, &operation).expect("apply");

        let written: toml::Table =
            toml::from_str(&fs::read_to_string(&path).expect("read")).expect("toml");
        let docs = &written["mcp_servers"]["docs"];
        assert_eq!(docs["command"].as_str(), Some("npx"));
        assert!(docs.get("bearer_token_env_var").is_none());
    }

    #[test]
    fn rewriting_a_toml_entry_keeps_fields_it_does_not_model() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            "[mcp_servers.docs]\nurl = \"https://old/mcp\"\nstartup_timeout_sec = 30\nenabled_tools = [\"search\"]\n",
        )
        .expect("seed");
        let operation = WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_toml_table_entries".to_string(),
            field: Some("mcp_servers".to_string()),
            remove_keys: Some(vec!["docs".to_string()]),
            content: r#"{"docs":{"url":"https://new/mcp"}}"#.to_string(),
        };
        apply_operation(&path, &operation).expect("apply");

        let written: toml::Table =
            toml::from_str(&fs::read_to_string(&path).expect("read")).expect("toml");
        let docs = &written["mcp_servers"]["docs"];
        assert_eq!(docs["url"].as_str(), Some("https://new/mcp"));
        assert_eq!(docs["startup_timeout_sec"].as_integer(), Some(30));
        assert!(docs.get("enabled_tools").is_some());
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

        let next: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        assert_eq!(next["theme"], "dark");
        assert_eq!(next["mcpServers"]["new"]["command"], "npx");
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

        let next: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        assert_eq!(next["theme"], "dark");
        assert_eq!(next["mcpServers"]["unknown"]["command"], "keep");
        assert_eq!(next["mcpServers"]["managed"]["command"], "new");
        assert_eq!(next["mcpServers"]["fresh"]["command"], "npx");
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

        let resolved = resolve_relative_path("config/servers.yaml");

        restore_test_runtime(previous_home, previous_dir);

        assert_eq!(
            resolved,
            expected_app_data_dir(&home).join("config/servers.yaml")
        );
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
    fn checked_batches_accept_their_own_prior_writes_but_preserve_external_edits() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let runtime = set_test_runtime(&home, temp.path());
        let a = temp.path().join("a.json");
        let b = temp.path().join("b.json");
        fs::write(&a, "{}").unwrap();
        fs::write(&b, "{}").unwrap();
        let operation = |path: &Path, field: &str| WriteOperation {
            path: path.to_string_lossy().into(),
            mode: "merge_json_field".into(),
            field: Some(field.into()),
            content: "1".into(),
            remove_keys: None,
        };
        let mut expected = super::Snapshot::default();
        expected.capture(&a).unwrap();
        expected.capture(&b).unwrap();
        let result = super::apply_operations_checked(
            vec![operation(&a, "first"), operation(&a, "second")],
            &expected,
        );
        let accepted = fs::read_to_string(&a).unwrap();
        fs::write(&a, "{}").unwrap();
        let failed = super::apply_operations_guarded_with(
            vec![
                operation(&b, "first"),
                operation(&a, "first"),
                operation(&a, "second"),
            ],
            Some(&expected),
            |index, path| {
                if index == 2 {
                    fs::write(path, r#"{"external":true}"#).unwrap();
                }
                Ok(())
            },
        );
        let external = fs::read_to_string(&a).unwrap();
        let restored = fs::read_to_string(&b).unwrap();
        restore_test_runtime(runtime.0, runtime.1);
        assert!(result.is_ok(), "{result:?}");
        let value: serde_json::Value = serde_json::from_str(&accepted).unwrap();
        assert_eq!(value["first"], 1);
        assert_eq!(value["second"], 1);
        assert!(failed.is_err());
        assert_eq!(external, r#"{"external":true}"#);
        assert_eq!(restored, "{}");
    }
    #[test]
    fn rollback_checks_all_written_targets_even_when_a_dependency_check_fails_elsewhere() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let runtime = set_test_runtime(&home, temp.path());
        let a = temp.path().join("a.json");
        let b = temp.path().join("b.json");
        fs::write(&a, "{}").unwrap();
        fs::write(&b, "{}").unwrap();
        let operation = |path: &Path| WriteOperation {
            path: path.to_string_lossy().into(),
            mode: "merge_json_field".into(),
            field: Some("owned".into()),
            content: "true".into(),
            remove_keys: None,
        };
        let mut expected = super::Snapshot::default();
        expected.capture(&a).unwrap();
        expected.capture(&b).unwrap();
        let result = super::apply_operations_guarded_with(
            vec![operation(&a), operation(&b), operation(&b)],
            Some(&expected),
            |index, _| {
                if index == 2 {
                    fs::write(&a, r#"{"external":true}"#).unwrap();
                    return Err("read dependency changed".into());
                }
                Ok(())
            },
        );
        let external = fs::read_to_string(&a).unwrap();
        let restored = fs::read_to_string(&b).unwrap();
        restore_test_runtime(runtime.0, runtime.1);
        assert!(result.is_err());
        assert_eq!(external, r#"{"external":true}"#);
        assert_eq!(restored, "{}");
    }

    #[test]
    fn rollback_checks_ownership_at_each_restore_instead_of_a_batch_filter() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        fs::write(&a, "original-a").unwrap();
        fs::write(&b, "original-b").unwrap();
        let mut snapshot = super::Snapshot::default();
        snapshot.capture(&a).unwrap();
        snapshot.capture(&b).unwrap();
        fs::write(&a, "owned").unwrap();
        fs::write(&b, "owned").unwrap();
        let written = snapshot
            .paths()
            .map(|p| (p.to_path_buf(), b"owned".to_vec()))
            .collect();
        snapshot
            .restore_owned_with(&written, |path| {
                if path.ends_with("a") {
                    fs::write(path, "external").unwrap();
                }
            })
            .unwrap();
        assert_eq!(fs::read_to_string(a).unwrap(), "external");
        assert_eq!(fs::read_to_string(b).unwrap(), "original-b");
    }

    #[test]
    fn aliases_share_a_single_checked_target() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let runtime = set_test_runtime(&home, temp.path());
        let path = temp.path().join("client.json");
        fs::write(&path, "{}").unwrap();
        fs::create_dir(temp.path().join("nested")).unwrap();
        let alias = temp.path().join("nested/../client.json");
        let mut snapshot = super::Snapshot::default();
        snapshot.capture(&path).unwrap();
        snapshot.capture(&alias).unwrap();
        let op = |path: &Path, field: &str| WriteOperation {
            path: path.to_string_lossy().into(),
            mode: "merge_json_field".into(),
            field: Some(field.into()),
            content: "true".into(),
            remove_keys: None,
        };
        let result = super::apply_operations_checked(
            vec![op(&path, "first"), op(&alias, "second")],
            &snapshot,
        );
        restore_test_runtime(runtime.0, runtime.1);
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(snapshot.paths().count(), 1);
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(value["first"], true);
        assert_eq!(value["second"], true);
    }

    #[test]
    fn operation_returns_the_payload_it_committed_not_a_later_external_revision() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let operation = WriteOperation {
            path: path.to_string_lossy().into(),
            mode: "replace_json".into(),
            field: None,
            remove_keys: None,
            content: r#"{"owned":true}"#.into(),
        };
        let committed = super::commit_text_with(&path, &operation.content, || {
            fs::write(&path, r#"{"external":true}"#).unwrap();
        })
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&committed).unwrap();
        assert_eq!(value["owned"], true);
        assert_ne!(committed, fs::read(path).unwrap());
    }
}
