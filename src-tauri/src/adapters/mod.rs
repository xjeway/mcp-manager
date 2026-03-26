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
mod qwen_code;
mod vscode;
mod windsurf;

use crate::core::{LocalConfigSource, MCPConfig, MCPServer, SupportedApp, WriteOperation};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};

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
    fn plan_apply(&self, ctx: &PlatformContext, config: &MCPConfig) -> WriteOperation;
}

pub fn standard_mcp_servers(config: &MCPConfig, app: SupportedApp) -> Value {
    let mut servers = Map::new();
    for server in &config.servers {
        if server.enabled && server.apps.get(&app).copied().unwrap_or(false) {
            let value = if server.transport.kind == "stdio" {
                serde_json::json!({
                    "command": server.command.as_ref().map(|c| c.program.clone()).unwrap_or_default(),
                    "args": server.command.as_ref().map(|c| c.args.clone()).unwrap_or_default(),
                    "env": server.command.as_ref().map(|c| c.env.clone()).unwrap_or_default()
                })
            } else {
                serde_json::json!({ "url": server.transport.url.clone().unwrap_or_default() })
            };
            servers.insert(server.id.clone(), value);
        }
    }
    Value::Object(servers)
}

pub fn opencode_mcp_servers(config: &MCPConfig, app: SupportedApp) -> Value {
    let mut servers = Map::new();
    for server in &config.servers {
        if server.enabled && server.apps.get(&app).copied().unwrap_or(false) {
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
    ]
}
