use crate::shared::services::file_open_service::FileOpenerService;

#[tauri::command]
pub async fn reveal_file_in_folder(path: String) -> Result<(), String> {
    // Вся логика скрыта в сервисе, команда только делегирует вызов
    FileOpenerService::reveal_in_folder(&path)
}
