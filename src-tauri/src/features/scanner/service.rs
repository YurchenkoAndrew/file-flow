use crate::features::scanner::models::{
    CategoryStat, DuplicateGroup, FileCategory, FileItem, ScanResultSummary,
};
use crate::features::scanner::repository::FileScannerRepository;
use rayon::prelude::*;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::Read;
use std::path::Path;
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

        category_stats.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        const MIN_HEAVY_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 МБ

        let mut largest_files: Vec<FileItem> = all_files
            .clone()
            .into_iter()
            .filter(|f| f.size >= MIN_HEAVY_FILE_SIZE)
            .collect();

        largest_files.sort_by(|a, b| b.size.cmp(&a.size));
        largest_files.truncate(50);

        let mut size_map: std::collections::HashMap<u64, Vec<FileItem>> =
            std::collections::HashMap::new();
        for file in &all_files {
            if file.size > 0 {
                size_map.entry(file.size).or_default().push(file.clone());
            }
        }

        let results: Vec<(u64, Vec<DuplicateGroup>)> = size_map
            .into_par_iter()
            .filter(|(_, group)| group.len() > 1)
            .map(|(_size, group)| {
                let mut local_dup_size: u64 = 0;
                let mut local_groups: Vec<DuplicateGroup> = Vec::new();
                let mut hash_map: std::collections::HashMap<String, Vec<FileItem>> =
                    std::collections::HashMap::new();

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
                                    if let Ok(tail_bytes) =
                                        Read::read(&mut file_handle, &mut tail_buffer[..])
                                    {
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

        let mut duplicates_estimated_size: u64 = 0;
        let mut duplicate_groups: Vec<DuplicateGroup> = Vec::new();

        for (dup_size, groups) in results {
            duplicates_estimated_size += dup_size;
            duplicate_groups.extend(groups);
        }

        let duplicates_count: usize = duplicate_groups
            .iter()
            .map(|g| g.files.len().saturating_sub(1))
            .sum();

        // Сохраняем сессию в базу данных SQLite и получаем её session_id
        let session_id = FileScannerRepository::save_scan_to_db(
            pool,
            target_path,
            total_size,
            total_files_count,
            duplicates_estimated_size,
            duplicates_count,
        )
        .await?;

        Ok(ScanResultSummary {
            session_id, // <-- Возвращаем ID сессии на фронтенд
            total_size,
            total_files_count,
            category_stats,
            largest_files,
            duplicates_estimated_size,
            duplicate_groups,
        })
    }
}
