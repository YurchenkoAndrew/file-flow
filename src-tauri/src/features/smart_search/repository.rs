use sqlx::SqlitePool;

pub struct SmartSearchRepository;

impl SmartSearchRepository {
    /// Получает все документы с их векторными представлениями из базы данных
    pub async fn fetch_all_embeddings(
        pool: &SqlitePool,
    ) -> Result<Vec<(i64, String, String, Vec<f32>)>, sqlx::Error> {
        let rows: Vec<(i64, String, String, Option<Vec<u8>>)> = sqlx::query_as(
            r#"
            SELECT id, file_path, text_content, embedding
            FROM neural_documents
            WHERE embedding IS NOT NULL
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut results = Vec::new();

        for (id, file_path, text_content, embedding_bytes_opt) in rows {
            if let Some(bytes) = embedding_bytes_opt {
                // Превращаем байты обратно в Vec<f32>
                let f32_slice = unsafe {
                    std::slice::from_raw_parts(
                        bytes.as_ptr() as *const f32,
                        bytes.len() / size_of::<f32>(),
                    )
                };
                results.push((id, file_path, text_content, f32_slice.to_vec()));
            }
        }

        Ok(results)
    }
}

/// Функция расчета косинусного сходства между двумя векторами
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}
