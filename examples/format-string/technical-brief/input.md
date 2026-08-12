# Markdown-to-Typst Node: Technical Brief

The node converts model-friendly Flow-Like Markdown into a safe Typst body fragment. The user retains ownership of document layout, branding, and global show rules.

## Data contract

| Direction | Pin | Type | Semantics |
|:----------|:----|:----:|:----------|
| Input | `markdown` | string | Flow-Like Markdown from a model or upstream node |
| Output | `typst_source` | string | Styleable Typst body fragment |

## Pipeline

```text
Model -> Markdown to Typst -> Format String -> Typst Source to PDF
```

Generated text is always emitted through Typst string literals. A model response containing `#pagebreak()` therefore displays that text rather than executing it.

## Implementation checklist

- [x] GFM tables and strikethrough
- [x] Nested ordered, unordered, and task lists
- [x] Flow-Like directives and alerts
- [x] Nivo and Plotly CSV/JSON fences
- [x] Package-free static SVG charts
- [x] URI, recursion, and input-size guards
- [ ] Interactive PDF chart tooltips

## Compile-time trend

```plotly
{
  "data": [
    { "type": "scatter", "mode": "lines+markers", "name": "Native tests", "x": ["v0.1.0", "parser", "charts", "hardening"], "y": [34, 38, 41, 39] },
    { "type": "scatter", "mode": "lines+markers", "name": "Release WASM", "x": ["v0.1.0", "parser", "charts", "hardening"], "y": [39, 42, 46, 45] }
  ],
  "layout": { "title": "Build Duration", "xaxis": { "title": "Milestone" }, "yaxis": { "title": "Seconds" }, "height": 275 }
}
```

:::tip Operational Guidance
Use a distinctive placeholder such as `{markdown_body}`. The Format String node treats other bare `{identifier}` sequences as additional dynamic inputs.
:::

The checked-in artifacts are generated with the same Typst version embedded by the node.[^engine]

[^engine]: The example generator disables Universe packages so chart output also proves the package-free path.
