use crate::adapters::{AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, extract_claude_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct ClaudeCodeAdapter;

impl AppAdapter for ClaudeCodeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::ClaudeCode
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (ctx.workspace_file(".mcp.json").to_string_lossy().to_string(), 10),
            (
                ctx.user_app_config_path(SupportedApp::ClaudeCode)
                    .to_string_lossy()
                    .to_string(),
                20,
            ),
        ]
    }

    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let normalized = if path.ends_with(".claude.json") {
            match extract_claude_mcp_json(content, &ctx.workspace_root.to_string_lossy()) {
                Ok(value) => value,
                Err(error) => {
                    return ParsedSources {
                        sources: vec![LocalConfigSource {
                            app: "claudeCode".to_string(),
                            path: path.to_string(),
                            exists: true,
                            format: "json".to_string(),
                            priority,
                            content: Some(content.to_string()),
                        }],
                        servers: vec![],
                        warnings: vec![],
                        errors: vec![error],
                    }
                }
            }
        } else {
            content.to_string()
        };

        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: "claudeCode".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(normalized),
            }],
            servers: enable_servers_for_app(parsed.servers, SupportedApp::ClaudeCode)
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
            if server.enabled && server.apps.get(&SupportedApp::ClaudeCode).copied().unwrap_or(false)
            {
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
                .user_app_config_path(SupportedApp::ClaudeCode)
                .to_string_lossy()
                .to_string(),
            mode: "merge_json_field".to_string(),
            field: Some("mcpServers".to_string()),
            content: serde_json::to_string(&serde_json::Value::Object(servers))
                .expect("serialize claude"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClaudeCodeAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn ctx() -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: PathBuf::from("/Users/test"),
            workspace_root: PathBuf::from("/workspace/project"),
        }
    }

    #[test]
    fn parses_claude_user_source() {
        let parsed = ClaudeCodeAdapter.parse_source(
            &ctx(),
            "/Users/test/.claude.json",
            20,
            r#"{"mcpServers":{"linear":{"url":"https://mcp.linear.app/mcp"}},"projects":{"/workspace/project":{"mcpServers":{"playwright":{"command":"npx"}}}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 2);
    }

    #[test]
    fn plans_claude_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::ClaudeCode, true);
        let op = ClaudeCodeAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    id: "playwright".to_string(),
                    name: "Playwright".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "npx".to_string(),
                        args: vec!["@playwright/mcp@latest".to_string()],
                        env: HashMap::new(),
                    }),
                    apps,
                }],
            },
        );
        assert_eq!(op.mode, "merge_json_field");
        assert_eq!(op.field.as_deref(), Some("mcpServers"));
    }
}
