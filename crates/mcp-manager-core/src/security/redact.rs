//! Removes secret values before they are written to history or logs.
//!
//! Header values are always secrets: the config model treats every HTTP header
//! as credential material. Environment values are redacted when the key is
//! recorded as secret (`CommandSpec::secret_env`, set from marketplace
//! `secret: true` inputs) or when the key or value looks like a credential.
//! Rollback does not read these redacted copies. It restores `servers.yaml`
//! from the protected config backup.

use crate::core::{CommandSpec, MCPConfig, MCPServer};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const REDACTED: &str = "[redacted]";

/// Stable identity of a config, including secret values. The digest is not
/// reversible; history stores it so a later edit can be detected without
/// keeping the secrets themselves.
pub fn config_fingerprint(config: &MCPConfig) -> Result<String, String> {
    let value = serde_json::to_value(config).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    Ok(hex_encode(Sha256::digest(bytes)))
}

pub fn redact_config(config: &MCPConfig) -> MCPConfig {
    let mut redacted = config.clone();
    for server in &mut redacted.servers {
        redact_server(server);
    }
    redacted
}

/// Replaces likely secrets in a free-form string. Used for log lines, not for
/// config files the client has to execute.
pub fn redact_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for token in text.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        if value_looks_secret(token) {
            out.push_str(REDACTED);
        } else {
            out.push_str(token);
        }
    }
    out
}

pub fn value_looks_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == REDACTED {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    const PREFIXES: &[&str] = &[
        "ghp_",
        "github_pat_",
        "gho_",
        "ghu_",
        "ghs_",
        "sk-",
        "sk_",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xoxs-",
        "glpat-",
        "pypi-",
        "ya29.",
        "bearer ",
        "akia",
    ];
    if PREFIXES.iter().any(|prefix| lower.starts_with(prefix)) {
        return true;
    }
    // A long single token with few separators is treated as a credential.
    // Package specifiers such as `@scope/name@1.2.3` contain extra punctuation
    // and are left alone.
    if trimmed.len() >= 32
        && !trimmed.contains([' ', '/', '@', ':'])
        && trimmed.chars().any(|c| c.is_ascii_digit())
        && trimmed.chars().any(|c| c.is_ascii_alphabetic())
    {
        return true;
    }
    false
}

pub fn env_name_is_secret(key: &str) -> bool {
    let normalized = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    const MARKERS: &[&str] = &[
        "SECRET",
        "TOKEN",
        "PASSWORD",
        "PASSWD",
        "CREDENTIAL",
        "API_KEY",
        "APIKEY",
        "PRIVATE_KEY",
        "ACCESS_KEY",
        "CLIENT_SECRET",
        "AUTHORIZATION",
        "SESSION",
    ];
    MARKERS.iter().any(|marker| {
        normalized == *marker
            || normalized.ends_with(&format!("_{marker}"))
            || normalized.contains(marker)
    })
}

fn redact_server(server: &mut MCPServer) {
    if let Some(url) = server.transport.url.as_mut() {
        *url = redact_url(url);
    }
    for value in server.transport.headers.values_mut() {
        if !value.is_empty() {
            *value = REDACTED.to_string();
        }
    }
    if let Some(command) = server.command.as_mut() {
        redact_command(command);
    }
}

fn redact_command(command: &mut CommandSpec) {
    let forced: BTreeSet<String> = command
        .secret_env
        .iter()
        .map(|key| key.to_ascii_lowercase())
        .collect();
    for (key, value) in command.env.iter_mut() {
        if forced.contains(&key.to_ascii_lowercase())
            || env_name_is_secret(key)
            || value_looks_secret(value)
        {
            *value = REDACTED.to_string();
        }
    }
    let mut redact_next = false;
    for arg in command.args.iter_mut() {
        if redact_next {
            *arg = REDACTED.to_string();
            redact_next = false;
            continue;
        }
        if let Some((flag, inline)) = split_secret_flag(arg) {
            if inline.is_some() {
                *arg = format!("{flag}={REDACTED}");
            } else {
                redact_next = true;
            }
            continue;
        }
        if value_looks_secret(arg) {
            *arg = REDACTED.to_string();
        }
    }
}

fn split_secret_flag(arg: &str) -> Option<(String, Option<String>)> {
    let (flag, inline) = match arg.split_once('=') {
        Some((flag, value)) => (flag, Some(value.to_string())),
        None => (arg, None),
    };
    let name = flag.trim_start_matches('-');
    if name.is_empty() || flag == name {
        return None;
    }
    env_name_is_secret(name).then(|| (flag.to_string(), inline))
}

fn redact_url(url: &str) -> String {
    let Some(scheme) = url.find("://") else {
        return url.to_string();
    };
    let rest = &url[scheme + 3..];
    let Some(at) = rest.find('@') else {
        return url.to_string();
    };
    let userinfo = &rest[..at];
    let Some((user, _)) = userinfo.split_once(':') else {
        return url.to_string();
    };
    if user.is_empty() {
        return url.to_string();
    }
    format!("{}://{user}:{REDACTED}@{}", &url[..scheme], &rest[at + 1..])
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CommandSpec, MCPConfig, MCPServer, TransportSpec};
    use std::collections::{BTreeSet, HashMap};

    fn server() -> MCPServer {
        MCPServer {
            description: None,
            homepage: None,
            id: "demo".to_string(),
            name: "Demo".to_string(),
            enabled: true,
            transport: TransportSpec {
                kind: "http".to_string(),
                url: Some(
                    "https://user:ghp_SUPERSECRETVALUE1234567890@example.com/mcp".to_string(),
                ),
                headers: [(
                    "Authorization".to_string(),
                    "Bearer ghp_SUPERSECRETVALUE1234567890".to_string(),
                )]
                .into_iter()
                .collect(),
            },
            command: Some(CommandSpec {
                program: "npx".to_string(),
                args: vec![
                    "--token".to_string(),
                    "ghp_SUPERSECRETVALUE1234567890".to_string(),
                ],
                env: HashMap::from([
                    (
                        "API_TOKEN".to_string(),
                        "ghp_SUPERSECRETVALUE1234567890".to_string(),
                    ),
                    ("MODE".to_string(), "stdio".to_string()),
                    (
                        "REGION".to_string(),
                        "not-a-typical-name-but-marked".to_string(),
                    ),
                ]),
                secret_env: BTreeSet::from(["REGION".to_string()]),
            }),
            apps: HashMap::new(),
            placements: Vec::new(),
        }
    }

    #[test]
    fn redacts_headers_marked_env_and_token_shapes_but_keeps_ordinary_env() {
        let redacted = redact_config(&MCPConfig {
            version: 1,
            servers: vec![server()],
        });
        let text = serde_json::to_string(&redacted).expect("json");
        assert!(!text.contains("ghp_SUPERSECRETVALUE1234567890"), "{text}");
        assert!(!text.contains("not-a-typical-name-but-marked"), "{text}");
        assert!(text.contains("\"MODE\":\"stdio\""), "{text}");
        assert!(text.contains(REDACTED), "{text}");
        assert!(text.contains("example.com"), "{text}");
    }

    #[test]
    fn fingerprints_change_when_a_secret_changes() {
        let mut config = MCPConfig {
            version: 1,
            servers: vec![server()],
        };
        let first = config_fingerprint(&config).expect("fingerprint");
        config.servers[0].transport.headers.insert(
            "Authorization".to_string(),
            "Bearer other-token-value-1234567890".to_string(),
        );
        let second = config_fingerprint(&config).expect("fingerprint");
        assert_ne!(first, second);
        assert!(!first.contains("ghp_"));
    }
}
