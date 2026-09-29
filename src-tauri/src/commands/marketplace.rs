//! Tauri commands for the online marketplace.
//!
//! The marketplace starts disabled; the frontend enables it from the user's preference
//! at startup. Every command checks the flag, so turning the setting off guarantees no
//! marketplace request leaves the app even if the UI misbehaves.

use crate::marketplace::{
    Marketplace, MarketplaceError, MarketplaceSource, ReqwestClient, SearchPage,
};
use mcp_manager_core::platform::PlatformContext;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static ENABLED: AtomicBool = AtomicBool::new(false);
static MARKETPLACE: OnceLock<Result<Marketplace<ReqwestClient>, String>> = OnceLock::new();

fn ensure_enabled() -> Result<(), MarketplaceError> {
    if ENABLED.load(Ordering::SeqCst) {
        Ok(())
    } else {
        Err(MarketplaceError::Disabled)
    }
}

fn marketplace() -> Result<&'static Marketplace<ReqwestClient>, MarketplaceError> {
    MARKETPLACE
        .get_or_init(|| {
            let data_dir = PlatformContext::current().app_data_dir();
            ReqwestClient::new().map(|http| {
                Marketplace::new(
                    http,
                    data_dir.join("marketplace-cache"),
                    data_dir.join("marketplace-sources.json"),
                )
            })
        })
        .as_ref()
        .map_err(|error| MarketplaceError::Network(error.clone()))
}

#[tauri::command]
pub fn marketplace_set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
}

#[tauri::command]
pub fn marketplace_sources() -> Result<Vec<MarketplaceSource>, MarketplaceError> {
    ensure_enabled()?;
    Ok(marketplace()?.sources())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// Returns cached data at once when there is any, even expired (then marked `stale`);
/// the frontend follows a stale page with `marketplace_refresh`. Runs on a blocking
/// thread because a source with nothing cached is fetched first.
#[tauri::command]
pub async fn marketplace_search(
    source_id: String,
    query: String,
    cursor: Option<String>,
) -> Result<SearchPage, MarketplaceError> {
    ensure_enabled()?;
    run_blocking(move || marketplace()?.search(&source_id, &query, cursor.as_deref(), now())).await
}

#[tauri::command]
pub async fn marketplace_refresh(
    source_id: String,
    query: String,
    cursor: Option<String>,
) -> Result<SearchPage, MarketplaceError> {
    ensure_enabled()?;
    run_blocking(move || marketplace()?.refresh(&source_id, &query, cursor.as_deref(), now())).await
}

/// Probes the URL over the network before saving it.
#[tauri::command]
pub async fn marketplace_add_source(
    label: String,
    base_url: String,
) -> Result<MarketplaceSource, MarketplaceError> {
    ensure_enabled()?;
    run_blocking(move || marketplace()?.add_source(&label, &base_url)).await
}

#[tauri::command]
pub fn marketplace_remove_source(source_id: String) -> Result<(), MarketplaceError> {
    ensure_enabled()?;
    marketplace()?.remove_source(&source_id)
}

async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, MarketplaceError> + Send + 'static,
) -> Result<T, MarketplaceError> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| MarketplaceError::Network(error.to_string()))?
}

/// Registry data is untrusted: only plain https links are handed to the system opener,
/// which also keeps the argument from being read as a command-line flag.
fn validate_external_url(url: &str) -> Result<String, String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| e.to_string())?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(format!("refusing to open {url}"));
    }
    Ok(parsed.to_string())
}

#[tauri::command]
pub fn marketplace_open_url(url: String) -> Result<(), String> {
    let url = validate_external_url(&url)?;

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&url).status();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer")
        .arg(&url)
        .spawn()
        .map(|_| ());
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(&url).status();

    #[cfg(target_os = "windows")]
    return result.map_err(|e| e.to_string());
    #[cfg(not(target_os = "windows"))]
    result.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("failed to open {url}"))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_urls_are_opened() {
        assert_eq!(
            validate_external_url("https://github.com/upstash/context7"),
            Ok("https://github.com/upstash/context7".to_string())
        );
        for rejected in [
            "http://example.com",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "-a Calculator",
            "https://",
            "",
        ] {
            assert!(validate_external_url(rejected).is_err(), "{rejected}");
        }
    }

    #[test]
    fn commands_refuse_to_run_while_disabled() {
        marketplace_set_enabled(false);
        assert_eq!(marketplace_sources(), Err(MarketplaceError::Disabled));
        let search = tauri::async_runtime::block_on(marketplace_search(
            "github".to_string(),
            String::new(),
            None,
        ));
        assert_eq!(search, Err(MarketplaceError::Disabled));
        let add = tauri::async_runtime::block_on(marketplace_add_source(
            String::new(),
            "https://mcp.acme.dev".to_string(),
        ));
        assert_eq!(add, Err(MarketplaceError::Disabled));
        assert_eq!(
            marketplace_remove_source("custom-x".to_string()),
            Err(MarketplaceError::Disabled)
        );

        marketplace_set_enabled(true);
        // User sources come from the real app data directory, so count built-ins only.
        assert_eq!(
            marketplace_sources().map(|s| s.iter().filter(|s| s.builtin).count()),
            Ok(2)
        );
        marketplace_set_enabled(false);
    }
}
