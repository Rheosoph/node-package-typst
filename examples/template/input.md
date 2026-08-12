# Executive Summary

Rheosoph scaled document automation without asking its models to learn Typst. The imported `report` function owns the brand and page system; this body remains model-friendly Markdown.

## Quarter at a glance

| Signal | Result |
| --- | ---: |
| Automated documents | 1,284 |
| Successful exports | 99.7% |
| Median compile time | 2.1 s |
| Approved template usage | 74% |

:::success Template and Markdown Combined
Global layout comes from `report.typ`; document content comes from the Markdown-to-Typst node.
:::

## Monthly delivery trend

```nivo
type: bar
title: Generated and Approved Documents
xLabel: Month
yLabel: Documents
height: 220
showLegend: true
---
month,Generated,Approved
April,286,271
May,318,304
June,331,322
July,349,342
```

## Decisions for next quarter

1. Expand the approved template library for customer-facing reports.
2. Connect brand tokens to the shared Typst template package.
3. Add rendered artifacts to release validation and accessibility review.
