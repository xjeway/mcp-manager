//! Edits to the unified config, ported from the app's frontend
//! (`src/services/workspacePersistence.ts`, `src/services/pendingChanges.ts`,
//! `src/view-models/workspace.ts` and `src/services/risk.ts`).
//!
//! `tests/fixtures/ops-cases.json` runs against both implementations, so
//! change them together.

use crate::core::{MCPConfig, MCPServer, PlacementScope, ServerPlacement, SupportedApp};
use serde::Serialize;

/// Adds `server`, or replaces the server being edited (`editing_id`, else the
/// server's own id) in place, so renaming keeps its position.
pub fn upsert_server(config: &mut MCPConfig, editing_id: Option<&str>, server: MCPServer) {
    let target_id = editing_id.unwrap_or(&server.id).to_string();
    match config.servers.iter_mut().find(|item| item.id == target_id) {
        Some(existing) => *existing = server,
        None => config.servers.push(server),
    }
}

pub fn remove_server(config: &mut MCPConfig, server_id: &str) {
    config.servers.retain(|server| server.id != server_id);
}

/// Turns `app` on or off for each server in `server_ids`. The user-level flag
/// and user-scope placements follow `enabled`; turning off also disables
/// project placements, but turning on never re-enables them, so a project is
/// only ever changed explicitly.
pub fn set_app_enabled(
    config: &mut MCPConfig,
    server_ids: &[String],
    app: SupportedApp,
    enabled: bool,
) {
    for server in config
        .servers
        .iter_mut()
        .filter(|server| server_ids.contains(&server.id))
    {
        server.apps.insert(app, enabled);
        for placement in server
            .placements
            .iter_mut()
            .filter(|placement| placement.app == app)
        {
            if placement.scope == PlacementScope::User || !enabled {
                placement.enabled = enabled;
            }
        }
    }
}

