//! Ties sources, the registry adapter, HTTP and the cache together.

use super::cache::Cache;
use super::http::HttpClient;
use super::registry::{
    matches_query, parse_page, search_url, ParsedPage, FULL_LIST_PAGE_SIZE, SEARCH_PAGE_SIZE,
};
use super::{builtin_sources, MarketplaceError, MarketplaceSource, SearchPage};
use std::path::PathBuf;

pub struct Marketplace<H: HttpClient> {
    http: H,
    cache: Cache,
    sources: Vec<MarketplaceSource>,
}

impl<H: HttpClient> Marketplace<H> {
    pub fn new(http: H, cache_dir: PathBuf) -> Self {
        Self {
            http,
            cache: Cache::new(cache_dir),
            sources: builtin_sources(),
        }
    }

    /// `now` is Unix seconds, passed in so cache expiry is testable.
    ///
    /// Sources with server-side search are queried page by page. Sources without it are
    /// fetched in full once (then cached) and filtered locally; `cursor` is ignored.
    pub fn search(
        &self,
        source_id: &str,
        query: &str,
        cursor: Option<&str>,
        now: u64,
    ) -> Result<SearchPage, MarketplaceError> {
        let source = self
            .sources
            .iter()
            .find(|source| source.id == source_id)
            .ok_or_else(|| MarketplaceError::UnknownSource(source_id.to_string()))?;

        if source.server_search {
            let key = format!(
                "{}|q={}|c={}",
                source.id,
                query.trim(),
                cursor.unwrap_or("")
            );
            return self.cached_or_fetch(&key, now, || {
                self.fetch_page(source, Some(query), cursor, SEARCH_PAGE_SIZE)
            });
        }

        let key = format!("{}|all", source.id);
        let mut page = self.cached_or_fetch(&key, now, || self.fetch_all(source))?;
        page.entries.retain(|entry| matches_query(entry, query));
        Ok(page)
    }

    fn cached_or_fetch(
        &self,
        key: &str,
        now: u64,
        fetch: impl FnOnce() -> Result<ParsedPage, MarketplaceError>,
    ) -> Result<SearchPage, MarketplaceError> {
        let cached = self.cache.read(key);
        if let Some(page) = cached.as_ref().filter(|page| Cache::is_fresh(page, now)) {
            return Ok(page.clone());
        }

        match fetch() {
            Ok(parsed) => {
                let page = SearchPage {
                    entries: parsed.entries,
                    next_cursor: parsed.next_cursor,
                    skipped: parsed.skipped,
                    stale: false,
                    fetched_at: now,
                };
                self.cache.write(key, &page);
                Ok(page)
            }
            Err(error) => cached
                .map(|page| SearchPage {
                    stale: true,
                    ..page
                })
                .ok_or(error),
        }
    }

    fn fetch_page(
        &self,
        source: &MarketplaceSource,
        query: Option<&str>,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<ParsedPage, MarketplaceError> {
        let url = search_url(source, query, cursor, page_size);
        let body = self
            .http
            .get_text(&url)
            .map_err(MarketplaceError::Network)?;
        parse_page(&source.id, &body)
    }

    fn fetch_all(&self, source: &MarketplaceSource) -> Result<ParsedPage, MarketplaceError> {
        let mut all = ParsedPage {
            entries: Vec::new(),
            next_cursor: None,
            skipped: 0,
        };
        let mut cursor: Option<String> = None;
        for _ in 0..MAX_FULL_LIST_PAGES {
            let page = self.fetch_page(source, None, cursor.as_deref(), FULL_LIST_PAGE_SIZE)?;
            all.skipped += page.skipped;
            for entry in page.entries {
                if !all.entries.iter().any(|existing| existing.id == entry.id) {
                    all.entries.push(entry);
                }
            }
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        Ok(all)
    }
}

/// Upper bound on pages fetched for a full list (GitHub has ~3 at 100 per page), so a
/// misbehaving source that always returns a cursor cannot loop forever.
const MAX_FULL_LIST_PAGES: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::cache::CACHE_TTL_SECS;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const GITHUB_PAGE1: &str = include_str!("../../tests/fixtures/marketplace/github-page1.json");
    const GITHUB_PAGE2: &str = include_str!("../../tests/fixtures/marketplace/github-page2.json");
    const OFFICIAL: &str = include_str!("../../tests/fixtures/marketplace/official-search.json");

    /// Serves canned bodies by URL substring and records every request.
    #[derive(Default)]
    struct FakeHttp {
        routes: Mutex<Vec<(String, Result<String, String>)>>,
        requests: Mutex<Vec<String>>,
    }

    impl FakeHttp {
        fn route(self, url_part: &str, body: Result<&str, &str>) -> Self {
            self.routes.lock().unwrap().push((
                url_part.to_string(),
                body.map(str::to_string).map_err(str::to_string),
            ));
            self
        }

        fn fail_everything(&self) {
            let mut routes = self.routes.lock().unwrap();
            for (_, body) in routes.iter_mut() {
                *body = Err("timed out".to_string());
            }
        }

        fn request_count(&self) -> usize {
            self.requests.lock().unwrap().len()
        }
    }

    impl HttpClient for &FakeHttp {
        fn get_text(&self, url: &str) -> Result<String, String> {
            self.requests.lock().unwrap().push(url.to_string());
            let routes = self.routes.lock().unwrap();
            // Last matching route wins so tests can override earlier ones.
            routes
                .iter()
                .rev()
                .find(|(part, _)| url.contains(part.as_str()))
                .map(|(_, body)| body.clone())
                .unwrap_or_else(|| Err(format!("no route for {url}")))
        }
    }

    fn github_http() -> FakeHttp {
        FakeHttp::default()
            .route("api.mcp.github.com", Ok(GITHUB_PAGE1))
            .route("cursor=page-2", Ok(GITHUB_PAGE2))
    }

    fn ids(page: &SearchPage) -> Vec<&str> {
        page.entries
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>()
    }

    /// Hits the real registries: `cargo test live_registries -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn live_registries() {
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(
            crate::marketplace::ReqwestClient::new().unwrap(),
            dir.path().to_path_buf(),
        );
        for (source, query) in [("github", ""), ("official", "github")] {
            let started = std::time::Instant::now();
            match marketplace.search(source, query, None, 1) {
                Ok(page) => {
                    let unsupported = page
                        .entries
                        .iter()
                        .filter(|e| {
                            e.install_options.iter().all(|o| {
                                matches!(o, crate::marketplace::InstallOption::Unsupported { .. })
                            })
                        })
                        .count();
                    println!(
                        "{source}: {} entries, {} skipped, {} without a runnable option, {:?}",
                        page.entries.len(),
                        page.skipped,
                        unsupported,
                        started.elapsed()
                    );
                }
                Err(error) => println!("{source}: {error} after {:?}", started.elapsed()),
            }
        }
    }

