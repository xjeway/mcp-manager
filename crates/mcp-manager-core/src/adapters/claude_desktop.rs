use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_json_field_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct ClaudeDesktopAdapter;

impl AppAdapter for ClaudeDesktopAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::ClaudeDesktop
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![(
            ctx.user_app_config_path(SupportedApp::ClaudeDesktop)
                .to_string_lossy()
                .to_string(),
            25,
        )]
    }

    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let normalized = match extract_json_field_mcp_json(content, "mcpServers") {
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
    use super::ClaudeDesktopAdapter;
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
    fn parses_claude_desktop_config() {
        let parsed = ClaudeDesktopAdapter.parse_source(
            &ctx(),
            "/Users/test/Library/Application Support/Claude/claude_desktop_config.json",
            25,
            r#"{"theme":"light","mcpServers":{"linear":{"url":"https://mcp.linear.app/mcp"}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_claude_desktop_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::ClaudeDesktop, true);
        let operations = ClaudeDesktopAdapter.plan_apply(
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
                        headers: Default::default(),
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "npx".to_string(),
                        args: vec!["@playwright/mcp@latest".to_string()],
                        env: HashMap::new(),
                        secret_env: Default::default(),
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
    fn does_not_write_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &ClaudeDesktopAdapter,
            include_str!("../../tests/fixtures/headers/claude_desktop.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: None,
                unrelated_key: "globalShortcut",
            },
        );
    }
}
