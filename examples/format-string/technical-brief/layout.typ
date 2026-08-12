#set document(title: "Flow-Like Technical Brief")
#set page(
  paper: "a4",
  margin: (top: 22mm, bottom: 18mm, x: 17mm),
  header: context [
    #set text(size: 8pt, fill: rgb("#64748b"))
    #grid(columns: (1fr, auto), [FLOW-LIKE / ENGINEERING], [`typst_markdown_to_typst`])
  ],
  footer: context [
    #align(center)[#set text(size: 8pt, fill: rgb("#94a3b8")); #counter(page).display()]
  ],
)
#set text(font: "Libertinus Serif", size: 10pt, fill: rgb("#1e293b"))
#set par(leading: 0.68em)
#set heading(numbering: "1.1")
#show heading.where(level: 1): it => block(fill: rgb("#ecfeff"), stroke: rgb("#a5f3fc"), radius: 3pt, inset: 9pt)[#set text(fill: rgb("#155e75")); #it]
#show heading.where(level: 2): set text(fill: rgb("#0e7490"))
#show raw.where(block: true): it => block(fill: rgb("#111827"), radius: 4pt, inset: 10pt)[#set text(fill: rgb("#e5e7eb"), size: 8.5pt); #it]
#show quote: it => block(fill: rgb("#f1f5f9"), stroke: rgb("#cbd5e1"), inset: 8pt)[#it]
#show table: it => block(stroke: rgb("#cbd5e1"), inset: 1pt)[#it]

{markdown_body}

