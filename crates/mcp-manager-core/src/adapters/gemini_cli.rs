use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_json_field_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct GeminiCliAdapter;

impl AppAdapter for GeminiCliAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::GeminiCli
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".gemini/settings.json")
                    .to_string_lossy()
                    .to_string(),
                10,
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
    use super::GeminiCliAdapter;
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
    fn parses_gemini_settings() {
        let parsed = GeminiCliAdapter.parse_source(
            &ctx(),
            "/Users/test/.gemini/settings.json",
            20,
            r#"{"theme":"dark","mcpServers":{"filesystem":{"command":"npx","args":["-y","@modelcontextprotocol/server-filesystem"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_gemini_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::GeminiCli, true);
        let operations = GeminiCliAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    id: "filesystem".to_string(),
                    name: "Filesystem".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                        headers: Default::default(),
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "npx".to_string(),
                        args: vec![
                            "-y".to_string(),
                            "@modelcontextprotocol/server-filesystem".to_string(),
                        ],
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
        assert_eq!(operations[0].field.as_deref(), Some("mcpServers"));
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &GeminiCliAdapter,
            include_str!("../../tests/fixtures/headers/gemini_cli.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "theme",
            },
        );
    }
}
