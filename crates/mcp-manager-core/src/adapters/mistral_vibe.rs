use crate::adapters::{
    managed_toml_array_writes, scope_for_path, vibe_mcp_servers_at, AppAdapter, ParsedSources,
};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, parse_mcp_json};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};

pub struct MistralVibeAdapter;

const FIELD: &str = "mcp_servers";

/// Splits a command line the way Python's `shlex.split`, which Vibe uses for a
/// string `command`, does in POSIX mode: whitespace separates words, quotes
/// group them, and a backslash escapes the next character outside single
/// quotes.
fn split_command(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        match ch {
            ch if ch.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut word));
                    in_word = false;
                }
            }
            '\'' => {
                in_word = true;
                for next in chars.by_ref() {
                    if next == '\'' {
                        break;
                    }
                    word.push(next);
                }
            }
            '"' => {
                in_word = true;
                while let Some(next) = chars.next() {
                    match next {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some(escaped @ ('"' | '\\' | '$' | '`')) => word.push(escaped),
                            Some(other) => {
                                word.push('\\');
                                word.push(other);
                            }
                            None => word.push('\\'),
                        },
                        other => word.push(other),
                    }
                }
            }
            '\\' => {
                in_word = true;
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            }
            other => {
                in_word = true;
                word.push(other);
            }
        }
    }
    if in_word {
        words.push(word);
    }
    words
}

/// `[[mcp_servers]]` as an `mcpServers` document the shared parser reads.
fn normalize(content: &str) -> Result<String, String> {
    let root: toml::Table = toml::from_str(content).map_err(|e| e.to_string())?;
    let mut servers = Map::new();
    let entries = root
        .get(FIELD)
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for entry in entries {
        let Ok(Value::Object(entry)) = serde_json::to_value(entry) else {
            continue;
        };
        let Some(name) = entry.get("name").and_then(Value::as_str) else {
            continue;
        };
        let mut server = Map::new();
        if entry.get("transport").and_then(Value::as_str) == Some("stdio") {
            let mut argv = match entry.get("command") {
                Some(Value::String(line)) => split_command(line),
                Some(Value::Array(words)) => words
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect(),
                _ => Vec::new(),
            };
            if argv.is_empty() {
                continue;
            }
            argv.extend(
                entry
                    .get("args")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_string),
            );
            server.insert("command".into(), Value::String(argv.remove(0)));
            server.insert("args".into(), serde_json::json!(argv));
            if let Some(env) = entry.get("env") {
                server.insert("env".into(), env.clone());
            }
        } else {
            let Some(url) = entry.get("url").and_then(Value::as_str) else {
                continue;
            };
            server.insert("url".into(), Value::String(url.to_string()));
            server.insert("type".into(), Value::String("http".to_string()));
            // Static headers live in `auth`; older files have them at the top.
            let headers = entry
                .get("auth")
                .filter(|auth| auth.get("type").and_then(Value::as_str) != Some("oauth"))
                .and_then(|auth| auth.get("headers"))
                .or_else(|| entry.get("headers"));
            if let Some(headers) = headers {
                server.insert("headers".into(), headers.clone());
            }
        }
        if let Some(disabled) = entry.get("disabled") {
            server.insert("disabled".into(), disabled.clone());
        }
        servers.insert(name.to_string(), Value::Object(server));
    }
    let mut document = Map::new();
    document.insert("mcpServers".to_string(), Value::Object(servers));
    serde_json::to_string(&Value::Object(document)).map_err(|e| e.to_string())
}

