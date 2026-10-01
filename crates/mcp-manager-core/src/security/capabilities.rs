//! How each supported client can receive a secret.
//!
//! MCP Manager writes literal values. It does not rewrite a token into
//! `${GITHUB_TOKEN}` or a keychain reference, because a client that does not
//! expand those forms would receive the reference as the secret and fail, or
//! worse, send the literal `${...}` to a remote server. The table is the
//! decision record for that choice. `materialize_env` returns the value
//! unchanged for every client until a row explicitly opts into interpolation.

use crate::core::SupportedApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretSupport {
    /// The config file stores the value that the client will pass through.
    LiteralOnly,
    /// The client can expand a reference. MCP Manager still writes literals
    /// until an adapter opts in; the note says what the client accepts.
    ReferenceAvailable,
}

#[derive(Debug, Clone, Copy)]
pub struct ClientSecretCapability {
    pub app: SupportedApp,
    pub env: SecretSupport,
    pub headers: SecretSupport,
    /// No supported client config file takes an OS keychain item id today.
    pub os_keychain: bool,
    pub notes: &'static str,
}

pub const CLIENT_SECRET_SUPPORT: &[ClientSecretCapability] = &[
    ClientSecretCapability {
        app: SupportedApp::Vscode,
        env: SecretSupport::ReferenceAvailable,
        headers: SecretSupport::ReferenceAvailable,
        os_keychain: false,
        notes: "VS Code MCP config can use input variables (${input:id}) and sometimes ${env:NAME}. MCP Manager does not rewrite values into those forms.",
    },
    ClientSecretCapability {
        app: SupportedApp::Cursor,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Cursor mcp.json stores literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::ClaudeCode,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Claude Code JSON stores literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::ClaudeDesktop,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Claude Desktop does not store MCP HTTP headers; env values are literal. Header writes are skipped by the adapter.",
    },
    ClientSecretCapability {
        app: SupportedApp::Codex,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Codex TOML env and http_headers values are literal strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::OpenCode,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "OpenCode JSON stores literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::GithubCopilot,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "GitHub Copilot MCP config stores literal env and header strings. Workspace MCP is the VS Code file.",
    },
    ClientSecretCapability {
        app: SupportedApp::GeminiCli,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Gemini CLI settings store literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::Antigravity,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Antigravity config stores literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::IFlow,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "iFlow settings store literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::QwenCode,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Qwen Code settings store literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::Cline,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Cline MCP settings store literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::Windsurf,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Windsurf MCP config stores literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::Kiro,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Kiro MCP config stores literal env and header strings.",
    },
    ClientSecretCapability {
        app: SupportedApp::Qoder,
        env: SecretSupport::LiteralOnly,
        headers: SecretSupport::LiteralOnly,
        os_keychain: false,
        notes: "Qoder settings store literal env and header strings.",
    },
];

/// Returns the env value to write for `app`. Today every client receives the
/// literal; the match forces a compile break if a new app is added without a
/// decision in [`CLIENT_SECRET_SUPPORT`].
pub fn materialize_env(app: SupportedApp, value: &str) -> String {
    let support = CLIENT_SECRET_SUPPORT
        .iter()
        .find(|row| row.app == app)
        .expect("every supported app has a secret-capability row");
    match support.env {
        SecretSupport::LiteralOnly | SecretSupport::ReferenceAvailable => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_supported_app_has_one_capability_row_and_literals_are_not_rewritten() {
        let mut seen = CLIENT_SECRET_SUPPORT
            .iter()
            .map(|row| row.app)
            .collect::<Vec<_>>();
        seen.sort_by_key(|app| app.as_str());
        seen.dedup();
        assert_eq!(seen.len(), SupportedApp::ALL.len());
        for app in SupportedApp::ALL {
            assert_eq!(materialize_env(app, "ghp_literal"), "ghp_literal");
            let row = CLIENT_SECRET_SUPPORT
                .iter()
                .find(|row| row.app == app)
                .expect("row");
            assert!(!row.os_keychain);
        }
    }
}
