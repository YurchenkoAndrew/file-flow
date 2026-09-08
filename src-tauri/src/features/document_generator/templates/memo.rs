use crate::features::document_generator::models::MemoTemplateData;
use docx_rs::{
    AlignmentType, BreakType, Docx, LineSpacing, PageMargin, Paragraph, Run, RunFonts,
    SpecialIndentType, Table, TableCell, TableLayoutType, TableRow, WidthType,
};

pub struct MemoTemplate;

impl MemoTemplate {
    pub fn build_typst(data: &MemoTemplateData) -> String {
        let template = include_str!("memo.typ");
        let formatted_body = data.body_text.replace('\n', " \\ \n");

        // Обработка опционального поля подразделения
        let dept_block = match &data.sender_department {
            Some(dept) if !dept.trim().is_empty() => format!("{}\\\n  ", dept),
            _ => "".to_string(),
        };

        // Обработка опционального поля темы
        let subject_block = match &data.subject {
            Some(subj) if !subj.trim().is_empty() => {
                format!(
                    "#set par(first-line-indent: 0cm)\n*Касательно:* {}\n#v(10pt)\n",
                    subj
                )
            }
            _ => "".to_string(),
        };

        template
            .replace("[[RECIPIENT_ROLE]]", &data.recipient_role)
            .replace("[[RECIPIENT_NAME]]", &data.recipient_name)
            .replace("[[SENDER_DEPARTMENT_BLOCK]]", &dept_block)
            .replace("[[SENDER_ROLE]]", &data.sender_role)
            .replace("[[SENDER_NAME]]", &data.sender_name)
            .replace("[[SUBJECT_BLOCK]]", &subject_block)
            .replace("[[BODY_TEXT]]", &formatted_body)
            .replace("[[DATE]]", &data.date)
            .replace("[[SIGNER_NAME]]", &data.signer_name)
    }

    pub fn build_docx(data: &MemoTemplateData) -> Docx {
        let cm = |value: f32| -> usize { (value * 567.0) as usize };
        let full_width = cm(17.0);
        let header_left = cm(8.5);
        let header_right = cm(8.5);
        let footer_half = cm(8.25);
        let spacing_tight = || LineSpacing::new().line(276);
        let spacing_wide = || LineSpacing::new().line(360);

        let mut docx = Docx::new()
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
            .default_size(24);

        // --- Формирование ячейки шапки ---
        let mut right_cell = TableCell::new()
            .width(header_right, WidthType::Dxa)
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
            .add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Right)
                    .line_spacing(spacing_tight()),
            ); // Пустая строка

        if let Some(dept) = &data.sender_department {
            if !dept.trim().is_empty() {
                right_cell = right_cell.add_paragraph(
                    Paragraph::new()
                        .align(AlignmentType::Right)
                        .line_spacing(spacing_tight())
                        .add_run(Run::new().add_text(dept)),
                );
            }
        }

        right_cell = right_cell
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
            );

        // --- Добавление шапки ---
        docx = docx
            .add_table(
                Table::new(vec![TableRow::new(vec![
                    TableCell::new().width(header_left, WidthType::Dxa),
                    right_cell,
                ])])
                .layout(TableLayoutType::Fixed)
                .width(full_width, WidthType::Dxa)
                .clear_all_border(),
            )
            .add_paragraph(Paragraph::new());

        // --- Заголовок ---
        docx = docx
            .add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(Run::new().bold().size(28).add_text("СЛУЖЕБНАЯ ЗАПИСКА")),
            )
            .add_paragraph(Paragraph::new());

        // --- Тема ---
        if let Some(subj) = &data.subject {
            if !subj.trim().is_empty() {
                docx = docx
                    .add_paragraph(
                        Paragraph::new()
                            .align(AlignmentType::Left)
                            .add_run(Run::new().bold().add_text("Касательно: "))
                            .add_run(Run::new().add_text(subj)),
                    )
                    .add_paragraph(Paragraph::new());
            }
        }

        // --- Основной текст ---
        docx = docx
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
            .add_paragraph(Paragraph::new());

        // --- Подвал ---
        docx.add_table(
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
