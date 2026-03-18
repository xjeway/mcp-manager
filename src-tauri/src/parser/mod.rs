use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
use serde_json::{Map, Value};
use std::collections::HashMap;
use toml::Table;

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub servers: Vec<MCPServer>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn parse_yaml_config(content: &str) -> Result<MCPConfig, String> {
    serde_yaml_compat::from_str(content)
}

fn strip_json_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '\0';
                    for next in chars.by_ref() {
                        if prev == '*' && next == '/' {
                            break;
                        }
                        if next == '\n' {
                            out.push('\n');
                        }
                        prev = next;
                    }
                    continue;
                }
                _ => {}
            }
        }

        out.push(ch);
    }

    out
}

fn parse_jsonc_value(content: &str) -> Result<Value, String> {
    serde_json::from_str(&strip_json_comments(content)).map_err(|e| e.to_string())
}

fn parse_server(id: &str, input: &Map<String, Value>) -> Result<MCPServer, String> {
    let command = input
        .get("command")
        .and_then(Value::as_str)
        .map(|program| crate::core::CommandSpec {
            program: program.to_string(),
            args: input
                .get("args")
                .and_then(Value::as_array)
                .map(|items| {
                    items.iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect()
                })
                .unwrap_or_default(),
            env: input
                .get("env")
                .and_then(Value::as_object)
                .map(|env| {
                    env.iter()
                        .map(|(key, value)| {
                            (
                                key.clone(),
                                value
                                    .as_str()
                                    .map(ToString::to_string)
                                    .unwrap_or_else(|| value.to_string()),
                            )
                        })
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
        });

    let transport = if let Some(url) = input.get("url").and_then(Value::as_str) {
        TransportSpec {
            kind: "http".to_string(),
            url: Some(url.to_string()),
        }
    } else if matches!(
        input.get("type").and_then(Value::as_str),
        Some("http") | Some("sse")
    ) {
        TransportSpec {
            kind: "http".to_string(),
            url: input
                .get("url")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        }
    } else {
        TransportSpec {
            kind: "stdio".to_string(),
            url: None,
        }
    };

    if command.is_none() && transport.url.is_none() {
        return Err(format!("server {id} 缺少 command/url"));
    }

    Ok(MCPServer {
        description: input
            .get("description")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string),
        homepage: input
            .get("homepage")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string),
        id: id.to_string(),
        name: input
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(id)
            .to_string(),
        enabled: input.get("enabled").and_then(Value::as_bool).unwrap_or(true),
        transport,
        command,
        apps: empty_apps(),
    })
}

pub fn parse_mcp_json(content: &str) -> ParseOutput {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut servers = Vec::new();

    let parsed: Value = match parse_jsonc_value(content) {
        Ok(value) => value,
        Err(error) => {
            return ParseOutput {
                servers,
                warnings,
                errors: vec![format!("JSON 解析失败: {error}")],
            }
        }
    };

    let Some(root) = parsed.as_object() else {
        return ParseOutput {
            servers,
            warnings,
            errors: vec!["JSON 顶层必须是对象".to_string()],
        };
    };

    let container = root
        .get("mcpServers")
        .and_then(Value::as_object)
        .or_else(|| root.get("servers").and_then(Value::as_object));

    if let Some(container) = container {
        for (id, value) in container {
            let Some(server) = value.as_object() else {
                warnings.push(format!("server {id} 不是对象，已忽略"));
                continue;
            };
            match parse_server(id, server) {
                Ok(parsed) => servers.push(parsed),
                Err(error) => errors.push(error),
            }
        }
    } else {
        match parse_server(
            root.get("id")
                .and_then(Value::as_str)
                .or_else(|| root.get("name").and_then(Value::as_str))
                .unwrap_or("imported-server"),
            root,
        ) {
            Ok(parsed) => servers.push(parsed),
            Err(error) => errors.push(error),
        }
    }

    ParseOutput {
        servers,
        warnings,
        errors,
    }
}

fn toml_value_to_json(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(v) => Value::String(v.clone()),
        toml::Value::Integer(v) => Value::Number((*v).into()),
        toml::Value::Float(v) => serde_json::Number::from_f64(*v)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(v) => Value::Bool(*v),
        toml::Value::Datetime(v) => Value::String(v.to_string()),
        toml::Value::Array(values) => Value::Array(values.iter().map(toml_value_to_json).collect()),
        toml::Value::Table(table) => Value::Object(
            table
                .iter()
                .map(|(key, value)| (key.clone(), toml_value_to_json(value)))
                .collect(),
        ),
    }
}

pub fn extract_codex_mcp_json(content: &str) -> Result<String, String> {
    let parsed: Table = toml::from_str(content).map_err(|e| e.to_string())?;
    let Some(mcp_servers) = parsed.get("mcp_servers").and_then(|value| value.as_table()) else {
        return Ok("{\"mcpServers\":{}}".to_string());
    };

    let mut servers = Map::new();
    for (server_id, config) in mcp_servers {
        let Some(table) = config.as_table() else {
            continue;
        };

        let mut server = Map::new();
        for (key, value) in table {
            server.insert(key.clone(), toml_value_to_json(value));
        }
        servers.insert(server_id.clone(), Value::Object(server));
    }

    serde_json::to_string(&Value::Object(Map::from_iter([(
        "mcpServers".to_string(),
        Value::Object(servers),
    )])))
    .map_err(|e| e.to_string())
}

