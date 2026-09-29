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
            let cache_dir = PlatformContext::current()
                .app_data_dir()
                .join("marketplace-cache");
            ReqwestClient::new().map(|http| Marketplace::new(http, cache_dir))
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
    Ok(crate::marketplace::builtin_sources())
}

/// Runs on a blocking thread: a full-list fetch can take several seconds.
#[tauri::command]
pub async fn marketplace_search(
    source_id: String,
    query: String,
    cursor: Option<String>,
) -> Result<SearchPage, MarketplaceError> {
    ensure_enabled()?;
    tauri::async_runtime::spawn_blocking(move || {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        marketplace()?.search(&source_id, &query, cursor.as_deref(), now)
    })
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

        marketplace_set_enabled(true);
        assert_eq!(marketplace_sources().map(|s| s.len()), Ok(2));
        marketplace_set_enabled(false);
    }
}
