use std::io::Read;
use std::path::Path;

pub struct DocumentParser;

impl DocumentParser {
    pub fn extract_text(
        path: &Path,
        ext: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match ext.to_lowercase().as_str() {
            "pdf" => {
                let text = pdf_extract::extract_text(path)?;
                Ok(text)
            }
            "docx" => {
                let text = Self::extract_text_from_docx(path)?;
                Ok(text)
            }
            _ => {
                // Для обычных текстовых форматов (txt, md, json, csv и т.д.)
                let text = std::fs::read_to_string(path)?;
                Ok(text)
            }
        }
    }

    // Простой и быстрый парсер .docx без тяжелых зависимостей
    fn extract_text_from_docx(
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut doc_file = archive.by_name("word/document.xml")?;
        let mut xml_content = String::new();
        doc_file.read_to_string(&mut xml_content)?;

        let mut text = String::new();
        let mut inside_tag = false;

        for c in xml_content.chars() {
            if c == '<' {
                inside_tag = true;
            } else if c == '>' {
                inside_tag = false;
            } else if !inside_tag {
                text.push(c);
            }
        }
        Ok(text)
    }
}
