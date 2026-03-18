mod commands;
mod storage;
mod core;
mod adapters;
mod parser;
mod security;
mod platform;

use commands::{
    apply_config, import_detected_configs, load_yaml_config, rollback_from_backups, save_yaml_config,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            load_yaml_config,
            save_yaml_config,
            import_detected_configs,
            apply_config,
            rollback_from_backups
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
