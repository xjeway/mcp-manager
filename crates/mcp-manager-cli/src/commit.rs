//! The shared end of every command that changes servers: preview the effect
//! on each client file, show a summary, confirm, then save and apply.

use crate::session::{Failure, Outcome, Session};
use mcp_manager_core::adapters::apply_warnings;
use mcp_manager_core::core::{ApplyWarning, MCPConfig, PlacementScope};
use mcp_manager_core::history::{self, ChangeRecord};
use mcp_manager_core::ops::{validate, Issue};
use mcp_manager_core::store;
use mcp_manager_core::workflow::{self, ChangeAction, PlannedChange};
use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

pub struct Change {
    /// servers.yaml as loaded before any question was asked.
    pub before: MCPConfig,
    pub after: MCPConfig,
    /// Servers this command is about; their unchanged entries are listed too.
    pub focus: Vec<String>,
}

#[derive(Serialize)]
struct Report<'a> {
    changes: &'a [PlannedChange],
    applied: bool,
    backups: Vec<String>,
    /// Servers a client cannot take as they are, e.g. SSE for Goose.
    warnings: &'a [ApplyWarning],
}

/// Returns whether anything was written.
pub fn commit(session: &Session, change: Change, dry_run: bool) -> Outcome<bool> {
    commit_with_confirmation(session, change, dry_run, |_| {
        session.confirm("Apply these changes?")
    })
}

/// Keep one explicit confirmation boundary before any shared-store or native-file write.
fn commit_with_confirmation(
    session: &Session,
    change: Change,
    dry_run: bool,
    confirm: impl FnOnce(&[ApplyWarning]) -> Outcome<()>,
) -> Outcome<bool> {
    commit_with_hooks(session, change, dry_run, confirm, || {})
}

/// Expose the apply/save boundary for deterministic compensation tests.
fn commit_with_hooks(
    session: &Session,
    change: Change,
    dry_run: bool,
    confirm: impl FnOnce(&[ApplyWarning]) -> Outcome<()>,
    after_apply: impl FnOnce(),
) -> Outcome<bool> {
    let blocking = validate(&change.after).blocking_errors;
    if !blocking.is_empty() {
        let lines = blocking.iter().map(describe_issue).collect::<Vec<_>>();
        return Err(Failure::Error(lines.join("\n")));
    }

    // Capture before presenting the preview; confirmation cannot authorize a
    // different active file or later edits to its contents.
    let prepared = workflow::prepare(&session.ctx, &change.after, Some(&change.before))?;
    let changes = prepared
        .changes
        .iter()
        .filter(|c| change.focus.contains(&c.server_id) || c.action != ChangeAction::Unchanged)
        .cloned()
        .collect::<Vec<_>>();
    let has_writes = changes.iter().any(|c| c.action != ChangeAction::Unchanged);
    // Reported even when nothing is written: a server a client cannot take
    // may be the reason its file does not change.
    let warnings = apply_warnings(&change.after)
        .into_iter()
        .chain(mcp_manager_core::adapters::shared_project_secret_warnings(
            &session.ctx,
            &change.after,
        ))
        .filter(|w| w.kind == "sharedProjectSecret" || change.focus.contains(&w.server_id))
        .collect::<Vec<_>>();

    if !session.json {
        for warning in &warnings {
            session.warn(describe_warning(warning))?;
        }
        if changes.is_empty() {
            session.info("No client config needs to change.")?;
        } else {
            session.note("Summary", &summary(session, &changes))?;
        }
    }

    if dry_run || !has_writes {
        if session.json {
            print_json(&Report {
                changes: &changes,
                applied: false,
                backups: vec![],
                warnings: &warnings,
            })?;
        } else if dry_run {
            session.outro("Dry run: nothing was written.")?;
        } else {
            session.outro("Already up to date.")?;
        }
        return Ok(false);
    }

    confirm(&warnings)?;

    let Change { before, after, .. } = change;

    // Client files as they were, in case servers.yaml cannot be saved after
    // they are written.
    let client_files = RefCell::new(None);
    let committed_files = RefCell::new(std::collections::BTreeMap::new());
    let backups = RefCell::new(Vec::new());
    let recorded = Cell::new(false);
    let record_error = RefCell::new(None);
    store::update_guarded(
        &session.ctx,
        |current| {
            if serde_json::to_value(&*current).ok() != serde_json::to_value(&before).ok() {
                return Err(
                    "The server list changed while this command was running (another MCP \
                     Manager window or command?). Nothing was written; run it again."
                        .to_string(),
                );
            }
            let files = prepared.snapshot(&session.ctx)?;
            // apply undoes its own partial writes when it fails.
            let applied = prepared.apply(&session.ctx)?;
            after_apply();
            *current = after.clone();
            // Recorded while the lock is held, so no other change or rollback
            // can slip in between. A record that fails does not stop the
            // change: it is already applied, and the user is told below.
            match ChangeRecord::new(
                &files,
                applied.backups.clone(),
                &applied.committed_files,
                before.clone(),
                after.clone(),
            )
            .and_then(|record| history::record(&session.ctx, record))
            {
                Ok(()) => recorded.set(true),
                Err(error) => *record_error.borrow_mut() = Some(error),
            }
            *committed_files.borrow_mut() = applied.committed_files;
            *client_files.borrow_mut() = Some(files);
            *backups.borrow_mut() = applied.backups;
            Ok(())
        },
        // The shared-store lock excludes other manager transactions; ownership
        // checks separately protect observed edits made by external clients.
        |error| {
            let mut error = match client_files.borrow().as_ref().map(|files| files.restore_owned(&committed_files.borrow())) {
                Some(Ok(skipped)) if skipped.is_empty() => {
                    format!("{error}. Client files were restored; nothing was changed.")
                }
                Some(Ok(skipped)) => format!("{error}. Owned client files were restored; external edits were preserved in {}. Review these files before retrying.", skipped.iter().map(|p|p.display().to_string()).collect::<Vec<_>>().join(", ")),
                Some(Err(restore_error)) => {
                    format!("{error}. Restoring client files also failed: {restore_error}")
                }
                None => error,
            };
            // Only this change's own record: never an earlier one.
            if recorded.get() {
                if let Err(forget_error) = history::forget_latest(&session.ctx) {
                    error.push_str(&format!(
                        " Its rollback record could not be removed either ({forget_error})."
                    ));
                }
            }
            error
        },
    )?;

    // The change is already applied; say so rather than promising a rollback
    // that was never recorded.
    if let Some(error) = record_error.into_inner() {
        return Err(format!(
            "The changes were applied, but they could not be recorded for rollback ({error}), \
             so `{} rollback` will not undo them.",
            env!("CARGO_BIN_NAME")
        )
        .into());
    }
    let backups = backups.into_inner();
    if session.json {
        print_json(&Report {
            changes: &changes,
            applied: true,
            backups,
            warnings: &warnings,
        })?;
    } else {
        let written = changes
            .iter()
            .filter(|c| c.action != ChangeAction::Unchanged)
            .count();
        session.outro(format!(
            "Applied {written} change{}. Undo with: {} rollback",
            if written == 1 { "" } else { "s" },
            env!("CARGO_BIN_NAME")
        ))?;
    }
    Ok(true)
}

