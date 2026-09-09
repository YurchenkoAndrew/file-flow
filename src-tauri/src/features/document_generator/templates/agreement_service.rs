use crate::features::document_generator::engines::docx_engine::DocxEngine;
use crate::features::document_generator::models::AgreementTemplateData;
use crate::features::document_generator::templates::agreement::AgreementTemplate;
use crate::shared::utils::pdf_helper::run_pdf_generation_thread;
use std::path::PathBuf;
use tauri::AppHandle;

pub async fn generate_agreement(
    app: &AppHandle,
    data: AgreementTemplateData,
    save_path: String,
    extension: String,
) -> Result<String, String> {
    match extension.as_str() {
        "pdf" => {
            let typst_content = AgreementTemplate::build_typst(&data);
            run_pdf_generation_thread(app, typst_content, save_path).await
        }
        "docx" => {
            let target_path = PathBuf::from(&save_path);
            let docx_content = AgreementTemplate::build_docx(&data);

            DocxEngine::generate_from_template(docx_content, &target_path)
                .map(|_| format!("Договор успешно сохранен: {}", save_path))
        }
        _ => Err(format!("Неподдерживаемый формат файла: {}", extension)),
    }
}