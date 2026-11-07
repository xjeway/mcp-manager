use crate::adapters::{managed_json_field_writes, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, PlacementScope, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_jsonc, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};
use std::fs;

/// Native ZCode MCP configuration; generic fallback files have their own writer.
pub struct ZCodeAdapter;

/// Reads the nested map without treating an invalid native config as empty.
fn native_servers(path: &str) -> Result<Map<String, Value>, String> {
    match fs::read_to_string(path) {
        Ok(content) => extract_native_servers(&content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(error) => Err(error.to_string()),
    }
}

/// Keeps missing settings distinct from an incompatible scalar/array section.
fn extract_native_servers(content: &str) -> Result<Map<String, Value>, String> {
    let root = parse_jsonc(content)?;
    if !root.is_object() {
        return Err("ZCode configuration must be an object".into());
    }
    let Some(mcp) = root.get("mcp") else {
        return Ok(Map::new());
    };
    let mcp = mcp
        .as_object()
        .ok_or_else(|| "mcp must be an object".to_string())?;
    match mcp.get("servers") {
        None => Ok(Map::new()),
        Some(Value::Object(servers)) => Ok(servers.clone()),
        _ => Err("mcp.servers must be an object".into()),
    }
}

/// Returns the shared fallback belonging to a native file's scope.
fn fallback_for(ctx: &PlatformContext, native: &str) -> String {
    if native
        == ctx
            .user_app_config_path(SupportedApp::ZCode)
            .to_string_lossy()
    {
        ctx.home_dir
            .join(".agents/mcp.json")
            .to_string_lossy()
            .into()
    } else {
        ctx.workspace_file(".agents/mcp.json")
            .to_string_lossy()
            .into()
    }
}

/// Creating a committed native file must not copy unreviewed fallback credentials.
/// User-level migration stays private; project migration requires prior import.
pub(crate) fn validate_project_fallback(
    ctx: &PlatformContext,
    operations: &[(SupportedApp, WriteOperation)],
) -> Result<(), String> {
    let Some(project) = crate::adapters::workspace_scope_path(ctx, SupportedApp::ZCode) else {
        return Ok(());
    };
    for (_, operation) in operations
        .iter()
        .filter(|(app, op)| *app == SupportedApp::ZCode && op.path == project)
    {
        let desired: Map<String, Value> =
            serde_json::from_str(&operation.content).map_err(|error| error.to_string())?;
        if desired.keys().any(|id| {
            !operation
                .remove_keys
                .as_ref()
                .is_some_and(|keys| keys.contains(id))
        }) {
            return Err(format!("ZCode project migration would copy unreviewed fallback servers. Import the detected configuration first, or add all fallback servers with --from, so shared-file secrets can be reviewed before writing {project}."));
        }
    }
    Ok(())
}

impl AppAdapter for ZCodeAdapter {
    /// Registry and serialized id.
    fn app(&self) -> SupportedApp {
        SupportedApp::ZCode
    }

    /// Native wins only when it defines servers, not merely when a file exists.
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        let user = ctx
            .user_app_config_path(self.app())
            .to_string_lossy()
            .to_string();
        // Native definitions outrank generic-reader imports of the same id.
        let mut scopes = vec![(user, 8)];
        if let Some(project) = crate::adapters::workspace_scope_path(ctx, self.app()) {
            scopes.push((project, 9));
        }
        let mut sources = Vec::new();
        for (native, priority) in scopes {
            if native_servers(&native).is_ok_and(|servers| servers.is_empty()) {
                let fallback = fallback_for(ctx, &native);
                if std::path::Path::new(&fallback).exists() {
                    sources.push((fallback, priority));
                }
            }
            sources.push((native, priority));
        }
        sources
    }

    /// Attributes fallback data to its future native write target, never to the shared file.
    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let user = ctx
            .user_app_config_path(self.app())
            .to_string_lossy()
            .to_string();
        let user_fallback = fallback_for(ctx, &user);
        let user_scope = path == user || path == user_fallback;
        let destination = if user_scope {
            user
        } else {
            ctx.workspace_file(".zcode/config.json")
                .to_string_lossy()
                .to_string()
        };
        let source = LocalConfigSource {
            app: self.app().as_str().into(),
            path: path.into(),
            exists: true,
            format: "json".into(),
            priority,
            content: Some(content.into()),
        };
        let normalized = if path == destination {
            match extract_native_servers(content) {
                Ok(servers) => serde_json::json!({"mcpServers": servers}).to_string(),
                Err(error) => {
                    return ParsedSources {
                        sources: vec![source],
                        servers: vec![],
                        warnings: vec![],
                        errors: vec![format!("{path}: {error}")],
                    }
                }
            }
        } else {
            content.into()
        };
        let parsed = parse_mcp_json(&normalized);
        let scope = if user_scope {
            PlacementScope::User
        } else {
            PlacementScope::Workspace
        };
        ParsedSources {
            sources: vec![source],
            servers: enable_servers_for_app_at(parsed.servers, self.app(), scope, destination)
                .into_iter()
                .map(|server| (server, priority))
                .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    /// Writes one nested map; when activating native settings, retain existing fallback data.
    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        let mut operations =
            managed_json_field_writes(ctx, "/mcp/servers", self.app(), config, previous);
        let user = ctx
            .user_app_config_path(self.app())
            .to_string_lossy()
            .to_string();
        let project = crate::adapters::workspace_scope_path(ctx, self.app());
        operations.retain(|op| op.path == user || project.as_deref() == Some(op.path.as_str()));
        for operation in &mut operations {
            operation.mode = "merge_json_object_path_entries".into();
            if !native_servers(&operation.path).is_ok_and(|servers| servers.is_empty()) {
                continue;
            }
            let fallback = fallback_for(ctx, &operation.path);
            let Ok(content) = fs::read_to_string(fallback) else {
                continue;
            };
            let Ok(root) = parse_jsonc(&content) else {
                continue;
            };
            let Some(fallback) = root.get("mcpServers").and_then(Value::as_object) else {
                continue;
            };
            let mut desired: Map<String, Value> =
                serde_json::from_str(&operation.content).expect("planned server map");
            for (id, old) in fallback {
                if let Some(new) = desired.get_mut(id) {
                    crate::storage::carry_unmanaged_json_fields(old, new);
                } else if !operation
                    .remove_keys
                    .as_ref()
                    .is_some_and(|keys| keys.contains(id))
                {
                    desired.insert(id.clone(), old.clone());
                }
            }
            operation.content = serde_json::to_string(&desired).expect("fallback migration map");
        }
        operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformOs;
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::path::Path;

    fn ctx(root: &Path) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: root.join("home"),
            workspace_root: root.join("project"),
        }
    }

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn imports_nested_user_and_workspace_maps_with_user_precedence() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let user = ctx.user_app_config_path(SupportedApp::ZCode);
        let project = ctx.workspace_file(".zcode/config.json");
        write(
            &user,
            r#"{"mcp":{"servers":{"same":{"command":"user"},"remote":{"type":"http","url":"https://example.com/mcp","headers":{"Authorization":"Bearer ${TOKEN}"}}}}}"#,
        );
        write(
            &project,
            r#"{"mcp":{"servers":{"same":{"type":"http","url":"https://project/mcp"},"events":{"type":"sse","url":"https://example.com/events"}}}}"#,
        );
        let imported = crate::workflow::import_detected(&ctx).unwrap();
        assert!(imported.errors.is_empty(), "{:?}", imported.errors);
        let same = imported
            .config
            .servers
            .iter()
            .find(|s| s.id == "same")
            .unwrap();
        assert_eq!(same.command.as_ref().unwrap().program, "user");
        assert_eq!(same.transport.kind, "stdio");
        assert!(same.transport.url.is_none());
        assert!(same.apps[&SupportedApp::ZCode]);
        assert_eq!(
            same.placements[0].path.as_deref(),
            Some(project.to_str().unwrap())
        );
        let events = imported
            .config
            .servers
            .iter()
            .find(|s| s.id == "events")
            .unwrap();
        assert_eq!(events.transport.kind, "sse");
        assert_eq!(
            ctx.user_app_config_path(SupportedApp::ZCode),
            ctx.home_dir.join(".zcode/cli/config.json")
        );
    }

    #[test]
    fn fallback_selection_depends_on_nonempty_native_definitions() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let user = ctx.user_app_config_path(SupportedApp::ZCode);
        let fallback = ctx.home_dir.join(".agents/mcp.json");
        write(
            &fallback,
            r#"{"mcpServers":{"fallback":{"command":"node"}}}"#,
        );
        for native in [r#"{"hooks":{}}"#, r#"{"mcp":{"servers":{}}}"#] {
            write(&user, native);
            assert!(ZCodeAdapter
                .detect_sources(&ctx)
                .iter()
                .any(|(path, _)| Path::new(path) == fallback.as_path()));
        }
        write(
            &user,
            r#"{"mcp":{"servers":{"native":{"command":"node","enable":false}}}}"#,
        );
        assert!(!ZCodeAdapter
            .detect_sources(&ctx)
            .iter()
            .any(|(path, _)| Path::new(path) == fallback.as_path()));
        write(&user, "{broken");
        assert!(!ZCodeAdapter
            .detect_sources(&ctx)
            .iter()
            .any(|(path, _)| Path::new(path) == fallback.as_path()));
    }

    #[test]
    fn edits_nested_servers_in_place_without_touching_sibling_settings() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::ZCode);
        let original = r#"{
  // Keep hooks exactly.
  "hooks": {"onSave": "custom"},
  "permissions": {"shell": "ask"},
  "plugins": {"enabled": ["example"]},
  "mcp.servers": {"literal-dot-key": true},
  "mcp": {
    "discovery": "manual",
    "servers": {
      "api": {
        // This service is intentionally disabled in ZCode.
        "type": "http", "url": "https://old/mcp", "enable": false,
        "headers": {"Old": "value"}, "timeoutMs": 9000, "oauth": {"enabled": true}
      },
      "unmanaged": {"command": "do-not-edit"}
    }
  }
}"#;
        write(&path, original);
        let parsed = ZCodeAdapter.parse_source(&ctx, &path.to_string_lossy(), 8, original);
        assert!(parsed.errors.is_empty());
        let mut api = parsed
            .servers
            .into_iter()
            .find(|(s, _)| s.id == "api")
            .unwrap()
            .0;
        api.transport.url = Some("https://new/mcp".into());
        api.transport.headers = [("Authorization".into(), "Bearer updated".into())].into();
        let config = MCPConfig {
            version: 1,
            servers: vec![api],
        };
        for _ in 0..2 {
            let ops = ZCodeAdapter.plan_apply(&ctx, &config, None);
            assert_eq!(ops[0].field.as_deref(), Some("/mcp/servers"));
            apply_operation(&path, &ops[0]).unwrap();
            let text = fs::read_to_string(&path).unwrap();
            assert!(text.contains("// Keep hooks exactly."));
            assert!(text.contains("// This service is intentionally disabled in ZCode."));
            assert!(text.contains("\"hooks\": {\"onSave\": \"custom\"}"));
            let root = parse_jsonc(&text).unwrap();
            assert_eq!(root["mcp"]["discovery"], "manual");
            assert_eq!(root["mcp.servers"]["literal-dot-key"], true);
            assert_eq!(root["plugins"]["enabled"], json!(["example"]));
            let api = &root["mcp"]["servers"]["api"];
            assert_eq!(api["enable"], false);
            assert_eq!(api["timeoutMs"], 9000);
            assert_eq!(api["oauth"]["enabled"], true);
            assert_eq!(api["headers"], json!({"Authorization":"Bearer updated"}));
        }
        let empty = MCPConfig {
            version: 1,
            servers: vec![],
        };
        for op in ZCodeAdapter.plan_apply(&ctx, &empty, Some(&config)) {
            apply_operation(&path, &op).unwrap();
        }
        let root = parse_jsonc(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(root["mcp"]["servers"].get("api").is_none());
        assert_eq!(
            root["mcp"]["servers"]["unmanaged"]["command"],
            "do-not-edit"
        );
    }

    #[test]
    fn first_native_write_carries_fallback_servers_without_editing_shared_file() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let native = ctx.workspace_file(".zcode/config.json");
        let fallback = ctx.workspace_file(".agents/mcp.json");
        let fallback_text = r#"{"mcpServers":{"keep":{"command":"node","enable":false,"timeoutMs":1234},"other":{"type":"http","url":"https://example.com/mcp"}}}"#;
        write(&fallback, fallback_text);
        write(
            &native,
            r#"{"hooks":{"save":"keep"},"mcp":{"extra":"keep"}}"#,
        );
        let parsed = ZCodeAdapter.parse_source(&ctx, &fallback.to_string_lossy(), 9, fallback_text);
        assert!(parsed
            .servers
            .iter()
            .all(|(s, _)| s.placements[0].path.as_deref() == native.to_str()));
        let mut keep = parsed
            .servers
            .into_iter()
            .find(|(s, _)| s.id == "keep")
            .unwrap()
            .0;
        keep.command.as_mut().unwrap().program = "updated".into();
        let config = MCPConfig {
            version: 1,
            servers: vec![keep],
        };
        let ops = ZCodeAdapter.plan_apply(&ctx, &config, None);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].path, native.to_string_lossy());
        apply_operation(&native, &ops[0]).unwrap();
        let root = parse_jsonc(&fs::read_to_string(&native).unwrap()).unwrap();
        assert_eq!(root["mcp"]["servers"]["keep"]["command"], "updated");
        assert_eq!(root["mcp"]["servers"]["keep"]["enable"], false);
        assert_eq!(root["mcp"]["servers"]["keep"]["timeoutMs"], 1234);
        assert_eq!(
            root["mcp"]["servers"]["other"]["url"],
            "https://example.com/mcp"
        );
        assert_eq!(root["hooks"]["save"], "keep");
        assert_eq!(fs::read_to_string(&fallback).unwrap(), fallback_text);
    }

    #[test]
    fn client_enable_switch_survives_a_transport_change() {
        let old = json!({"type":"http","url":"https://old/mcp","enable":false,"headers":{"Authorization":"old"}});
        let mut new = json!({"type":"stdio","command":"node"});
        crate::storage::carry_unmanaged_json_fields(&old, &mut new);
        assert_eq!(new["enable"], false);
        assert!(new.get("url").is_none());
        assert!(new.get("headers").is_none());
    }

    #[test]
    fn project_migration_requires_importing_fallbacks_before_secret_review() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let native = ctx.workspace_file(".zcode/config.json");
        let fallback = ctx.workspace_file(".agents/mcp.json");
        let text =
            r#"{"mcpServers":{"existing":{"command":"node","env":{"API_KEY":"literal-secret"}}}}"#;
        write(&fallback, text);
        let parsed = ZCodeAdapter.parse_source(
            &ctx,
            &native.to_string_lossy(),
            9,
            r#"{"mcp":{"servers":{"new":{"command":"node"}}}}"#,
        );
        let mut config = MCPConfig {
            version: 1,
            servers: parsed.servers.into_iter().map(|(s, _)| s).collect(),
        };
        assert!(crate::workflow::prepare(&ctx, &config, None)
            .err()
            .unwrap()
            .contains("unreviewed fallback"));
        assert!(crate::workflow::apply(&ctx, &config, None).is_err());
        assert!(!native.exists());
        assert_eq!(fs::read_to_string(&fallback).unwrap(), text);
        let imported = ZCodeAdapter.parse_source(&ctx, &fallback.to_string_lossy(), 9, text);
        config
            .servers
            .extend(imported.servers.into_iter().map(|(s, _)| s));
        assert!(crate::workflow::prepare(&ctx, &config, None).is_ok());
        assert_eq!(
            crate::adapters::shared_project_secret_warnings(&ctx, &config).len(),
            1
        );
    }

    #[test]
    fn incompatible_native_sections_are_not_replaced_or_treated_as_fallbacks() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::ZCode);
        let parsed = ZCodeAdapter.parse_source(
            &ctx,
            &path.to_string_lossy(),
            8,
            r#"{"mcp":{"servers":{"local":{"command":"node"}}}}"#,
        );
        let config = MCPConfig {
            version: 1,
            servers: parsed.servers.into_iter().map(|(s, _)| s).collect(),
        };
        for original in [r#"{"mcp":"keep"}"#, r#"{"mcp":{"servers":[]}}"#] {
            write(&path, original);
            assert!(!ZCodeAdapter
                .parse_source(&ctx, &path.to_string_lossy(), 8, original)
                .errors
                .is_empty());
            let ops = ZCodeAdapter.plan_apply(&ctx, &config, None);
            assert!(apply_operation(&path, &ops[0]).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
        }
    }

    #[test]
    fn creates_nested_maps_and_previews_the_real_nested_entries() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.user_app_config_path(SupportedApp::ZCode);
        let parsed = ZCodeAdapter.parse_source(
            &ctx,
            &path.to_string_lossy(),
            8,
            r#"{"mcp":{"servers":{"local":{"command":"node"}}}}"#,
        );
        let config = MCPConfig {
            version: 1,
            servers: parsed.servers.into_iter().map(|(s, _)| s).collect(),
        };
        let op = &ZCodeAdapter.plan_apply(&ctx, &config, None)[0];
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        apply_operation(&path, op).unwrap();
        let root: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(root["mcp"]["servers"]["local"]["command"], "node");
        let preview = crate::workflow::preview(&ctx, &config, Some(&config));
        assert!(preview
            .iter()
            .all(|change| change.action == crate::workflow::ChangeAction::Unchanged));
        let mut updated = config.clone();
        updated.servers[0].command.as_mut().unwrap().program = "changed".into();
        assert!(crate::workflow::preview(&ctx, &updated, Some(&config))
            .iter()
            .any(|change| change.action == crate::workflow::ChangeAction::Update));
    }
}
