use sqlx::SqlitePool;

pub struct FileScannerRepository;
impl FileScannerRepository {


    // Вспомогательный метод для сохранения или обновления сессии в БД
    pub async fn save_scan_to_db(
        pool: &SqlitePool,
        path: &str,
        total_size: u64,
        total_files_count: usize,
        duplicates_size: u64,
        duplicates_count: usize,
    ) -> Result<i64, sqlx::Error> {
        // 1. Ищем существующую «пустышку» (сессию без очистки и сортировки) для этого пути
        let existing_session: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT id FROM scan_sessions
            WHERE path = ? AND is_duplicates_removed = 0 AND is_optimized = 0
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
            .bind(path)
            .fetch_optional(pool)
            .await?;

        let session_id = if let Some(id) = existing_session {
            // 2. Если нашли черновик — обновляем его свежими данными
            sqlx::query(
                r#"
                UPDATE scan_sessions
                SET total_size = ?,
                    total_files_count = ?,
                    duplicates_size = ?,
                    duplicates_count = ?,
                    created_at = CURRENT_TIMESTAMP
                WHERE id = ?
                "#,
            )
                .bind(total_size as i64)
                .bind(total_files_count as i64)
                .bind(duplicates_size as i64)
                .bind(duplicates_count as i64)
                .bind(id)
                .execute(pool)
                .await?;

            id
        } else {
            // 3. Если сессии нет — создаем новую без упоминания удаленной колонки status
            let new_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO scan_sessions (
                    path, total_size, total_files_count, duplicates_size, duplicates_count,
                    is_scanned, is_duplicates_removed, is_optimized
                )
                VALUES (?, ?, ?, ?, ?, 1, 0, 0)
                RETURNING id
                "#,
            )
                .bind(path)
                .bind(total_size as i64)
                .bind(total_files_count as i64)
                .bind(duplicates_size as i64)
                .bind(duplicates_count as i64)
                .fetch_one(pool)
                .await?;

            new_id
        };

        Ok(session_id)
    }
}