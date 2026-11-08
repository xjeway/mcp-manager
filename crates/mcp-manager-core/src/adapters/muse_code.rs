use crate::adapters::{
    managed_json_field_writes_with, standard_mcp_servers_at, AppAdapter, ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, PlacementScope, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::{json, Value};

/// Muse settings use a versioned user-only schema, distinct from its SDK wire protocol.
pub struct MuseCodeAdapter;

/// Refuse ambiguous legacy keys and schemas rather than silently disabling Muse's servers.
pub(crate) fn validate_document(content: &str) -> Result<Value, String> {
    let value: Value = serde_json::from_str(content).map_err(|error| error.to_string())?;
    if !value.is_object() || value.get("schema_version") != Some(&json!(1)) {
        return Err(
            "Muse settings require schema_version: 1; existing schema is left untouched".into(),
        );
    }
    if value.get("mcpServers").is_some() {
        return Err("Muse settings contain legacy mcpServers; resolve it manually before writing mcp_servers".into());
    }
    if value
        .get("mcp_servers")
        .is_some_and(|entries| !entries.is_object())
    {
        return Err("Muse mcp_servers must be an object".into());
    }
    Ok(value)
}

/// Guard the complete apply before any client file is modified.
pub(crate) fn validate_muse_context(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
) -> Result<(), String> {
    let targeted = std::iter::once(config)
        .chain(previous)
        .flat_map(|config| &config.servers)
        .any(|server| {
            server
                .apps
                .get(&SupportedApp::MuseCode)
                .copied()
                .unwrap_or(false)
                || server
                    .placements
                    .iter()
                    .any(|p| p.app == SupportedApp::MuseCode && p.managed)
        });
    if targeted {
        let path = ctx.user_app_config_path(SupportedApp::MuseCode);
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                validate_document(&content)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
    }
    Ok(())
}

/// Serialize only documented native transports and preserve the client's enabled switch.
fn content_at(config: &MCPConfig, app: SupportedApp, path: &str, ctx: &PlatformContext) -> Value {
    let old = std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok());
    let mut entries = standard_mcp_servers_at(config, app, path, ctx);
    if let Some(entries) = entries.as_object_mut() {
        for (id, entry) in entries {
            let fields = entry.as_object_mut().expect("server object");
            let transport = fields.remove("type").unwrap_or_else(|| json!("stdio"));
            fields.insert(
                "transport".into(),
                if transport == "stdio" {
                    json!("stdio")
                } else {
                    json!("streamable_http")
                },
            );
            if let Some(previous) = old
                .as_ref()
                .and_then(|value| value.get("mcp_servers"))
                .and_then(|entries| entries.get(id))
            {
                if let Some(enabled) = previous.get("enabled") {
                    fields.insert("enabled".into(), enabled.clone());
                }
                if let Some(mode) = previous.get("mode") {
                    fields.insert("mode".into(), mode.clone());
                }
            }
        }
    }
    entries
}

impl AppAdapter for MuseCodeAdapter {
    /// Registry identity and CLI alias muse.
    fn app(&self) -> SupportedApp {
        SupportedApp::MuseCode
    }

