mod antigravity;
mod claude_code;
mod claude_desktop;
mod cline;
mod codex;
mod cursor;
mod gemini_cli;
mod github_copilot;
mod iflow;
mod kiro;
mod opencode;
mod qoder;
mod qwen_code;
mod vscode;
mod windsurf;

use crate::core::{
    ApplyWarning, LocalConfigSource, MCPConfig, MCPServer, PlacementScope, SupportedApp,
    WriteOperation,
};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub use antigravity::AntigravityAdapter;
pub use claude_code::ClaudeCodeAdapter;
pub use claude_desktop::ClaudeDesktopAdapter;
pub use cline::ClineAdapter;
pub use codex::CodexAdapter;
pub use cursor::CursorAdapter;
pub use gemini_cli::GeminiCliAdapter;
pub use github_copilot::GithubCopilotAdapter;
pub use iflow::IFlowAdapter;
pub use kiro::KiroAdapter;
pub use opencode::OpenCodeAdapter;
pub use qoder::QoderAdapter;
pub use qwen_code::QwenCodeAdapter;
pub use vscode::VSCodeAdapter;
pub use windsurf::WindsurfAdapter;

pub struct ParsedSources {
    pub sources: Vec<LocalConfigSource>,
    pub servers: Vec<(MCPServer, u32)>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub trait AppAdapter {
    fn app(&self) -> SupportedApp;
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)>;
    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources;
    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous_config: Option<&MCPConfig>,
    ) -> Vec<WriteOperation>;
}

fn user_scope_path(ctx: &PlatformContext, app: SupportedApp) -> String {
    ctx.user_app_config_path(app).to_string_lossy().to_string()
}

/// The project-level config file for `app`, or `None` when the app has no
/// project scope or there is no current project.
pub fn workspace_scope_path(ctx: &PlatformContext, app: SupportedApp) -> Option<String> {
    if !ctx.has_workspace() {
        return None;
    }

    let relative = match app {
        SupportedApp::Vscode => ".vscode/mcp.json",
        SupportedApp::Cursor => ".cursor/mcp.json",
        SupportedApp::ClaudeCode => ".mcp.json",
        SupportedApp::OpenCode => "opencode.json",
        SupportedApp::GeminiCli => ".gemini/settings.json",
        SupportedApp::IFlow => ".iflow/settings.json",
        SupportedApp::QwenCode => ".qwen/settings.json",
        SupportedApp::Kiro => ".kiro/settings/mcp.json",
        _ => return None,
    };
    Some(ctx.workspace_file(relative).to_string_lossy().to_string())
}

pub fn scope_for_path(ctx: &PlatformContext, path: &str) -> PlacementScope {
    if ctx.has_workspace() && ctx.resolve_path(path).starts_with(&ctx.workspace_root) {
        PlacementScope::Workspace
    } else {
        PlacementScope::User
    }
}

fn placement_path(
    ctx: &PlatformContext,
    app: SupportedApp,
    placement: &crate::core::ServerPlacement,
) -> Option<String> {
    placement.path.clone().or_else(|| match placement.scope {
        PlacementScope::User => Some(user_scope_path(ctx, app)),
        PlacementScope::Workspace => workspace_scope_path(ctx, app),
    })
}

fn server_enabled_for_path(
    ctx: &PlatformContext,
    server: &MCPServer,
    app: SupportedApp,
    path: &str,
) -> bool {
    let placements_at_path = server
        .placements
        .iter()
        .filter(|placement| {
            placement.app == app && placement_path(ctx, app, placement).as_deref() == Some(path)
        })
        .collect::<Vec<_>>();

    if !placements_at_path.is_empty() {
        return placements_at_path.iter().any(|placement| placement.enabled);
    }

    server.apps.get(&app).copied().unwrap_or(false) && user_scope_path(ctx, app) == path
}

fn collect_target_paths(
    ctx: &PlatformContext,
    config: &MCPConfig,
    app: SupportedApp,
    paths: &mut BTreeSet<String>,
) {
    for server in &config.servers {
        if server.apps.get(&app).copied().unwrap_or(false) {
            paths.insert(user_scope_path(ctx, app));
        }

        for placement in server
            .placements
            .iter()
            .filter(|placement| placement.app == app)
        {
            if placement.managed && placement.enabled {
                paths.extend(placement_path(ctx, app, placement));
            }
        }
    }
}

fn target_paths(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
    app: SupportedApp,
) -> Vec<String> {
    let mut paths = BTreeSet::new();
    collect_target_paths(ctx, config, app, &mut paths);
    if let Some(previous_config) = previous_config {
        collect_target_paths(ctx, previous_config, app, &mut paths);
    }
    paths.into_iter().collect()
}

