//! Blocking HTTP client used by the marketplace. Kept behind [`HttpClient`] so the
//! service can be tested without network access.

use std::time::Duration;

pub trait HttpClient: Send + Sync {
    fn get_text(&self, url: &str) -> Result<String, String>;
}

/// Full-list pages from the GitHub registry are ~5 MB uncompressed, so allow more than
/// a typical API call but still fail fast enough to fall back to the cache.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

pub struct ReqwestClient {
    client: reqwest::blocking::Client,
}

impl ReqwestClient {
    pub fn new() -> Result<Self, String> {
        // Same provider tauri-plugin-updater installs; whichever runs first wins.
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("mcp-manager/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { client })
    }
}

impl HttpClient for ReqwestClient {
    fn get_text(&self, url: &str) -> Result<String, String> {
        let response = self.client.get(url).send().map_err(|e| e.to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP {status}"));
        }
        response.text().map_err(|e| e.to_string())
    }
}
