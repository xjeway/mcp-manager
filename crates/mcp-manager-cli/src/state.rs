//! What the CLI remembers between runs, next to the app's servers.yaml.
//! (What `rollback` can undo lives in the core's history, shared with the app.)

use mcp_manager_core::core::SupportedApp;
use mcp_manager_core::platform::PlatformContext;
use mcp_manager_core::storage::atomic_write;
use mcp_manager_core::store;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct State {
    /// Clients picked in the last interactive `add`, preselected next time.
    pub last_apps: Vec<SupportedApp>,
}

fn path(ctx: &PlatformContext) -> PathBuf {
    ctx.app_data_dir().join("cli-state.json")
}

/// Missing or unreadable state is treated as empty: it only holds conveniences.
fn read(path: &Path) -> State {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn load(ctx: &PlatformContext) -> State {
    read(&path(ctx))
}

/// Changes the state under a lock, so two CLI runs never overwrite each other.
pub fn update(ctx: &PlatformContext, edit: impl FnOnce(&mut State)) -> Result<(), String> {
    let path = path(ctx);
    store::with_lock(&path, || {
        let mut state = read(&path);
        edit(&mut state);
        let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
        // A torn file would load as empty state.
        atomic_write(&path, &content)
    })
}
