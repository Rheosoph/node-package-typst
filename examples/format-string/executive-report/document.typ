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

#heading(level: 1)[#text("FY2026 Operating Review")]

#par[#text("Northstar closed the quarter ahead of plan, with revenue growth accelerating and operating leverage improving. The central execution risk is onboarding capacity as enterprise demand expands.")]

#heading(level: 2)[#text("Executive scorecard")]

#table(
columns: 4,
align: (left, right, right, right),
table.header([#text("Metric")], [#text("Actual")], [#text("Plan")], [#text("Variance")]),
[#text("Revenue")],
[#text("EUR 20.5m")],
[#text("EUR 18.0m")],
[#strong[#text("+13.9%")]],
[#text("Gross margin")],
[#text("71.4%")],
[#text("69.0%")],
[#strong[#text("+2.4 pp")]],
[#text("Net retention")],
[#text("118%")],
[#text("112%")],
[#strong[#text("+6 pp")]],
[#text("Enterprise wins")],
[#text("23")],
[#text("18")],
[#strong[#text("+5")]]
)

#heading(level: 2)[#text("Quarterly trajectory")]

#block(width: 100%)[#image(bytes("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"285\" viewBox=\"0 0 800 285\" role=\"img\"><title>Revenue and Operating Profit</title><rect x=\"0.5\" y=\"0.5\" width=\"799\" height=\"284.0\" rx=\"12\" fill=\"#ffffff\" stroke=\"#e7e5e4\"/><text x=\"28.00\" y=\"27.00\" font-family=\"Arial, sans-serif\" font-size=\"19.0\" font-weight=\"600\" fill=\"#292524\" text-anchor=\"start\">Revenue and Operating Profit</text><line x1=\"74.0\" y1=\"201.0\" x2=\"770.0\" y2=\"201.0\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"204.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">0</text><line x1=\"74.0\" y1=\"174.2\" x2=\"770.0\" y2=\"174.2\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"177.70\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">5</text><line x1=\"74.0\" y1=\"147.4\" x2=\"770.0\" y2=\"147.4\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"150.90\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">10</text><line x1=\"74.0\" y1=\"120.6\" x2=\"770.0\" y2=\"120.6\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"124.10\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">15</text><line x1=\"74.0\" y1=\"93.8\" x2=\"770.0\" y2=\"93.8\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"97.30\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">20</text><line x1=\"74.0\" y1=\"67.0\" x2=\"770.0\" y2=\"67.0\" stroke=\"#e7ebf0\" stroke-width=\"1\"/><text x=\"66.00\" y=\"70.50\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"end\">25</text><line x1=\"74.0\" y1=\"201.0\" x2=\"770.0\" y2=\"201.0\" stroke=\"#aeb8c5\"/><text x=\"422.00\" y=\"239.00\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#475569\" text-anchor=\"middle\">Quarter</text><text x=\"18.0\" y=\"134.0\" font-family=\"Arial, sans-serif\" font-size=\"13.0\" font-weight=\"600\" fill=\"#475569\" text-anchor=\"middle\" transform=\"rotate(-90 18.0 134.0)\">EUR millions</text><rect x=\"96.62\" y=\"124.89\" width=\"57.94\" height=\"76.11\" rx=\"2\" fill=\"#4f46e5\"/><rect x=\"161.00\" y=\"191.35\" width=\"57.94\" height=\"9.65\" rx=\"2\" fill=\"#14b8a6\"/><text x=\"161.00\" y=\"218.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">Q1</text><rect x=\"270.62\" y=\"110.95\" width=\"57.94\" height=\"90.05\" rx=\"2\" fill=\"#4f46e5\"/><rect x=\"335.00\" y=\"187.60\" width=\"57.94\" height=\"13.40\" rx=\"2\" fill=\"#14b8a6\"/><text x=\"335.00\" y=\"218.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">Q2</text><rect x=\"444.62\" y=\"100.77\" width=\"57.94\" height=\"100.23\" rx=\"2\" fill=\"#4f46e5\"/><rect x=\"509.00\" y=\"182.78\" width=\"57.94\" height=\"18.22\" rx=\"2\" fill=\"#14b8a6\"/><text x=\"509.00\" y=\"218.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">Q3</text><rect x=\"618.62\" y=\"91.12\" width=\"57.94\" height=\"109.88\" rx=\"2\" fill=\"#4f46e5\"/><rect x=\"683.00\" y=\"178.49\" width=\"57.94\" height=\"22.51\" rx=\"2\" fill=\"#14b8a6\"/><text x=\"683.00\" y=\"218.00\" font-family=\"Arial, sans-serif\" font-size=\"12.0\" font-weight=\"400\" fill=\"#64748b\" text-anchor=\"middle\">Q4</text><rect x=\"220.0\" y=\"247.0\" width=\"10\" height=\"10\" rx=\"2\" fill=\"#4f46e5\"/><text x=\"235.00\" y=\"256.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#4b5563\" text-anchor=\"start\">Revenue</text><rect x=\"400.0\" y=\"247.0\" width=\"10\" height=\"10\" rx=\"2\" fill=\"#14b8a6\"/><text x=\"415.00\" y=\"256.00\" font-family=\"Arial, sans-serif\" font-size=\"12.5\" font-weight=\"400\" fill=\"#4b5563\" text-anchor=\"start\">Operating profit</text></svg>"), format: "svg", width: 100%, alt: "Revenue and Operating Profit")]

#block(width: 100%, breakable: true, fill: rgb("#fffbeb"), stroke: 0.8pt + rgb("#fbbf24"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#92400e"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "!")]]], [#text(fill: rgb("#92400e"), weight: "bold", "Warning: Execution Watch")])#block(above: 0.35em)[#par[#text("Enterprise onboarding time increased from ")#strong[#text("11 days to 15 days")]#text(". Hiring and enablement must land before the next sales cohort starts.")]]]

#heading(level: 2)[#text("Decisions requested")]

#block(width: 100%)[#enum(
start: 1,
[#text("Approve six implementation hires for the enterprise onboarding team.")],
[#text("Fund the document automation program through the next two quarters.")],
[#text("Retire two low-adoption integrations and redirect maintenance capacity.")]
)]

#quote(block: true)[#par[#text("The plan protects near-term delivery while preserving the product investment required for durable growth.")]]

