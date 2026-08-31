use rayon::prelude::*;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

use super::embedder::NeuralEmbedder;
use super::models::ExtractedDocument;
use super::repository::NeuralScannerRepository;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
const VALID_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "json", "xml", "html", "htm", "log", "pdf", "docx", "doc", "xlsx", "xls",
    "xlsb", "ods", "jpg", "jpeg", "png", "heic", "tiff", "tif", "bmp", "webp",
];

pub struct NeuralScannerService;

impl NeuralScannerService {
    fn is_valid_extension(ext: &str) -> bool {
        VALID_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    fn is_text_valid(content: &str, ext: &str) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() { return false; }

        let letters_count = trimmed.chars().filter(|c| c.is_alphabetic()).count();
        let is_img = super::ocr::ImageOcr::is_image(ext);

        // Для картинок снизили порог с 25 до 10 букв, для документов оставили 3
        if (is_img && letters_count < 10) || (!is_img && letters_count < 3) {
            return false;
        }
        true
    }

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
                let parsed_text =
                    crate::features::neural_scanner::parser::DocumentParser::extract_text(
                        &path, &ext,
                    )
                    .unwrap_or_default();
                let letters_count = parsed_text.chars().filter(|c| c.is_alphabetic()).count();

                if ext.to_lowercase() == "pdf" && letters_count < 10 {
                    println!(
                        "🔄 PDF без текстового слоя, запускаем растеризацию и OCR: {}",
                        path.display()
                    );
                    crate::features::neural_scanner::ocr::ImageOcr::extract_text_from_pdf(
                        &app_handle,
                        &path,
                    )
                    .map_err(|e| e.to_string())
                } else {
                    Ok(parsed_text)
                }
            }
        })
        .await
        .map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )) as Box<dyn std::error::Error>
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

        // Запрашиваем ОБЕ таблицы: валидные документы и отбракованные пустышки
        let existing_files =
            NeuralScannerRepository::get_existing_files_map(pool, target_path).await?;
        let rejected_files =
            NeuralScannerRepository::get_rejected_files_map(pool, target_path).await?;

        let mut db_paths: HashMap<String, bool> = HashMap::new();
        for k in existing_files.keys() {
            db_paths.insert(k.clone(), false);
        }
        for k in rejected_files.keys() {
            db_paths.insert(k.clone(), false);
        }

        let disk_files = tokio::task::spawn_blocking(move || {
            let mut scanned = Vec::new();
            for entry in WalkDir::new(path_clone).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
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
        NeuralScannerRepository::update_progress(pool, session_id, 0, total_files, "InProgress")
            .await?;

        let mut files_to_process = Vec::new();

        // 3. СВЕРКА ПО ДВУМ КЭШАМ
        for (path, disk_mtime) in disk_files {
            let path_str = path.to_string_lossy().to_string();
            db_paths.insert(path_str.clone(), true);

            let mut skip = false;
            // Проверяем в полезных документах
            if let Some(&db_mtime) = existing_files.get(&path_str) {
                if db_mtime == disk_mtime {
                    skip = true;
                }
            }
            // Проверяем в отбракованных файлах
            else if let Some(&rej_mtime) = rejected_files.get(&path_str) {
                if rej_mtime == disk_mtime {
                    skip = true;
                }
            }

            if skip {
                continue;
            }
            files_to_process.push((path, disk_mtime));
        }

        let missing_paths: Vec<String> = db_paths
            .into_iter()
            .filter(|(_, found)| !found)
            .map(|(path, _)| path)
            .collect();
        NeuralScannerRepository::delete_missing_files(pool, &missing_paths).await?;

        let mut processed = total_files - files_to_process.len(); // Чтобы прогрессбар сразу учитывал пропущенные

        for chunk in files_to_process.chunks(16) {
            let mut current_batch_docs = Vec::new();
            let mut current_batch_texts = Vec::new();
            let mut current_batch_meta = Vec::new();
            let mut current_batch_rejected = Vec::new(); // <-- Собираем мусорные файлы

            for (path, mtime) in chunk {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() <= MAX_FILE_SIZE {
                        let ext = path
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let path_buf = path.clone();
                        let app_clone = app_handle.clone();

                        println!(">>> НАЧАЛО ОБРАБОТКИ ФАЙЛА: {}", path_buf.display());
                        let content_res =
                            Self::extract_file_content(app_clone, path_buf.clone(), ext.clone())
                                .await;

                        match content_res {
                            Ok(content) => {
                                if Self::is_text_valid(&content, &ext) {
                                    current_batch_texts.push(content.clone());
                                    current_batch_meta.push((path, mtime));

                                    current_batch_docs.push(ExtractedDocument {
                                        id: None,
                                        session_id,
                                        file_path: path.to_string_lossy().to_string(),
                                        file_extension: ext,
                                        text_content: content,
                                        embedding: None,
                                    });
                                } else {
                                    println!(
                                        "⚠️ Файл отбракован (заносим в кэш): {}",
                                        path_buf.display()
                                    );
                                    current_batch_rejected
                                        .push((path.to_string_lossy().to_string(), *mtime));
                                }
                            }
                            Err(e) => {
                                println!("❌ ОШИБКА обработки файла {}: {}", path_buf.display(), e);
                                // Ошибочные файлы тоже заносим в кэш, чтобы не биться об них головой каждый раз
                                current_batch_rejected
                                    .push((path.to_string_lossy().to_string(), *mtime));
                            }
                        }
                    }
                }
            }

            // Заносим отбракованные файлы в служебную таблицу
            for (rej_path, rej_mtime) in current_batch_rejected {
                let _ = NeuralScannerRepository::mark_file_as_rejected(pool, &rej_path, rej_mtime)
                    .await;
            }

            if !current_batch_texts.is_empty() {
                let cleaned_texts: Vec<String> = current_batch_texts
                    .into_par_iter()
                    .map(|mut text| {
                        if text.chars().count() > 2500 {
                            text = text.chars().take(2500).collect();
                        }
                        text.trim().to_string()
                    })
                    .collect();

                let (embeddings_result, returned_embedder) =
                    tokio::task::spawn_blocking(move || {
                        let res = embedder
                            .generate_embeddings(cleaned_texts)
                            .map_err(|e| e.to_string());
                        (res, embedder)
                    })
                    .await?;
                embedder = returned_embedder;

                if let Ok(embeddings) = embeddings_result {
                    for (i, (mut doc, embedding)) in
                        current_batch_docs.into_iter().zip(embeddings).enumerate()
                    {
                        doc.embedding = Some(embedding);
                        let _ = NeuralScannerRepository::save_document(
                            pool,
                            &doc,
                            *current_batch_meta[i].1,
                        )
                        .await;
                    }
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

    pub async fn scan_single_file(
        app_handle: &tauri::AppHandle,
        pool: &SqlitePool,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path_str = path.to_string_lossy().to_string();
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if !Self::is_valid_extension(&ext) {
            return Ok(());
        }

        let mtime = if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > MAX_FILE_SIZE {
                println!(
                    "⚠️ [ВОТЧЕР] Файл слишком большой (пропущен): {}",
                    path.display()
                );
                return Ok(());
            }
            metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        } else {
            return Ok(());
        };

        println!(">>> [ВОТЧЕР] НАЧАЛО ОБРАБОТКИ ФАЙЛА: {}", path.display());

        // ВАЖНО: Сразу превращаем ошибку в строку, чтобы непотокобезопасный тип
        // Box<dyn Error> не "зависал" в памяти при вызовах БД
        let content_res =
            Self::extract_file_content(app_handle.clone(), path.to_path_buf(), ext.clone())
                .await
                .map_err(|e| e.to_string());

        match content_res {
            Ok(content) => {
                if !Self::is_text_valid(&content, &ext) {
                    println!(
                        "⚠️ [ВОТЧЕР] Файл отбракован (заносим в кэш пустышек): {}",
                        path.display()
                    );
                    NeuralScannerRepository::mark_file_as_rejected(pool, &path_str, mtime).await?;
                    return Ok(());
                }

                let trimmed = content.trim();
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
                        file_path: path_str.clone(),
                        file_extension: ext,
                        text_content: cleaned_text,
                        embedding: Some(embedding),
                    };
                    NeuralScannerRepository::save_document(pool, &doc, mtime).await?;
                    println!(
                        "✅ [ВОТЧЕР] Файл успешно проиндексирован и добавлен в БД: {}",
                        path.display()
                    );
                }
            }
            Err(e_msg) => {
                println!(
                    "❌ [ВОТЧЕР] ОШИБКА обработки файла {}: {}",
                    path.display(),
                    e_msg
                );
                NeuralScannerRepository::mark_file_as_rejected(pool, &path_str, mtime).await?;
            }
        }

        Ok(())
    }
}
