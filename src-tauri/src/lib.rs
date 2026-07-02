mod commands;
mod engine;
mod models;

use tauri::Manager;

use commands::{LibraryRoot, ScanRegistry};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = app.path().app_data_dir()?.join("library");
            app.manage(LibraryRoot(root));
            app.manage(ScanRegistry::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::validate_rules,
            commands::scan_paths,
            commands::cancel_scan,
            commands::read_hex_region,
            commands::export_report,
            commands::library_list,
            commands::library_save,
            commands::library_read,
            commands::library_delete,
            commands::library_create_collection,
            commands::library_delete_collection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
