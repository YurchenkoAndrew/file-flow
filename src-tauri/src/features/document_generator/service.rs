use crate::features::document_generator::models::{AgreementTemplateData, ApplicationTemplateData, GenerateDocumentRequest, MemoTemplateData};
use crate::features::document_generator::templates::agreement_service::generate_agreement;
use crate::features::document_generator::templates::application_service::generate_application;
use crate::features::document_generator::templates::memo_service::generate_memo;
use tauri::AppHandle;

pub struct DocumentService;

impl DocumentService {
    pub async fn process_generation(
        app: &AppHandle,
        req: GenerateDocumentRequest,
    ) -> Result<String, String> {
        match req.template_id.as_str() {
            "agreement" => {
                let data: AgreementTemplateData = serde_json::from_value(req.data)
                    .map_err(|e| format!("Ошибка парсинга полей договора: {}", e))?;
                generate_agreement(app, data, req.save_path, req.extension).await
            }
            "application" => {
                // Превращаем универсальный JSON в строго типизированную структуру заявления
                let data: ApplicationTemplateData = serde_json::from_value(req.data)
                    .map_err(|e| format!("Ошибка парсинга полей заявления: {}", e))?;
                generate_application(app, data, req.save_path, req.extension).await
            }
            "memo" => {
                let data: MemoTemplateData = serde_json::from_value(req.data)
                    .map_err(|e| format!("Ошибка парсинга полей служебной записки: {}", e))?;
                generate_memo(app, data, req.save_path, req.extension).await
            }
            _ => Err(format!(
                "Неизвестный идентификатор шаблона: {}",
                req.template_id
            )),
        }
    }
}
