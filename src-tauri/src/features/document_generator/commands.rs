use crate::features::document_generator::models::GenerateDocumentRequest;
use crate::features::document_generator::service::DocumentService;
use tauri::{command, AppHandle};

#[command]
pub async fn generate_document(
    app: AppHandle,
    request: GenerateDocumentRequest,
) -> Result<String, String> {
    DocumentService::process_generation(&app, request).await
}
