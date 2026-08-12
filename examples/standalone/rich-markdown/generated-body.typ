#heading(level: 1)[#text("Rich Markdown Structures")]

#heading(level: 2)[#text("Lists and tasks")]

#block(width: 100%)[#list(
marker: [],
[#box(width: 1.35em)[#text("•")]#text("Product launch")#linebreak()#block(width: 100%)[#enum(
start: 1,
[#text("Finalize the narrative")],
[#text("Validate the generated PDF")#linebreak()#block(width: 100%)[#list(
[#text("Check tables")],
[#text("Check charts")]
)]]
)]],
[#box(width: 1.35em)[#text("☒")]#text("Markdown converted")],
[#box(width: 1.35em)[#text("☒")]#text("Typst compiled")],
[#box(width: 1.35em)[#text("☐")]#text("Stakeholder approval")]
)]

#block(width: 100%)[#enum(
start: 5,
[#text("Ordered lists can start at a non-default number")],
[#text("Their numbering remains semantic")]
)]

#heading(level: 2)[#text("Aligned table")]

#table(
columns: 4,
align: (left, center, right, left),
table.header([#text("Workstream")], [#text("Owner")], [#text("Confidence")], [#text("Status")]),
[#text("Parser")],
[#text("Ada")],
[#text("96%")],
[#strong[#text("Ready")]],
[#text("Charts")],
[#text("Lin")],
[#text("91%")],
[#emph[#text("Review")]],
[#text("Templates")],
[#text("Noor")],
[#text("88%")],
[#raw("testing")]
)

#heading(level: 2)[#text("Quotes and alerts")]

#quote(block: true)[#par[#text("A normal blockquote can span lines.")#linebreak()#text("It keeps the author's words together.")]#quote(block: true)[#par[#text("Nested quotes remain nested content.")]]]

#block(width: 100%, breakable: true, fill: rgb("#fffbeb"), stroke: 0.8pt + rgb("#fbbf24"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#92400e"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "!")]]], [#text(fill: rgb("#92400e"), weight: "bold", "Important")])#block(above: 0.35em)[#par[#text("Generated content inherits the surrounding Typst document's show and set rules.")]]]

#heading(level: 2)[#text("Definitions and footnotes")]

#block(above: 0.35em, below: 0.35em)[#strong[#text("Body fragment")]#block(inset: (left: 1em), above: 0.15em)[#text("Typst markup generated from Markdown and inserted into a user-owned document.")]]
#block(above: 0.35em, below: 0.35em)[#strong[#text("Static chart")]#block(inset: (left: 1em), above: 0.15em)[#text("A package-free SVG representation suitable for PDF and SVG export.")]]

#par[#text("The conversion behavior is documented alongside the node.")#footnote[#text("Footnotes can contain ")#strong[#text("formatted evidence")]#text(" and links such as ")#link("https://github.com/flow-like/flow-like")[#text("the repository")]#text(".")]]