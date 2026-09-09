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

#[derive(Debug, Serialize, Deserialize)]
pub struct AgreementTemplateData {
    pub agreement_number: String,
    pub city: String,
    pub date: String,

    // Заказчик (Сторона 1)
    pub customer_name: String,
    pub customer_position: String,
    pub customer_signatory_name: String,
    pub customer_genitive: String,
    pub customer_basis: String,
    pub customer_reg_info: Option<String>,
    pub customer_address: String,
    pub customer_id_type: String, // "БИН" или "ИИН"
    pub customer_iin_bin: String,
    pub customer_kbe: Option<String>,
    pub customer_iik: String,
    pub customer_bank: String,
    pub customer_bik: String,

    // Подрядчик (Сторона 2)
    pub contractor_name: String,
    pub contractor_position: String,
    pub contractor_signatory_name: String,
    pub contractor_genitive: String,
    pub contractor_basis: String,
    pub contractor_reg_info: Option<String>,
    pub contractor_address: String,
    pub contractor_id_type: String, // "БИН" или "ИИН"
    pub contractor_iin_bin: String,
    pub contractor_kbe: Option<String>,
    pub contractor_iik: String,
    pub contractor_bank: String,
    pub contractor_bik: String,

    pub preamble_closing: String,
    pub body_text: String,
}
