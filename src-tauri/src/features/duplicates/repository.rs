use sqlx::SqlitePool;

pub struct DuplicatesRepository;
impl DuplicatesRepository {
    pub async fn update_duplicate_cleanup_stats(
        pool: &SqlitePool,
        session_id: i64,
        deleted_count: usize,
        freed_space: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        UPDATE scan_sessions
        SET cleaned_size = ?,
            cleaned_files_count = ?,
            is_duplicates_removed = 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
        )
            .bind(freed_space as i64)
            .bind(deleted_count as i64)
            .bind(session_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}