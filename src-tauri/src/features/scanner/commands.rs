use crate::database::DatabaseManager;
use crate::features::scanner::models::ScanResultSummary;
use crate::features::scanner::service::FileScanner;
use tauri::State;

#[tauri::command]
pub async fn start_scan(
    path: String,
    db_manager: State<'_, DatabaseManager>,
) -> Result<ScanResultSummary, String> {
    let pool = db_manager.pool();

    let result = FileScanner::scan_directory(pool, &path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}
