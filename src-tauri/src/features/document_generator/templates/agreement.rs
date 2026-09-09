use crate::features::document_generator::models::AgreementTemplateData;
use docx_rs::{
    AlignmentType, Docx, LineSpacing, PageMargin, Paragraph, Run, RunFonts,
    SpecialIndentType, Table, TableCell, TableLayoutType, TableRow, WidthType,
};

pub struct AgreementTemplate;

impl AgreementTemplate {
    pub fn build_typst(data: &AgreementTemplateData) -> String {
        let template = include_str!("agreement.typ");

        let customer_reg = data.customer_reg_info.as_deref().filter(|s| !s.trim().is_empty())
            .map(|s| format!(" {}", s)).unwrap_or_default();
        let contractor_reg = data.contractor_reg_info.as_deref().filter(|s| !s.trim().is_empty())
            .map(|s| format!(" {}", s)).unwrap_or_default();

        let preamble = format!(
            "*{}*, в лице {}, действующего на основании {}{}, с одной стороны, и *{}*, в лице {}, действующего на основании {}{}, с другой стороны, {}",
            data.customer_name,
            data.customer_genitive,
            data.customer_basis,
            customer_reg,
            data.contractor_name,
            data.contractor_genitive,
            data.contractor_basis,
            contractor_reg,
            data.preamble_closing
        );

        let body_typst = Self::html_to_typst(&data.body_text);

        let customer_kbe = data.customer_kbe.as_deref().filter(|s| !s.trim().is_empty())
            .map(|k| format!("КБе: {} \\\n", k)).unwrap_or_default();
        let contractor_kbe = data.contractor_kbe.as_deref().filter(|s| !s.trim().is_empty())
            .map(|k| format!("КБе: {} \\\n", k)).unwrap_or_default();

        template
            .replace("[[CITY]]", &data.city)
            .replace("[[DATE]]", &data.date)
            .replace("[[AGREEMENT_NUMBER]]", &data.agreement_number)
            .replace("[[PREAMBLE_BLOCK]]", &preamble)
            .replace("[[BODY_BLOCK]]", &body_typst)
            .replace("[[CUSTOMER_NAME]]", &data.customer_name)
            .replace("[[CUSTOMER_ADDRESS]]", &data.customer_address)
            .replace("[[CUSTOMER_ID_TYPE]]", &data.customer_id_type)
            .replace("[[CUSTOMER_IIN_BIN]]", &data.customer_iin_bin)
            .replace("[[CUSTOMER_KBE_BLOCK]]", &customer_kbe)
            .replace("[[CUSTOMER_IIK]]", &data.customer_iik)
            .replace("[[CUSTOMER_BANK]]", &data.customer_bank)
            .replace("[[CUSTOMER_BIK]]", &data.customer_bik)
            .replace("[[CUSTOMER_POSITION]]", &data.customer_position)
            .replace("[[CUSTOMER_SIGNATORY_NAME]]", &data.customer_signatory_name)
            .replace("[[CONTRACTOR_NAME]]", &data.contractor_name)
            .replace("[[CONTRACTOR_ADDRESS]]", &data.contractor_address)
            .replace("[[CONTRACTOR_ID_TYPE]]", &data.contractor_id_type)
            .replace("[[CONTRACTOR_IIN_BIN]]", &data.contractor_iin_bin)
            .replace("[[CONTRACTOR_KBE_BLOCK]]", &contractor_kbe)
            .replace("[[CONTRACTOR_IIK]]", &data.contractor_iik)
            .replace("[[CONTRACTOR_BANK]]", &data.contractor_bank)
            .replace("[[CONTRACTOR_BIK]]", &data.contractor_bik)
            .replace("[[CONTRACTOR_POSITION]]", &data.contractor_position)
            .replace("[[CONTRACTOR_SIGNATORY_NAME]]", &data.contractor_signatory_name)
    }

