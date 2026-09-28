//! Decides, for each wizard step, whether flags and context already answer it
//! or the user has to be asked, and with which defaults. No I/O here, so the
//! rules can be tested directly; `ui` renders the questions.

use mcp_manager_core::core::SupportedApp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision<T> {
    /// Settled without asking; the note, if any, tells the user what was chosen.
    /// Single-choice steps hold exactly one value.
    Use(Vec<T>, Option<String>),
    Ask {
        options: Vec<T>,
        preselected: Vec<T>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    User,
    Project,
}

/// Which of the servers found in the source to add.
pub fn servers(candidates: &[String], requested: &[String]) -> Result<Decision<String>, String> {
    if !requested.is_empty() {
        let unknown = requested
            .iter()
            .filter(|id| !candidates.contains(id))
            .cloned()
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            return Err(format!(
                "No server named {} in the source. Available: {}",
                unknown.join(", "),
                candidates.join(", ")
            ));
        }
        return Ok(Decision::Use(requested.to_vec(), None));
    }

    match candidates {
        [] => Err("The source does not define any MCP server.".to_string()),
        [only] => Ok(Decision::Use(vec![only.clone()], None)),
        _ => Ok(Decision::Ask {
            options: candidates.to_vec(),
            preselected: candidates.to_vec(),
        }),
    }
}

/// Which clients to install into.
///
/// Explicit `--app` wins, then the agent the CLI runs inside. A single
/// installed client is used without asking. Several are offered with the ones
/// picked last time (or all of them) preselected; with none detected every
/// client is offered and nothing is preselected.
pub fn apps(
    requested: Option<Vec<SupportedApp>>,
    agent_app: Option<SupportedApp>,
    detected: &[SupportedApp],
    last: &[SupportedApp],
) -> Decision<SupportedApp> {
    if let Some(requested) = requested {
        return Decision::Use(requested, None);
    }
    if let Some(app) = agent_app {
        return Decision::Use(vec![app], None);
    }

    match detected {
        [only] => Decision::Use(
            vec![*only],
            Some(format!(
                "Adding to {} (the only client found)",
                only.display_name()
            )),
        ),
        [] => Decision::Ask {
            options: SupportedApp::ALL.to_vec(),
            preselected: vec![],
        },
        _ => {
            let remembered = detected
                .iter()
                .copied()
                .filter(|app| last.contains(app))
                .collect::<Vec<_>>();
            let mut options = detected.to_vec();
            options.extend(
                SupportedApp::ALL
                    .into_iter()
                    .filter(|app| !detected.contains(app)),
            );
            Decision::Ask {
                options,
                preselected: if remembered.is_empty() {
                    detected.to_vec()
                } else {
                    remembered
                },
            }
        }
    }
}

/// User-level config or the current project's. Only asked when there is a
/// project and one of the chosen clients has a project-level config.
pub fn scope(
    requested: Option<Scope>,
    has_project: bool,
    any_app_supports_project: bool,
) -> Result<Decision<Scope>, String> {
    match requested {
        Some(Scope::Project) if !has_project => Err(
            "--project needs to run inside a project directory (not your home directory)."
                .to_string(),
        ),
        Some(Scope::Project) if !any_app_supports_project => Err(
            "None of the chosen clients has a project-level MCP config; use --global.".to_string(),
        ),
        Some(scope) => Ok(Decision::Use(vec![scope], None)),
        None if has_project && any_app_supports_project => Ok(Decision::Ask {
            options: vec![Scope::User, Scope::Project],
            preselected: vec![Scope::User],
        }),
        None => Ok(Decision::Use(vec![Scope::User], None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SupportedApp::{ClaudeCode, Codex, Cursor};

    fn ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn a_single_server_is_taken_without_asking() {
        assert_eq!(
            servers(&ids(&["github"]), &[]).unwrap(),
            Decision::Use(ids(&["github"]), None)
        );
    }

    #[test]
    fn several_servers_are_offered_all_preselected() {
        assert_eq!(
            servers(&ids(&["a", "b"]), &[]).unwrap(),
            Decision::Ask {
                options: ids(&["a", "b"]),
                preselected: ids(&["a", "b"])
            }
        );
    }

    #[test]
    fn requested_servers_must_exist() {
        assert_eq!(
            servers(&ids(&["a", "b"]), &ids(&["b"])).unwrap(),
            Decision::Use(ids(&["b"]), None)
        );
        let error = servers(&ids(&["a"]), &ids(&["x"])).unwrap_err();
        assert!(
            error.contains("x") && error.contains("Available: a"),
            "{error}"
        );
    }

    #[test]
    fn explicit_apps_beat_the_agent_and_detection() {
        assert_eq!(
            apps(Some(vec![Codex]), Some(ClaudeCode), &[Cursor], &[]),
            Decision::Use(vec![Codex], None)
        );
        assert_eq!(
            apps(None, Some(ClaudeCode), &[Cursor, Codex], &[]),
            Decision::Use(vec![ClaudeCode], None)
        );
    }

    #[test]
    fn a_single_detected_client_is_used_with_a_note() {
        let Decision::Use(chosen, Some(note)) = apps(None, None, &[Cursor], &[]) else {
            panic!("expected a settled choice with a note");
        };
        assert_eq!(chosen, vec![Cursor]);
        assert!(note.contains("Cursor"));
    }

    #[test]
    fn several_detected_clients_come_first_with_last_choice_preselected() {
        let Decision::Ask {
            options,
            preselected,
        } = apps(None, None, &[Cursor, Codex], &[Codex, ClaudeCode])
        else {
            panic!("expected a question");
        };
        assert_eq!(&options[..2], &[Cursor, Codex]);
        assert_eq!(options.len(), SupportedApp::ALL.len());
        assert_eq!(preselected, vec![Codex]);

        let Decision::Ask { preselected, .. } = apps(None, None, &[Cursor, Codex], &[]) else {
            panic!("expected a question");
        };
        assert_eq!(preselected, vec![Cursor, Codex]);
    }

    #[test]
    fn nothing_detected_offers_everything_unselected() {
        assert_eq!(
            apps(None, None, &[], &[Cursor]),
            Decision::Ask {
                options: SupportedApp::ALL.to_vec(),
                preselected: vec![]
            }
        );
    }

    #[test]
    fn scope_is_only_asked_inside_a_project_with_project_capable_clients() {
        assert_eq!(
            scope(None, true, true).unwrap(),
            Decision::Ask {
                options: vec![Scope::User, Scope::Project],
                preselected: vec![Scope::User]
            }
        );
        assert_eq!(
            scope(None, false, true).unwrap(),
            Decision::Use(vec![Scope::User], None)
        );
        assert_eq!(
            scope(None, true, false).unwrap(),
            Decision::Use(vec![Scope::User], None)
        );
        assert!(scope(Some(Scope::Project), false, true).is_err());
        assert!(scope(Some(Scope::Project), true, false).is_err());
        assert_eq!(
            scope(Some(Scope::Project), true, true).unwrap(),
            Decision::Use(vec![Scope::Project], None)
        );
    }
}
