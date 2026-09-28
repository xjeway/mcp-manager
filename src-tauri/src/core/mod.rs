use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportedApp {
    Vscode,
    Cursor,
    ClaudeCode,
    ClaudeDesktop,
    Codex,
    OpenCode,
    GithubCopilot,
    GeminiCli,
    Antigravity,
    IFlow,
    QwenCode,
    Cline,
    Windsurf,
    Kiro,
}

impl SupportedApp {
    pub const ALL: [SupportedApp; 14] = [
        SupportedApp::Vscode,
        SupportedApp::Cursor,
        SupportedApp::ClaudeCode,
        SupportedApp::ClaudeDesktop,
        SupportedApp::Codex,
        SupportedApp::OpenCode,
        SupportedApp::GithubCopilot,
        SupportedApp::GeminiCli,
        SupportedApp::Antigravity,
        SupportedApp::IFlow,
        SupportedApp::QwenCode,
        SupportedApp::Cline,
        SupportedApp::Windsurf,
        SupportedApp::Kiro,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            SupportedApp::Vscode => "vscode",
            SupportedApp::Cursor => "cursor",
            SupportedApp::ClaudeCode => "claudeCode",
            SupportedApp::ClaudeDesktop => "claudeDesktop",
            SupportedApp::Codex => "codex",
            SupportedApp::OpenCode => "openCode",
            SupportedApp::GithubCopilot => "githubCopilot",
            SupportedApp::GeminiCli => "geminiCli",
            SupportedApp::Antigravity => "antigravity",
            SupportedApp::IFlow => "iFlow",
            SupportedApp::QwenCode => "qwenCode",
            SupportedApp::Cline => "cline",
            SupportedApp::Windsurf => "windsurf",
            SupportedApp::Kiro => "kiro",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlacementScope {
    User,
    Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPlacement {
    pub app: SupportedApp,
    pub scope: PlacementScope,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub enabled: bool,
    #[serde(default = "default_managed")]
    pub managed: bool,
}

fn default_managed() -> bool {
    true
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
    #[serde(default)]
    pub placements: Vec<ServerPlacement>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_keys: Option<Vec<String>>,
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
        (SupportedApp::ClaudeDesktop, false),
        (SupportedApp::Codex, false),
        (SupportedApp::OpenCode, false),
        (SupportedApp::GithubCopilot, false),
        (SupportedApp::GeminiCli, false),
        (SupportedApp::Antigravity, false),
        (SupportedApp::IFlow, false),
        (SupportedApp::QwenCode, false),
        (SupportedApp::Cline, false),
        (SupportedApp::Windsurf, false),
        (SupportedApp::Kiro, false),
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
        placements: server.placements.clone(),
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

    for placement in &incoming.placements {
        let existing = merged.placements.iter_mut().find(|current| {
            current.app == placement.app
                && current.scope == placement.scope
                && current.path == placement.path
        });
        match existing {
            Some(existing) => {
                existing.enabled = existing.enabled || placement.enabled;
                existing.managed = existing.managed || placement.managed;
            }
            None => merged.placements.push(placement.clone()),
        }
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

#[cfg(test)]
mod tests {
    use super::{
        empty_apps, merge_servers, MCPServer, PlacementScope, ServerPlacement, SupportedApp,
        TransportSpec,
    };

    fn server_with_placement(enabled: bool) -> MCPServer {
        MCPServer {
            description: None,
            homepage: None,
            id: "playwright".to_string(),
            name: "Playwright".to_string(),
            enabled: true,
            transport: TransportSpec {
                kind: "stdio".to_string(),
                url: None,
            },
            command: None,
            apps: empty_apps(),
            placements: vec![ServerPlacement {
                app: SupportedApp::Vscode,
                scope: PlacementScope::Workspace,
                path: Some("/workspace/project/.vscode/mcp.json".to_string()),
                enabled,
                managed: true,
            }],
        }
    }

    #[test]
    fn reimport_reenables_an_existing_disabled_placement() {
        let merged = merge_servers(
            Some(&server_with_placement(false)),
            &server_with_placement(true),
        );

        assert_eq!(merged.placements.len(), 1);
        assert!(merged.placements[0].enabled);
    }
}
