use crate::features::document_generator::engines::pdf_engine::PdfEngine;
use crate::features::document_generator::models::{ApplicationTemplateData, GenerateDocumentRequest};
use crate::features::document_generator::templates::application::ApplicationTemplate;
use crate::shared::utils::font_utils;
use std::path::PathBuf;
use tauri::AppHandle;

pub struct DocumentService;

impl DocumentService {

    pub async fn process_generation(
        app: &AppHandle,
        req: GenerateDocumentRequest
    ) -> Result<String, String> {
        match req.template_id.as_str() {
            "application" => {
                // Превращаем универсальный JSON в строго типизированную структуру заявления
                let data: ApplicationTemplateData = serde_json::from_value(req.data)
                    .map_err(|e| format!("Ошибка парсинга полей заявления: {}", e))?;

                Self::generate_application(app, data, req.save_path, req.extension).await
            },
            "memo" => {
                // Когда появится служебная записка:
                // let data: MemoTemplateData = serde_json::from_value(req.data)...
                // Self::generate_memo(app, data, req.save_path, req.extension).await
                Err("Шаблон служебной записки находится в разработке".into())
            },
            _ => Err(format!("Неизвестный идентификатор шаблона: {}", req.template_id))
        }
    }
    pub async fn generate_application(
        app: &AppHandle,
        data: ApplicationTemplateData,
        save_path: String,
        extension: String,
    ) -> Result<String, String> {
        match extension.as_str() {
            "pdf" => {
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

                            println!("📝 [Thread] Формирование Typst шаблона...");
                            let typst_content = ApplicationTemplate::build_typst(&data);
                            println!("📝 [Thread] Шаблон построен. Длина: {} символов", typst_content.len());

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
                rx.await.unwrap_or_else(|_| Err("Поток генерации PDF был аварийно завершен операционной системой".to_string()))
            },
            "docx" => {
                // TODO: Реализовать логику сборки DOCX через соответствующий движок
                // DocxEngine::generate_from_template(&data, &save_path)?;
                Err("Генерация формата DOCX находится в разработке".into())
            },
            _ => {
                Err(format!("Неподдерживаемый формат файла: {}", extension))
            }
        }
    }
}