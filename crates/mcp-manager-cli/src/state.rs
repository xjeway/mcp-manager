//! What the CLI remembers between runs, next to the app's servers.yaml.

use mcp_manager_core::core::{MCPConfig, SupportedApp};
use mcp_manager_core::platform::PlatformContext;
use mcp_manager_core::storage::atomic_write;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct State {
    /// Clients picked in the last interactive `add`, preselected next time.
    pub last_apps: Vec<SupportedApp>,
    /// Client-file backups from the last change, for `rollback`.
    pub last_backups: Vec<String>,
    /// servers.yaml as it was before the last change, for `rollback`.
    pub last_previous_config: Option<MCPConfig>,
    /// servers.yaml as the last change wrote it. `rollback` refuses to run if
    /// the file no longer matches, so it never undoes someone else's edit.
    pub last_applied_config: Option<MCPConfig>,
    /// Client files the last change wrote, as it left them. `rollback`
    /// refuses if any was edited since, so it never discards that edit.
    pub last_written_files: Vec<WrittenFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WrittenFile {
    pub path: String,
    /// `store::fingerprint` of the file right after the change.
    pub fingerprint: String,
}

impl State {
    pub fn has_rollback(&self) -> bool {
        self.last_previous_config.is_some()
    }

    pub fn clear_rollback(&mut self) {
        self.last_backups.clear();
        self.last_previous_config = None;
        self.last_applied_config = None;
        self.last_written_files.clear();
    }
}

fn path(ctx: &PlatformContext) -> PathBuf {
    ctx.app_data_dir().join("cli-state.json")
}

/// Missing or unreadable state is treated as empty: it only holds conveniences.
pub fn load(ctx: &PlatformContext) -> State {
    std::fs::read_to_string(path(ctx))
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save(ctx: &PlatformContext, state: &State) -> Result<(), String> {
    let content = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    // A torn file would load as empty state and lose the rollback record.
    atomic_write(&path(ctx), &content)
}