    pub fn build_docx(data: &AgreementTemplateData) -> Docx {
        let cm = |value: f32| -> usize { (value * 567.0) as usize };
        let full_width = cm(17.0);
        let col_half = cm(8.3);
        let col_gap = cm(0.4);

        let spacing_body = || LineSpacing::new().line(324);
        let spacing_tight = || LineSpacing::new().line(260);

        let mut docx = Docx::new()
            .page_margin(
                PageMargin::new()
                    .top(cm(2.0) as i32)
                    .bottom(cm(2.0) as i32)
                    .left(cm(2.0) as i32)
                    .right(cm(2.0) as i32),
            )
            .default_fonts(RunFonts::new().ascii("Roboto").hi_ansi("Roboto").cs("Roboto"))
            .default_size(21); // 10.5pt

        // 1. Город и дата
        docx = docx.add_table(
            Table::new(vec![TableRow::new(vec![
                TableCell::new()
                    .width(col_half, WidthType::Dxa)
                    .add_paragraph(
                        Paragraph::new()
                            .align(AlignmentType::Left)
                            .line_spacing(spacing_tight())
                            .add_run(Run::new().add_text(&data.city)),
                    ),
                TableCell::new().width(col_gap, WidthType::Dxa),
                TableCell::new()
                    .width(col_half, WidthType::Dxa)
                    .add_paragraph(
                        Paragraph::new()
                            .align(AlignmentType::Right)
                            .line_spacing(spacing_tight())
                            .add_run(Run::new().add_text(&data.date)),
                    ),
            ])])
                .layout(TableLayoutType::Fixed)
                .width(full_width, WidthType::Dxa)
                .clear_all_border(),
        )
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(180)));

        // 2. Название договора
        docx = docx.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .line_spacing(spacing_tight())
                .add_run(Run::new().bold().size(24).add_text(format!("ДОГОВОР № {}", data.agreement_number))),
        )
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(180)));

        // 3. Преамбула (с красной строкой 0.8 см = 454 dxa)
        let customer_reg = data.customer_reg_info.as_deref().filter(|s| !s.trim().is_empty())
            .map(|s| format!(" {}", s)).unwrap_or_default();
        let contractor_reg = data.contractor_reg_info.as_deref().filter(|s| !s.trim().is_empty())
            .map(|s| format!(" {}", s)).unwrap_or_default();

        docx = docx.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Both)
                .line_spacing(spacing_body())
                .indent(None, Some(SpecialIndentType::FirstLine(454)), None, None)
                .add_run(Run::new().bold().add_text(&data.customer_name))
                .add_run(Run::new().add_text(format!(", в лице {}, действующего на основании {}{}, с одной стороны, и ", data.customer_genitive, data.customer_basis, customer_reg)))
                .add_run(Run::new().bold().add_text(&data.contractor_name))
                .add_run(Run::new().add_text(format!(", в лице {}, действующего на основании {}{}, с другой стороны, {}", data.contractor_genitive, data.contractor_basis, contractor_reg, data.preamble_closing))),
        )
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(140)));

        // 4. Текст тела договора
        for block in Self::parse_html_paragraphs(&data.body_text) {
            let clean_text = Self::strip_tags(&block);

            if clean_text.trim().is_empty() {
                docx = docx.add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(140)));
                continue;
            }

            // Проверка: содержит ли блок strong или b, либо является коротким заголовком раздела
            let has_bold = block.contains("<strong>") || block.contains("<b>");
            let is_section_heading = has_bold && clean_text.trim().len() < 80;

            let mut p = Paragraph::new().line_spacing(spacing_body());

            if is_section_heading {
                // Главный заголовок пункта договора (1. ПРЕДМЕТ ДОГОВОРА)
                p = p.align(AlignmentType::Left)
                    .add_run(Run::new().bold().add_text(clean_text.trim()));
            } else {
                // Обычный абзац или подпункт договора
                p = p.align(AlignmentType::Both);
                let runs = Self::html_to_runs(&block);
                for run in runs {
                    p = p.add_run(run);
                }
            }

            docx = docx.add_paragraph(p);
        }

        // 5. Заголовок реквизитов
        docx = docx
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(240)))
            .add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .line_spacing(spacing_tight())
                    .add_run(Run::new().bold().size(21).add_text("ЮРИДИЧЕСКИЕ АДРЕСА И РЕКВИЗИТЫ СТОРОН")),
            )
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(160)));

        // 6. Колонка Заказчика (9.5pt = 19 half-points)
        let mut c_left = TableCell::new()
            .width(col_half, WidthType::Dxa)
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().bold().size(19).add_text("Заказчик:")))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().bold().size(19).add_text(&data.customer_name)))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("Адрес: {}", data.customer_address))))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("{}: {}", data.customer_id_type, data.customer_iin_bin))));

        if let Some(kbe) = &data.customer_kbe {
            if !kbe.trim().is_empty() {
                c_left = c_left.add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("КБе: {}", kbe))));
            }
        }
        c_left = c_left
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("ИИК: {}", data.customer_iik))))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(&data.customer_bank)))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("БИК: {}", data.customer_bik))));

        // 7. Колонка Подрядчика (9.5pt = 19 half-points)
        let mut c_right = TableCell::new()
            .width(col_half, WidthType::Dxa)
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().bold().size(19).add_text("Подрядчик:")))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().bold().size(19).add_text(&data.contractor_name)))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("Адрес: {}", data.contractor_address))))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("{}: {}", data.contractor_id_type, data.contractor_iin_bin))));

        if let Some(kbe) = &data.contractor_kbe {
            if !kbe.trim().is_empty() {
                c_right = c_right.add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("КБе: {}", kbe))));
            }
        }
        c_right = c_right
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("ИИК: {}", data.contractor_iik))))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(&data.contractor_bank)))
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(format!("БИК: {}", data.contractor_bik))));

        // 8. Подписи
        let sign_left = TableCell::new()
            .width(col_half, WidthType::Dxa)
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(&data.customer_position)))
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(720)))
            .add_paragraph(
                Paragraph::new()
                    .line_spacing(spacing_tight())
                    .add_run(Run::new().size(19).add_text("М.П. ___________________ / "))
                    .add_run(Run::new().size(19).add_text(format!("{} /", data.customer_signatory_name))),
            );

        let sign_right = TableCell::new()
            .width(col_half, WidthType::Dxa)
            .add_paragraph(Paragraph::new().line_spacing(spacing_tight()).add_run(Run::new().size(19).add_text(&data.contractor_position)))
            .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(720)))
            .add_paragraph(
                Paragraph::new()
                    .line_spacing(spacing_tight())
                    .add_run(Run::new().size(19).add_text("М.П. ___________________ / "))
                    .add_run(Run::new().size(19).add_text(format!("{} /", data.contractor_signatory_name))),
            );

        docx.add_table(
            Table::new(vec![
                TableRow::new(vec![c_left, TableCell::new().width(col_gap, WidthType::Dxa), c_right]),
                TableRow::new(vec![
                    TableCell::new().width(col_half, WidthType::Dxa).add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(120))),
                    TableCell::new().width(col_gap, WidthType::Dxa),
                    TableCell::new().width(col_half, WidthType::Dxa).add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().line(120))),
                ]),
                TableRow::new(vec![sign_left, TableCell::new().width(col_gap, WidthType::Dxa), sign_right]),
            ])
                .layout(TableLayoutType::Fixed)
                .width(full_width, WidthType::Dxa)
                .clear_all_border(),
        )
    }

    /// Преобразует HTML-абзац в последовательность Runs с сохранением bold/italic
    fn html_to_runs(html: &str) -> Vec<Run> {
        let mut runs = Vec::new();
        let mut current_text = String::new();
        let mut in_tag = false;
        let mut tag_buffer = String::new();
        let mut is_bold = false;
        let mut is_italic = false;

        let flush_current = |text: &mut String, runs: &mut Vec<Run>, bold: bool, italic: bool| {
            if !text.is_empty() {
                let clean = text
                    .replace("&nbsp;", " ")
                    .replace("&quot;", "\"")
                    .replace("&amp;", "&")
                    .replace("&lt;", "<")
                    .replace("&gt;", ">");
                let mut run = Run::new().add_text(clean);
                if bold {
                    run = run.bold();
                }
                if italic {
                    run = run.italic();
                }
                runs.push(run);
                text.clear();
            }
        };

        for ch in html.chars() {
            if ch == '<' {
                flush_current(&mut current_text, &mut runs, is_bold, is_italic);
                in_tag = true;
                tag_buffer.clear();
            } else if ch == '>' {
                in_tag = false;
                let tag = tag_buffer.to_lowercase();
                if tag == "strong" || tag == "b" {
                    is_bold = true;
                } else if tag == "/strong" || tag == "/b" {
                    is_bold = false;
                } else if tag == "em" || tag == "i" {
                    is_italic = true;
                } else if tag == "/em" || tag == "/i" {
                    is_italic = false;
                }
            } else if in_tag {
                tag_buffer.push(ch);
            } else {
                current_text.push(ch);
            }
        }

        flush_current(&mut current_text, &mut runs, is_bold, is_italic);

        if runs.is_empty() {
            let clean = Self::strip_tags(html);
            if !clean.trim().is_empty() {
                runs.push(Run::new().add_text(clean));
            }
        }

        runs
    }

    fn strip_tags(html: &str) -> String {
        let mut in_tag = false;
        let mut output = String::new();
        for ch in html.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                output.push(ch);
            }
        }
        output
            .replace("&nbsp;", " ")
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }

    fn parse_html_paragraphs(html: &str) -> Vec<String> {
        let mut clean = html.to_string();
        clean = clean.replace("<br>", " ").replace("<br/>", " ").replace("<br />", " ");

        clean
            .split("</p>")
            .map(|s| {
                let s = s.trim();
                let start = s.find('<').unwrap_or(0);
                s[start..].to_string()
            })
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    fn html_to_typst(html: &str) -> String {
        let mut text = html.to_string();

        text = text.replace("<p><br></p>", "\n#v(1.0em)\n");
        text = text.replace("<p>&nbsp;</p>", "\n#v(1.0em)\n");
        text = text.replace("<p></p>", "\n#v(1.0em)\n");
        text = text.replace("<br>", "\n");

        text = text.replace("<strong>", "*");
        text = text.replace("</strong>", "*");
        text = text.replace("<b>", "*");
        text = text.replace("</b>", "*");
        text = text.replace("<em>", "_");
        text = text.replace("</em>", "_");
        text = text.replace("<i>", "_");
        text = text.replace("</i>", "_");

        text = text.replace("<p>", "");
        text = text.replace("</p>", "\n\n");

        Self::strip_tags(&text)
    }
}