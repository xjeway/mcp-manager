//! Detects when the CLI runs inside an AI coding agent, which cannot answer
//! prompts. Mirrors the environment checks of `@vercel/detect-agent`, which
//! vercel-labs/skills uses for the same purpose.

use mcp_manager_core::core::SupportedApp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub name: String,
    /// The client this agent reads MCP servers from, when we manage it.
    pub app: Option<SupportedApp>,
}

pub fn detect(env: impl Fn(&str) -> Option<String>) -> Option<Agent> {
    let set = |key: &str| env(key).is_some_and(|value| !value.is_empty());

    let name = if let Some(name) = env("AI_AGENT").map(|v| v.trim().to_string()) {
        if name.is_empty() {
            return None;
        }
        if name == "github-copilot-cli" {
            "github-copilot".to_string()
        } else {
            name
        }
    } else if set("CURSOR_TRACE_ID") {
        "cursor".to_string()
    } else if set("CURSOR_AGENT")
        || env("CURSOR_EXTENSION_HOST_ROLE").as_deref() == Some("agent-exec")
    {
        "cursor-cli".to_string()
    } else if set("GEMINI_CLI") {
        "gemini".to_string()
    } else if set("CODEX_SANDBOX") || set("CODEX_CI") || set("CODEX_THREAD_ID") {
        "codex".to_string()
    } else if set("ANTIGRAVITY_AGENT") {
        "antigravity".to_string()
    } else if set("AUGMENT_AGENT") {
        "augment-cli".to_string()
    } else if set("OPENCODE_CLIENT") {
        "opencode".to_string()
    } else if set("CLAUDECODE") || set("CLAUDE_CODE") {
        if set("CLAUDE_CODE_IS_COWORK") {
            "cowork".to_string()
        } else {
            "claude".to_string()
        }
    } else if set("REPL_ID") {
        "replit".to_string()
    } else if set("COPILOT_MODEL") || set("COPILOT_ALLOW_ALL") || set("COPILOT_GITHUB_TOKEN") {
        "github-copilot".to_string()
    } else {
        return None;
    };

    let app = match name.as_str() {
        "cursor" | "cursor-cli" => Some(SupportedApp::Cursor),
        "claude" => Some(SupportedApp::ClaudeCode),
        "gemini" => Some(SupportedApp::GeminiCli),
        "codex" => Some(SupportedApp::Codex),
        "antigravity" => Some(SupportedApp::Antigravity),
        "opencode" => Some(SupportedApp::OpenCode),
        "github-copilot" => Some(SupportedApp::GithubCopilot),
        other => SupportedApp::parse(other),
    };
    Some(Agent { name, app })
}

#[cfg(test)]
mod tests {
    use super::detect;
    use mcp_manager_core::core::SupportedApp;
    use std::collections::HashMap;

    fn detect_with(vars: &[(&str, &str)]) -> Option<super::Agent> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        detect(|key| vars.get(key).cloned())
    }

    #[test]
    fn maps_known_agents_to_their_client() {
        let claude = detect_with(&[("CLAUDECODE", "1")]).expect("agent");
        assert_eq!(claude.name, "claude");
        assert_eq!(claude.app, Some(SupportedApp::ClaudeCode));

        assert_eq!(
            detect_with(&[("CODEX_THREAD_ID", "t")]).and_then(|a| a.app),
            Some(SupportedApp::Codex)
        );
        assert_eq!(
            detect_with(&[("CURSOR_EXTENSION_HOST_ROLE", "agent-exec")]).and_then(|a| a.app),
            Some(SupportedApp::Cursor)
        );
    }

    #[test]
    fn ai_agent_wins_and_unknown_agents_have_no_client() {
        let agent = detect_with(&[("AI_AGENT", "devin"), ("CLAUDECODE", "1")]).expect("agent");
        assert_eq!(agent.name, "devin");
        assert_eq!(agent.app, None);

        assert_eq!(
            detect_with(&[("AI_AGENT", "kiro")]).and_then(|a| a.app),
            Some(SupportedApp::Kiro)
        );
    }

    #[test]
    fn plain_terminals_are_not_agents() {
        assert_eq!(detect_with(&[]), None);
        assert_eq!(detect_with(&[("CLAUDECODE", "")]), None);
        assert_eq!(detect_with(&[("CURSOR_EXTENSION_HOST_ROLE", "user")]), None);
    }
}
