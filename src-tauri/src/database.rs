use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::Manager;

// Импортируем репозитории фичей, которым нужны свои таблицы
use crate::features::scanner::repository::FileScannerRepository;
use crate::features::neural_scanner::repository::NeuralScannerRepository;

pub struct DatabaseManager {
    pub pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn init(app_handle: &tauri::AppHandle) -> Result<Self, sqlx::Error> {
        let base_app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        let app_dir = base_app_dir.join("file-flow");
        tokio::fs::create_dir_all(&app_dir).await.ok();

        let db_path = app_dir.join("file_flow.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

        let pool = SqlitePool::connect(&db_url).await?;

        // ОРКЕСТРАЦИЯ МИГРАЦИЙ
        // Каждый модуль сам создает свои таблицы, если их еще нет
        FileScannerRepository::create_tables(&pool).await?;
        NeuralScannerRepository::create_tables(&pool).await?;

        // Если в будущем добавишь фичу, просто допишешь сюда одну строку:
        // SomeNewFeatureRepository::create_tables(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}