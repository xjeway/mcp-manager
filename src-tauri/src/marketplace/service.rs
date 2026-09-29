//! Ties sources, the registry adapter, HTTP and the cache together.

use super::cache::Cache;
use super::http::HttpClient;
use super::registry::{
    full_list_url, matches_query, parse_page, search_url, ParsedPage, SEARCH_PAGE_SIZE,
};
use super::sources::{normalize_base_url, source_id, source_label, SourceStore};
use super::{
    builtin_sources, MarketplaceError, MarketplaceSource, SearchPage, SourceKind, SourceTrust,
};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Marketplace<H: HttpClient> {
    http: H,
    cache: Cache,
    store: SourceStore,
    custom: Mutex<Vec<MarketplaceSource>>,
}

impl<H: HttpClient> Marketplace<H> {
    pub fn new(http: H, cache_dir: PathBuf, sources_path: PathBuf) -> Self {
        let store = SourceStore::new(sources_path);
        let custom = Mutex::new(store.load());
        Self {
            http,
            cache: Cache::new(cache_dir),
            store,
            custom,
        }
    }

    /// Built-in sources first, then the user's in the order they were added.
    pub fn sources(&self) -> Vec<MarketplaceSource> {
        let mut sources = builtin_sources();
        sources.extend(self.custom.lock().unwrap().iter().cloned());
        sources
    }

