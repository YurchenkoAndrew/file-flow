use super::repository::{cosine_similarity, SmartSearchRepository};
use crate::features::neural_scanner::embedder::NeuralEmbedder;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Serialize, Clone)]
pub struct SearchResultDto {
    pub id: i64,
    pub file_path: String,
    pub snippet: String,
    pub score: f32, // Процент релевантности (от 0.0 до 1.0)
}

pub struct SmartSearchService;

impl SmartSearchService {
    pub async fn search(
        app_handle: &tauri::AppHandle,
        pool: &SqlitePool,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultDto>, Box<dyn std::error::Error + Send + Sync>> {
        if query_text.trim().is_empty() {
            return Ok(vec![]);
        }

        // 1. Инициализируем эмбеддер и генерируем вектор для поискового запроса
        let mut embedder = NeuralEmbedder::new(app_handle).map_err(|e| e.to_string())?;

        let query_string = query_text.to_string();

        let query_embedding = tokio::task::spawn_blocking(move || {
            let embeddings = embedder
                .generate_embeddings(vec![query_string])
                .map_err(|e| e.to_string())?;

            Ok::<Vec<f32>, String>(embeddings.into_iter().next().unwrap_or_default())
        })
        .await
        .map_err(|e| e.to_string())??;

        if query_embedding.is_empty() {
            return Ok(vec![]);
        }

        // 2. Выгружаем все документы из базы
        let docs = SmartSearchRepository::fetch_all_embeddings(pool).await?;

        // 3. Считаем косинусное расстояние для каждого документа
        let mut scored_results: Vec<SearchResultDto> = docs
            .into_iter()
            .map(|(id, file_path, text_content, doc_embedding)| {
                let score = cosine_similarity(&query_embedding, &doc_embedding);

                // Делаем короткий красивый сниппет текста для интерфейса
                let snippet = if text_content.chars().count() > 150 {
                    format!("{}...", text_content.chars().take(150).collect::<String>())
                } else {
                    text_content
                };

                SearchResultDto {
                    id,
                    file_path,
                    snippet,
                    score,
                }
            })
            .collect();

        // 4. Сортируем от самых релевантных (score ближе к 1.0) к менее релевантным
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Ограничиваем выдачу топом (например, топ-10)
        scored_results.truncate(limit);

        Ok(scored_results)
    }
}
