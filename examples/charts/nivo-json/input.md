# Nivo Native JSON

Use JSON when chart data needs structures that the simple CSV form cannot express.

## Flat-record bar data

```nivo
{
  "chartType": "bar",
  "title": "Regional Revenue",
  "indexBy": "region",
  "keys": ["software", "services"],
  "colors": { "scheme": "paired" },
  "height": 280,
  "showLegend": true,
  "legendPosition": "bottom",
  "data": [
    { "region": "North", "software": 82, "services": 36 },
    { "region": "South", "software": 67, "services": 44 },
    { "region": "West", "software": 94, "services": 51 }
  ]
}
```

## Nested line series

```nivo
{
  "chartType": "line",
  "title": "Conversion Rate",
  "xLabel": "Month",
  "yLabel": "Percent",
  "height": 270,
  "data": [
    { "id": "Baseline", "data": [{ "x": "Jan", "y": 3.1 }, { "x": "Feb", "y": 3.4 }, { "x": "Mar", "y": 3.8 }, { "x": "Apr", "y": 4.0 }] },
    { "id": "New flow", "data": [{ "x": "Jan", "y": 3.3 }, { "x": "Feb", "y": 3.9 }, { "x": "Mar", "y": 4.6 }, { "x": "Apr", "y": 5.2 }] }
  ]
}
```

## Pie records

```nivo
{
  "chartType": "pie",
  "title": "Document Formats",
  "height": 260,
  "data": [
    { "id": "PDF", "label": "PDF", "value": 58 },
    { "id": "SVG", "label": "SVG", "value": 27 },
    { "id": "Other", "label": "Other", "value": 15 }
  ]
}
```

## Heatmap records

```nivo
{
  "chartType": "heatmap",
  "title": "Quality Signals",
  "height": 280,
  "data": [
    { "id": "Parser", "data": [{ "x": "Unit", "y": 96 }, { "x": "Integration", "y": 91 }, { "x": "Visual", "y": 88 }] },
    { "id": "Renderer", "data": [{ "x": "Unit", "y": 94 }, { "x": "Integration", "y": 93 }, { "x": "Visual", "y": 95 }] },
    { "id": "Templates", "data": [{ "x": "Unit", "y": 89 }, { "x": "Integration", "y": 92 }, { "x": "Visual", "y": 90 }] }
  ]
}
```

