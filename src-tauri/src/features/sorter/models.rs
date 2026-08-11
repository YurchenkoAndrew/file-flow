use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SorterOptions {
    pub source_path: String,
    pub target_directory: String,
    pub copy_files: bool,
    pub group_by_year: bool,
    pub session_id: Option<i64>, // <-- Добавили ID сессии
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortResultSummary {
    pub total_processed: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}