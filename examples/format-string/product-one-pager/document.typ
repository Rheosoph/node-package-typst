#set document(title: "Atlas Product One-pager")
#set page(width: 210mm, height: 270mm, margin: (top: 16mm, bottom: 16mm, x: 18mm), fill: rgb("#fffaf5"))
#set text(font: "Libertinus Serif", size: 10.5pt, fill: rgb("#292524"))
#set par(leading: 0.7em)
#show heading.where(level: 1): it => block(fill: rgb("#7c2d12"), radius: 7pt, inset: 13pt, above: 2pt, below: 12pt)[#set text(fill: white, size: 22pt); #it]
#show heading.where(level: 2): it => block(above: 13pt, below: 5pt)[#set text(fill: rgb("#c2410c"), size: 14pt); #it]
#show table.where(columns: (auto, auto)): it => {
  let children = it.fields().at("children")
  let header-cells = children.first().fields().at("children")
  let data-cells = children.slice(1)
  let body-of(cell) = cell.fields().at("body")

  block(
    width: 174mm,
    above: 1pt,
    below: 2pt,
  )[
    #block(
      width: 100%,
      stroke: (
        top: 1.2pt + rgb("#c2410c"),
        bottom: 0.55pt + rgb("#fdba74"),
      ),
      inset: (top: 6pt, bottom: 5pt),
    )[
      #grid(
        columns: (58mm, 1fr),
        column-gutter: 10pt,
        [#text(fill: rgb("#78716c"), size: 8.5pt, weight: "semibold", tracking: 0.06em)[#smallcaps(body-of(header-cells.at(0)))]],
        [#text(fill: rgb("#78716c"), size: 8.5pt, weight: "semibold", tracking: 0.06em)[#smallcaps(body-of(header-cells.at(1)))]],
      )
    ]
    #for index in range(0, data-cells.len(), step: 2) {
      let capability = body-of(data-cells.at(index))
      let benefit = body-of(data-cells.at(index + 1))
      let rule = if index + 2 < data-cells.len() {
        0.35pt + rgb("#e7d8cc")
      } else {
        0.7pt + rgb("#fdba74")
      }

      block(
        width: 100%,
        stroke: (bottom: rule),
        inset: (top: 7pt, bottom: 7pt),
      )[
        #grid(
          columns: (58mm, 1fr),
          column-gutter: 10pt,
          align: (left + horizon, left + horizon),
          [#text(fill: rgb("#9a3412"), size: 9.5pt, weight: "semibold", capability)],
          [#text(fill: rgb("#57534e"), size: 9.5pt, benefit)],
        )
      ]
    }
  ]
}
#show quote: it => block(fill: rgb("#ffedd5"), stroke: rgb("#fdba74"), radius: 6pt, inset: 11pt)[#set text(fill: rgb("#7c2d12")); #it]

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

