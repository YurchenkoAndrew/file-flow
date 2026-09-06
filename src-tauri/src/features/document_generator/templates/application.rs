use crate::features::document_generator::models::ApplicationTemplateData;

pub struct ApplicationTemplate;

impl ApplicationTemplate {
    // 1. Переименовали метод под вызов из сервиса
    pub fn build_typst(data: &ApplicationTemplateData) -> String {
        // 2. Подключаем правильный файл шаблона
        let template = include_str!("application.typ");

        // 3. Typst использует \ для переноса строки, а не <br>
        // Добавляем пробел и бэкслеш перед переносом каретки
        let formatted_body = data.body_text.replace('\n', " \\ \n");

        // Заменяем плейсхолдеры на данные
        template
            .replace("[[RECIPIENT_ROLE]]", &data.recipient_role)
            .replace("[[RECIPIENT_NAME]]", &data.recipient_name)
            .replace("[[SENDER_ROLE]]", &data.sender_role)
            .replace("[[SENDER_NAME]]", &data.sender_name)
            .replace("[[TITLE]]", &data.title)
            .replace("[[BODY_TEXT]]", &formatted_body)
            .replace("[[DATE]]", &data.date)
            .replace("[[SIGNER_NAME]]", &data.signer_name)
    }
}