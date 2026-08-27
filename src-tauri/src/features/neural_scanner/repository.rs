use super::models::{ExtractedDocument, NeuralScanSession, NeuralScanStatus};
use sqlx::{QueryBuilder, Row, SqlitePool};
use std::collections::HashMap;

pub struct NeuralScannerRepository;

impl NeuralScannerRepository {
    /// 1. Создание таблиц (миграция)
    /// Эту функцию мы потом вызовем в главном DatabaseManager::init()
    pub async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            -- Таблица для хранения сессий сканирования
            CREATE TABLE IF NOT EXISTS neural_scan_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                target_path TEXT NOT NULL,
                total_files INTEGER NOT NULL DEFAULT 0,
                processed_files INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL, -- Pending, InProgress, Completed, Failed
                error_message TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Таблица для хранения распознанных текстов
            CREATE TABLE IF NOT EXISTS neural_documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_extension TEXT NOT NULL,
                text_content TEXT NOT NULL,
                embedding BLOB,
                last_modified INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES neural_scan_sessions(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 2. Создание новой сессии сканирования
    pub async fn create_session(pool: &SqlitePool, target_path: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO neural_scan_sessions (target_path, status)
            VALUES (?, 'Pending')
            "#,
        )
        .bind(target_path)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 3. Обновление прогресса сессии
    pub async fn update_progress(
        pool: &SqlitePool,
        session_id: i64,
        processed: usize,
        total: usize,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE neural_scan_sessions
            SET processed_files = ?,
                total_files = ?,
                status = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(processed as i64)
        .bind(total as i64)
        .bind(status)
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Сохранение извлеченного текста, его вектора и даты модификации
    pub async fn save_document(
        pool: &SqlitePool,
        doc: &ExtractedDocument,
        last_modified: i64,
    ) -> Result<i64, sqlx::Error> {
        let embedding_bytes: Option<Vec<u8>> = doc.embedding.as_ref().map(|vec_f32| {
            let bytes_slice = unsafe {
                std::slice::from_raw_parts(
                    vec_f32.as_ptr() as *const u8,
                    vec_f32.len() * std::mem::size_of::<f32>(),
                )
            };
            bytes_slice.to_vec()
        });

        let result = sqlx::query(
            r#"
            INSERT INTO neural_documents (session_id, file_path, file_extension, text_content, embedding, last_modified)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(doc.session_id)
            .bind(&doc.file_path)
            .bind(&doc.file_extension)
            .bind(&doc.text_content)
            .bind(embedding_bytes)
            .bind(last_modified) // Сохраняем таймстамп файла
            .execute(pool)
            .await?;

        Ok(result.last_insert_rowid())
    }
    /// 5. Получение информации о сессии по ID
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
            // Превращаем строку из базы в наш Enum
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
                // В SQLite целые числа часто возвращаются как i64, приводим к usize
                total_files: r.try_get::<i64, _>("total_files")? as usize,
                processed_files: r.try_get::<i64, _>("processed_files")? as usize,
                status,
                error_message: r.try_get("error_message").ok(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Возвращает карту существующих файлов: путь -> last_modified
    pub async fn get_existing_files_map(
        pool: &SqlitePool,
    ) -> Result<HashMap<String, i64>, sqlx::Error> {
        let rows: Vec<(String, i64)> =
            sqlx::query_as("SELECT file_path, last_modified FROM neural_documents")
                .fetch_all(pool)
                .await?;

        Ok(rows.into_iter().collect())
    }

    /// Удаление документов-призраков по списку путей, которых больше нет на диске
    pub async fn delete_missing_files(
        pool: &SqlitePool,
        paths_to_delete: &[String],
    ) -> Result<(), sqlx::Error> {
        if paths_to_delete.is_empty() {
            return Ok(());
        }

        // Используем QueryBuilder для безопасной сборки динамического IN (...)
        let mut query_builder: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("DELETE FROM neural_documents WHERE file_path IN (");

        let mut separated = query_builder.separated(", ");
        for path in paths_to_delete {
            separated.push_bind(path);
        }
        separated.push_unseparated(")");

        let query = query_builder.build();
        query.execute(pool).await?;

        Ok(())
    }
}