#block(width: 100%)[#image(bytes("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"320\" viewBox=\"0 0 800 320\" role=\"img\"><title>Documents Created This Month</title><rect x=\"0.5\" y=\"0.5\" width=\"799\" height=\"319.0\" rx=\"12\" fill=\"#ffffff\" stroke=\"#e7e5e4\"/><text x=\"28.00\" y=\"27.00\" font-family=\"Arial, sans-serif\" font-size=\"19.0\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"start\">Documents Created This Month</text><circle cx=\"284.00\" cy=\"183.50\" r=\"91.71\" fill=\"none\" stroke=\"#e7e5e4\" stroke-width=\"40.26\"/><path d=\"M 285.34 71.67 A 111.84 111.84 0 0 1 339.05 280.85 L 319.23 245.81 A 71.58 71.58 0 0 0 284.86 111.93 Z\" fill=\"#9a3412\"><title>Client reports: 126 (42.0%)</title></path><path d=\"M 336.70 282.15 A 111.84 111.84 0 0 1 178.06 219.33 L 216.20 206.43 A 71.58 71.58 0 0 0 317.73 246.63 Z\" fill=\"#ea580c\"><title>Internal briefs: 84 (28.0%)</title></path><path d=\"M 177.23 216.78 A 111.84 111.84 0 0 1 211.68 98.19 L 237.72 128.90 A 71.58 71.58 0 0 0 215.67 204.80 Z\" fill=\"#f59e0b\"><title>Proposals: 57 (19.0%)</title></path><path d=\"M 213.75 96.48 A 111.84 111.84 0 0 1 282.66 71.67 L 283.14 111.93 A 71.58 71.58 0 0 0 239.04 127.80 Z\" fill=\"#d6a25e\"><title>Research notes: 33 (11.0%)</title></path><circle cx=\"284.00\" cy=\"183.50\" r=\"71.58\" fill=\"#ffffff\"/><text x=\"284.00\" y=\"181.50\" font-family=\"Arial, sans-serif\" font-size=\"36.0\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"middle\">300</text><text x=\"284.00\" y=\"204.50\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#78716c\" text-anchor=\"middle\">TOTAL</text><text x=\"438.84\" y=\"106.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#78716c\" text-anchor=\"start\">BREAKDOWN</text><text x=\"579.84\" y=\"106.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#78716c\" text-anchor=\"end\">VALUE</text><text x=\"627.84\" y=\"106.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#78716c\" text-anchor=\"end\">SHARE</text><line x1=\"438.8\" y1=\"113.5\" x2=\"627.8\" y2=\"113.5\" stroke=\"#d6d3d1\" stroke-width=\"0.9\"/><g><title>Client reports: 126 (42.0%)</title><rect x=\"425.8\" y=\"122.5\" width=\"3\" height=\"28.0\" rx=\"1.5\" fill=\"#9a3412\"/><text x=\"438.84\" y=\"132.50\" font-family=\"Arial, sans-serif\" font-size=\"15.0\" font-weight=\"400\" fill=\"#57534e\" text-anchor=\"start\">Client reports</text><text x=\"579.84\" y=\"148.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">126</text><text x=\"627.84\" y=\"148.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">42%</text><line x1=\"438.8\" y1=\"156.5\" x2=\"627.8\" y2=\"156.5\" stroke=\"#e7e5e4\" stroke-width=\"0.7\"/></g><g><title>Internal briefs: 84 (28.0%)</title><rect x=\"425.8\" y=\"162.5\" width=\"3\" height=\"28.0\" rx=\"1.5\" fill=\"#ea580c\"/><text x=\"438.84\" y=\"172.50\" font-family=\"Arial, sans-serif\" font-size=\"15.0\" font-weight=\"400\" fill=\"#57534e\" text-anchor=\"start\">Internal briefs</text><text x=\"579.84\" y=\"188.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">84</text><text x=\"627.84\" y=\"188.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">28%</text><line x1=\"438.8\" y1=\"196.5\" x2=\"627.8\" y2=\"196.5\" stroke=\"#e7e5e4\" stroke-width=\"0.7\"/></g><g><title>Proposals: 57 (19.0%)</title><rect x=\"425.8\" y=\"202.5\" width=\"3\" height=\"28.0\" rx=\"1.5\" fill=\"#f59e0b\"/><text x=\"438.84\" y=\"212.50\" font-family=\"Arial, sans-serif\" font-size=\"15.0\" font-weight=\"400\" fill=\"#57534e\" text-anchor=\"start\">Proposals</text><text x=\"579.84\" y=\"228.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">57</text><text x=\"627.84\" y=\"228.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">19%</text><line x1=\"438.8\" y1=\"236.5\" x2=\"627.8\" y2=\"236.5\" stroke=\"#e7e5e4\" stroke-width=\"0.7\"/></g><g><title>Research notes: 33 (11.0%)</title><rect x=\"425.8\" y=\"242.5\" width=\"3\" height=\"28.0\" rx=\"1.5\" fill=\"#d6a25e\"/><text x=\"438.84\" y=\"252.50\" font-family=\"Arial, sans-serif\" font-size=\"15.0\" font-weight=\"400\" fill=\"#57534e\" text-anchor=\"start\">Research notes</text><text x=\"579.84\" y=\"268.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">33</text><text x=\"627.84\" y=\"268.50\" font-family=\"Arial, sans-serif\" font-size=\"14.6\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"end\">11%</text></g></svg>"), format: "svg", width: 100%, alt: "Documents Created This Month")]

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
