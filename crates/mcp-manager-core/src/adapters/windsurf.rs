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
        // Project files, read by the Devin CLI: the gitignored local file
        // overrides the shared one, and both override the user file.
        let mut sources = vec![
            (
                ctx.workspace_file(".devin/mcp_config.local.json")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.workspace_file(".devin/mcp_config.json")
                    .to_string_lossy()
                    .to_string(),
                11,
            ),
        ];
        // Once Devin has its own file it ignores the Windsurf-era ones, so a
        // server left there must not come back on import.
        if let Some(path) = ctx.devin_config_path() {
            sources.push((path.to_string_lossy().to_string(), 20));
            return sources;
        }
        sources.extend([
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
        ]);
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
    use super::WindsurfAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{
        empty_apps, MCPConfig, MCPServer, PlacementScope, ServerPlacement, SupportedApp,
        TransportSpec,
    };
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
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
    fn detects_project_primary_and_legacy_paths() {
        let sources = WindsurfAdapter.detect_sources(&ctx());
        let paths = sources
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            [
                "/workspace/project/.devin/mcp_config.local.json",
                "/workspace/project/.devin/mcp_config.json",
                "/Users/test/.codeium/windsurf/mcp_config.json",
                "/Users/test/.codeium/mcp_config.json",
            ]
        );
        assert!(sources[0].1 < sources[1].1 && sources[1].1 < sources[2].1);
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
        // The user-level files, after the project ones.
        let sources = || {
            WindsurfAdapter
                .detect_sources(&ctx)
                .into_iter()
                .skip(2)
                .map(|(path, _)| PathBuf::from(path))
                .collect::<Vec<_>>()
        };
        assert_eq!(sources()[0], legacy);
        assert_eq!(ctx.user_app_config_path(SupportedApp::Windsurf), legacy);

        std::fs::create_dir_all(devin.parent().unwrap()).unwrap();
        std::fs::write(&devin, "{}").unwrap();
        assert_eq!(sources(), std::slice::from_ref(&devin));
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
    fn project_and_local_servers_are_written_back_to_their_own_files() {
        let root = tempfile::tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: root.path().join("home"),
            workspace_root: root.path().join("project"),
        };
        let devin = ctx.workspace_root.join(".devin");
        std::fs::create_dir_all(&devin).unwrap();
        let shared = devin.join("mcp_config.json");
        let local = devin.join("mcp_config.local.json");
        std::fs::write(
            &shared,
            r#"{
  // committed with the repo
  "mcpServers": { "docs": { "serverUrl": "https://docs.example.com/mcp" } }
}"#,
        )
        .unwrap();
        std::fs::write(
            &local,
            r#"{ "mcpServers": { "db": { "command": "uvx", "args": ["db-mcp"] } } }"#,
        )
        .unwrap();

        let mut servers = Vec::new();
        for (path, priority) in WindsurfAdapter.detect_sources(&ctx) {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let parsed = WindsurfAdapter.parse_source(&ctx, &path, priority, &content);
            assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
            servers.extend(parsed.servers.into_iter().map(|(server, _)| server));
        }
        assert_eq!(servers.len(), 2);
        for server in &servers {
            assert!(
                !server.apps[&SupportedApp::Windsurf],
                "not a user-level install"
            );
            assert_eq!(server.placements[0].scope, PlacementScope::Workspace);
        }

        let mut config = MCPConfig {
            version: 1,
            servers,
        };
        for server in &mut config.servers {
            if let Some(command) = server.command.as_mut() {
                command.args.push("--readonly".to_string());
            }
        }
        let operations = WindsurfAdapter.plan_apply(&ctx, &config, None);
        let mut paths = operations
            .iter()
            .map(|operation| PathBuf::from(&operation.path))
            .collect::<Vec<_>>();
        paths.sort();
        assert_eq!(paths, [shared.clone(), local.clone()]);
        for operation in &operations {
            apply_operation(&PathBuf::from(&operation.path), operation).expect("apply");
        }

        let shared_text = std::fs::read_to_string(&shared).unwrap();
        assert!(
            shared_text.contains("// committed with the repo"),
            "{shared_text}"
        );
        assert!(shared_text.contains("serverUrl"), "{shared_text}");
        assert!(!shared_text.contains("db-mcp"), "{shared_text}");
        let local_text = std::fs::read_to_string(&local).unwrap();
        assert!(local_text.contains("--readonly"), "{local_text}");
        assert!(!local_text.contains("docs.example.com"), "{local_text}");
    }

    #[test]
    fn new_project_servers_go_to_the_shared_project_file() {
        let mut server = MCPServer {
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
            apps: empty_apps(),
            placements: vec![],
            description: None,
            homepage: None,
        };
        server.placements.push(ServerPlacement {
            app: SupportedApp::Windsurf,
            scope: PlacementScope::Workspace,
            path: None,
            enabled: true,
            managed: true,
        });
        let operations = WindsurfAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![server],
            },
            None,
        );
        assert_eq!(operations.len(), 1);
        assert_eq!(
            operations[0].path,
            "/workspace/project/.devin/mcp_config.json"
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
