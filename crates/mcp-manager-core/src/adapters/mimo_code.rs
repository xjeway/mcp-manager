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
        {
            for server in config.servers.iter().filter(|server| {
                server.enabled
                    && server.placements.iter().any(|p| {
                        p.app == SupportedApp::MiMoCode
                            && p.scope == PlacementScope::Workspace
                            && p.enabled
                            && p.managed
                    })
            }) {
                let managed_before = previous.map_or_else(
                    || {
                        effective(ctx, None)
                            .ok()
                            .and_then(|root| {
                                root.get("mcp")
                                    .and_then(|entries| entries.get(&server.id))
                                    .cloned()
                            })
                            .is_some()
                    },
                    |previous| {
                        previous
                            .servers
                            .iter()
                            .find(|old| old.id == server.id)
                            .is_some_and(|old| {
                                old.placements.iter().any(|p| {
                                    p.app == SupportedApp::MiMoCode
                                        && p.scope == PlacementScope::Workspace
                                        && p.managed
                                })
                            })
                    },
                );
                if managed_before && project_entry(ctx, &server.id).is_none() {
                    return Err(format!("MiMo project entry {} no longer exists. Re-import before saving it; private user values were not copied.",server.id));
                }
            }
        }
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
    let config = MCPConfig {
        version: 1,
        servers,
    };
    let mut native = opencode_mcp_servers_at(&config, SupportedApp::MiMoCode, path, ctx);
    for server in &config.servers {
        if server.transport.kind != "stdio" && !server.transport.headers.is_empty() {
            if let Some(entry) = native.get_mut(&server.id) {
                entry["headers"] =
                    serde_json::to_value(&server.transport.headers).expect("headers");
            }
        }
    }
    native
}

