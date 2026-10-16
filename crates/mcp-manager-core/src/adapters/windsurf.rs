use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct WindsurfAdapter;

impl AppAdapter for WindsurfAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Windsurf
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        // Once Devin has its own file it ignores the Windsurf-era ones, so a
        // server left there must not come back on import.
        if let Some(path) = ctx.devin_config_path() {
            return vec![(path.to_string_lossy().to_string(), 20)];
        }
        vec![
            (
                ctx.user_app_config_path(self.app())
                    .to_string_lossy()
                    .to_string(),
                20,
            ),
            (
                ctx.home_dir
                    .join(".codeium/mcp_config.json")
                    .to_string_lossy()
                    .to_string(),
                22,
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
    use super::WindsurfAdapter;
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
    fn detects_primary_and_legacy_paths() {
        let sources = WindsurfAdapter.detect_sources(&ctx());
        assert_eq!(sources.len(), 2);
        assert!(sources[0].0.ends_with(".codeium/windsurf/mcp_config.json"));
        assert!(sources[1].0.ends_with(".codeium/mcp_config.json"));
    }

    #[test]
    fn switches_to_devin_desktop_file_once_it_exists() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let legacy = home.path().join(crate::platform::WINDSURF_LEGACY_CONFIG);
        let devin = home.path().join(".config/devin/mcp_config.json");
        let sources = || {
            WindsurfAdapter
                .detect_sources(&ctx)
                .into_iter()
                .map(|(path, _)| PathBuf::from(path))
                .collect::<Vec<_>>()
        };
        assert_eq!(sources()[0], legacy);
        assert_eq!(ctx.user_app_config_path(SupportedApp::Windsurf), legacy);

        std::fs::create_dir_all(devin.parent().unwrap()).unwrap();
        std::fs::write(&devin, "{}").unwrap();
        assert_eq!(sources(), [devin.clone()]);
        assert_eq!(ctx.user_app_config_path(SupportedApp::Windsurf), devin);
    }

    #[test]
    fn plans_windsurf_replace_json() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Windsurf, true);
        let operations = WindsurfAdapter.plan_apply(
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
        assert!(operations[0].content.contains("playwright"));
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &WindsurfAdapter,
            include_str!("../../tests/fixtures/headers/windsurf.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "telemetry",
            },
        );
    }

    #[test]
    fn keeps_windsurf_only_switch() {
        crate::adapters::client_switch_fixture::assert_switch_kept(
            &WindsurfAdapter,
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
