use crate::adapters::{AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct CursorAdapter;

impl AppAdapter for CursorAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Cursor
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (ctx.workspace_file(".cursor/mcp.json").to_string_lossy().to_string(), 10),
            (
                ctx.user_app_config_path(SupportedApp::Cursor)
                    .to_string_lossy()
                    .to_string(),
                20,
            ),
        ]
    }

    fn parse_source(
        &self,
        _ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let parsed = parse_mcp_json(content);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: "cursor".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(content.to_string()),
            }],
            servers: enable_servers_for_app(parsed.servers, SupportedApp::Cursor)
                .into_iter()
                .map(|server| (server, priority))
                .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    fn plan_apply(&self, ctx: &PlatformContext, config: &MCPConfig) -> WriteOperation {
        let mut servers = serde_json::Map::new();
        for server in &config.servers {
            if server.enabled && server.apps.get(&SupportedApp::Cursor).copied().unwrap_or(false) {
                let value = if server.transport.kind == "stdio" {
                    serde_json::json!({
                        "command": server.command.as_ref().map(|c| c.program.clone()).unwrap_or_default(),
                        "args": server.command.as_ref().map(|c| c.args.clone()).unwrap_or_default(),
                        "env": server.command.as_ref().map(|c| c.env.clone()).unwrap_or_default()
                    })
                } else {
                    serde_json::json!({ "url": server.transport.url.clone().unwrap_or_default() })
                };
                servers.insert(server.id.clone(), value);
            }
        }

        WriteOperation {
            path: ctx
                .user_app_config_path(SupportedApp::Cursor)
                .to_string_lossy()
                .to_string(),
            mode: "replace_json".to_string(),
            field: None,
            content: serde_json::to_string(&serde_json::json!({ "mcpServers": servers }))
                .expect("serialize cursor"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CursorAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use std::path::PathBuf;

    fn ctx() -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: PathBuf::from("/Users/test"),
            workspace_root: PathBuf::from("/workspace/project"),
        }
    }

    #[test]
    fn detects_cursor_sources() {
        let sources = CursorAdapter.detect_sources(&ctx());
        assert_eq!(sources.len(), 2);
        assert!(sources[0].0.ends_with(".cursor/mcp.json"));
        assert!(sources[1].0.ends_with(".cursor/mcp.json"));
    }

    #[test]
    fn plans_cursor_payload() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Cursor, true);
        let op = CursorAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    id: "linear".to_string(),
                    name: "Linear".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "http".to_string(),
                        url: Some("https://mcp.linear.app/mcp".to_string()),
                    },
                    command: None,
                    apps,
                }],
            },
        );
        assert_eq!(op.mode, "replace_json");
        assert!(op.content.contains("\"mcpServers\""));
        assert!(op.content.contains("linear"));
    }
}
