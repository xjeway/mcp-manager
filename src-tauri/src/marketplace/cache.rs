//! On-disk cache of normalized search pages, used both to avoid refetching and as a
//! fallback when a source is unreachable.

use super::SearchPage;
use std::fs;
use std::path::PathBuf;

pub(super) const CACHE_TTL_SECS: u64 = 6 * 60 * 60;

pub(super) struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn read(&self, key: &str) -> Option<SearchPage> {
        let content = fs::read_to_string(self.path(key)).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Best effort: a cache that cannot be written only costs a refetch.
    pub fn write(&self, key: &str, page: &SearchPage) {
        let Ok(content) = serde_json::to_string(page) else {
            return;
        };
        if fs::create_dir_all(&self.dir).is_ok() {
            let _ = mcp_manager_core::storage::atomic_write(&self.path(key), &content);
        }
    }

    pub fn is_fresh(page: &SearchPage, now: u64) -> bool {
        now >= page.fetched_at && now - page.fetched_at < CACHE_TTL_SECS
    }

    /// Keys contain queries and cursors, so they are hashed into a safe file name.
    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{:016x}.json", fnv1a(key)))
    }
}

/// Stable across Rust releases, unlike `DefaultHasher`, so cache files stay valid.
pub(super) fn fnv1a(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(fetched_at: u64) -> SearchPage {
        SearchPage {
            entries: Vec::new(),
            next_cursor: Some("next".to_string()),
            skipped: 2,
            stale: false,
            fetched_at,
        }
    }

    #[test]
    fn round_trips_a_page_by_key() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("marketplace-cache"));
        assert_eq!(cache.read("github|all"), None);

        cache.write("github|all", &page(100));
        assert_eq!(cache.read("github|all"), Some(page(100)));
        assert_eq!(cache.read("official|q=x"), None);
    }

    #[test]
    fn keys_with_path_characters_stay_inside_the_cache_dir() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        let cache = Cache::new(cache_dir.clone());
        cache.write("../../etc/passwd|q=a/b", &page(1));
        let files: Vec<_> = std::fs::read_dir(&cache_dir).unwrap().collect();
        assert_eq!(files.len(), 1);
        assert_eq!(cache.read("../../etc/passwd|q=a/b"), Some(page(1)));
    }

    #[test]
    fn ignores_a_corrupt_cache_file() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        cache.write("k", &page(1));
        for file in std::fs::read_dir(dir.path()).unwrap() {
            std::fs::write(file.unwrap().path(), "not json").unwrap();
        }
        assert_eq!(cache.read("k"), None);
    }

    #[test]
    fn freshness_follows_the_ttl() {
        assert!(Cache::is_fresh(&page(1_000), 1_000 + CACHE_TTL_SECS - 1));
        assert!(!Cache::is_fresh(&page(1_000), 1_000 + CACHE_TTL_SECS));
        assert!(
            !Cache::is_fresh(&page(2_000), 1_000),
            "clock went backwards"
        );
    }
}
