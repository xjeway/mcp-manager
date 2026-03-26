use crate::adapters::{standard_mcp_servers, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, parse_mcp_json};
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
        _ctx: &PlatformContext,
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
            mode: "replace_json".to_string(),
            field: None,
            content: serde_json::to_string(&serde_json::json!({
                "mcpServers": standard_mcp_servers(config, self.app())
            }))
            .expect("serialize cline"),
        }
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
        let op = ClineAdapter.plan_apply(
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
        assert_eq!(op.mode, "replace_json");
        assert!(op.content.contains("\"mcpServers\""));
    }
}
