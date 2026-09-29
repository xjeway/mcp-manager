use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

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
    Qoder,
}

impl SupportedApp {
    pub const ALL: [SupportedApp; 15] = [
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
        SupportedApp::Qoder,
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
            SupportedApp::Qoder => "qoder",
        }
    }

    /// The product name, as the app's client list shows it.
    pub fn display_name(self) -> &'static str {
        match self {
            SupportedApp::Vscode => "VS Code",
            SupportedApp::Cursor => "Cursor",
            SupportedApp::ClaudeCode => "Claude Code",
            SupportedApp::ClaudeDesktop => "Claude Desktop",
            SupportedApp::Codex => "Codex",
            SupportedApp::OpenCode => "OpenCode",
            SupportedApp::GithubCopilot => "GitHub Copilot",
            SupportedApp::GeminiCli => "Gemini CLI",
            SupportedApp::Antigravity => "Antigravity",
            SupportedApp::IFlow => "iFlow",
            SupportedApp::QwenCode => "Qwen Code",
            SupportedApp::Cline => "Cline",
            SupportedApp::Windsurf => "Windsurf",
            SupportedApp::Kiro => "Kiro",
            SupportedApp::Qoder => "Qoder",
        }
    }

    /// Id for command lines, e.g. `claude-code`.
    pub fn cli_id(self) -> &'static str {
        match self {
            SupportedApp::Vscode => "vscode",
            SupportedApp::Cursor => "cursor",
            SupportedApp::ClaudeCode => "claude-code",
            SupportedApp::ClaudeDesktop => "claude-desktop",
            SupportedApp::Codex => "codex",
            SupportedApp::OpenCode => "opencode",
            SupportedApp::GithubCopilot => "github-copilot",
            SupportedApp::GeminiCli => "gemini-cli",
            SupportedApp::Antigravity => "antigravity",
            SupportedApp::IFlow => "iflow",
            SupportedApp::QwenCode => "qwen-code",
            SupportedApp::Cline => "cline",
            SupportedApp::Windsurf => "windsurf",
            SupportedApp::Kiro => "kiro",
            SupportedApp::Qoder => "qoder",
        }
    }

    /// Accepts the kebab-case id, the camelCase id, or the display name, ignoring case.
    pub fn parse(value: &str) -> Option<SupportedApp> {
        let key = |text: &str| {
            text.chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase()
        };
        let wanted = key(value);
        SupportedApp::ALL.into_iter().find(|app| {
            [app.cli_id(), app.as_str(), app.display_name()]
                .into_iter()
                .any(|name| key(name) == wanted)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub url: Option<String>,
    /// Request headers for remote transports; ignored for stdio.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
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

/// A non-blocking problem found while applying, e.g. a setting a client cannot store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyWarning {
    pub kind: String,
    pub app: SupportedApp,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub backups: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<ApplyWarning>,
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
        (SupportedApp::Qoder, false),
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
    } else if merged.transport.headers.is_empty() && merged.transport.url == incoming.transport.url
    {
        merged.transport.headers = incoming.transport.headers.clone();
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
    #[test]
    fn cli_ids_round_trip_through_parse() {
        use super::SupportedApp;
        for app in SupportedApp::ALL {
            assert_eq!(SupportedApp::parse(app.cli_id()), Some(app));
            assert_eq!(SupportedApp::parse(app.as_str()), Some(app));
            assert_eq!(SupportedApp::parse(app.display_name()), Some(app));
        }
        assert_eq!(SupportedApp::ClaudeCode.cli_id(), "claude-code");
        assert_eq!(SupportedApp::IFlow.cli_id(), "iflow");
        assert_eq!(SupportedApp::parse("iflow"), Some(SupportedApp::IFlow));
        assert_eq!(SupportedApp::parse("nope"), None);
    }

    use super::{
        empty_apps, merge_servers, MCPConfig, MCPServer, PlacementScope, ServerPlacement,
        SupportedApp, TransportSpec,
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
                headers: Default::default(),
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

    fn http_server(url: &str, headers: &[(&str, &str)]) -> MCPServer {
        MCPServer {
            transport: TransportSpec {
                kind: "http".to_string(),
                url: Some(url.to_string()),
                headers: headers
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
            },
            ..server_with_placement(true)
        }
    }

    #[test]
    fn loads_legacy_yaml_without_headers() {
        let config: MCPConfig = serde_yaml::from_str(
            "version: 1\nservers:\n  - id: linear\n    name: Linear\n    enabled: true\n    transport:\n      type: http\n      url: https://mcp.linear.app/mcp\n    apps: {}\n",
        )
        .expect("legacy yaml");
        assert!(config.servers[0].transport.headers.is_empty());
    }

    #[test]
    fn serializes_headers_only_when_present() {
        let plain = serde_json::to_value(http_server("https://x/mcp", &[])).expect("json");
        assert!(plain["transport"].get("headers").is_none());

        let with_headers = serde_json::to_value(http_server(
            "https://x/mcp",
            &[("Authorization", "Bearer t")],
        ))
        .expect("json");
        assert_eq!(
            with_headers["transport"]["headers"]["Authorization"],
            "Bearer t"
        );
    }

    #[test]
    fn merge_fills_headers_for_the_same_url() {
        let merged = merge_servers(
            Some(&http_server("https://x/mcp", &[])),
            &http_server("https://x/mcp", &[("Authorization", "Bearer t")]),
        );
        assert_eq!(
            merged
                .transport
                .headers
                .get("Authorization")
                .map(String::as_str),
            Some("Bearer t")
        );
    }

    #[test]
    fn merge_keeps_own_headers_and_ignores_other_urls() {
        let merged = merge_servers(
            Some(&http_server("https://x/mcp", &[("Authorization", "mine")])),
            &http_server("https://x/mcp", &[("Authorization", "theirs")]),
        );
        assert_eq!(merged.transport.headers["Authorization"], "mine");

        let merged = merge_servers(
            Some(&http_server("https://x/mcp", &[])),
            &http_server("https://other/mcp", &[("Authorization", "theirs")]),
        );
        assert!(merged.transport.headers.is_empty());
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
