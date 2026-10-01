//! The unified server list (`servers.yaml`) that the app and the CLI share.

use crate::core::MCPConfig;
use crate::parser::parse_yaml_config;
use crate::platform::PlatformContext;
use crate::storage::atomic_write;
use serde::Serialize;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

pub const CONFIG_RELATIVE_PATH: &str = "config/servers.yaml";

/// Prefix of the error a checked write returns when the file changed since it
/// was read, so callers can tell a conflict from other failures.
pub const CONFLICT_ERROR: &str = "CONFIG_CONFLICT";

/// The newest servers.yaml format this build writes. A file with a higher
/// `version` came from a newer MCP Manager or mcpmgr: it can still be read, but
/// writing it could lose what that version added.
pub const CONFIG_VERSION: u64 = 1;

/// Prefix of the error a write returns for a file in a newer format.
pub const TOO_NEW_ERROR: &str = "CONFIG_TOO_NEW";

const EMPTY_CONFIG_YAML: &str = "version: 1\nservers: []\n";
pub(crate) const ABSENT_FINGERPRINT: &str = "absent";

pub fn config_path(ctx: &PlatformContext) -> PathBuf {
    ctx.app_data_dir().join(CONFIG_RELATIVE_PATH)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredText {
    pub content: String,
    /// Identifies this exact file state; pass it back to [`write_text_checked`].
    pub fingerprint: String,
}

/// FNV-1a of the file's bytes: stable across builds and platforms, unlike
/// std's hasher, and plenty to notice that another process wrote the file.
fn fingerprint_of(content: &str) -> String {
    let hash = content
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("{hash:016x}")
}

/// [`fingerprint`] of a file holding `bytes`.
pub(crate) fn fingerprint_of_bytes(bytes: &[u8]) -> Result<String, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    Ok(fingerprint_of(text))
}

pub fn fingerprint(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(ABSENT_FINGERPRINT.to_string());
    }
    Ok(fingerprint_of(
        &fs::read_to_string(path).map_err(|e| e.to_string())?,
    ))
}

