//! The shared end of every command that changes servers: preview the effect
//! on each client file, show a summary, confirm, then save and apply.

use crate::session::{Failure, Outcome, Session};
use mcp_manager_core::core::{MCPConfig, PlacementScope};
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
}

/// Returns whether anything was written.
pub fn commit(session: &Session, change: Change, dry_run: bool) -> Outcome<bool> {
    let blocking = validate(&change.after).blocking_errors;
    if !blocking.is_empty() {
        let lines = blocking.iter().map(describe_issue).collect::<Vec<_>>();
        return Err(Failure::Error(lines.join("\n")));
    }

    let changes = workflow::preview(&session.ctx, &change.after, Some(&change.before))
        .into_iter()
        .filter(|c| change.focus.contains(&c.server_id) || c.action != ChangeAction::Unchanged)
        .collect::<Vec<_>>();
    let has_writes = changes.iter().any(|c| c.action != ChangeAction::Unchanged);

    if !session.json {
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
            })?;
        } else if dry_run {
            session.outro("Dry run: nothing was written.")?;
        } else {
            session.outro("Already up to date.")?;
        }
        return Ok(false);
    }

    session.confirm("Apply these changes?")?;

    let Change { before, after, .. } = change;

    // Client files as they were, in case servers.yaml cannot be saved after
    // they are written.
    let client_files = RefCell::new(None);
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
            let files = workflow::snapshot(&session.ctx, &after, Some(&before))?;
            // apply undoes its own partial writes when it fails.
            let applied = workflow::apply(&session.ctx, &after, Some(&before))?;
            *current = after.clone();
            // Recorded while the lock is held, so no other change or rollback
            // can slip in between. A record that fails does not stop the
            // change: it is already applied, and the user is told below.
            match ChangeRecord::new(
                &session.ctx.app_data_dir(),
                &files,
                applied.backups.clone(),
                before.clone(),
                after.clone(),
            )
            .and_then(|record| history::record(&session.ctx, record))
            {
                Ok(()) => recorded.set(true),
                Err(error) => *record_error.borrow_mut() = Some(error),
            }
            *client_files.borrow_mut() = Some(files);
            *backups.borrow_mut() = applied.backups;
            Ok(())
        },
        // Runs before the lock is released, so nothing else can write the
        // client files while they are put back.
        |error| {
            let mut error = match client_files.borrow().as_ref().map(|files| files.restore()) {
                Some(Ok(())) => {
                    format!("{error}. Client files were restored; nothing was changed.")
                }
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

fn describe_issue(issue: &Issue) -> String {
    issue.summary()
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
