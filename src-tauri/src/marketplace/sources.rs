//! Sources the user added, persisted as JSON in the app data directory.

use super::cache::fnv1a;
use super::{MarketplaceError, MarketplaceSource};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Longest label kept for a user source, so the source tabs stay readable.
const MAX_LABEL_CHARS: usize = 40;

pub(super) struct SourceStore {
    path: PathBuf,
}

impl SourceStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// A missing or unreadable file means no user sources; built-in ones still work.
    pub fn load(&self) -> Vec<MarketplaceSource> {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|content| serde_json::from_str::<Vec<MarketplaceSource>>(&content).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|source| MarketplaceSource {
                builtin: false,
                ..source
            })
            .collect()
    }

    pub fn save(&self, sources: &[MarketplaceSource]) -> Result<(), MarketplaceError> {
        let content = serde_json::to_string_pretty(sources)
            .map_err(|e| MarketplaceError::Parse(e.to_string()))?;
        let io_error = |e: std::io::Error| MarketplaceError::Network(e.to_string());
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(io_error)?;
        }
        // Written aside and renamed over the target, so a failed write never leaves a
        // truncated file. The name is unique per process and write, so saves from other
        // app instances never share a temporary file.
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let temp = self.path.with_extension(format!(
            "json.{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let result = fs::write(&temp, content).and_then(|()| fs::rename(&temp, &self.path));
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result.map_err(io_error)
    }
}

/// Accepts a registry base URL, or the full `/v0/servers` endpoint pasted from docs,
/// and returns the base URL without a trailing slash. Only https is allowed: the
/// registry decides which commands the user is offered to run.
pub(super) fn normalize_base_url(input: &str) -> Result<String, MarketplaceError> {
    let invalid = |reason: &str| MarketplaceError::InvalidUrl(reason.to_string());
    let mut url = reqwest::Url::parse(input.trim()).map_err(|e| invalid(&e.to_string()))?;
    if url.scheme() != "https" {
        return Err(invalid("only https URLs are supported"));
    }
    if url.host_str().is_none() {
        return Err(invalid("missing host"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(invalid("URLs with credentials are not supported"));
    }
    url.set_query(None);
    url.set_fragment(None);

    let mut path = url.path().trim_end_matches('/').to_string();
    for suffix in ["/v0/servers", "/v0"] {
        if let Some(stripped) = path.strip_suffix(suffix) {
            path = stripped.to_string();
            break;
        }
    }
    url.set_path(&path);
    Ok(url.as_str().trim_end_matches('/').to_string())
}

/// Stable per URL, so the cache stays valid if a source is removed and added again.
pub(super) fn source_id(base_url: &str) -> String {
    format!("custom-{:016x}", fnv1a(base_url))
}

pub(super) fn source_label(label: &str, base_url: &str) -> String {
    let label = label.trim();
    let label = if label.is_empty() {
        reqwest::Url::parse(base_url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_else(|| base_url.to_string())
    } else {
        label.to_string()
    };
    label.chars().take(MAX_LABEL_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::{SourceKind, SourceTrust};

    #[test]
    fn normalizes_base_urls() {
        for (input, expected) in [
            (
                "https://registry.example.com",
                "https://registry.example.com",
            ),
            (
                " https://registry.example.com/ ",
                "https://registry.example.com",
            ),
            (
                "https://example.com/mcp/v0/servers?limit=5",
                "https://example.com/mcp",
            ),
            ("https://example.com/mcp/v0/", "https://example.com/mcp"),
            (
                "https://api.mcp.github.com/2025-09-15/",
                "https://api.mcp.github.com/2025-09-15",
            ),
        ] {
            assert_eq!(
                normalize_base_url(input).as_deref(),
                Ok(expected),
                "{input}"
            );
        }
    }

    #[test]
    fn rejects_unusable_urls() {
        for input in [
            "",
            "registry.example.com",
            "http://registry.example.com",
            "file:///etc/passwd",
            "https://user:secret@example.com",
        ] {
            assert!(
                matches!(
                    normalize_base_url(input),
                    Err(MarketplaceError::InvalidUrl(_))
                ),
                "{input}"
            );
        }
    }

    #[test]
    fn labels_default_to_the_host_and_are_capped() {
        assert_eq!(
            source_label("  ", "https://mcp.acme.dev/api"),
            "mcp.acme.dev"
        );
        assert_eq!(source_label(" Acme ", "https://mcp.acme.dev"), "Acme");
        assert_eq!(
            source_label(&"x".repeat(100), "https://a.dev").len(),
            MAX_LABEL_CHARS
        );
    }

    #[test]
    fn round_trips_sources_and_never_loads_them_as_builtin() {
        let dir = tempfile::tempdir().unwrap();
        let store = SourceStore::new(dir.path().join("nested").join("sources.json"));
        assert!(store.load().is_empty());

        let source = MarketplaceSource {
            id: source_id("https://mcp.acme.dev"),
            kind: SourceKind::McpRegistry,
            label: "Acme".to_string(),
            base_url: "https://mcp.acme.dev".to_string(),
            trust: SourceTrust::Community,
            server_search: true,
            builtin: true,
        };
        store.save(std::slice::from_ref(&source)).unwrap();
        assert_eq!(
            store.load(),
            [MarketplaceSource {
                builtin: false,
                ..source
            }]
        );
    }

    #[test]
    fn saving_leaves_no_temporary_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = SourceStore::new(dir.path().join("sources.json"));
        store.save(&[]).unwrap();
        store.save(&[]).unwrap();
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["sources.json"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_save_keeps_the_previous_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sources.json");
        fs::write(&path, "[]").unwrap();
        // A read-only directory makes writing the temporary file fail.
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
        let result = SourceStore::new(path.clone()).save(&[]);
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "[]");
    }

    #[test]
    fn a_corrupt_file_loads_as_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sources.json");
        fs::write(&path, "not json").unwrap();
        assert!(SourceStore::new(path).load().is_empty());
    }
}
