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
        let query_lower = query_text.to_lowercase();

        // Разбиваем запрос на значимые слова (числа или слова от 3 букв)
        let query_words: Vec<String> = query_lower
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()).to_string())
            .filter(|w| w.chars().all(char::is_numeric) || w.chars().count() > 2)
            .collect();

        // 3. Вычисляем релевантность (Вектор + Текст)
        let mut scored_results: Vec<SearchResultDto> = docs
            .into_iter()
            .filter_map(|(id, file_path, text_content, doc_embedding)| {
                let text_lower = text_content.to_lowercase();
                let path_lower = file_path.to_lowercase();

                // Базовая оценка от ИИ (от -1.0 до 1.0, либо 0.0 до 1.0 в зависимости от математики)
                let semantic_score = cosine_similarity(&query_embedding, &doc_embedding);
                let mut final_score = semantic_score;
                let mut matched_words_count = 0;

                // Накидываем бонусы за точные ключевые слова
                if !query_words.is_empty() {
                    for word in &query_words {
                        if text_lower.contains(word) || path_lower.contains(word) {
                            matched_words_count += 1;
                            // Жирный плюс за каждое точное совпадение
                            final_score += 0.25;
                        }
                    }

                    // Огромный бонус, если нашли прямо всю фразу целиком
                    if text_lower.contains(&query_lower) || path_lower.contains(&query_lower) {
                        final_score += 0.6;
                    }
                }

                // МЯГКИЙ ФИЛЬТР: Оставляем файл, если:
                // 1) ИИ уверен, что это оно (вектор > 0.25)
                // ИЛИ 2) Нейросеть сомневается, но есть точное совпадение хотя бы одного слова
                if semantic_score < 0.25 && matched_words_count == 0 {
                    return None; // Вот теперь мы отсекаем только откровенный мусор
                }

                let snippet = if text_content.chars().count() > 150 {
                    format!("{}...", text_content.chars().take(150).collect::<String>())
                } else {
                    text_content
                };

                Some(SearchResultDto {
                    id,
                    file_path,
                    snippet,
                    score: final_score,
                })
            })
            .collect();

        // 4. Сортируем от самых релевантных к наименее
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Оставляем только нужный лимит (топ результатов)
        scored_results.truncate(limit);

        // 5. Честная нормализация баллов для интерфейса
        if let Some(top_result) = scored_results.first() {
            let max_score = top_result.score;
            for res in &mut scored_results {
                // Если топ-результат набрал больше 1.0 (за счет бонусов), масштабируем всех относительно лидера
                if max_score > 0.99 {
                    res.score = (res.score / max_score) * 0.99;
                } else if res.score > 0.99 {
                    // Защита от выхода за 100% при низком максимуме
                    res.score = 0.99;
                }
            }
        }

        Ok(scored_results)
    }
}