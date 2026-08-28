use rayon::prelude::*;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

use super::embedder::NeuralEmbedder;
use super::models::ExtractedDocument;
use super::repository::NeuralScannerRepository;

// Выносим константы на уровень модуля
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5 МБ
const VALID_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "json", "xml", "html", "log", "pdf", "docx", "jpg", "jpeg", "png", "heic",
    "tiff", "tif", "bmp", "webp",
];



pub struct NeuralScannerService;

impl NeuralScannerService {
    /// Хелпер: проверка расширения файла
    fn is_valid_extension(ext: &str) -> bool {
        VALID_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    /// Хелпер: проверка текста на мусор после OCR/парсера
    fn is_text_valid(content: &str, ext: &str) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }

        let letters_count = trimmed.chars().filter(|c| c.is_alphabetic()).count();
        let is_img = super::ocr::ImageOcr::is_image(ext);

        // Для картинок требуем минимум 25 букв, для документов — 3 буквы
        if (is_img && letters_count < 25) || (!is_img && letters_count < 3) {
            return false;
        }
        true
    }

    /// Хелпер: извлечение текста из файла или картинки в фоновом потоке
    async fn extract_file_content(
        app_handle: tauri::AppHandle,
        path: std::path::PathBuf,
        ext: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        tokio::task::spawn_blocking(move || {
            if super::ocr::ImageOcr::is_image(&ext) {
                super::ocr::ImageOcr::extract_text_from_image(&app_handle, &path)
                    .map_err(|e| e.to_string())
            } else {
                crate::features::neural_scanner::parser::DocumentParser::extract_text(&path, &ext)
                    .map_err(|e| e.to_string())
            }
        })
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                    as Box<dyn std::error::Error>
            })?
            .map_err(|e| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
                    as Box<dyn std::error::Error>
            })
    }

    pub async fn run_scan(
        app_handle: &tauri::AppHandle,
        pool: &SqlitePool,
        target_path: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        use tauri::Manager;

        NeuralScannerRepository::add_watched_folder(pool, target_path).await?;
        if let Some(watcher_tx) = app_handle.try_state::<tokio::sync::mpsc::Sender<String>>() {
            let _ = watcher_tx.send(target_path.to_string()).await;
        }

        let session_id = NeuralScannerRepository::create_session(pool, target_path).await?;
        let path_clone = target_path.to_string();

        let mut embedder = NeuralEmbedder::new(app_handle)?;

        let existing_files = NeuralScannerRepository::get_existing_files_map(pool).await?;
        let mut db_paths: HashMap<String, bool> =
            existing_files.keys().map(|k| (k.clone(), false)).collect();

        // 2. Сканируем дисковую систему
        let disk_files = tokio::task::spawn_blocking(move || {
            let mut scanned = Vec::new();
            for entry in WalkDir::new(path_clone).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        // Используем наш хелпер
                        if Self::is_valid_extension(ext) {
                            if let Ok(metadata) = fs::metadata(path) {
                                if let Ok(mtime) = metadata.modified() {
                                    let duration = mtime
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default();
                                    scanned.push((path.to_path_buf(), duration.as_secs() as i64));
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
        NeuralScannerRepository::update_progress(pool, session_id, 0, total_files, "InProgress").await?;

        let mut files_to_process = Vec::new();

        // 3. СВЕРКА
        for (path, disk_mtime) in disk_files {
            let path_str = path.to_string_lossy().to_string();
            db_paths.insert(path_str.clone(), true);

            if let Some(&db_mtime) = existing_files.get(&path_str) {
                if db_mtime == disk_mtime { continue; }
            }
            files_to_process.push((path, disk_mtime));
        }

        // 4. УДАЛЕНИЕ ПРИЗРАКОВ
        let missing_paths: Vec<String> = db_paths
            .into_iter()
            .filter(|(_, found)| !found)
            .map(|(path, _)| path)
            .collect();

        NeuralScannerRepository::delete_missing_files(pool, &missing_paths).await?;

        let mut processed = 0;

        // 5. БАТЧИНГ
        for chunk in files_to_process.chunks(16) {
            let mut current_batch_docs = Vec::new();
            let mut current_batch_texts = Vec::new();
            let mut current_batch_meta = Vec::new();

            for (path, mtime) in chunk {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() <= MAX_FILE_SIZE { // Используем константу
                        let ext = path.extension().unwrap_or_default().to_string_lossy().to_string();
                        let path_buf = path.clone();
                        let app_clone = app_handle.clone();

                        println!(">>> НАЧАЛО ОБРАБОТКИ ФАЙЛА: {}", path_buf.display());

                        let ext_clone = ext.clone();
                        let content_res = Self::extract_file_content(app_clone, path_buf, ext_clone).await;

                        if let Ok(content) = content_res {
                            // Используем хелпер валидации текста
                            if Self::is_text_valid(&content, &ext) {
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

            if current_batch_texts.is_empty() { continue; }

            let cleaned_texts: Vec<String> = current_batch_texts
                .into_par_iter()
                .map(|mut text| {
                    if text.chars().count() > 2500 {
                        text = text.chars().take(2500).collect();
                    }
                    text.trim().to_string()
                })
                .collect();

            let (embeddings_result, returned_embedder) = tokio::task::spawn_blocking(move || {
                let res = embedder.generate_embeddings(cleaned_texts).map_err(|e| e.to_string());
                (res, embedder)
            }).await?;
            embedder = returned_embedder;

            if let Ok(embeddings) = embeddings_result {
                for (i, (mut doc, embedding)) in current_batch_docs.into_iter().zip(embeddings).enumerate() {
                    doc.embedding = Some(embedding);
                    let _ = NeuralScannerRepository::save_document(pool, &doc, *current_batch_meta[i].1).await;
                }
            }

            processed += chunk.len();
            let _ = NeuralScannerRepository::update_progress(pool, session_id, processed, total_files, "InProgress").await;
        }

        NeuralScannerRepository::update_progress(pool, session_id, total_files, total_files, "Completed").await?;
        Ok(session_id)
    }

    pub async fn scan_single_file(
        app_handle: &tauri::AppHandle,
        pool: &SqlitePool,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path_str = path.to_string_lossy().to_string();
        let ext = path.extension().unwrap_or_default().to_string_lossy().to_string();

        // Проверяем расширение через хелпер
        if !Self::is_valid_extension(&ext) {
            return Ok(());
        }

        let mtime = if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > MAX_FILE_SIZE { // Проверяем размер через константу
                return Ok(());
            }
            metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64
        } else {
            return Ok(());
        };

        let app_clone = app_handle.clone();
        let path_buf = path.to_path_buf();
        let ext_clone = ext.clone();

        let content_res = Self::extract_file_content(app_clone, path_buf, ext_clone).await?;

        // Валидация текста через хелпер
        if !Self::is_text_valid(&content_res, &ext) {
            return Ok(());
        }

        let trimmed = content_res.trim();
        let cleaned_text: String = if trimmed.chars().count() > 2500 {
            trimmed.chars().take(2500).collect()
        } else {
            trimmed.to_string()
        };

        let mut embedder = NeuralEmbedder::new(app_handle)?;
        let embeddings = embedder.generate_embeddings(vec![cleaned_text.clone()])?;

        if let Some(embedding) = embeddings.into_iter().next() {
            let doc = ExtractedDocument {
                id: None,
                session_id: 0,
                file_path: path_str,
                file_extension: ext,
                text_content: cleaned_text,
                embedding: Some(embedding),
            };
            NeuralScannerRepository::save_document(pool, &doc, mtime).await?;
        }

        Ok(())
    }
}