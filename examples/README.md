# Example gallery

This directory contains reproducible inputs and rendered documents for the
Typst nodes. The Markdown examples exercise the complete pipeline:

```text
input.md -> Markdown to Typst -> generated-body.typ
         -> optional Format String layout -> document.typ
         -> Typst Source to PDF/SVG -> output.pdf / output.svg
```

Every generated artifact uses the crate's pinned Typst engine with Universe
packages disabled. This is more representative than compiling with whatever
Typst CLI happens to be installed locally.

## Regenerating the gallery

```bash
mise run examples
```

Check that committed artifacts are current without changing them:

```bash
mise run examples-check
```

The generator first converts and compiles every case in memory. It writes files
only after all examples succeed, so a broken example cannot leave a partially
updated gallery.

## Files in each example

- `input.md` is the value connected to `Markdown to Typst.markdown`.
- `generated-body.typ` is the exact `typst_source` output from that node.
- `layout.typ` is an optional user-owned Format String containing one
  `{markdown_body}` placeholder.
- `document.typ` is the final source sent to `Typst Source to PDF`.
- `output.pdf` and `output.svg` are generated document artifacts.

The imported-template example keeps `report.typ`, `document.typ`, and
`data.json` for the template nodes. Its Markdown path additionally generates
`generated-body.typ`, `resolved-document.typ`, PDF, and SVG.

## Examples

| Example | Assembly | Highlights | Rendered output |
| --- | --- | --- | --- |
| [Core Markdown](standalone/core-markdown/input.md) | No template | Headings, emphasis, code, links, rules, and visible line breaks | [PDF](standalone/core-markdown/output.pdf) · [SVG](standalone/core-markdown/output.svg) |
| [Rich Markdown](standalone/rich-markdown/input.md) | No template | Nested lists, tasks, tables, quotes, alerts, definitions, and footnotes | [PDF](standalone/rich-markdown/output.pdf) · [SVG](standalone/rich-markdown/output.svg) |
| [Flow-Like directives](standalone/directives/input.md) | No template | Info, warning, error, success, tip, spoiler, and fenced-code safety | [PDF](standalone/directives/output.pdf) · [SVG](standalone/directives/output.svg) |
| [Safety and degradation](standalone/safety-and-degradation/input.md) | No template | Typst-looking text, unsafe URLs, remote images, malformed charts, cyclic footnotes | [PDF](standalone/safety-and-degradation/output.pdf) · [SVG](standalone/safety-and-degradation/output.svg) |
| [Nivo CSV - core](charts/nivo-csv-core/input.md) | No template | Bar, line, area, scatter, pie, radar, heatmap, and funnel | [PDF](charts/nivo-csv-core/output.pdf) · [SVG](charts/nivo-csv-core/output.svg) |
| [Nivo CSV - extended](charts/nivo-csv-extended/input.md) | No template | All config keys, bump, areaBump, stream, radialBar, waffle, calendar, autodetection | [PDF](charts/nivo-csv-extended/output.pdf) · [SVG](charts/nivo-csv-extended/output.svg) |
| [Nivo JSON](charts/nivo-json/input.md) | No template | Flat records, nested series, slices, heatmaps, colors, axes, and legends | [PDF](charts/nivo-json/output.pdf) · [SVG](charts/nivo-json/output.svg) |
| [Plotly CSV](charts/plotly-csv/input.md) | No template | Bar, line, area, scatter, pie, and heatmap | [PDF](charts/plotly-csv/output.pdf) · [SVG](charts/plotly-csv/output.svg) |
| [Plotly JSON](charts/plotly-json/input.md) | No template | Bars, histogram, lines, scatter, area, pie, heatmap, and polar radar | [PDF](charts/plotly-json/output.pdf) · [SVG](charts/plotly-json/output.svg) |
| [Chart fallbacks](charts/fallbacks/input.md) | No template | Unsupported shapes, incompatible traces, invalid JSON, and exact fence matching | [PDF](charts/fallbacks/output.pdf) · [SVG](charts/fallbacks/output.svg) |
| [Executive report](format-string/executive-report/input.md) | Format String | Branded report layout, KPI table, chart, callout, header, and footer | [PDF](format-string/executive-report/output.pdf) · [SVG](format-string/executive-report/output.svg) |
| [Technical brief](format-string/technical-brief/input.md) | Format String | Code-oriented styling, implementation checklist, chart, and footnote | [PDF](format-string/technical-brief/output.pdf) · [SVG](format-string/technical-brief/output.svg) |
| [Product one-pager](format-string/product-one-pager/input.md) | Format String | Marketing layout, feature matrix, pie chart, quote, and call to action | [PDF](format-string/product-one-pager/output.pdf) · [SVG](format-string/product-one-pager/output.svg) |
| [Imported report template](template/input.md) | Template node | Markdown body plus imported function and JSON template data | [PDF](template/output.pdf) · [SVG](template/output.svg) |

## Flow-Like chart configuration

CSV chart fences use Flow-Like's small `key: value` header followed by an exact
`---` separator and simple comma-separated rows:

````markdown
```nivo
type: bar
title: Quarterly revenue
xLabel: Quarter
yLabel: Revenue
colors: paired
height: 280
showLegend: true
legendPosition: bottom
stacked: false
layout: vertical
animate: false
---
quarter,revenue,profit
Q1,120,30
Q2,150,42
```
````

This header is intentionally not YAML, and its CSV does not implement quoted
commas. Use native Nivo or Plotly JSON when data needs richer structure.
