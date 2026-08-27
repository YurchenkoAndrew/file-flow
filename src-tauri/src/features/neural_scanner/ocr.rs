use std::path::Path;
use image::GenericImageView;

pub struct ImageOcr;

impl ImageOcr {
    /// Проверяем, относится ли файл к поддерживаемым изображениям
    pub fn is_image(extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "heic" | "tiff" | "tif" | "bmp" | "webp"
        )
    }

    /// Извлечение текста из изображения для бухгалтерских сканов и чеков
    pub fn extract_text_from_image(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        // Открываем изображение с помощью крейта image
        let img = image::open(path)?;
        let (width, height) = img.dimensions();

        // Базовая предварительная проверка: если файл слишком мелкий или битый
        if width == 0 || height == 0 {
            return Ok(String::new());
        }

        // Предобработка: перевод в оттенки серого (grayscale)
        // Это стандартный шаг для OCR, который повышает точность распознавания текста на чеках
        let _gray_img = img.grayscale();

        // Заглушка-интеграция под локальный OCR (например, tesseract-rs или встроенные бинарники)
        // В продакшн-контуре здесь вызывается движок распознавания символов.
        // Пока возвращаем метаданные изображения и пустую строку под будущий текст,
        // чтобы конвейер векторизации не ломался на пустых сканах.
        let placeholder_text = format!(
            "Изображение скана: [путь: {:?}], разрешение: {}x{}",
            path.file_name().unwrap_or_default(),
            width,
            height
        );

        Ok(placeholder_text)
    }
}