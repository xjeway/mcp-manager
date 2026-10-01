use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct ClineAdapter;

impl AppAdapter for ClineAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Cline
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        let mut sources = Vec::new();
        if let Ok(cline_dir) = std::env::var("CLINE_DIR") {
            sources.push((
                format!("{cline_dir}/data/settings/cline_mcp_settings.json"),
                18,
            ));
        }
        sources.push((
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        ));
        sources
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
                app: self.app().as_str().to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(content.to_string()),
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
    use super::ClineAdapter;
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
    fn parses_cline_settings() {
        let parsed = ClineAdapter.parse_source(
            &ctx(),
            "/Users/test/.cline/data/settings/cline_mcp_settings.json",
            20,
            r#"{"mcpServers":{"github":{"command":"uvx","args":["mcp-server-github"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_cline_replace_json() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Cline, true);
        let operations = ClineAdapter.plan_apply(
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
                    description: None,
                    homepage: None,
                }],
            },
            None,
        );
        assert_eq!(operations[0].mode, "merge_json_object_entries");
        assert!(operations[0].content.contains("github"));
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &ClineAdapter,
            include_str!("../../tests/fixtures/headers/cline.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "telemetry",
            },
        );
    }
}
