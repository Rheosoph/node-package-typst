#set document(title: "Atlas Product One-pager")
#set page(width: 210mm, height: 270mm, margin: (top: 16mm, bottom: 16mm, x: 18mm), fill: rgb("#fffaf5"))
#set text(font: "Libertinus Serif", size: 10.5pt, fill: rgb("#292524"))
#set par(leading: 0.7em)
#show heading.where(level: 1): it => block(fill: rgb("#7c2d12"), radius: 7pt, inset: 13pt, above: 2pt, below: 12pt)[#set text(fill: white, size: 22pt); #it]
#show heading.where(level: 2): it => block(above: 13pt, below: 5pt)[#set text(fill: rgb("#c2410c"), size: 14pt); #it]
#set table(
  inset: (x: 9pt, y: 6pt),
  stroke: (x, y) => (
    top: if y == 0 { none } else { 0.45pt + rgb("#fed7aa") },
    bottom: none,
    left: none,
    right: none,
  ),
  fill: (x, y) => if y == 0 {
    rgb("#7c2d12")
  } else if x == 0 {
    rgb("#fff7ed")
  } else if calc.even(y) {
    rgb("#fffaf5")
  } else {
    white
  },
)
#show table.cell.where(y: 0): it => {
  set text(fill: white, weight: "semibold", size: 9.5pt)
  smallcaps(it)
}
#show table.cell.where(x: 0): it => {
  if it.y == 0 { it } else {
    text(fill: rgb("#9a3412"), weight: "semibold", it)
  }
}
#show table.where(columns: (auto, auto)): it => {
  let children = it.fields().at("children")
  block(
    width: 174mm,
    fill: white,
    stroke: 0.75pt + rgb("#fdba74"),
    radius: 7pt,
    clip: true,
  )[
    #set text(size: 9.5pt)
    #table(
      // Fixed tracks make the regenerated Markdown table span the full
      // 174 mm content measure while preserving the intended 34/66 split.
      columns: (59mm, 115mm),
      align: (left + horizon, left + horizon),
      ..children,
    )
  ]
}
#show quote: it => block(fill: rgb("#ffedd5"), stroke: rgb("#fdba74"), radius: 6pt, inset: 11pt)[#set text(fill: rgb("#7c2d12")); #it]

{markdown_body}