    #[test]
    fn unknown_source_is_an_error() {
        let http = FakeHttp::default();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());
        assert_eq!(
            marketplace.search("nope", "", None, 0),
            Err(MarketplaceError::UnknownSource("nope".to_string()))
        );
    }

    #[test]
    fn local_search_source_fetches_every_page_then_filters() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());

        let all = marketplace.search("github", "", None, 1_000).unwrap();
        assert_eq!(all.entries.len(), 7);
        assert_eq!(all.next_cursor, None);
        assert_eq!(http.request_count(), 2);

        let filtered = marketplace
            .search("github", "mongodb", None, 1_001)
            .unwrap();
        assert_eq!(ids(&filtered), ["io.github.mongodb-js/mongodb-mcp-server"]);
        assert_eq!(
            http.request_count(),
            2,
            "second search is served from cache"
        );
        assert!(!filtered.stale);
    }

    #[test]
    fn server_search_source_passes_the_query_and_cursor_through() {
        let http = FakeHttp::default().route("registry.modelcontextprotocol.io", Ok(OFFICIAL));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());

        let page = marketplace
            .search("official", "docker", None, 1_000)
            .unwrap();
        assert_eq!(page.entries.len(), 4);
        assert_eq!(page.skipped, 1);
        assert!(page.next_cursor.is_some());
        marketplace
            .search("official", "docker", Some("abc"), 1_000)
            .unwrap();

        let requests = http.requests.lock().unwrap();
        assert!(requests[0].contains("search=docker"));
        assert!(!requests[0].contains("cursor="));
        assert!(requests[1].contains("cursor=abc"));
    }

    #[test]
    fn expired_cache_is_refreshed() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());
        marketplace.search("github", "", None, 1_000).unwrap();
        let page = marketplace
            .search("github", "", None, 1_000 + CACHE_TTL_SECS)
            .unwrap();
        assert_eq!(http.request_count(), 4);
        assert_eq!(page.fetched_at, 1_000 + CACHE_TTL_SECS);
    }

    #[test]
    fn falls_back_to_stale_cache_when_the_source_fails() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());
        marketplace.search("github", "", None, 1_000).unwrap();

        http.fail_everything();
        let page = marketplace
            .search("github", "context7", None, 1_000 + CACHE_TTL_SECS + 1)
            .unwrap();
        assert!(page.stale);
        assert_eq!(page.fetched_at, 1_000);
        assert_eq!(ids(&page), ["io.github.upstash/context7"]);
    }

    #[test]
    fn reports_a_network_error_without_cache() {
        let http = FakeHttp::default().route("registry.modelcontextprotocol.io", Err("timed out"));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());
        assert_eq!(
            marketplace.search("official", "x", None, 1_000),
            Err(MarketplaceError::Network("timed out".to_string()))
        );
    }

    #[test]
    fn a_failing_later_page_fails_the_full_fetch() {
        let http = github_http().route("cursor=page-2", Err("HTTP 502"));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());
        assert_eq!(
            marketplace.search("github", "", None, 1_000),
            Err(MarketplaceError::Network("HTTP 502".to_string()))
        );
    }

    #[test]
    fn stops_following_cursors_after_a_page_limit() {
        // A source that always returns a next cursor must not loop forever.
        let looping = GITHUB_PAGE1.replace("\"page-2\"", "\"again\"");
        let http = FakeHttp::default().route("api.mcp.github.com", Ok(&looping));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = Marketplace::new(&http, dir.path().to_path_buf());
        let page = marketplace.search("github", "", None, 1_000).unwrap();
        assert_eq!(http.request_count(), 10);
        let mut unique: HashMap<&str, ()> = HashMap::new();
        for entry in &page.entries {
            unique.insert(entry.id.as_str(), ());
        }
        assert_eq!(
            unique.len(),
            page.entries.len(),
            "duplicates across pages are collapsed"
        );
    }
}
