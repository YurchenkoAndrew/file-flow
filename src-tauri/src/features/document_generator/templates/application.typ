#set page(paper: "a4", margin: (x: 2cm, y: 2.5cm))
#set text(font: "Roboto", size: 12pt)

#align(right)[
  *[[RECIPIENT_ROLE]]* \
  [[RECIPIENT_NAME]] \
  \
  *[[SENDER_ROLE]]* \
  [[SENDER_NAME]]
]

#v(20pt)
#align(center)[*[[TITLE]]*]
#v(10pt)

#set par(first-line-indent: 1.5em, justify: true)
[[BODY_TEXT]]

#v(40pt)
#set par(first-line-indent: 0cm)
#grid(
  columns: (1fr, 1fr),
  [ [[DATE]] ], align(right)[#box(width: 4cm, stroke: (bottom: 1pt)) / [[SIGNER_NAME]] /],
)
