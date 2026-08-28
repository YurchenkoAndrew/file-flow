use super::models::{ExtractedDocument, NeuralScanSession, NeuralScanStatus};
use sqlx::{QueryBuilder, Row, SqlitePool};
use std::collections::HashMap;

pub struct NeuralScannerRepository;

impl NeuralScannerRepository {
    pub async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS neural_scan_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                target_path TEXT NOT NULL,
                total_files INTEGER NOT NULL DEFAULT 0,
                processed_files INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                error_message TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Вставляем "нулевую" системную сессию для фонового вотчера
            INSERT OR IGNORE INTO neural_scan_sessions (id, target_path, status)
            VALUES (0, 'Background Watcher', 'Completed');

            CREATE TABLE IF NOT EXISTS neural_documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                file_path TEXT NOT NULL UNIQUE, -- ВАЖНО: UNIQUE для UPSERT
                file_extension TEXT NOT NULL,
                text_content TEXT NOT NULL,
                embedding BLOB,
                last_modified INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES neural_scan_sessions(id) ON DELETE CASCADE
            );

            -- Новая таблица для фонового мониторинга
            CREATE TABLE IF NOT EXISTS watched_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                folder_path TEXT NOT NULL UNIQUE,
                is_watched BOOLEAN NOT NULL DEFAULT 1
            );
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Добавляем папку в фоновое отслеживание
    pub async fn add_watched_folder(pool: &SqlitePool, path: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO watched_folders (folder_path) VALUES (?)")
            .bind(path)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Получаем все папки для вотчера при запуске
    pub async fn get_watched_folders(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT folder_path FROM watched_folders WHERE is_watched = 1")
                .fetch_all(pool)
                .await?;
        Ok(rows.into_iter().map(|(r,)| r).collect())
    }

    pub async fn create_session(pool: &SqlitePool, target_path: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO neural_scan_sessions (target_path, status) VALUES (?, 'Pending')",
        )
        .bind(target_path)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update_progress(
        pool: &SqlitePool,
        session_id: i64,
        processed: usize,
        total: usize,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE neural_scan_sessions SET processed_files = ?, total_files = ?, status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(processed as i64).bind(total as i64).bind(status).bind(session_id)
            .execute(pool).await?;
        Ok(())
    }

    pub async fn save_document(
        pool: &SqlitePool,
        doc: &ExtractedDocument,
        last_modified: i64,
    ) -> Result<i64, sqlx::Error> {
        let embedding_bytes: Option<Vec<u8>> = doc.embedding.as_ref().map(|vec_f32| {
            unsafe {
                std::slice::from_raw_parts(
                    vec_f32.as_ptr() as *const u8,
                    vec_f32.len() * size_of::<f32>(),
                )
            }
            .to_vec()
        });

        // ИЗМЕНЕНО НА INSERT OR REPLACE (UPSERT)
        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO neural_documents (session_id, file_path, file_extension, text_content, embedding, last_modified)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(doc.session_id)
            .bind(&doc.file_path)
            .bind(&doc.file_extension)
            .bind(&doc.text_content)
            .bind(embedding_bytes)
            .bind(last_modified)
            .execute(pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// Возвращает карту существующих файлов ТОЛЬКО для текущей сканируемой папки
    pub async fn get_existing_files_map(
        pool: &SqlitePool,
        target_path: &str, // <-- Добавлен параметр
    ) -> Result<HashMap<String, i64>, sqlx::Error> {
        let like_path = format!("{}%", target_path);

        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT file_path, last_modified FROM neural_documents WHERE file_path LIKE ?"
        )
            .bind(like_path) // <-- Фильтруем БД по текущей папке
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().collect())
    }

    pub async fn delete_missing_files(
        pool: &SqlitePool,
        paths_to_delete: &[String],
    ) -> Result<(), sqlx::Error> {
        if paths_to_delete.is_empty() {
            return Ok(());
        }
        let mut query_builder: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("DELETE FROM neural_documents WHERE file_path IN (");
        let mut separated = query_builder.separated(", ");
        for path in paths_to_delete {
            separated.push_bind(path);
        }
        separated.push_unseparated(")");
        query_builder.build().execute(pool).await?;
        Ok(())
    }

    /// Получение информации о сессии по ID
    pub async fn get_session(
        pool: &SqlitePool,
        session_id: i64,
    ) -> Result<Option<NeuralScanSession>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, target_path, total_files, processed_files, status, error_message
            FROM neural_scan_sessions
            WHERE id = ?
            "#,
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        if let Some(r) = row {
            let status_str: String = r.try_get("status")?;
            let status = match status_str.as_str() {
                "InProgress" => NeuralScanStatus::InProgress,
                "Completed" => NeuralScanStatus::Completed,
                "Failed" => NeuralScanStatus::Failed,
                "Paused" => NeuralScanStatus::Paused,
                _ => NeuralScanStatus::Pending,
            };

            Ok(Some(NeuralScanSession {
                id: r.try_get("id")?,
                target_path: r.try_get("target_path")?,
                total_files: r.try_get::<i64, _>("total_files")? as usize,
                processed_files: r.try_get::<i64, _>("processed_files")? as usize,
                status,
                error_message: r.try_get("error_message").ok(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Удаляем папку из отслеживаемых и вычищаем все её файлы из базы
    pub async fn remove_watched_folder(pool: &SqlitePool, path: &str) -> Result<(), sqlx::Error> {
        // 1. Удаляем папку из списка отслеживаемых
        sqlx::query("DELETE FROM watched_folders WHERE folder_path = ?")
            .bind(path)
            .execute(pool)
            .await?;

        // 2. Удаляем все файлы, путь которых начинается с удаляемой папки
        let like_path = format!("{}%", path);
        sqlx::query("DELETE FROM neural_documents WHERE file_path LIKE ?")
            .bind(like_path)
            .execute(pool)
            .await?;

        Ok(())
    }
}
