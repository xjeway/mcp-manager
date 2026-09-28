//! The unified server list (`servers.yaml`) that the app and the CLI share.

use crate::core::MCPConfig;
use crate::parser::parse_yaml_config;
use crate::platform::PlatformContext;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_RELATIVE_PATH: &str = "config/servers.yaml";

const EMPTY_CONFIG_YAML: &str = "version: 1\nservers: []\n";

pub fn config_path(ctx: &PlatformContext) -> PathBuf {
    ctx.app_data_dir().join(CONFIG_RELATIVE_PATH)
}

/// The file's text, or an empty config when it does not exist yet.
pub fn read_text(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(EMPTY_CONFIG_YAML.to_string());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn write_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn load(ctx: &PlatformContext) -> Result<MCPConfig, String> {
    parse_yaml_config(&read_text(&config_path(ctx))?)
}

pub fn save(ctx: &PlatformContext, config: &MCPConfig) -> Result<(), String> {
    write_text(&config_path(ctx), &to_yaml(config)?)
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
    use super::{load, read_text, save, to_yaml};
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

    #[test]
    fn missing_file_loads_as_empty_and_save_creates_it() {
        let dir = tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: dir.path().to_path_buf(),
            workspace_root: dir.path().to_path_buf(),
        };

        assert!(load(&ctx).expect("load").servers.is_empty());
        assert!(read_text(&dir.path().join("nope.yaml"))
            .unwrap()
            .contains("servers: []"));

        save(&ctx, &sample()).expect("save");
        assert_eq!(load(&ctx).expect("reload").servers.len(), 2);
    }
}
