use crate::features::document_generator::engines::pdf_engine::PdfEngine;
use crate::shared::utils::font_utils;
use std::path::PathBuf;
use tauri::AppHandle;

pub async fn run_pdf_generation_thread(
    app: &AppHandle,
    typst_content: String,
    save_path: String,
) -> Result<String, String> {
    let app_handle = app.clone();

    // Создаем одноразовый канал для передачи ответа из изолированного потока
    let (tx, rx) = tokio::sync::oneshot::channel();

    // Запускаем выделенный системный поток с размером стека 8 МБ для Typst
    let thread_result = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .name("typst-compiler-thread".into())
        .spawn(move || {
            let result = (|| -> Result<String, String> {
                println!("🟢 [Thread] Поток запущен. Подготовка путей...");
                let target_path = PathBuf::from(&save_path);

                let fonts_root = font_utils::find_fonts_root(&app_handle)
                    .map_err(|e| format!("Ошибка поиска шрифтов: {}", e))?;
                let roboto_dir = fonts_root.join("roboto");

                println!(
                    "📝 [Thread] Шаблон построен. Длина: {} символов",
                    typst_content.len()
                );

                println!("🚀 [Thread] Вызов PdfEngine...");
                PdfEngine::generate_from_template(typst_content, &target_path, &roboto_dir)
                    .map_err(|e| format!("Ошибка рендера PDF: {}", e))?;

                Ok(format!("Документ успешно сохранен: {}", save_path))
            })();

            let _ = tx.send(result);
        });

    if let Err(e) = thread_result {
        return Err(format!("Не удалось создать поток компилятора: {}", e));
    }

    // Ожидаем ответ. Если канал разорван, значит поток снова упал
    rx.await.unwrap_or_else(|_| {
        Err("Поток генерации PDF был аварийно завершен операционной системой".to_string())
    })
}
