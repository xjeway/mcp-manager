//! Online MCP marketplace: sources, adapters and the unified entry model.
//!
//! A marketplace is a list of sources; each source kind has an adapter that fetches
//! and normalizes entries. The frontend only ever sees [`MarketplaceEntry`].

mod cache;
mod http;
mod install;
mod registry;
mod service;

pub use http::ReqwestClient;
pub use service::Marketplace;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// Any endpoint implementing the MCP Registry API (`/v0/servers`).
    McpRegistry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceTrust {
    Curated,
    Official,
    Community,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSource {
    pub id: String,
    pub kind: SourceKind,
    pub label: String,
    pub base_url: String,
    pub trust: SourceTrust,
    /// Whether the source honours the `search` query parameter. Sources that do not
    /// are fetched in full and filtered locally.
    pub server_search: bool,
}

pub fn builtin_sources() -> Vec<MarketplaceSource> {
    vec![
        MarketplaceSource {
            id: "github".to_string(),
            kind: SourceKind::McpRegistry,
            label: "GitHub MCP Registry".to_string(),
            base_url: "https://api.mcp.github.com/2025-09-15".to_string(),
            trust: SourceTrust::Curated,
            server_search: false,
        },
        MarketplaceSource {
            id: "official".to_string(),
            kind: SourceKind::McpRegistry,
            label: "MCP Registry".to_string(),
            base_url: "https://registry.modelcontextprotocol.io".to_string(),
            trust: SourceTrust::Community,
            server_search: true,
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceEntry {
    pub source_id: String,
    /// Registry name, e.g. `io.github.upstash/context7`.
    pub id: String,
    pub title: String,
    pub description: String,
    pub version: Option<String>,
    pub repository_url: Option<String>,
    pub website_url: Option<String>,
    pub stars: Option<u64>,
    pub readme_excerpt: Option<String>,
    pub install_options: Vec<InstallOption>,
}

/// One way to run an entry. String values may contain `{key}` placeholders that refer
/// to an [`InstallInput`] with the same key; the frontend substitutes user input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum InstallOption {
    Stdio {
        package_type: PackageType,
        identifier: String,
        program: String,
        /// Argument groups: a named flag and its value stay together so an optional
        /// flag can be dropped as a unit when its placeholder is left empty.
        arg_groups: Vec<Vec<String>>,
        env: BTreeMap<String, String>,
        inputs: Vec<InstallInput>,
        /// True when the package type or runtime was guessed rather than declared.
        inferred: bool,
    },
    Http {
        transport: String,
        url: String,
        headers: BTreeMap<String, String>,
        inputs: Vec<InstallInput>,
    },
    Unsupported {
        package_type: String,
        identifier: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    Npm,
    Pypi,
    Oci,
    /// A publisher-declared runtime we do not model (e.g. `uv`, `python`).
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInput {
    pub key: String,
    pub description: Option<String>,
    pub required: bool,
    pub secret: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub entries: Vec<MarketplaceEntry>,
    pub next_cursor: Option<String>,
    /// Records the adapter could not parse and left out.
    pub skipped: usize,
    /// True when the source could not be reached and cached data was returned.
    pub stale: bool,
    /// Unix seconds when the data was fetched from the source.
    pub fetched_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", content = "message", rename_all = "kebab-case")]
pub enum MarketplaceError {
    Disabled,
    UnknownSource(String),
    Network(String),
    Parse(String),
}

impl std::fmt::Display for MarketplaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplaceError::Disabled => write!(f, "marketplace is disabled"),
            MarketplaceError::UnknownSource(id) => write!(f, "unknown marketplace source: {id}"),
            MarketplaceError::Network(message) => write!(f, "network error: {message}"),
            MarketplaceError::Parse(message) => write!(f, "invalid response: {message}"),
        }
    }
}
