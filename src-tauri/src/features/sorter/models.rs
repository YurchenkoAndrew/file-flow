use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SorterOptions {
    pub source_path: String,          // Откуда берем файлы
    pub target_directory: String,     // Куда раскладываем
    pub copy_files: bool,             // true — копировать, false — перемещать
    pub group_by_year: bool,          // Создавать ли подпапки по годам
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortResultSummary {
    pub total_processed: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}