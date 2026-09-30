use super::{app_label, parse_apps, require_json_mode_is_non_interactive};
use crate::cli::AddArgs;
use crate::commit::{commit, Change};
use crate::resolve::{self, Scope};
use crate::session::{Outcome, Session};
use crate::source::{self, Source};
use crate::state;
use mcp_manager_core::adapters::workspace_scope_path;
use mcp_manager_core::core::MCPServer;
use mcp_manager_core::{ops, store};

pub fn run(session: &Session, args: &AddArgs) -> Outcome<()> {
    require_json_mode_is_non_interactive(session, args.dry_run)?;
    session.intro(concat!(env!("CARGO_BIN_NAME"), " add"))?;
    let ctx = &session.ctx;
    let before = store::load(ctx)?;

    let resolved = source::resolve(&Source {
        id: args.id.as_deref(),
        url: args.url.as_deref(),
        transport: args.transport.as_deref(),
        from: args.from.as_deref(),
        command: &args.command,
    })?;
    for warning in &resolved.warnings {
        session.warn(warning)?;
    }

    let candidates = resolved
        .servers
        .iter()
        .map(|s| s.id.clone())
        .collect::<Vec<_>>();
    let chosen = session.choose_many(
        resolve::servers(&candidates, &args.servers)?,
        "Which servers do you want to add?",
        |id| {
            let hint = resolved
                .servers
                .iter()
                .find(|s| &s.id == id)
                .and_then(|s| s.description.clone())
                .unwrap_or_default();
            (id.clone(), hint)
        },
        "--server <id>",
    )?;
    let mut servers = chosen
        .iter()
        .filter_map(|id| resolved.servers.iter().find(|s| &s.id == id).cloned())
        .collect::<Vec<_>>();

    source::apply_env(&mut servers, &args.env)?;
    fill_missing_env(session, &mut servers)?;

    let detected = ctx.detect_installed_apps();
    let remembered = state::load(ctx);
    let apps = session.choose_many(
        resolve::apps(
            parse_apps(&args.apps)?,
            session.agent.as_ref().and_then(|agent| agent.app),
            &detected,
            &remembered.last_apps,
        ),
        "Which clients should get them?",
        |app| app_label(app, &detected),
        "--app <client> (or --app '*')",
    )?;

    let with_project = apps
        .iter()
        .filter(|app| workspace_scope_path(ctx, **app).is_some())
        .count();
    let requested_scope = match (args.global, args.project) {
        (true, _) => Some(Scope::User),
        (_, true) => Some(Scope::Project),
        _ => None,
    };
    let scope = session.choose_one(
        resolve::scope(requested_scope, ctx.has_workspace(), with_project > 0)?,
        "Where should they be configured?",
        |scope| match scope {
            Scope::User => ("User".to_string(), "available in every project".to_string()),
            Scope::Project => (
                "Project".to_string(),
                format!(
                    "files in {}, shareable with your team",
                    ctx.workspace_root.display()
                ),
            ),
        },
        Scope::User,
    )?;

    let mut after = before.clone();
    ops::import_servers(&mut after, &servers);
    let mut user_only = Vec::new();
    for app in &apps {
        let project_path = workspace_scope_path(ctx, *app).filter(|_| scope == Scope::Project);
        if scope == Scope::Project && project_path.is_none() {
            user_only.push(app.display_name());
        }
        for id in &chosen {
            match &project_path {
                Some(path) => ops::set_workspace_placement(&mut after, id, *app, path, true),
                None => ops::set_app_enabled(&mut after, std::slice::from_ref(id), *app, true),
            }
        }
    }
    if !user_only.is_empty() {
        session.info(format!(
            "{} {} no project-level config; using the user-level one.",
            user_only.join(", "),
            if user_only.len() == 1 { "has" } else { "have" }
        ))?;
    }

    let written = commit(
        session,
        Change {
            before,
            after,
            focus: chosen,
        },
        args.dry_run,
    )?;

    if written && session.interactive {
        let _ = state::update(ctx, |remembered| remembered.last_apps = apps);
    }
    Ok(())
}

/// Prompts for environment variables the source left empty, such as API tokens.
fn fill_missing_env(session: &Session, servers: &mut [MCPServer]) -> Outcome<()> {
    for server in servers.iter_mut() {
        let Some(command) = server.command.as_mut() else {
            continue;
        };
        let mut missing = command
            .env
            .iter()
            .filter(|(_, value)| value.is_empty())
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        missing.sort();
        for key in missing {
            if session.interactive {
                let value = session.secret(&format!(
                    "{key} for {} (saved in each client's config file; leave empty to skip)",
                    server.id
                ))?;
                command.env.insert(key, value);
            } else {
                session.warn(format!(
                    "{}: {key} is empty; set it with --env {key}=<value>",
                    server.id
                ))?;
            }
        }
    }
    Ok(())
}
