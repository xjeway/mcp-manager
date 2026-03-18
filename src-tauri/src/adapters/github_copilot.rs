use crate::adapters::{standard_mcp_servers, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, extract_json_field_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct GithubCopilotAdapter;

impl AppAdapter for GithubCopilotAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::GithubCopilot
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (ctx.workspace_file(".vscode/mcp.json").to_string_lossy().to_string(), 10),
            (
                ctx.user_app_config_path(SupportedApp::Vscode)
                    .to_string_lossy()
                    .to_string(),
                15,
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
            field: Some("mcpServers".to_string()),
            content: serde_json::to_string(&standard_mcp_servers(config, self.app()))
                .expect("serialize github copilot"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GithubCopilotAdapter;
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
        let op = GithubCopilotAdapter.plan_apply(
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
                    description: None,
                    homepage: None,
                }],
            },
        );
        assert_eq!(op.mode, "merge_json_field");
        assert_eq!(op.field.as_deref(), Some("mcpServers"));
    }
}
