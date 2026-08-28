pub mod database;
pub mod error;
pub mod features;
pub mod shared;

// Импорты команд
use crate::features::duplicates::commands::clean_duplicates_command;
use crate::features::neural_scanner::commands::{
    get_neural_scan_status, get_watched_folders_command, remove_watched_folder_command,
    start_neural_scan,
};
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
    // 1. ИСПРАВЛЕНИЕ КРАША ПРИ АВТОЗАПУСКЕ
    // Принудительно меняем рабочую папку на ту, где лежит наш .exe файл,
    // чтобы Windows не пыталась запустить нас из System32.
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let _ = std::env::set_current_dir(exe_dir);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // 2. ИСПРАВЛЕНИЕ ОТОБРАЖЕНИЯ ОКНА
            // Читаем аргументы запуска и проверяем, есть ли там флаг --minimized
            let args: Vec<String> = std::env::args().collect();
            let is_minimized = args.contains(&"--minimized".to_string());

            // Прячем окно, если это автозапуск
            if is_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            } else {
                // Если запустили вручную с ярлыка — показываем окно
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;

                let autostart_manager = app.autolaunch();
                if !autostart_manager.is_enabled().unwrap_or(false) {
                    let _ = autostart_manager.enable();
                }
            }

            let handle = app.handle().clone();

            // Асинхронная инициализация базы данных и фоновых процессов
            tauri::async_runtime::spawn(async move {
                match DatabaseManager::init(&handle).await {
                    Ok(db) => {
                        println!("База данных успешно инициализирована!");
                        let pool_clone = db.pool.clone();
                        handle.manage(db);

                        let watcher_tx =
                            features::neural_scanner::watcher::start_background_watcher(
                                handle.clone(),
                                pool_clone,
                            )
                            .await;

                        handle.manage(watcher_tx);
                    }
                    Err(e) => {
                        eprintln!("Ошибка инициализации базы данных: {}", e);
                    }
                }
            });

            // Настройка системного трея
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
            smart_search_command,
            get_watched_folders_command,
            remove_watched_folder_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
