use serde::{Deserialize, Serialize};

/// Статусы процесса нейросканирования
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NeuralScanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Paused,
}

/// Сессия сканирования (чтобы фронтенд мог показывать прогресс-бар)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralScanSession {
    pub id: i64,
    pub target_path: String,
    pub total_files: usize,
    pub processed_files: usize,
    pub status: NeuralScanStatus,
    pub error_message: Option<String>,
}

/// Структура извлеченного текста из файла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDocument {
    pub id: Option<i64>,
    pub session_id: i64,
    pub file_path: String,
    pub file_extension: String,
    pub text_content: String,
    pub embedding: Option<Vec<f32>>,
}
#[derive(serde::Serialize)]
pub struct GlobalScanStatus {
    pub is_running: bool,
    pub processed: usize,
    pub total: usize,
}