pub fn extract_claude_mcp_json(content: &str, workspace_root: &str) -> Result<String, String> {
    let parsed: Value = parse_jsonc_value(content)?;
    let mut servers = Map::new();

    if let Some(global) = parsed.get("mcpServers").and_then(Value::as_object) {
        for (server_id, config) in global {
            servers.insert(server_id.clone(), config.clone());
        }
    }

    if let Some(project) = parsed
        .get("projects")
        .and_then(Value::as_object)
        .and_then(|projects| projects.get(workspace_root))
        .and_then(Value::as_object)
        .and_then(|project| project.get("mcpServers"))
        .and_then(Value::as_object)
    {
        for (server_id, config) in project {
            servers.insert(server_id.clone(), config.clone());
        }
    }

    serde_json::to_string(&Value::Object(Map::from_iter([(
        "mcpServers".to_string(),
        Value::Object(servers),
    )])))
    .map_err(|e| e.to_string())
}

pub fn extract_json_field_mcp_json(content: &str, field: &str) -> Result<String, String> {
    let parsed = parse_jsonc_value(content)?;
    let servers = parsed
        .as_object()
        .and_then(|root| root.get(field))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    serde_json::to_string(&Value::Object(Map::from_iter([(
        "mcpServers".to_string(),
        Value::Object(servers),
    )])))
    .map_err(|e| e.to_string())
}

pub fn extract_opencode_mcp_json(content: &str) -> Result<String, String> {
    let parsed = parse_jsonc_value(content)?;
    let Some(root) = parsed.as_object() else {
        return Err("OpenCode config 顶层必须是对象".to_string());
    };
    let Some(mcp) = root.get("mcp").and_then(Value::as_object) else {
        return Ok("{\"mcpServers\":{}}".to_string());
    };

    let mut servers = Map::new();
    for (server_id, config) in mcp {
        let Some(config) = config.as_object() else {
            continue;
        };

        let mut server = Map::new();
        match config.get("type").and_then(Value::as_str) {
            Some("local") => {
                let command = config
                    .get("command")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if let Some(program) = command.first().and_then(Value::as_str) {
                    server.insert("command".to_string(), Value::String(program.to_string()));
                    server.insert(
                        "args".to_string(),
                        Value::Array(command.into_iter().skip(1).collect()),
                    );
                }
                if let Some(env) = config.get("environment").and_then(Value::as_object) {
                    server.insert("env".to_string(), Value::Object(env.clone()));
                }
            }
            Some("remote") => {
                if let Some(url) = config.get("url").and_then(Value::as_str) {
                    server.insert("url".to_string(), Value::String(url.to_string()));
                    server.insert("type".to_string(), Value::String("http".to_string()));
                }
            }
            _ => continue,
        }
        servers.insert(server_id.clone(), Value::Object(server));
    }

    serde_json::to_string(&Value::Object(Map::from_iter([(
        "mcpServers".to_string(),
        Value::Object(servers),
    )])))
    .map_err(|e| e.to_string())
}

pub fn enable_servers_for_app(mut servers: Vec<MCPServer>, app: SupportedApp) -> Vec<MCPServer> {
    for server in &mut servers {
        server.apps.insert(app, true);
    }
    servers
}

mod serde_yaml_compat {
    use crate::core::MCPConfig;

    pub fn from_str(content: &str) -> Result<MCPConfig, String> {
        serde_yaml::from_str(content).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        extract_claude_mcp_json, extract_codex_mcp_json, extract_json_field_mcp_json,
        extract_opencode_mcp_json, parse_mcp_json,
    };
    use serde_json::Value;

    #[test]
    fn parses_json_server_containers() {
        let parsed = parse_mcp_json(
            r#"{"servers":{"github":{"command":"uvx","args":["mcp-server-github"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
        assert_eq!(parsed.servers[0].id, "github");
    }

    #[test]
    fn extracts_codex_servers() {
        let json = extract_codex_mcp_json(
            r#"[mcp_servers.playwright]
command = "npx"
args = ["@playwright/mcp@latest"]
"#,
        )
        .expect("extract codex");
        let value: Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["mcpServers"]["playwright"]["command"], "npx");
    }

    #[test]
    fn extracts_claude_project_and_global_servers() {
        let workspace = "/workspace/project";
        let json = extract_claude_mcp_json(
            r#"{"mcpServers":{"linear":{"url":"https://mcp.linear.app/mcp"}},"projects":{"/workspace/project":{"mcpServers":{"playwright":{"command":"npx"}}}}}"#,
            workspace,
        )
        .expect("extract claude");
        let value: Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["mcpServers"]["linear"]["url"], "https://mcp.linear.app/mcp");
        assert_eq!(value["mcpServers"]["playwright"]["command"], "npx");
    }

    #[test]
    fn extracts_json_field_servers() {
        let json = extract_json_field_mcp_json(
            r#"{"theme":"dark","mcpServers":{"linear":{"url":"https://mcp.linear.app/mcp"}}}"#,
            "mcpServers",
        )
        .expect("extract field");
        let value: Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["mcpServers"]["linear"]["url"], "https://mcp.linear.app/mcp");
    }

    #[test]
    fn extracts_opencode_servers() {
        let json = extract_opencode_mcp_json(
            r#"{
  // comment
  "mcp": {
    "playwright": {
      "type": "local",
      "command": ["npx", "@playwright/mcp@latest"],
      "environment": { "FOO": "bar" }
    }
  }
}"#,
        )
        .expect("extract opencode");
        let value: Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["mcpServers"]["playwright"]["command"], "npx");
        assert_eq!(
            value["mcpServers"]["playwright"]["args"][0],
            "@playwright/mcp@latest"
        );
        assert_eq!(value["mcpServers"]["playwright"]["env"]["FOO"], "bar");
    }
}
