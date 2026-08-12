// A FlowLike template function receives JSON fields as named arguments.
// Keep `doc` as the final positional argument: Typst's show rule supplies it.
#let report(
  title: "Untitled report",
  client: "",
  items: (),
  draft: false,
  doc,
) = [
  #let accent = rgb("#1d4ed8")
  #let accent-dark = rgb("#1e3a8a")
  #let accent-soft = rgb("#eff6ff")
  #let border = rgb("#bfdbfe")
  #let muted = rgb("#64748b")

  #set page(
    paper: "a4",
    margin: (top: 22mm, bottom: 18mm, x: 18mm),
    header: context [
      #set text(size: 8pt, fill: muted)
      #grid(
        columns: (1fr, auto),
        [FLOW-LIKE / IMPORTED TEMPLATE],
        [#if client != "" [#client] else [DOCUMENT AUTOMATION]],
      )
      #v(4pt)
      #line(length: 100%, stroke: border)
    ],
    footer: context [
      #line(length: 100%, stroke: border)
      #v(4pt)
      #set text(size: 8pt, fill: muted)
      #grid(columns: (1fr, auto), [CONFIDENTIAL], [Page #counter(page).display()])
    ],
  )
  #set text(font: "Libertinus Serif", size: 10pt, fill: rgb("#172033"))
  #set par(leading: 0.68em)
  #set document(title: title)
  #set heading(numbering: "1.1")
  #show heading.where(level: 1): it => block(
    fill: accent-soft,
    stroke: border,
    radius: 5pt,
    inset: 9pt,
    above: 10pt,
    below: 7pt,
  )[
    #set text(fill: accent-dark, size: 17pt)
    #it
  ]
  #show heading.where(level: 2): it => block(above: 9pt, below: 4pt)[
    #set text(fill: accent, size: 13pt)
    #it
  ]
  #show table: it => block(fill: white, stroke: border, radius: 4pt, inset: 2pt)[#it]
  #show quote: it => block(fill: accent-soft, stroke: border, radius: 4pt, inset: 9pt)[#it]

  #block(width: 100%, fill: accent-dark, radius: 8pt, inset: 12pt)[
    #set text(fill: white)
    #grid(
      columns: (1fr, auto),
      gutter: 10pt,
      [
        #text(size: 22pt, weight: "bold")[#title]
        #if client != "" [
          #linebreak()
          #text(size: 9pt)[Prepared for #client]
        ]
      ],
      [
        #if draft [
          #box(fill: white, radius: 99pt, inset: (x: 7pt, y: 3pt))[
            #set text(fill: accent-dark, size: 8pt, weight: "bold")
            DRAFT
          ]
        ]
      ],
    )
  ]

  #if items.len() > 0 [
    #v(7pt)
    #text(size: 8pt, weight: "bold", fill: muted)[WORKFLOW HIGHLIGHTS]
    #v(4pt)
    #for item in items [
      #box(fill: accent-soft, stroke: border, radius: 99pt, inset: (x: 8pt, y: 4pt))[
        #set text(size: 8.5pt, fill: accent-dark)
        #item
      ]
      #h(4pt)
    ]
  ]

  #v(3pt)
  #doc
]
