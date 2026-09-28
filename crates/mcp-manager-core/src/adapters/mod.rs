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
    LocalConfigSource, MCPConfig, MCPServer, PlacementScope, SupportedApp, WriteOperation,
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
}

fn standard_json_app_profile(app: SupportedApp) -> StandardJsonAppProfile {
    match app {
        SupportedApp::GithubCopilot => StandardJsonAppProfile {
            include_transport_type: true,
            tools: Some(&["*"]),
        },
        _ => StandardJsonAppProfile {
            include_transport_type: true,
            tools: None,
        },
    }
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
                serde_json::json!({
                    "type": "remote",
                    "enabled": true,
                    "url": server.transport.url.clone().unwrap_or_default(),
                })
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
