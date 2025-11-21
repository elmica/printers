// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod printer;
mod watcher;

use commands::*;
use watcher::FileWatcher;

fn main() {
    tauri::Builder::default()
        .manage(FileWatcher::default())
        .invoke_handler(tauri::generate_handler![
            get_system_printers,
            get_station_config,
            save_station_config,
            get_printers,
            save_printer,
            delete_printer,
            start_file_watcher,
            stop_file_watcher,
            test_print_escpos,
            test_print_zpl,
            test_print_office,
            test_print_pdf,
        ])
        .setup(|app| {
            // Initialize config directory
            if let Some(config_dir) = app.path_resolver().app_config_dir() {
                let _ = std::fs::create_dir_all(&config_dir);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
