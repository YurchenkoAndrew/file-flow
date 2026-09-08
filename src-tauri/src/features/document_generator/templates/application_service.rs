use std::path::PathBuf;
use tauri::AppHandle;
use crate::features::document_generator::engines::docx_engine::DocxEngine;
use crate::features::document_generator::models::ApplicationTemplateData;
use crate::features::document_generator::templates::application::ApplicationTemplate;
use crate::shared::utils::pdf_helper::run_pdf_generation_thread;

pub async fn generate_application(
    app: &AppHandle,
    data: ApplicationTemplateData,
    save_path: String,
    extension: String,
) -> Result<String, String> {
    match extension.as_str() {
        "pdf" => {
            let typst_content = ApplicationTemplate::build_typst(&data);
            run_pdf_generation_thread(app, typst_content, save_path).await
        },
        "docx" => {
            let target_path = PathBuf::from(&save_path);
            let docx_content = ApplicationTemplate::build_docx(&data);

            DocxEngine::generate_from_template(docx_content, &target_path)
                .map(|_| format!("Документ успешно сохранен: {}", save_path))
        },
        _ => Err(format!("Неподдерживаемый формат файла: {}", extension))
    }
}