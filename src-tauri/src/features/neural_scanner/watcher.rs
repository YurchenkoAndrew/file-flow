use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use sqlx::SqlitePool;
use std::path::Path;
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::mpsc;
use crate::features::neural_scanner::service::NeuralScannerService;
use super::repository::NeuralScannerRepository;

pub async fn start_background_watcher(app_handle: AppHandle, pool: SqlitePool) -> mpsc::Sender<String> {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(10);
    let (std_cmd_tx, std_cmd_rx) = std::sync::mpsc::channel::<String>();

    // Мост: передаем новые папки из асинхронного Tauri в синхронный поток вотчера
    tokio::spawn(async move {
        while let Some(path) = cmd_rx.recv().await {
            let _ = std_cmd_tx.send(path);
        }
    });

    let (event_tx, mut event_rx) = mpsc::channel(100);
    let pool_clone = pool.clone();
    let app_clone = app_handle.clone();

    // Асинхронный обработчик пойманных событий ФС
    tokio::spawn(async move {
        while let Some(events) = event_rx.recv().await {
            let evs: Vec<notify_debouncer_mini::DebouncedEvent> = events;
            for event in evs {
                let path = event.path;
                let path_str = path.to_string_lossy().to_string();

                if !path.exists() {
                    // Файл был удален
                    println!("🗑 Фоновый вотчер: удален файл {}", path.display());
                    let _ = NeuralScannerRepository::delete_missing_files(&pool_clone, &[path_str]).await;
                } else if path.is_file() {
                    // Файл добавлен или изменен -> точечное пересканирование
                    println!("🔄 Фоновый вотчер: изменен файл {}", path.display());
                    let _ = NeuralScannerService::scan_single_file(&app_clone, &pool_clone, &path).await;
                }
            }
        }
    });

    // Блокирующий поток ядра вотчера
    tokio::task::spawn_blocking(move || {
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_secs(2), notify_tx).unwrap(); // Задержка 2 сек, чтобы файл успел сохраниться на диск

        // Подгружаем папки при старте
        let rt = tokio::runtime::Handle::current();
        let folders = rt.block_on(async {
            NeuralScannerRepository::get_watched_folders(&pool).await.unwrap_or_default()
        });

        for folder in folders {
            let _ = debouncer.watcher().watch(Path::new(&folder), RecursiveMode::Recursive);
            println!("👀 Вотчер подписался на папку: {}", folder);
        }

        // Бесконечный цикл прослушивания
        loop {
            // Ждем события от операционной системы (с тайм-аутом)
            match notify_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(events)) => { let _ = event_tx.blocking_send(events); }
                _ => {}
            }

            // Проверяем, не попросил ли юзер следить за новой папкой
            if let Ok(new_folder) = std_cmd_rx.try_recv() {
                let _ = debouncer.watcher().watch(Path::new(&new_folder), RecursiveMode::Recursive);
                println!("👀 Вотчер добавил новую папку на лету: {}", new_folder);
            }
        }
    });

    cmd_tx
}