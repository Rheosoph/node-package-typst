#heading(level: 1)[#text("Flow-Like Markdown: Core Formatting")]

#par[#text("This document is compiled directly from the Markdown-to-Typst body fragment without a user template.")#linebreak()#text("This second source line becomes a visible line break, matching Flow-Like's ")#raw("remarkBreaks")#text(" behavior.")]

#heading(level: 2)[#text("Emphasis and inline content")]

#par[#text("Use ")#strong[#text("bold")]#text(", ")#emph[#text("italic")]#text(", ")#emph[#strong[#text("bold italic")]]#text(", ")#strike[#text("strikethrough")]#text(", and ")#raw("inline code")#text(" in the same paragraph.")]

#par[#text("A safe link points to ")#link("https://github.com/flow-like/flow-like")[#text("Flow-Like")]#text(", while a remote image becomes a link: ")#link("https://example.com/flow-like.png")[#text("Flow-Like logo")]#text(".")]

#line(length: 100%)

#heading(level: 2)[#text("Heading levels")]

#heading(level: 3)[#text("Heading level three")]

#heading(level: 4)[#text("Heading level four")]

#heading(level: 5)[#text("Heading level five")]

#heading(level: 6)[#text("Heading level six")]

#heading(level: 2)[#text("Fenced code")]

#block(width: 100%)[#raw("fn main() {\n    println!(\"Markdown stays readable, Typst stays valid.\");\n}\n", block: true, lang: "rust")]

#block(width: 100%)[#raw("{\n  \"node\": \"typst_markdown_to_typst\",\n  \"pure\": true,\n  \"permissions\": []\n}\n", block: true, lang: "json")]