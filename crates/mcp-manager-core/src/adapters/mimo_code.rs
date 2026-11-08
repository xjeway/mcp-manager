use crate::adapters::{
    managed_json_field_writes_with, opencode_mcp_servers_at, scope_for_path, AppAdapter,
    ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, PlacementScope, SupportedApp, WriteOperation};
use crate::parser::{
    enable_servers_for_app_at, extract_opencode_mcp_json, parse_jsonc, parse_mcp_json,
};
use crate::platform::PlatformContext;
use serde_json::Value;
use std::fs;

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
    let mut result = serde_json::json!({});
    for (path, _) in sources(ctx)?.into_iter().rev() {
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

/// Keeps MiMo's inherited client switch even if its defining file is a different layer.
fn content_at(config: &MCPConfig, app: SupportedApp, path: &str, ctx: &PlatformContext) -> Value {
    let mut entries = opencode_mcp_servers_at(config, app, path, ctx);
    if let (Some(entries), Ok(effective)) = (entries.as_object_mut(), effective(ctx, None)) {
        for (id, entry) in entries {
            if let Some(enabled) = effective
                .get("mcp")
                .and_then(|mcp| mcp.get(id))
                .and_then(|server| server.get("enabled"))
                .filter(|value| value.is_boolean())
            {
                entry["enabled"] = enabled.clone();
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
            let mut merged = effective(ctx, Some((path, content)))?;
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
                .map(|server| (server, priority))
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
        managed_json_field_writes_with(ctx, "mcp", self.app(), config, previous, content_at)
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
}
