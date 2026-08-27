use std::path::Path;
use std::process::Command;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
        let resource_dir = app.path().resource_dir().unwrap_or_default();

        let candidate_paths = vec![
            resource_dir.join("resources").join("ai_models").join("tesseract"),
            resource_dir.join("ai_models").join("tesseract"),
            std::env::current_dir().unwrap_or_default().join("resources").join("ai_models").join("tesseract"),
            std::env::current_dir().unwrap_or_default().join("src-tauri").join("resources").join("ai_models").join("tesseract"),
        ];

        let mut tesseract_dir = None;
        for p in candidate_paths {
            if p.join("tesseract.exe").exists() {
                tesseract_dir = Some(p);
                break;
            }
        }

        let tesseract_dir = tesseract_dir.ok_or_else(|| {
            "tesseract.exe не найден ни по одному из путей поиска!".to_string()
        })?;

        // Очищаем путь от Windows UNC-префикса (\\?\), который ломает Tesseract C++
        let tesseract_dir_str = tesseract_dir.to_string_lossy().replace(r"\\?\", "");
        let clean_tesseract_dir = std::path::PathBuf::from(&tesseract_dir_str);

        let tesseract_exe = clean_tesseract_dir.join("tesseract.exe");
        let tessdata_dir = clean_tesseract_dir.join("tessdata");

        let mut cmd = Command::new(&tesseract_exe);
        cmd.current_dir(&clean_tesseract_dir) // Задаем рабочую директорию, чтобы tesseract видел свои .dll
            .arg(path)
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
}