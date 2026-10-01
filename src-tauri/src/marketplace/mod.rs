//! Online MCP marketplace: sources, adapters and the unified entry model.
//!
//! A marketplace is a list of sources; each source kind has an adapter that fetches
//! and normalizes entries. The frontend only ever sees [`MarketplaceEntry`].

mod cache;
mod http;
mod install;
mod registry;
mod service;
mod sources;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Built-in sources ship with the app; the others were added by the user and can
    /// be removed.
    #[serde(default)]
    pub builtin: bool,
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
            builtin: true,
        },
        MarketplaceSource {
            id: "official".to_string(),
            kind: SourceKind::McpRegistry,
            label: "MCP Registry".to_string(),
            base_url: "https://registry.modelcontextprotocol.io".to_string(),
            trust: SourceTrust::Community,
            server_search: true,
            builtin: true,
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
        /// Whether the package reference is immutable, missing, or a moving tag.
        /// `"latest"` stays [`PinStatus::Unpinned`] and is not rewritten as a pin.
        pin_status: PinStatus,
        /// The publisher named an executable this app does not recognise.
        /// Registry inclusion is not a review of that program.
        arbitrary_runtime: bool,
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

/// How reproducible a marketplace package reference is.
///
/// A digest is pinned. A version tag such as `1.2.3` on an OCI image can move,
/// so it is [`PinStatus::MutableTag`]. A missing version or `"latest"` is
/// [`PinStatus::Unpinned`] and is never displayed as if it were pinned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PinStatus {
    Pinned,
    Unpinned,
    MutableTag,
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
    /// A source URL the user entered is not a usable https address.
    InvalidUrl(String),
    /// A source with the same URL is already in the list.
    DuplicateSource(String),
}

impl std::fmt::Display for MarketplaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplaceError::Disabled => write!(f, "marketplace is disabled"),
            MarketplaceError::UnknownSource(id) => write!(f, "unknown marketplace source: {id}"),
            MarketplaceError::Network(message) => write!(f, "network error: {message}"),
            MarketplaceError::Parse(message) => write!(f, "invalid response: {message}"),
            MarketplaceError::InvalidUrl(message) => write!(f, "invalid source URL: {message}"),
            MarketplaceError::DuplicateSource(url) => write!(f, "source already added: {url}"),
        }
    }
}
