#heading(level: 1)[#text("Markdown-to-Typst Node: Technical Brief")]

#par[#text("The node converts model-friendly Flow-Like Markdown into a safe Typst body fragment. The user retains ownership of document layout, branding, and global show rules.")]

#heading(level: 2)[#text("Data contract")]

#table(
columns: 4,
align: (left, left, center, left),
table.header([#text("Direction")], [#text("Pin")], [#text("Type")], [#text("Semantics")]),
[#text("Input")],
[#raw("markdown")],
[#text("string")],
[#text("Flow-Like Markdown from a model or upstream node")],
[#text("Output")],
[#raw("typst_source")],
[#text("string")],
[#text("Styleable Typst body fragment")]
)

#heading(level: 2)[#text("Pipeline")]

#block(width: 100%)[#raw("Model -> Markdown to Typst -> Format String -> Typst Source to PDF\n", block: true, lang: "text")]

#par[#text("Generated text is always emitted through Typst string literals. A model response containing ")#raw("#pagebreak()")#text(" therefore displays that text rather than executing it.")]

#heading(level: 2)[#text("Implementation checklist")]

#block(width: 100%)[#list(
marker: [],
[#box(width: 1.35em)[#text("☒")]#text("GFM tables and strikethrough")],
[#box(width: 1.35em)[#text("☒")]#text("Nested ordered, unordered, and task lists")],
[#box(width: 1.35em)[#text("☒")]#text("Flow-Like directives and alerts")],
[#box(width: 1.35em)[#text("☒")]#text("Nivo and Plotly CSV/JSON fences")],
[#box(width: 1.35em)[#text("☒")]#text("Package-free static SVG charts")],
[#box(width: 1.35em)[#text("☒")]#text("URI, recursion, and input-size guards")],
[#box(width: 1.35em)[#text("☐")]#text("Interactive PDF chart tooltips")]
)]

#heading(level: 2)[#text("Compile-time trend")]

#block(width: 100%)[#image(bytes("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"275\" viewBox=\"0 0 800 275\" role=\"img\"><title>Build Duration</title><rect x=\"0.5\" y=\"0.5\" width=\"799\" height=\"274.0\" rx=\"12\" fill=\"#ffffff\" stroke=\"#d8dee9\"/><text x=\"28.00\" y=\"27.00\" font-family=\"Arial, sans-serif\" font-size=\"19.0\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"start\">Build Duration</text><line x1=\"74.0\" y1=\"191.0\" x2=\"770.0\" y2=\"191.0\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"194.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">0</text><line x1=\"74.0\" y1=\"166.2\" x2=\"770.0\" y2=\"166.2\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"169.70\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">10</text><line x1=\"74.0\" y1=\"141.4\" x2=\"770.0\" y2=\"141.4\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"144.90\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">20</text><line x1=\"74.0\" y1=\"116.6\" x2=\"770.0\" y2=\"116.6\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"120.10\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">30</text><line x1=\"74.0\" y1=\"91.8\" x2=\"770.0\" y2=\"91.8\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"95.30\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">40</text><line x1=\"74.0\" y1=\"67.0\" x2=\"770.0\" y2=\"67.0\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"70.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">50</text><line x1=\"74.0\" y1=\"191.0\" x2=\"770.0\" y2=\"191.0\" stroke=\"#aeb8c5\"/><text x=\"422.00\" y=\"229.00\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#475569\" text-anchor=\"middle\">Milestone</text><text x=\"18.0\" y=\"129.0\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#475569\" text-anchor=\"middle\" transform=\"rotate(-90 18.0 129.0)\">Seconds</text><path d=\"M 74.00 106.68 L 306.00 96.76 L 538.00 89.32 L 770.00 94.28\" fill=\"none\" stroke=\"#fb562d\" stroke-width=\"2.2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><circle cx=\"74.00\" cy=\"106.68\" r=\"3.5\" fill=\"#fb562d\" stroke=\"#ffffff\" stroke-width=\"1\"/><circle cx=\"306.00\" cy=\"96.76\" r=\"3.5\" fill=\"#fb562d\" stroke=\"#ffffff\" stroke-width=\"1\"/><circle cx=\"538.00\" cy=\"89.32\" r=\"3.5\" fill=\"#fb562d\" stroke=\"#ffffff\" stroke-width=\"1\"/><circle cx=\"770.00\" cy=\"94.28\" r=\"3.5\" fill=\"#fb562d\" stroke=\"#ffffff\" stroke-width=\"1\"/><path d=\"M 74.00 94.28 L 306.00 86.84 L 538.00 76.92 L 770.00 79.40\" fill=\"none\" stroke=\"#8b61e3\" stroke-width=\"2.2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><circle cx=\"74.00\" cy=\"94.28\" r=\"3.5\" fill=\"#8b61e3\" stroke=\"#ffffff\" stroke-width=\"1\"/><circle cx=\"306.00\" cy=\"86.84\" r=\"3.5\" fill=\"#8b61e3\" stroke=\"#ffffff\" stroke-width=\"1\"/><circle cx=\"538.00\" cy=\"76.92\" r=\"3.5\" fill=\"#8b61e3\" stroke=\"#ffffff\" stroke-width=\"1\"/><circle cx=\"770.00\" cy=\"79.40\" r=\"3.5\" fill=\"#8b61e3\" stroke=\"#ffffff\" stroke-width=\"1\"/><text x=\"74.00\" y=\"208.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">v0.1.0</text><text x=\"306.00\" y=\"208.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">parser</text><text x=\"538.00\" y=\"208.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">charts</text><text x=\"770.00\" y=\"208.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">hardening</text><rect x=\"220.0\" y=\"237.0\" width=\"10\" height=\"10\" rx=\"2\" fill=\"#fb562d\"/><text x=\"235.00\" y=\"246.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#4b5563\" text-anchor=\"start\">Native tests</text><rect x=\"400.0\" y=\"237.0\" width=\"10\" height=\"10\" rx=\"2\" fill=\"#8b61e3\"/><text x=\"415.00\" y=\"246.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#4b5563\" text-anchor=\"start\">Release WASM</text></svg>"), format: "svg", width: 100%, alt: "Build Duration")]

#block(width: 100%, breakable: true, fill: rgb("#f5f3ff"), stroke: 0.8pt + rgb("#c4b5fd"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#5b21b6"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "*")]]], [#text(fill: rgb("#5b21b6"), weight: "bold", "Tip: Operational Guidance")])#block(above: 0.35em)[#par[#text("Use a distinctive placeholder such as ")#raw("{markdown_body}")#text(". The Format String node treats other bare ")#raw("{identifier}")#text(" sequences as additional dynamic inputs.")]]]

#par[#text("The checked-in artifacts are generated with the same Typst version embedded by the node.")#footnote[#text("The example generator disables Universe packages so chart output also proves the package-free path.")]]