/// The file's text and fingerprint, or an empty config when it does not exist yet.
pub fn read_text(path: &Path) -> Result<StoredText, String> {
    if !path.exists() {
        return Ok(StoredText {
            content: EMPTY_CONFIG_YAML.to_string(),
            fingerprint: ABSENT_FINGERPRINT.to_string(),
        });
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(StoredText {
        fingerprint: fingerprint_of(&content),
        content,
    })
}

/// Holds an exclusive lock on `<path>.lock` until dropped, so the app and the
/// CLI never interleave a read-modify-write of the same file.
struct WriteLock(#[allow(dead_code)] File);

fn lock(path: &Path) -> Result<WriteLock, String> {
    let mut lock_path = path.as_os_str().to_owned();
    lock_path.push(".lock");
    let lock_path = PathBuf::from(lock_path);
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|e| e.to_string())?;
    file.lock().map_err(|e| e.to_string())?;
    Ok(WriteLock(file))
}

/// Runs `f` while holding the exclusive lock on `<path>.lock`, for a
/// read-modify-write of another file that the app and the CLI both touch.
/// Take it after (never before) the servers.yaml lock when holding both.
pub fn with_lock<T>(path: &Path, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _lock = lock(path)?;
    f()
}

/// The file parsed as plain data, keeping fields this build has no model for.
fn raw_value(content: &str) -> Result<Value, String> {
    if content.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_yaml::from_str(content).map_err(|e| e.to_string())
}

fn ensure_writable_version(path: &Path, raw: &Value) -> Result<(), String> {
    match raw.get("version").and_then(Value::as_u64) {
        Some(version) if version > CONFIG_VERSION => Err(format!(
            "{TOO_NEW_ERROR}: {} was saved by a newer version of MCP Manager or mcpmgr \
             (format {version}; this one writes format {CONFIG_VERSION}). Update to change it.",
            path.to_string_lossy()
        )),
        _ => Ok(()),
    }
}

/// Writes `content` unless the file no longer matches `expected` (the
/// fingerprint it was read with), returning the new fingerprint. `None` skips
/// the check. Refuses content or a file in a newer format.
pub fn write_text_checked(
    path: &Path,
    content: &str,
    expected: Option<&str>,
) -> Result<String, String> {
    let _lock = lock(path)?;
    // A file that does not parse has no version to protect, and saving over
    // it is how the app repairs it.
    for text in [read_text(path)?.content.as_str(), content] {
        if let Ok(raw) = raw_value(text) {
            ensure_writable_version(path, &raw)?;
        }
    }
    if let Some(expected) = expected {
        if fingerprint(path)? != expected {
            return Err(format!(
                "{CONFLICT_ERROR}: {} was changed by another program",
                path.to_string_lossy()
            ));
        }
    }
    atomic_write(path, content)?;
    Ok(fingerprint_of(content))
}

pub fn load(ctx: &PlatformContext) -> Result<MCPConfig, String> {
    parse_yaml_config(&read_text(&config_path(ctx))?.content)
}

/// Loads the config, lets `edit` change it, and saves the result, all under
/// the write lock. Nothing is written when `edit` fails, and `edit` is not
/// called for a file in a newer format. Fields and clients this build does
/// not know are written back as they were.
pub fn update<T>(
    ctx: &PlatformContext,
    edit: impl FnOnce(&mut MCPConfig) -> Result<T, String>,
) -> Result<T, String> {
    update_guarded(ctx, edit, |error| error)
}

/// Like [`update`], but when `edit` succeeded and saving the file then fails,
/// `on_save_failed` runs while the lock is still held (so nothing else can
/// write in between) and turns the save error into the one returned. Use it
/// to undo what `edit` did outside the file.
pub fn update_guarded<T>(
    ctx: &PlatformContext,
    edit: impl FnOnce(&mut MCPConfig) -> Result<T, String>,
    on_save_failed: impl FnOnce(String) -> String,
) -> Result<T, String> {
    let path = config_path(ctx);
    let _lock = lock(&path)?;
    let content = read_text(&path)?.content;
    let raw = raw_value(&content)?;
    ensure_writable_version(&path, &raw)?;
    let mut config = parse_yaml_config(&content)?;
    let parsed = to_value(&config)?;
    let result = edit(&mut config)?;
    let mut updated = to_value(&config)?;
    restore_unknown(&raw, &parsed, &mut updated);
    serde_yaml::to_string(&updated)
        .map_err(|e| e.to_string())
        .and_then(|yaml| atomic_write(&path, &yaml))
        .map_err(on_save_failed)?;
    Ok(result)
}

fn to_value(config: &MCPConfig) -> Result<Value, String> {
    let mut value = serde_json::to_value(config).map_err(|e| e.to_string())?;
    drop_nulls(&mut value);
    Ok(value)
}

/// Copies into `updated` what parsing `raw` dropped: anything in `raw` but not
/// in `parsed` (the same file after a parse and serialize). What the edit
/// itself removed is in `parsed`, so it stays removed.
fn restore_unknown(raw: &Value, parsed: &Value, updated: &mut Value) {
    match (raw, parsed, updated) {
        (Value::Object(raw), Value::Object(parsed), Value::Object(updated)) => {
            for (key, raw_value) in raw {
                match parsed.get(key) {
                    None if !raw_value.is_null() => {
                        updated
                            .entry(key.clone())
                            .or_insert_with(|| raw_value.clone());
                    }
                    Some(parsed_value) => {
                        if let Some(updated_value) = updated.get_mut(key) {
                            restore_unknown(raw_value, parsed_value, updated_value);
                        }
                    }
                    None => {}
                }
            }
        }
        (Value::Array(raw), Value::Array(parsed), Value::Array(updated)) => {
            for raw_item in raw {
                let Some(identity) = identity(raw_item) else {
                    continue;
                };
                let find = |items: &[Value]| {
                    items
                        .iter()
                        .position(|item| self::identity(item).as_ref() == Some(&identity))
                };
                match (find(parsed), find(updated)) {
                    // Skipped by parsing (a client this build does not know).
                    (None, _) => updated.push(raw_item.clone()),
                    (Some(p), Some(u)) => restore_unknown(raw_item, &parsed[p], &mut updated[u]),
                    // Removed by the edit.
                    (Some(_), None) => {}
                }
            }
        }
        _ => {}
    }
}

/// What makes a list entry the same entry before and after an edit: a
/// server's `id`, or a placement's client, scope and path.
fn identity(item: &Value) -> Option<Value> {
    let item = item.as_object()?;
    if let Some(id) = item.get("id") {
        return Some(id.clone());
    }
    item.get("app")?;
    Some(Value::Array(
        ["app", "scope", "path"]
            .iter()
            .map(|key| item.get(*key).cloned().unwrap_or(Value::Null))
            .collect(),
    ))
}

/// Serializes like the app's frontend does: absent optional fields are left
/// out instead of written as `null`, and keys come out in a stable order.
pub fn to_yaml(config: &MCPConfig) -> Result<String, String> {
    let mut value = serde_json::to_value(config).map_err(|e| e.to_string())?;
    drop_nulls(&mut value);
    serde_yaml::to_string(&value).map_err(|e| e.to_string())
}

fn drop_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            map.values_mut().for_each(drop_nulls);
        }
        Value::Array(items) => items.iter_mut().for_each(drop_nulls),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::to_yaml;
    use super::{load, read_text, update, write_text_checked, CONFLICT_ERROR, TOO_NEW_ERROR};
    use crate::core::{
        empty_apps, CommandSpec, MCPConfig, MCPServer, PlacementScope, ServerPlacement,
        SupportedApp, TransportSpec,
    };
    use crate::parser::parse_yaml_config;
    use crate::platform::{PlatformContext, PlatformOs};
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn sample() -> MCPConfig {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Cursor, true);
        MCPConfig {
            version: 1,
            servers: vec![
                MCPServer {
                    description: None,
                    homepage: None,
                    id: "linear".to_string(),
                    name: "linear".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "http".to_string(),
                        url: Some("https://mcp.linear.app/mcp".to_string()),
                        headers: Default::default(),
                    },
                    command: None,
                    apps: apps.clone(),
                    placements: vec![],
                },
                MCPServer {
                    description: Some("Docs lookup".to_string()),
                    homepage: None,
                    id: "context7".to_string(),
                    name: "context7".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                        headers: Default::default(),
                    },
                    command: Some(CommandSpec {
                        program: "npx".to_string(),
                        args: vec!["-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
                        env: HashMap::from([("TOKEN".to_string(), "${env:TOKEN}".to_string())]),
                        secret_env: Default::default(),
                    }),
                    apps,
                    placements: vec![ServerPlacement {
                        app: SupportedApp::ClaudeCode,
                        scope: PlacementScope::Workspace,
                        path: None,
                        enabled: true,
                        managed: true,
                    }],
                },
            ],
        }
    }

    #[test]
    fn writes_no_nulls_and_reads_back_the_same_config() {
        let yaml = to_yaml(&sample()).expect("to yaml");

        assert!(!yaml.contains("null"), "{yaml}");
        let parsed = parse_yaml_config(&yaml).expect("parse");
        assert_eq!(
            serde_json::to_value(&parsed).unwrap(),
            serde_json::to_value(sample()).unwrap()
        );
    }

    #[test]
    fn output_is_stable() {
        assert_eq!(to_yaml(&sample()).unwrap(), to_yaml(&sample()).unwrap());
    }

    #[test]
    fn reads_the_frontend_fixture() {
        let yaml = include_str!("../tests/fixtures/servers.frontend.yaml");
        let config = parse_yaml_config(yaml).expect("parse frontend yaml");

        assert_eq!(config.servers.len(), 2);
        assert!(config.servers[0].command.is_none());
        assert_eq!(config.servers[1].placements.len(), 1);
    }

    fn temp_ctx(dir: &std::path::Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::Linux,
            home_dir: dir.to_path_buf(),
            workspace_root: dir.to_path_buf(),
        }
    }

    #[test]
    fn missing_file_loads_as_empty_and_update_creates_it() {
        let dir = tempdir().expect("tempdir");
        let ctx = temp_ctx(dir.path());

        assert!(load(&ctx).expect("load").servers.is_empty());
        assert!(read_text(&dir.path().join("nope.yaml"))
            .unwrap()
            .content
            .contains("servers: []"));

        update(&ctx, |config| {
            *config = sample();
            Ok(())
        })
        .expect("update");
        assert_eq!(load(&ctx).expect("reload").servers.len(), 2);
    }

    #[test]
    fn failed_update_writes_nothing() {
        let dir = tempdir().expect("tempdir");
        let ctx = temp_ctx(dir.path());

        let result = update(&ctx, |config| {
            config.servers.clear();
            Err::<(), _>("nope".to_string())
        });

        assert!(result.is_err());
        assert!(!super::config_path(&ctx).exists());
    }

    #[test]
    fn checked_write_rejects_a_file_changed_since_it_was_read() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("servers.yaml");

        let absent = read_text(&path).unwrap().fingerprint;
        let first = write_text_checked(&path, "version: 1\nservers: []\n", Some(&absent))
            .expect("first write");
        assert_eq!(first, read_text(&path).unwrap().fingerprint);

        // Another program rewrites the file.
        std::fs::write(&path, "version: 1\nservers: [] # edited\n").unwrap();

        let error = write_text_checked(&path, "version: 1\n", Some(&first)).unwrap_err();
        assert!(error.starts_with(CONFLICT_ERROR), "{error}");
        assert!(read_text(&path).unwrap().content.contains("edited"));

        // Without an expected fingerprint the write goes through.
        write_text_checked(&path, "version: 1\n", None).expect("unchecked write");
    }

    #[test]
    fn a_file_created_elsewhere_conflicts_with_an_absent_read() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("servers.yaml");
        let absent = read_text(&path).unwrap().fingerprint;

        std::fs::write(&path, "version: 1\nservers: []\n").unwrap();

        assert!(write_text_checked(&path, "x", Some(&absent))
            .unwrap_err()
            .starts_with(CONFLICT_ERROR));
    }

    /// servers.yaml as a newer build might write it: a field on every level
    /// and a client this build does not know.
    const NEWER_FIELDS: &str = "\
version: 1
profiles: [work]
servers:
- id: linear
  name: linear
  enabled: true
  tags: [work]
  transport:
    type: http
    url: https://mcp.linear.app/mcp
    auth: oauth
  apps:
    cursor: true
    zed: true
  placements:
  - app: cursor
    scope: user
    enabled: true
    pinned: true
  - app: zed
    scope: user
    enabled: true
