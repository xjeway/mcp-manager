use crate::core::WriteOperation;
use crate::platform::PlatformContext;
use chrono::Utc;
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use toml::Table;

fn base_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn resolve_path(path: &str) -> PathBuf {
    PlatformContext::current().resolve_path(path)
}

pub fn resolve_relative_path(relative_path: &str) -> PathBuf {
    resolve_path(relative_path)
}

pub fn ensure_parent(path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn backup_file(target: &PathBuf) -> Result<Option<String>, String> {
    if !target.exists() {
        return Ok(None);
    }

    let backup_dir = base_dir().join("backups");
    let relative_target = target
        .strip_prefix("/")
        .map(PathBuf::from)
        .unwrap_or_else(|_| target.clone());
    let backup_parent = backup_dir.join(
        relative_target
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default(),
    );
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

pub fn apply_operations(artifacts: Vec<WriteOperation>) -> Result<Vec<String>, String> {
    let mut backups = Vec::new();

    for item in artifacts {
        let path = resolve_path(&item.path);
        if let Some(backup) = backup_file(&path)? {
            backups.push(backup);
        }
        match item.mode.as_str() {
            "replace_json" => apply_replace_json(&path, &item.content)?,
            "merge_json_field" => apply_merge_json_field(
                &path,
                item.field
                    .as_deref()
                    .ok_or_else(|| "missing JSON merge field".to_string())?,
                &item.content,
            )?,
            "merge_toml_field" => apply_merge_toml_field(
                &path,
                item.field
                    .as_deref()
                    .ok_or_else(|| "missing TOML merge field".to_string())?,
                &item.content,
            )?,
            other => return Err(format!("unsupported artifact mode: {other}")),
        }
    }

    Ok(backups)
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
            .split('.')
            .next()
            .ok_or_else(|| "invalid backup name format".to_string())?;

        let target = PathBuf::from("/")
            .join(
                relative
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("")),
            )
            .join(original_name);
        ensure_parent(&target)?;
        fs::copy(src, target).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_operations;
    use crate::core::WriteOperation;
    use std::fs;

    #[test]
    fn preserves_unrelated_json_fields_when_merging() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("claude.json");
        fs::write(
            &path,
            r#"{"theme":"dark","mcpServers":{"old":{"command":"old"}}}"#,
        )
        .expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_json_field".to_string(),
            field: Some("mcpServers".to_string()),
            content: r#"{"new":{"command":"npx"}}"#.to_string(),
        }])
        .expect("apply");

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("\"theme\": \"dark\""));
        assert!(next.contains("\"new\""));
    }

    #[test]
    fn preserves_unrelated_toml_fields_when_merging() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("config.toml");
        fs::write(&path, "model = \"gpt-5.4\"\n").expect("seed");

        apply_operations(vec![WriteOperation {
            path: path.to_string_lossy().to_string(),
            mode: "merge_toml_field".to_string(),
            field: Some("mcp_servers".to_string()),
            content: r#"{"playwright":{"command":"npx","args":["@playwright/mcp@latest"]}}"#
                .to_string(),
        }])
        .expect("apply");

        let next = fs::read_to_string(&path).expect("read");
        assert!(next.contains("model = \"gpt-5.4\""));
        assert!(next.contains("[mcp_servers.playwright]"));
    }
}
