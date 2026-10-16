use crate::adapters::{
    hermes_mcp_servers_at, managed_yaml_mapping_writes, scope_for_path, AppAdapter, ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};

pub struct HermesAdapter;

const FIELD: &str = "mcp_servers";

/// `mcp_servers` as an `mcpServers` document the shared parser reads.
///
/// `enabled: false` turns a server off in Hermes only. It is not read as the
/// server being off everywhere, and the next apply keeps it in the file.
fn normalize(content: &str) -> Result<String, String> {
    let root: Value = if content.trim().is_empty() {
        Value::Null
    } else {
        serde_yaml::from_str(content).map_err(|e| e.to_string())?
    };
    let mut servers = Map::new();
    let entries = root
        .get(FIELD)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (id, entry) in entries {
        let Some(entry) = entry.as_object() else {
            continue;
        };
        let mut server = Map::new();
        if let Some(command) = entry.get("command") {
            server.insert("command".into(), command.clone());
            for key in ["args", "env"] {
                if let Some(value) = entry.get(key) {
                    server.insert(key.into(), value.clone());
                }
            }
        } else if let Some(url) = entry.get("url") {
            server.insert("url".into(), url.clone());
            let sse = entry.get("transport").and_then(Value::as_str) == Some("sse");
            server.insert("type".into(), Value::from(if sse { "sse" } else { "http" }));
            if let Some(headers) = entry.get("headers") {
                server.insert("headers".into(), headers.clone());
            }
        } else {
            continue;
        }
        servers.insert(id, Value::Object(server));
    }
    let mut document = Map::new();
    document.insert("mcpServers".to_string(), Value::Object(servers));
    serde_json::to_string(&Value::Object(document)).map_err(|e| e.to_string())
}

impl AppAdapter for HermesAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Hermes
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        )]
    }

    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let source = |content: String| LocalConfigSource {
            app: self.app().as_str().to_string(),
            path: path.to_string(),
            exists: true,
            format: "json".to_string(),
            priority,
            content: Some(content),
        };
        let normalized = match normalize(content) {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![source(content.to_string())],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![error],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![source(normalized)],
            servers: enable_servers_for_app_at(
                parsed.servers,
                self.app(),
                scope_for_path(ctx, path),
                path.to_string(),
            )
            .into_iter()
            .map(|server| (server, priority))
            .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous_config: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        managed_yaml_mapping_writes(
            ctx,
            FIELD,
            self.app(),
            config,
            previous_config,
            hermes_mcp_servers_at,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::HermesAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::PathBuf;

    const CONFIG: &str = "# Hermes config
model:
  default: anthropic/claude  # picked by hand

mcp_servers:
  # Local files
  filesystem:
    command: npx
    args: [\"-y\", \"@modelcontextprotocol/server-filesystem\", \"/home/me\"]
    idle_timeout_seconds: 600
    enabled: false

  legacy:
    url: https://legacy.example.com/sse
    transport: sse
    headers:
      Authorization: Bearer old

# Terminal settings
terminal:
  backend: local
";

    fn ctx(home: PathBuf) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.clone(),
            workspace_root: home,
        }
    }

    #[test]
    fn imports_stdio_and_sse_servers_and_keeps_hermes_only_switches_out() {
        let parsed = HermesAdapter.parse_source(
            &ctx(PathBuf::from("/home/test")),
            "/home/test/.hermes/config.yaml",
            20,
            CONFIG,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let server = |id: &str| {
            parsed
                .servers
                .iter()
                .find(|(server, _)| server.id == id)
                .map(|(server, _)| server.clone())
                .unwrap()
        };
        let filesystem = server("filesystem");
        assert!(filesystem.enabled, "turned off in Hermes only");
        assert_eq!(filesystem.command.unwrap().args.len(), 3);
        let legacy = server("legacy");
        assert_eq!(legacy.transport.kind, "sse");
        assert_eq!(legacy.transport.headers["Authorization"], "Bearer old");
    }

    #[test]
    fn apply_edits_entries_in_place() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(home.path().to_path_buf());
        let path = ctx.user_app_config_path(SupportedApp::Hermes);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, CONFIG).unwrap();

        let parsed = HermesAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, CONFIG);
        let mut servers = parsed
            .servers
            .into_iter()
            .map(|(server, _)| server)
            .collect::<Vec<_>>();
        for server in &mut servers {
            if server.id == "legacy" {
                // Moved to streamable HTTP.
                server.transport.kind = "http".into();
                server.transport.url = Some("https://legacy.example.com/mcp".into());
            }
            if server.id == "filesystem" {
                server.command.as_mut().unwrap().args = vec!["-y".into(), "fs-mcp".into()];
            }
        }
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Hermes, true);
        servers.push(MCPServer {
            id: "fresh".into(),
            name: "fresh".into(),
            enabled: true,
            transport: TransportSpec {
                kind: "sse".into(),
                url: Some("https://fresh.example.com/sse".into()),
                headers: Default::default(),
            },
            command: None,
            apps,
            placements: vec![],
            description: None,
            homepage: None,
        });
        let config = MCPConfig {
            version: 1,
            servers,
        };
        for operation in HermesAdapter.plan_apply(&ctx, &config, None) {
            assert_eq!(operation.mode, "merge_yaml_mapping_entries");
            apply_operation(&PathBuf::from(&operation.path), &operation).expect("apply");
        }

        let written = fs::read_to_string(&path).unwrap();
        for kept in [
            "# Hermes config\n",
            "  default: anthropic/claude  # picked by hand\n",
            "  # Local files\n",
            "# Terminal settings\nterminal:\n  backend: local\n",
        ] {
            assert!(written.contains(kept), "{kept:?} missing from\n{written}");
        }
        let parsed: Value = serde_yaml::from_str(&written).unwrap();
        let servers = &parsed["mcp_servers"];
        // Hermes-only settings stay, including the server being off there.
        assert_eq!(servers["filesystem"]["enabled"], false);
        assert_eq!(servers["filesystem"]["idle_timeout_seconds"], 600);
        assert_eq!(servers["filesystem"]["args"], json!(["-y", "fs-mcp"]));
        // Moving off SSE drops the old transport.
        assert_eq!(
            servers["legacy"],
            json!({"url": "https://legacy.example.com/mcp", "headers": {"Authorization": "Bearer old"}})
        );
        assert_eq!(servers["fresh"]["transport"], "sse");
    }
}
