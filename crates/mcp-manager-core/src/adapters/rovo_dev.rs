use crate::adapters::{
    managed_json_field_writes_with, standard_mcp_servers_at, AppAdapter, ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, PlacementScope, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::Value;

/// Owns only Rovo Dev's user MCP file selected by its read-only YAML settings.
pub struct RovoDevAdapter;

/// Normalize the documented transport key before using the common MCP parser.
fn normalize(content: &str) -> Result<String, String> {
    let mut value: Value = serde_json::from_str(content).map_err(|error| error.to_string())?;
    let entries = value
        .get_mut("mcpServers")
        .and_then(Value::as_object_mut)
        .ok_or("Rovo Dev configuration requires an mcpServers object")?;
    for (id, entry) in entries {
        let transport = entry
            .get("transport")
            .or_else(|| entry.get("type"))
            .and_then(Value::as_str);
        match transport {
            Some("http" | "sse") => {
                entry["type"] = transport.unwrap().into();
            }
            Some("stdio") => {
                if entry.get("url").is_some()
                    || entry.get("httpUrl").is_some()
                    || entry.get("serverUrl").is_some()
                {
                    return Err(format!(
                        "server {id}: stdio transport cannot contain a remote URL"
                    ));
                }
                entry["type"] = "stdio".into();
            }
            Some(other) => {
                return Err(format!(
                    "server {id}: unsupported Rovo Dev transport {other}"
                ))
            }
            None => {}
        }
    }
    Ok(value.to_string())
}

/// Checks YAML resolution and JSON input before any client is changed.
pub(crate) fn validate_rovo_context(
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
                .get(&SupportedApp::RovoDev)
                .copied()
                .unwrap_or(false)
                || server
                    .placements
                    .iter()
                    .any(|p| p.app == SupportedApp::RovoDev && p.managed)
        });
    if targeted {
        let path = ctx.rovo_mcp_path()?;
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                normalize(&content).map_err(|error| format!("{}: {error}", path.display()))?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
    }
    Ok(())
}

/// Emits canonical transport names and preserves the native per-client enabled switch.
fn content_at(config: &MCPConfig, app: SupportedApp, path: &str, ctx: &PlatformContext) -> Value {
    let old = std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok());
    let mut entries = standard_mcp_servers_at(config, app, path, ctx);
    if let Some(entries) = entries.as_object_mut() {
        for (id, entry) in entries {
            let fields = entry.as_object_mut().expect("MCP entry object");
            let transport = fields
                .remove("type")
                .unwrap_or_else(|| Value::String("stdio".into()));
            let transport = if transport == "streamable-http" {
                Value::String("http".into())
            } else {
                transport
            };
            fields.insert("transport".into(), transport);
            if let Some(enabled) = old
                .as_ref()
                .and_then(|value| value.get("mcpServers"))
                .and_then(|entries| entries.get(id))
                .and_then(|entry| entry.get("enabled"))
            {
                fields.insert("enabled".into(), enabled.clone());
            }
        }
    }
    entries
}

impl AppAdapter for RovoDevAdapter {
    /// Registry identity and CLI aliases.
    fn app(&self) -> SupportedApp {
        SupportedApp::RovoDev
    }

