use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct VSCodeAdapter;

impl AppAdapter for VSCodeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Vscode
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
        let parsed = parse_mcp_json(content);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: "vscode".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(content.to_string()),
            }],
            servers: enable_servers_for_app_at(
                parsed.servers,
                SupportedApp::Vscode,
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
            "servers",
            SupportedApp::Vscode,
            config,
            previous_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::VSCodeAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::test_paths::abs;
    use crate::platform::{PlatformContext, PlatformOs};
    use std::collections::HashMap;

    fn ctx() -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: abs("/Users/test"),
            workspace_root: abs("/workspace/project"),
        }
    }

    #[test]
    fn detects_workspace_and_user_sources() {
        let sources = VSCodeAdapter.detect_sources(&ctx());
        assert_eq!(sources.len(), 2);
        assert!(sources[0].0.ends_with(".vscode/mcp.json"));
        assert!(sources[1].0.ends_with("Code/User/mcp.json"));
    }

    #[test]
    fn plans_vscode_servers_payload() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Vscode, true);
        let operations = VSCodeAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    description: None,
                    homepage: None,
                    id: "github".to_string(),
                    name: "GitHub".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                        headers: Default::default(),
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "uvx".to_string(),
                        args: vec!["mcp-server-github".to_string()],
                        env: HashMap::new(),
                        secret_env: Default::default(),
                    }),
                    apps,
                    placements: vec![],
                }],
            },
            None,
        );
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].mode, "merge_json_object_entries");
        assert!(operations[0].content.contains("github"));
    }

    #[test]
    fn plans_vscode_user_and_workspace_payloads_from_placements() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Vscode, true);
        let operations = VSCodeAdapter.plan_apply(
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
                    apps,
                    placements: vec![
                        crate::core::ServerPlacement {
                            app: SupportedApp::Vscode,
                            scope: crate::core::PlacementScope::Workspace,
                            path: Some(
                                ctx()
                                    .workspace_file(".vscode/mcp.json")
                                    .to_string_lossy()
                                    .to_string(),
                            ),
                            enabled: true,
                            managed: true,
                        },
                        crate::core::ServerPlacement {
                            app: SupportedApp::Vscode,
                            scope: crate::core::PlacementScope::User,
                            path: Some(
                                ctx()
                                    .user_app_config_path(SupportedApp::Vscode)
                                    .to_string_lossy()
                                    .to_string(),
                            ),
                            enabled: true,
                            managed: true,
                        },
                    ],
                }],
            },
            None,
        );

        assert_eq!(operations.len(), 2);
        assert!(operations
            .iter()
            .any(|op| op.path.ends_with(".vscode/mcp.json") && op.content.contains("playwright")));
        assert!(
            operations
                .iter()
                .any(|op| op.path.ends_with("Code/User/mcp.json")
                    && op.content.contains("playwright"))
        );
    }

    #[test]
    fn plans_workspace_cleanup_when_managed_placement_is_disabled() {
        let current_apps = empty_apps();
        let previous_apps = empty_apps();

        let current = MCPConfig {
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
                placements: vec![crate::core::ServerPlacement {
                    app: SupportedApp::Vscode,
                    scope: crate::core::PlacementScope::Workspace,
                    path: Some("/workspace/project/.vscode/mcp.json".to_string()),
                    enabled: false,
                    managed: true,
                }],
            }],
        };

        let previous = MCPConfig {
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
                apps: previous_apps,
                placements: vec![crate::core::ServerPlacement {
                    app: SupportedApp::Vscode,
                    scope: crate::core::PlacementScope::Workspace,
                    path: Some("/workspace/project/.vscode/mcp.json".to_string()),
                    enabled: true,
                    managed: true,
                }],
            }],
        };

        let operations = VSCodeAdapter.plan_apply(&ctx(), &current, Some(&previous));
        let workspace_operation = operations
            .iter()
            .find(|operation| operation.path == "/workspace/project/.vscode/mcp.json")
            .expect("workspace cleanup operation");

        assert_eq!(workspace_operation.content, "{}");
        assert_eq!(
            workspace_operation.remove_keys.as_deref(),
            Some(&["playwright".to_string()][..])
        );
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &VSCodeAdapter,
            include_str!("../../tests/fixtures/headers/vscode.json"),
            crate::adapters::header_fixture::Expect {
                field: "servers",
                headers_key: Some("headers"),
                unrelated_key: "inputs",
            },
        );
    }
}
