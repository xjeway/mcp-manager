use crate::adapters::{
    managed_json_field_writes_with, standard_mcp_servers_at, AppAdapter, ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, PlacementScope, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::Value;

/// Owns only Command Code's user file; Claude Code owns the shared project file.
pub struct CommandCodeAdapter;

/// Normalize the documented transport key before using the common MCP parser.
fn normalize(content: &str) -> Result<String, String> {
    let mut value: Value = serde_json::from_str(content).map_err(|error| error.to_string())?;
    let entries = value
        .get_mut("mcpServers")
        .and_then(Value::as_object_mut)
        .ok_or("Command Code configuration requires an mcpServers object")?;
    for (id, entry) in entries {
        let transport = entry
            .get("transport")
            .or_else(|| entry.get("type"))
            .and_then(Value::as_str);
        match transport {
            Some("http") => {
                entry["type"] = "http".into();
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
                    "server {id}: unsupported Command Code transport {other}"
                ))
            }
            None => {}
        }
    }
    Ok(value.to_string())
}

/// The client reads strict JSON, so reject JSONC before any app or store is written.
pub(crate) fn validate_command_context(
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
                .get(&SupportedApp::CommandCode)
                .copied()
                .unwrap_or(false)
                || server
                    .placements
                    .iter()
                    .any(|p| p.app == SupportedApp::CommandCode && p.managed)
        });
    if targeted {
        let path = ctx.user_app_config_path(SupportedApp::CommandCode);
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

impl AppAdapter for CommandCodeAdapter {
    /// Registry and CLI identity.
    fn app(&self) -> SupportedApp {
        SupportedApp::CommandCode
    }

    /// Project and private local files have no second writer here.
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
        let normalized = match normalize(content) {
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
    use crate::core::TransportSpec;
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::fs;

    #[test]
    fn canonical_transport_roundtrip_keeps_refs_oauth_and_disabled_state() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let path = ctx.user_app_config_path(SupportedApp::CommandCode);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = json!({"settings":{"keep":true},"mcpServers":{
            "api":{"transport":"http","type":"stdio","enabled":false,"url":"https://example.com/mcp","headers":{"Authorization":"Bearer ${TOKEN:-fallback}","X-Literal":"$${NAME}"},"oauth":{"clientId":"client"}},
            "local":{"transport":"stdio","command":"node","args":["server.js"],"env":{"TOKEN":"${TOKEN}"}},
            "unmanaged":{"transport":"stdio","command":"keep"}
        }}).to_string();
        fs::write(&path, &original).unwrap();
        let parsed = CommandCodeAdapter.parse_source(&ctx, path.to_str().unwrap(), 20, &original);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let mut servers: Vec<_> = parsed
            .servers
            .into_iter()
            .map(|(server, _)| server)
            .filter(|server| server.id != "unmanaged")
            .collect();
        let api = servers.iter().find(|server| server.id == "api").unwrap();
        assert!(api.enabled);
        assert_eq!(api.transport.kind, "http");
        assert_eq!(api.transport.headers["X-Literal"], "$${NAME}");
        assert_eq!(
            api.transport.headers["Authorization"],
            "Bearer ${TOKEN:-fallback}"
        );
        let config = MCPConfig {
            version: 1,
            servers: servers.clone(),
        };
        for operation in CommandCodeAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&path, &operation).unwrap();
        }
        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["mcpServers"]["api"]["transport"], "http");
        assert!(written["mcpServers"]["api"].get("type").is_none());
        assert_eq!(written["mcpServers"]["api"]["enabled"], false);
        assert_eq!(
            written["mcpServers"]["api"]["oauth"],
            json!({"clientId":"client"})
        );
        assert_eq!(written["settings"], json!({"keep":true}));
        assert_eq!(written["mcpServers"]["unmanaged"]["command"], "keep");
        let command = servers
            .iter()
            .find(|server| server.id == "local")
            .unwrap()
            .command
            .clone();
        let api = servers
            .iter_mut()
            .find(|server| server.id == "api")
            .unwrap();
        api.transport = TransportSpec {
            kind: "stdio".into(),
            url: None,
            headers: Default::default(),
        };
        api.command = command;
        let changed = MCPConfig {
            version: 1,
            servers,
        };
        for operation in CommandCodeAdapter.plan_apply(&ctx, &changed, Some(&config)) {
            apply_operation(&path, &operation).unwrap();
        }
        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["mcpServers"]["api"]["transport"], "stdio");
        assert_eq!(written["mcpServers"]["api"]["enabled"], false);
        assert!(written["mcpServers"]["api"].get("url").is_none());
    }

    #[test]
    fn recognizes_alias_and_rejects_unsupported_transports() {
        assert_eq!(SupportedApp::parse("cmd"), Some(SupportedApp::CommandCode));
        let normalized =
            normalize(r#"{"mcpServers":{"api":{"type":"http","url":"https://example.com"}}}"#)
                .unwrap();
        assert_eq!(
            parse_mcp_json(&normalized).servers[0].transport.kind,
            "http"
        );
        for content in [
            r#"{"mcpServers":{"api":{"transport":"sse","url":"https://example.com"}}}"#,
            r#"{"mcpServers":{"api":{"transport":"stdio","url":"https://example.com"}}}"#,
        ] {
            assert!(normalize(content).is_err());
        }
    }

    #[test]
    fn windows_system_shell_does_not_count_as_command_code() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::Windows,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        assert!(!ctx
            .detect_installed_apps()
            .contains(&SupportedApp::CommandCode));
        let path = ctx.user_app_config_path(SupportedApp::CommandCode);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "{}").unwrap();
        assert!(ctx
            .detect_installed_apps()
            .contains(&SupportedApp::CommandCode));
    }
    #[test]
    fn strict_jsonc_is_rejected_before_other_clients_are_written() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let path = ctx.user_app_config_path(SupportedApp::CommandCode);
        let parsed = CommandCodeAdapter.parse_source(
            &ctx,
            path.to_str().unwrap(),
            20,
            r#"{"mcpServers":{"api":{"transport":"http","url":"https://example.com"}}}"#,
        );
        let config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        for text in [
            "{ // not strict JSON\n\"mcpServers\":{}}",
            "{\"mcpServers\":{},}",
        ] {
            fs::write(&path, text).unwrap();
            assert!(crate::workflow::prepare(&ctx, &config, None).is_err());
            assert!(crate::workflow::apply(&ctx, &config, None).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
        }
    }
}
