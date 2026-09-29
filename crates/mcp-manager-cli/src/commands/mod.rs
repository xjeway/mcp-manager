pub mod add;
pub mod list;
pub mod remove;
pub mod rollback;

use crate::session::{Failure, Outcome, Session};
use mcp_manager_core::core::{MCPServer, SupportedApp};

/// Parses `--app` values; `*` means every client.
pub fn parse_apps(values: &[String]) -> Result<Option<Vec<SupportedApp>>, String> {
    if values.is_empty() {
        return Ok(None);
    }
    if values.iter().any(|value| value == "*") {
        return Ok(Some(SupportedApp::ALL.to_vec()));
    }
    let mut apps = Vec::new();
    for value in values {
        let Some(app) = SupportedApp::parse(value) else {
            let valid = SupportedApp::ALL.map(SupportedApp::cli_id).join(", ");
            return Err(format!("Unknown client {value:?}. Valid clients: {valid}"));
        };
        if !apps.contains(&app) {
            apps.push(app);
        }
    }
    Ok(Some(apps))
}

pub fn app_label(app: &SupportedApp, detected: &[SupportedApp]) -> (String, String) {
    let hint = if detected.contains(app) {
        format!("detected · {}", app.cli_id())
    } else {
        app.cli_id().to_string()
    };
    (app.display_name().to_string(), hint)
}

/// Clients the server is installed in at any scope.
pub fn installed_apps(server: &MCPServer) -> Vec<SupportedApp> {
    SupportedApp::ALL
        .into_iter()
        .filter(|app| {
            server.apps.get(app).copied().unwrap_or(false)
                || server.placements.iter().any(|p| p.enabled && p.app == *app)
        })
        .collect()
}

pub fn require_json_mode_is_non_interactive(session: &Session, dry_run: bool) -> Outcome<()> {
    if session.json && !session.yes && !dry_run {
        return Err(Failure::NeedsInput(
            "--json never prompts: add --yes to apply, or --dry-run to preview.".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_apps;
    use mcp_manager_core::core::SupportedApp;

    #[test]
    fn parses_client_ids_names_and_star() {
        let values = ["claude-code", "Cursor", "cursor"].map(String::from);
        assert_eq!(
            parse_apps(&values).unwrap(),
            Some(vec![SupportedApp::ClaudeCode, SupportedApp::Cursor])
        );
        assert_eq!(
            parse_apps(&["*".to_string()]).unwrap().unwrap().len(),
            SupportedApp::ALL.len()
        );
        assert_eq!(parse_apps(&[]).unwrap(), None);
        assert!(parse_apps(&["emacs".to_string()])
            .unwrap_err()
            .contains("claude-code"));
    }
}
