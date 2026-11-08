//! End-to-end operations over every client: import what is installed, plan and
//! preview a new config, and apply it. The app's commands and the CLI both call
//! these, so they behave the same.

use crate::adapters::{adapters, scope_for_path, workspace_scope_path, AppAdapter};
use crate::core::{
    build_import_result, ApplyResult, DetectedServer, ImportResult, LocalConfigSource, MCPConfig,
    PlacementScope, SupportedApp, WriteOperation,
};
use crate::parser::parse_yaml_config;
use crate::platform::PlatformContext;
use crate::storage::Snapshot;
use crate::store::config_path;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

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

/// Build client operations and reconcile ZCode migration with projected same-batch Agents changes.
fn plan_by_app(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<(SupportedApp, WriteOperation)> {
    let mut operations: Vec<_> = adapters()
        .into_iter()
        .flat_map(|adapter| {
            let app = adapter.app();
            adapter
                .plan_apply(ctx, config, previous_config)
                .into_iter()
                .map(move |operation| (app, operation))
        })
        .collect();
    let native =
        crate::adapters::plan_zcode_with_fallback(ctx, config, previous_config, &operations);
    let mut native = native.into_iter();
    for (app, operation) in &mut operations {
        if *app == SupportedApp::ZCode {
            if let Some(reconciled) = native.next() {
                *operation = reconciled;
            }
        }
    }
    operations
}

/// Every client file [`apply`] would write, as it is now, so a caller can put
/// them back if something after the apply fails.
pub fn snapshot(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Result<Snapshot, String> {
    let mut snapshot = Snapshot::default();
    for operation in plan(ctx, config, previous_config) {
        snapshot.capture(&ctx.resolve_path(&operation.path))?;
    }
    Ok(snapshot)
}

/// An inherited parent file cannot be disabled by removing its saved placement.
pub fn validate_inherited_agents(
    ctx: &PlatformContext,
    next: &MCPConfig,
    previous: Option<&MCPConfig>,
) -> Result<(), String> {
    if !ctx.has_workspace() {
        return Ok(());
    }
    let Some(parent) = ctx.workspace_root.parent() else {
        return Ok(());
    };
    let inherited_path = parent.join(".agents/mcp.json");
    let inherited_path = inherited_path.canonicalize().unwrap_or(inherited_path);
    let loaded;
    let previous = match previous {
        Some(previous) => previous,
        None => {
            loaded = crate::store::load(ctx)?;
            &loaded
        }
    };
    for server in &previous.servers {
        let inherited = server.placements.iter().find(|placement| {
            placement.app == SupportedApp::Agents
                && placement.scope == PlacementScope::Workspace
                && placement.enabled
                && !placement.managed
                && placement.path.as_ref().is_some_and(|path| {
                    let path = ctx.resolve_path(path);
                    path.canonicalize().unwrap_or(path) == inherited_path
                })
        });
        if inherited.is_none() {
            continue;
        }
        let retained = next
            .servers
            .iter()
            .find(|next| next.id == server.id)
            .is_some_and(|next| {
                (!server.enabled || next.enabled)
                    && next.placements.iter().any(|placement| {
                        placement.app == SupportedApp::Agents
                            && placement.enabled
                            && !placement.managed
                            && placement.path.as_ref().is_some_and(|path| {
                                let path = ctx.resolve_path(path);
                                path.canonicalize().unwrap_or(path) == inherited_path
                            })
                    })
            });
        if !retained {
            return Err(format!("Server {} is inherited from {}. Edit that parent file to disable or remove it. Nothing was written.", server.id, inherited_path.display()));
        }
    }
    Ok(())
}

/// Writes `config` to every client, backing up each file first.
pub fn apply(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Result<ApplyResult, String> {
    prepare(ctx, config, previous_config)?.apply(ctx)
}

/// Applies exactly these operations without resolving client paths again.
fn apply_plan(
    ctx: &PlatformContext,
    config: &MCPConfig,
    operations: Vec<WriteOperation>,
    expected: &Snapshot,
    inputs: &Snapshot,
    bound_mimo_path: Option<&Path>,
) -> Result<ApplyResult, String> {
    for operation in &operations {
        let path = ctx.resolve_path(&operation.path);
        if !ctx.can_write(&path) {
            let error = format!("permission denied for {}", path.to_string_lossy());
            if crate::security::is_high_risk_condition(&error) {
                return Err(error);
            }
        }
    }

    let committed =
        crate::storage::apply_operations_checked_with_inputs(operations, expected, inputs, || {
            validate_mimo_binding(ctx, bound_mimo_path)
        })?;
    Ok(ApplyResult {
        backups: committed.backups,
        committed_files: committed.files,
        warnings: crate::adapters::apply_warnings(config),
        history_error: None,
    })
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

/// A single preview and its approved target files. Applying never replans it.
pub struct PreparedApply {
    operations: Vec<WriteOperation>,
    snapshot: Snapshot,
    inputs: Snapshot,
    bound_mimo_path: Option<PathBuf>,
    config: MCPConfig,
    previous: Option<MCPConfig>,
    pub changes: Vec<PlannedChange>,
}

impl PreparedApply {
    /// Rejects changed contents or active paths before any write is attempted.
    pub fn validate(&self, ctx: &PlatformContext) -> Result<(), String> {
        validate_mimo_binding(ctx, self.bound_mimo_path.as_deref())?;
        validate_inherited_agents(ctx, &self.config, self.previous.as_ref())?;
        crate::adapters::validate_command_context(ctx, &self.config, self.previous.as_ref())?;
        let inputs = planning_inputs(ctx, &self.config, self.previous.as_ref())?;
        if self.inputs.fingerprints()? != inputs.fingerprints()? {
            return Err("Client configuration inputs (contents or active path) changed after the preview. Nothing was written; review the new configuration.".into());
        }
        crate::adapters::validate_rovo_context(ctx, &self.config, self.previous.as_ref())?;
        let current = snapshot(ctx, &self.config, self.previous.as_ref())?;
        if self.snapshot.fingerprints()? != current.fingerprints()? {
            return Err("Client configuration changed after the preview (contents or active path). Nothing was written; run the command again to review the new target.".into());
        }
        Ok(())
    }

    /// Captures rollback state for the frozen targets without resolving paths again.
    pub fn snapshot(&self, ctx: &PlatformContext) -> Result<Snapshot, String> {
        let mut snapshot = Snapshot::default();
        for operation in &self.operations {
            snapshot.capture(&ctx.resolve_path(&operation.path))?;
        }
        Ok(snapshot)
    }

    /// Rechecks the targets and applies the frozen, previewed operations only.
    pub fn apply(&self, ctx: &PlatformContext) -> Result<ApplyResult, String> {
        self.validate(ctx)?;
        self.apply_validated(ctx)
    }

    /// Keeps the final-validation/write boundary explicit for race regression tests.
    fn apply_validated(&self, ctx: &PlatformContext) -> Result<ApplyResult, String> {
        apply_plan(
            ctx,
            &self.config,
            self.operations.clone(),
            &self.snapshot,
            &self.inputs,
            self.bound_mimo_path.as_deref(),
        )
    }
}

/// Fingerprint read-only dependencies separately from files eligible for rollback.
fn planning_inputs(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
) -> Result<Snapshot, String> {
    let mut inputs = Snapshot::default();
    let footprint = plan_by_app(ctx, config, previous);
    for (_, operation) in &footprint {
        inputs.capture(&ctx.resolve_path(&operation.path))?;
    }
    if footprint
        .iter()
        .any(|(app, _)| *app == SupportedApp::CodeBuddy)
    {
        for path in [
            ".codebuddy/.mcp.json",
            ".codebuddy/mcp.json",
            ".codebuddy.json",
        ] {
            inputs.capture(&ctx.home_dir.join(path))?;
        }
    }
    for path in crate::adapters::zcode_fallback_input_paths(ctx, &footprint) {
        inputs.capture(&ctx.resolve_path(&path))?;
    }
    if footprint
        .iter()
        .any(|(app, _)| *app == SupportedApp::MiMoCode)
    {
        for (path, _) in crate::adapters::MiMoCodeAdapter.detect_sources(ctx) {
            inputs.capture(&ctx.resolve_path(&path))?;
        }
    }
    if footprint
        .iter()
        .any(|(app, _)| *app == SupportedApp::RovoDev)
    {
        inputs.capture(&ctx.home_dir.join(".rovodev/config.yml"))?;
    }
    Ok(inputs)
}

/// A validated primary alias is part of the approved context, not a later lookup.
fn validate_mimo_binding(ctx: &PlatformContext, path: Option<&Path>) -> Result<(), String> {
    if let Some(path) = path {
        if !ctx.has_workspace() || ctx.mimo_project_path() != path {
            return Err("MiMo project configuration priority changed. Refresh the project selection before applying.".into());
        }
    }
    Ok(())
}

/// Freeze implicit project placements to the same alias validated for this transaction.
fn bind_mimo_placements(config: &MCPConfig, path: Option<&Path>) -> MCPConfig {
    let mut config = config.clone();
    if let Some(path) = path {
        for placement in config
            .servers
            .iter_mut()
            .flat_map(|server| &mut server.placements)
            .filter(|placement| {
                placement.app == SupportedApp::MiMoCode
                    && placement.scope == PlacementScope::Workspace
                    && placement.path.is_none()
            })
        {
            placement.path = Some(path.to_string_lossy().to_string());
        }
    }
    config
}

/// Prepares the preview, file fingerprints and writes from one resolved plan.
pub fn prepare(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
) -> Result<PreparedApply, String> {
    prepare_with_mimo_project_path(ctx, config, previous, None)
}

/// Prepare one bound project selection for native commands and the history transaction.
pub fn prepare_with_mimo_project_path(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
    expected: Option<&str>,
) -> Result<PreparedApply, String> {
    prepare_with_capture_hook(ctx, config, previous, expected, || {})
}

/// Keep the semantic-validation/input-capture boundary explicit for concurrency regressions.
fn prepare_with_capture_hook(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
    expected: Option<&str>,
    before_capture: impl FnOnce(),
) -> Result<PreparedApply, String> {
    let implicit = std::iter::once(config)
        .chain(previous)
        .flat_map(|config| &config.servers)
        .flat_map(|server| &server.placements)
        .any(|placement| {
            placement.app == SupportedApp::MiMoCode
                && placement.scope == PlacementScope::Workspace
                && placement.managed
                && placement.path.is_none()
        });
    let bound_mimo_path = expected
        .map(PathBuf::from)
        .or_else(|| (implicit && ctx.has_workspace()).then(|| ctx.mimo_project_path()));
    validate_mimo_binding(ctx, bound_mimo_path.as_deref())?;
    let bound_config = bind_mimo_placements(config, bound_mimo_path.as_deref());
    let bound_previous =
        previous.map(|previous| bind_mimo_placements(previous, bound_mimo_path.as_deref()));
    let config = &bound_config;
    let previous = bound_previous.as_ref();
    validate_inherited_agents(ctx, config, previous)?;
    crate::adapters::validate_command_context(ctx, config, previous)?;
    crate::adapters::validate_mimo_context(ctx, config, previous)?;
    crate::adapters::validate_muse_context(ctx, config, previous)?;
    crate::adapters::validate_rovo_context(ctx, config, previous)?;
    before_capture();
    let inputs = planning_inputs(ctx, config, previous)?;
    // Validate native semantics against already captured dependencies; later edits fail fingerprints.
    crate::adapters::validate_mimo_context(ctx, config, previous)?;
    let by_app = plan_by_app(ctx, config, previous);
    crate::adapters::validate_project_fallback(ctx, &by_app)?;
    let operations = by_app
        .iter()
        .map(|(_, operation)| operation.clone())
        .collect::<Vec<_>>();
    let mut snapshot = Snapshot::default();
    for operation in &operations {
        snapshot.capture(&ctx.resolve_path(&operation.path))?;
    }
    let prepared = PreparedApply {
        changes: preview_plan(ctx, &by_app, previous),
        operations,
        snapshot,
        inputs,
        bound_mimo_path,
        config: config.clone(),
        previous: previous.cloned(),
    };
    // Fail conservatively if the file changed while the preview was read.
    prepared.validate(ctx)?;
    Ok(prepared)
}

/// What [`apply`] would do to each server in each client file, without writing.
pub fn preview(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<PlannedChange> {
    preview_plan(
        ctx,
        &plan_by_app(ctx, config, previous_config),
        previous_config,
    )
}

/// Describes the supplied operations rather than re-resolving their paths.
fn preview_plan(
    ctx: &PlatformContext,
    operations: &[(SupportedApp, WriteOperation)],
    previous_config: Option<&MCPConfig>,
) -> Vec<PlannedChange> {
    let previously_written = previous_config
        .map(|previous| plan_by_app(ctx, previous, None))
        .unwrap_or_default();

    let mut changes = Vec::new();
    for (app, operation) in operations {
        let Some(field) = operation.field.as_deref() else {
            continue;
        };
        if !is_entry_merge(&operation.mode) {
            continue;
        }

        let desired = object_from_json(&operation.content);
        let existing = existing_entries(ctx, operation, field);
        let managed_before = previously_written
            .iter()
            .filter(|(previous_app, previous)| {
                previous_app == app
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
                app: *app,
                scope,
                path: operation.path.clone(),
                action,
            })
        };

        for (id, value) in &desired {
            let action = match existing.get(id) {
                None => ChangeAction::Add,
                Some(current) if current == &with_kept_fields(current, value, &operation.mode) => {
                    ChangeAction::Unchanged
                }
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

/// `desired` as the apply will write it over `current`, which keeps the
/// fields MCP Manager does not model.
fn with_kept_fields(current: &Value, desired: &Value, mode: &str) -> Value {
    let mut written = desired.clone();
    if matches!(
        mode,
        "merge_json_object_entries"
            | "merge_mimo_entries"
            | "merge_json_object_path_entries"
            | "merge_muse_mcp_entries"
    ) {
        crate::storage::carry_unmanaged_json_fields(current, &mut written);
    } else {
        crate::storage::carry_unmanaged_fields(current, &mut written);
    }
    if mode == "merge_mimo_entries" {
        if let Some(enabled) = desired.get("enabled") {
            written["enabled"] = enabled.clone();
        }
    }
    written
}

/// Whether preview comparisons operate on individual entries rather than a whole field.
fn is_entry_merge(mode: &str) -> bool {
    matches!(
        mode,
        "merge_json_object_entries"
            | "merge_mimo_entries"
            | "merge_json_object_path_entries"
            | "merge_muse_mcp_entries"
            | "merge_toml_table_entries"
            | "merge_toml_array_entries"
            | "merge_yaml_mapping_entries"
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
    let host = if operation.mode.starts_with("merge_toml") {
        toml::from_str::<toml::Table>(&content)
            .ok()
            .and_then(|table| serde_json::to_value(table).ok())
    } else if operation.mode == "merge_yaml_mapping_entries" {
        serde_yaml::from_str::<Value>(&content).ok()
    } else {
        crate::parser::parse_jsonc(&content).ok()
    };
    match host.as_ref().and_then(|host| {
        if operation.mode == "merge_json_object_path_entries" {
            host.pointer(field)
        } else {
            host.get(field)
        }
    }) {
        Some(Value::Object(entries)) => entries.clone(),
        // `[[field]]` entries, keyed by the `name` the operation uses.
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(|entry| {
                let mut entry = entry.as_object()?.clone();
                let Value::String(name) = entry.remove("name")? else {
                    return None;
                };
                Some((name, Value::Object(entry)))
            })
            .collect(),
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
    fn prepared_apply_rejects_changes_after_validation_without_touching_either_file() {
        use crate::adapters::{AppAdapter, CodeBuddyAdapter};
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let legacy = ctx.home_dir.join(".codebuddy/mcp.json");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        let original = r#"{"mcpServers":{"api":{"command":"old"}}}"#;
        fs::write(&legacy, original).unwrap();
        let mut server = CodeBuddyAdapter
            .parse_source(&ctx, &legacy.to_string_lossy(), 20, original)
            .servers
            .remove(0)
            .0;
        server.command.as_mut().unwrap().program = "new".into();
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        let prepared = super::prepare(&ctx, &config, None).unwrap();
        assert_eq!(prepared.changes[0].path, legacy.to_string_lossy());
        prepared.validate(&ctx).unwrap();
        // Simulates an external client changing the active path after validation.
        let preferred = ctx.home_dir.join(".codebuddy/.mcp.json");
        fs::write(&preferred, "{}").unwrap();
        let rollback = prepared.snapshot(&ctx).unwrap();
        assert_eq!(
            rollback.paths().collect::<Vec<_>>(),
            vec![legacy.canonicalize().unwrap().as_path()]
        );
        assert!(prepared.apply(&ctx).unwrap_err().contains("active path"));
        assert_eq!(fs::read_to_string(&preferred).unwrap(), "{}");
        assert_eq!(fs::read_to_string(&legacy).unwrap(), original);
        fs::remove_file(preferred).unwrap();
        fs::write(&legacy, original.replace("old", "external-edit")).unwrap();
        assert!(prepared.apply(&ctx).is_err());
        assert!(fs::read_to_string(&legacy)
            .unwrap()
            .contains("external-edit"));
        // Successful prepared writes are covered by isolated CLI integration tests.
        // This test only exercises rejection paths, so it creates no global backups.
    }

    #[test]
    fn prepared_write_detects_an_external_edit_after_its_final_validation() {
        use crate::adapters::{AppAdapter, CodeBuddyAdapter};
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let legacy = ctx.home_dir.join(".codebuddy/mcp.json");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        let original = r#"{"mcpServers":{"api":{"command":"old"}}}"#;
        fs::write(&legacy, original).unwrap();
        let mut server = CodeBuddyAdapter
            .parse_source(&ctx, &legacy.to_string_lossy(), 20, original)
            .servers
            .remove(0)
            .0;
        server.command.as_mut().unwrap().program = "new".into();
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        let prepared = super::prepare(&ctx, &config, None).unwrap();
        prepared.validate(&ctx).unwrap();
        // Injection at the exact boundary between final validation and storage write.
        let preferred = ctx.home_dir.join(".codebuddy/.mcp.json");
        fs::write(&preferred, "{}").unwrap();
        let edited = original.to_string();
        assert!(prepared
            .apply_validated(&ctx)
            .unwrap_err()
            .contains("changed before writing"));
        assert_eq!(fs::read_to_string(&legacy).unwrap(), edited);
        assert_eq!(fs::read_to_string(&preferred).unwrap(), "{}");
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
        assert_eq!(
            info.placement_paths
                .get(&SupportedApp::Codex)
                .map(UnixPath::unix)
                .as_deref(),
            Some("/workspace/project/.codex/config.toml")
        );
        // Claude Desktop has no project scope.
        assert!(!info
            .placement_paths
            .contains_key(&SupportedApp::ClaudeDesktop));
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
                headers: Default::default(),
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
    fn fields_an_apply_keeps_do_not_count_as_a_change() {
        let home = tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let cursor = SupportedApp::Cursor;
        fs::create_dir_all(home.path().join(".cursor")).unwrap();
        // A hand-added timeout, a comment and a trailing comma.
        fs::write(
            home.path().join(".cursor/mcp.json"),
            r#"{ "mcpServers": {
                // tuned by hand
                "same": { "command": "a", "args": [], "env": {}, "type": "stdio", "timeout": 9000 },
            } }"#,
        )
        .unwrap();
        let servers = config(vec![stdio_server("same", "a", &[cursor])]);

        let changes = preview(&ctx, &servers, Some(&servers));

        assert_eq!(
            action_for(&changes, "same", cursor),
            Some(ChangeAction::Unchanged)
        );
    }

    #[test]
    fn previews_toml_list_clients() {
        let home = tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.path().to_path_buf(),
            workspace_root: home.path().to_path_buf(),
        };
        let vibe = SupportedApp::MistralVibe;
        let next = config(vec![stdio_server("ctx", "npx", &[vibe])]);
        assert_eq!(
            action_for(&preview(&ctx, &next, None), "ctx", vibe),
            Some(ChangeAction::Add)
        );

        for operation in super::plan(&ctx, &next, None) {
            let path = ctx.resolve_path(&operation.path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            crate::storage::apply_operation(&path, &operation).expect("apply");
        }
        // A hand-tuned field an apply keeps is not a change.
        let path = ctx.user_app_config_path(vibe);
        let tuned = fs::read_to_string(&path).unwrap() + "tool_timeout_sec = 120\n";
        fs::write(&path, tuned).unwrap();

        assert_eq!(
            action_for(&preview(&ctx, &next, Some(&next)), "ctx", vibe),
            Some(ChangeAction::Unchanged)
        );
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
    #[test]
    fn fallback_changed_after_final_validation_is_rejected_at_the_write_boundary() {
        use crate::adapters::{AppAdapter, ZCodeAdapter};
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let fallback = ctx.home_dir.join(".agents/mcp.json");
        fs::create_dir_all(fallback.parent().unwrap()).unwrap();
        let content = r#"{"mcpServers":{"fixture-fallback":{"command":"node"}}}"#;
        fs::write(&fallback, content).unwrap();
        let parsed = ZCodeAdapter.parse_source(&ctx, fallback.to_str().unwrap(), 8, content);
        let config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        let prepared = super::prepare(&ctx, &config, None).unwrap();
        prepared.validate(&ctx).unwrap();
        let external = r#"{"mcpServers":{"fixture-fallback":{"command":"external"}}}"#;
        fs::write(&fallback, external).unwrap();
        assert!(prepared.apply_validated(&ctx).is_err());
        assert_eq!(fs::read_to_string(&fallback).unwrap(), external);
        assert!(!ctx.user_app_config_path(SupportedApp::ZCode).exists());
    }
    #[test]
    fn global_cursor_apply_does_not_capture_an_unrelated_unreadable_project_source() {
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        fs::create_dir_all(ctx.workspace_file(".cursor/mcp.json")).unwrap();
        let mut server =
            crate::parser::parse_mcp_json(r#"{"mcpServers":{"global":{"command":"node"}}}"#)
                .servers
                .remove(0);
        server.apps.insert(SupportedApp::Cursor, true);
        let prepared = super::prepare(
            &ctx,
            &MCPConfig {
                version: 1,
                servers: vec![server],
            },
            None,
        )
        .unwrap();
        assert!(prepared
            .inputs
            .paths()
            .all(|path| path != ctx.workspace_file(".cursor/mcp.json")));
    }

    #[test]
    fn mimo_deletion_between_semantic_validation_and_capture_is_rejected() {
        use crate::adapters::{AppAdapter, MiMoCodeAdapter};
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        fs::create_dir_all(&ctx.workspace_root).unwrap();
        let project = ctx.workspace_file("mimocode.jsonc");
        let content = r#"{"mcp":{"capture-race":{"type":"local","command":["public"]}}}"#;
        fs::write(&project, content).unwrap();
        let private = ctx.home_dir.join(".mimocode/mimocode.jsonc");
        fs::create_dir_all(private.parent().unwrap()).unwrap();
        let private_content = r#"{"mcp":{"capture-race":{"type":"local","command":["private"],"environment":{"TOKEN":"secret"}}}}"#;
        fs::write(&private, private_content).unwrap();
        let config = MCPConfig {
            version: 1,
            servers: MiMoCodeAdapter
                .parse_source(&ctx, project.to_str().unwrap(), 13, content)
                .servers
                .into_iter()
                .map(|(s, _)| s)
                .collect(),
        };
        for previous in [Some(&config), None] {
            fs::write(&project, content).unwrap();
            let result = super::prepare_with_capture_hook(&ctx, &config, previous, None, || {
                fs::write(&project, "{}").unwrap()
            });
            assert!(result.err().unwrap().contains("no longer exists"));
            assert_eq!(fs::read_to_string(&project).unwrap(), "{}");
            assert_eq!(fs::read_to_string(&private).unwrap(), private_content);
        }
    }

    #[test]
    fn implicit_mimo_placements_use_the_bound_alias_and_reject_a_later_priority_change() {
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let mut server =
            crate::parser::parse_mcp_json(r#"{"mcpServers":{"fixture-bound":{"command":"node"}}}"#)
                .servers
                .remove(0);
        server.placements.push(crate::core::ServerPlacement {
            app: SupportedApp::MiMoCode,
            scope: PlacementScope::Workspace,
            path: None,
            enabled: true,
            managed: true,
        });
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        let previous = MCPConfig {
            version: 1,
            servers: vec![],
        };
        let expected = ctx.mimo_project_path();
        let prepared = super::prepare_with_mimo_project_path(
            &ctx,
            &config,
            Some(&previous),
            Some(expected.to_str().unwrap()),
        )
        .unwrap();
        assert_eq!(
            prepared.config.servers[0].placements[0].path.as_deref(),
            Some(expected.to_str().unwrap())
        );
        assert!(prepared
            .operations
            .iter()
            .any(|operation| std::path::Path::new(&operation.path) == expected));
        let higher = ctx.workspace_file(".mimocode/mimocode.jsonc");
        fs::create_dir_all(higher.parent().unwrap()).unwrap();
        fs::write(&higher, "{}").unwrap();
        assert!(prepared.apply_validated(&ctx).is_err());
        assert!(!expected.exists());
        assert_eq!(fs::read_to_string(higher).unwrap(), "{}");
    }
    #[test]
    fn a_targeted_fallback_is_checked_before_the_first_dependent_native_write() {
        let temp = tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let fallback = ctx.home_dir.join(".agents/mcp.json");
        fs::create_dir_all(fallback.parent().unwrap()).unwrap();
        fs::write(&fallback, r#"{"mcpServers":{"agents":{"command":"old"}}}"#).unwrap();
        let mut servers = crate::parser::parse_mcp_json(
            r#"{"mcpServers":{"agents":{"command":"new"},"native":{"command":"node"}}}"#,
        )
        .servers;
        servers
            .iter_mut()
            .find(|server| server.id == "agents")
            .unwrap()
            .apps
            .insert(SupportedApp::Agents, true);
        servers
            .iter_mut()
            .find(|server| server.id == "native")
            .unwrap()
            .apps
            .insert(SupportedApp::ZCode, true);
        let config = MCPConfig {
            version: 1,
            servers,
        };
        let prepared = super::prepare(&ctx, &config, None).unwrap();
        prepared.validate(&ctx).unwrap();
        let external = r#"{"mcpServers":{"agents":{"command":"external"}}}"#;
        fs::write(&fallback, external).unwrap();
        assert!(prepared.apply_validated(&ctx).is_err());
        assert!(!ctx.user_app_config_path(SupportedApp::ZCode).exists());
        assert_eq!(fs::read_to_string(fallback).unwrap(), external);
    }
}
