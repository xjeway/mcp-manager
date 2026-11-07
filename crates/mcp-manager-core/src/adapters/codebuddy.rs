use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;

/// User-scope CodeBuddy adapter; the shared project file has one Claude Code writer.
pub struct CodeBuddyAdapter;

impl AppAdapter for CodeBuddyAdapter {
    /// Identifies the client for registry dispatch and configuration serialization.
    fn app(&self) -> SupportedApp {
        SupportedApp::CodeBuddy
    }

    /// Reads only the first existing user file, matching CodeBuddy precedence.
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        // The shared project `.mcp.json` is handled by Claude Code, also for
        // CodeBuddy and Copilot. Never register a second writer for that file.
        vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        )]
    }

    /// Imports the selected JSONC file while leaving private project entries untouched.
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

    /// Plans an in-place merge into the currently active user file.
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
    use super::*;
    use crate::core::{PlacementScope, TransportSpec};
    use crate::parser::parse_jsonc;
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::{fs, path::Path};

    fn ctx(root: &Path, os: PlatformOs) -> PlatformContext {
        PlatformContext {
            os,
            home_dir: root.join("home"),
            workspace_root: root.join("project"),
        }
    }

    #[test]
    fn resolves_only_first_existing_file_on_every_platform() {
        for os in [PlatformOs::MacOS, PlatformOs::Linux, PlatformOs::Windows] {
            for mask in 0..8 {
                let temp = tempfile::tempdir().unwrap();
                let ctx = ctx(temp.path(), os);
                let paths = [
                    ".codebuddy/.mcp.json",
                    ".codebuddy/mcp.json",
                    ".codebuddy.json",
                ]
                .map(|relative| ctx.home_dir.join(relative));
                for (index, path) in paths.iter().enumerate() {
                    if mask & (1 << index) != 0 {
                        fs::create_dir_all(path.parent().unwrap()).unwrap();
                        fs::write(path, "{}").unwrap();
                    }
                }
                let selected = (0..3).find(|index| mask & (1 << index) != 0).unwrap_or(0);
                assert_eq!(
                    ctx.user_app_config_path(SupportedApp::CodeBuddy),
                    paths[selected]
                );
                let sources = CodeBuddyAdapter.detect_sources(&ctx);
                assert_eq!(sources.len(), 1);
                assert_eq!(sources[0].0, paths[selected].to_string_lossy());
            }
        }
    }

    #[test]
    fn preferred_empty_or_malformed_file_never_revives_legacy_servers() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path(), PlatformOs::MacOS);
        fs::create_dir_all(ctx.home_dir.join(".codebuddy")).unwrap();
        fs::write(
            ctx.home_dir.join(".codebuddy/mcp.json"),
            r#"{"mcpServers":{"stale":{"command":"old"}}}"#,
        )
        .unwrap();
        let preferred = ctx.home_dir.join(".codebuddy/.mcp.json");
        for (content, has_errors) in [("{}", false), ("{broken", true)] {
            fs::write(&preferred, content).unwrap();
            let imported = crate::workflow::import_detected(&ctx).unwrap();
            assert!(imported.config.servers.is_empty());
            assert_eq!(!imported.errors.is_empty(), has_errors);
            assert_eq!(
                CodeBuddyAdapter.detect_sources(&ctx)[0].0,
                preferred.to_string_lossy()
            );
        }
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &CodeBuddyAdapter,
            include_str!("../../tests/fixtures/headers/codebuddy.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "telemetry",
            },
        );
    }

    #[test]
    fn imports_stdio_http_sse_and_description() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path(), PlatformOs::MacOS);
        let path = ctx.user_app_config_path(SupportedApp::CodeBuddy);
        let parsed = CodeBuddyAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, r#"{
            // CodeBuddy supports comments and trailing commas.
            "mcpServers": {
                "local":{"type":"stdio","command":"node","args":["server.js"],"env":{"TOKEN":"${TOKEN}"},"description":"Local tool"},
                "http":{"type":"http","url":"https://example.com/mcp","headers":{"Authorization":"Bearer ${TOKEN}"}},
                "events":{"type":"sse","url":"https://example.com/events"},
            },
        }"#);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert_eq!(parsed.servers.len(), 3);
        let find = |id: &str| {
            &parsed
                .servers
                .iter()
                .find(|(server, _)| server.id == id)
                .unwrap()
                .0
        };
        assert_eq!(find("local").description.as_deref(), Some("Local tool"));
        assert_eq!(
            find("local").command.as_ref().unwrap().env["TOKEN"],
            "${TOKEN}"
        );
        assert_eq!(find("events").transport.kind, "sse");
        assert_eq!(
            find("http").transport.headers["Authorization"],
            "Bearer ${TOKEN}"
        );
        assert!(parsed.servers.iter().all(
            |(server, _)| server.apps[&SupportedApp::CodeBuddy] && server.placements.is_empty()
        ));
        assert!(crate::adapters::workspace_scope_path(&ctx, SupportedApp::CodeBuddy).is_none());
    }

    #[test]
    fn updates_active_legacy_file_in_place_and_preserves_private_and_tool_settings() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path(), PlatformOs::MacOS);
        fs::create_dir_all(&ctx.home_dir).unwrap();
        let path = ctx.home_dir.join(".codebuddy.json");
        let original = r#"{
  // Keep preferences and private project data.
  "theme": "dark",
  "projects": {"/private": {"mcpServers": {"private": {"command":"private-tool"}}}},
  "disabledMcpServers": ["api"],
  "mcpServers": {
    "api": {"type":"http","url":"https://old/mcp","description":"API tool","headers":{"Old":"value"},"defer_loading":true,"tools":{"search":{"defer_loading":false}}},
    "unmanaged": {"command":"untouched"},
  },
}"#;
        fs::write(&path, original).unwrap();
        let parsed = CodeBuddyAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, original);
        let mut server = parsed
            .servers
            .into_iter()
            .find(|(server, _)| server.id == "api")
            .unwrap()
            .0;
        server.transport = TransportSpec {
            kind: "streamable-http".into(),
            url: Some("https://new/mcp".into()),
            headers: [("Authorization".into(), "Bearer rotated".into())].into(),
        };
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        for _ in 0..2 {
            let ops = CodeBuddyAdapter.plan_apply(&ctx, &config, None);
            assert_eq!(ops.len(), 1);
            assert_eq!(ops[0].path, path.to_string_lossy());
            apply_operation(&path, &ops[0]).unwrap();
            let text = fs::read_to_string(&path).unwrap();
            assert!(text.contains("// Keep preferences and private project data."));
            assert!(text.contains("\"unmanaged\": {\"command\":\"untouched\"}"));
            let written = parse_jsonc(&text).unwrap();
            assert_eq!(written["theme"], "dark");
            assert_eq!(
                written["projects"]["/private"]["mcpServers"]["private"]["command"],
                "private-tool"
            );
            assert_eq!(written["disabledMcpServers"], json!(["api"]));
            let entry = &written["mcpServers"]["api"];
            assert_eq!(entry["type"], "http");
            assert_eq!(entry["url"], "https://new/mcp");
            assert_eq!(entry["headers"], json!({"Authorization":"Bearer rotated"}));
            assert_eq!(entry["tools"], json!({"search":{"defer_loading":false}}));
            assert_eq!(entry["defer_loading"], true);
            assert_eq!(entry["description"], "API tool");
            assert!(!ctx.home_dir.join(".codebuddy/.mcp.json").exists());
        }
        let empty = MCPConfig {
            version: 1,
            servers: vec![],
        };
        for op in CodeBuddyAdapter.plan_apply(&ctx, &empty, Some(&config)) {
            apply_operation(&path, &op).unwrap();
        }
        let written = parse_jsonc(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(written["mcpServers"].get("api").is_none());
        assert_eq!(written["mcpServers"]["unmanaged"]["command"], "untouched");
        assert!(written["projects"]["/private"]["mcpServers"]
            .get("private")
            .is_some());
    }

    #[test]
    fn command_updates_preserve_comments_inside_managed_server_objects() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path(), PlatformOs::MacOS);
        let path = ctx.user_app_config_path(SupportedApp::CodeBuddy);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = r#"{
  "mcpServers": {
    "local": {
      "command": "old",
      // This description explains the server.
      "description": "Keep me", /* inline explanation */
      "tools": {
        // This tool is deliberately deferred.
        "search": {"defer_loading": true}
      }
    }
  }
}"#;
        fs::write(&path, text).unwrap();
        let parsed = CodeBuddyAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, text);
        let mut server = parsed.servers.into_iter().next().unwrap().0;
        server.command.as_mut().unwrap().program = "new".into();
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        for op in CodeBuddyAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&path, &op).unwrap();
        }
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("// This description explains the server."));
        assert!(written.contains("/* inline explanation */"));
        assert!(written.contains("// This tool is deliberately deferred."));
        assert_eq!(
            parse_jsonc(&written).unwrap()["mcpServers"]["local"]["command"],
            "new"
        );
    }

    #[test]
    fn project_mcp_json_is_imported_and_written_only_by_claude_code() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path(), PlatformOs::MacOS);
        let path = ctx.workspace_file(".mcp.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"mcpServers":{"shared":{"command":"node"}}}"#).unwrap();
        let imported = crate::workflow::import_detected(&ctx).unwrap();
        assert!(imported.errors.is_empty(), "{:?}", imported.errors);
        let server = &imported.config.servers[0];
        assert_eq!(server.placements.len(), 1);
        assert_eq!(server.placements[0].app, SupportedApp::ClaudeCode);
        assert_eq!(server.placements[0].scope, PlacementScope::Workspace);
        let ops = crate::workflow::plan(&ctx, &imported.config, None);
        assert_eq!(
            ops.iter()
                .filter(|op| op.path == path.to_string_lossy())
                .count(),
            1
        );
        assert!(CodeBuddyAdapter
            .plan_apply(&ctx, &imported.config, None)
            .is_empty());
    }
}
