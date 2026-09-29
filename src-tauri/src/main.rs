mod commands;
mod marketplace;

use commands::{
    apply_config, current_workspace, detect_installed_apps, import_detected_configs,
    load_yaml_config, marketplace_add_source, marketplace_open_url, marketplace_refresh,
    marketplace_remove_source, marketplace_search, marketplace_set_enabled, marketplace_sources,
    open_path, open_releases_link, open_repository_link, reset_yaml_config, restart_app,
    rollback_from_backups, save_yaml_config, yaml_config_fingerprint, yaml_config_path,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            load_yaml_config,
            save_yaml_config,
            yaml_config_fingerprint,
            yaml_config_path,
            reset_yaml_config,
            open_releases_link,
            open_repository_link,
            open_path,
            current_workspace,
            import_detected_configs,
            detect_installed_apps,
            apply_config,
            rollback_from_backups,
            restart_app,
            marketplace_set_enabled,
            marketplace_sources,
            marketplace_search,
            marketplace_refresh,
            marketplace_add_source,
            marketplace_remove_source,
            marketplace_open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
