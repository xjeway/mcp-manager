//! Adapter for sources implementing the MCP Registry API (`GET /v0/servers`).

use super::install::{install_options, RawPackage, RawRemote};
use super::{MarketplaceEntry, MarketplaceError, MarketplaceSource};
use serde::Deserialize;

/// Page size requested from sources that support server-side search.
pub(super) const SEARCH_PAGE_SIZE: usize = 30;
/// Page size used when a source has to be fetched in full.
pub(super) const FULL_LIST_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ParsedPage {
    pub entries: Vec<MarketplaceEntry>,
    pub next_cursor: Option<String>,
    pub skipped: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawServer {
    name: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    repository: Option<RawRepository>,
    #[serde(default, alias = "website_url")]
    website_url: Option<String>,
    #[serde(default)]
    packages: Option<Vec<RawPackage>>,
    #[serde(default)]
    remotes: Option<Vec<RawRemote>>,
}

#[derive(Debug, Deserialize)]
struct RawRepository {
    #[serde(default)]
    url: Option<String>,
}

const OFFICIAL_META: &str = "io.modelcontextprotocol.registry/official";
const PUBLISHER_META: &str = "io.modelcontextprotocol.registry/publisher-provided";
const README_EXCERPT_CHARS: usize = 600;

/// Builds a standard Registry API request. Only `limit` is used for page size: the
/// GitHub registry switches to page-number paging without a cursor if it sees `per_page`.
pub(super) fn search_url(
    source: &MarketplaceSource,
    query: Option<&str>,
    cursor: Option<&str>,
    page_size: usize,
) -> String {
    let mut params = vec![
        ("version", "latest".to_string()),
        ("limit", page_size.to_string()),
    ];
    if let Some(query) = query.map(str::trim).filter(|q| !q.is_empty()) {
        params.push(("search", query.to_string()));
    }
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor.to_string()));
    }
    let base = format!("{}/v0/servers", source.base_url.trim_end_matches('/'));
    reqwest::Url::parse_with_params(&base, &params)
        .map(String::from)
        .unwrap_or(base)
}

pub(super) fn parse_page(source_id: &str, body: &str) -> Result<ParsedPage, MarketplaceError> {
    let root: serde_json::Value =
        serde_json::from_str(body).map_err(|e| MarketplaceError::Parse(e.to_string()))?;
    let items = root
        .get("servers")
        .and_then(|servers| servers.as_array())
        .ok_or_else(|| MarketplaceError::Parse("missing `servers` array".to_string()))?;

    let mut entries: Vec<(MarketplaceEntry, bool)> = Vec::new();
    let mut skipped = 0;
    for item in items {
        match parse_item(source_id, item) {
            Some((entry, is_latest)) => {
                // Keep one record per server, preferring the one marked latest.
                match entries
                    .iter_mut()
                    .find(|(existing, _)| existing.id == entry.id)
                {
                    Some(slot) if is_latest && !slot.1 => *slot = (entry, is_latest),
                    Some(_) => {}
                    None => entries.push((entry, is_latest)),
                }
            }
            None => skipped += 1,
        }
    }

    let metadata = root.get("metadata");
    let next_cursor = metadata
        .and_then(|m| m.get("nextCursor").or_else(|| m.get("next_cursor")))
        .and_then(|cursor| cursor.as_str())
        .filter(|cursor| !cursor.is_empty())
        .map(str::to_string);

    Ok(ParsedPage {
        entries: entries.into_iter().map(|(entry, _)| entry).collect(),
        next_cursor,
        skipped,
    })
}

/// Official responses wrap each record as `{ server, _meta }`; the dated GitHub API
/// returns the server object directly with `_meta` inside it.
fn parse_item(source_id: &str, item: &serde_json::Value) -> Option<(MarketplaceEntry, bool)> {
    let server_value = item.get("server").unwrap_or(item);
    let server: RawServer = serde_json::from_value(server_value.clone()).ok()?;
    if server.name.trim().is_empty() {
        return None;
    }

    let meta = |key: &str| {
        item.get("_meta")
            .and_then(|m| m.get(key))
            .or_else(|| server_value.get("_meta").and_then(|m| m.get(key)))
    };
    let is_latest = meta(OFFICIAL_META)
        .and_then(|m| m.get("isLatest").or_else(|| m.get("is_latest")))
        .and_then(|latest| latest.as_bool())
        .unwrap_or(true);
    let github = meta(PUBLISHER_META).and_then(|m| m.get("github"));
    let github_str = |key: &str| {
        github
            .and_then(|g| g.get(key))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
    };

    let title = server
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .or_else(|| github_str("display_name"))
        .unwrap_or_else(|| {
            server
                .name
                .rsplit('/')
                .next()
                .unwrap_or(&server.name)
                .to_string()
        });

    let entry = MarketplaceEntry {
        source_id: source_id.to_string(),
        id: server.name.clone(),
        title,
        description: server.description.clone().unwrap_or_default(),
        version: server.version.clone(),
        repository_url: server
            .repository
            .as_ref()
            .and_then(|r| r.url.clone())
            .filter(|url| !url.is_empty()),
        website_url: server
            .website_url
            .clone()
            .filter(|url| !url.is_empty())
            .or_else(|| github_str("homepage_url")),
        stars: github
            .and_then(|g| g.get("stargazer_count"))
            .and_then(|v| v.as_u64()),
        readme_excerpt: github_str("readme").as_deref().and_then(readme_excerpt),
        install_options: install_options(
            server.packages.as_deref().unwrap_or_default(),
            server.remotes.as_deref().unwrap_or_default(),
        ),
    };
    Some((entry, is_latest))
}

