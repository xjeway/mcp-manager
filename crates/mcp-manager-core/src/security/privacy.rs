//! Network preferences stored under the application-data directory.
//!
//! Automatic update checks and the "prefer pinned installs" choice live here
//! so the desktop UI and the Rust commands share one file. Marketplace
//! enablement is a separate runtime switch enforced before any registry
//! request. There is no telemetry flag: MCP Manager does not send telemetry.

use crate::platform::PlatformContext;
use crate::storage::atomic_write;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const FILE_NAME: &str = "config/privacy.json";

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettings {
    /// When false, startup must not contact the update server. A manual check
    /// still may.
    #[serde(default = "default_true")]
    pub automatic_update_checks: bool,
    /// When true, an unpinned or moving marketplace package needs an explicit
    /// acknowledgement before it is added.
    #[serde(default = "default_true")]
    pub prefer_pinned_marketplace_installs: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            automatic_update_checks: true,
            prefer_pinned_marketplace_installs: true,
        }
    }
}

pub fn privacy_path(ctx: &PlatformContext) -> PathBuf {
    ctx.app_data_dir().join(FILE_NAME)
}

pub fn load_privacy(ctx: &PlatformContext) -> PrivacySettings {
    let Ok(text) = fs::read_to_string(privacy_path(ctx)) else {
        return PrivacySettings::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_privacy(ctx: &PlatformContext, settings: &PrivacySettings) -> Result<(), String> {
    let path = privacy_path(ctx);
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    atomic_write(&path, &content)
}

/// `automatic` is true for the check the app runs on its own at startup.
pub fn automatic_update_check_allowed(
    ctx: &PlatformContext,
    automatic: bool,
) -> Result<(), String> {
    if automatic && !load_privacy(ctx).automatic_update_checks {
        return Err(
            "automatic update checks are disabled; a manual check is still allowed".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformOs;

    #[test]
    fn automatic_checks_stop_when_the_preference_is_off_and_manual_checks_do_not() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ctx = PlatformContext {
            os: PlatformOs::Linux,
            home_dir: dir.path().join("home"),
            workspace_root: dir.path().join("workspace"),
        };
        assert!(automatic_update_check_allowed(&ctx, true).is_ok());
        save_privacy(
            &ctx,
            &PrivacySettings {
                automatic_update_checks: false,
                prefer_pinned_marketplace_installs: true,
            },
        )
        .expect("save");
        assert!(automatic_update_check_allowed(&ctx, true).is_err());
        assert!(automatic_update_check_allowed(&ctx, false).is_ok());
        let loaded = load_privacy(&ctx);
        assert!(!loaded.automatic_update_checks);
        assert!(loaded.prefer_pinned_marketplace_installs);
    }
}