";

    fn write_config(ctx: &PlatformContext, content: &str) {
        let path = super::config_path(ctx);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn saved_value(ctx: &PlatformContext) -> serde_json::Value {
        serde_yaml::from_str(&std::fs::read_to_string(super::config_path(ctx)).unwrap()).unwrap()
    }

    #[test]
    fn unknown_clients_are_skipped_when_loading() {
        let dir = tempdir().expect("tempdir");
        let ctx = temp_ctx(dir.path());
        write_config(&ctx, NEWER_FIELDS);

        let config = load(&ctx).expect("load");

        let server = &config.servers[0];
        assert_eq!(server.apps.get(&SupportedApp::Cursor), Some(&true));
        assert_eq!(server.apps.len(), 1);
        assert_eq!(server.placements.len(), 1);
        assert_eq!(server.placements[0].app, SupportedApp::Cursor);
    }

    #[test]
    fn update_keeps_fields_and_clients_it_does_not_know() {
        let dir = tempdir().expect("tempdir");
        let ctx = temp_ctx(dir.path());
        write_config(&ctx, NEWER_FIELDS);

        update(&ctx, |config| {
            config.servers[0].name = "Linear".to_string();
            config.servers.push(sample().servers.remove(1));
            Ok(())
        })
        .expect("update");

        let saved = saved_value(&ctx);
        let linear = &saved["servers"][0];
        assert_eq!(linear["name"], "Linear");
        assert_eq!(saved["profiles"], serde_json::json!(["work"]));
        assert_eq!(linear["tags"], serde_json::json!(["work"]));
        assert_eq!(linear["transport"]["auth"], "oauth");
        assert_eq!(linear["apps"]["zed"], true);
        let placements = linear["placements"].as_array().unwrap();
        assert_eq!(placements.len(), 2);
        assert_eq!(placements[0]["pinned"], true);
        assert_eq!(placements[1]["app"], "zed");
        assert_eq!(saved["servers"][1]["id"], "context7");
    }

    #[test]
    fn update_still_removes_what_the_edit_removed() {
        let dir = tempdir().expect("tempdir");
        let ctx = temp_ctx(dir.path());
        write_config(&ctx, NEWER_FIELDS);

        update(&ctx, |config| {
            config.servers[0].placements.clear();
            config.servers[0].apps.remove(&SupportedApp::Cursor);
            Ok(())
        })
        .expect("update");
        let saved = saved_value(&ctx);
        let linear = &saved["servers"][0];
        assert!(linear["apps"].get("cursor").is_none());
        assert_eq!(linear["apps"]["zed"], true);
        let placements = linear["placements"].as_array().unwrap();
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0]["app"], "zed");

        update(&ctx, |config| {
            config.servers.clear();
            Ok(())
        })
        .expect("update");
        assert_eq!(saved_value(&ctx)["servers"], serde_json::json!([]));
        assert_eq!(saved_value(&ctx)["profiles"], serde_json::json!(["work"]));
    }

    #[test]
    fn a_newer_format_can_be_read_but_not_written() {
        let dir = tempdir().expect("tempdir");
        let ctx = temp_ctx(dir.path());
        let newer = NEWER_FIELDS.replace("version: 1", "version: 2");
        write_config(&ctx, &newer);

        assert_eq!(load(&ctx).expect("load").servers.len(), 1);

        let mut called = false;
        let error = update(&ctx, |_| {
            called = true;
            Ok(())
        })
        .unwrap_err();
        assert!(error.starts_with(TOO_NEW_ERROR), "{error}");
        assert!(
            !called,
            "the edit must not run, so no client file is written"
        );

        let path = super::config_path(&ctx);
        let current = read_text(&path).unwrap().fingerprint;
        let error =
            write_text_checked(&path, "version: 1\nservers: []\n", Some(&current)).unwrap_err();
        assert!(error.starts_with(TOO_NEW_ERROR), "{error}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), newer);
    }

    #[test]
    fn checked_write_refuses_newer_content_but_repairs_a_broken_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("servers.yaml");

        let error = write_text_checked(&path, "version: 2\nservers: []\n", None).unwrap_err();
        assert!(error.starts_with(TOO_NEW_ERROR), "{error}");

        std::fs::write(&path, "servers: [unclosed\n").unwrap();
        write_text_checked(&path, "version: 1\nservers: []\n", None).expect("repair");
    }
}
