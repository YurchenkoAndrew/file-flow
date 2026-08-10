use serde::{Serialize, Deserialize};

// Группа дубликатов с одинаковым содержимым
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DuplicateGroup {
    pub size: u64,              // Размер файлов в группе в байтах
    pub files: Vec<FileItem>,   // Список файлов-дубликатов с путями
}

// Статистика по конкретной категории для круговой диаграммы и списков
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryStat {
    pub category: FileCategory, // Сама категория (Image, Video и т.д.)
    pub total_size: u64,        // Суммарный размер файлов в байтах
    pub files_count: usize,     // Количество файлов в этой категории
    pub percentage: f32,        // Процент от общего объема сканирования (0.0 - 100.0)
}

// Главная структура данных одного найденного файла
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileItem {
    pub path: String,           // Абсолютный путь к файлу на диске
    pub name: String,           // Имя файла с расширением
    pub extension: String,      // Расширение (например, "jpg", "mp4")
    pub size: u64,              // Размер в байтах
    pub category: FileCategory, // Автоматически определяемая категория
}

// Итоговая сводка, которую сканер возвращает во фронтенд за один раз
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScanResultSummary {
    pub total_size: u64,                   // Общий объем в байтах
    pub total_files_count: usize,          // Всего файлов
    pub category_stats: Vec<CategoryStat>, // Статистика по категориям для графиков
    pub largest_files: Vec<FileItem>,      // Топ-тяжелых файлов
    pub duplicates_estimated_size: u64,    // Примерный объем дубликатов
    pub duplicate_groups: Vec<DuplicateGroup>, // Список найденных групп дубликатов
}

// Перечисление категорий файлов
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Image,
    Video,
    Document,
    Audio,
    Archive,
    Code,
    Software,
    Mobile,
    DiskImages,
    Fonts,
    DesignProjects, // Дизайн-исходники (PSD, AI, CDR и т.д.)
    VideoProjects,  // Проекты видеомонтажа (Premiere, After Effects, Vegas)
    Other,
}

impl FileCategory {
    pub fn from_extension(ext: &str) -> Self {
        let ext = ext.to_lowercase();
        match ext.as_str() {
            // Изображения и графические исходники
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "heic" | "heif" | "bmp" | "tiff" | "tif"
            | "ico" | "cur" | "tga" | "xcf" | "pdn" | "raw" | "cr2"
            | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "avif" => FileCategory::Image,

            // Дизайн-исходники (Photoshop, Illustrator, Corel)
            "psd" | "psb" | "ai" | "cdr" | "blend" | "fig" => FileCategory::DesignProjects,

            // Проекты видеомонтажа и анимации (Premiere, After Effects, Vegas, DaVinci)
            "prproj" | "aep" | "aepx" | "veg" | "drp" | "sesx" => FileCategory::VideoProjects,

            // Видео
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "m4v" | "3gp" | "3g2"
            | "m2ts" | "mts" | "vob" | "ogv" | "f4v" | "mxf" | "rm" | "rmvb" | "asf" | "divx" => FileCategory::Video,

            // Документы
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "xls" | "xlsx" | "ppt" | "pptx" | "odt"
            | "ods" | "csv" => FileCategory::Document,

            // Аудио
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "opus" | "m4a" | "wma" | "amr" => {
                FileCategory::Audio
            }

            // Архивы
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => FileCategory::Archive,

            // Код и скрипты
            "js" | "ts" | "rs" | "py" | "json" | "xml" | "yaml" | "yml" | "html" | "css"
            | "scss" | "sql" | "sh" | "bat" | "ps1" => FileCategory::Code,

            // Программы для ПК
            "exe" | "msi" | "dmg" | "pkg" | "appimage" | "deb" | "rpm" => FileCategory::Software,

            // Мобильные приложения
            "apk" | "apks" | "xapk" | "ipa" => FileCategory::Mobile,

            // Образы дисков
            "iso" | "img" | "vhd" | "vhdx" | "bin" => FileCategory::DiskImages,

            // Шрифты
            "ttf" | "otf" | "woff" | "woff2" | "eot" => FileCategory::Fonts,

            _ => FileCategory::Other,
        }
    }
}