use rayon::prelude::*;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::fs;
use tokio::fs as async_fs;
use walkdir::WalkDir;

use super::embedder::NeuralEmbedder;
use super::models::ExtractedDocument;
use super::repository::NeuralScannerRepository;

pub struct NeuralScannerService;

impl NeuralScannerService {
    pub async fn run_scan(
        app_handle: &tauri::AppHandle,
        pool: &SqlitePool,
        target_path: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let session_id = NeuralScannerRepository::create_session(pool, target_path).await?;
        let path_clone = target_path.to_string();

        let mut embedder = NeuralEmbedder::new(app_handle)?;

        // 1. Загружаем из базы текущий слепок файлов: Путь -> mtime
        let existing_files = NeuralScannerRepository::get_existing_files_map(pool).await?;
        let mut db_paths: HashMap<String, bool> =
            existing_files.keys().map(|k| (k.clone(), false)).collect();

        // 2. Сканируем дисковую систему
        let disk_files = tokio::task::spawn_blocking(move || {
            // Вместо только текстовых добавляем бухгалтерские форматы изображений
            let valid_extensions = vec![
                "txt", "md", "csv", "json", "xml", "html", "log", // Текст
                "jpg", "jpeg", "png", "heic", "tiff", "tif", "bmp",
                "webp", // Изображения
            ];
            let mut scanned = Vec::new();

            for entry in WalkDir::new(path_clone).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if valid_extensions.contains(&ext.to_lowercase().as_str()) {
                            // Получаем время модификации файла
                            if let Ok(metadata) = fs::metadata(path) {
                                if let Ok(mtime) = metadata.modified() {
                                    let duration = mtime
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default();
                                    let mtime_sec = duration.as_secs() as i64;
                                    scanned.push((path.to_path_buf(), mtime_sec));
                                }
                            }
                        }
                    }
                }
            }
            scanned
        })
        .await?;

        let total_files = disk_files.len();
        NeuralScannerRepository::update_progress(pool, session_id, 0, total_files, "InProgress")
            .await?;

        let max_file_size: u64 = 5 * 1024 * 1024;
        let mut files_to_process = Vec::new();

        // 3. СВЕРКА: Фильтруем то, что не изменилось
        for (path, disk_mtime) in disk_files {
            let path_str = path.to_string_lossy().to_string();

            // Отмечаем, что файл существует на диске
            db_paths.insert(path_str.clone(), true);

            // Если файл есть в базе и его время модификации совпадает — пропускаем ИИ!
            if let Some(&db_mtime) = existing_files.get(&path_str) {
                if db_mtime == disk_mtime {
                    continue; // Файл не трогаем, он актуален
                }
            }

            // Иначе добавляем в список на обработку (новый или измененный)
            files_to_process.push((path, disk_mtime));
        }

        // 4. УДАЛЕНИЕ ПРИЗРАКОВ: Собираем пути, которых больше нет на диске
        let missing_paths: Vec<String> = db_paths
            .into_iter()
            .filter(|(_, found)| !found)
            .map(|(path, _)| path)
            .collect();

        NeuralScannerRepository::delete_missing_files(pool, &missing_paths).await?;

        let mut processed = 0;

        // 5. БАТЧИНГ ДЛЯ ИЗМЕНЕННЫХ/НОВЫХ ФАЙЛОВ
        for chunk in files_to_process.chunks(16) {
            let mut current_batch_docs = Vec::new();
            let mut current_batch_texts = Vec::new();
            let mut current_batch_meta = Vec::new();

            for (path, mtime) in chunk {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() <= max_file_size {
                        let ext = path.extension().unwrap_or_default().to_string_lossy();

                        // Определяем, откуда брать текст: из файла или из картинки через OCR
                        let content_res = if super::ocr::ImageOcr::is_image(&ext) {
                            super::ocr::ImageOcr::extract_text_from_image(path)
                        } else {
                            async_fs::read_to_string(path).await.map_err(|e| e.into())
                        };

                        if let Ok(content) = content_res {
                            if !content.trim().is_empty() {
                                current_batch_texts.push(content.clone());
                                current_batch_meta.push((path, mtime));

                                current_batch_docs.push(ExtractedDocument {
                                    id: None,
                                    session_id,
                                    file_path: path.to_string_lossy().to_string(),
                                    file_extension: ext.to_string(),
                                    text_content: content,
                                    embedding: None,
                                });
                            }
                        }
                    }
                }
            }

            if current_batch_texts.is_empty() {
                continue;
            }

            // Rayon: очистка текста
            let cleaned_texts: Vec<String> = current_batch_texts
                .into_par_iter()
                .map(|mut text| {
                    if text.len() > 2500 {
                        text.truncate(2500);
                    }
                    text.trim().to_string()
                })
                .collect();

            // Генерация векторов через ИИ
            let (embeddings_result, returned_embedder) = tokio::task::spawn_blocking(move || {
                let res = embedder
                    .generate_embeddings(cleaned_texts)
                    .map_err(|e| e.to_string());
                (res, embedder)
            })
            .await?;

            embedder = returned_embedder;

            // Сохранение в базу с учетом mtime
            if let Ok(embeddings) = embeddings_result {
                for (i, (mut doc, embedding)) in
                    current_batch_docs.into_iter().zip(embeddings).enumerate()
                {
                    doc.embedding = Some(embedding);
                    // Сохраняем документ (если нужно, можно сделать UPSERT, но пока сохраняем как есть)
                    let _ = NeuralScannerRepository::save_document(
                        pool,
                        &doc,
                        *current_batch_meta[i].1,
                    )
                    .await;
                }
            }

            processed += chunk.len();
            let _ = NeuralScannerRepository::update_progress(
                pool,
                session_id,
                processed,
                total_files,
                "InProgress",
            )
            .await;
        }

        NeuralScannerRepository::update_progress(
            pool,
            session_id,
            total_files,
            total_files,
            "Completed",
        )
        .await?;
        Ok(session_id)
    }
}
