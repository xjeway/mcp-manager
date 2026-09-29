use super::{app_label, installed_apps, parse_apps, require_json_mode_is_non_interactive};
use crate::cli::RemoveArgs;
use crate::commit::{commit, Change};
use crate::resolve::Decision;
use crate::session::{Failure, Outcome, Session};
use mcp_manager_core::{ops, store};

pub fn run(session: &Session, args: &RemoveArgs) -> Outcome<()> {
    require_json_mode_is_non_interactive(session, args.dry_run)?;
    session.intro(concat!(env!("CARGO_BIN_NAME"), " remove"))?;
    let before = store::load(&session.ctx)?;
    let known = before
        .servers
        .iter()
        .map(|s| s.id.clone())
        .collect::<Vec<_>>();

    let ids = if args.ids.is_empty() {
        if known.is_empty() {
            session.outro("There are no servers to remove.")?;
            return Ok(());
        }
        session.choose_many(
            Decision::Ask {
                options: known.clone(),
                preselected: vec![],
            },
            "Which servers do you want to remove?",
            |id| (id.clone(), String::new()),
            "the server ids",
        )?
    } else {
        let unknown = args
            .ids
            .iter()
            .filter(|id| !known.contains(id))
            .cloned()
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            return Err(Failure::Error(format!(
                "No server named {}. Known servers: {}",
                unknown.join(", "),
                known.join(", ")
            )));
        }
        args.ids.clone()
    };

    let mut after = before.clone();
    let present = before
        .servers
        .iter()
        .filter(|s| ids.contains(&s.id))
        .flat_map(installed_apps)
        .fold(Vec::new(), |mut apps, app| {
            if !apps.contains(&app) {
                apps.push(app);
            }
            apps
        });

    let only = match parse_apps(&args.apps)? {
        Some(apps) => Some(apps),
        None if session.interactive && present.len() > 1 => {
            let chosen = session.choose_many(
                Decision::Ask {
                    options: present.clone(),
                    preselected: present.clone(),
                },
                "Remove from which clients?",
                |app| app_label(app, &[]),
                "--app <client>",
            )?;
            (chosen.len() < present.len()).then_some(chosen)
        }
        None => None,
    };

    match only {
        Some(apps) => {
            for app in apps {
                ops::set_app_enabled(&mut after, &ids, app, false);
            }
        }
        None => {
            for id in &ids {
                ops::remove_server(&mut after, id);
            }
        }
    }

    commit(
        session,
        Change {
            before,
            after,
            focus: ids,
        },
        args.dry_run,
    )?;
    Ok(())
}