/// Enables or disables the project-level placement of `app` at `path`,
/// adding it if missing.
pub fn set_workspace_placement(
    config: &mut MCPConfig,
    server_id: &str,
    app: SupportedApp,
    path: &str,
    enabled: bool,
) {
    let Some(server) = config
        .servers
        .iter_mut()
        .find(|server| server.id == server_id)
    else {
        return;
    };

    let target = server.placements.iter_mut().find(|placement| {
        placement.app == app
            && placement.scope == PlacementScope::Workspace
            && placement.path.as_deref() == Some(path)
    });
    match target {
        Some(placement) => placement.enabled = enabled,
        None => server.placements.push(ServerPlacement {
            app,
            scope: PlacementScope::Workspace,
            path: Some(path.to_string()),
            enabled,
            managed: true,
        }),
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportOutcome {
    pub added: Vec<String>,
    pub updated: Vec<String>,
}

/// Adds `servers` to the config. A server whose id already exists takes the
/// incoming definition but keeps the clients and placements it already had,
/// so re-importing never uninstalls anything.
pub fn import_servers(config: &mut MCPConfig, servers: &[MCPServer]) -> ImportOutcome {
    let mut outcome = ImportOutcome::default();

    for incoming in servers {
        let Some(current) = config
            .servers
            .iter_mut()
            .find(|server| server.id == incoming.id)
        else {
            config.servers.push(incoming.clone());
            outcome.added.push(incoming.id.clone());
            continue;
        };

        let mut apps = current.apps.clone();
        for (app, enabled) in &incoming.apps {
            let entry = apps.entry(*app).or_insert(false);
            *entry = *entry || *enabled;
        }

        let same_placement = |left: &ServerPlacement, right: &ServerPlacement| {
            left.app == right.app && left.scope == right.scope && left.path == right.path
        };
        let mut placements = current
            .placements
            .iter()
            .map(|existing| {
                let enabled_by_import = incoming
                    .placements
                    .iter()
                    .any(|placement| placement.enabled && same_placement(existing, placement));
                ServerPlacement {
                    enabled: existing.enabled || enabled_by_import,
                    ..existing.clone()
                }
            })
            .collect::<Vec<_>>();
        placements.extend(
            incoming
                .placements
                .iter()
                .filter(|placement| {
                    !current
                        .placements
                        .iter()
                        .any(|existing| same_placement(existing, placement))
                })
                .cloned(),
        );

        *current = MCPServer {
            apps,
            placements,
            ..incoming.clone()
        };
        outcome.updated.push(incoming.id.clone());
    }

    outcome
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum Issue {
    NoServers,
    EmptyId,
    /// Ids become path fragments in some clients. `/`, `\`, NUL and `..` are rejected.
    #[serde(rename_all = "camelCase")]
    InvalidId {
        server_id: String,
    },
    #[serde(rename_all = "camelCase")]
    MissingProgram {
        server_id: String,
    },
    #[serde(rename_all = "camelCase")]
    InvalidUrl {
        server_id: String,
    },
    #[serde(rename_all = "camelCase")]
    NoClientEnabled {
        server_id: String,
    },
    /// Request headers on a plain-http URL to a non-loopback host.
    #[serde(rename_all = "camelCase")]
    InsecureHeaders {
        server_id: String,
    },
    #[serde(rename_all = "camelCase")]
    DuplicateHeader {
        server_id: String,
        header: String,
    },
}

impl Issue {
    /// Stable wording shared by the CLI and the GUI apply path.
    pub fn summary(&self) -> String {
        match self {
            Issue::NoServers => "There are no servers.".to_string(),
            Issue::EmptyId => "A server has an empty id.".to_string(),
            Issue::InvalidId { server_id } => {
                format!("{server_id}: server id must not contain '/', '\\', a NUL byte, or '..'.")
            }
            Issue::MissingProgram { server_id } => {
                format!("{server_id}: stdio server has no command.")
            }
            Issue::InvalidUrl { server_id } => {
                format!("{server_id}: URL must be an http:// or https:// URL with a host.")
            }
            Issue::NoClientEnabled { server_id } => {
                format!("{server_id}: not enabled for any client.")
            }
            Issue::InsecureHeaders { server_id } => format!(
                "{server_id}: request headers would be sent unencrypted; use an https:// URL."
            ),
            Issue::DuplicateHeader { server_id, header } => {
                format!("{server_id}: header {header} is set more than once (names ignore case).")
            }
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validation {
    pub warnings: Vec<Issue>,
    /// Applying must not proceed while any of these remain.
    pub blocking_errors: Vec<Issue>,
}

/// Checks a config before it is applied. Issues are codes, not messages, so
/// each front end words them itself.
pub fn validate(config: &MCPConfig) -> Validation {
    let mut result = Validation::default();

    if config.servers.is_empty() {
        result.warnings.push(Issue::NoServers);
    }

    for server in &config.servers {
        let server_id = server.id.clone();
        if server.id.trim().is_empty() {
            result.blocking_errors.push(Issue::EmptyId);
        } else if server_id_is_unsafe(&server.id) {
            result.blocking_errors.push(Issue::InvalidId {
                server_id: server_id.clone(),
            });
        }

        match server.transport.kind.as_str() {
            "stdio"
                if server
                    .command
                    .as_ref()
                    .is_none_or(|command| command.program.is_empty()) =>
            {
                result.blocking_errors.push(Issue::MissingProgram {
                    server_id: server_id.clone(),
                });
            }
            "http" => {
                let url = server.transport.url.as_deref().unwrap_or_default();
                if !http_url_has_host(url) {
                    result.blocking_errors.push(Issue::InvalidUrl {
                        server_id: server_id.clone(),
                    });
                }
            }
            _ => {}
        }

        if server.transport.sends_headers_insecurely() {
            result.blocking_errors.push(Issue::InsecureHeaders {
                server_id: server_id.clone(),
            });
        }
        if let Some(header) = server.transport.duplicate_header() {
            result.blocking_errors.push(Issue::DuplicateHeader {
                server_id: server_id.clone(),
                header,
            });
        }

        if !server.apps.values().any(|enabled| *enabled) {
            result.warnings.push(Issue::NoClientEnabled { server_id });
        }
    }

    result
}

fn server_id_is_unsafe(id: &str) -> bool {
    id.contains(['/', '\\', '\0']) || id.split(['/', '\\']).any(|part| part == "..")
}

fn http_url_has_host(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.rsplit('@').next().unwrap_or("");
    !host.is_empty() && !host.starts_with(':')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TransportSpec;
    use serde::Deserialize;
    use serde_json::{Map, Value};
    use std::collections::HashMap;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OpsCase {
        name: String,
        op: String,
        args: Map<String, Value>,
        input: MCPConfig,
        expected: Value,
        expected_added: Option<Vec<String>>,
        expected_updated: Option<Vec<String>>,
    }

    fn arg<T: serde::de::DeserializeOwned>(args: &Map<String, Value>, key: &str) -> T {
        serde_json::from_value(args.get(key).cloned().unwrap_or(Value::Null))
            .unwrap_or_else(|e| panic!("arg {key}: {e}"))
    }

    /// Drops nulls and empty placement lists, which both sides treat as absent.
    fn normalize(mut value: Value) -> Value {
        fn walk(value: &mut Value) {
            match value {
                Value::Object(map) => {
                    map.retain(|key, v| {
                        !v.is_null()
                            && !(key == "placements" && v.as_array().is_some_and(Vec::is_empty))
                    });
                    map.values_mut().for_each(walk);
                }
                Value::Array(items) => items.iter_mut().for_each(walk),
                _ => {}
            }
        }
        walk(&mut value);
        value
    }

    #[test]
    fn matches_the_frontend_on_shared_cases() {
        let cases: Vec<OpsCase> =
            serde_json::from_str(include_str!("../tests/fixtures/ops-cases.json"))
                .expect("parse ops cases");
        assert!(!cases.is_empty());

        for case in cases {
            let mut config = case.input;
            let args = &case.args;
            let mut outcome = None;
            match case.op.as_str() {
                "upsertServer" => upsert_server(
                    &mut config,
                    arg::<Option<String>>(args, "editingId").as_deref(),
                    arg(args, "server"),
                ),
                "removeServer" => remove_server(&mut config, &arg::<String>(args, "serverId")),
                "setAppEnabled" => set_app_enabled(
                    &mut config,
                    &arg::<Vec<String>>(args, "serverIds"),
                    arg(args, "app"),
                    arg(args, "enabled"),
                ),
                "setWorkspacePlacement" => set_workspace_placement(
                    &mut config,
                    &arg::<String>(args, "serverId"),
                    arg(args, "app"),
                    &arg::<String>(args, "path"),
                    arg(args, "enabled"),
                ),
                "importServers" => {
                    outcome = Some(import_servers(
                        &mut config,
                        &arg::<Vec<MCPServer>>(args, "servers"),
                    ))
                }
                other => panic!("unknown op {other}"),
            }

            assert_eq!(
                normalize(serde_json::to_value(&config).unwrap()),
                normalize(case.expected),
                "case: {}",
                case.name
            );
            if let Some(added) = case.expected_added {
                let outcome = outcome.expect("import outcome");
                assert_eq!(outcome.added, added, "case: {}", case.name);
                assert_eq!(
                    Some(outcome.updated),
                    case.expected_updated,
                    "case: {}",
                    case.name
                );
            }
        }
    }

    fn server(id: &str, kind: &str, url: Option<&str>, program: Option<&str>) -> MCPServer {
        MCPServer {
            description: None,
            homepage: None,
            id: id.to_string(),
            name: id.to_string(),
            enabled: true,
            transport: TransportSpec {
                kind: kind.to_string(),
                url: url.map(str::to_string),
                headers: Default::default(),
            },
            command: program.map(|program| crate::core::CommandSpec {
                program: program.to_string(),
                args: vec![],
                env: HashMap::new(),
                secret_env: Default::default(),
            }),
            apps: HashMap::from([(SupportedApp::Cursor, true)]),
            placements: vec![],
        }
    }

    #[test]
    fn validation_blocks_what_the_frontend_blocks() {
        let mut no_apps = server("quiet", "stdio", None, Some("npx"));
        no_apps.apps.clear();
        let config = MCPConfig {
            version: 1,
            servers: vec![
                server("ok", "stdio", None, Some("npx")),
                server(" ", "http", Some("https://example.com/mcp"), None),
                server("noprog", "stdio", None, None),
                server("badurl", "http", Some("ftp://example.com"), None),
                no_apps,
            ],
        };

        let result = validate(&config);

        assert_eq!(
            result.blocking_errors,
            vec![
                Issue::EmptyId,
                Issue::MissingProgram {
                    server_id: "noprog".to_string()
                },
                Issue::InvalidUrl {
                    server_id: "badurl".to_string()
                },
            ]
        );
        assert_eq!(
            result.warnings,
            vec![Issue::NoClientEnabled {
                server_id: "quiet".to_string()
            }]
        );
        assert_eq!(
            validate(&MCPConfig {
                version: 1,
                servers: vec![]
            })
            .warnings,
            vec![Issue::NoServers]
        );
    }

    fn with_headers(id: &str, url: &str, headers: &[(&str, &str)]) -> MCPServer {
        let mut remote = server(id, "http", Some(url), None);
        remote.transport.headers = headers
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        remote
    }

    #[test]
    fn headers_over_plain_http_are_blocked_except_on_loopback() {
        let config = MCPConfig {
            version: 1,
            servers: vec![
                with_headers(
                    "leaky",
                    "http://mcp.example.com/mcp",
                    &[("Authorization", "Bearer t")],
                ),
                with_headers(
                    "secure",
                    "https://mcp.example.com/mcp",
                    &[("Authorization", "Bearer t")],
                ),
                with_headers(
                    "local",
                    "http://localhost:3000/mcp",
                    &[("Authorization", "Bearer t")],
                ),
                with_headers("loopback", "http://127.0.0.1:3000/mcp", &[("X-Key", "k")]),
                with_headers("ipv6", "http://[::1]:3000/mcp", &[("X-Key", "k")]),
                with_headers("plain", "http://mcp.example.com/mcp", &[]),
                // Hostnames that merely look like loopback are remote.
                with_headers(
                    "lookalike",
                    "http://127.evil.com/mcp",
                    &[("Authorization", "Bearer t")],
                ),
                with_headers(
                    "bad-octet",
                    "http://127.0.0.256/mcp",
                    &[("Authorization", "Bearer t")],
                ),
                with_headers(
                    "loopback-range",
                    "http://127.1.2.3/mcp",
                    &[("Authorization", "Bearer t")],
                ),
            ],
        };
        let insecure = |id: &str| Issue::InsecureHeaders {
            server_id: id.to_string(),
        };
        assert_eq!(
            validate(&config).blocking_errors,
            vec![
                insecure("leaky"),
                insecure("lookalike"),
                insecure("bad-octet")
            ]
        );
    }

    #[test]
    fn header_names_differing_only_by_case_are_blocked() {
        let config = MCPConfig {
            version: 1,
            servers: vec![with_headers(
                "dupe",
                "https://mcp.example.com/mcp",
                &[("Authorization", "Bearer a"), ("authorization", "Bearer b")],
            )],
        };
        assert_eq!(
            validate(&config).blocking_errors,
            vec![Issue::DuplicateHeader {
                server_id: "dupe".to_string(),
                header: "authorization".to_string(),
            }]
        );
    }

    #[test]
    fn rejects_unsafe_server_ids_and_urls_without_a_host() {
        let mut traversal = server("ok", "stdio", None, Some("npx"));
        traversal.id = "../secret".to_string();
        let mut slash = server("ok", "stdio", None, Some("npx"));
        slash.id = "a/b".to_string();
        let config = MCPConfig {
            version: 1,
            servers: vec![
                traversal,
                slash,
                server("nohost", "http", Some("http://"), None),
                server("scheme", "http", Some("https://example.com/mcp"), None),
            ],
        };
        let codes = validate(&config)
            .blocking_errors
            .into_iter()
            .map(|issue| match issue {
                Issue::InvalidId { server_id } => format!("id:{server_id}"),
                Issue::InvalidUrl { server_id } => format!("url:{server_id}"),
                other => format!("other:{}", other.summary()),
            })
            .collect::<Vec<_>>();
        assert!(codes.contains(&"id:../secret".to_string()), "{codes:?}");
        assert!(codes.contains(&"id:a/b".to_string()), "{codes:?}");
        assert!(codes.contains(&"url:nohost".to_string()), "{codes:?}");
        assert!(
            !codes.iter().any(|code| code.contains("scheme")),
            "{codes:?}"
        );
    }
}
