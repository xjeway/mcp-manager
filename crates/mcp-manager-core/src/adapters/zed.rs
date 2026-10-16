use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_jsonc, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};

pub struct ZedAdapter;

/// Zed keeps MCP servers in its settings file, next to every other setting.
const FIELD: &str = "context_servers";

/// `context_servers` as an `mcpServers` document the shared parser reads.
///
/// Older Zed versions nest a local server's command as
/// `"command": {"path", "args", "env"}`; it is flattened here. Servers provided
/// by Zed extensions have only `settings`, no command or URL: they are left
/// out, so MCP Manager neither imports nor rewrites them.
fn normalize(content: &str) -> Result<String, String> {
    let root = parse_jsonc(content)?;
    let mut servers = Map::new();
    let entries = root
        .get(FIELD)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (id, entry) in entries {
        let Some(mut entry) = entry.as_object().cloned() else {
            continue;
        };
        if let Some(Value::Object(command)) = entry.remove("command") {
            for (from, to) in [("path", "command"), ("args", "args"), ("env", "env")] {
                if let Some(value) = command.get(from) {
                    entry.insert(to.to_string(), value.clone());
                }
            }
        }
        let local = entry.get("command").is_some_and(Value::is_string);
        let remote = entry.get("url").is_some_and(Value::is_string);
        if local || remote {
            servers.insert(id, Value::Object(entry));
        }
    }
    let mut document = Map::new();
    document.insert("mcpServers".to_string(), Value::Object(servers));
    serde_json::to_string(&Value::Object(document)).map_err(|e| e.to_string())
}

impl AppAdapter for ZedAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Zed
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".zed/settings.json")
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
        let source = |content: String| LocalConfigSource {
            app: self.app().as_str().to_string(),
            path: path.to_string(),
            exists: true,
            format: "json".to_string(),
            priority,
            content: Some(content),
        };
        let normalized = match normalize(content) {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![source(content.to_string())],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![error],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![source(normalized)],
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
        managed_json_field_writes(ctx, FIELD, self.app(), config, previous_config)
    }
}

#[cfg(test)]
mod tests {
    use super::ZedAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
    use std::fs;
    use std::path::PathBuf;

    fn ctx(home: PathBuf) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home.clone(),
            workspace_root: home,
        }
    }

    const SETTINGS: &str = r#"// Zed settings
{
  "theme": "One Dark",
  "context_servers": {
    // Installed from the extension store.
    "mcp-server-github": {
      "settings": { "github_personal_access_token": "t" },
    },
    "legacy": {
      "source": "custom",
      "command": { "path": "npx", "args": ["-y", "legacy-mcp"], "env": {} },
    },
    "remote": {
      "url": "https://mcp.example.com/mcp",
      "headers": { "Authorization": "Bearer fixture-token" },
      "timeout": 30,
    },
  },
  "vim_mode": true,
}
"#;

    #[test]
    fn imports_flat_and_legacy_servers_but_not_extension_ones() {
        let parsed = ZedAdapter.parse_source(
            &ctx(PathBuf::from("/Users/test")),
            "/Users/test/.config/zed/settings.json",
            20,
            SETTINGS,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let mut ids = parsed
            .servers
            .iter()
            .map(|(server, _)| server.id.as_str())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, ["legacy", "remote"]);
        let legacy = &parsed
            .servers
            .iter()
            .find(|(s, _)| s.id == "legacy")
            .unwrap()
            .0;
        assert_eq!(legacy.command.as_ref().unwrap().program, "npx");
        assert_eq!(legacy.command.as_ref().unwrap().args, ["-y", "legacy-mcp"]);
        let remote = &parsed
            .servers
            .iter()
            .find(|(s, _)| s.id == "remote")
            .unwrap()
            .0;
        assert_eq!(
            remote.transport.headers["Authorization"],
            "Bearer fixture-token"
        );
    }

    #[test]
    fn apply_rewrites_its_servers_and_leaves_the_settings_file_alone() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(home.path().to_path_buf());
        let path = ctx.user_app_config_path(SupportedApp::Zed);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, SETTINGS).unwrap();

        let parsed = ZedAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, SETTINGS);
        let mut servers = parsed
            .servers
            .into_iter()
            .map(|(server, _)| server)
            .collect::<Vec<_>>();
        // A new server, and the legacy one moved to a new package.
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Zed, true);
        servers.push(MCPServer {
            id: "fresh".to_string(),
            name: "fresh".to_string(),
            enabled: true,
            transport: TransportSpec {
                kind: "stdio".to_string(),
                url: None,
                headers: Default::default(),
            },
            command: Some(crate::core::CommandSpec {
                program: "uvx".to_string(),
                args: vec!["fresh-mcp".to_string()],
                env: Default::default(),
            }),
            apps,
            placements: vec![],
            description: None,
            homepage: None,
        });
        for server in &mut servers {
            if server.id == "legacy" {
                server.command.as_mut().unwrap().args = vec!["-y".into(), "legacy-mcp@2".into()];
            }
        }
        let config = MCPConfig {
            version: 1,
            servers,
        };
        for operation in ZedAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&PathBuf::from(&operation.path), &operation).expect("apply");
        }

        let written = fs::read_to_string(&path).unwrap();
        // Everything outside the managed entries is untouched.
        for kept in [
            "// Zed settings",
            "\"theme\": \"One Dark\"",
            "// Installed from the extension store.",
            "\"github_personal_access_token\": \"t\"",
            "\"vim_mode\": true,",
        ] {
            assert!(written.contains(kept), "{kept} missing from\n{written}");
        }
        let settings = crate::parser::parse_jsonc(&written).expect("valid JSONC");
        let servers = &settings["context_servers"];
        // Legacy entry flattened to the current format, without `source`.
        assert_eq!(
            servers["legacy"],
            serde_json::json!({"command": "npx", "args": ["-y", "legacy-mcp@2"], "env": {}})
        );
        assert_eq!(servers["fresh"]["command"], "uvx");
        assert!(servers["fresh"].get("type").is_none());
        // Remote keeps its hand-set timeout and has no `type`.
        assert_eq!(servers["remote"]["timeout"], 30);
        assert_eq!(servers["remote"]["url"], "https://mcp.example.com/mcp");
        assert!(servers["remote"].get("type").is_none());
    }
}
