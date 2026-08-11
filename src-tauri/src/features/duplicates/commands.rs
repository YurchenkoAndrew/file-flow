use crate::features::duplicates::service::DuplicateCleaner;
use crate::features::scanner::models::DuplicateGroup;

#[tauri::command]
pub async fn remove_duplicates(groups: Vec<DuplicateGroup>) -> Result<(usize, u64), String> {
    // Вызываем сервис очистки дубликатов
    let result = DuplicateCleaner::remove_duplicates(groups)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}
