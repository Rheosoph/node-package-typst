# Plotly Native JSON

These examples use standard Plotly trace and layout objects, normalized into static document charts.

## Grouped bars

```plotly
{
  "data": [
    { "type": "bar", "name": "Plan", "x": ["Q1", "Q2", "Q3", "Q4"], "y": [110, 135, 150, 180], "marker": { "color": "#7c3aed" } },
    { "type": "bar", "name": "Actual", "x": ["Q1", "Q2", "Q3", "Q4"], "y": [104, 142, 157, 191], "marker": { "color": "#06b6d4" } }
  ],
  "layout": {
    "title": { "text": "Plan versus Actual" },
    "xaxis": { "title": { "text": "Quarter" } },
    "yaxis": { "title": "EUR thousands" },
    "barmode": "group",
    "showlegend": true,
    "legend": { "orientation": "h", "y": -0.2 },
    "height": 290
  },
  "config": { "responsive": true }
}
```

## Horizontal stacked bars

```plotly
{
  "data": [
    { "type": "bar", "orientation": "h", "name": "Complete", "y": ["Parser", "Charts", "Docs"], "x": [92, 84, 76] },
    { "type": "bar", "orientation": "h", "name": "Remaining", "y": ["Parser", "Charts", "Docs"], "x": [8, 16, 24] }
  ],
  "layout": { "title": "Delivery Progress", "barmode": "stack", "height": 270 }
}
```

## Histogram

```plotly
{
  "data": [
    { "type": "histogram", "name": "Compile time", "x": [1.9, 2.1, 2.2, 2.2, 2.4, 2.5, 2.5, 2.7, 2.8, 3.0, 3.1, 3.4] }
  ],
  "layout": { "title": "Compile-time Distribution", "xaxis": { "title": "Seconds" }, "height": 260 }
}
```

## Lines and markers

```plotly
{
  "data": [
    { "type": "scatter", "mode": "lines+markers", "name": "Conversion", "x": ["Jan", "Feb", "Mar", "Apr", "May"], "y": [3.1, 3.4, 4.0, 4.6, 5.2], "line": { "color": "#2563eb" } },
    { "type": "scatter", "mode": "lines+markers", "name": "Target", "x": ["Jan", "Feb", "Mar", "Apr", "May"], "y": [3.5, 3.7, 4.1, 4.5, 5.0], "line": { "color": "#f59e0b" } }
  ],
  "layout": { "title": "Monthly Conversion", "xaxis": { "title": "Month" }, "yaxis": { "title": "Percent" }, "height": 270 }
}
```

## Marker-only scatter

```plotly
[
  { "type": "scatter", "mode": "markers", "name": "Stable", "x": [90, 120, 155, 190], "y": [42, 47, 53, 61] },
  { "type": "scatter", "mode": "markers", "name": "Candidate", "x": [95, 130, 170, 215], "y": [35, 39, 46, 54] }
]
```

## Filled area

```plotly
{
  "data": [
    { "type": "scatter", "mode": "lines", "fill": "tozeroy", "stackgroup": "usage", "name": "Documents", "x": ["Jan", "Feb", "Mar", "Apr"], "y": [24, 31, 39, 48] },
    { "type": "scatter", "mode": "lines", "fill": "tonexty", "stackgroup": "usage", "name": "Media", "x": ["Jan", "Feb", "Mar", "Apr"], "y": [12, 15, 19, 23] }
  ],
  "layout": { "title": "Storage Growth", "height": 260 }
}
```

## Pie

```plotly
{
  "data": [{ "type": "pie", "labels": ["PDF", "SVG", "Other"], "values": [58, 29, 13] }],
  "layout": { "title": "Export Formats", "height": 250, "colorway": ["#7c3aed", "#06b6d4", "#f59e0b"] }
}
```

## Heatmap

```plotly
{
  "data": [{ "type": "heatmap", "x": ["Unit", "Integration", "Visual"], "y": ["Parser", "Charts", "Templates"], "z": [[96, 93, 90], [91, 94, 97], [89, 92, 95]] }],
  "layout": { "title": "Test Confidence", "height": 270 }
}
```

## Polar radar

```plotly
{
  "data": [
    { "type": "scatterpolar", "name": "Current", "r": [72, 84, 68, 79, 72], "theta": ["Quality", "Speed", "Coverage", "Usability", "Quality"], "fill": "toself" },
    { "type": "scatterpolar", "name": "Target", "r": [90, 90, 88, 94, 90], "theta": ["Quality", "Speed", "Coverage", "Usability", "Quality"], "fill": "toself" }
  ],
  "layout": { "title": "Capability Profile", "height": 290 }
}
```

