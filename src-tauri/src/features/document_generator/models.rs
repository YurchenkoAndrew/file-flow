use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct GenerateDocumentRequest {
    pub template_id: String,
    pub extension: String,
    pub save_path: String,
    pub data: Value, // Сырой JSON, чтобы принять любые поля от любого шаблона
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationTemplateData {
    // Шапка (Кому)
    pub recipient_role: String, // Например: "Директору ТОО FileFlow"
    pub recipient_name: String, // Например: "Иванову И.И."

    // Шапка (От кого)
    pub sender_role: String, // Например: "от системного инженера"
    pub sender_name: String, // Например: "Юрченко А."

    // Центральная часть
    pub title: String, // Обычно "ЗАЯВЛЕНИЕ"

    // Основной текст
    pub body_text: String, // Прошу предоставить мне...

    // Подвал
    pub date: String,        // Дата составления
    pub signer_name: String, // Расшифровка подписи
}

#[derive(Deserialize)]
pub struct MemoTemplateData {
    pub recipient_role: String,
    pub recipient_name: String,
    pub sender_department: Option<String>, // Опционально, так как может быть пустым
    pub sender_role: String,
    pub sender_name: String,
    pub subject: Option<String>, // Опционально
    pub body_text: String,
    pub date: String,
    pub signer_name: String,
}
