#heading(level: 1)[#text("Safety and Graceful Degradation")]

#par[#text("Model-authored Typst-looking text is displayed literally: ")#raw("#pagebreak()")#text(" in code and #pagebreak() in prose do not execute. Brackets ")#text("[")#text("like these")#text("]")#text(", dollar signs $secret$, angle brackets ")#text("<unsafe-tag>")#text(", quotes \"quoted\", and backslashes \\ remain content.")]

#heading(level: 2)[#text("Links and images")]

#par[#link("https://example.com")[#text("HTTPS links work")]#text(", ")#link("mailto:hello@example.com")[#text("email links work")]#text(", and ")#text("unsafe JavaScript destinations degrade to plain text")#text(".")]

#par[#link("https://example.com/image.png")[#text("A remote image is represented by a link")]]

#par[#text("A data URI cannot become an image")]

#heading(level: 2)[#text("Malformed chart")]

#par[#text("An invalid chart does not abort the entire document. Its escaped source and diagnostic are shown instead.")]

#block(width: 100%, breakable: true, fill: rgb("#fef2f2"), stroke: 0.8pt + rgb("#fca5a5"), radius: 4pt, inset: (x: 10pt, y: 8pt), above: 0.5em, below: 0.5em)[#grid(columns: (1.4em, 1fr), column-gutter: 0.4em, align: (center, left), [#box(width: 1.4em, height: 1.4em, fill: rgb("#991b1b"), radius: 50%)[#align(center + horizon)[#text(fill: white, size: 0.72em, weight: "bold", "x")]]], [#text(fill: rgb("#991b1b"), weight: "bold", "Nivo chart could not be rendered")])#block(above: 0.35em)[#par[#text("invalid JSON in nivo chart: key must be a string at line 1 column 3")]#block(above: 0.45em)[#raw("{ this is not valid JSON }\n", block: true, lang: "nivo")]]]

#heading(level: 2)[#text("Cyclic footnote")]

#par[#text("This reference is safe even though its definition refers to itself.")#footnote[#text("The cycle ends as a literal ")#text("[^loop]")#text(" marker.")]]