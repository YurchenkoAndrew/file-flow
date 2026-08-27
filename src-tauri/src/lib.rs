pub mod database;
pub mod error;
pub mod features;
pub mod shared;

// Импорты команд
use crate::features::duplicates::commands::clean_duplicates_command;
use crate::features::neural_scanner::commands::{get_neural_scan_status, start_neural_scan};
use crate::features::scanner::commands::start_scan;
use crate::features::sorter::commands::start_sorting;
use crate::shared::commands::reveal_file_in_folder;
use database::DatabaseManager;
// Базовые импорты Tauri + встроенный трей
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;

// Единственный сторонний плагин для автозапуска
use crate::features::smart_search::commands::smart_search_command;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Стандартные плагины
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        // Плагин автозапуска
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        // Настройка приложения при старте
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::MacosLauncher;

                // Инициализируем плагин
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;

                // Делаем автозапуск активным по умолчанию при первом запуске
                let autostart_manager = app.autolaunch();
                if !autostart_manager.is_enabled().unwrap_or(false) {
                    let _ = autostart_manager.enable();
                }
            }

            let handle = app.handle().clone();

            // Асинхронная инициализация базы данных
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

            // Настройка системного трея (работает благодаря фиче "tray-icon" в Cargo.toml)
            let show_i = MenuItem::with_id(app, "show", "Развернуть", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("File Flow Scanner")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        // Перехват нажатия на крестик (сворачивание в трей)
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        // Регистрация команд
        .invoke_handler(tauri::generate_handler![
            start_scan,
            reveal_file_in_folder,
            start_sorting,
            clean_duplicates_command,
            start_neural_scan,
            get_neural_scan_status,
            smart_search_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
