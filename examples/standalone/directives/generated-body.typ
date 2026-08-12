#heading(level: 1)[#text("Flow-Like Directives")]

#par[#text("Flow-Like directive blocks are converted to static callouts in Typst documents.")]

#block(width: 100%, breakable: true, fill: rgb("#eff6ff"), stroke: 0.8pt + rgb("#93c5fd"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#1d4ed8"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "i")]]], [#text(fill: rgb("#1d4ed8"), weight: "bold", "Info")])#block(above: 0.35em)[#par[#text("This informational note uses its default title and supports ")#strong[#text("Markdown")]#text(".")]]]

#block(width: 100%, breakable: true, fill: rgb("#f5f3ff"), stroke: 0.8pt + rgb("#c4b5fd"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#5b21b6"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "*")]]], [#text(fill: rgb("#5b21b6"), weight: "bold", "Tip: Model Prompting Tip")])#block(above: 0.35em)[#par[#text("Ask the model to use simple tables and explicit chart types when document fidelity matters.")]]]

#block(width: 100%, breakable: true, fill: rgb("#f0fdf4"), stroke: 0.8pt + rgb("#86efac"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#166534"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "+")]]], [#text(fill: rgb("#166534"), weight: "bold", "Success: Validation Complete")])#block(above: 0.35em)[#block(width: 100%)[#list(
[#text("Markdown parsed")],
[#text("Typst generated")],
[#text("PDF compiled")]
)]]]

#block(width: 100%, breakable: true, fill: rgb("#fffbeb"), stroke: 0.8pt + rgb("#fbbf24"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#92400e"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "!")]]], [#text(fill: rgb("#92400e"), weight: "bold", "Warning: Review Required")])#block(above: 0.35em)[#table(
columns: 2,
align: (left, left),
table.header([#text("Check")], [#text("Result")]),
[#text("Brand colors")],
[#text("Pending")],
[#text("Page breaks")],
[#text("Passed")]
)]]

#block(width: 100%, breakable: true, fill: rgb("#fef2f2"), stroke: 0.8pt + rgb("#fca5a5"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#991b1b"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "x")]]], [#text(fill: rgb("#991b1b"), weight: "bold", "Error: Export Failed")])#block(above: 0.35em)[#par[#text("Use this style for errors that must remain visible in a static document.")]]]

#block(width: 100%, breakable: true, fill: rgb("#f8fafc"), stroke: 0.8pt + rgb("#cbd5e1"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#334155"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", ">")]]], [#text(fill: rgb("#334155"), weight: "bold", "Details: Supporting Detail")])#block(above: 0.35em)[#par[#text("Spoilers are rendered as a static ")#strong[#text("Details")]#text(" callout because PDF output is not interactive.")]]]

#heading(level: 2)[#text("Directives inside code stay literal")]

#block(width: 100%)[#raw(":::warning\nThis is example source inside a fenced block, not a live directive.\n:::\n", block: true, lang: "markdown")]

#block(width: 100%, breakable: true, fill: rgb("#eff6ff"), stroke: 0.8pt + rgb("#93c5fd"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#1d4ed8"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "i")]]], [#text(fill: rgb("#1d4ed8"), weight: "bold", "Info: Code Inside a Directive")])#block(above: 0.35em)[#par[#text("The directive preprocessor also ignores delimiter-looking text inside its fenced code:")]

#block(width: 100%)[#raw("::: is data here, not the end of the directive\n", block: true, lang: "text")]

#par[#text("Content after the code block remains inside the callout.")]]]