/// Apply only user-edited modeled fields to a layer's own definition.
/// Unchanged private values from another layer must never be materialized here.
fn patch_changes(layer: &mut Value, baseline: &Value, desired: &Value, server_level: bool) {
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
        if (server_level && key == "enabled") || baseline.get(key) == desired.get(key) {
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
                    false,
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
    fs::read_to_string(ctx.resolve_path(path))
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
    let old = fs::read_to_string(ctx.resolve_path(path))
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
                    patch_changes(&mut layer, base, desired, true);
                    *desired = layer;
                }
            }
            let transport = crate::core::TransportSpec {
                kind: if desired.get("type").and_then(Value::as_str) == Some("local") {
                    "stdio".into()
                } else {
                    "http".into()
                },
                url: desired
                    .get("url")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                headers: desired
                    .get("headers")
                    .cloned()
                    .and_then(|headers| serde_json::from_value(headers).ok())
                    .unwrap_or_default(),
            };
            if transport.sends_headers_insecurely() {
                desired
                    .as_object_mut()
                    .expect("entry object")
                    .remove("headers");
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
        // Keep a disabled workspace entry in its native file so a later enable
        // can recover project-only values without using the merged home overlay.
        let mut retained = config.clone();
        for server in &mut retained.servers {
            if !server.enabled {
                server.apps.insert(self.app(), false);
                for p in &mut server.placements {
                    if p.scope == PlacementScope::User {
                        p.enabled = false;
                    }
                }
            }
            if let Some(old) =
                previous.and_then(|c| c.servers.iter().find(|old| old.id == server.id))
            {
                for placement in &mut server.placements {
                    if placement.app == self.app()
                        && placement.scope == PlacementScope::Workspace
                        && placement.managed
                        && old.placements.iter().any(|p| {
                            p.app == self.app()
                                && p.scope == placement.scope
                                && p.path == placement.path
                                && p.managed
                        })
                        && retained_project_entry(
                            ctx,
                            placement.path.as_deref().unwrap_or(""),
                            &server.id,
                        )
                        .is_some()
                    {
                        placement.enabled = true;
                        server.enabled = true;
                    }
                }
            }
        }
        // Disabled native project entries remain managed and must participate in deletion.
        let mut removal_previous = previous.cloned();
        if let Some(previous) = &mut removal_previous {
            for server in &mut previous.servers {
                if !server.enabled {
                    server.apps.insert(self.app(), false);
                    for p in &mut server.placements {
                        if p.scope == PlacementScope::User {
                            p.enabled = false;
                        }
                    }
                }
                for p in &mut server.placements {
                    if p.app == self.app() && p.scope == PlacementScope::Workspace && p.managed {
                        p.enabled = true;
                        server.enabled = true;
                    }
                }
            }
        }
        managed_json_field_writes_with(
            ctx,
            "mcp",
            self.app(),
            &retained,
            removal_previous.as_ref(),
            opencode_mcp_servers_at,
        )
        .into_iter()
        .filter(|operation| {
            let path = ctx.resolve_path(&operation.path);
            allowed.contains(&path.canonicalize().unwrap_or(path))
        })
        .map(|mut operation| {
            let mut entries = layer_entries(ctx, &retained, previous, &operation.path);
            for server in &config.servers {
                let selected = |s: &MCPServer| {
                    s.placements
                        .iter()
                        .find(|p| {
                            p.app == self.app()
                                && p.scope == PlacementScope::Workspace
                                && p.managed
                                && p.path.as_ref().is_some_and(|path| {
                                    ctx.resolve_path(path) == ctx.resolve_path(&operation.path)
                                })
                        })
                        .map(|p| s.enabled && p.enabled)
                };
                if let Some(active) = selected(server) {
                    let old_active = previous
                        .and_then(|c| c.servers.iter().find(|old| old.id == server.id))
                        .and_then(selected);
                    if !active || old_active == Some(false) {
                        if let Some(entry) = entries.get_mut(&server.id) {
                            entry["enabled"] = Value::Bool(active);
                        }
                    }
                }
            }
            operation.mode = "merge_mimo_entries".into();
            operation.content = entries.to_string();
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
    #[test]
    fn a_deleted_managed_project_entry_is_not_recreated_from_private_home() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let project = ctx.workspace_file("mimocode.jsonc");
        let home = ctx.home_dir.join(".mimocode/mimocode.jsonc");
        let public = r#"{"mcp":{"fixture-stale":{"type":"local","command":["project"]}}}"#;
        let private = r#"{"mcp":{"fixture-stale":{"type":"local","command":["home"],"environment":{"TOKEN":"private"}}}}"#;
        write(&project, public);
        write(&home, private);
        let mut detected = vec![];
        for (path, priority, content) in [(&project, 13, public), (&home, 5, private)] {
            detected.extend(
                MiMoCodeAdapter
                    .parse_source(&ctx, path.to_str().unwrap(), priority, content)
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
        let previous = build_import_result(vec![], detected, vec![], vec![]).config;
        write(&project, "{}");
        assert!(crate::workflow::prepare(&ctx, &previous, Some(&previous)).is_err());
        assert!(crate::workflow::prepare(&ctx, &previous, None).is_err());
        assert_eq!(fs::read_to_string(project).unwrap(), "{}");
        assert_eq!(fs::read_to_string(home).unwrap(), private);
    }

    #[test]
    fn disabling_and_reenabling_project_keeps_only_project_values() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let project = ctx.workspace_file("mimocode.jsonc");
        let home = ctx.home_dir.join(".mimocode/mimocode.jsonc");
        let public = r#"{"mcp":{"roundtrip":{"type":"local","command":["project"],"environment":{"TOKEN":"public"}}}}"#;
        write(&project, public);
        write(
            &home,
            r#"{"mcp":{"roundtrip":{"type":"local","command":["home"],"environment":{"TOKEN":"private"}}}}"#,
        );
        let mut active = MCPConfig {
            version: 1,
            servers: MiMoCodeAdapter
                .parse_source(&ctx, project.to_str().unwrap(), 13, public)
                .servers
                .into_iter()
                .map(|(s, _)| s)
                .collect(),
        };
        // Model the private merged canonical definition to ensure it is never used as project raw data.
        active.servers[0]
            .command
            .as_mut()
            .unwrap()
            .env
            .insert("TOKEN".into(), "private".into());
        let mut disabled = active.clone();
        disabled.servers[0].placements[0].enabled = false;
        for op in MiMoCodeAdapter.plan_apply(&ctx, &disabled, Some(&active)) {
            apply_operation(Path::new(&op.path), &op).unwrap();
        }
        let off = fs::read_to_string(&project).unwrap();
        assert_eq!(
            parse_jsonc(&off).unwrap()["mcp"]["roundtrip"]["enabled"],
            false
        );
        assert!(!off.contains("private"));
        for remove_placement_only in [false, true] {
            let mut removed = disabled.clone();
            if remove_placement_only {
                removed.servers[0].placements.clear();
            } else {
                removed.servers.clear();
            }
            for op in MiMoCodeAdapter.plan_apply(&ctx, &removed, Some(&disabled)) {
                apply_operation(Path::new(&op.path), &op).unwrap();
            }
            assert!(
                parse_jsonc(&fs::read_to_string(&project).unwrap()).unwrap()["mcp"]
                    .get("roundtrip")
                    .is_none()
            );
            write(&project, &off);
        }

        validate_mimo_context(&ctx, &active, Some(&disabled)).unwrap();
        for op in MiMoCodeAdapter.plan_apply(&ctx, &active, Some(&disabled)) {
            apply_operation(Path::new(&op.path), &op).unwrap();
        }
        let on = fs::read_to_string(project).unwrap();
        assert_eq!(
            parse_jsonc(&on).unwrap()["mcp"]["roundtrip"]["enabled"],
            true
        );
        assert!(!on.contains("private"));
    }

    #[test]
    fn tilde_user_layer_reads_its_own_raw_values() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let content = r#"{"mcp":{"tilde":{"type":"local","command":["local"],"environment":{"TOKEN":"layer-value"}}}}"#;
        write(&ctx.home_dir.join(".mimocode/mimocode.jsonc"), content);
        let previous = MCPConfig {
            version: 1,
            servers: MiMoCodeAdapter
                .parse_source(&ctx, "~/.mimocode/mimocode.jsonc", 5, content)
                .servers
                .into_iter()
                .map(|(s, _)| s)
                .collect(),
        };
        let mut desired = previous.clone();
        desired.servers[0]
            .command
            .as_mut()
            .unwrap()
            .env
            .insert("EXTRA".into(), "new".into());
        let entries = layer_entries(
            &ctx,
            &desired,
            Some(&previous),
            "~/.mimocode/mimocode.jsonc",
        );
        assert_eq!(entries["tilde"]["environment"]["TOKEN"], "layer-value");
        assert_eq!(entries["tilde"]["environment"]["EXTRA"], "new");
    }

    #[test]
    fn zcode_precedence_is_not_hidden_by_mimo_layer_precedence() {
        let mut current =
            parse_mcp_json(r#"{"mcpServers":{"same":{"url":"https://generic.example/mcp"}}}"#)
                .servers
                .remove(0);
        current.apps.insert(SupportedApp::MiMoCode, true);
        let mut incoming = parse_mcp_json(r#"{"mcpServers":{"same":{"command":"native-zcode"}}}"#)
            .servers
            .remove(0);
        incoming.apps.insert(SupportedApp::MiMoCode, true);
        incoming.apps.insert(SupportedApp::ZCode, true);
        let merged = crate::core::merge_servers(Some(&current), &incoming);
        assert_eq!(merged.transport.kind, "stdio");
        assert_eq!(merged.command.unwrap().program, "native-zcode");
    }

    #[test]
    fn nested_enabled_values_are_data_and_insecure_headers_are_never_restored() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = ctx(temp.path());
        let path = ctx.home_dir.join(".mimocode/mimocode.jsonc");
        let content = r#"{"mcp":{
            "fixture-env":{"type":"local","command":["node"],"enabled":false,"environment":{"enabled":"old"}},
            "fixture-header":{"type":"remote","url":"https://example.com/mcp","enabled":false,"headers":{"enabled":"old","Other":"keep"}},
            "fixture-loopback":{"type":"remote","url":"http://127.0.0.1:8300/mcp","headers":{"Authorization":"keep-local"}},
            "fixture-unsafe":{"type":"remote","url":"http://remote.example/mcp","headers":{"Authorization":"must-remove"}}
        }}"#;
        write(&path, content);
        let parsed = MiMoCodeAdapter.parse_source(&ctx, path.to_str().unwrap(), 5, content);
        let previous = MCPConfig {
            version: 1,
            servers: parsed
                .servers
                .into_iter()
                .map(|(server, _)| server)
                .collect(),
        };
        let mut next = previous.clone();
        next.servers
            .iter_mut()
            .find(|server| server.id == "fixture-env")
            .unwrap()
            .command
            .as_mut()
            .unwrap()
            .env
            .insert("enabled".into(), "new".into());
        next.servers
            .iter_mut()
            .find(|server| server.id == "fixture-header")
            .unwrap()
            .transport
            .headers
            .remove("enabled");
        next.servers
            .iter_mut()
            .find(|server| server.id == "fixture-unsafe")
            .unwrap()
            .transport
            .headers
            .clear();
        for operation in MiMoCodeAdapter.plan_apply(&ctx, &next, Some(&previous)) {
            apply_operation(Path::new(&operation.path), &operation).unwrap();
        }
        let value = parse_jsonc(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(value["mcp"]["fixture-env"]["enabled"], false);
        assert_eq!(value["mcp"]["fixture-env"]["environment"]["enabled"], "new");
        assert!(value["mcp"]["fixture-header"]["headers"]
            .get("enabled")
            .is_none());
        assert_eq!(value["mcp"]["fixture-header"]["headers"]["Other"], "keep");
        assert_eq!(
            value["mcp"]["fixture-loopback"]["headers"]["Authorization"],
            "keep-local"
        );
        assert!(value["mcp"]["fixture-unsafe"].get("headers").is_none());
    }
}
