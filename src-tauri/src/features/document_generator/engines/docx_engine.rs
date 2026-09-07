use docx_rs::Docx;
use std::fs::File;
use std::path::Path;

pub struct DocxEngine;

impl DocxEngine {
    pub fn generate_from_template(docx: Docx, target_path: &Path) -> Result<(), String> {
        let file = File::create(target_path)
            .map_err(|e| format!("Ошибка создания DOCX файла: {}", e))?;

        docx.build()
            .pack(file)
            .map_err(|e| format!("Ошибка сборки DOCX архива: {}", e))?;

        Ok(())
    }
}