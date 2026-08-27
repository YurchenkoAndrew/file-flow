use super::service::{SearchResultDto, SmartSearchService};
use crate::database::DatabaseManager;
use tauri::State;

#[tauri::command]
pub async fn smart_search_command(
    app_handle: tauri::AppHandle,
    db_state: State<'_, DatabaseManager>,
    query: String,
) -> Result<Vec<SearchResultDto>, String> {
    let pool = db_state.pool();

    SmartSearchService::search(&app_handle, pool, &query, 10)
        .await
        .map_err(|e| e.to_string())
}
