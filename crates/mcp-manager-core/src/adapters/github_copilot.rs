use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_json_field_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct GithubCopilotAdapter;

impl AppAdapter for GithubCopilotAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::GithubCopilot
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".vscode/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.user_app_config_path(SupportedApp::Vscode)
                    .to_string_lossy()
                    .to_string(),
                15,
            ),
            (
                ctx.user_app_config_path(self.app())
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
        let normalized = if path.ends_with("mcp-config.json") {
            content.to_string()
        } else {
            match extract_json_field_mcp_json(content, "servers") {
                Ok(value) => value,
                Err(error) => {
                    return ParsedSources {
                        sources: vec![LocalConfigSource {
                            app: self.app().as_str().to_string(),
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
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: self.app().as_str().to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(normalized),
            }],
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
        managed_json_field_writes(ctx, "mcpServers", self.app(), config, previous_config)
    }
}

#[cfg(test)]
mod tests {
    use super::GithubCopilotAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use serde_json::Value;
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
    fn detects_cli_and_vscode_sources() {
        let sources = GithubCopilotAdapter.detect_sources(&ctx());
        assert_eq!(sources.len(), 3);
        assert!(sources[0].0.ends_with(".vscode/mcp.json"));
        assert!(sources[2].0.ends_with(".copilot/mcp-config.json"));
    }

    #[test]
    fn plans_copilot_cli_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::GithubCopilot, true);
        let operations = GithubCopilotAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    id: "github".to_string(),
                    name: "GitHub".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "uvx".to_string(),
                        args: vec!["mcp-server-github".to_string()],
                        env: HashMap::new(),
                    }),
                    apps,
                    placements: vec![],
                    description: None,
                    homepage: None,
                }],
            },
            None,
        );
        assert_eq!(operations[0].mode, "merge_json_object_entries");
        assert_eq!(operations[0].field.as_deref(), Some("mcpServers"));
    }

    #[test]
    fn writes_copilot_transport_types_and_tools() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::GithubCopilot, true);

        let operations = GithubCopilotAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![
                    MCPServer {
                        id: "github".to_string(),
                        name: "GitHub".to_string(),
                        enabled: true,
                        transport: TransportSpec {
                            kind: "stdio".to_string(),
                            url: None,
                        },
                        command: Some(crate::core::CommandSpec {
                            program: "uvx".to_string(),
                            args: vec!["mcp-server-github".to_string()],
                            env: HashMap::new(),
                        }),
                        apps: apps.clone(),
                        placements: vec![],
                        description: None,
                        homepage: None,
                    },
                    MCPServer {
                        id: "linear".to_string(),
                        name: "Linear".to_string(),
                        enabled: true,
                        transport: TransportSpec {
                            kind: "sse".to_string(),
                            url: Some("https://mcp.linear.app/sse".to_string()),
                        },
                        command: None,
                        apps,
                        placements: vec![],
                        description: None,
                        homepage: None,
                    },
                ],
            },
            None,
        );

        let payload: Value = serde_json::from_str(&operations[0].content).expect("payload");
        assert_eq!(payload["github"]["type"], "stdio");
        assert_eq!(payload["github"]["tools"][0], "*");
        assert_eq!(payload["linear"]["type"], "sse");
        assert_eq!(payload["linear"]["tools"][0], "*");
    }
}
