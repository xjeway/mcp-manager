use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_jsonc, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct AuggieAdapter;

impl AppAdapter for AuggieAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Auggie
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".augment/settings.local.json")
                    .to_string_lossy()
                    .to_string(),
                9,
            ),
            (
                ctx.workspace_file(".augment/settings.json")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.user_app_config_path(self.app())
                    .to_string_lossy()
                    .to_string(),
                20,
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
        // Older Auggie examples used `transport`; prefer the current `type`
        // when both exist. Keep the original source for the in-place writer.
        let normalized = parse_jsonc(content).ok().map(|mut root| {
            if let Some(servers) = root
                .get_mut("mcpServers")
                .and_then(serde_json::Value::as_object_mut)
            {
                for server in servers
                    .values_mut()
                    .filter_map(serde_json::Value::as_object_mut)
                {
                    if !server.contains_key("type") {
                        if let Some(transport) = server.get("transport").cloned() {
                            server.insert("type".into(), transport);
                        }
                    }
                }
            }
            root.to_string()
        });
        let parsed = parse_mcp_json(normalized.as_deref().unwrap_or(content));
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
    use super::*;
    use crate::core::{build_import_result, DetectedServer};
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::fs;
    use std::path::Path;

    fn ctx(root: &Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: root.join("home"),
            workspace_root: root.join("project"),
        }
    }

    #[test]
    fn imports_current_and_legacy_transport_fields() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::Auggie);
        let parsed = AuggieAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, r#"{
            // Valid Auggie JSONC, including a trailing comma.
            "mcpServers": {
                "local": {"command":"node","args":["server.js"],"env":{"TOKEN":"${TOKEN}"}},
                "http": {"type":"http","url":"https://example.com/mcp","headers":{"Authorization":"Bearer ${TOKEN}"}},
                "events": {"type":"sse","url":"https://example.com/events"},
                "old": {"transport":"sse","url":"https://example.com/legacy"},
                "both": {"type":"http","transport":"sse","url":"https://example.com/both"},
            },
        }"#);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert_eq!(parsed.servers.len(), 5);
        let find = |id: &str| &parsed.servers.iter().find(|(s, _)| s.id == id).unwrap().0;
        assert_eq!(
            find("local").command.as_ref().unwrap().env["TOKEN"],
            "${TOKEN}"
        );
        assert_eq!(find("http").transport.kind, "http");
        assert_eq!(
            find("http").transport.headers["Authorization"],
            "Bearer ${TOKEN}"
        );
        assert_eq!(find("events").transport.kind, "sse");
        assert_eq!(find("old").transport.kind, "sse");
        assert_eq!(find("both").transport.kind, "http");
        assert!(find("http").apps[&SupportedApp::Auggie]);
    }

    #[test]
    fn private_local_files_do_not_warn_about_committed_secrets() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        for (path, expected) in [
            (".augment/settings.local.json", 0),
            (".augment/settings.json", 1),
        ] {
            let path = ctx.workspace_file(path);
            let parsed = AuggieAdapter.parse_source(&ctx, &path.to_string_lossy(), 10,
                r#"{"mcpServers":{"api":{"type":"http","url":"https://example.com","headers":{"Authorization":"Bearer private"}}}}"#);
            let config = MCPConfig {
                version: 1,
                servers: parsed
                    .servers
                    .into_iter()
                    .map(|(server, _)| server)
                    .collect(),
            };
            assert_eq!(
                crate::adapters::shared_project_secret_warnings(&ctx, &config).len(),
                expected
            );
        }
    }

    #[test]
    fn user_path_is_home_relative_on_every_platform() {
        let temp = tempfile::tempdir().unwrap();
        for os in [PlatformOs::MacOS, PlatformOs::Linux, PlatformOs::Windows] {
            let mut ctx = ctx(temp.path());
            ctx.os = os;
            assert_eq!(
                ctx.user_app_config_path(SupportedApp::Auggie),
                ctx.home_dir.join(".augment/settings.json")
            );
        }
    }

    #[test]
    fn writes_cross_client_streamable_http_as_auggie_http() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::Auggie);
        let parsed = AuggieAdapter.parse_source(
            &ctx,
            &path.to_string_lossy(),
            20,
            r#"{"mcpServers":{"api":{"type":"streamable-http","url":"https://example.com/mcp"}}}"#,
        );
        let config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        let operations = AuggieAdapter.plan_apply(&ctx, &config, None);
        let written: serde_json::Value = serde_json::from_str(&operations[0].content).unwrap();
        assert_eq!(written["api"]["type"], "http");
    }

    #[test]
    fn round_trips_http_headers() {
        crate::adapters::header_fixture::assert_round_trip(
            &AuggieAdapter,
            include_str!("../../tests/fixtures/headers/auggie.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "theme",
            },
        );
    }

    #[test]
    fn updates_and_removes_only_managed_entries_in_place() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::Auggie);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = r#"{
  // Keep theme and comments exactly.
  "theme": "dark",
  "mcpServers": {
    "events": {"transport":"sse","url":"https://old/events","timeout":9000,"headers":{"Old":"value"}},
    "unmanaged": {"command":"untouched"},
  },
  "toolPermissions": {"shell": "ask"},
}"#;
        fs::write(&path, original).unwrap();
        let parsed = AuggieAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, original);
        let mut server = parsed
            .servers
            .into_iter()
            .find(|(s, _)| s.id == "events")
            .unwrap()
            .0;
        server.transport.url = Some("https://new/events".into());
        server.transport.headers = [("Authorization".into(), "Bearer updated".into())].into();
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        for _ in 0..2 {
            let operations = AuggieAdapter.plan_apply(&ctx, &config, None);
            assert_eq!(operations.len(), 1);
            apply_operation(&path, &operations[0]).unwrap();
            let text = fs::read_to_string(&path).unwrap();
            assert!(text.contains("// Keep theme and comments exactly."));
            assert!(text.contains("\"unmanaged\": {\"command\":\"untouched\"}"));
            let written = parse_jsonc(&text).unwrap();
            assert_eq!(written["theme"], "dark");
            assert_eq!(written["toolPermissions"]["shell"], "ask");
            let entry = &written["mcpServers"]["events"];
            assert_eq!(entry["type"], "sse");
            assert!(entry.get("transport").is_none());
            assert_eq!(entry["url"], "https://new/events");
            assert_eq!(entry["timeout"], 9000);
            assert_eq!(entry["headers"], json!({"Authorization":"Bearer updated"}));
        }
        let empty = MCPConfig {
            version: 1,
            servers: vec![],
        };
        for op in AuggieAdapter.plan_apply(&ctx, &empty, Some(&config)) {
            apply_operation(&path, &op).unwrap();
        }
        let written = parse_jsonc(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(written["mcpServers"].get("events").is_none());
        assert_eq!(written["mcpServers"]["unmanaged"]["command"], "untouched");
    }

    #[test]
    fn imports_local_before_project_before_user_and_writes_back_to_each_source() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let sources = AuggieAdapter.detect_sources(&ctx);
        assert!(sources[0].0.ends_with("settings.local.json"));
        assert!(sources[0].1 < sources[1].1 && sources[1].1 < sources[2].1);
        let mut detected = vec![];
        for (index, (path, priority)) in sources.iter().enumerate() {
            let content = json!({"mcpServers": {"same": {"type":"http", "url": format!("https://example.com/{index}")}}}).to_string();
            let parsed = AuggieAdapter.parse_source(&ctx, path, *priority, &content);
            detected.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
        let config = build_import_result(vec![], detected, vec![], vec![]).config;
        assert_eq!(
            config.servers[0].transport.url.as_deref(),
            Some("https://example.com/0")
        );
        assert_eq!(config.servers[0].placements.len(), 2);
        let ops = AuggieAdapter.plan_apply(&ctx, &config, None);
        assert_eq!(ops.len(), 3);
        for (path, _) in sources {
            assert!(ops.iter().any(|op| op.path == path));
        }
    }
}
