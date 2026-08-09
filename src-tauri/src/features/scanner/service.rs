use std::fs::File;
use std::io::Read;
use crate::features::scanner::models::{CategoryStat, DuplicateGroup, FileCategory, FileItem, ScanResultSummary};
use sqlx::SqlitePool;
use std::path::Path;
use rayon::prelude::*; // Подключаем параллельные итераторы Rayon
use walkdir::WalkDir;

pub struct FileScanner;

impl FileScanner {
    // Основной метод сканирования директории
    pub async fn scan_directory(
        pool: &SqlitePool,
        target_path: &str,
    ) -> Result<ScanResultSummary, Box<dyn std::error::Error>> {
        let path = Path::new(target_path);
        if !path.exists() || !path.is_dir() {
            return Err("Указанный путь не существует или не является директорией".into());
        }

        let mut total_size: u64 = 0;
        let mut total_files_count: usize = 0;
        let mut category_map: std::collections::HashMap<FileCategory, (u64, usize)> =
            std::collections::HashMap::new();
        let mut all_files: Vec<FileItem> = Vec::new();

        // Рекурсивный обход директории с помощью walkdir
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let file_path_buf = entry.path();
                let size = metadata.len();
                total_size += size;
                total_files_count += 1;

                let name = file_path_buf
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let extension = file_path_buf
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let category = FileCategory::from_extension(&extension);

                // Накапливаем статистику по категории
                let entry_stat = category_map.entry(category.clone()).or_insert((0, 0));
                entry_stat.0 += size;
                entry_stat.1 += 1;

                let file_item = FileItem {
                    path: file_path_buf.to_string_lossy().to_string(),
                    name,
                    extension,
                    size,
                    category,
                };

                all_files.push(file_item);
            }
        }

        // Формируем статистику по категориям с процентами
        let mut category_stats = Vec::new();
        for (cat, (size, count)) in category_map {
            let percentage = if total_size > 0 {
                (size as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };

            category_stats.push(CategoryStat {
                category: cat,
                total_size: size,
                files_count: count,
                percentage,
            });
        }

        // Сортируем категории по размеру (от больших к меньшим)
        category_stats.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        // Находим топ тяжелых файлов (например, топ-50)
        // Устанавливаем порог для тяжелых файлов: например, всё, что больше 100 МБ
        const MIN_HEAVY_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 МБ

        let mut largest_files: Vec<FileItem> = all_files
            .clone()
            .into_iter()
            .filter(|f| f.size >= MIN_HEAVY_FILE_SIZE)
            .collect();

        largest_files.sort_by(|a, b| b.size.cmp(&a.size));
        largest_files.truncate(50);


        // 1. Группируем файлы по размеру (проверяем все файлы больше 0 байт)
        let mut size_map: std::collections::HashMap<u64, Vec<FileItem>> = std::collections::HashMap::new();
        for file in &all_files {
            if file.size > 0 {
                size_map.entry(file.size).or_default().push(file.clone());
            }
        }

        // let mut duplicates_estimated_size: u64 = 0;
        // let mut duplicate_groups: Vec<Vec<FileItem>> = Vec::new();

        // 2. Параллельно обрабатываем группы файлов с одинаковым размером
        let results: Vec<(u64, Vec<DuplicateGroup>)> = size_map
            .into_par_iter()
            .filter(|(_, group)| group.len() > 1)
            .map(|(_size, group)| {
                let mut local_dup_size: u64 = 0;
                let mut local_groups: Vec<DuplicateGroup> = Vec::new();
                let mut hash_map: std::collections::HashMap<String, Vec<FileItem>> = std::collections::HashMap::new();

                for file in group {
                    if let Ok(mut file_handle) = File::open(&file.path) {
                        let mut context = md5::Context::new();
                        let mut head_buffer = [0u8; 4096];

                        if let Ok(head_bytes) = Read::read(&mut file_handle, &mut head_buffer[..]) {
                            context.consume(&head_buffer[..head_bytes]);

                            if file.size > 8192 {
                                use std::io::Seek;
                                use std::io::SeekFrom;

                                if file_handle.seek(SeekFrom::End(-4096)).is_ok() {
                                    let mut tail_buffer = [0u8; 4096];
                                    if let Ok(tail_bytes) = Read::read(&mut file_handle, &mut tail_buffer[..]) {
                                        context.consume(&tail_buffer[..tail_bytes]);
                                    }
                                }
                            }

                            let hash = format!("{:x}", context.finalize());
                            hash_map.entry(hash).or_default().push(file);
                        }
                    }
                }

                for (_hash, dup_group) in hash_map {
                    if dup_group.len() > 1 {
                        let duplicate_count = dup_group.len() - 1;
                        if let Some(first_file) = dup_group.first() {
                            local_dup_size += first_file.size * (duplicate_count as u64);
                            local_groups.push(DuplicateGroup {
                                size: first_file.size,
                                files: dup_group,
                            });
                        }
                    }
                }

                (local_dup_size, local_groups)
            })
            .collect();

        // Аккуратно собираем результаты со всех потоков без задвоений
        let mut duplicates_estimated_size: u64 = 0;
        let mut duplicate_groups: Vec<DuplicateGroup> = Vec::new();

        for (dup_size, groups) in results {
            duplicates_estimated_size += dup_size;
            duplicate_groups.extend(groups);
        }

        // Сохраняем сессию в базу данных SQLite
        Self::save_scan_to_db(
            pool,
            target_path,
            total_size,
            total_files_count,
            duplicates_estimated_size,
            &all_files,
        )
        .await?;

        Ok(ScanResultSummary {
            total_size,
            total_files_count,
            category_stats,
            largest_files,
            duplicates_estimated_size,
            duplicate_groups,
        })
    }

    // Вспомогательный метод для сохранения результатов в БД
    async fn save_scan_to_db(
        pool: &SqlitePool,
        path: &str,
        total_size: u64,
        total_files_count: usize,
        duplicates_size: u64,
        files: &[FileItem],
    ) -> Result<(), sqlx::Error> {
        // Начинаем транзакцию
        let mut tx = pool.begin().await?;

        // Сохраняем сессию сканирования
        let session_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO scan_sessions (path, total_size, total_files_count, duplicates_size, status)
            VALUES (?, ?, ?, ?, 'scanned')
            RETURNING id
            "#,
        )
        .bind(path)
        .bind(total_size as i64)
        .bind(total_files_count as i64)
        .bind(duplicates_size as i64)
        .fetch_one(&mut *tx)
        .await?;

        // Пакетное сохранение файлов (или поочередное)
        for file in files {
            let category_str = format!("{:?}", file.category); // Или сохраняем строковое представление
            sqlx::query(
                r#"
                INSERT INTO file_items (session_id, path, name, extension, size, category)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(session_id)
            .bind(&file.path)
            .bind(&file.name)
            .bind(&file.extension)
            .bind(file.size as i64)
            .bind(category_str)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
