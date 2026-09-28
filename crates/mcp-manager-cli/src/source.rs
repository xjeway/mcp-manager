//! Turns `add`'s arguments into server definitions.

use mcp_manager_core::core::{empty_apps, CommandSpec, MCPServer, TransportSpec};
use mcp_manager_core::parser::parse_mcp_json;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

pub struct Source<'a> {
    pub id: Option<&'a str>,
    pub url: Option<&'a str>,
    /// `sse` or `http` for `--url` servers.
    pub transport: Option<&'a str>,
    /// A JSON file in any client's format (`mcpServers`/`servers`), or `-` for stdin.
    pub from: Option<&'a Path>,
    /// Program and arguments after `--`.
    pub command: &'a [String],
}

#[derive(Debug)]
pub struct Resolved {
    pub servers: Vec<MCPServer>,
    pub warnings: Vec<String>,
}

pub const USAGE: &str = "Nothing to add. Use one of:\n  add <id> --url <https://…>\n  add <id> -- <command> [args…]\n  add --from <mcp.json>";

pub fn resolve(source: &Source) -> Result<Resolved, String> {
    let kinds = [
        source.from.is_some(),
        source.url.is_some(),
        !source.command.is_empty(),
    ];
    if kinds.iter().filter(|given| **given).count() > 1 {
        return Err("Use only one of --from, --url, or -- <command>.".to_string());
    }

    if let Some(path) = source.from {
        if source.id.is_some() {
            return Err("--from reads ids from the file; pick servers with --server.".to_string());
        }
        return from_json(&read_source(path)?);
    }

    let Some(id) = source.id else {
        return Err(USAGE.to_string());
    };
    if id.trim().is_empty() || id.chars().any(char::is_whitespace) {
        return Err(format!(
            "Invalid server id {id:?}: use a name without spaces."
        ));
    }

    let server = if let Some(url) = source.url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(format!("{url} is not an http(s) URL."));
        }
        let kind = match source.transport {
            None | Some("http") => "http",
            Some("sse") => "sse",
            Some(other) => return Err(format!("Unknown transport {other}; use http or sse.")),
        };
        new_server(
            id,
            TransportSpec {
                kind: kind.to_string(),
                url: Some(url.to_string()),
            },
            None,
        )
    } else if let [program, args @ ..] = source.command {
        new_server(
            id,
            TransportSpec {
                kind: "stdio".to_string(),
                url: None,
            },
            Some(CommandSpec {
                program: program.clone(),
                args: args.to_vec(),
                env: HashMap::new(),
            }),
        )
    } else {
        return Err(USAGE.to_string());
    };

    Ok(Resolved {
        servers: vec![server],
        warnings: vec![],
    })
}

fn new_server(id: &str, transport: TransportSpec, command: Option<CommandSpec>) -> MCPServer {
    MCPServer {
        description: None,
        homepage: None,
        id: id.to_string(),
        name: id.to_string(),
        enabled: true,
        transport,
        command,
        apps: empty_apps(),
        placements: vec![],
    }
}

fn read_source(path: &Path) -> Result<String, String> {
    if path == Path::new("-") {
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .map_err(|e| format!("Could not read stdin: {e}"))?;
        return Ok(content);
    }
    std::fs::read_to_string(path).map_err(|e| format!("Could not read {}: {e}", path.display()))
}

fn from_json(content: &str) -> Result<Resolved, String> {
    let parsed = parse_mcp_json(content);
    if !parsed.errors.is_empty() {
        return Err(parsed.errors.join("\n"));
    }
    Ok(Resolved {
        servers: parsed.servers,
        warnings: parsed.warnings,
    })
}

/// Applies `KEY=VALUE` pairs to every stdio server's environment.
pub fn apply_env(servers: &mut [MCPServer], pairs: &[String]) -> Result<(), String> {
    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(format!("--env expects KEY=VALUE, got {pair:?}."));
        };
        for command in servers.iter_mut().filter_map(|s| s.command.as_mut()) {
            command.env.insert(key.to_string(), value.to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{apply_env, from_json, resolve, Source};
    use std::path::Path;

    fn source<'a>(id: Option<&'a str>, command: &'a [String]) -> Source<'a> {
        Source {
            id,
            url: None,
            transport: None,
            from: None,
            command,
        }
    }

    #[test]
    fn builds_a_stdio_server_from_the_command_after_dashes() {
        let command = vec!["npx".to_string(), "-y".to_string(), "pkg".to_string()];
        let resolved = resolve(&source(Some("ctx"), &command)).unwrap();

        let server = &resolved.servers[0];
        assert_eq!(server.id, "ctx");
        assert_eq!(server.transport.kind, "stdio");
        let command = server.command.as_ref().unwrap();
        assert_eq!(command.program, "npx");
        assert_eq!(command.args, vec!["-y", "pkg"]);
    }

    #[test]
    fn builds_remote_servers_from_a_url() {
        let resolved = resolve(&Source {
            url: Some("https://mcp.linear.app/sse"),
            transport: Some("sse"),
            ..source(Some("linear"), &[])
        })
        .unwrap();
        assert_eq!(resolved.servers[0].transport.kind, "sse");

        assert!(resolve(&Source {
            url: Some("ftp://x"),
            ..source(Some("x"), &[])
        })
        .is_err());
    }

    #[test]
    fn rejects_missing_or_mixed_sources() {
        assert!(resolve(&source(Some("x"), &[]))
            .unwrap_err()
            .contains("Nothing to add"));
        assert!(resolve(&source(None, &[])).is_err());
        let command = vec!["npx".to_string()];
        assert!(resolve(&Source {
            url: Some("https://x"),
            ..source(Some("x"), &command)
        })
        .is_err());
        assert!(resolve(&Source {
            from: Some(Path::new("a.json")),
            ..source(Some("x"), &[])
        })
        .is_err());
        assert!(resolve(&source(Some("has space"), &command)).is_err());
    }

    #[test]
    fn reads_any_client_json_shape() {
        let resolved = from_json(
            r#"{ "mcpServers": {
                "github": { "command": "npx", "args": ["gh"], "env": { "TOKEN": "" } },
                "linear": { "url": "https://mcp.linear.app/mcp" }
            } }"#,
        )
        .unwrap();
        let ids = resolved
            .servers
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["github", "linear"]);

        assert!(from_json("{ nope").is_err());
    }

    #[test]
    fn env_pairs_reach_every_stdio_server() {
        let mut servers = from_json(r#"{ "mcpServers": { "a": { "command": "x" } } }"#)
            .unwrap()
            .servers;
        apply_env(&mut servers, &["TOKEN=abc=def".to_string()]).unwrap();
        assert_eq!(servers[0].command.as_ref().unwrap().env["TOKEN"], "abc=def");
        assert!(apply_env(&mut servers, &["NOEQUALS".to_string()]).is_err());
    }
}