impl AppAdapter for MistralVibeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::MistralVibe
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            // Vibe only loads it from a trusted folder.
            (
                ctx.workspace_file(".vibe/config.toml")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.user_app_config_path(self.app())
                    .to_string_lossy()
                    .to_string(),
                20,
            ),
        ]
    }

    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let source = |content: String| LocalConfigSource {
            app: self.app().as_str().to_string(),
            path: path.to_string(),
            exists: true,
            format: "json".to_string(),
            priority,
            content: Some(content),
        };
        let normalized = match normalize(content) {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![source(content.to_string())],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![error],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![source(normalized)],
            servers: enable_servers_for_app_at(
                parsed.servers,
                self.app(),
                scope_for_path(ctx, path),
                path.to_string(),
            )
            .into_iter()
            .map(|server| (server, priority))
            .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous_config: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        managed_toml_array_writes(
            ctx,
            FIELD,
            self.app(),
            config,
            previous_config,
            vibe_mcp_servers_at,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{split_command, MistralVibeAdapter};
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, CommandSpec, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn splits_commands_like_shlex() {
        assert_eq!(
            split_command("uvx mcp-server-fetch"),
            ["uvx", "mcp-server-fetch"]
        );
        assert_eq!(
            split_command(r#"'/Applications/My Tools/mcp' --name "a b" c\ d"#),
            ["/Applications/My Tools/mcp", "--name", "a b", "c d"]
        );
        assert_eq!(split_command(r#"a "" b"#), ["a", "", "b"]);
        assert!(split_command("   ").is_empty());
    }

    const CONFIG: &str = r#"# My Vibe setup
active_model = "devstral"

[[mcp_servers]]
name = "fetch_server"
transport = "stdio"
command = "uvx mcp-server-fetch"
args = ["--verbose"]
tool_timeout_sec = 120

[[mcp_servers]]
name = "linear"
transport = "streamable-http"
url = "https://mcp.linear.app/mcp"

[mcp_servers.auth]
type = "oauth"
scopes = []

[[mcp_servers]]
name = "docs"
transport = "http"
url = "https://docs.example.com/mcp"
headers = { Authorization = "Bearer legacy" }
api_key_env = "DOCS_KEY"

[tools.fetch_server_get]
permission = "always"
"#;

    fn ctx(home: PathBuf) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.clone(),
            workspace_root: home,
        }
    }

    #[test]
    fn imports_stdio_oauth_and_legacy_header_servers() {
        let parsed = MistralVibeAdapter.parse_source(
            &ctx(PathBuf::from("/home/test")),
            "/home/test/.vibe/config.toml",
            20,
            CONFIG,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let server = |id: &str| {
            parsed
                .servers
                .iter()
                .find(|(server, _)| server.id == id)
                .map(|(server, _)| server.clone())
                .unwrap_or_else(|| panic!("{id} imported"))
        };
        let fetch = server("fetch_server");
        let command = fetch.command.as_ref().unwrap();
        assert_eq!(command.program, "uvx");
        assert_eq!(command.args, ["mcp-server-fetch", "--verbose"]);
        assert_eq!(
            server("linear").transport.url.as_deref(),
            Some("https://mcp.linear.app/mcp")
        );
        assert!(server("linear").transport.headers.is_empty());
        assert_eq!(
            server("docs").transport.headers["Authorization"],
            "Bearer legacy"
        );
    }

    #[test]
    fn apply_rewrites_entries_in_place_and_keeps_the_rest_of_the_file() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(home.path().to_path_buf());
        let path = ctx.user_app_config_path(SupportedApp::MistralVibe);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, CONFIG).unwrap();

        let parsed = MistralVibeAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, CONFIG);
        let mut servers = parsed
            .servers
            .into_iter()
            .map(|(server, _)| server)
            .collect::<Vec<_>>();
        for server in &mut servers {
            match server.id.as_str() {
                "fetch_server" => {
                    server.command.as_mut().unwrap().program = "/opt/My Tools/fetch".into();
                }
                "docs" => {
                    server.transport.headers =
                        [("Authorization".to_string(), "Bearer new".to_string())].into();
                }
                _ => {}
            }
        }
        let mut apps = empty_apps();
        apps.insert(SupportedApp::MistralVibe, true);
        let remote = |id: &str, kind: &str| MCPServer {
            id: id.to_string(),
            name: id.to_string(),
            enabled: true,
            transport: TransportSpec {
                kind: kind.to_string(),
                url: Some(format!("https://{id}.example.com/mcp")),
                headers: Default::default(),
            },
            command: None,
            apps: apps.clone(),
            placements: vec![],
            description: None,
            homepage: None,
        };
        servers.push(remote("fresh", "http"));
        servers.push(remote("legacy-sse", "sse"));
        servers.push(MCPServer {
            command: Some(CommandSpec {
                program: "npx".into(),
                args: vec!["-y".into(), "ctx".into()],
                env: [("A".to_string(), "1".to_string())].into(),
            }),
            transport: TransportSpec {
                kind: "stdio".into(),
                url: None,
                headers: Default::default(),
            },
            ..remote("ctx", "stdio")
        });
        let config = MCPConfig {
            version: 1,
            servers,
        };
        for operation in MistralVibeAdapter.plan_apply(&ctx, &config, None) {
            assert_eq!(operation.mode, "merge_toml_array_entries");
            apply_operation(&PathBuf::from(&operation.path), &operation).expect("apply");
        }

        let written = fs::read_to_string(&path).unwrap();
        assert!(
            written.starts_with("# My Vibe setup\nactive_model = \"devstral\"\n"),
            "{written}"
        );
        let parsed: toml::Table = toml::from_str(&written).expect("valid TOML");
        assert_eq!(
            parsed["tools"]["fetch_server_get"]["permission"].as_str(),
            Some("always")
        );
        let servers = parsed["mcp_servers"].as_array().unwrap();
        let entry = |name: &str| {
            servers
                .iter()
                .find(|server| server["name"].as_str() == Some(name))
                .unwrap_or_else(|| panic!("{name} written:\n{written}"))
        };
        let names = servers
            .iter()
            .map(|server| server["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(names, ["fetch_server", "linear", "docs", "ctx", "fresh"]);

        // A program with a space is written as a list; the tuned timeout stays.
        let fetch = entry("fetch_server");
        assert_eq!(
            fetch["command"].as_array().unwrap()[0].as_str(),
            Some("/opt/My Tools/fetch")
        );
        assert_eq!(fetch["tool_timeout_sec"].as_integer(), Some(120));
        // OAuth is not modeled, so the entry keeps its auth table.
        assert_eq!(entry("linear")["auth"]["type"].as_str(), Some("oauth"));
        // Legacy header keys move into `auth`, which Vibe requires alone, so
        // the API-key setting survives the rewrite.
        let docs = entry("docs");
        assert_eq!(docs["auth"]["type"].as_str(), Some("static"));
        assert_eq!(docs["auth"]["api_key_env"].as_str(), Some("DOCS_KEY"));
        assert_eq!(
            docs["auth"]["headers"]["Authorization"].as_str(),
            Some("Bearer new")
        );
        assert!(
            docs.get("headers").is_none() && docs.get("api_key_env").is_none(),
            "{written}"
        );
        assert_eq!(docs["transport"].as_str(), Some("streamable-http"));
        assert_eq!(
            entry("fresh")["transport"].as_str(),
            Some("streamable-http")
        );
        assert!(entry("fresh").get("type").is_none());
        let ctx_entry = entry("ctx");
        assert_eq!(ctx_entry["command"].as_str(), Some("npx"));
        assert_eq!(ctx_entry["env"]["A"].as_str(), Some("1"));
        // Vibe has no SSE client.
        assert!(!names.contains(&"legacy-sse"));
    }

    #[test]
    fn sse_servers_are_reported_instead_of_written() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::MistralVibe, true);
        apps.insert(SupportedApp::Cursor, true);
        let warnings = crate::adapters::apply_warnings(&MCPConfig {
            version: 1,
            servers: vec![MCPServer {
                id: "old".into(),
                name: "old".into(),
                enabled: true,
                transport: TransportSpec {
                    kind: "sse".into(),
                    url: Some("https://old.example.com/sse".into()),
                    headers: Default::default(),
                },
                command: None,
                apps,
                placements: vec![],
                description: None,
                homepage: None,
            }],
        });
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].kind, "sseUnsupported");
        assert_eq!(warnings[0].app, SupportedApp::MistralVibe);
    }
}
