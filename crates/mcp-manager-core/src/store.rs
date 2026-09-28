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

const EMPTY_CONFIG_YAML: &str = "version: 1\nservers: []\n";
const ABSENT_FINGERPRINT: &str = "absent";

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

/// Writes `content` unless the file no longer matches `expected` (the
/// fingerprint it was read with), returning the new fingerprint. `None` skips
/// the check.
pub fn write_text_checked(
    path: &Path,
    content: &str,
    expected: Option<&str>,
) -> Result<String, String> {
    let _lock = lock(path)?;
    if let Some(expected) = expected {
        if fingerprint(path)? != expected {
            return Err(format!(
                "{CONFLICT_ERROR}: {} was changed by another program",
                path.to_string_lossy()
            ));
        }
    }
    atomic_write(&path.to_path_buf(), content)?;
    Ok(fingerprint_of(content))
}

pub fn load(ctx: &PlatformContext) -> Result<MCPConfig, String> {
    parse_yaml_config(&read_text(&config_path(ctx))?.content)
}

/// Loads the config, lets `edit` change it, and saves the result, all under
/// the write lock. Nothing is written when `edit` fails.
pub fn update<T>(
    ctx: &PlatformContext,
    edit: impl FnOnce(&mut MCPConfig) -> Result<T, String>,
) -> Result<T, String> {
    let path = config_path(ctx);
    let _lock = lock(&path)?;
    let mut config = parse_yaml_config(&read_text(&path)?.content)?;
    let result = edit(&mut config)?;
    atomic_write(&path, &to_yaml(&config)?)?;
    Ok(result)
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
    use super::{load, read_text, update, write_text_checked, CONFLICT_ERROR};
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
                    },
                    command: Some(CommandSpec {
                        program: "npx".to_string(),
                        args: vec!["-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
                        env: HashMap::from([("TOKEN".to_string(), "${env:TOKEN}".to_string())]),
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
}
