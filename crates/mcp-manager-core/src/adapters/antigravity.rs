use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct AntigravityAdapter;

impl AppAdapter for AntigravityAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Antigravity
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        // Only the file Antigravity reads: once the 2.0 shared file exists, a
        // leftover 1.x file would bring back servers removed from the shared one.
        vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        )]
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
    use super::AntigravityAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use std::path::PathBuf;

    fn ctx() -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: PathBuf::from("/Users/test"),
            workspace_root: PathBuf::from("/workspace/project"),
        }
    }

    #[test]
    fn parses_antigravity_config() {
        let parsed = AntigravityAdapter.parse_source(
            &ctx(),
            "/Users/test/.gemini/antigravity/mcp_config.json",
            20,
            r#"{"mcpServers":{"chrome-devtools":{"command":"npx","args":["chrome-devtools-mcp@latest"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_antigravity_payload() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Antigravity, true);
        let operations = AntigravityAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    id: "linear".to_string(),
                    name: "Linear".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "http".to_string(),
                        url: Some("https://mcp.linear.app/mcp".to_string()),
                        headers: Default::default(),
                    },
                    command: None,
                    apps,
                    placements: vec![],
                    description: None,
                    homepage: None,
                }],
            },
            None,
        );
        assert_eq!(operations[0].mode, "merge_json_object_entries");
        assert!(operations[0].content.contains("linear"));
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &AntigravityAdapter,
            include_str!("../../tests/fixtures/headers/antigravity.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "telemetry",
            },
        );
    }

    #[test]
    fn reads_only_the_file_antigravity_uses() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let legacy = home.path().join(crate::platform::ANTIGRAVITY_LEGACY_CONFIG);
        let shared = home.path().join(crate::platform::ANTIGRAVITY_SHARED_CONFIG);
        let sources = || {
            AntigravityAdapter
                .detect_sources(&ctx)
                .into_iter()
                .map(|(path, _)| PathBuf::from(path))
                .collect::<Vec<_>>()
        };
        assert_eq!(sources(), [legacy]);

        std::fs::create_dir_all(shared.parent().unwrap()).unwrap();
        std::fs::write(&shared, "{}").unwrap();
        assert_eq!(sources(), [shared]);
    }

    #[test]
    fn keeps_antigravity_only_switch() {
        crate::adapters::client_switch_fixture::assert_switch_kept(
            &AntigravityAdapter,
            r#"{
  "mcpServers": {
    "off": { "command": "npx", "args": ["-y", "off-mcp"], "disabled": true },
    "on": { "command": "npx", "args": ["-y", "on-mcp"] }
  }
}"#,
            "mcpServers",
            "disabled",
            true,
        );
    }
}
