use crate::features::document_generator::models::ApplicationTemplateData;
use docx_rs::{
    AlignmentType, BreakType, Docx, LineSpacing, PageMargin, Paragraph, Run, RunFonts,
    SpecialIndentType, Table, TableCell, TableLayoutType, TableRow, WidthType,
};

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
    pub fn build_docx(data: &ApplicationTemplateData) -> Docx {
        let cm = |value: f32| -> usize { (value * 567.0) as usize };

        let full_width = cm(17.0);
        let header_left = cm(8.5);
        let header_right = cm(8.5);
        let footer_half = cm(8.25);

        let spacing_tight = || LineSpacing::new().line(276);
        let spacing_wide = || LineSpacing::new().line(360);

        Docx::new()
            .page_margin(
                PageMargin::new()
                    .top(cm(2.0) as i32)
                    .bottom(cm(2.0) as i32)
                    .left(cm(2.0) as i32)
                    .right(cm(2.0) as i32),
            )
            .default_fonts(
                RunFonts::new()
                    .ascii("Roboto")
                    .hi_ansi("Roboto")
                    .cs("Roboto"),
            )
            .default_size(24) // 12pt, чтобы получить значение в pt- нужно разделить на 2. Или само значение, чтоб получить - нужно 12pt * 2 = 24
            // --- Шапка ---
            .add_table(
                Table::new(vec![TableRow::new(vec![
                    TableCell::new().width(header_left, WidthType::Dxa),
                    TableCell::new()
                        .width(header_right, WidthType::Dxa)
                        // Выравнивание по правому краю и Кому
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Right)
                                .line_spacing(spacing_tight())
                                .add_run(Run::new().bold().add_text(&data.recipient_role)),
                        )
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Right)
                                .line_spacing(spacing_tight())
                                .add_run(Run::new().add_text(&data.recipient_name)),
                        )
                        // Пустая строка (пробел) между блоками
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Right)
                                .line_spacing(spacing_tight()),
                        )
                        // От кого
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Right)
                                .line_spacing(spacing_tight())
                                .add_run(Run::new().bold().add_text(&data.sender_role)),
                        )
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Right)
                                .line_spacing(spacing_tight())
                                .add_run(Run::new().add_text(&data.sender_name)),
                        ),
                ])])
                .layout(TableLayoutType::Fixed)
                .width(full_width, WidthType::Dxa)
                .clear_all_border(),
            )
            .add_paragraph(Paragraph::new())
            // --- Заголовок ---
            .add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(Run::new().bold().size(28).add_text(&data.title)),
            )
            .add_paragraph(Paragraph::new())
            // --- Основной текст ---
            .add_paragraph({
                let mut p = Paragraph::new()
                    .align(AlignmentType::Both)
                    .line_spacing(spacing_wide())
                    .indent(None, Some(SpecialIndentType::FirstLine(360)), None, None);
                for (i, line) in data.body_text.split('\n').enumerate() {
                    if i > 0 {
                        p = p.add_run(Run::new().add_break(BreakType::TextWrapping));
                    }
                    p = p.add_run(Run::new().add_text(line));
                }
                p
            })
            .add_paragraph(Paragraph::new())
            .add_paragraph(Paragraph::new())
            // --- Подвал ---
            .add_table(
                Table::new(vec![TableRow::new(vec![
                    TableCell::new()
                        .width(footer_half, WidthType::Dxa)
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Left)
                                .line_spacing(spacing_tight())
                                .add_run(Run::new().add_text(&data.date)),
                        ),
                    TableCell::new()
                        .width(footer_half, WidthType::Dxa)
                        .add_paragraph(
                            Paragraph::new()
                                .align(AlignmentType::Right)
                                .line_spacing(spacing_tight())
                                // Добавили линию для подписи перед расшифровкой
                                .add_run(Run::new().add_text("_________________ "))
                                .add_run(Run::new().add_text(format!("/ {} /", data.signer_name))),
                        ),
                ])])
                .layout(TableLayoutType::Fixed)
                .width(full_width, WidthType::Dxa)
                .clear_all_border(),
            )
    }
}