fn collect_managed_server_ids(
    ctx: &PlatformContext,
    config: &MCPConfig,
    app: SupportedApp,
    path: &str,
    ids: &mut BTreeSet<String>,
) {
    for server in &config.servers {
        if server_enabled_for_path(ctx, server, app, path) {
            ids.insert(server.id.clone());
        }
    }
}

fn managed_server_ids(
    ctx: &PlatformContext,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
    app: SupportedApp,
    path: &str,
) -> Vec<String> {
    let mut ids = BTreeSet::new();
    collect_managed_server_ids(ctx, config, app, path, &mut ids);
    if let Some(previous_config) = previous_config {
        collect_managed_server_ids(ctx, previous_config, app, path, &mut ids);
    }
    ids.into_iter().collect()
}

pub fn managed_json_field_writes(
    ctx: &PlatformContext,
    field: &str,
    app: SupportedApp,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<WriteOperation> {
    managed_json_field_writes_with(
        ctx,
        field,
        app,
        config,
        previous_config,
        standard_mcp_servers_at,
    )
}

pub fn managed_json_field_writes_with(
    ctx: &PlatformContext,
    field: &str,
    app: SupportedApp,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
    content_for_path: fn(&MCPConfig, SupportedApp, &str, &PlatformContext) -> Value,
) -> Vec<WriteOperation> {
    target_paths(ctx, config, previous_config, app)
        .into_iter()
        .map(|path| WriteOperation {
            content: serde_json::to_string(&content_for_path(config, app, &path, ctx))
                .expect("serialize managed json field"),
            path: path.clone(),
            mode: "merge_json_object_entries".to_string(),
            field: Some(field.to_string()),
            remove_keys: Some(managed_server_ids(ctx, config, previous_config, app, &path)),
        })
        .collect()
}

pub fn managed_toml_field_writes(
    ctx: &PlatformContext,
    field: &str,
    app: SupportedApp,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> Vec<WriteOperation> {
    target_paths(ctx, config, previous_config, app)
        .into_iter()
        .map(|path| WriteOperation {
            content: serde_json::to_string(&standard_mcp_servers_at(config, app, &path, ctx))
                .expect("serialize managed toml field"),
            path: path.clone(),
            mode: "merge_toml_table_entries".to_string(),
            field: Some(field.to_string()),
            remove_keys: Some(managed_server_ids(ctx, config, previous_config, app, &path)),
        })
        .collect()
}

#[derive(Clone, Copy)]
struct StandardJsonAppProfile {
    include_transport_type: bool,
    tools: Option<&'static [&'static str]>,
    /// Key for remote request headers; `None` when the client cannot take them.
    headers_key: Option<&'static str>,
}

fn standard_json_app_profile(app: SupportedApp) -> StandardJsonAppProfile {
    let headers_key = match app {
        SupportedApp::Codex => Some("http_headers"),
        // claude_desktop_config.json only describes local servers; remote ones
        // (and their auth) are set up as Connectors in the app.
        SupportedApp::ClaudeDesktop => None,
        _ => Some("headers"),
    };
    StandardJsonAppProfile {
        include_transport_type: true,
        tools: match app {
            SupportedApp::GithubCopilot => Some(&["*"]),
            _ => None,
        },
        headers_key,
    }
}

/// Whether `app` can store request headers for remote servers.
pub fn http_headers_supported(app: SupportedApp) -> bool {
    standard_json_app_profile(app).headers_key.is_some()
}

/// Servers whose headers will be dropped because a client they are enabled for
/// cannot store them.
pub fn unsupported_header_warnings(config: &MCPConfig) -> Vec<ApplyWarning> {
    config
        .servers
        .iter()
        .filter(|server| server.enabled && headers_value(server).is_some())
        .flat_map(|server| {
            SupportedApp::ALL
                .into_iter()
                .filter(|app| !http_headers_supported(*app))
                .filter(move |app| {
                    server.apps.get(app).copied().unwrap_or(false)
                        || server
                            .placements
                            .iter()
                            .any(|placement| placement.app == *app && placement.enabled)
                })
                .map(move |app| ApplyWarning {
                    kind: "httpHeadersUnsupported".to_string(),
                    app,
                    server_id: server.id.clone(),
                })
        })
        .collect()
}

fn headers_value(server: &MCPServer) -> Option<Value> {
    let transport = &server.transport;
    (transport.kind != "stdio"
        && !transport.headers.is_empty()
        && !transport.sends_headers_insecurely())
    .then(|| serde_json::to_value(&server.transport.headers).expect("serialize transport headers"))
}

fn standard_json_transport_type(kind: &str) -> &'static str {
    match kind {
        "http" => "http",
        "sse" => "sse",
        "streamable-http" => "streamable-http",
        _ => "stdio",
    }
}

