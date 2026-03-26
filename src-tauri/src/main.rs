mod adapters;
mod commands;
mod core;
mod parser;
mod platform;
mod security;
mod storage;

use commands::{
    apply_config, detect_installed_apps, import_detected_configs, load_yaml_config,
    open_releases_link, open_repository_link, rollback_from_backups, save_yaml_config,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            load_yaml_config,
            save_yaml_config,
            open_releases_link,
            open_repository_link,
            import_detected_configs,
            detect_installed_apps,
            apply_config,
            rollback_from_backups
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
