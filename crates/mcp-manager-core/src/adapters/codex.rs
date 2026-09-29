use crate::adapters::{managed_toml_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_codex_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct CodexAdapter;

impl AppAdapter for CodexAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Codex
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.home_dir
                    .join(".codex/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                30,
            ),
            (
                ctx.user_app_config_path(SupportedApp::Codex)
                    .to_string_lossy()
                    .to_string(),
                35,
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
        let normalized = if path.ends_with("config.toml") {
            match extract_codex_mcp_json(content) {
                Ok(value) => value,
                Err(error) => {
                    return ParsedSources {
                        sources: vec![LocalConfigSource {
                            app: "codex".to_string(),
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
                app: "codex".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(normalized),
            }],
            servers: enable_servers_for_app_at(
                parsed.servers,
                SupportedApp::Codex,
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
        managed_toml_field_writes(
            ctx,
            "mcp_servers",
            SupportedApp::Codex,
            config,
            previous_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CodexAdapter;
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
    fn parses_codex_toml_source() {
        let parsed = CodexAdapter.parse_source(
            &ctx(),
            "/Users/test/.codex/config.toml",
            35,
            r#"[mcp_servers.playwright]
command = "npx"
args = ["@playwright/mcp@latest"]
"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_codex_toml_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Codex, true);
        let operations = CodexAdapter.plan_apply(
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
                    }),
                    apps,
                    placements: vec![],
                }],
            },
            None,
        );
        assert_eq!(operations[0].mode, "merge_toml_table_entries");
        assert_eq!(operations[0].field.as_deref(), Some("mcp_servers"));
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &CodexAdapter,
            include_str!("../../tests/fixtures/headers/codex.toml"),
            crate::adapters::header_fixture::Expect {
                field: "mcp_servers",
                headers_key: Some("http_headers"),
                unrelated_key: "model",
            },
        );
    }
}
