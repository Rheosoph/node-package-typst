# Atlas: model Markdown to finished documents

Give your model a format it already understands. Atlas converts Flow-Like Markdown into safe, styleable Typst and lets your design system do the rest.

## Why teams choose Atlas

| Capability | Benefit |
|:-----------|:--------|
| Markdown-native authoring | Better model reliability and easier prompt design |
| User-owned Typst layouts | Full control over typography, spacing, and brand rules |
| Static Nivo and Plotly charts | Rich data stories without a browser or package download |
| Safe literal emission | Model text cannot silently become Typst code |

## Typical document mix

```nivo
type: pie
title: Documents Created This Month
colors: [#c2410c, #f97316, #fb923c, #fdba74]
height: 300
legendPosition: right
---
document,value
Client reports,126
Internal briefs,84
Proposals,57
Research notes,33
```

> "We kept our existing Typst design system and gave the model a language it could produce consistently."

## Start with one flow

1. Add your Typst layout to a Format String node.
2. Connect model Markdown to **Markdown to Typst**.
3. Insert `typst_source` at `{markdown_body}`.
4. Compile the formatted string with **Typst Source to PDF**.

:::success Ready for Production Documents
The same generated body can feed PDF, SVG, or combined export nodes.
:::
