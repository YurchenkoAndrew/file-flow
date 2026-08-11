pub mod database;
pub mod error;
pub mod features;
pub mod shared;

// Импортируем нашу команду из структуры фичи сканера
use crate::features::scanner::commands::start_scan;
use crate::features::sorter::commands::start_sorting;
use crate::shared::commands::reveal_file_in_folder;
use database::DatabaseManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                match DatabaseManager::init(&handle).await {
                    Ok(db) => {
                        println!("База данных успешно инициализирована!");
                        handle.manage(db);
                    }
                    Err(e) => {
                        eprintln!("Ошибка инициализации базы данных: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_scan,
            reveal_file_in_folder,
            start_sorting
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
