use pdfium_render::prelude::*;
use std::path::Path;
use tauri::{AppHandle, Manager};

pub fn init_pdfium(app: &AppHandle) -> Result<Pdfium, Box<dyn std::error::Error + Send + Sync>> {
    let resource_dir = app.path().resource_dir().unwrap_or_default();
    let exe_dir = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();

    let candidate_paths = vec![
        resource_dir.join("resources").join("pdfium"),
        resource_dir.join("pdfium"),
        exe_dir.join("resources").join("pdfium"), // <-- Добавили прямой путь рядом с exe
        exe_dir.join("..").join("resources").join("pdfium"),
        exe_dir.join("..").join("..").join("resources").join("pdfium"),
        exe_dir.join("..").join("..").join("src-tauri").join("resources").join("pdfium"),
        std::env::current_dir().unwrap_or_default().join("resources").join("pdfium"),
        std::env::current_dir().unwrap_or_default().join("src-tauri").join("resources").join("pdfium"),
    ];

    let mut pdfium_bind = None;

    for p in &candidate_paths {
        let lib_path = Pdfium::pdfium_platform_library_name_at_path(p.to_string_lossy().as_ref());
        println!("🔍 Проверяем путь к Pdfium: {:?}", lib_path); // <-- Выведем в консоль для отладки
        if Path::new(&lib_path).exists() {
            if let Ok(bind) = Pdfium::bind_to_library(&lib_path) {
                println!("✅ Pdfium успешно загружен из: {:?}", lib_path);
                pdfium_bind = Some(bind);
                break;
            }
        }
    }

    let bind = pdfium_bind
        .or_else(|| {
            println!("⚠️ Пробуем подключить системный Pdfium...");
            Pdfium::bind_to_system_library().ok()
        })
        .ok_or_else(|| "Движок pdfium не найден ни в ресурсах, ни в системе!".to_string())?;

    Ok(Pdfium::new(bind))
}