    /// Reads the resolved user MCP file; no project MCP configuration is assumed.
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        )]
    }

    /// Imports raw interpolation and escape strings without resolving credentials.
    fn parse_source(
        &self,
        ctx: &PlatformContext,
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
        let normalized = match ctx.rovo_mcp_path().and_then(|_| normalize(content)) {
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

    /// Rewrites managed definitions in the user file while preserving unrelated entries.
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
        managed_json_field_writes_with(ctx, "mcpServers", self.app(), config, previous, content_at)
            .into_iter()
            .filter(|operation| operation.path == user)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::{fs, path::Path};

    fn ctx(root: &Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: root.join("home"),
            workspace_root: root.join("project"),
        }
    }

    #[test]
    fn resolves_default_tilde_and_absolute_overrides_and_rejects_ambiguous_yaml() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        assert_eq!(
            ctx.rovo_mcp_path().unwrap(),
            ctx.home_dir.join(".rovodev/mcp.json")
        );
        let yaml = ctx.home_dir.join(".rovodev/config.yml");
        fs::create_dir_all(yaml.parent().unwrap()).unwrap();
        fs::write(
            &yaml,
            "mcp:\n  mcpConfigPath: '~/custom/mcp.json'\n  disabledMcpServers: [api]\n",
        )
        .unwrap();
        assert_eq!(
            ctx.rovo_mcp_path().unwrap(),
            ctx.home_dir.join("custom/mcp.json")
        );
        let absolute = temp.path().join("custom.json");
        fs::write(
            &yaml,
            format!(
                "mcp:\n  mcpConfigPath: {}\n",
                serde_json::to_string(&absolute).unwrap()
            ),
        )
        .unwrap();
        assert_eq!(ctx.rovo_mcp_path().unwrap(), absolute);
        for text in [
            "mcp: [",
            "mcp: false",
            "mcp:\n  mcpConfigPath: false",
            "mcp:\n  mcpConfigPath: relative.json",
        ] {
            fs::write(&yaml, text).unwrap();
            assert!(ctx.rovo_mcp_path().is_err());
        }
    }

    #[test]
    fn native_transports_write_in_place_without_changing_disabled_server_lists() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let yaml = ctx.home_dir.join(".rovodev/config.yml");
        fs::create_dir_all(yaml.parent().unwrap()).unwrap();
        let settings = "mcp:\n  mcpConfigPath: '~/custom/mcp.json'\n  disabledMcpServers: [api]\n";
        fs::write(&yaml, settings).unwrap();
        let path = ctx.rovo_mcp_path().unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original=json!({"metadata":{"keep":true},"mcpServers":{
            "api":{"transport":"http","url":"https://example.com/mcp","headers":{"Authorization":"Bearer ${TOKEN}"},"oauth":{"clientId":"keep"}},
            "events":{"transport":"sse","url":"https://example.com/sse"},
            "local":{"transport":"stdio","command":"node","args":["server.js"],"env":{"TOKEN":"${TOKEN}"}},
            "unmanaged":{"command":"keep"}
        }}).to_string();
        fs::write(&path, &original).unwrap();
        let parsed = RovoDevAdapter.parse_source(&ctx, path.to_str().unwrap(), 20, &original);
        assert!(parsed.errors.is_empty());
        let mut config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(s, _)| s)
                .filter(|s| s.id != "unmanaged")
                .collect(),
        };
        assert!(
            config
                .servers
                .iter()
                .find(|s| s.id == "api")
                .unwrap()
                .enabled
        );
        assert_eq!(
            config
                .servers
                .iter()
                .find(|s| s.id == "events")
                .unwrap()
                .transport
                .kind,
            "sse"
        );
        for _ in 0..2 {
            for operation in RovoDevAdapter.plan_apply(&ctx, &config, None) {
                apply_operation(&path, &operation).unwrap();
            }
        }
        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["mcpServers"]["api"]["transport"], "http");
        assert_eq!(value["mcpServers"]["events"]["transport"], "sse");
        assert_eq!(value["mcpServers"]["api"]["oauth"]["clientId"], "keep");
        assert_eq!(
            value["mcpServers"]["api"]["headers"]["Authorization"],
            "Bearer ${TOKEN}"
        );
        assert_eq!(value["metadata"]["keep"], true);
        assert_eq!(fs::read_to_string(&yaml).unwrap(), settings);
        let previous = config.clone();
        config.servers.retain(|s| s.id != "api");
        for operation in RovoDevAdapter.plan_apply(&ctx, &config, Some(&previous)) {
            apply_operation(&path, &operation).unwrap();
        }
        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(value["mcpServers"].get("api").is_none());
        assert!(value["mcpServers"].get("unmanaged").is_some());
        assert!(!ctx.home_dir.join(".rovodev/mcp.json").exists());
    }

    #[test]
    fn changed_override_or_invalid_yaml_aborts_a_prepared_write() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::RovoDev);
        let parsed = RovoDevAdapter.parse_source(
            &ctx,
            path.to_str().unwrap(),
            20,
            r#"{"mcpServers":{"local":{"transport":"stdio","command":"node"}}}"#,
        );
        let config = MCPConfig {
            version: 1,
            servers: parsed.servers.into_iter().map(|(s, _)| s).collect(),
        };
        let prepared = crate::workflow::prepare(&ctx, &config, None).unwrap();
        let yaml = ctx.home_dir.join(".rovodev/config.yml");
        fs::create_dir_all(yaml.parent().unwrap()).unwrap();
        for content in ["mcp:\n  mcpConfigPath: '~/other/mcp.json'", "mcp: ["] {
            fs::write(&yaml, content).unwrap();
            assert!(prepared.apply(&ctx).is_err());
            assert!(!path.exists());
            assert!(!ctx.home_dir.join("other/mcp.json").exists());
        }
    }
}
