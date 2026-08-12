# Chart Fallbacks and Exact Parsing

The converter favors a valid, inspectable document when an interactive chart has no faithful static equivalent.

## Unsupported Nivo CSV type

Treemap is recognized as Flow-Like chart input but represented as a deterministic data card.

```nivo
type: treemap
title: Portfolio Structure
height: 260
---
group,item,value
Core,Parser,44
Core,Renderer,31
Platform,Templates,25
```

## Unsupported Nivo JSON shape

```nivo
{
  "chartType": "sankey",
  "title": "Visitor Flow",
  "height": 220,
  "data": {
    "nodes": [{ "id": "Visitors" }, { "id": "Trials" }, { "id": "Customers" }],
    "links": [
      { "source": "Visitors", "target": "Trials", "value": 320 },
      { "source": "Trials", "target": "Customers", "value": 94 }
    ]
  }
}
```

## Unsupported Plotly trace

```plotly
{
  "data": [{ "type": "sankey", "node": { "label": ["A", "B"] }, "link": { "source": [0], "target": [1], "value": [12] } }],
  "layout": { "title": "Unsupported Sankey", "height": 220 }
}
```

## Incompatible Plotly traces

```plotly
{
  "data": [
    { "type": "bar", "x": ["A", "B"], "y": [3, 5] },
    { "type": "pie", "labels": ["A", "B"], "values": [3, 5] }
  ],
  "layout": { "title": "Mixed Bar and Pie", "height": 220 }
}
```

## Malformed JSON

Malformed chart input falls back to escaped source and includes its diagnostic.

```nivo
{ "chartType": "bar", definitely not JSON }
```

## Malformed configuration

```plotly
type: bar
height: very-tall
---
label,value
A,12
B,18
```

## Fence names are exact

Uppercase `Nivo` is an ordinary fenced code block rather than a chart.

```Nivo
type: pie
---
label,value
A,1
B,2
```

## CSV remains deliberately simple

Quoted commas are not supported. This input demonstrates why JSON is preferable for such labels.

```nivo
type: pie
title: Quoted-comma Limitation
height: 240
---
city,value
"New York, NY",12
Berlin,9
```