pub fn standard_mcp_servers_at(
    config: &MCPConfig,
    app: SupportedApp,
    path: &str,
    ctx: &PlatformContext,
) -> Value {
    let mut servers = Map::new();
    let profile = standard_json_app_profile(app);

    for server in &config.servers {
        if server.enabled && server_enabled_for_path(ctx, server, app, path) {
            let mut value = if server.transport.kind == "stdio" {
                Map::from_iter([
                    (
                        "command".to_string(),
                        Value::String(
                            server
                                .command
                                .as_ref()
                                .map(|c| c.program.clone())
                                .unwrap_or_default(),
                        ),
                    ),
                    (
                        "args".to_string(),
                        Value::Array(
                            server
                                .command
                                .as_ref()
                                .map(|c| {
                                    c.args
                                        .iter()
                                        .cloned()
                                        .map(Value::String)
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default(),
                        ),
                    ),
                    (
                        "env".to_string(),
                        serde_json::to_value(
                            server
                                .command
                                .as_ref()
                                .map(|c| c.env.clone())
                                .unwrap_or_default(),
                        )
                        .expect("serialize command env"),
                    ),
                ])
            } else {
                Map::from_iter([(
                    "url".to_string(),
                    Value::String(server.transport.url.clone().unwrap_or_default()),
                )])
            };

            if profile.include_transport_type {
                value.insert(
                    "type".to_string(),
                    Value::String(standard_json_transport_type(&server.transport.kind).to_string()),
                );
            }

            if let (Some(key), Some(headers)) = (profile.headers_key, headers_value(server)) {
                value.insert(key.to_string(), headers);
            }

            if let Some(tools) = profile.tools {
                value.insert(
                    "tools".to_string(),
                    Value::Array(
                        tools
                            .iter()
                            .map(|tool| Value::String((*tool).to_string()))
                            .collect(),
                    ),
                );
            }

            servers.insert(server.id.clone(), Value::Object(value));
        }
    }
    Value::Object(servers)
}

pub fn opencode_mcp_servers_at(
    config: &MCPConfig,
    app: SupportedApp,
    path: &str,
    ctx: &PlatformContext,
) -> Value {
    let mut servers = Map::new();
    for server in &config.servers {
        if server.enabled && server_enabled_for_path(ctx, server, app, path) {
            let value = if server.transport.kind == "stdio" {
                let mut command = vec![Value::String(
                    server
                        .command
                        .as_ref()
                        .map(|c| c.program.clone())
                        .unwrap_or_default(),
                )];
                command.extend(
                    server
                        .command
                        .as_ref()
                        .map(|c| {
                            c.args
                                .iter()
                                .cloned()
                                .map(Value::String)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                );
                serde_json::json!({
                    "type": "local",
                    "enabled": true,
                    "command": command,
                    "environment": server.command.as_ref().map(|c| c.env.clone()).unwrap_or_default(),
                })
            } else {
                let mut remote = serde_json::json!({
                    "type": "remote",
                    "enabled": true,
                    "url": server.transport.url.clone().unwrap_or_default(),
                });
                if let Some(headers) = headers_value(server) {
                    remote["headers"] = headers;
                }
                remote
            };
            servers.insert(server.id.clone(), value);
        }
    }
    Value::Object(servers)
}

pub fn adapters() -> Vec<Box<dyn AppAdapter>> {
    vec![
        Box::new(VSCodeAdapter),
        Box::new(CursorAdapter),
        Box::new(ClaudeCodeAdapter),
        Box::new(CodexAdapter),
        Box::new(ClaudeDesktopAdapter),
        Box::new(OpenCodeAdapter),
        Box::new(GithubCopilotAdapter),
        Box::new(GeminiCliAdapter),
        Box::new(AntigravityAdapter),
        Box::new(IFlowAdapter),
        Box::new(QwenCodeAdapter),
        Box::new(ClineAdapter),
        Box::new(WindsurfAdapter),
        Box::new(KiroAdapter),
        Box::new(QoderAdapter),
    ]
}

/// Shared fixture round-trip for per-adapter header tests.
#[cfg(test)]
pub(crate) mod header_fixture {
    use super::AppAdapter;
    use crate::core::MCPConfig;
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    pub struct Expect {
        /// Host field holding the server entries.
        pub field: &'static str,
        /// Key the client reads headers from; `None` when it has no header support.
        pub headers_key: Option<&'static str>,
        /// A top-level host setting that apply must leave untouched.
        pub unrelated_key: &'static str,
    }

    fn read_host(path: &PathBuf) -> Value {
        let content = fs::read_to_string(path).expect("read host config");
        if path.extension().is_some_and(|ext| ext == "toml") {
            let table: toml::Table = toml::from_str(&content).expect("toml");
            serde_json::to_value(table).expect("toml to json")
        } else {
            serde_json::from_str(&content).expect("json")
        }
    }

    /// Imports `fixture` from the app's user config, changes the `remote` server's
    /// headers, applies, and checks what the client file ends up with.
    pub fn assert_round_trip(adapter: &dyn AppAdapter, fixture: &str, expect: Expect) {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: home.path().to_path_buf(),
            // Same as home: no current project.
            workspace_root: home.path().to_path_buf(),
        };
        let path = ctx.user_app_config_path(adapter.app());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, fixture).unwrap();

        let parsed = adapter.parse_source(&ctx, &path.to_string_lossy(), 20, fixture);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let mut remote = parsed
            .servers
            .iter()
            .map(|(server, _)| server.clone())
            .find(|server| server.id == "remote")
            .expect("remote server imported");
        if expect.headers_key.is_some() {
            assert_eq!(
                remote
                    .transport
                    .headers
                    .get("Authorization")
                    .map(String::as_str),
                Some("Bearer fixture-token"),
                "headers imported"
            );
            assert_eq!(remote.transport.headers["X-Api-Key"], "fixture-key");
        }
        assert!(remote.apps[&adapter.app()], "enabled for the app");

        remote.transport.headers = [("Authorization".to_string(), "Bearer rotated".to_string())]
            .into_iter()
            .collect();
        let config = MCPConfig {
            version: 1,
            servers: vec![remote],
        };
        let operations = adapter.plan_apply(&ctx, &config, None);
        assert_eq!(operations.len(), 1);
        for operation in &operations {
            apply_operation(&PathBuf::from(&operation.path), operation).expect("apply");
        }

        let host = read_host(&path);
        assert!(
            host.get(expect.unrelated_key).is_some(),
            "unrelated `{}` preserved: {host}",
            expect.unrelated_key
        );
        let entries = &host[expect.field];
        assert!(entries.get("other").is_some(), "unmanaged server preserved");
        let written = &entries["remote"];
        match expect.headers_key {
            Some(key) => {
                assert_eq!(written[key]["Authorization"], "Bearer rotated", "{written}");
                assert!(written[key].get("X-Api-Key").is_none());
            }
            None => {
                assert!(written.get("headers").is_none(), "{written}");
                assert!(written.get("http_headers").is_none(), "{written}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::unsupported_header_warnings;
    use crate::core::{
        empty_apps, ApplyWarning, MCPConfig, MCPServer, PlacementScope, ServerPlacement,
        SupportedApp, TransportSpec,
    };

    fn remote(id: &str, headers: bool, apps: &[SupportedApp]) -> MCPServer {
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
                kind: "http".to_string(),
                url: Some("https://x/mcp".to_string()),
                headers: if headers {
                    [("Authorization".to_string(), "Bearer t".to_string())].into()
                } else {
                    Default::default()
                },
            },
            command: None,
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

    #[test]
    fn headers_are_not_written_for_plain_http_remote_hosts() {
        let mut leaky = remote("leaky", true, &[SupportedApp::Vscode]);
        leaky.transport.url = Some("http://mcp.example.com/mcp".to_string());
        let mut local = remote("local", true, &[SupportedApp::Vscode]);
        local.transport.url = Some("http://localhost:3000/mcp".to_string());
        assert_eq!(super::headers_value(&leaky), None);
        assert!(super::headers_value(&local).is_some());
        assert!(super::headers_value(&remote("secure", true, &[])).is_some());
    }

    #[test]
    fn warns_when_headers_target_a_client_without_header_support() {
        let warnings = unsupported_header_warnings(&config(vec![remote(
            "linear",
            true,
            &[SupportedApp::ClaudeDesktop, SupportedApp::Vscode],
        )]));
        assert_eq!(
            warnings,
            vec![ApplyWarning {
                kind: "httpHeadersUnsupported".to_string(),
                app: SupportedApp::ClaudeDesktop,
                server_id: "linear".to_string(),
            }]
        );
        let json = serde_json::to_value(&warnings[0]).unwrap();
        assert_eq!(json["serverId"], "linear");
        assert_eq!(json["app"], "claudeDesktop");
    }

    #[test]
    fn no_warning_without_headers_or_when_disabled() {
        let mut disabled = remote("off", true, &[SupportedApp::ClaudeDesktop]);
        disabled.enabled = false;
        assert!(unsupported_header_warnings(&config(vec![
            remote("plain", false, &[SupportedApp::ClaudeDesktop]),
            disabled,
            remote("vscode-only", true, &[SupportedApp::Vscode]),
        ]))
        .is_empty());
    }

    #[test]
    fn warns_for_enabled_placements_too() {
        let mut server = remote("linear", true, &[]);
        server.placements.push(ServerPlacement {
            app: SupportedApp::ClaudeDesktop,
            scope: PlacementScope::User,
            path: None,
            enabled: true,
            managed: true,
        });
        assert_eq!(unsupported_header_warnings(&config(vec![server])).len(), 1);
    }
}
