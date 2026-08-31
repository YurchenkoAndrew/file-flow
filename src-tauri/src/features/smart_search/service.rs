use super::repository::{cosine_similarity, SmartSearchRepository};
use crate::features::neural_scanner::embedder::NeuralEmbedder;
use serde::Serialize;
use sqlx::SqlitePool;
use strsim::normalized_levenshtein; // ДОБАВЛЕНО для нечеткого поиска

#[derive(Serialize, Clone)]
pub struct SearchResultDto {
    pub id: i64,
    pub file_path: String,
    pub snippet: String,
    pub score: f32,
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

        // 1. Инициализируем эмбеддер и генерируем вектор
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

        let query_words: Vec<String> = query_lower
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_string()
            })
            .filter(|w| w.chars().all(char::is_numeric) || w.chars().count() > 2)
            .collect();

        // 3. Вычисляем релевантность (Вектор + Текст + Fuzzy)
        let mut scored_results: Vec<SearchResultDto> = docs
            .into_iter()
            .filter_map(|(id, file_path, text_content, doc_embedding)| {
                let text_lower = text_content.to_lowercase();
                let path_lower = file_path.to_lowercase();

                // ВЫДЕЛЯЕМ ШАПКУ ДОКУМЕНТА (Первые 250 символов)
                let header_len = text_lower.chars().count().min(250);
                let header_text: String = text_lower.chars().take(header_len).collect();

                // Готовим слова документа для нечеткого поиска
                let doc_words: Vec<String> = text_lower
                    .split_whitespace()
                    .map(|w| {
                        w.trim_matches(|c: char| c.is_ascii_punctuation())
                            .to_string()
                    })
                    .collect();

                let semantic_score = cosine_similarity(&query_embedding, &doc_embedding);
                let mut final_score = semantic_score;
                let mut matched_words_count = 0;

                if !query_words.is_empty() {
                    for query_word in &query_words {
                        let mut best_match_score = 0.0_f32;
                        let mut in_path = false;
                        let mut in_header = false;

                        // А. Точное совпадение в пути
                        if path_lower.contains(query_word) {
                            best_match_score = 1.0;
                            in_path = true;
                        }
                        // Б. Точное совпадение в заголовке
                        else if header_text.contains(query_word) {
                            best_match_score = 1.0;
                            in_header = true;
                        }
                        // В. Точное совпадение в теле
                        else if text_lower.contains(query_word) {
                            best_match_score = 1.0;
                        }
                        // Г. НЕЧЕТКИЙ ПОИСК (Прощение ошибок OCR)
                        else {
                            for doc_word in &doc_words {
                                // Оптимизация: пропускаем слова с разницей в длине больше 2 букв
                                if (doc_word.chars().count() as i32
                                    - query_word.chars().count() as i32)
                                    .abs()
                                    > 2
                                {
                                    continue;
                                }

                                let sim = normalized_levenshtein(query_word, doc_word) as f32;
                                // Если сходство > 80% (допускаем 1-2 ошибки, например "удостоверение" и "улостоверение")
                                if sim > 0.80 {
                                    if sim > best_match_score {
                                        best_match_score = sim;
                                        if header_text.contains(doc_word) {
                                            in_header = true;
                                        }
                                    }
                                    if best_match_score > 0.95 {
                                        break;
                                    } // Почти идеал, не ищем дальше
                                }
                            }
                        }

                        // НАЧИСЛЯЕМ БАЛЛЫ С УЧЕТОМ ВЕСОВ
                        if best_match_score > 0.0 {
                            matched_words_count += 1;

                            // Базовый балл зависит от качества совпадения (1.0 для точного, ~0.85 для нечеткого)
                            let mut word_bonus = best_match_score * 0.25;

                            if in_path {
                                word_bonus += 0.5; // Наивысший приоритет имени файла
                            } else if in_header {
                                word_bonus += 0.3; // Высокий приоритет шапке документа
                            }

                            final_score += word_bonus;
                        }
                    }

                    // Огромный бонус, если нашли прямо всю фразу целиком
                    if text_lower.contains(&query_lower) || path_lower.contains(&query_lower) {
                        final_score += 0.6;
                    }
                }

                // Отсекаем мусор
                if semantic_score < 0.25 && matched_words_count == 0 {
                    return None;
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

        // 4. Сортируем
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored_results.truncate(limit);

        // 5. Нормализация
        if let Some(top_result) = scored_results.first() {
            let max_score = top_result.score;
            for res in &mut scored_results {
                if max_score > 0.99 {
                    res.score = (res.score / max_score) * 0.99;
                } else if res.score > 0.99 {
                    res.score = 0.99;
                }
            }
        }

        Ok(scored_results)
    }
}