/// Every whitespace-separated term must appear in the id, title or description.
pub(super) fn matches_query(entry: &MarketplaceEntry, query: &str) -> bool {
    let haystack = format!("{} {} {}", entry.id, entry.title, entry.description).to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|term| haystack.contains(term))
}

/// Plain-text summary of a README: the first few prose paragraphs, markup stripped.
pub(super) fn readme_excerpt(markdown: &str) -> Option<String> {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut in_fence = false;
    for block in markdown.split("\n\n") {
        let mut lines = Vec::new();
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence
                || trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('<')
                || trimmed.starts_with('|')
                || trimmed.starts_with("---")
            {
                continue;
            }
            let text = strip_inline_markup(trimmed);
            if !text.trim().is_empty() {
                lines.push(text.trim().to_string());
            }
        }
        if !lines.is_empty() {
            paragraphs.push(lines.join(" "));
        }
        if paragraphs.iter().map(|p| p.chars().count()).sum::<usize>() >= README_EXCERPT_CHARS {
            break;
        }
    }

    let text = paragraphs.join("\n\n");
    if text.is_empty() {
        return None;
    }
    if text.chars().count() <= README_EXCERPT_CHARS {
        return Some(text);
    }
    let mut cut: String = text.chars().take(README_EXCERPT_CHARS).collect();
    cut = cut.trim_end().to_string();
    cut.push('…');
    Some(cut)
}

