use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub fn find_fonts_root(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let resource_dir = app.path().resource_dir().unwrap_or_default();
    let exe_dir = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();

    let candidate_paths = vec![
        resource_dir.join("resources").join("fonts"),
        resource_dir.join("fonts"),
        exe_dir.join("resources").join("fonts"),
        exe_dir.join("..").join("resources").join("fonts"),
        exe_dir.join("..").join("..").join("resources").join("fonts"),
        exe_dir.join("..").join("..").join("src-tauri").join("resources").join("fonts"),
        std::env::current_dir().unwrap_or_default().join("resources").join("fonts"),
        std::env::current_dir().unwrap_or_default().join("src-tauri").join("resources").join("fonts"),
    ];

    for p in &candidate_paths {
        println!("🔍 Проверяем корень шрифтов: {:?}", p);
        if p.exists() && p.is_dir() {
            println!("✅ Корень шрифтов успешно найден в: {:?}", p);
            return Ok(p.clone());
        }
    }

    Err("Корневая папка 'fonts' не найдена ни по одному из путей!".into())
}