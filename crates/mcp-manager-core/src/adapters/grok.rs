use crate::adapters::{managed_toml_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_codex_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

/// Grok Build (xAI). Servers are `[mcp_servers.<name>]` tables in
/// `config.toml`, the same layout as Codex, with remote headers in `headers`.
/// Grok also reads `~/.claude.json`, `.cursor/mcp.json` and `.mcp.json`; those
/// files belong to their own clients and are not imported again here.
pub struct GrokAdapter;

impl AppAdapter for GrokAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Grok
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            // A project server replaces a user server of the same name.
            (
                ctx.workspace_file(".grok/config.toml")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.user_app_config_path(self.app())
                    .to_string_lossy()
                    .to_string(),
                30,
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
        let source = |content: &str| LocalConfigSource {
            app: self.app().as_str().to_string(),
            path: path.to_string(),
            exists: true,
            format: "json".to_string(),
            priority,
            content: Some(content.to_string()),
        };
        let normalized = match extract_codex_mcp_json(content) {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![source(content)],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![error],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![source(&normalized)],
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
        managed_toml_field_writes(ctx, "mcp_servers", self.app(), config, previous_config)
    }
}

#[cfg(test)]
mod tests {
    use super::GrokAdapter;
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

    fn stdio_server(apps: HashMap<SupportedApp, bool>) -> MCPServer {
        MCPServer {
            description: None,
            homepage: None,
            id: "filesystem".to_string(),
            name: "filesystem".to_string(),
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
            }),
            apps,
            placements: vec![],
        }
    }

    #[test]
    fn detects_project_and_user_config() {
        let sources = GrokAdapter.detect_sources(&ctx());
        if std::env::var_os("GROK_HOME").is_none() {
            assert_eq!(sources[1].0, "/Users/test/.grok/config.toml");
        }
        assert_eq!(sources[0].0, "/workspace/project/.grok/config.toml");
        assert!(sources[0].1 < sources[1].1, "project wins over user");
    }

    #[test]
    fn parses_stdio_http_and_sse_servers() {
        let parsed = GrokAdapter.parse_source(
            &ctx(),
            "/Users/test/.grok/config.toml",
            30,
            r#"[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]
env = { ROOT = "/tmp" }

[mcp_servers.sentry]
url = "https://mcp.sentry.dev/mcp"
headers = { "Authorization" = "Bearer ${SENTRY_TOKEN}" }

[mcp_servers.linear]
url = "https://mcp.linear.app/sse"
type = "sse"
"#,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let server = |id: &str| {
            parsed
                .servers
                .iter()
                .map(|(server, _)| server)
                .find(|server| server.id == id)
                .unwrap_or_else(|| panic!("{id} imported"))
        };
        assert_eq!(server("filesystem").transport.kind, "stdio");
        assert_eq!(
            server("filesystem").command.as_ref().unwrap().env["ROOT"],
            "/tmp"
        );
        assert_eq!(server("sentry").transport.kind, "http");
        assert_eq!(
            server("sentry").transport.headers["Authorization"],
            "Bearer ${SENTRY_TOKEN}"
        );
        assert_eq!(server("linear").transport.kind, "sse");
        assert!(server("filesystem").apps[&SupportedApp::Grok]);
    }

    #[test]
    fn invalid_toml_is_reported() {
        let parsed = GrokAdapter.parse_source(
            &ctx(),
            "/Users/test/.grok/config.toml",
            30,
            "[mcp_servers.broken\n",
        );
        assert!(parsed.servers.is_empty());
        assert_eq!(parsed.errors.len(), 1);
    }

    #[test]
    fn project_servers_are_written_to_the_project_file() {
        let parsed = GrokAdapter.parse_source(
            &ctx(),
            "/workspace/project/.grok/config.toml",
            10,
            "[mcp_servers.filesystem]\ncommand = \"npx\"\n",
        );
        let (server, _) = &parsed.servers[0];
        assert!(
            !server.apps[&SupportedApp::Grok],
            "not a user-level install"
        );
        assert_eq!(server.placements[0].scope, PlacementScope::Workspace);

        let mut server = stdio_server(empty_apps());
        server.placements = vec![ServerPlacement {
            app: SupportedApp::Grok,
            scope: PlacementScope::Workspace,
            path: None,
            enabled: true,
            managed: true,
        }];
        let operations = GrokAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![server],
            },
            None,
        );
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].path, "/workspace/project/.grok/config.toml");
        assert_eq!(operations[0].mode, "merge_toml_table_entries");
        assert_eq!(operations[0].field.as_deref(), Some("mcp_servers"));
    }

    #[test]
    fn apply_keeps_grok_only_settings() {
        // The user file would be the real `$GROK_HOME/config.toml`.
        if std::env::var_os("GROK_HOME").is_some() {
            return;
        }
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let path = ctx.user_app_config_path(SupportedApp::Grok);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = r#"# my grok config
[mcp_servers.filesystem]
command = "node"
enabled = false # paused for now
startup_timeout_sec = 90
"#;
        std::fs::write(&path, original).unwrap();

        let mut apps = empty_apps();
        apps.insert(SupportedApp::Grok, true);
        let config = MCPConfig {
            version: 1,
            servers: vec![stdio_server(apps)],
        };
        for operation in GrokAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&PathBuf::from(&operation.path), &operation).expect("apply");
        }

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("# my grok config"), "{written}");
        let table: toml::Table = toml::from_str(&written).expect("valid toml");
        let entry = &table["mcp_servers"]["filesystem"];
        assert_eq!(entry["command"].as_str(), Some("npx"));
        assert_eq!(entry["enabled"].as_bool(), Some(false), "{written}");
        assert_eq!(entry["startup_timeout_sec"].as_integer(), Some(90));
    }

    #[test]
    fn round_trips_http_headers() {
        if std::env::var_os("GROK_HOME").is_some() {
            return;
        }
        crate::adapters::header_fixture::assert_round_trip(
            &GrokAdapter,
            include_str!("../../tests/fixtures/headers/grok.toml"),
            crate::adapters::header_fixture::Expect {
                field: "mcp_servers",
                headers_key: Some("headers"),
                unrelated_key: "memory",
            },
        );
    }
}
