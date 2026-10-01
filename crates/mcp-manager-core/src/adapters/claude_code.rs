use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_claude_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct ClaudeCodeAdapter;

impl AppAdapter for ClaudeCodeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::ClaudeCode
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".mcp.json")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
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
            servers: enable_servers_for_app_at(
                parsed.servers,
                SupportedApp::ClaudeCode,
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
        managed_json_field_writes(
            ctx,
            "mcpServers",
            SupportedApp::ClaudeCode,
            config,
            previous_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClaudeCodeAdapter;
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
    fn plans_claude_safe_merge_and_tracks_managed_ids() {
        let mut previous_apps = empty_apps();
        previous_apps.insert(SupportedApp::ClaudeCode, true);

        let mut current_apps = empty_apps();
        current_apps.insert(SupportedApp::ClaudeCode, true);

        let operations = ClaudeCodeAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    description: None,
                    homepage: None,
                    id: "playwright".to_string(),
                    name: "Playwright".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                        headers: Default::default(),
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "npx".to_string(),
                        args: vec!["@playwright/mcp@latest".to_string()],
                        env: HashMap::new(),
                        secret_env: Default::default(),
                    }),
                    apps: current_apps,
                    placements: vec![],
                }],
            },
            Some(&MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    description: None,
                    homepage: None,
                    id: "legacy".to_string(),
                    name: "Legacy".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                        headers: Default::default(),
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "uvx".to_string(),
                        args: vec!["legacy-server".to_string()],
                        env: HashMap::new(),
                        secret_env: Default::default(),
                    }),
                    apps: previous_apps,
                    placements: vec![],
                }],
            }),
        );
        assert_eq!(operations[0].mode, "merge_json_object_entries");
        assert_eq!(operations[0].field.as_deref(), Some("mcpServers"));
        assert_eq!(
            operations[0].remove_keys.as_deref(),
            Some(&["legacy".to_string(), "playwright".to_string()][..])
        );
    }

    #[test]
    fn writes_transport_types_for_claude_code_servers() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::ClaudeCode, true);

        let operations = ClaudeCodeAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![
                    MCPServer {
                        description: None,
                        homepage: None,
                        id: "playwright".to_string(),
                        name: "Playwright".to_string(),
                        enabled: true,
                        transport: TransportSpec {
                            kind: "stdio".to_string(),
                            url: None,
                            headers: Default::default(),
                        },
                        command: Some(crate::core::CommandSpec {
                            program: "npx".to_string(),
                            args: vec!["@playwright/mcp@latest".to_string()],
                            env: HashMap::new(),
                            secret_env: Default::default(),
                        }),
                        apps: apps.clone(),
                        placements: vec![],
                    },
                    MCPServer {
                        description: None,
                        homepage: None,
                        id: "linear".to_string(),
                        name: "Linear".to_string(),
                        enabled: true,
                        transport: TransportSpec {
                            kind: "sse".to_string(),
                            url: Some("https://mcp.linear.app/sse".to_string()),
                            headers: Default::default(),
                        },
                        command: None,
                        apps,
                        placements: vec![],
                    },
                ],
            },
            None,
        );

        let payload: Value = serde_json::from_str(&operations[0].content).expect("payload");
        assert_eq!(payload["playwright"]["type"], "stdio");
        assert_eq!(payload["linear"]["type"], "sse");
        assert_eq!(payload["linear"]["url"], "https://mcp.linear.app/sse");
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &ClaudeCodeAdapter,
            include_str!("../../tests/fixtures/headers/claude_code.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "numStartups",
            },
        );
    }
}
