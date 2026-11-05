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

{markdown_body}
