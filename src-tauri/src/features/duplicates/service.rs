use std::path::Path;
use tokio::fs;
use crate::features::scanner::models::DuplicateGroup;

pub struct DuplicateCleaner;

impl DuplicateCleaner {
    pub async fn remove_duplicates(groups: Vec<DuplicateGroup>) -> Result<(usize, u64), Box<dyn std::error::Error>> {
        let mut deleted_count = 0;
        let mut freed_space: u64 = 0;

        for group in groups {
            if group.files.len() < 2 {
                continue;
            }

            // Сортируем файлы в группе по дате изменения (сначала самые старые — оригиналы)
            let mut sorted_files = group.files;
            sorted_files.sort_by(|a, b| {
                let meta_a = std::fs::metadata(&a.path).ok();
                let meta_b = std::fs::metadata(&b.path).ok();

                // 1. Сравниваем по времени создания (created), если оно доступно
                let created_a = meta_a.as_ref().and_then(|m| m.created().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let created_b = meta_b.as_ref().and_then(|m| m.created().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                if created_a != created_b {
                    return created_a.cmp(&created_b);
                }

                // 2. Если время создания одинаковое, сравниваем по времени изменения (modified)
                let mod_a = meta_a.as_ref().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let mod_b = meta_b.as_ref().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                if mod_a != mod_b {
                    return mod_a.cmp(&mod_b);
                }

                // 3. Если даты идентичны, выбираем тот путь, который короче
                a.path.len().cmp(&b.path.len())
            });

            // Безопасно забираем первый элемент (оригинал оставляем), а остальное — дубликаты на удаление
            let (_original, duplicates_to_remove) = sorted_files.split_at(1);

            for file in duplicates_to_remove {
                if Path::new(&file.path).exists() {
                    if fs::remove_file(&file.path).await.is_ok() {
                        deleted_count += 1;
                        freed_space += file.size;
                    }
                }
            }
        }

        Ok((deleted_count, freed_space))
    }
}