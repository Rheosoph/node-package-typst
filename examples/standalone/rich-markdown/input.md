# Rich Markdown Structures

## Lists and tasks

- Product launch
  1. Finalize the narrative
  2. Validate the generated PDF
     - Check tables
     - Check charts
- [x] Markdown converted
- [x] Typst compiled
- [ ] Stakeholder approval

5. Ordered lists can start at a non-default number
6. Their numbering remains semantic

## Aligned table

| Workstream | Owner | Confidence | Status |
|:-----------|:-----:|-----------:|:-------|
| Parser | Ada | 96% | **Ready** |
| Charts | Lin | 91% | *Review* |
| Templates | Noor | 88% | `testing` |

## Quotes and alerts

> A normal blockquote can span lines.
> It keeps the author's words together.
>> Nested quotes remain nested content.

> [!IMPORTANT]
> Generated content inherits the surrounding Typst document's show and set rules.

## Definitions and footnotes

Body fragment
: Typst markup generated from Markdown and inserted into a user-owned document.

Static chart
: A package-free SVG representation suitable for PDF and SVG export.

The conversion behavior is documented alongside the node.[^docs]

[^docs]: Footnotes can contain **formatted evidence** and links such as [the repository](https://github.com/flow-like/flow-like).