/// Drops images and HTML tags, keeps link text, removes emphasis and code markers.
fn strip_inline_markup(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            // `![alt](url)` and `[![alt](img)](url)` are badges/images: drop entirely.
            '!' if chars.get(i + 1) == Some(&'[') => {
                i = skip_link(&chars, i + 1);
            }
            '[' if chars.get(i + 1) == Some(&'!') => {
                i = skip_link(&chars, i);
            }
            '[' => {
                let close = chars[i + 1..]
                    .iter()
                    .position(|&c| c == ']')
                    .map(|p| p + i + 1);
                match close {
                    Some(close) if chars.get(close + 1) == Some(&'(') => {
                        out.extend(&chars[i + 1..close]);
                        let end = chars[close + 1..].iter().position(|&c| c == ')');
                        i = end.map_or(chars.len(), |p| close + 1 + p + 1);
                    }
                    _ => {
                        out.push('[');
                        i += 1;
                    }
                }
            }
            '<' => {
                let end = chars[i..].iter().position(|&c| c == '>');
                i = end.map_or(chars.len(), |p| i + p + 1);
            }
            '*' | '_' | '`' => i += 1,
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Skips a `[text](url)` link starting at `start` (which must be `[`), handling one
/// level of nesting as used by badge links.
fn skip_link(chars: &[char], start: usize) -> usize {
    let mut depth = 0;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if chars.get(i + 1) != Some(&'(') {
        return i + 1;
    }
    let end = chars[i + 1..].iter().position(|&c| c == ')');
    end.map_or(chars.len(), |p| i + 1 + p + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::{builtin_sources, InstallOption};

    const GITHUB_PAGE1: &str = include_str!("../../tests/fixtures/marketplace/github-page1.json");
    const GITHUB_PAGE2: &str = include_str!("../../tests/fixtures/marketplace/github-page2.json");
    const OFFICIAL: &str = include_str!("../../tests/fixtures/marketplace/official-search.json");

    fn source(id: &str) -> MarketplaceSource {
        builtin_sources().into_iter().find(|s| s.id == id).unwrap()
    }

    fn entry<'a>(page: &'a ParsedPage, id: &str) -> &'a MarketplaceEntry {
        page.entries
            .iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("missing {id}"))
    }

    #[test]
    fn parses_github_registry_page_in_snake_case_schema() {
        let page = parse_page("github", GITHUB_PAGE1).unwrap();
        assert_eq!(page.entries.len(), 6);
        assert_eq!(page.skipped, 0);
        assert_eq!(page.next_cursor.as_deref(), Some("page-2"));

        let context7 = entry(&page, "io.github.upstash/context7");
        assert_eq!(context7.source_id, "github");
        assert_eq!(context7.title, "Context7");
        assert_eq!(
            context7.repository_url.as_deref(),
            Some("https://github.com/upstash/context7")
        );
        assert!(context7.stars.unwrap() > 1000);
        assert!(context7.readme_excerpt.as_deref().unwrap_or("").len() > 20);
        assert!(matches!(
            context7.install_options[0],
            InstallOption::Stdio { .. }
        ));
        assert!(context7
            .install_options
            .iter()
            .any(|o| matches!(o, InstallOption::Http { .. })));
    }

    #[test]
    fn parses_last_github_page_without_cursor() {
        let page = parse_page("github", GITHUB_PAGE2).unwrap();
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn parses_official_registry_page_and_keeps_only_latest_versions() {
        let page = parse_page("official", OFFICIAL).unwrap();
        let ids: Vec<&str> = page.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "io.github.upstash/context7",
                "io.github.GavinLucas/docker-mcp-server",
                "io.github.Evozim/dockerfile-credential-mcp",
                "io.github.hypnosis/docker-mcp-server",
            ]
        );
        assert_eq!(
            entry(&page, "io.github.upstash/context7")
                .version
                .as_deref(),
            Some("4.1.1")
        );
        assert_eq!(
            page.skipped, 1,
            "the malformed record is skipped, not fatal"
        );
        assert_eq!(
            page.next_cursor.as_deref(),
            Some("io.github.hypnosis/docker-mcp-server:1.0.0")
        );
        assert_eq!(
            entry(&page, "io.github.upstash/context7")
                .website_url
                .as_deref(),
            Some("https://context7.com")
        );
    }

    #[test]
    fn falls_back_to_the_last_name_segment_for_the_title() {
        let body = r#"{"servers":[{"server":{"name":"io.github.acme/widget-mcp"}}],"metadata":{}}"#;
        let page = parse_page("official", body).unwrap();
        assert_eq!(page.entries[0].title, "widget-mcp");
        assert_eq!(page.entries[0].description, "");
    }

    #[test]
    fn rejects_a_body_that_is_not_a_registry_response() {
        assert!(matches!(
            parse_page("official", "<html>"),
            Err(MarketplaceError::Parse(_))
        ));
        assert!(matches!(
            parse_page("official", "{}"),
            Err(MarketplaceError::Parse(_))
        ));
    }

    #[test]
    fn builds_search_urls() {
        assert_eq!(
            search_url(&source("official"), Some("context 7"), None, SEARCH_PAGE_SIZE),
            "https://registry.modelcontextprotocol.io/v0/servers?version=latest&limit=30&search=context+7"
        );
        assert_eq!(
            search_url(&source("official"), None, Some("a/b:1.0"), SEARCH_PAGE_SIZE),
            "https://registry.modelcontextprotocol.io/v0/servers?version=latest&limit=30&cursor=a%2Fb%3A1.0"
        );
        assert_eq!(
            search_url(&source("github"), None, None, FULL_LIST_PAGE_SIZE),
            // No `per_page`: GitHub switches to page-number paging and drops the cursor.
            "https://api.mcp.github.com/2025-09-15/v0/servers?version=latest&limit=100"
        );
    }

    #[test]
    fn matches_every_term_case_insensitively() {
        let page = parse_page("github", GITHUB_PAGE1).unwrap();
        let context7 = entry(&page, "io.github.upstash/context7");
        assert!(matches_query(context7, ""));
        assert!(matches_query(context7, "CONTEXT7"));
        assert!(matches_query(context7, "upstash context7"));
        assert!(!matches_query(context7, "context7 mongodb"));
    }

    #[test]
    fn readme_excerpt_strips_markup_and_keeps_prose() {
        let markdown = "# Title\n\n[![Badge](https://img)](https://x)\n\n<p align=\"center\"><img src=\"x\"></p>\n\nA **useful** server for [docs](https://docs).\n\n```bash\nnpm i\n```\n\nSecond paragraph.\n";
        assert_eq!(
            readme_excerpt(markdown).as_deref(),
            Some("A useful server for docs.\n\nSecond paragraph.")
        );
        assert_eq!(readme_excerpt("# Only a heading"), None);
    }

    #[test]
    fn readme_excerpt_is_capped() {
        let long = "word ".repeat(1000);
        let excerpt = readme_excerpt(&long).unwrap();
        assert!(excerpt.chars().count() <= 601);
        assert!(excerpt.ends_with('…'));
    }
}
