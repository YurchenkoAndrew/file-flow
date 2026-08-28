// Путь к твоему менеджеру базы данных (убедись, что он правильный)
use super::models::NeuralScanSession;
use super::repository::NeuralScannerRepository;
use super::service::NeuralScannerService;
use crate::DatabaseManager;
use tauri::{AppHandle, State};

/// Команда для запуска сканирования
#[tauri::command]
pub async fn start_neural_scan(
    target_path: String,
    db: State<'_, DatabaseManager>,
    app_handle: AppHandle, // <-- Запрашиваем дескриптор приложения у Tauri
) -> Result<i64, String> {
    let pool = db.pool();

    // Передаем app_handle в сервис
    match NeuralScannerService::run_scan(&app_handle, pool, &target_path).await {
        Ok(session_id) => Ok(session_id),
        Err(e) => Err(e.to_string()),
    }
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
    db: tauri::State<'_, crate::database::DatabaseManager>,
) -> Result<Vec<String>, String> {
    let pool = db.pool();
    NeuralScannerRepository::get_watched_folders(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_watched_folder_command(
    path: String,
    db: tauri::State<'_, crate::database::DatabaseManager>,
) -> Result<(), String> {
    let pool = db.pool();
    NeuralScannerRepository::remove_watched_folder(pool, &path)
        .await
        .map_err(|e| e.to_string())
}
