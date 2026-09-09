#set page(paper: "a4", margin: (x: 2cm, top: 2cm, bottom: 2cm))
#set text(font: "Roboto", size: 10.5pt)

#grid(
  columns: (1fr, 1fr),
  align(left)[*[[CITY]]*],
  align(right)[*[[DATE]]*],
)

#v(8pt)
#align(center)[#text(size: 12pt)[*ДОГОВОР № [[AGREEMENT_NUMBER]]*]]
#v(10pt)

// Преамбула с аккуратным отступом
#set par(justify: true, first-line-indent: 0.8cm, leading: 0.55em)
[[PREAMBLE_BLOCK]]

#v(8pt)
// Основные пункты договора: без сдвига вправо, чтобы цифры 1.1 не уплывали
#set par(justify: true, first-line-indent: 0cm, leading: 0.55em)
[[BODY_BLOCK]]

#v(12pt)
#block(breakable: false)[
  #set par(justify: false, first-line-indent: 0cm)
  #align(center)[*ЮРИДИЧЕСКИЕ АДРЕСА И РЕКВИЗИТЫ СТОРОН*]
  #v(8pt)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 20pt,
    [
      *Заказчик:* \
      *[[CUSTOMER_NAME]]* \
      Адрес: [[CUSTOMER_ADDRESS]] \
      [[CUSTOMER_ID_TYPE]]: [[CUSTOMER_IIN_BIN]] \
      [[CUSTOMER_KBE_BLOCK]]
      ИИК: [[CUSTOMER_IIK]] \
      [[CUSTOMER_BANK]] \
      БИК: [[CUSTOMER_BIK]]
    ],
    [
      *Подрядчик:* \
      *[[CONTRACTOR_NAME]]* \
      Адрес: [[CONTRACTOR_ADDRESS]] \
      [[CONTRACTOR_ID_TYPE]]: [[CONTRACTOR_IIN_BIN]] \
      [[CONTRACTOR_KBE_BLOCK]]
      ИИК: [[CONTRACTOR_IIK]] \
      [[CONTRACTOR_BANK]] \
      БИК: [[CONTRACTOR_BIK]]
    ]
  )

  #v(14pt)
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 20pt,
    [
      [[CUSTOMER_POSITION]] \ \
      #v(28pt)
      #grid(
        columns: (1fr, 1fr),
        [М.П. #box(width: 2.5cm, stroke: (bottom: 0.5pt))],
        align(right)[\/ [[CUSTOMER_SIGNATORY_NAME]] \/]
      )
    ],
    [
      [[CONTRACTOR_POSITION]] \ \
      #v(28pt)
      #grid(
        columns: (1fr, 1fr),
        [М.П. #box(width: 2.5cm, stroke: (bottom: 0.5pt))],
        align(right)[\/ [[CONTRACTOR_SIGNATORY_NAME]] \/]
      )
    ]
  )
]