pub fn print_json(value: &impl Serialize) -> Outcome<()> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    println!("{text}");
    Ok(())
}

fn describe_warning(warning: &ApplyWarning) -> String {
    let client = warning.app.display_name();
    let server = &warning.server_id;
    match warning.kind.as_str() {
        "sharedProjectSecret" => {
            let path = warning.path.as_deref().unwrap_or_default();
            let key = warning.key.as_deref().unwrap_or_default();
            let advice = match warning.app {
                mcp_manager_core::core::SupportedApp::Agents => "Use a $VAR environment reference.",
                mcp_manager_core::core::SupportedApp::Windsurf => {
                    "Use .devin/mcp_config.local.json for private values."
                }
                mcp_manager_core::core::SupportedApp::Grok
                | mcp_manager_core::core::SupportedApp::ClaudeCode
                | mcp_manager_core::core::SupportedApp::FactoryDroid => {
                    "Use an environment reference such as ${VAR}."
                }
                _ => "Keep secrets in a private user configuration.",
            };
            format!("{client}: {server}: {path} will contain a literal value for {key} that may be a secret. Committing this file could expose it. {advice}")
        }
        "sseUnsupported" => format!(
            "{client} has no SSE support, so the SSE server {server} was not written there. \
             Use the server's streamable HTTP endpoint instead, if it has one."
        ),
        "httpHeadersUnsupported" => format!(
            "{client} cannot store request headers, so the headers of {server} were not \
             written there. Add them in {client} manually."
        ),
        other => format!("{client}: {server}: {other}"),
    }
}

fn describe_issue(issue: &Issue) -> String {
    match issue {
        Issue::NoServers => "There are no servers.".to_string(),
        Issue::EmptyId => "A server has an empty id.".to_string(),
        Issue::MissingProgram { server_id } => format!("{server_id}: stdio server has no command."),
        Issue::InvalidUrl { server_id } => {
            format!("{server_id}: URL must start with http:// or https://.")
        }
        Issue::NoClientEnabled { server_id } => format!("{server_id}: not enabled for any client."),
        Issue::InsecureHeaders { server_id } => {
            format!("{server_id}: request headers would be sent unencrypted; use an https:// URL.")
        }
        Issue::DuplicateHeader { server_id, header } => {
            format!("{server_id}: header {header} is set more than once (names ignore case).")
        }
    }
}

fn action_label(action: ChangeAction) -> &'static str {
    match action {
        ChangeAction::Add => "add",
        ChangeAction::Update => "update",
        ChangeAction::Remove => "remove",
        ChangeAction::Unchanged => "unchanged",
        ChangeAction::OverwriteUnmanaged => "overwrites an entry not added by MCP Manager",
    }
}

