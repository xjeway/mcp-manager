use crate::adapters::{
    goose_extensions_at, managed_yaml_mapping_writes, scope_for_path, AppAdapter, ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};

pub struct GooseAdapter;

const FIELD: &str = "extensions";

/// The MCP servers among Goose's `extensions`, as an `mcpServers` document the
/// shared parser reads.
///
/// Built-in and platform extensions are not MCP servers MCP Manager can
/// manage; they are left out, so an apply never touches them. Goose's
/// `enabled` switch is Goose's own: a server turned off there is not read as
/// off everywhere, and the next apply keeps the switch as it is.
fn normalize(content: &str) -> Result<String, String> {
    let root: Value = if content.trim().is_empty() {
        Value::Null
    } else {
        serde_yaml::from_str(content).map_err(|e| e.to_string())?
    };
    let mut servers = Map::new();
    let entries = root
        .get(FIELD)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (key, entry) in entries {
        let mut server = Map::new();
        match entry.get("type").and_then(Value::as_str) {
            Some("stdio") => {
                let Some(cmd) = entry.get("cmd") else {
                    continue;
                };
                server.insert("command".into(), cmd.clone());
                if let Some(args) = entry.get("args") {
                    server.insert("args".into(), args.clone());
                }
                if let Some(envs) = entry.get("envs").or_else(|| entry.get("env")) {
                    server.insert("env".into(), envs.clone());
                }
            }
            Some("streamable_http") => {
                let Some(uri) = entry.get("uri") else {
                    continue;
                };
                server.insert("url".into(), uri.clone());
                server.insert("type".into(), Value::from("http"));
                if let Some(headers) = entry.get("headers") {
                    server.insert("headers".into(), headers.clone());
                }
            }
            _ => continue,
        }
        if let Some(name) = entry.get("name") {
            server.insert("name".into(), name.clone());
        }
        if let Some(description) = entry.get("description").filter(|value| value != &"") {
            server.insert("description".into(), description.clone());
        }
        servers.insert(key, Value::Object(server));
    }
    let mut document = Map::new();
    document.insert("mcpServers".to_string(), Value::Object(servers));
    serde_json::to_string(&Value::Object(document)).map_err(|e| e.to_string())
}

impl AppAdapter for GooseAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Goose
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![(
            ctx.user_app_config_path(self.app())
                .to_string_lossy()
                .to_string(),
            20,
        )]
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
        managed_yaml_mapping_writes(
            ctx,
            FIELD,
            self.app(),
            config,
            previous_config,
            goose_extensions_at,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::GooseAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::PathBuf;

    const CONFIG: &str = "GOOSE_PROVIDER: anthropic
GOOSE_MODEL: claude-sonnet  # default model
extensions:
  developer:
    bundled: true
    display_name: Developer
    enabled: true
    name: developer
    timeout: 300
    type: builtin
  github:
    args:
    - -y
    - '@modelcontextprotocol/server-github'
    cmd: npx
    description: GitHub tools
    enabled: false
    env_keys:
    - GITHUB_PERSONAL_ACCESS_TOKEN
    envs: {}
    name: GitHub
    timeout: 300
    type: stdio
  remote:
    enabled: true
    name: remote
    type: streamable_http
    uri: https://mcp.example.com/mcp
    headers:
      X-Team: core
";

    fn ctx(home: PathBuf) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home.clone(),
            workspace_root: home,
        }
    }

    #[test]
    fn imports_mcp_extensions_only() {
        let parsed = GooseAdapter.parse_source(
            &ctx(PathBuf::from("/Users/test")),
            "/Users/test/.config/goose/config.yaml",
            20,
            CONFIG,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let mut ids = parsed
            .servers
            .iter()
            .map(|(server, _)| server.id.as_str())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, ["github", "remote"]);
        let github = &parsed
            .servers
            .iter()
            .find(|(s, _)| s.id == "github")
            .unwrap()
            .0;
        assert!(github.enabled, "turned off in Goose only");
        assert_eq!(github.name, "GitHub");
        assert_eq!(github.command.as_ref().unwrap().program, "npx");
        let remote = &parsed
            .servers
            .iter()
            .find(|(s, _)| s.id == "remote")
            .unwrap()
            .0;
        assert_eq!(remote.transport.headers["X-Team"], "core");
    }

    #[test]
    fn apply_keeps_builtins_goose_switches_and_secrets() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(home.path().to_path_buf());
        let path = ctx.user_app_config_path(SupportedApp::Goose);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, CONFIG).unwrap();

        let parsed = GooseAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, CONFIG);
        let mut servers = parsed
            .servers
            .into_iter()
            .map(|(server, _)| server)
            .collect::<Vec<_>>();
        for server in &mut servers {
            if server.id == "github" {
                server.command.as_mut().unwrap().args =
                    vec!["-y".into(), "@modelcontextprotocol/server-github@2".into()];
            }
        }
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Goose, true);
        let remote = |id: &str, name: &str, kind: &str| MCPServer {
            id: id.into(),
            name: name.into(),
            enabled: true,
            transport: TransportSpec {
                kind: kind.into(),
                url: Some(format!("https://{id}.example.com/mcp")),
                headers: Default::default(),
            },
            command: None,
            apps: apps.clone(),
            placements: vec![],
            description: None,
            homepage: None,
        };
        servers.push(remote("context7", "Context7", "http"));
        servers.push(remote("my-docs", "Docs!", "http"));
        servers.push(remote("old-sse", "old-sse", "sse"));
        let config = MCPConfig {
            version: 1,
            servers,
        };
        for operation in GooseAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&PathBuf::from(&operation.path), &operation).expect("apply");
        }

        let written = fs::read_to_string(&path).unwrap();
        assert!(
            written.starts_with(
                "GOOSE_PROVIDER: anthropic\nGOOSE_MODEL: claude-sonnet  # default model\n"
            ),
            "{written}"
        );
        let parsed: Value = serde_yaml::from_str(&written).unwrap();
        let extensions = &parsed["extensions"];
        // The built-in extension is untouched.
        assert_eq!(extensions["developer"]["type"], "builtin");
        // Goose's own switch and keyring secret names stay.
        let github = &extensions["github"];
        assert_eq!(github["enabled"], false);
        assert_eq!(github["env_keys"], json!(["GITHUB_PERSONAL_ACCESS_TOKEN"]));
        assert_eq!(github["name"], "GitHub");
        assert_eq!(github["args"][1], "@modelcontextprotocol/server-github@2");
        assert_eq!(github["description"], "GitHub tools");
        // New entries are on, typed, and named so Goose maps them to their key.
        assert_eq!(
            extensions["context7"],
            json!({"enabled": true, "name": "Context7", "type": "streamable_http", "uri": "https://context7.example.com/mcp"})
        );
        assert_eq!(extensions["my-docs"]["name"], "my-docs");
        // Goose has no SSE client.
        assert!(extensions.get("old-sse").is_none());
    }
}
