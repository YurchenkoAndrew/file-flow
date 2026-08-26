use crate::database::DatabaseManager;
use crate::features::duplicates::models::CleanupDuplicatesResponse;
use crate::features::duplicates::service::DuplicateCleaner;
use crate::features::scanner::models::DuplicateGroup;

// Пример того, как это будет вызываться в команде Tauri:
#[tauri::command]
pub async fn clean_duplicates_command(
    session_id: i64, // Фронтенд должен передать ID сессии
    groups: Vec<DuplicateGroup>,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<CleanupDuplicatesResponse, String> {
    let pool = db.pool();

    match DuplicateCleaner::remove_duplicates(pool, session_id, groups).await {
        Ok((count, space)) => Ok(CleanupDuplicatesResponse {
            count,
            freed_space: space,
        }),
        Err(e) => Err(e.to_string()),
    }
}
