#heading(level: 1)[#text("Atlas: model Markdown to finished documents")]

#par[#text("Give your model a format it already understands. Atlas converts Flow-Like Markdown into safe, styleable Typst and lets your design system do the rest.")]

#heading(level: 2)[#text("Why teams choose Atlas")]

#table(
columns: 2,
align: (left, left),
table.header([#text("Capability")], [#text("Benefit")]),
[#text("Markdown-native authoring")],
[#text("Better model reliability and easier prompt design")],
[#text("User-owned Typst layouts")],
[#text("Full control over typography, spacing, and brand rules")],
[#text("Static Nivo and Plotly charts")],
[#text("Rich data stories without a browser or package download")],
[#text("Safe literal emission")],
[#text("Model text cannot silently become Typst code")]
)

#heading(level: 2)[#text("Typical document mix")]

#block(width: 100%)[#image(bytes("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"300\" viewBox=\"0 0 800 300\" role=\"img\"><title>Documents Created This Month</title><rect x=\"0.5\" y=\"0.5\" width=\"799\" height=\"299.0\" rx=\"12\" fill=\"#ffffff\" stroke=\"#d8dee9\"/><text x=\"28.00\" y=\"27.00\" font-family=\"Arial, sans-serif\" font-size=\"19.0\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"start\">Documents Created This Month</text><circle cx=\"284.00\" cy=\"173.50\" r=\"74.08\" fill=\"none\" stroke=\"#eef2f7\" stroke-width=\"52.06\"/><path d=\"M 284.00 73.39 A 100.11 100.11 0 0 1 332.23 261.23 L 307.15 215.61 A 48.05 48.05 0 0 0 284.00 125.45 Z\" fill=\"#c2410c\" stroke=\"#ffffff\" stroke-width=\"3\" stroke-linejoin=\"round\"/><path d=\"M 332.23 261.23 A 100.11 100.11 0 0 1 188.79 204.44 L 238.30 188.35 A 48.05 48.05 0 0 0 307.15 215.61 Z\" fill=\"#f97316\" stroke=\"#ffffff\" stroke-width=\"3\" stroke-linejoin=\"round\"/><path d=\"M 188.79 204.44 A 100.11 100.11 0 0 1 220.19 96.36 L 253.37 136.47 A 48.05 48.05 0 0 0 238.30 188.35 Z\" fill=\"#fb923c\" stroke=\"#ffffff\" stroke-width=\"3\" stroke-linejoin=\"round\"/><path d=\"M 220.19 96.36 A 100.11 100.11 0 0 1 284.00 73.39 L 284.00 125.45 A 48.05 48.05 0 0 0 253.37 136.47 Z\" fill=\"#fdba74\" stroke=\"#ffffff\" stroke-width=\"3\" stroke-linejoin=\"round\"/><circle cx=\"284.00\" cy=\"173.50\" r=\"47.25\" fill=\"#ffffff\" stroke=\"#e2e8f0\" stroke-width=\"1.4\"/><text x=\"284.00\" y=\"171.50\" font-family=\"Arial, sans-serif\" font-size=\"20.0\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"middle\">300</text><text x=\"284.00\" y=\"189.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">total</text><circle cx=\"563.0\" cy=\"132.0\" r=\"5\" fill=\"#c2410c\"/><text x=\"576.00\" y=\"136.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#334155\" text-anchor=\"start\">Client reports</text><text x=\"752.00\" y=\"136.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"end\">42%</text><circle cx=\"563.0\" cy=\"159.0\" r=\"5\" fill=\"#f97316\"/><text x=\"576.00\" y=\"163.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#334155\" text-anchor=\"start\">Internal briefs</text><text x=\"752.00\" y=\"163.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"end\">28%</text><circle cx=\"563.0\" cy=\"186.0\" r=\"5\" fill=\"#fb923c\"/><text x=\"576.00\" y=\"190.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#334155\" text-anchor=\"start\">Proposals</text><text x=\"752.00\" y=\"190.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"end\">19%</text><circle cx=\"563.0\" cy=\"213.0\" r=\"5\" fill=\"#fdba74\"/><text x=\"576.00\" y=\"217.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#334155\" text-anchor=\"start\">Research notes</text><text x=\"752.00\" y=\"217.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"end\">11%</text></svg>"), format: "svg", width: 100%, alt: "Documents Created This Month")]

#quote(block: true)[#par[#text("\"We kept our existing Typst design system and gave the model a language it could produce consistently.\"")]]

#heading(level: 2)[#text("Start with one flow")]

#block(width: 100%)[#enum(
start: 1,
[#text("Add your Typst layout to a Format String node.")],
[#text("Connect model Markdown to ")#strong[#text("Markdown to Typst")]#text(".")],
[#text("Insert ")#raw("typst_source")#text(" at ")#raw("{markdown_body}")#text(".")],
[#text("Compile the formatted string with ")#strong[#text("Typst Source to PDF")]#text(".")]
)]

#block(width: 100%, breakable: true, fill: rgb("#f0fdf4"), stroke: 0.8pt + rgb("#86efac"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#166534"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "+")]]], [#text(fill: rgb("#166534"), weight: "bold", "Success: Ready for Production Documents")])#block(above: 0.35em)[#par[#text("The same generated body can feed PDF, SVG, or combined export nodes.")]]]