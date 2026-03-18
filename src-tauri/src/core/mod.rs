use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportedApp {
    Vscode,
    Cursor,
    ClaudeCode,
    Codex,
}

impl SupportedApp {
    pub const ALL: [SupportedApp; 4] = [
        SupportedApp::Vscode,
        SupportedApp::Cursor,
        SupportedApp::ClaudeCode,
        SupportedApp::Codex,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            SupportedApp::Vscode => "vscode",
            SupportedApp::Cursor => "cursor",
            SupportedApp::ClaudeCode => "claudeCode",
            SupportedApp::Codex => "codex",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub transport: TransportSpec,
    #[serde(default)]
    pub command: Option<CommandSpec>,
    pub apps: HashMap<SupportedApp, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub version: u32,
    pub servers: Vec<MCPServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfigSource {
    pub app: String,
    pub path: String,
    pub exists: bool,
    pub format: String,
    pub priority: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub config: MCPConfig,
    pub sources: Vec<LocalConfigSource>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOperation {
    pub path: String,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub backups: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedServer {
    pub server: MCPServer,
    pub priority: u32,
}

pub fn empty_apps() -> HashMap<SupportedApp, bool> {
    HashMap::from([
        (SupportedApp::Vscode, false),
        (SupportedApp::Cursor, false),
        (SupportedApp::ClaudeCode, false),
        (SupportedApp::Codex, false),
    ])
}

fn clone_server(server: &MCPServer) -> MCPServer {
    MCPServer {
        description: server.description.clone(),
        homepage: server.homepage.clone(),
        id: server.id.clone(),
        name: server.name.clone(),
        enabled: server.enabled,
        transport: server.transport.clone(),
        command: server.command.clone(),
        apps: server.apps.clone(),
    }
}

pub fn merge_servers(current: Option<&MCPServer>, incoming: &MCPServer) -> MCPServer {
    let Some(current) = current else {
        return clone_server(incoming);
    };

    let mut merged = clone_server(current);
    if merged.name.is_empty() {
        merged.name = incoming.name.clone();
    }
    if merged.description.is_none() && incoming.description.is_some() {
        merged.description = incoming.description.clone();
    }
    if merged.homepage.is_none() && incoming.homepage.is_some() {
        merged.homepage = incoming.homepage.clone();
    }
    merged.enabled = merged.enabled || incoming.enabled;

    for app in SupportedApp::ALL {
        let enabled = merged.apps.get(&app).copied().unwrap_or(false)
            || incoming.apps.get(&app).copied().unwrap_or(false);
        merged.apps.insert(app, enabled);
    }

    if merged.command.is_none() && incoming.command.is_some() {
        merged.command = incoming.command.clone();
    }

    if merged.transport.url.is_none() && incoming.transport.url.is_some() {
        merged.transport = incoming.transport.clone();
    }

    merged
}

pub fn build_import_result(
    detected_sources: Vec<LocalConfigSource>,
    detected_servers: Vec<DetectedServer>,
    warnings: Vec<String>,
    errors: Vec<String>,
) -> ImportResult {
    let mut merged: HashMap<String, MCPServer> = HashMap::new();

    let mut ordered = detected_servers;
    ordered.sort_by_key(|item| item.priority);

    for detected in ordered {
        let existing = merged.get(&detected.server.id);
        let next = merge_servers(existing, &detected.server);
        merged.insert(next.id.clone(), next);
    }

    let mut servers = merged.into_values().collect::<Vec<_>>();
    servers.sort_by(|a, b| a.id.cmp(&b.id));

    ImportResult {
        config: MCPConfig {
            version: 1,
            servers,
        },
        sources: detected_sources,
        warnings,
        errors,
    }
}
