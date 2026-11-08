use crate::adapters::{
    managed_json_field_writes_with, opencode_mcp_servers_at, scope_for_path, AppAdapter,
    ParsedSources,
};
use crate::core::{
    LocalConfigSource, MCPConfig, MCPServer, PlacementScope, ServerPlacement, SupportedApp,
    WriteOperation,
};
use crate::parser::{
    enable_servers_for_app_at, extract_opencode_mcp_json, parse_jsonc, parse_mcp_json,
};
use crate::platform::PlatformContext;
use serde_json::Value;
use std::{fs, path::Path};

/// Xiaomi MiMo Code's local JSON/JSONC layers; never executes configs or resolves secrets.
pub struct MiMoCodeAdapter;

/// Supported layers ordered by winner first; later upstream layers have smaller priorities.
fn sources(ctx: &PlatformContext) -> Result<Vec<(String, u32)>, String> {
    let base = ctx.mimo_config_dir()?;
    let mut files = vec![
        (ctx.home_dir.join(".mimocode/mimocode.jsonc"), 5),
        (ctx.home_dir.join(".mimocode/mimocode.json"), 6),
    ];
    if ctx.has_workspace() {
        for (path, priority) in [
            (".mimocode/mimocode.jsonc", 10),
            (".mimocode/mimocode.json", 11),
            ("mimocode.jsonc", 12),
            ("mimocode.json", 13),
        ] {
            files.push((ctx.workspace_file(path), priority));
        }
    }
    files.extend([
        (base.join("mimocode.jsonc"), 20),
        (base.join("mimocode.json"), 21),
        (base.join("config.json"), 22),
    ]);
    let mut seen = std::collections::BTreeSet::new();
    Ok(files
        .into_iter()
        .map(|(path, priority)| (path.to_string_lossy().to_string(), priority))
        .filter(|(path, _)| seen.insert(path.clone()))
        .collect())
}

/// Deep merge matches the upstream configuration loader; arrays and scalars replace.
fn merge(target: &mut Value, source: Value) {
    if let (Some(target), Some(source)) = (target.as_object_mut(), source.as_object()) {
        for (key, value) in source {
            merge(
                target.entry(key.clone()).or_insert(Value::Null),
                value.clone(),
            );
        }
    } else {
        *target = source;
    }
}

/// Produces effective MCP data while keeping original references as strings.
fn effective(ctx: &PlatformContext, input: Option<(&str, &str)>) -> Result<Value, String> {
    effective_for(ctx, input, false)
}

/// Project imports exclude the later private home overlay.
fn effective_for(
    ctx: &PlatformContext,
    input: Option<(&str, &str)>,
    project: bool,
) -> Result<Value, String> {
    let mut result = serde_json::json!({});
    for (path, priority) in sources(ctx)?.into_iter().rev() {
        if project && priority < 10 {
            continue;
        }
        let content = if let Some((_, content)) = input.filter(|(source, _)| *source == path) {
            content.to_string()
        } else {
            match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(format!("{path}: {error}")),
            }
        };
        let value = parse_jsonc(&content).map_err(|error| format!("{path}: {error}"))?;
        if !value.is_object() || value.get("mcp").is_some_and(|value| !value.is_object()) {
            return Err(format!(
                "{path}: MiMo configuration and its mcp section must be objects"
            ));
        }
        merge(&mut result, value);
    }
    Ok(result)
}

/// Invalid profiles or input layers must fail before any client file is written.
pub(crate) fn validate_mimo_context(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
) -> Result<(), String> {
    let targeted = std::iter::once(config)
        .chain(previous)
        .flat_map(|config| &config.servers)
        .any(|server| {
            server
                .apps
                .get(&SupportedApp::MiMoCode)
                .copied()
                .unwrap_or(false)
                || server
                    .placements
                    .iter()
                    .any(|p| p.app == SupportedApp::MiMoCode && p.managed)
        });
    if targeted {
        effective(ctx, None)?;
    }
    Ok(())
}

