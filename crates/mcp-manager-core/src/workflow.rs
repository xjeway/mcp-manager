//! End-to-end operations over every client: import what is installed, plan and
//! preview a new config, and apply it. The app's commands and the CLI both call
//! these, so they behave the same.

use crate::adapters::{adapters, scope_for_path, workspace_scope_path};
use crate::core::{
    build_import_result, ApplyResult, DetectedServer, ImportResult, LocalConfigSource, MCPConfig,
    PlacementScope, SupportedApp, WriteOperation,
};
use crate::parser::parse_yaml_config;
use crate::platform::PlatformContext;
use crate::storage::apply_operations;
use crate::store::config_path;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs;

/// Reads the unified `servers.yaml` and every client's config files.
pub fn import_detected(ctx: &PlatformContext) -> Result<ImportResult, String> {
    let mut sources = Vec::new();
    let mut detected_servers = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let yaml_path = config_path(ctx);
    if yaml_path.exists() {
        let content = fs::read_to_string(&yaml_path).map_err(|e| e.to_string())?;
        match parse_yaml_config(&content) {
            Ok(config) => {
                sources.push(LocalConfigSource {
                    app: "yaml".to_string(),
                    path: yaml_path.to_string_lossy().to_string(),
                    exists: true,
                    format: "yaml".to_string(),
                    priority: 0,
                    content: Some(content),
                });
                for server in config.servers {
                    detected_servers.push(DetectedServer {
                        server,
                        priority: 0,
                    });
                }
            }
            Err(error) => errors.push(format!("{}: {}", yaml_path.to_string_lossy(), error)),
        }
    } else {
        sources.push(LocalConfigSource {
            app: "yaml".to_string(),
            path: yaml_path.to_string_lossy().to_string(),
            exists: false,
            format: "yaml".to_string(),
            priority: 0,
            content: None,
        });
    }

    for adapter in adapters() {
        for (path, priority) in adapter.detect_sources(ctx) {
            let resolved = ctx.resolve_path(&path);
            if !resolved.exists() {
                sources.push(LocalConfigSource {
                    app: adapter.app().as_str().to_string(),
                    path: resolved.to_string_lossy().to_string(),
                    exists: false,
                    format: "json".to_string(),
                    priority,
                    content: None,
                });
                continue;
            }

            let content = fs::read_to_string(&resolved).map_err(|e| e.to_string())?;
            let parsed = adapter.parse_source(ctx, &resolved.to_string_lossy(), priority, &content);
            sources.extend(parsed.sources);
            warnings.extend(parsed.warnings);
            errors.extend(parsed.errors);
            detected_servers.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
    }

    Ok(build_import_result(
        sources,
        detected_servers,
        warnings,
        errors,
    ))
}

/// The file writes that turn `previous_config` into `config` for every client.
pub fn plan(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<WriteOperation> {
    plan_by_app(ctx, config, previous_config)
        .into_iter()
        .map(|(_, operation)| operation)
        .collect()
}

fn plan_by_app(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<(SupportedApp, WriteOperation)> {
    adapters()
        .into_iter()
        .flat_map(|adapter| {
            let app = adapter.app();
            adapter
                .plan_apply(ctx, config, previous_config)
                .into_iter()
                .map(move |operation| (app, operation))
        })
        .collect()
}

/// Writes `config` to every client, backing up each file first.
pub fn apply(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Result<ApplyResult, String> {
    let operations = plan(ctx, config, previous_config);

    for operation in &operations {
        let path = ctx.resolve_path(&operation.path);
        if !ctx.can_write(&path) {
            let error = format!("permission denied for {}", path.to_string_lossy());
            if crate::security::is_high_risk_condition(&error) {
                return Err(error);
            }
        }
    }

    let backups = apply_operations(operations)?;
    Ok(ApplyResult { backups })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeAction {
    Add,
    Update,
    Remove,
    Unchanged,
    /// The file already has an entry with this id that the previous config
    /// did not write, e.g. one the user added by hand.
    OverwriteUnmanaged,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedChange {
    pub server_id: String,
    pub app: SupportedApp,
    pub scope: PlacementScope,
    pub path: String,
    pub action: ChangeAction,
}

/// What [`apply`] would do to each server in each client file, without writing.
pub fn preview(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<PlannedChange> {
    let previously_written = previous_config
        .map(|previous| plan_by_app(ctx, previous, None))
        .unwrap_or_default();

    let mut changes = Vec::new();
    for (app, operation) in plan_by_app(ctx, config, previous_config) {
        let Some(field) = operation.field.as_deref() else {
            continue;
        };
        if !is_entry_merge(&operation.mode) {
            continue;
        }

        let desired = object_from_json(&operation.content);
        let existing = existing_entries(ctx, &operation, field);
        let managed_before = previously_written
            .iter()
            .filter(|(previous_app, previous)| {
                *previous_app == app
                    && previous.path == operation.path
                    && previous.field == operation.field
            })
            .flat_map(|(_, previous)| {
                object_from_json(&previous.content)
                    .into_iter()
                    .map(|(id, _)| id)
            })
            .collect::<BTreeSet<_>>();

        let scope = scope_for_path(ctx, &operation.path);
        let mut push = |server_id: &str, action| {
            changes.push(PlannedChange {
                server_id: server_id.to_string(),
                app,
                scope,
                path: operation.path.clone(),
                action,
            })
        };

        for (id, value) in &desired {
            let action = match existing.get(id) {
                None => ChangeAction::Add,
                Some(current) if current == value => ChangeAction::Unchanged,
                Some(_) if managed_before.contains(id) => ChangeAction::Update,
                Some(_) => ChangeAction::OverwriteUnmanaged,
            };
            push(id, action);
        }

        for id in operation.remove_keys.iter().flatten() {
            if !desired.contains_key(id) && existing.contains_key(id) {
                push(id, ChangeAction::Remove);
            }
        }
    }
    changes
}

fn is_entry_merge(mode: &str) -> bool {
    matches!(
        mode,
        "merge_json_object_entries" | "merge_toml_table_entries"
    )
}

fn object_from_json(content: &str) -> Map<String, Value> {
    match serde_json::from_str(content) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

/// The entries currently under `field` in the operation's file, as JSON.
fn existing_entries(
    ctx: &PlatformContext,
    operation: &WriteOperation,
    field: &str,
) -> Map<String, Value> {
    let Ok(content) = fs::read_to_string(ctx.resolve_path(&operation.path)) else {
        return Map::new();
    };
    let host = if operation.mode == "merge_toml_table_entries" {
        toml::from_str::<toml::Table>(&content)
            .ok()
            .and_then(|table| serde_json::to_value(table).ok())
    } else {
        serde_json::from_str::<Value>(&crate::parser::strip_json_comments(&content)).ok()
    };
    match host.as_ref().and_then(|host| host.get(field)) {
        Some(Value::Object(entries)) => entries.clone(),
        _ => Map::new(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    /// Empty when there is no current project.
    pub root: String,
    /// Project-level config file per app that supports one.
    pub placement_paths: HashMap<SupportedApp, String>,
}

pub fn workspace_info(ctx: &PlatformContext) -> WorkspaceInfo {
    if !ctx.has_workspace() {
        return WorkspaceInfo {
            root: String::new(),
            placement_paths: HashMap::new(),
        };
    }

    WorkspaceInfo {
        root: ctx.workspace_root.to_string_lossy().to_string(),
        placement_paths: SupportedApp::ALL
            .into_iter()
            .filter_map(|app| workspace_scope_path(ctx, app).map(|path| (app, path)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{preview, workspace_info, ChangeAction};
    use crate::core::{
        empty_apps, CommandSpec, MCPConfig, MCPServer, PlacementScope, SupportedApp, TransportSpec,
    };
    use crate::platform::test_paths::{abs, UnixPath};
    use crate::platform::{PlatformContext, PlatformOs};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    fn ctx(workspace_root: &str) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: abs("/Users/test"),
            workspace_root: abs(workspace_root),
        }
    }

    #[test]
    fn reports_project_paths_from_the_backend_path_table() {
        let info = workspace_info(&ctx("/workspace/project"));

        assert_eq!(info.root.unix(), "/workspace/project");
        assert_eq!(
            info.placement_paths
                .get(&SupportedApp::Vscode)
                .map(UnixPath::unix)
                .as_deref(),
            Some("/workspace/project/.vscode/mcp.json")
        );
        assert!(!info.placement_paths.contains_key(&SupportedApp::Codex));
    }

    #[test]
    fn reports_no_workspace_when_launched_outside_a_project() {
        let info = workspace_info(&ctx("/"));

        assert!(info.root.is_empty());
        assert!(info.placement_paths.is_empty());
    }

    fn stdio_server(id: &str, program: &str, apps: &[SupportedApp]) -> MCPServer {
        let mut enabled = empty_apps();
        for app in apps {
            enabled.insert(*app, true);
        }
        MCPServer {
            description: None,
            homepage: None,
            id: id.to_string(),
            name: id.to_string(),
            enabled: true,
            transport: TransportSpec {
                kind: "stdio".to_string(),
                url: None,
            },
            command: Some(CommandSpec {
                program: program.to_string(),
                args: vec![],
                env: HashMap::new(),
            }),
            apps: enabled,
            placements: vec![],
        }
    }

    fn config(servers: Vec<MCPServer>) -> MCPConfig {
        MCPConfig {
            version: 1,
            servers,
        }
    }

    fn action_for(
        changes: &[super::PlannedChange],
        id: &str,
        app: SupportedApp,
    ) -> Option<ChangeAction> {
        changes
            .iter()
            .find(|change| change.server_id == id && change.app == app)
            .map(|change| change.action)
    }

    #[test]
    fn previews_add_unchanged_update_overwrite_and_remove() {
        let home = tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let cursor = SupportedApp::Cursor;
        fs::create_dir_all(home.path().join(".cursor")).unwrap();
        // "same" and "changed" were written by the previous config; "manual"
        // was added by hand; "gone" was written before and is now removed.
        fs::write(
            home.path().join(".cursor/mcp.json"),
            r#"{ "mcpServers": {
                "same":    { "command": "a", "args": [], "env": {}, "type": "stdio" },
                "changed": { "command": "old", "args": [], "env": {}, "type": "stdio" },
                "manual":  { "command": "mine" },
                "gone":    { "command": "g", "args": [], "env": {}, "type": "stdio" }
            } }"#,
        )
        .unwrap();

        let previous = config(vec![
            stdio_server("same", "a", &[cursor]),
            stdio_server("changed", "old", &[cursor]),
            stdio_server("gone", "g", &[cursor]),
        ]);
        let next = config(vec![
            stdio_server("same", "a", &[cursor]),
            stdio_server("changed", "new", &[cursor]),
            stdio_server("manual", "ours", &[cursor]),
            stdio_server("fresh", "f", &[cursor]),
        ]);

        let changes = preview(&ctx, &next, Some(&previous));

        assert_eq!(
            action_for(&changes, "same", cursor),
            Some(ChangeAction::Unchanged)
        );
        assert_eq!(
            action_for(&changes, "changed", cursor),
            Some(ChangeAction::Update)
        );
        assert_eq!(
            action_for(&changes, "manual", cursor),
            Some(ChangeAction::OverwriteUnmanaged)
        );
        assert_eq!(
            action_for(&changes, "fresh", cursor),
            Some(ChangeAction::Add)
        );
        assert_eq!(
            action_for(&changes, "gone", cursor),
            Some(ChangeAction::Remove)
        );
        assert!(changes
            .iter()
            .all(|change| change.scope == PlacementScope::User));
    }

    #[test]
    fn previews_toml_clients() {
        let home = tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let codex = SupportedApp::Codex;
        let next = config(vec![stdio_server("ctx", "npx", &[codex])]);

        let changes = preview(&ctx, &next, None);
        assert_eq!(action_for(&changes, "ctx", codex), Some(ChangeAction::Add));

        // Write what apply would, without apply's backups (which go to the real
        // app data dir).
        for operation in super::plan(&ctx, &next, None) {
            let entries: serde_json::Value = serde_json::from_str(&operation.content).unwrap();
            let mut host = toml::Table::new();
            host.insert(
                operation.field.clone().unwrap(),
                serde_json::from_value(entries).unwrap(),
            );
            let path = ctx.resolve_path(&operation.path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, toml::to_string(&host).unwrap()).unwrap();
        }
        let changes = preview(&ctx, &next, Some(&next));
        assert_eq!(
            action_for(&changes, "ctx", codex),
            Some(ChangeAction::Unchanged)
        );
    }
}
