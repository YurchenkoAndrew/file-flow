use super::models::{NeuroScanStatus, NeuralScanSession};
use super::repository::NeuralScannerRepository;
use super::service::NeuralScannerService;
use crate::DatabaseManager;
use tauri::{AppHandle, State};

/// Команда для запуска сканирования
/// Команда для фонового запуска всех отслеживаемых папок
#[tauri::command]
pub async fn start_neural_scan(
    folders: Vec<String>,
    db: State<'_, DatabaseManager>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let pool = db.pool().clone();

    // Защита от двойного запуска
    if let Ok(status) = NeuralScannerRepository::get_neuro_status(&pool).await {
        if status.is_running {
            return Err("Индексация уже запущена в фоне".into());
        }
    }

    // 1. МГНОВЕННАЯ РЕГИСТРАЦИЯ: сразу добавляем все выбранные папки в базу
    // Фронтенд получит их при следующем запросе, даже если сканирование еще не дошло до них
    for folder in &folders {
        let _ = NeuralScannerRepository::add_watched_folder(&pool, folder).await;
    }

    // 2. ФОНОВОЕ ВЫПОЛНЕНИЕ: отпускаем интерфейс и запускаем тяжелую работу
    tauri::async_runtime::spawn(async move {
        for folder in folders {
            let _ = NeuralScannerService::run_scan(&app_handle, &pool, &folder).await;
        }
    });

    Ok(())
}

/// Команда для поллинга прогресс-бара из Angular
#[tauri::command]
pub async fn get_neural_scan_progress(
    db: State<'_, DatabaseManager>,
) -> Result<NeuroScanStatus, String> {
    NeuralScannerRepository::get_neuro_status(db.pool())
        .await
        .map_err(|e| e.to_string())
}

/// Команда для получения статуса сканирования (для прогресс-бара)
#[tauri::command]
pub async fn get_neural_scan_status(
    session_id: i64,
    db: State<'_, DatabaseManager>,
) -> Result<Option<NeuralScanSession>, String> {
    let pool = db.pool();

    match NeuralScannerRepository::get_session(pool, session_id).await {
        Ok(session) => Ok(session),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_watched_folders_command(
    db: State<'_, DatabaseManager>,
) -> Result<Vec<String>, String> {
    let pool = db.pool();
    NeuralScannerRepository::get_watched_folders(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_watched_folder_command(
    path: String,
    db: State<'_, DatabaseManager>,
) -> Result<(), String> {
    let pool = db.pool();
    NeuralScannerRepository::remove_watched_folder(pool, &path)
        .await
        .map_err(|e| e.to_string())
}