/// Serialize a canonical model for comparison, independently of its installation flags.
fn native_models(servers: Vec<MCPServer>, path: &str, ctx: &PlatformContext) -> Value {
    let servers = servers
        .into_iter()
        .map(|mut server| {
            server.enabled = true;
            server.placements = vec![ServerPlacement {
                app: SupportedApp::MiMoCode,
                scope: PlacementScope::User,
                path: Some(path.into()),
                enabled: true,
                managed: true,
            }];
            server
        })
        .collect();
    opencode_mcp_servers_at(
        &MCPConfig {
            version: 1,
            servers,
        },
        SupportedApp::MiMoCode,
        path,
        ctx,
    )
}

/// Apply only user-edited modeled fields to a layer's own definition.
/// Unchanged private values from another layer must never be materialized here.
fn patch_changes(layer: &mut Value, baseline: &Value, desired: &Value) {
    let (Some(baseline), Some(desired)) = (baseline.as_object(), desired.as_object()) else {
        *layer = desired.clone();
        return;
    };
    if !layer.is_object() {
        *layer = serde_json::json!({});
    }
    let layer = layer.as_object_mut().expect("layer object");
    let keys: std::collections::BTreeSet<_> = baseline.keys().chain(desired.keys()).collect();
    for key in keys {
        if key == "enabled" || baseline.get(key) == desired.get(key) {
            continue;
        }
        match desired.get(key) {
            Some(value) if value.is_object() && baseline.get(key).is_some_and(Value::is_object) => {
                patch_changes(
                    layer
                        .entry(key.clone())
                        .or_insert_with(|| serde_json::json!({})),
                    &baseline[key],
                    value,
                );
            }
            Some(value) => {
                layer.insert(key.clone(), value.clone());
            }
            None => {
                layer.remove(key);
            }
        }
    }
}

/// A newly selected project alias may inherit existing project data, never home overlay data.
fn project_entry(ctx: &PlatformContext, id: &str) -> Option<Value> {
    let mut project = serde_json::json!({});
    for (path, priority) in sources(ctx).ok()?.into_iter().rev() {
        if !(10..20).contains(&priority) {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = parse_jsonc(&content) else {
            continue;
        };
        merge(&mut project, value);
    }
    project.get("mcp")?.get(id).cloned()
}

/// Read only project-layer data for retained-field warnings and alias migration.
pub(crate) fn retained_project_entry(ctx: &PlatformContext, path: &str, id: &str) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| parse_jsonc(&content).ok())
        .and_then(|root| root.get("mcp").and_then(|entries| entries.get(id)).cloned())
        .or_else(|| project_entry(ctx, id))
}

/// Keep each managed layer's raw values; distribute only explicit edits between layers.
fn layer_entries(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous: Option<&MCPConfig>,
    path: &str,
) -> Value {
    let old = fs::read_to_string(path)
        .ok()
        .and_then(|content| parse_jsonc(&content).ok());
    let baseline_servers = previous
        .map(|config| config.servers.clone())
        .unwrap_or_else(|| {
            effective(ctx, None)
                .ok()
                .and_then(|value| extract_opencode_mcp_json(&value.to_string()).ok())
                .map(|content| parse_mcp_json(&content).servers)
                .unwrap_or_default()
        });
    let baseline = native_models(baseline_servers, path, ctx);
    let mut entries = opencode_mcp_servers_at(config, SupportedApp::MiMoCode, path, ctx);
    if let Some(entries) = entries.as_object_mut() {
        for (id, desired) in entries {
            let project = scope_for_path(ctx, path) == PlacementScope::Workspace
                && !Path::new(path).starts_with(ctx.mimo_config_dir().unwrap_or_default());
            let raw = old
                .as_ref()
                .and_then(|root| root.get("mcp"))
                .and_then(|entries| entries.get(id))
                .cloned()
                .or_else(|| project.then(|| project_entry(ctx, id)).flatten());
            let owned = previous.is_none_or(|previous| {
                previous
                    .servers
                    .iter()
                    .find(|server| server.id == *id)
                    .is_some_and(|server| {
                        server.placements.iter().any(|placement| {
                            placement.app == SupportedApp::MiMoCode
                                && placement.managed
                                && (placement.path.as_deref() == Some(path)
                                    || (project
                                        && placement.scope == PlacementScope::Workspace
                                        && placement.path.as_ref().is_some_and(|path| {
                                            ctx.resolve_path(path).starts_with(&ctx.workspace_root)
                                        })))
                        }) || (server
                            .apps
                            .get(&SupportedApp::MiMoCode)
                            .copied()
                            .unwrap_or(false)
                            && ctx
                                .user_app_config_path(SupportedApp::MiMoCode)
                                .to_string_lossy()
                                == path)
                    })
            });
            if !owned {
                continue;
            }
            if let (Some(mut layer), Some(base)) = (raw, baseline.get(id)) {
                if base.get("type") != desired.get("type") {
                    if let Some(enabled) = layer.get("enabled").cloned() {
                        desired["enabled"] = enabled;
                    }
                } else {
                    patch_changes(&mut layer, base, desired);
                    *desired = layer;
                }
            }
        }
    }
    entries
}