    /// `now` is Unix seconds, passed in so cache expiry is testable.
    ///
    /// Cached data is returned even when it has expired, marked `stale`, so the page
    /// opens at once; the caller then asks for [`Marketplace::refresh`]. Only a source
    /// with nothing cached is fetched before returning.
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
        let source = self.source(source_id)?;
        let key = cache_key(&source, query, cursor);
        let page = match self.cache.read(&key) {
            Some(page) => SearchPage {
                stale: !Cache::is_fresh(&page, now),
                ..page
            },
            None => self.fetch(&source, &key, query, cursor, now)?,
        };
        Ok(filter_locally(&source, page, query))
    }

    /// Fetches from the source regardless of the cache. On failure the cache is left
    /// as it was, so stale data the caller already shows stays valid.
    pub fn refresh(
        &self,
        source_id: &str,
        query: &str,
        cursor: Option<&str>,
        now: u64,
    ) -> Result<SearchPage, MarketplaceError> {
        let source = self.source(source_id)?;
        let key = cache_key(&source, query, cursor);
        let page = self.fetch(&source, &key, query, cursor, now)?;
        Ok(filter_locally(&source, page, query))
    }

    /// Checks that `base_url` answers the Registry API before saving it, and detects
    /// whether it searches server-side by asking for a name nothing should match.
    pub fn add_source(
        &self,
        label: &str,
        base_url: &str,
    ) -> Result<MarketplaceSource, MarketplaceError> {
        let base_url = normalize_base_url(base_url)?;
        if self
            .sources()
            .iter()
            .any(|source| source.base_url.trim_end_matches('/') == base_url)
        {
            return Err(MarketplaceError::DuplicateSource(base_url));
        }

        let mut source = MarketplaceSource {
            id: source_id(&base_url),
            kind: SourceKind::McpRegistry,
            label: source_label(label, &base_url),
            base_url,
            trust: SourceTrust::Community,
            server_search: false,
            builtin: false,
        };
        self.fetch_page(&source, None, None, 1)?;
        let probe = self.fetch_page(&source, Some(SEARCH_PROBE), None, 1)?;
        source.server_search = probe.entries.is_empty();

        let mut custom = self.custom.lock().unwrap();
        let mut updated = custom.clone();
        updated.push(source.clone());
        self.store.save(&updated)?;
        *custom = updated;
        Ok(source)
    }

    pub fn remove_source(&self, source_id: &str) -> Result<(), MarketplaceError> {
        let mut custom = self.custom.lock().unwrap();
        let updated: Vec<MarketplaceSource> = custom
            .iter()
            .filter(|source| source.id != source_id)
            .cloned()
            .collect();
        if updated.len() == custom.len() {
            return Err(MarketplaceError::UnknownSource(source_id.to_string()));
        }
        self.store.save(&updated)?;
        *custom = updated;
        Ok(())
    }

    fn source(&self, source_id: &str) -> Result<MarketplaceSource, MarketplaceError> {
        self.sources()
            .into_iter()
            .find(|source| source.id == source_id)
            .ok_or_else(|| MarketplaceError::UnknownSource(source_id.to_string()))
    }

    fn fetch(
        &self,
        source: &MarketplaceSource,
        key: &str,
        query: &str,
        cursor: Option<&str>,
        now: u64,
    ) -> Result<SearchPage, MarketplaceError> {
        let parsed = if source.server_search {
            self.fetch_page(source, Some(query), cursor, SEARCH_PAGE_SIZE)?
        } else {
            self.fetch_all(source)?
        };
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

    fn fetch_page(
        &self,
        source: &MarketplaceSource,
        query: Option<&str>,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<ParsedPage, MarketplaceError> {
        self.get_page(source, &search_url(source, query, cursor, page_size))
    }

    fn get_page(
        &self,
        source: &MarketplaceSource,
        url: &str,
    ) -> Result<ParsedPage, MarketplaceError> {
        let body = self.http.get_text(url).map_err(MarketplaceError::Network)?;
        parse_page(&source.id, &body)
    }

    /// When the first page reports a page count, the remaining pages are requested at
    /// once: a GitHub registry page is ~5 MB and served uncompressed, so fetching them
    /// one after another made the first open take 10 seconds or more. Sources that only
    /// page by cursor are followed page by page.
    fn fetch_all(&self, source: &MarketplaceSource) -> Result<ParsedPage, MarketplaceError> {
        let first = self.get_page(source, &full_list_url(source, 1, None))?;
        let mut pages = Vec::new();
        match first.total_pages.filter(|&total| total > 1) {
            Some(total) => {
                let last = total.min(MAX_FULL_LIST_PAGES);
                let rest = std::thread::scope(|scope| {
                    let handles: Vec<_> = (2..=last)
                        .map(|page| {
                            scope.spawn(move || {
                                self.get_page(source, &full_list_url(source, page, None))
                            })
                        })
                        .collect();
                    handles
                        .into_iter()
                        .map(|handle| handle.join().expect("page fetch panicked"))
                        .collect::<Result<Vec<_>, _>>()
                })?;
                pages.push(first);
                pages.extend(rest);
            }
            None => {
                let mut cursor = first.next_cursor.clone();
                pages.push(first);
                for page in 2..=MAX_FULL_LIST_PAGES {
                    let Some(next) = cursor.take() else { break };
                    let parsed =
                        self.get_page(source, &full_list_url(source, page, Some(&next)))?;
                    cursor = parsed.next_cursor.clone();
                    pages.push(parsed);
                }
            }
        }

        let mut all = ParsedPage {
            entries: Vec::new(),
            next_cursor: None,
            skipped: 0,
            total_pages: None,
        };
        for page in pages {
            all.skipped += page.skipped;
            for entry in page.entries {
                if !all.entries.iter().any(|existing| existing.id == entry.id) {
                    all.entries.push(entry);
                }
            }
        }
        Ok(all)
    }
}

fn cache_key(source: &MarketplaceSource, query: &str, cursor: Option<&str>) -> String {
    if source.server_search {
        format!(
            "{}|q={}|c={}",
            source.id,
            query.trim(),
            cursor.unwrap_or("")
        )
    } else {
        format!("{}|all", source.id)
    }
}

fn filter_locally(source: &MarketplaceSource, mut page: SearchPage, query: &str) -> SearchPage {
    if !source.server_search {
        page.entries.retain(|entry| matches_query(entry, query));
    }
    page
}

/// Upper bound on pages fetched for a full list (GitHub has ~3 at 100 per page), so a
/// misbehaving source that always returns a cursor cannot loop forever.
const MAX_FULL_LIST_PAGES: usize = 10;

/// A name no real server has; a source that searches server-side returns nothing for it.
const SEARCH_PROBE: &str = "mcp-manager-search-probe-7f3c1e";

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
            .route("&page=2", Ok(GITHUB_PAGE2))
    }

    fn open_marketplace<'a>(
        http: &'a FakeHttp,
        dir: &tempfile::TempDir,
    ) -> Marketplace<&'a FakeHttp> {
        Marketplace::new(
            http,
            dir.path().join("cache"),
            dir.path().join("sources.json"),
        )
    }

    /// A cursor-only registry: no page count, pages chained by `nextCursor`.
    fn cursor_registry_page(names: &[&str], next: Option<&str>) -> String {
        let servers: Vec<String> = names
            .iter()
            .map(|name| format!(r#"{{"server":{{"name":"{name}"}}}}"#))
            .collect();
        let metadata = next
            .map(|cursor| format!(r#"{{"nextCursor":"{cursor}"}}"#))
            .unwrap_or_else(|| "{}".to_string());
        format!(
            r#"{{"servers":[{}],"metadata":{metadata}}}"#,
            servers.join(",")
        )
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
            dir.path().join("cache"),
            dir.path().join("sources.json"),
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
        let marketplace = open_marketplace(&http, &dir);
        assert_eq!(
            marketplace.search("nope", "", None, 0),
            Err(MarketplaceError::UnknownSource("nope".to_string()))
        );
    }

    #[test]
    fn local_search_source_fetches_every_page_then_filters() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);

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
        let marketplace = open_marketplace(&http, &dir);

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
    fn expired_cache_is_returned_at_once_and_marked_stale() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        marketplace.search("github", "", None, 1_000).unwrap();
        let requests = http.request_count();

        let page = marketplace
            .search("github", "context7", None, 1_000 + CACHE_TTL_SECS)
            .unwrap();
        assert!(page.stale);
        assert_eq!(page.fetched_at, 1_000);
        assert_eq!(ids(&page), ["io.github.upstash/context7"]);
        assert_eq!(
            http.request_count(),
            requests,
            "no request before returning"
        );
    }

    #[test]
    fn refresh_fetches_and_replaces_the_cache() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        marketplace.search("github", "", None, 1_000).unwrap();

        let later = 1_000 + CACHE_TTL_SECS;
        let refreshed = marketplace
            .refresh("github", "mongodb", None, later)
            .unwrap();
        assert!(!refreshed.stale);
        assert_eq!(refreshed.fetched_at, later);
        assert_eq!(ids(&refreshed), ["io.github.mongodb-js/mongodb-mcp-server"]);

        let cached = marketplace.search("github", "", None, later + 1).unwrap();
        assert!(!cached.stale);
        assert_eq!(cached.fetched_at, later);
    }

    #[test]
    fn a_failed_refresh_keeps_the_cache() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        marketplace.search("github", "", None, 1_000).unwrap();

        http.fail_everything();
        let later = 1_000 + CACHE_TTL_SECS + 1;
        assert_eq!(
            marketplace.refresh("github", "", None, later),
            Err(MarketplaceError::Network("timed out".to_string()))
        );
        let page = marketplace.search("github", "", None, later).unwrap();
        assert!(page.stale);
        assert_eq!(page.entries.len(), 7);
    }

    #[test]
    fn reports_a_network_error_without_cache() {
        let http = FakeHttp::default().route("registry.modelcontextprotocol.io", Err("timed out"));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        assert_eq!(
            marketplace.search("official", "x", None, 1_000),
            Err(MarketplaceError::Network("timed out".to_string()))
        );
    }

    #[test]
    fn a_failing_later_page_fails_the_full_fetch() {
        let http = github_http().route("&page=2", Err("HTTP 502"));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        assert_eq!(
            marketplace.search("github", "", None, 1_000),
            Err(MarketplaceError::Network("HTTP 502".to_string()))
        );
    }

    #[test]
    fn requests_every_counted_page_by_number() {
        let http = github_http();
        let dir = tempfile::tempdir().unwrap();
        open_marketplace(&http, &dir)
            .search("github", "", None, 1_000)
            .unwrap();
        let mut pages: Vec<String> = http
            .requests
            .lock()
            .unwrap()
            .iter()
            .map(|url| url.split("&page=").nth(1).unwrap().to_string())
            .collect();
        pages.sort();
        assert_eq!(pages, ["1", "2"]);
    }

    #[test]
    fn caps_the_page_count_a_source_reports() {
        let huge = GITHUB_PAGE1.replace("\"total_pages\": 2", "\"total_pages\": 5000");
        let http = FakeHttp::default().route("api.mcp.github.com", Ok(&huge));
        let dir = tempfile::tempdir().unwrap();
        open_marketplace(&http, &dir)
            .search("github", "", None, 1_000)
            .unwrap();
        assert_eq!(http.request_count(), MAX_FULL_LIST_PAGES);
    }

    #[test]
    fn follows_cursors_when_a_source_reports_no_page_count() {
        let page1 = cursor_registry_page(&["a/one", "a/two"], Some("c2"));
        let page2 = cursor_registry_page(&["a/three"], None);
        let http = FakeHttp::default()
            .route("mcp.acme.dev", Ok(&page1))
            .route("cursor=c2", Ok(&page2));
        let dir = tempfile::tempdir().unwrap();
        let store = SourceStore::new(dir.path().join("sources.json"));
        store
            .save(&[MarketplaceSource {
                id: "custom-acme".to_string(),
                kind: SourceKind::McpRegistry,
                label: "Acme".to_string(),
                base_url: "https://mcp.acme.dev".to_string(),
                trust: SourceTrust::Community,
                server_search: false,
                builtin: false,
            }])
            .unwrap();
        let page = open_marketplace(&http, &dir)
            .search("custom-acme", "", None, 1_000)
            .unwrap();
        assert_eq!(ids(&page), ["a/one", "a/two", "a/three"]);
        assert_eq!(http.request_count(), 2);
    }

    #[test]
    fn stops_following_cursors_after_a_page_limit() {
        // A source that always returns a next cursor must not loop forever.
        let looping = GITHUB_PAGE1
            .replace("\"page-2\"", "\"again\"")
            .replace("\"total_pages\": 2", "\"total_pages\": null");
        let http = FakeHttp::default().route("api.mcp.github.com", Ok(&looping));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        let page = marketplace.search("github", "", None, 1_000).unwrap();
        assert_eq!(http.request_count(), MAX_FULL_LIST_PAGES);
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

    #[test]
    fn adds_a_source_that_answers_the_registry_api() {
        let searching = cursor_registry_page(&["a/one"], None);
        let empty = cursor_registry_page(&[], None);
        let http = FakeHttp::default()
            .route("mcp.acme.dev", Ok(&searching))
            .route("search=", Ok(&empty));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);

        let acme = marketplace
            .add_source(" Acme ", "https://mcp.acme.dev/v0/servers")
            .unwrap();
        assert_eq!(acme.label, "Acme");
        assert_eq!(acme.base_url, "https://mcp.acme.dev");
        assert!(
            acme.server_search,
            "an empty probe result means server search"
        );
        assert!(!acme.builtin);

        // A host that ignores `search` answers the probe with its full list.
        let http2 = FakeHttp::default().route("ignores.dev", Ok(&searching));
        let dir2 = tempfile::tempdir().unwrap();
        let other = open_marketplace(&http2, &dir2)
            .add_source("", "https://ignores.dev")
            .unwrap();
        assert!(!other.server_search);
        assert_eq!(other.label, "ignores.dev");

        let ids: Vec<String> = marketplace.sources().into_iter().map(|s| s.id).collect();
        assert_eq!(ids, ["github", "official", acme.id.as_str()]);

        // Persisted: a new instance over the same directory sees it.
        let reopened = open_marketplace(&http, &dir);
        assert_eq!(reopened.sources().len(), 3);
        assert_eq!(
            reopened
                .search(&acme.id, "", None, 1_000)
                .map(|p| p.entries.len()),
            Ok(1)
        );
    }

    #[test]
    fn rejects_duplicate_and_unreachable_sources() {
        let http = FakeHttp::default().route("down.dev", Err("timed out"));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);

        assert!(matches!(
            marketplace.add_source("", "https://registry.modelcontextprotocol.io/"),
            Err(MarketplaceError::DuplicateSource(_))
        ));
        assert!(matches!(
            marketplace.add_source("", "http://insecure.dev"),
            Err(MarketplaceError::InvalidUrl(_))
        ));
        assert_eq!(
            marketplace.add_source("", "https://down.dev"),
            Err(MarketplaceError::Network("timed out".to_string()))
        );
        assert_eq!(marketplace.sources().len(), 2, "nothing was saved");
    }

    #[test]
    fn removes_only_user_sources() {
        let page = cursor_registry_page(&["a/one"], None);
        let http = FakeHttp::default().route("mcp.acme.dev", Ok(&page));
        let dir = tempfile::tempdir().unwrap();
        let marketplace = open_marketplace(&http, &dir);
        let acme = marketplace.add_source("", "https://mcp.acme.dev").unwrap();

        assert_eq!(
            marketplace.remove_source("github"),
            Err(MarketplaceError::UnknownSource("github".to_string()))
        );
        marketplace.remove_source(&acme.id).unwrap();
        assert_eq!(marketplace.sources().len(), 2);
        assert_eq!(open_marketplace(&http, &dir).sources().len(), 2);
    }
}
