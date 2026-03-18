use crate::adapters::{opencode_mcp_servers, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, extract_opencode_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct OpenCodeAdapter;

impl AppAdapter for OpenCodeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::OpenCode
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (ctx.workspace_file("opencode.json").to_string_lossy().to_string(), 10),
            (
                ctx.workspace_file(".opencode/opencode.jsonc")
                    .to_string_lossy()
                    .to_string(),
                12,
            ),
            (
                ctx.user_app_config_path(self.app()).to_string_lossy().to_string(),
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
            servers: enable_servers_for_app(parsed.servers, self.app())
                .into_iter()
                .map(|server| (server, priority))
                .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    fn plan_apply(&self, ctx: &PlatformContext, config: &MCPConfig) -> WriteOperation {
        WriteOperation {
            path: ctx
                .user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            mode: "merge_json_field".to_string(),
            field: Some("mcp".to_string()),
            content: serde_json::to_string(&opencode_mcp_servers(config, self.app()))
                .expect("serialize opencode"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpenCodeAdapter;
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
    fn parses_opencode_config() {
        let parsed = OpenCodeAdapter.parse_source(
            &ctx(),
            "/workspace/project/opencode.json",
            10,
            r#"{"mcp":{"playwright":{"type":"local","command":["npx","@playwright/mcp@latest"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_opencode_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::OpenCode, true);
        let op = OpenCodeAdapter.plan_apply(
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
                    description: None,
                    homepage: None,
                }],
            },
        );
        assert_eq!(op.mode, "merge_json_field");
        assert_eq!(op.field.as_deref(), Some("mcp"));
    }
}
