use pdfium_render::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Константы для растеризации PDF-страниц под OCR
const PDF_RENDER_TARGET_WIDTH: u32 = 4000;
const PDF_RENDER_MAX_HEIGHT: u32 = 6000;

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

    /// Публичный метод для обычных картинок из файловой системы
    pub fn extract_text_from_image(
        app: &AppHandle,
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Запускаем в простом, быстром режиме
        Self::perform_ocr(app, path, false)
    }

    /// Внутренний метод, который решает, как именно сканировать (просто или с перебором)
    fn perform_ocr(
        app: &AppHandle,
        path: &Path,
        is_complex_pdf: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        let mut _temp_file = None;

        let target_path = if ext == "heic" {
            println!("🔄 Конвертируем HEIC в PNG на лету: {}", path.display());
            let heic_img = heif_oxide::decode_file(path)
                .map_err(|e| format!("Ошибка декодирования HEIC: {}", e))?;
            let img =
                image::RgbaImage::from_raw(heic_img.width, heic_img.height, heic_img.to_rgba8())
                    .ok_or_else(|| "Не удалось собрать буфер изображения HEIC".to_string())?;

            let temp = tempfile::Builder::new()
                .suffix(".png")
                .tempfile()
                .map_err(|e| e.to_string())?;
            img.save_with_format(temp.path(), image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;

            let p = temp.path().to_path_buf();
            _temp_file = Some(temp);
            p
        } else {
            path.to_path_buf()
        };

        let resource_dir = app.path().resource_dir().unwrap_or_default();
        let candidate_paths = vec![
            resource_dir.join("resources").join("ai_models").join("tesseract"),
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

        let raw_canonical = std::fs::canonicalize(&tesseract_dir).unwrap_or_else(|_| tesseract_dir);
        let clean_tesseract_path =
            std::path::PathBuf::from(raw_canonical.to_string_lossy().replace(r"\\?\", ""));

        let tessdata_dir = clean_tesseract_path.join("tessdata");
        let tessdata_dir_arg = tessdata_dir.to_string_lossy().to_string();
        let tessdata_prefix_env = clean_tesseract_path.to_string_lossy().to_string();

        // Если это сложный PDF, применяем выжигание фона. Иначе — просто берем оригинал.
        let processed_path = if is_complex_pdf {
            if let Ok(img) = image::open(&target_path) {
                let mut gray_img = img.grayscale().to_luma8();
                for pixel in gray_img.pixels_mut() {
                    if pixel[0] > 170 {
                        pixel[0] = 255;
                    } else {
                        pixel[0] = 0;
                    }
                }
                if let Ok(temp_gray) = tempfile::Builder::new().suffix(".png").tempfile() {
                    if gray_img.save_with_format(temp_gray.path(), image::ImageFormat::Png).is_ok() {
                        let p = temp_gray.path().to_path_buf();
                        _temp_file = Some(temp_gray);
                        p
                    } else {
                        target_path.clone()
                    }
                } else {
                    target_path.clone()
                }
            } else {
                target_path.clone()
            }
        } else {
            target_path.clone()
        };

        let tesseract_exe = clean_tesseract_path.join("tesseract.exe");

        // Для PDF перебираем все режимы. Для обычных картинок — только стандартный быстрый режим 3.
        let psm_modes = if is_complex_pdf {
            vec!["3", "4", "11", "6", "1"]
        } else {
            vec!["3"]
        };

        let mut best_text = String::new();

        for psm in psm_modes {
            if is_complex_pdf {
                println!("🔄 Пробуем OCR с режимом PSM: {}", psm);
            }

            let mut cmd = Command::new(&tesseract_exe);
            cmd.current_dir(&clean_tesseract_path)
                .arg(&processed_path)
                .arg("stdout")
                .arg("-l")
                .arg("rus+eng+kaz")
                .arg("--dpi")
                .arg("300")
                .arg("--psm")
                .arg(psm)
                .arg("--tessdata-dir")
                .arg(&tessdata_dir_arg)
                .env("TESSDATA_PREFIX", &tessdata_prefix_env);

            #[cfg(target_os = "windows")]
            cmd.creation_flags(0x08000000);

            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let err_log = String::from_utf8_lossy(&output.stderr).trim().to_string();

                    if is_complex_pdf && !err_log.is_empty() && !err_log.contains("Too few characters") {
                        println!("⚠️ [TESSERACT STDERR PSM {}]: {}", psm, err_log);
                    }

                    let valid_chars_count = text.chars().filter(|c| c.is_alphanumeric()).count();

                    // Если нашли текст, прерываем цикл
                    if valid_chars_count > 15 {
                        if is_complex_pdf {
                            println!(
                                "✅ Успешное распознавание на PSM {}. Полезных букв/цифр: {}",
                                psm, valid_chars_count
                            );
                        }
                        return Ok(text);
                    } else {
                        let best_valid_count = best_text.chars().filter(|c| c.is_alphanumeric()).count();
                        if valid_chars_count > best_valid_count {
                            best_text = text;
                        }
                    }
                }
            }
        }

        let best_valid_count = best_text.chars().filter(|c| c.is_alphanumeric()).count();

        if is_complex_pdf {
            println!("⚠️ Перебор PSM завершен. Ни один режим не нашел достаточно текста. Лучший результат: {} полезных символов.", best_valid_count);
        }

        if best_valid_count < 15 {
            return Ok(String::new());
        }

        Ok(best_text)
    }

    /// Резервный метод: рендерит "глухие" PDF в картинки и прогоняет через Tesseract с полным перебором
    pub fn extract_text_from_pdf(
        app: &AppHandle,
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use tempfile::Builder;

        let mut engine_guard = PDFIUM_ENGINE.lock().unwrap();

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
                exe_dir.join("..").join("..").join("resources").join("pdfium"),
                exe_dir.join("..").join("..").join("src-tauri").join("resources").join("pdfium"),
                std::env::current_dir().unwrap_or_default().join("resources").join("pdfium"),
                std::env::current_dir().unwrap_or_default().join("src-tauri").join("resources").join("pdfium"),
            ];

            let mut pdfium_bind = None;

            for p in &candidate_paths {
                let lib_path = Pdfium::pdfium_platform_library_name_at_path(p.to_string_lossy().as_ref());
                if Path::new(&lib_path).exists() {
                    if let Ok(bind) = Pdfium::bind_to_library(&lib_path) {
                        pdfium_bind = Some(bind);
                        break;
                    }
                }
            }

            let bind = pdfium_bind
                .or_else(|| Pdfium::bind_to_system_library().ok())
                .ok_or_else(|| "Движок pdfium не найден ни в ресурсах, ни в системе!".to_string())?;

            *engine_guard = Some(Pdfium::new(bind));
            println!("🚀 Движок Pdfium успешно инициализирован и сохранен в памяти.");
        }

        let pdfium = engine_guard.as_ref().unwrap();

        let document = pdfium
            .load_pdf_from_file(path, None)
            .map_err(|e| e.to_string())?;

        let mut full_text = String::new();

        let render_config = PdfRenderConfig::new()
            .set_target_width(PDF_RENDER_TARGET_WIDTH as Pixels)
            .set_maximum_height(PDF_RENDER_MAX_HEIGHT as Pixels);

        for (index, page) in document.pages().iter().enumerate() {
            let bmp = page
                .render_with_config(&render_config)
                .map_err(|e| format!("Ошибка рендеринга страницы PDF: {}", e))?;

            let image = bmp.as_image().map_err(|e| e.to_string())?;

            let temp_file = Builder::new()
                .suffix(".png")
                .tempfile()
                .map_err(|e| e.to_string())?;
            let temp_path = temp_file.path();

            image
                .save_with_format(temp_path, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;

            // Вызываем с флагом true (включаем фильтры и перебор PSM)
            match Self::perform_ocr(app, temp_path, true) {
                Ok(text) => {
                    if !text.is_empty() {
                        println!("👀 [ТЕСТ TESSERACT] Распознанный текст со страницы {}:\n{}", index, text);
                        full_text.push_str(&text);
                        full_text.push_str("\n\n");
                    }
                }
                Err(e) => println!("❌ Ошибка OCR на странице {}: {}", index, e),
            }
        }

        Ok(full_text.trim().to_string())
    }
}