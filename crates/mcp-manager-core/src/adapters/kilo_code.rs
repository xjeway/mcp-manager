use crate::adapters::{
    managed_json_field_writes_with, opencode_mcp_servers_at, scope_for_path, AppAdapter,
    ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_opencode_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct KiloCodeAdapter;

impl AppAdapter for KiloCodeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::KiloCode
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file("kilo.json").to_string_lossy().to_string(),
                10,
            ),
            (
                ctx.workspace_file("kilo.jsonc").to_string_lossy().to_string(),
                11,
            ),
            (
                ctx.workspace_file(".kilo/kilo.json").to_string_lossy().to_string(),
                12,
            ),
            (
                ctx.workspace_file(".kilo/kilo.jsonc").to_string_lossy().to_string(),
                13,
            ),
            (
                ctx.user_app_config_path(self.app()).to_string_lossy().to_string(),
                20,
            ),
            (
                ctx.home_dir.join(".config/kilo/kilo.jsonc").to_string_lossy().to_string(),
                21,
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
        let normalized = match extract_opencode_mcp_json(content) {
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
        managed_json_field_writes_with(
            ctx,
            "mcp",
            self.app(),
            config,
            previous_config,
            opencode_mcp_servers_at,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::KiloCodeAdapter;
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
    fn parses_kilo_code_config() {
        let parsed = KiloCodeAdapter.parse_source(
            &ctx(),
            "/workspace/project/kilo.json",
            10,
            r#"{"mcp":{"playwright":{"type":"local","command":["npx","@playwright/mcp@latest"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_kilo_code_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::KiloCode, true);
        let operations = KiloCodeAdapter.plan_apply(
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
        assert_eq!(operations[0].field.as_deref(), Some("mcp"));
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &KiloCodeAdapter,
            include_str!("../../tests/fixtures/headers/kilo_code.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcp",
                headers_key: Some("headers"),
                unrelated_key: "theme",
            },
        );
    }
}
