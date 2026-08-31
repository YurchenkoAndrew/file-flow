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
            "doc" => {
                let text = Self::extract_text_from_old_doc(path)?;
                Ok(text)
            }
            "docx" => {
                let text = Self::extract_text_from_docx(path)?;
                Ok(text)
            }
            "html" | "htm" => {
                let raw_html = std::fs::read_to_string(path)?;
                let text = Self::extract_text_from_html(&raw_html);
                Ok(text)
            }
            "xlsx" | "xls" | "xlsb" | "ods" => {
                let text = Self::extract_text_from_excel(path)?;
                Ok(text)
            }
            _ => {
                let text = std::fs::read_to_string(path)?;
                Ok(text)
            }
        }
    }

    // Извлекает текст из любых Excel-таблиц (.xlsx, .xls, .ods)
    fn extract_text_from_excel(
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use calamine::{open_workbook_auto, Data, Reader}; // <-- ИСПРАВЛЕНО: Data вместо DataType

        // Автоматически определяем формат (xls, xlsx, и т.д.)
        let mut workbook = open_workbook_auto(path)?;
        let mut text = String::new();

        // Проходимся по всем листам в таблице
        let sheet_names = workbook.sheet_names().to_owned();
        for sheet in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet) {
                for row in range.rows() {
                    for cell in row {
                        match cell {
                            Data::String(s) => {
                                text.push_str(s);
                                text.push(' ');
                            }
                            Data::Float(f) => {
                                text.push_str(&f.to_string());
                                text.push(' ');
                            }
                            Data::Int(i) => {
                                text.push_str(&i.to_string());
                                text.push(' ');
                            }
                            // Булевы значения и пустые ячейки просто игнорируем
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(text.split_whitespace().collect::<Vec<&str>>().join(" "))
    }

    // Очищает HTML от тегов, стилей и скриптов, оставляя только полезный текст
    fn extract_text_from_html(html: &str) -> String {
        let mut text = String::new();
        let mut in_tag = false;
        let mut skip_content = false; // Флаг для пропуска содержимого <script> и <style>
        let mut tag_name = String::new();

        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '<' {
                in_tag = true;
                tag_name.clear();
                i += 1;

                // Проверяем, закрывающий ли это тег
                let is_closing = if i < chars.len() && chars[i] == '/' {
                    i += 1;
                    true
                } else {
                    false
                };

                // Читаем имя тега
                while i < chars.len() && chars[i].is_ascii_alphanumeric() {
                    tag_name.push(chars[i].to_ascii_lowercase());
                    i += 1;
                }

                // Если это скрипт или стиль, мы должны игнорировать весь текст внутри них
                if tag_name == "script" || tag_name == "style" {
                    skip_content = !is_closing;
                }

                // Дочитываем до конца тега
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
            } else if chars[i] == '>' {
                in_tag = false;
            } else if !in_tag && !skip_content {
                text.push(chars[i]);
            }
            i += 1;
        }

        // Очищаем от HTML-сущностей и множественных пробелов
        text.replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .split_whitespace() // Убирает лишние пробелы и переносы строк
            .collect::<Vec<&str>>()
            .join(" ")
    }

    // Извлекает текст из старых бинарных файлов .doc (OLE2 Compound Document)
    fn extract_text_from_old_doc(
        path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Библиотека сама справляется с бинарным OLE-потоком и кодировками
        let doc = litchi::Document::open(path)?;
        Ok(doc.text()?)
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
