use super::models::{SortResultSummary, SorterOptions};
use crate::features::scanner::models::{FileCategory, FileItem};
use crate::features::scanner::service::FileScanner;
use rayon::prelude::*;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::features::sorter::repository::SorterRepository;

pub struct FileSorter;

impl FileSorter {
    /// Основной метод запуска сортировки с поддержкой автоматического сканирования при отсутствии данных
    pub async fn sort_files(
        pool: &SqlitePool,
        options: SorterOptions,
        preloaded_files: Option<Vec<FileItem>>,
    ) -> Result<SortResultSummary, Box<dyn std::error::Error>> {
        let target = Path::new(&options.target_directory);

        // 1. Получаем список файлов: либо из памяти (предыдущий шаг), либо собираем с диска
        let files_to_process = match preloaded_files {
            Some(files) if !files.is_empty() => files,
            _ => {
                // Если файлов нет в памяти, запускаем сканирование и собираем файлы с диска
                FileScanner::scan_directory(pool, &options.source_path)
                    .await
                    .ok();
                Self::collect_files_from_disk(&options.source_path)?
            }
        };

        if files_to_process.is_empty() {
            return Err("Нет файлов для сортировки".into());
        }

        let total_processed = files_to_process.len();
        let group_by_year = options.group_by_year;
        let copy_files = options.copy_files;
        let target_base = target.to_path_buf();

        // 2. Параллельно подготавливаем пути назначения на основе готовых FileItem
        let operations: Vec<(PathBuf, PathBuf, String)> = files_to_process
            .into_par_iter()
            .filter_map(|file_item| {
                let file_path = PathBuf::from(&file_item.path);
                let category_name = format!("{:?}", file_item.category);

                // Получаем год изменения файла
                let year_str = if group_by_year {
                    match std::fs::metadata(&file_path).and_then(|m| m.modified()) {
                        Ok(time) => {
                            let datetime: chrono::DateTime<chrono::Local> = time.into();
                            datetime.format("%Y").to_string()
                        }
                        Err(_) => "Unknown_Year".to_string(),
                    }
                } else {
                    "".to_string()
                };

                // Формируем путь: TargetDir / Category / Year / FileName
                let mut dest_dir = target_base.clone();
                dest_dir.push(category_name);
                if group_by_year {
                    dest_dir.push(year_str);
                }

                if let Some(file_name) = file_path.file_name() {
                    let dest_file_path = dest_dir.join(file_name);
                    Some((file_path, dest_file_path, file_item.name))
                } else {
                    None
                }
            })
            .collect();

        // 3. Асинхронно выполняем перемещение / копирование
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (src, dest, name) in operations {
            if let Some(parent) = dest.parent() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    error_count += 1;
                    errors.push(format!("Не удалось создать папку для {}: {}", name, e));
                    continue;
                }
            }

            let final_dest = Self::resolve_name_collision(dest).await;

            let res = if copy_files {
                fs::copy(&src, &final_dest).await.map(|_| ())
            } else {
                fs::rename(&src, &final_dest).await
            };

            match res {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!("Ошибка обработки файла {}: {}", name, e));
                }
            }
        }

        // 4. Обновляем флаг оптимизации сессии в базе данных
        SorterRepository::mark_session_as_optimized(pool, options.session_id, &options.source_path)
            .await
            .ok();

        Ok(SortResultSummary {
            total_processed,
            success_count,
            error_count,
            errors,
        })
    }

    /// Резервный сбор файлов через WalkDir, если пользователь вызвал сортер напрямую
    fn collect_files_from_disk(
        target_path: &str,
    ) -> Result<Vec<FileItem>, Box<dyn std::error::Error>> {
        let path = Path::new(target_path);
        if !path.exists() || !path.is_dir() {
            return Err("Указанный путь не существует".into());
        }

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let file_path = entry.path();
                let size = metadata.len();
                let name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let extension = file_path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let category = FileCategory::from_extension(&extension);

                files.push(FileItem {
                    path: file_path.to_string_lossy().to_string(),
                    name,
                    extension,
                    size,
                    category,
                });
            }
        }
        Ok(files)
    }

    /// Вспомогательный метод для разрешения конфликтов имен
    async fn resolve_name_collision(path: PathBuf) -> PathBuf {
        if !path.exists() {
            return path;
        }

        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let extension = path.extension().map(|e| e.to_string_lossy().to_string());
        let parent = path.parent().unwrap_or_else(|| Path::new(""));

        let mut counter = 1;
        loop {
            let new_file_name = match &extension {
                Some(ext) => format!("{}_{}.{}", stem, counter, ext),
                None => format!("{}_{}", stem, counter),
            };
            let new_path = parent.join(new_file_name);
            if !new_path.exists() {
                return new_path;
            }
            counter += 1;
        }
    }
}
