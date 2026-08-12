#heading(level: 1)[#text("Executive Summary")]

#par[#text("Rheosoph scaled document automation without asking its models to learn Typst. The imported ")#raw("report")#text(" function owns the brand and page system; this body remains model-friendly Markdown.")]

#heading(level: 2)[#text("Quarter at a glance")]

#table(
columns: 2,
align: (left, right),
table.header([#text("Signal")], [#text("Result")]),
[#text("Automated documents")],
[#text("1,284")],
[#text("Successful exports")],
[#text("99.7%")],
[#text("Median compile time")],
[#text("2.1 s")],
[#text("Approved template usage")],
[#text("74%")]
)

#block(width: 100%, breakable: true, fill: rgb("#f0fdf4"), stroke: 0.8pt + rgb("#86efac"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#166534"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "+")]]], [#text(fill: rgb("#166534"), weight: "bold", "Success: Template and Markdown Combined")])#block(above: 0.35em)[#par[#text("Global layout comes from ")#raw("report.typ")#text("; document content comes from the Markdown-to-Typst node.")]]]

#heading(level: 2)[#text("Monthly delivery trend")]

#block(width: 100%)[#image(bytes("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"220\" viewBox=\"0 0 800 220\" role=\"img\"><title>Generated and Approved Documents</title><rect x=\"0.5\" y=\"0.5\" width=\"799\" height=\"219.0\" rx=\"12\" fill=\"#ffffff\" stroke=\"#d8dee9\"/><text x=\"28.00\" y=\"27.00\" font-family=\"Arial, sans-serif\" font-size=\"19.0\" font-weight=\"600\" fill=\"#172033\" text-anchor=\"start\">Generated and Approved Documents</text><line x1=\"74.0\" y1=\"136.0\" x2=\"770.0\" y2=\"136.0\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"139.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">0</text><line x1=\"74.0\" y1=\"118.8\" x2=\"770.0\" y2=\"118.8\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"122.25\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">100</text><line x1=\"74.0\" y1=\"101.5\" x2=\"770.0\" y2=\"101.5\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"105.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">200</text><line x1=\"74.0\" y1=\"84.2\" x2=\"770.0\" y2=\"84.2\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"87.75\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">300</text><line x1=\"74.0\" y1=\"67.0\" x2=\"770.0\" y2=\"67.0\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"70.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">400</text><line x1=\"74.0\" y1=\"136.0\" x2=\"770.0\" y2=\"136.0\" stroke=\"#aeb8c5\"/><text x=\"422.00\" y=\"174.00\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#475569\" text-anchor=\"middle\">Month</text><text x=\"18.0\" y=\"101.5\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#475569\" text-anchor=\"middle\" transform=\"rotate(-90 18.0 101.5)\">Documents</text><rect x=\"96.62\" y=\"86.66\" width=\"57.94\" height=\"49.34\" rx=\"2\" fill=\"#fb562d\"/><rect x=\"161.00\" y=\"89.25\" width=\"57.94\" height=\"46.75\" rx=\"2\" fill=\"#8b61e3\"/><text x=\"161.00\" y=\"153.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">April</text><rect x=\"270.62\" y=\"81.14\" width=\"57.94\" height=\"54.86\" rx=\"2\" fill=\"#fb562d\"/><rect x=\"335.00\" y=\"83.56\" width=\"57.94\" height=\"52.44\" rx=\"2\" fill=\"#8b61e3\"/><text x=\"335.00\" y=\"153.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">May</text><rect x=\"444.62\" y=\"78.90\" width=\"57.94\" height=\"57.10\" rx=\"2\" fill=\"#fb562d\"/><rect x=\"509.00\" y=\"80.45\" width=\"57.94\" height=\"55.55\" rx=\"2\" fill=\"#8b61e3\"/><text x=\"509.00\" y=\"153.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">June</text><rect x=\"618.62\" y=\"75.80\" width=\"57.94\" height=\"60.20\" rx=\"2\" fill=\"#fb562d\"/><rect x=\"683.00\" y=\"77.00\" width=\"57.94\" height=\"59.00\" rx=\"2\" fill=\"#8b61e3\"/><text x=\"683.00\" y=\"153.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">July</text><rect x=\"220.0\" y=\"182.0\" width=\"10\" height=\"10\" rx=\"2\" fill=\"#fb562d\"/><text x=\"235.00\" y=\"191.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#4b5563\" text-anchor=\"start\">Generated</text><rect x=\"400.0\" y=\"182.0\" width=\"10\" height=\"10\" rx=\"2\" fill=\"#8b61e3\"/><text x=\"415.00\" y=\"191.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#4b5563\" text-anchor=\"start\">Approved</text></svg>"), format: "svg", width: 100%, alt: "Generated and Approved Documents")]

#heading(level: 2)[#text("Decisions for next quarter")]

#block(width: 100%)[#enum(
start: 1,
[#text("Expand the approved template library for customer-facing reports.")],
[#text("Connect brand tokens to the shared Typst template package.")],
[#text("Add rendered artifacts to release validation and accessibility review.")]
)]