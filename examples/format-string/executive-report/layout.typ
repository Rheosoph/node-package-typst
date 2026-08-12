#set document(title: "Northstar Executive Review")
#set page(
  paper: "a4",
  margin: (top: 24mm, bottom: 20mm, x: 18mm),
  header: context [
    #set text(size: 8pt, fill: rgb("#64748b"))
    #grid(columns: (1fr, auto), [*NORTHSTAR ANALYTICS*], [EXECUTIVE REVIEW / AUGUST 2026])
    #v(3pt)
    #line(length: 100%, stroke: rgb("#cbd5e1"))
  ],
  footer: context [
    #line(length: 100%, stroke: rgb("#e2e8f0"))
    #v(3pt)
    #set text(size: 8pt, fill: rgb("#64748b"))
    #grid(columns: (1fr, auto), [CONFIDENTIAL], [Page #counter(page).display()])
  ],
)
#set text(font: "Libertinus Serif", size: 10.5pt, fill: rgb("#172033"))
#set par(leading: 0.72em, justify: true)
#set heading(numbering: "1.1")
#show heading.where(level: 1): it => block(
  fill: rgb("#eef2ff"),
  stroke: rgb("#c7d2fe"),
  radius: 5pt,
  inset: 10pt,
  above: 16pt,
  below: 8pt,
)[#set text(fill: rgb("#3730a3"), size: 18pt); #it]
#show heading.where(level: 2): set text(fill: rgb("#4338ca"), size: 13pt)
#show quote: it => block(fill: rgb("#f8fafc"), stroke: rgb("#cbd5e1"), radius: 4pt, inset: 10pt)[#it]
#show table: it => block(stroke: rgb("#dbe3ef"), radius: 4pt, inset: 2pt)[#it]
#show raw.where(block: true): it => block(fill: rgb("#0f172a"), radius: 4pt, inset: 10pt)[#set text(fill: white); #it]

{markdown_body}

