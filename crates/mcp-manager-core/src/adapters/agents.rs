use crate::adapters::{managed_json_field_writes, AppAdapter, ParsedSources};
use crate::core::{
    LocalConfigSource, MCPConfig, PlacementScope, ServerPlacement, SupportedApp, WriteOperation,
};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;

/// One writer for the .agents file shared by Codebuff and Freebuff.
pub struct AgentsAdapter;

impl AppAdapter for AgentsAdapter {
    /// Shared-file identity, independent of the client reading it.
    fn app(&self) -> SupportedApp {
        SupportedApp::Agents
    }

    /// The upstream loader lets home override parent and parent override project.
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        let mut sources = vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            10,
        )];
        if ctx.has_workspace() {
            if let Some(parent) = ctx.workspace_root.parent() {
                sources.push((
                    parent
                        .join(".agents/mcp.json")
                        .to_string_lossy()
                        .to_string(),
                    15,
                ));
            }
            sources.push((
                ctx.workspace_file(".agents/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                20,
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        sources.retain(|(path, _)| seen.insert(path.clone()));
        sources
    }

    /// Imports raw references; resolving them here would expose host environment values.
    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let source = LocalConfigSource {
            app: self.app().as_str().into(),
            path: path.into(),
            exists: true,
            format: "json".into(),
            priority,
            content: Some(content.into()),
        };
        // Codebuff's loader calls JSON.parse, so do not accept invalid JSONC here.
        if let Err(error) = serde_json::from_str::<serde_json::Value>(content) {
            return ParsedSources {
                sources: vec![source],
                servers: vec![],
                warnings: vec![],
                errors: vec![format!(
                    "{path}: .agents MCP configuration requires strict JSON: {error}"
                )],
            };
        }
        let parsed = parse_mcp_json(content);
        let user = ctx
            .user_app_config_path(self.app())
            .to_string_lossy()
            .to_string();
        let project = ctx
            .workspace_file(".agents/mcp.json")
            .to_string_lossy()
            .to_string();
        let servers = if path == user {
            enable_servers_for_app_at(
                parsed.servers,
                self.app(),
                PlacementScope::User,
                path.into(),
            )
        } else {
            parsed
                .servers
                .into_iter()
                .map(|mut server| {
                    server.placements.push(ServerPlacement {
                        app: self.app(),
                        scope: PlacementScope::Workspace,
                        path: Some(path.into()),
                        enabled: true,
                        managed: path == project,
                    });
                    server
                })
                .collect()
        };
        ParsedSources {
            sources: vec![source],
            servers: servers
                .into_iter()
                .map(|server| (server, priority))
                .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    /// Never writes the imported parent file, even if a saved placement is tampered with.
    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        let user = ctx
            .user_app_config_path(self.app())
            .to_string_lossy()
            .to_string();
        let project = crate::adapters::workspace_scope_path(ctx, self.app());
        managed_json_field_writes(ctx, "mcpServers", self.app(), config, previous)
            .into_iter()
            .filter(|operation| {
                operation.path == user || project.as_deref() == Some(operation.path.as_str())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{build_import_result, DetectedServer};
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::{fs, path::Path};

    fn ctx(root: &Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: root.join("home"),
            workspace_root: root.join("monorepo/project"),
        }
    }

    #[test]
    fn preserves_agents_transport_when_other_clients_have_the_same_id() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx
            .user_app_config_path(SupportedApp::Agents)
            .to_string_lossy()
            .to_string();
        let parsed = AgentsAdapter.parse_source(
            &ctx,
            &path,
            10,
            r#"{"mcpServers":{"same":{"command":"global"}}}"#,
        );
        let current = &parsed.servers[0].0;
        let incoming = parse_mcp_json(
            r#"{"mcpServers":{"same":{"url":"https://other.example/mcp","type":"http"}}}"#,
        )
        .servers
        .remove(0);
        let merged = crate::core::merge_servers(Some(current), &incoming);
        assert_eq!(merged.transport.kind, "stdio");
        assert!(merged.transport.url.is_none());
        assert_eq!(merged.command.unwrap().program, "global");
        let merged = crate::core::merge_servers(Some(&incoming), current);
        assert_eq!(merged.transport.kind, "stdio");
        assert!(merged.transport.url.is_none());
        assert_eq!(merged.command.unwrap().program, "global");
    }

    #[test]
    fn inherited_removal_is_rejected_in_core_apply_for_the_active_workspace_only() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let parent = ctx
            .workspace_root
            .parent()
            .unwrap()
            .join(".agents/mcp.json");
        let parsed = AgentsAdapter.parse_source(
            &ctx,
            parent.to_str().unwrap(),
            15,
            r#"{"mcpServers":{"parent":{"command":"keep"}}}"#,
        );
        let previous = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        let removed = MCPConfig {
            version: 1,
            servers: vec![],
        };
        assert!(crate::workflow::prepare(&ctx, &removed, Some(&previous)).is_err());
        assert!(crate::workflow::apply(&ctx, &removed, Some(&previous)).is_err());
        let other = PlatformContext {
            workspace_root: temp.path().join("other/project"),
            ..ctx.clone()
        };
        assert!(
            crate::workflow::validate_inherited_agents(&other, &removed, Some(&previous)).is_ok()
        );
        crate::store::update(&ctx, |config| {
            *config = previous.clone();
            Ok(())
        })
        .unwrap();
        assert!(crate::workflow::validate_inherited_agents(&ctx, &removed, None).is_err());
    }

    #[test]
    fn detects_parent_only_installation_without_a_cli() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let parent = ctx
            .workspace_root
            .parent()
            .unwrap()
            .join(".agents/mcp.json");
        fs::create_dir_all(parent.parent().unwrap()).unwrap();
        fs::write(parent, "{}").unwrap();
        assert!(ctx.detect_installed_apps().contains(&SupportedApp::Agents));
    }

    #[test]
    fn imports_full_definitions_in_global_parent_project_order() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let sources = AgentsAdapter.detect_sources(&ctx);
        assert_eq!(sources.len(), 3);
        let mut detected = vec![];
        for (index, (path, priority)) in sources.iter().enumerate() {
            let config = match index {
                0 => json!({"mcpServers":{"same":{"command":"global","env":{"TOKEN":"$TOKEN"}}}}),
                1 => {
                    json!({"mcpServers":{"same":{"type":"http","url":"https://parent/mcp"},"parentOnly":{"command":"parent"}}})
                }
                _ => {
                    json!({"mcpServers":{"same":{"type":"http","url":"https://project/mcp"},"projectOnly":{"command":"project"}}})
                }
            };
            let parsed = AgentsAdapter.parse_source(&ctx, path, *priority, &config.to_string());
            assert!(parsed.errors.is_empty());
            detected.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
        let config = build_import_result(vec![], detected, vec![], vec![]).config;
        let same = config
            .servers
            .iter()
            .find(|server| server.id == "same")
            .unwrap();
        assert_eq!(same.transport.kind, "stdio");
        assert!(same.transport.url.is_none());
        assert_eq!(same.command.as_ref().unwrap().program, "global");
        assert_eq!(same.command.as_ref().unwrap().env["TOKEN"], "$TOKEN");
        assert!(same.apps[&SupportedApp::Agents]);
        assert_eq!(same.placements.len(), 2);
        let parent = config
            .servers
            .iter()
            .find(|server| server.id == "parentOnly")
            .unwrap();
        assert!(!parent.placements[0].managed);
        let operations = AgentsAdapter.plan_apply(&ctx, &config, None);
        assert_eq!(operations.len(), 2);
        assert!(operations
            .iter()
            .all(|operation| operation.path != sources[1].0));
    }

    #[test]
    fn parent_overrides_project_and_remote_headers_do_not_fall_through() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let sources = AgentsAdapter.detect_sources(&ctx);
        let mut detected = vec![];
        for (path, priority) in &sources[1..] {
            let headers = if *priority == 15 {
                json!({})
            } else {
                json!({"Authorization":"project-secret"})
            };
            let content = json!({"mcpServers":{"api":{"type":"http","url":"https://example.com/mcp","headers":headers}}}).to_string();
            let parsed = AgentsAdapter.parse_source(&ctx, path, *priority, &content);
            detected.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
        let config = build_import_result(vec![], detected, vec![], vec![]).config;
        assert!(config.servers[0].transport.headers.is_empty());
        let operations = AgentsAdapter.plan_apply(&ctx, &config, None);
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].path, sources[2].0);
        // Even a saved/tampered placement must never turn the parent into a write target.
        let mut tampered = config.clone();
        tampered.servers[0].placements[0].managed = true;
        assert!(AgentsAdapter
            .plan_apply(&ctx, &tampered, None)
            .iter()
            .all(|op| op.path != sources[1].0));
    }

    #[test]
    fn round_trips_remote_headers_and_query_parameters() {
        crate::adapters::header_fixture::assert_round_trip(
            &AgentsAdapter,
            include_str!("../../tests/fixtures/headers/agents.json"),
            crate::adapters::header_fixture::Expect {
                field: "mcpServers",
                headers_key: Some("headers"),
                unrelated_key: "metadata",
            },
        );
    }

    #[test]
    fn updates_only_managed_entries_and_preserves_params() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::Agents);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = include_str!("../../tests/fixtures/headers/agents.json");
        fs::write(&path, text).unwrap();
        let parsed = AgentsAdapter.parse_source(&ctx, &path.to_string_lossy(), 10, text);
        let mut server = parsed
            .servers
            .into_iter()
            .find(|(server, _)| server.id == "remote")
            .unwrap()
            .0;
        server.transport.url = Some("https://new.example.com/events".into());
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        for op in AgentsAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&path, &op).unwrap();
        }
        let written: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            written["mcpServers"]["remote"]["params"],
            json!({"workspace":"demo"})
        );
        assert_eq!(written["mcpServers"]["remote"]["type"], "sse");
        assert_eq!(written["mcpServers"]["other"]["env"]["TOKEN"], "$TOKEN");
        let empty = MCPConfig {
            version: 1,
            servers: vec![],
        };
        for op in AgentsAdapter.plan_apply(&ctx, &empty, Some(&config)) {
            apply_operation(&path, &op).unwrap();
        }
        let written: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert!(written["mcpServers"].get("remote").is_none());
        assert!(written["mcpServers"].get("other").is_some());
    }

    #[test]
    fn rejects_jsonc_and_deduplicates_home_parent_paths() {
        let temp = tempfile::tempdir().unwrap();
        let mut ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::Agents);
        for text in ["{//comment\n}", "{\"mcpServers\":{},}"] {
            assert!(!AgentsAdapter
                .parse_source(&ctx, &path.to_string_lossy(), 10, text)
                .errors
                .is_empty());
        }
        ctx.workspace_root = ctx.home_dir.join("project");
        assert_eq!(AgentsAdapter.detect_sources(&ctx).len(), 2);
        ctx.workspace_root = ctx.home_dir.clone();
        assert_eq!(AgentsAdapter.detect_sources(&ctx).len(), 1);
    }

    #[test]
    fn warns_for_written_secrets_but_not_dollar_references_or_read_only_parents() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let sources = AgentsAdapter.detect_sources(&ctx);
        for (path, priority) in &sources {
            for value in ["literal-secret", "$TOKEN", "Bearer $TOKEN"] {
                let content = json!({"mcpServers":{"api":{"type":"http","url":"https://example.com/mcp","headers":{"Authorization":value}}}}).to_string();
                let parsed = AgentsAdapter.parse_source(&ctx, path, *priority, &content);
                let config = MCPConfig {
                    version: 1,
                    servers: parsed
                        .servers
                        .into_iter()
                        .map(|(server, _)| server)
                        .collect(),
                };
                let warnings = crate::adapters::shared_project_secret_warnings(&ctx, &config);
                assert_eq!(
                    warnings.len(),
                    usize::from(*priority == 20 && value == "literal-secret")
                );
            }
        }
    }
}
