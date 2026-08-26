use crate::features::duplicates::repository::DuplicatesRepository;
// <-- Добавили импорт SqlitePool
use crate::features::scanner::models::DuplicateGroup;
use sqlx::SqlitePool;
use std::path::Path;
use tokio::fs; // <-- Импортируем репозиторий

pub struct DuplicateCleaner;

impl DuplicateCleaner {
    pub async fn remove_duplicates(
        pool: &SqlitePool,           // <-- Принимаем пул БД
        session_id: i64,             // <-- Принимаем ID текущей сессии
        groups: Vec<DuplicateGroup>
    ) -> Result<(usize, u64), Box<dyn std::error::Error>> {
        let mut deleted_count = 0;
        let mut freed_space: u64 = 0;

        for group in groups {
            if group.files.len() < 2 {
                continue;
            }

            // Сортируем файлы в группе по приоритетам, чтобы оставить ИСТИННЫЙ оригинал
            let mut sorted_files = group.files;
            sorted_files.sort_by(|a, b| {
                let meta_a = std::fs::metadata(&a.path).ok();
                let meta_b = std::fs::metadata(&b.path).ok();

                // 1. ПРИОРИТЕТ: Время ИЗМЕНЕНИЯ (Modified).
                // Это самый честный показатель возраста файла, который сохраняется при копировании.
                let mod_a = meta_a.as_ref().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let mod_b = meta_b.as_ref().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                if mod_a != mod_b {
                    return mod_a.cmp(&mod_b);
                }

                // 2. ПРИОРИТЕТ: Длина имени.
                // Если даты изменения совпадают (у дубликатов так всегда),
                // оригинал имеет более короткое имя ("файл.pdf" против "файл - копия.pdf").
                let name_cmp = a.name.len().cmp(&b.name.len());
                if name_cmp != std::cmp::Ordering::Equal {
                    return name_cmp;
                }

                // 3. ПРИОРИТЕТ: Время создания (Created).
                // Используем как запасной вариант (помня, что при копировании оно сбрасывается на "сейчас").
                let created_a = meta_a.as_ref().and_then(|m| m.created().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let created_b = meta_b.as_ref().and_then(|m| m.created().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                if created_a != created_b {
                    return created_a.cmp(&created_b);
                }

                // 4. ПРИОРИТЕТ: Алфавитный порядок.
                // Если файлы полностью идентичны по всем параметрам, жестко фиксируем порядок.
                a.name.cmp(&b.name)
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

        // === СОХРАНЯЕМ В БАЗУ ДАННЫХ ===
        // Если ID сессии валидный (например, передали не 0), то обновляем статистику
        if session_id > 0 {
            DuplicatesRepository::update_duplicate_cleanup_stats(
                pool,
                session_id,
                deleted_count,
                freed_space
            ).await?;
        }

        Ok((deleted_count, freed_space))
    }
}