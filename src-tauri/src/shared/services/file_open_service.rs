use std::process::Command;
use std::path::Path;

pub struct FileOpenerService;

impl FileOpenerService {
    pub fn reveal_in_folder(file_path: &str) -> Result<(), String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("Файл не найден на диске".into());
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg("/select,")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Не удалось открыть проводник: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Не удалось открыть Finder: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(parent) = path.parent() {
                Command::new("xdg-open")
                    .arg(parent)
                    .spawn()
                    .map_err(|e| format!("Не удалось открыть файловый менеджер: {}", e))?;
            }
        }

        Ok(())
    }
}