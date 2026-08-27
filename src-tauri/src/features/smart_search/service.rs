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

        // 3. Умная разбивка запроса: очищаем от пунктуации и фильтруем предлоги
        let query_lower = query_text.to_lowercase();

        let query_words: Vec<String> = query_lower
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()).to_string()) // Убираем точки, чтобы "г." стало "г"
            .filter(|w| {
                // Оставляем слово, ЕСЛИ это число (даже из 2 цифр, например "18")
                // ИЛИ если в нем больше 2 букв (отсекаем "на", "от", "по", "г")
                w.chars().all(char::is_numeric) || w.chars().count() > 2
            })
            .collect();

        let mut scored_results: Vec<SearchResultDto> = docs
            .into_iter()
            .filter_map(|(id, file_path, text_content, doc_embedding)| {
                let text_lower = text_content.to_lowercase();
                let path_lower = file_path.to_lowercase();

                // Базовая оценка смысла от нейросети
                let mut score = cosine_similarity(&query_embedding, &doc_embedding);

                if !query_words.is_empty() {
                    let mut matched_words_count = 0;

                    for word in &query_words {
                        if text_lower.contains(word) || path_lower.contains(word) {
                            matched_words_count += 1;
                            // Накидываем жирный бонус за КАЖДОЕ найденное слово (+15%)
                            score += 0.15;
                        }
                    }

                    // ЖЕСТКИЙ ФИЛЬТР: если ни одно слово не совпало - выбрасываем файл
                    if matched_words_count == 0 {
                        return None;
                    }

                    // Супер-бонус за точную фразу целиком
                    if text_lower.contains(&query_lower) || path_lower.contains(&query_lower) {
                        score += 0.5;
                    }
                }

                // ВАЖНО: Мы БОЛЬШЕ НЕ обрезаем score до 0.99 здесь!
                // Пусть он растет хоть до 2.0 или 3.0, чтобы точно зафиксировать разницу.

                let snippet = if text_content.chars().count() > 150 {
                    format!("{}...", text_content.chars().take(150).collect::<String>())
                } else {
                    text_content
                };

                Some(SearchResultDto {
                    id,
                    file_path,
                    snippet,
                    score,
                })
            })
            .collect();

        // 4. Сортируем по-честному, используя реальные необрезанные баллы (например, 1.8 против 1.2)
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Ограничиваем выдачу топом
        scored_results.truncate(limit);

        // 5. Нормализуем проценты для интерфейса (чтобы не было 180% или 250%)
        // Берем самый высокий балл (он теперь первый в списке) и пропорционально уменьшаем остальные
        if let Some(top_result) = scored_results.first() {
            let max_score = top_result.score;
            if max_score > 0.99 {
                for res in &mut scored_results {
                    // Идеальное масштабирование: лидер получает 99.9%, остальные - пропорционально меньше
                    res.score = (res.score / max_score) * 0.999;
                }
            }
        }

        Ok(scored_results)
    }
}
