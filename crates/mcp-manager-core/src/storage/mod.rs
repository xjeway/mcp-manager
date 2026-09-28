use crate::core::WriteOperation;
use crate::platform::PlatformContext;
use chrono::Utc;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Component, Path, PathBuf, Prefix};
use toml::Table;

fn base_dir() -> PathBuf {
    PlatformContext::current().app_data_dir()
}

pub fn resolve_path(path: &str) -> PathBuf {
    PlatformContext::current().resolve_path(path)
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

pub fn backup_file(target: &PathBuf) -> Result<Option<String>, String> {
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

    let stamp = Utc::now().format("%Y%m%dT%H%M%S").to_string();
    let backup = backup_parent.join(format!("{}.{}.bak", file_name, stamp));
    fs::copy(target, &backup).map_err(|e| e.to_string())?;

    Ok(Some(backup.to_string_lossy().to_string()))
}

pub fn atomic_write(path: &PathBuf, content: &str) -> Result<(), String> {
    ensure_parent(path)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).map_err(|e| e.to_string())?;
    fs::rename(tmp, path).map_err(|e| e.to_string())?;
    Ok(())
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

fn apply_replace_json(path: &PathBuf, content: &str) -> Result<(), String> {
    let parsed: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let pretty = serde_json::to_string_pretty(&parsed).map_err(|e| e.to_string())?;
    atomic_write(path, &pretty)
}

fn apply_merge_json_field(path: &PathBuf, field: &str, content: &str) -> Result<(), String> {
    let field_value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        "{}".to_string()
    };
    let mut host =
        serde_json::from_str::<Value>(&existing).unwrap_or_else(|_| Value::Object(Map::new()));
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
    path: &PathBuf,
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
    let mut host =
        serde_json::from_str::<Value>(&existing).unwrap_or_else(|_| Value::Object(Map::new()));
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

fn apply_merge_toml_field(path: &PathBuf, field: &str, content: &str) -> Result<(), String> {
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
    path: &PathBuf,
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

fn apply_operation(path: &PathBuf, item: &WriteOperation) -> Result<(), String> {
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
    // Each file touched so far with its content before the batch; `None`
    // when the batch created it.
    let mut originals: Vec<(PathBuf, Option<Vec<u8>>)> = Vec::new();

    for item in artifacts {
        let path = resolve_path(&item.path);
        let result = (|| {
            if !originals.iter().any(|(seen, _)| seen == &path) {
                let original = if path.exists() {
                    Some(fs::read(&path).map_err(|e| e.to_string())?)
                } else {
                    None
                };
                originals.push((path.clone(), original));
            }
            if let Some(backup) = backup_file(&path)? {
                backups.push(backup);
            }
            apply_operation(&path, &item)
        })();

        if let Err(error) = result {
            return Err(match restore_originals(&originals) {
                Ok(()) => error,
                Err(restore_error) => format!(
                    "{error}; restoring the files written before it also failed: {restore_error}"
                ),
            });
        }
    }

    Ok(backups)
}

fn restore_originals(originals: &[(PathBuf, Option<Vec<u8>>)]) -> Result<(), String> {
    let failures = originals
        .iter()
        .rev()
        .filter_map(|(path, original)| {
            let restored = match original {
                Some(content) => fs::write(path, content),
                None if path.exists() => fs::remove_file(path),
                None => Ok(()),
            };
            restored
                .err()
                .map(|e| format!("{}: {e}", path.to_string_lossy()))
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

pub fn rollback(backups: Vec<String>) -> Result<(), String> {
    let backup_root = base_dir().join("backups");

    for backup in backups {
        let src = PathBuf::from(&backup);
        if !src.exists() {
            continue;
        }

        let relative = src
            .strip_prefix(&backup_root)
            .map_err(|_| "backup is outside backup root".to_string())?;

        let file_name = relative
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| "invalid backup file name".to_string())?
            .to_string();

        // filename format: <original>.<stamp>.bak
        let original_name = file_name
            .strip_suffix(".bak")
            .and_then(|name| name.rsplit_once('.'))
            .map(|(original, _stamp)| original)
            .ok_or_else(|| "invalid backup name format".to_string())?;

        let target = restore_dir(relative.parent().unwrap_or(Path::new(""))).join(original_name);
        ensure_parent(&target)?;
        fs::copy(src, target).map_err(|e| e.to_string())?;
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
}
