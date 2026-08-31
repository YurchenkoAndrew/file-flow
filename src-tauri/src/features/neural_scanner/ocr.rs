use pdfium_render::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex; // ДОБАВЛЕНО для глобального хранилища
use tauri::{AppHandle, Manager}; // ДОБАВЛЕНО в корень файла

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Глобальное хранилище для движка: загружается ровно 1 раз
static PDFIUM_ENGINE: Mutex<Option<Pdfium>> = Mutex::new(None);
pub struct ImageOcr;

impl ImageOcr {
    pub fn is_image(extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "heic" | "tiff" | "tif" | "bmp" | "webp"
        )
    }

    pub fn extract_text_from_image(
        app: &AppHandle,
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        // Создаем "якорь" для временного файла, чтобы он не удалился до завершения OCR
        let mut _temp_file = None;

        let target_path = if ext == "heic" {
            println!("🔄 Конвертируем HEIC в PNG на лету: {}", path.display());

            // Распаковываем HEIC с помощью чистого Rust-декодера
            let heic_img = heif_oxide::decode_file(path)
                .map_err(|e| format!("Ошибка декодирования HEIC: {}", e))?;

            // Превращаем сырые пиксели в формат изображения (ширина и высота)
            let img =
                image::RgbaImage::from_raw(heic_img.width, heic_img.height, heic_img.to_rgba8())
                    .ok_or_else(|| "Не удалось собрать буфер изображения HEIC".to_string())?;

            // Сохраняем во временный PNG
            let temp = tempfile::Builder::new()
                .suffix(".png")
                .tempfile()
                .map_err(|e| e.to_string())?;
            img.save_with_format(temp.path(), image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;

            let p = temp.path().to_path_buf();
            _temp_file = Some(temp); // Сохраняем объект в переменной, продлевая ему жизнь
            p
        } else {
            // Для JPG, PNG и других форматов оставляем оригинальный путь
            path.to_path_buf()
        };

        let resource_dir = app.path().resource_dir().unwrap_or_default();

        let candidate_paths = vec![
            resource_dir
                .join("resources")
                .join("ai_models")
                .join("tesseract"),
            resource_dir.join("ai_models").join("tesseract"),
            std::env::current_dir()
                .unwrap_or_default()
                .join("resources")
                .join("ai_models")
                .join("tesseract"),
            std::env::current_dir()
                .unwrap_or_default()
                .join("src-tauri")
                .join("resources")
                .join("ai_models")
                .join("tesseract"),
        ];

        let mut tesseract_dir = None;
        for p in candidate_paths {
            if p.join("tesseract.exe").exists() {
                tesseract_dir = Some(p);
                break;
            }
        }

        let tesseract_dir = tesseract_dir
            .ok_or_else(|| "tesseract.exe не найден ни по одному из путей поиска!".to_string())?;

        let tesseract_dir_str = tesseract_dir.to_string_lossy().replace(r"\\?\", "");
        let clean_tesseract_dir = std::path::PathBuf::from(&tesseract_dir_str);

        let tesseract_exe = clean_tesseract_dir.join("tesseract.exe");
        let tessdata_dir = clean_tesseract_dir.join("tessdata");

        let mut cmd = Command::new(&tesseract_exe);
        cmd.current_dir(&clean_tesseract_dir)
            .arg(&target_path) // <-- ВАЖНО: передаем target_path, а не оригинальный path
            .arg("stdout")
            .arg("-l")
            .arg("rus+eng+kaz")
            .arg("--tessdata-dir")
            .arg(&tessdata_dir)
            .env("TESSDATA_PREFIX", &tessdata_dir);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let output = cmd.output()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(text)
        } else {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("Tesseract OCR error: {}", err).into())
        }
    }

    /// Резервный метод: рендерит "глухие" PDF в картинки и прогоняет через Tesseract
    pub fn extract_text_from_pdf(
        app: &AppHandle,
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use tempfile::Builder;

        // 1. Блокируем глобальное хранилище на время обработки
        let mut engine_guard = PDFIUM_ENGINE.lock().unwrap();

        // 2. Если движок еще ни разу не загружался — инициализируем его
        if engine_guard.is_none() {
            let resource_dir = app.path().resource_dir().unwrap_or_default();
            let exe_dir = std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(Path::new(""))
                .to_path_buf();

            let candidate_paths = vec![
                resource_dir.join("resources").join("pdfium"),
                resource_dir.join("pdfium"),
                exe_dir
                    .join("..")
                    .join("..")
                    .join("resources")
                    .join("pdfium"),
                exe_dir
                    .join("..")
                    .join("..")
                    .join("src-tauri")
                    .join("resources")
                    .join("pdfium"),
                std::env::current_dir()
                    .unwrap_or_default()
                    .join("resources")
                    .join("pdfium"),
                std::env::current_dir()
                    .unwrap_or_default()
                    .join("src-tauri")
                    .join("resources")
                    .join("pdfium"),
            ];

            let mut pdfium_bind = None;

            for p in &candidate_paths {
                let lib_path =
                    Pdfium::pdfium_platform_library_name_at_path(p.to_string_lossy().as_ref());
                if Path::new(&lib_path).exists() {
                    if let Ok(bind) = Pdfium::bind_to_library(&lib_path) {
                        pdfium_bind = Some(bind);
                        break;
                    }
                }
            }

            let bind = pdfium_bind
                .or_else(|| Pdfium::bind_to_system_library().ok())
                .ok_or_else(|| {
                    "Движок pdfium не найден ни в ресурсах, ни в системе!".to_string()
                })?;

            // Сохраняем загруженный движок навсегда!
            *engine_guard = Some(Pdfium::new(bind));
            println!("🚀 Движок Pdfium успешно инициализирован и сохранен в памяти.");
        }

        // 3. Берем ссылку на уже готовый движок
        let pdfium = engine_guard.as_ref().unwrap();

        // 4. Дальше парсим PDF как обычно
        let document = pdfium
            .load_pdf_from_file(path, None)
            .map_err(|e| e.to_string())?;

        let mut full_text = String::new();
        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(3000);

        for (index, page) in document.pages().iter().enumerate() {
            let bmp = page
                .render_with_config(&render_config)
                .map_err(|e| e.to_string())?;
            let image = bmp.as_image().map_err(|e| e.to_string())?;

            let temp_file = Builder::new()
                .suffix(".png")
                .tempfile()
                .map_err(|e| e.to_string())?;
            let temp_path = temp_file.path();

            image
                .save_with_format(temp_path, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;

            match Self::extract_text_from_image(app, temp_path) {
                Ok(text) => {
                    full_text.push_str(&text);
                    full_text.push_str("\n\n");
                }
                Err(e) => println!("Ошибка OCR на странице {}: {}", index, e),
            }
        }

        Ok(full_text.trim().to_string())
    }
}