impl AppAdapter for MiMoCodeAdapter {
    /// Registry identity and CLI alias `mimo`.
    fn app(&self) -> SupportedApp {
        SupportedApp::MiMoCode
    }

    /// Lists every supported file because MiMo merges them rather than choosing one.
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        sources(ctx).unwrap_or_else(|_| {
            vec![(
                ctx.user_app_config_path(self.app())
                    .to_string_lossy()
                    .to_string(),
                20,
            )]
        })
    }

    /// Decodes effective entries but tracks only IDs actually declared by this source.
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
        let normalized = (|| {
            let declared = parse_jsonc(content)?;
            let ids = declared
                .get("mcp")
                .and_then(Value::as_object)
                .map(|mcp| {
                    mcp.keys()
                        .cloned()
                        .collect::<std::collections::BTreeSet<_>>()
                })
                .unwrap_or_default();
            let mut merged =
                effective_for(ctx, Some((path, content)), (10..20).contains(&priority))?;
            if let Some(mcp) = merged.get_mut("mcp").and_then(Value::as_object_mut) {
                mcp.retain(|id, _| ids.contains(id));
            }
            extract_opencode_mcp_json(&merged.to_string())
        })();
        let normalized = match normalized {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![source],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![format!("{path}: {error}")],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        let user_source = ctx
            .mimo_config_dir()
            .ok()
            .is_some_and(|base| std::path::Path::new(path).starts_with(base))
            || std::path::Path::new(path).starts_with(ctx.home_dir.join(".mimocode"));
        let scope = if user_source {
            PlacementScope::User
        } else {
            scope_for_path(ctx, path)
        };
        ParsedSources {
            sources: vec![source],
            servers: enable_servers_for_app_at(parsed.servers, self.app(), scope, path.into())
                .into_iter()
                .map(|mut server| {
                    if scope == PlacementScope::User {
                        server.placements.push(ServerPlacement {
                            app: self.app(),
                            scope,
                            path: Some(path.into()),
                            enabled: true,
                            managed: true,
                        });
                    }
                    (server, priority)
                })
                .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    /// Changes only managed entries under mcp and keeps unrelated runtime settings.
    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        if ctx.mimo_config_dir().is_err() {
            return vec![];
        }
        let allowed: std::collections::BTreeSet<_> = sources(ctx)
            .unwrap_or_default()
            .into_iter()
            .map(|(path, _)| {
                let path = ctx.resolve_path(&path);
                path.canonicalize().unwrap_or(path)
            })
            .collect();
        managed_json_field_writes_with(
            ctx,
            "mcp",
            self.app(),
            config,
            previous,
            opencode_mcp_servers_at,
        )
        .into_iter()
        .filter(|operation| {
            let path = ctx.resolve_path(&operation.path);
            allowed.contains(&path.canonicalize().unwrap_or(path))
        })
        .map(|mut operation| {
            operation.content = layer_entries(ctx, config, previous, &operation.path).to_string();
            operation
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{build_import_result, DetectedServer};
    use crate::platform::test_paths::abs;
    use crate::platform::{resolve_mimo_config_dir, PlatformOs};
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
    fn profile_resolution_matches_the_upstream_resolver_without_changing_process_env() {
        let home = abs("/home/test");
        let profile = abs("/profiles/work");
        let xdg = abs("/xdg");
        assert_eq!(
            resolve_mimo_config_dir(&home, None, None).unwrap(),
            home.join(".config/mimocode")
        );
        assert_eq!(
            resolve_mimo_config_dir(&home, None, Some(&xdg)).unwrap(),
            xdg.join("mimocode")
        );
        assert_eq!(
            resolve_mimo_config_dir(&home, Some(&profile), Some(&xdg)).unwrap(),
            profile.join("config")
        );
        assert!(resolve_mimo_config_dir(&home, Some(Path::new("relative")), None).is_err());
    }

    #[test]
    fn deep_merges_partial_jsonc_layers_before_decoding_transport_and_env() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let base = ctx.workspace_file("mimocode.json");
        let higher = ctx.workspace_file("mimocode.jsonc");
        write(
            &base,
            r#"{"mcp":{"fixture-stdio":{"type":"local","command":["node","old.js"],"environment":{"API_KEY":"{env:API_KEY}","KEEP":"yes"}},"fixture-remote":{"type":"remote","url":"https://example.com/mcp","headers":{"Authorization":"Bearer t"}}}}"#,
        );
        write(
            &higher,
            r#"{ // partial overrides are valid
          "mcp": {
            "fixture-stdio": {"command":["node","new.js"],"environment":{"ADDED":"yes"},"enabled":false},
            "fixture-remote": {"headers":{"X-Region":"local"}},
          },
        }"#,
        );
        let mut detected = Vec::new();
        for (path, priority) in [(base, 13), (higher.clone(), 12)] {
            let parsed = MiMoCodeAdapter.parse_source(
                &ctx,
                &path.to_string_lossy(),
                priority,
                &fs::read_to_string(&path).unwrap(),
            );
            assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
            detected.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
        let config = build_import_result(vec![], detected, vec![], vec![]).config;
        let local = config
            .servers
            .iter()
            .find(|server| server.id == "fixture-stdio")
            .unwrap();
        let command = local.command.as_ref().unwrap();
        assert_eq!(command.args, ["new.js"]);
        assert_eq!(command.env["KEEP"], "yes");
        assert_eq!(command.env["ADDED"], "yes");
        assert_eq!(command.env["API_KEY"], "{env:API_KEY}");
        assert!(local.enabled, "a client switch is not global disablement");
        assert_eq!(local.placements.len(), 2);
        let remote = config
            .servers
            .iter()
            .find(|server| server.id == "fixture-remote")
            .unwrap();
        assert_eq!(remote.transport.headers["Authorization"], "Bearer t");
        assert_eq!(remote.transport.headers["X-Region"], "local");
        assert_eq!(ctx.mimo_project_path(), higher);
        let operations = MiMoCodeAdapter.plan_apply(&ctx, &config, None);
        for operation in &operations {
            apply_operation(Path::new(&operation.path), operation).unwrap();
        }
        let written = parse_jsonc(&fs::read_to_string(&higher).unwrap()).unwrap();
        assert_eq!(written["mcp"]["fixture-stdio"]["enabled"], false);
        assert_eq!(
            written["mcp"]["fixture-stdio"]["command"],
            json!(["node", "new.js"])
        );
    }

    #[test]
    fn hidden_project_file_wins_and_in_place_apply_keeps_oauth_timeouts_and_comments() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let root = ctx.workspace_file("mimocode.jsonc");
        let hidden = ctx.workspace_file(".mimocode/mimocode.jsonc");
        write(
            &root,
            r#"{"mcp":{"fixture-api":{"type":"remote","url":"https://lower/mcp"}}}"#,
        );
        let original = r#"{
  // Keep models and preferences.
  "model": "xiaomi/mimo-v2.5-pro",
  "mcp": {
    "fixture-api": {
      // OAuth is managed by the client.
      "type":"remote", "url":"https://winner/mcp", "enabled":false,
      "headers":{"Authorization":"Bearer t"}, "timeout":9000, "oauth":{"clientId":"keep"}
    },
    "unmanaged": {"type":"local","command":["node","untouched.js"]}
  }
}"#;
        write(&hidden, original);
        assert_eq!(ctx.mimo_project_path(), hidden);
        let parsed = MiMoCodeAdapter.parse_source(&ctx, &hidden.to_string_lossy(), 10, original);
        assert!(parsed.errors.is_empty());
        let mut server = parsed
            .servers
            .into_iter()
            .find(|(server, _)| server.id == "fixture-api")
            .unwrap()
            .0;
        server
            .transport
            .headers
            .insert("Authorization".into(), "Bearer updated".into());
        let config = MCPConfig {
            version: 1,
            servers: vec![server],
        };
        for _ in 0..2 {
            let ops = MiMoCodeAdapter.plan_apply(&ctx, &config, None);
            assert_eq!(ops.len(), 1);
            apply_operation(&hidden, &ops[0]).unwrap();
            let text = fs::read_to_string(&hidden).unwrap();
            assert!(text.contains("// Keep models and preferences."));
            assert!(text.contains("// OAuth is managed by the client."));
            let written = parse_jsonc(&text).unwrap();
            assert_eq!(written["model"], "xiaomi/mimo-v2.5-pro");
            let entry = &written["mcp"]["fixture-api"];
            assert_eq!(entry["headers"]["Authorization"], "Bearer updated");
            assert_eq!(entry["timeout"], 9000);
            assert_eq!(entry["enabled"], false);
            assert_eq!(entry["oauth"]["clientId"], "keep");
        }
        let empty = MCPConfig {
            version: 1,
            servers: vec![],
        };
        for op in MiMoCodeAdapter.plan_apply(&ctx, &empty, Some(&config)) {
            apply_operation(&hidden, &op).unwrap();
        }
        let written = parse_jsonc(&fs::read_to_string(&hidden).unwrap()).unwrap();
        assert!(written["mcp"].get("fixture-api").is_none());
        assert!(written["mcp"].get("unmanaged").is_some());
    }

    #[test]
    fn malformed_supported_layers_abort_instead_of_overwriting_them() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.workspace_file("mimocode.jsonc");
        let valid = r#"{"mcp":{"fixture":{"type":"local","command":["node"]}}}"#;
        let parsed = MiMoCodeAdapter.parse_source(&ctx, &path.to_string_lossy(), 12, valid);
        let config = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        write(&path, "{broken");
        assert!(crate::workflow::prepare(&ctx, &config, None).is_err());
        assert!(crate::workflow::apply(&ctx, &config, None).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{broken");
    }
    #[test]
    fn private_home_overrides_never_replace_project_layer_values() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let project = ctx.workspace_file("mimocode.jsonc");
        let home = ctx.home_dir.join(".mimocode/mimocode.jsonc");
        let project_text = r#"{"mcp":{"fixture-private":{"type":"local","command":["project-server"],"environment":{"TOKEN":"project-literal-token","PROJECT_ONLY":"public"}}}}"#;
        let home_text = r#"{"mcp":{"fixture-private":{"type":"local","command":["private-server"],"environment":{"TOKEN":"${HOME_TOKEN}","HOME_ONLY":"private-home-value"}}}}"#;
        write(&project, project_text);
        write(&home, home_text);
        let project_parsed =
            MiMoCodeAdapter.parse_source(&ctx, project.to_str().unwrap(), 13, project_text);
        assert_eq!(
            project_parsed.servers[0].0.command.as_ref().unwrap().env["TOKEN"],
            "project-literal-token"
        );
        let home_parsed = MiMoCodeAdapter.parse_source(&ctx, home.to_str().unwrap(), 5, home_text);
        let detected = project_parsed
            .servers
            .into_iter()
            .chain(home_parsed.servers)
            .map(|(server, priority)| DetectedServer { server, priority })
            .collect();
        let config = build_import_result(vec![], detected, vec![], vec![]).config;
        let warnings = crate::adapters::shared_project_secret_warnings(&ctx, &config);
        assert!(warnings
            .iter()
            .any(|warning| warning.key.as_deref() == Some("environment.TOKEN")));
        assert!(!serde_json::to_string(&warnings)
            .unwrap()
            .contains("project-literal-token"));
        assert_eq!(
            config.servers[0].command.as_ref().unwrap().program,
            "private-server"
        );
        for operation in MiMoCodeAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(Path::new(&operation.path), &operation).unwrap();
        }
        let value = parse_jsonc(&fs::read_to_string(&project).unwrap()).unwrap();
        assert_eq!(
            value["mcp"]["fixture-private"]["command"],
            json!(["project-server"])
        );
        assert_eq!(
            value["mcp"]["fixture-private"]["environment"]["TOKEN"],
            "project-literal-token"
        );
        assert!(!value.to_string().contains("private-home-value"));
        let mut changed = config.clone();
        changed.servers[0]
            .command
            .as_mut()
            .unwrap()
            .env
            .insert("LOG_LEVEL".into(), "debug".into());
        for operation in MiMoCodeAdapter.plan_apply(&ctx, &changed, Some(&config)) {
            apply_operation(Path::new(&operation.path), &operation).unwrap();
        }
        let value = parse_jsonc(&fs::read_to_string(&project).unwrap()).unwrap();
        assert_eq!(
            value["mcp"]["fixture-private"]["environment"]["LOG_LEVEL"],
            "debug"
        );
        assert_eq!(
            value["mcp"]["fixture-private"]["environment"]["TOKEN"],
            "project-literal-token"
        );
        assert!(!value.to_string().contains("private-home-value"));
    }

    #[test]
    fn imports_pin_all_user_layers_for_complete_managed_removal() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let paths = [
            ctx.home_dir.join(".mimocode/mimocode.jsonc"),
            ctx.mimo_config_dir().unwrap().join("mimocode.json"),
        ];
        let mut detected = vec![];
        for (index, path) in paths.iter().enumerate() {
            // Feed the base source in memory; never write a process-level XDG/profile directory.
            let parsed = MiMoCodeAdapter.parse_source(
                &ctx,
                path.to_str().unwrap(),
                if index == 0 { 5 } else { 21 },
                r#"{"mcp":{"fixture-remove":{"type":"local","command":["node"]}}}"#,
            );
            detected.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
        let previous = build_import_result(vec![], detected, vec![], vec![]).config;
        assert_eq!(
            previous.servers[0]
                .placements
                .iter()
                .filter(|p| p.scope == PlacementScope::User)
                .count(),
            2
        );
        let next = MCPConfig {
            version: 1,
            servers: vec![],
        };
        let operations = MiMoCodeAdapter.plan_apply(&ctx, &next, Some(&previous));
        for path in paths {
            assert!(operations
                .iter()
                .any(|operation| Path::new(&operation.path) == path
                    && operation
                        .remove_keys
                        .as_ref()
                        .unwrap()
                        .contains(&"fixture-remove".to_string())));
        }
    }

    #[test]
    fn project_only_files_are_configuration_markers_for_both_clients() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        write(&ctx.workspace_file(".mimocode/mimocode.jsonc"), "{}");
        write(&ctx.workspace_file(".zcode/config.json"), "{}");
        assert!(ctx.config_marker_exists(SupportedApp::MiMoCode));
        assert!(ctx.config_marker_exists(SupportedApp::ZCode));
    }
    #[test]
    fn new_user_assignment_replaces_an_unmanaged_same_id_definition() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.home_dir.join(".mimocode/mimocode.jsonc");
        write(
            &path,
            r#"{"mcp":{"fixture-new":{"type":"local","command":["unmanaged"]}}}"#,
        );
        let parsed = parse_mcp_json(r#"{"mcpServers":{"fixture-new":{"command":"desired"}}}"#);
        let previous = MCPConfig {
            version: 1,
            servers: parsed.servers,
        };
        let mut next = previous.clone();
        next.servers[0].apps.insert(SupportedApp::MiMoCode, true);
        for operation in MiMoCodeAdapter.plan_apply(&ctx, &next, Some(&previous)) {
            apply_operation(Path::new(&operation.path), &operation).unwrap();
        }
        let value = parse_jsonc(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["mcp"]["fixture-new"]["command"], json!(["desired"]));
    }
}
