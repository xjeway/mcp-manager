use crate::adapters::adapters;
use crate::core::{build_import_result, ApplyResult, DetectedServer, ImportResult, MCPConfig};
use crate::platform::PlatformContext;
use crate::storage::{apply_operations, resolve_relative_path, rollback};
use std::fs;
use std::process::Command;

const REPOSITORY_URL: &str = "https://github.com/xjeway/mcp-manager";
const RELEASES_URL: &str = "https://github.com/xjeway/mcp-manager/releases";

#[tauri::command]
pub fn load_yaml_config(relative_path: String) -> Result<String, String> {
    let path = resolve_relative_path(&relative_path);
    if !path.exists() {
        return Ok("version: 1\nservers: []\n".to_string());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_yaml_config(relative_path: String, content: String) -> Result<(), String> {
    let path = resolve_relative_path(&relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_repository_link() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(REPOSITORY_URL).status();

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", "", REPOSITORY_URL])
        .status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(REPOSITORY_URL).status();

    status.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("failed to open {}", REPOSITORY_URL))
        }
    })
}

#[tauri::command]
pub fn open_releases_link() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(RELEASES_URL).status();

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", "", RELEASES_URL])
        .status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(RELEASES_URL).status();

    status.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("failed to open {}", RELEASES_URL))
        }
    })
}

#[tauri::command]
pub fn import_detected_configs() -> Result<ImportResult, String> {
    let ctx = PlatformContext::current();
    let mut sources = Vec::new();
    let mut detected_servers = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let yaml_path = resolve_relative_path("config/servers.yaml");
    if yaml_path.exists() {
        let content = fs::read_to_string(&yaml_path).map_err(|e| e.to_string())?;
        match crate::parser::parse_yaml_config(&content) {
            Ok(config) => {
                sources.push(crate::core::LocalConfigSource {
                    app: "yaml".to_string(),
                    path: yaml_path.to_string_lossy().to_string(),
                    exists: true,
                    format: "yaml".to_string(),
                    priority: 0,
                    content: Some(content),
                });
                for server in config.servers {
                    detected_servers.push(DetectedServer {
                        server,
                        priority: 0,
                    });
                }
            }
            Err(error) => errors.push(format!("{}: {}", yaml_path.to_string_lossy(), error)),
        }
    } else {
        sources.push(crate::core::LocalConfigSource {
            app: "yaml".to_string(),
            path: yaml_path.to_string_lossy().to_string(),
            exists: false,
            format: "yaml".to_string(),
            priority: 0,
            content: None,
        });
    }

    for adapter in adapters() {
        for (path, priority) in adapter.detect_sources(&ctx) {
            let resolved = ctx.resolve_path(&path);
            if !resolved.exists() {
                sources.push(crate::core::LocalConfigSource {
                    app: adapter.app().as_str().to_string(),
                    path: resolved.to_string_lossy().to_string(),
                    exists: false,
                    format: "json".to_string(),
                    priority,
                    content: None,
                });
                continue;
            }

            let content = fs::read_to_string(&resolved).map_err(|e| e.to_string())?;
            let parsed =
                adapter.parse_source(&ctx, &resolved.to_string_lossy(), priority, &content);
            sources.extend(parsed.sources);
            warnings.extend(parsed.warnings);
            errors.extend(parsed.errors);
            detected_servers.extend(
                parsed
                    .servers
                    .into_iter()
                    .map(|(server, priority)| DetectedServer { server, priority }),
            );
        }
    }

    Ok(build_import_result(
        sources,
        detected_servers,
        warnings,
        errors,
    ))
}

#[tauri::command]
pub fn detect_installed_apps() -> Result<Vec<String>, String> {
    let ctx = PlatformContext::current();
    Ok(ctx
        .detect_installed_apps()
        .into_iter()
        .map(|app| app.as_str().to_string())
        .collect())
}

#[tauri::command]
pub fn apply_config(config: MCPConfig) -> Result<ApplyResult, String> {
    let ctx = PlatformContext::current();
    let operations = adapters()
        .into_iter()
        .map(|adapter| adapter.plan_apply(&ctx, &config))
        .collect::<Vec<_>>();

    for operation in &operations {
        let path = ctx.resolve_path(&operation.path);
        if !ctx.can_write(&path) {
            let error = format!("permission denied for {}", path.to_string_lossy());
            if crate::security::is_high_risk_condition(&error) {
                return Err(error);
            }
        }
    }

    let backups = apply_operations(operations)?;
    Ok(ApplyResult { backups })
}

#[tauri::command]
pub fn rollback_from_backups(backups: Vec<String>) -> Result<(), String> {
    rollback(backups)
}
