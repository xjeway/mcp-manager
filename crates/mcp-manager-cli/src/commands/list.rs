use super::installed_apps;
use crate::commit::{print_json, scope_label, short_path};
use crate::session::{Outcome, Session};
use mcp_manager_core::core::{MCPServer, PlacementScope, SupportedApp};
use mcp_manager_core::store;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Listed<'a> {
    id: &'a str,
    enabled: bool,
    transport: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<Vec<&'a str>>,
    clients: Vec<Installed>,
}

#[derive(Serialize)]
struct Installed {
    client: &'static str,
    scope: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

fn installs(session: &Session, server: &MCPServer) -> Vec<Installed> {
    let mut found = Vec::new();
    for app in installed_apps(server) {
        let user_placement = server
            .placements
            .iter()
            .any(|p| p.enabled && p.app == app && p.scope == PlacementScope::User);
        if server.apps.get(&app).copied().unwrap_or(false) || user_placement {
            found.push(Installed {
                client: app.cli_id(),
                scope: scope_label(PlacementScope::User),
                path: None,
            });
        }
        for placement in server
            .placements
            .iter()
            .filter(|p| p.enabled && p.app == app && p.scope == PlacementScope::Workspace)
        {
            found.push(Installed {
                client: app.cli_id(),
                scope: scope_label(PlacementScope::Workspace),
                path: placement
                    .path
                    .as_deref()
                    .map(|path| short_path(session, path)),
            });
        }
    }
    found
}

pub fn run(session: &Session) -> Outcome<()> {
    let config = store::load(&session.ctx)?;

    if session.json {
        let listed = config
            .servers
            .iter()
            .map(|server| Listed {
                id: &server.id,
                enabled: server.enabled,
                transport: &server.transport.kind,
                url: server.transport.url.as_deref(),
                command: server.command.as_ref().map(|c| {
                    std::iter::once(c.program.as_str())
                        .chain(c.args.iter().map(String::as_str))
                        .collect()
                }),
                clients: installs(session, server),
            })
            .collect::<Vec<_>>();
        return print_json(&listed);
    }

    if config.servers.is_empty() {
        println!(
            "No MCP servers yet. Add one with:\n  {} add <id> -- <command>",
            env!("CARGO_BIN_NAME")
        );
        return Ok(());
    }

    for server in &config.servers {
        let target = match (&server.transport.url, &server.command) {
            (Some(url), _) => url.clone(),
            (None, Some(command)) => std::iter::once(command.program.clone())
                .chain(command.args.iter().cloned())
                .collect::<Vec<_>>()
                .join(" "),
            (None, None) => String::new(),
        };
        let disabled = if server.enabled { "" } else { "  (disabled)" };
        println!(
            "{}  {}  {target}{disabled}",
            server.id, server.transport.kind
        );

        let installed = installs(session, server);
        if installed.is_empty() {
            println!("  not installed in any client");
        }
        for scope in ["user", "project"] {
            let names = installed
                .iter()
                .filter(|i| i.scope == scope)
                .map(|i| {
                    let name =
                        SupportedApp::parse(i.client).map_or(i.client, SupportedApp::display_name);
                    match &i.path {
                        Some(path) => format!("{name} ({path})"),
                        None => name.to_string(),
                    }
                })
                .collect::<Vec<_>>();
            if !names.is_empty() {
                println!("  {scope:<8} {}", names.join(", "));
            }
        }
    }
    Ok(())
}
