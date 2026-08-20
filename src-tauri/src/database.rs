use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::Manager;

pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    // Инициализация базы данных и создание таблиц
    // Инициализация базы данных и создание таблиц
    pub async fn init(app_handle: &tauri::AppHandle) -> Result<Self, sqlx::Error> {
        // Получаем стандартную системную папку для данных приложения (AppData\Roaming\kz.wsa)
        let base_app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        // Изолируем базу в отдельную подпапку file-flow
        let app_dir = base_app_dir.join("file-flow");

        // Создаем директорию, если её нет
        tokio::fs::create_dir_all(&app_dir).await.ok();

        let db_path = app_dir.join("file_flow.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

        // Подключаемся к SQLite пуллу
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.run_migrations().await?;

        Ok(manager)
    }

    // Создание структуры таблиц для хранения легкой истории сессий и аналитики
    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                total_size BIGINT NOT NULL,
                total_files_count INTEGER NOT NULL,
                duplicates_size BIGINT NOT NULL,
                duplicates_count INTEGER NOT NULL DEFAULT 0,
                cleaned_size BIGINT NOT NULL DEFAULT 0,

                -- Флаги этапов (будут становиться true по мере прохождения)
                is_scanned BOOLEAN DEFAULT 1,
                is_duplicates_removed BOOLEAN DEFAULT 0,
                is_optimized BOOLEAN DEFAULT 0,

                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
            "#,
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}