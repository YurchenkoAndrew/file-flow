use sqlx::SqlitePool;

pub struct SorterRepository;
impl SorterRepository {
    /// Обновление флага оптимизации в таблице scan_sessions
    pub async fn mark_session_as_optimized(
        pool: &SqlitePool,
        session_id: Option<i64>,
        path: &str
    ) -> Result<(), sqlx::Error> {
        if let Some(id) = session_id {
            // Если пришел точный ID сессии — обновляем конкретную запись
            sqlx::query(
                r#"
                UPDATE scan_sessions
                SET is_optimized = 1
                WHERE id = ?
                "#,
            )
                .bind(id)
                .execute(pool)
                .await?;
        } else {
            // Резервный вариант по пути, если ID не передали
            sqlx::query(
                r#"
                UPDATE scan_sessions
                SET is_optimized = 1
                WHERE path = ? AND is_optimized = 0
                ORDER BY created_at DESC
                LIMIT 1
                "#,
            )
                .bind(path)
                .execute(pool)
                .await?;
        }

        Ok(())
    }
}