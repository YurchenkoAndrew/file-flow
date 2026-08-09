use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::Manager;

pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    // Инициализация базы данных и создание таблиц
    pub async fn init(app_handle: &tauri::AppHandle) -> Result<Self, sqlx::Error> {
        // Получаем стандартную системную папку для данных приложения (например, AppData на Windows)
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

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

    // Создание структуры таблиц для хранения сессий «До / После»
    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                total_size BIGINT NOT NULL,
                total_files_count INTEGER NOT NULL,
                duplicates_size BIGINT NOT NULL,
                status TEXT NOT NULL, -- 'scanned' или 'optimized'
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS file_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                path TEXT NOT NULL,
                name TEXT NOT NULL,
                extension TEXT NOT NULL,
                size BIGINT NOT NULL,
                category TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES scan_sessions (id) ON DELETE CASCADE
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
