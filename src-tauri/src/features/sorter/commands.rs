use crate::database::DatabaseManager;
use crate::features::sorter::models::{SorterOptions, SortResultSummary};
use crate::features::sorter::service::FileSorter;
use tauri::State;

#[tauri::command]
pub async fn start_sorting(
    options: SorterOptions,
    db_manager: State<'_, DatabaseManager>,
) -> Result<SortResultSummary, String> {
    let pool = db_manager.pool();

    let result = FileSorter::sort_files(pool, options, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}