    /// Muse has no project MCP settings file.
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        )]
    }

    /// Normalize native settings without expanding environment values.
    fn parse_source(
        &self,
        _ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let source = LocalConfigSource {
            app: self.app().as_str().into(),
            path: path.into(),
            exists: true,
            format: "json".into(),
            priority,
            content: Some(content.into()),
        };
        let normalized = (|| {
            let mut value = validate_document(content)?;
            let mut entries = value
                .get_mut("mcp_servers")
                .cloned()
                .unwrap_or_else(|| json!({}));
            for (id, entry) in entries.as_object_mut().expect("validated map") {
                let transport = match entry.get("transport").and_then(Value::as_str) {
                    Some("stdio") => "stdio",
                    Some("streamable_http") => "http",
                    _ => {
                        return Err(format!(
                            "server {id}: Muse requires stdio or streamable_http transport"
                        ))
                    }
                };
                if transport == "stdio" && entry.get("url").is_some() {
                    return Err(format!("server {id}: stdio entry cannot contain a URL"));
                }
                entry["type"] = transport.into();
            }
            Ok(json!({"mcpServers":entries}).to_string())
        })();
        let normalized = match normalized {
            Ok(content) => content,
            Err(error) => {
                return ParsedSources {
                    sources: vec![source],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![format!("{path}: {error}")],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![source],
            servers: enable_servers_for_app_at(
                parsed.servers,
                self.app(),
                PlacementScope::User,
                path.into(),
            )
            .into_iter()
            .map(|server| (server, priority))
            .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    /// One atomic schema-aware merge; no project or legacy-key writes.
    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        let user = ctx
            .user_app_config_path(self.app())
            .to_string_lossy()
            .to_string();
        managed_json_field_writes_with(ctx, "mcp_servers", self.app(), config, previous, content_at)
            .into_iter()
            .filter(|operation| operation.path == user)
            .map(|mut operation| {
                operation.mode = "merge_muse_mcp_entries".into();
                operation
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use std::{fs, path::Path};

    fn ctx(root: &Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: root.join("home"),
            workspace_root: root.join("project"),
        }
    }

    #[test]
    fn native_roundtrip_keeps_settings_mode_and_environment_references() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::MuseCode);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = json!({"schema_version":1,"hooks":{"keep":true},"mcp_servers":{
            "api":{"transport":"streamable_http","url":"https://example.com/mcp","headers":{"Authorization":"Bearer ${TOKEN}"},"enabled":false,"mode":"optional"},
            "local":{"transport":"stdio","command":"node","args":["server.js"],"env":{"TOKEN":"${TOKEN}"},"framing":"line"},
            "unmanaged":{"transport":"stdio","command":"keep"}
        }}).to_string();
        fs::write(&path, &original).unwrap();
        let parsed = MuseCodeAdapter.parse_source(&ctx, path.to_str().unwrap(), 20, &original);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let mut config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .filter(|server| server.id != "unmanaged")
                .collect(),
        };
        assert_eq!(
            config
                .servers
                .iter()
                .find(|s| s.id == "api")
                .unwrap()
                .transport
                .headers["Authorization"],
            "Bearer ${TOKEN}"
        );
        for _ in 0..2 {
            for operation in MuseCodeAdapter.plan_apply(&ctx, &config, None) {
                apply_operation(&path, &operation).unwrap();
            }
        }
        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert!(value.get("mcpServers").is_none());
        assert_eq!(value["hooks"], json!({"keep":true}));
        assert_eq!(value["mcp_servers"]["api"]["transport"], "streamable_http");
        assert_eq!(value["mcp_servers"]["api"]["enabled"], false);
        assert_eq!(value["mcp_servers"]["api"]["mode"], "optional");
        assert_eq!(value["mcp_servers"]["local"]["framing"], "line");
        assert_eq!(value["mcp_servers"]["unmanaged"]["command"], "keep");
        let previous = config.clone();
        config.servers.retain(|server| server.id != "api");
        for operation in MuseCodeAdapter.plan_apply(&ctx, &config, Some(&previous)) {
            apply_operation(&path, &operation).unwrap();
        }
        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(value["mcp_servers"].get("api").is_none());
        assert!(value["mcp_servers"].get("unmanaged").is_some());
    }

    #[test]
    fn initializes_required_schema_in_a_single_operation_and_previews_entries() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let content = r#"{"schema_version":1,"mcp_servers":{"api":{"transport":"streamable_http","url":"https://example.com/mcp"}}}"#;
        let path = ctx.user_app_config_path(SupportedApp::MuseCode);
        let parsed = MuseCodeAdapter.parse_source(&ctx, path.to_str().unwrap(), 20, content);
        let config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        let prepared = crate::workflow::prepare(&ctx, &config, None).unwrap();
        assert!(prepared
            .changes
            .iter()
            .any(|change| change.app == SupportedApp::MuseCode));
        let operations = MuseCodeAdapter.plan_apply(&ctx, &config, None);
        assert_eq!(operations.len(), 1);
        apply_operation(&path, &operations[0]).unwrap();
        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["mcp_servers"]["api"]["transport"], "streamable_http");
    }

    #[test]
    fn conflicting_keys_or_unknown_schemas_abort_before_any_write() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::MuseCode);
        let parsed = MuseCodeAdapter.parse_source(&ctx,path.to_str().unwrap(),20,r#"{"schema_version":1,"mcp_servers":{"local":{"transport":"stdio","command":"node"}}}"#);
        let config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        for content in [
            r#"{"schema_version":1,"mcpServers":{}}"#,
            r#"{"schema_version":1,"mcp_servers":{},"mcpServers":{}}"#,
            r#"{"schema_version":2}"#,
            r#"{"mcp_servers":{}}"#,
            "",
        ] {
            fs::write(&path, content).unwrap();
            assert!(crate::workflow::prepare(&ctx, &config, None).is_err());
            assert!(crate::workflow::apply(&ctx, &config, None).is_err());
            assert!(!MuseCodeAdapter
                .parse_source(&ctx, path.to_str().unwrap(), 20, content)
                .errors
                .is_empty());
            assert_eq!(fs::read_to_string(&path).unwrap(), content);
        }
    }
}