/// Paths under the project or home directory are shown relative to them.
pub fn short_path(session: &Session, path: &str) -> String {
    let resolved = session.ctx.resolve_path(path);
    if session.ctx.has_workspace() {
        if let Ok(relative) = resolved.strip_prefix(&session.ctx.workspace_root) {
            return format!("./{}", relative.to_string_lossy());
        }
    }
    match resolved.strip_prefix(&session.ctx.home_dir) {
        Ok(relative) => format!("~/{}", relative.to_string_lossy()),
        Err(_) => path.to_string(),
    }
}

pub fn scope_label(scope: PlacementScope) -> &'static str {
    match scope {
        PlacementScope::User => "user",
        PlacementScope::Workspace => "project",
    }
}

/// Format the preview with scope labels and per-client changes before confirmation.
fn summary(session: &Session, changes: &[PlannedChange]) -> String {
    let mut by_server: BTreeMap<&str, Vec<&PlannedChange>> = BTreeMap::new();
    for change in changes {
        by_server.entry(&change.server_id).or_default().push(change);
    }

    let rows = changes
        .iter()
        .map(|c| {
            (
                c.app.display_name().len(),
                short_path(session, &c.path).len(),
            )
        })
        .fold((0, 0), |(a, p), (ca, cp)| (a.max(ca), p.max(cp)));

    let mut lines = Vec::new();
    for (server, changes) in by_server {
        lines.push(server.to_string());
        for change in changes {
            lines.push(format!(
                "  {:app$}  {:7}  {:path$}  {}",
                change.app.display_name(),
                scope_label(change.scope),
                short_path(session, &change.path),
                action_label(change.action),
                app = rows.0,
                path = rows.1,
            ));
        }
    }
    let backups = session.ctx.app_data_dir().join("backups");
    lines.push(String::new());
    lines.push(format!(
        "Existing files are backed up to {}",
        short_path(session, &backups.to_string_lossy())
    ));
    lines.join("\n")
}

#[cfg(test)]
mod confirmation_tests {
    use super::*;
    use mcp_manager_core::adapters::{AppAdapter, ZCodeAdapter};
    use mcp_manager_core::platform::{PlatformContext, PlatformOs};
    use std::fs;

    #[test]
    fn failed_store_save_preserves_an_external_edit_after_apply() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let mut server = mcp_manager_core::parser::parse_mcp_json(
            r#"{"mcpServers":{"compensation":{"command":"node"}}}"#,
        )
        .servers
        .remove(0);
        server
            .apps
            .insert(mcp_manager_core::core::SupportedApp::Cursor, true);
        let change = Change {
            before: MCPConfig {
                version: 1,
                servers: vec![],
            },
            after: MCPConfig {
                version: 1,
                servers: vec![server],
            },
            focus: vec!["compensation".into()],
        };
        let session = Session {
            ctx,
            interactive: false,
            yes: true,
            json: true,
            agent: None,
        };
        let native = session.ctx.home_dir.join(".cursor/mcp.json");
        let result = commit_with_hooks(
            &session,
            change,
            false,
            |_| Ok(()),
            || {
                fs::write(&native, r#"{"external":true}"#).unwrap();
                fs::create_dir(store::config_path(&session.ctx)).unwrap();
            },
        );
        let Failure::Error(message) = result.unwrap_err() else {
            panic!("expected store save failure")
        };
        assert!(
            message.contains("external edits were preserved"),
            "{message}"
        );
        assert!(!message.contains("nothing was changed"));
        assert_eq!(fs::read_to_string(native).unwrap(), r#"{"external":true}"#);
    }

    #[test]
    fn retained_secret_warning_reaches_confirmation_before_any_write() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("project"),
        };
        let fallback = ctx.workspace_file(".agents/mcp.json");
        fs::create_dir_all(fallback.parent().unwrap()).unwrap();
        let content = r#"{"mcpServers":{"api":{"command":"node","params":{"api_key":"pre-confirmation-secret"}}}}"#;
        fs::write(&fallback, content).unwrap();
        let parsed = ZCodeAdapter.parse_source(&ctx, fallback.to_str().unwrap(), 9, content);
        let change = Change {
            before: MCPConfig {
                version: 1,
                servers: vec![],
            },
            after: MCPConfig {
                version: 1,
                servers: parsed
                    .servers
                    .into_iter()
                    .map(|(server, _)| server)
                    .collect(),
            },
            focus: vec!["api".into()],
        };
        let session = Session {
            ctx,
            interactive: false,
            yes: false,
            json: true,
            agent: None,
        };
        let native = session.ctx.workspace_file(".zcode/config.json");
        let reached = Cell::new(false);
        let result = commit_with_confirmation(&session, change, false, |warnings| {
            reached.set(true);
            assert!(warnings
                .iter()
                .any(|warning| warning.key.as_deref() == Some("params.api_key")));
            assert!(!serde_json::to_string(warnings)
                .unwrap()
                .contains("pre-confirmation-secret"));
            assert!(!native.exists());
            assert!(!store::config_path(&session.ctx).exists());
            Err(Failure::Cancelled)
        });
        assert!(matches!(result, Err(Failure::Cancelled)));
        assert!(reached.get());
        assert!(!native.exists());
        assert_eq!(fs::read_to_string(fallback).unwrap(), content);
    }
}
