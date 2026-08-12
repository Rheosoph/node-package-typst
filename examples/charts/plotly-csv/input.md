# Plotly CSV Charts

The same compact Flow-Like config and CSV syntax can target Plotly fences.

## Bar

```plotly
type: bar
title: Tickets by Priority
colors: category10
height: 260
showLegend: true
legendPosition: bottom
---
priority,Open,Closed
Critical,8,13
High,19,31
Medium,37,52
Low,21,44
```

## Line

```plotly
type: line
title: Build Duration
xLabel: Build
yLabel: Minutes
height: 250
---
build,Main,Release
101,8.2,10.4
102,7.8,10.1
103,7.5,9.6
104,7.1,9.2
```

## Area

```plotly
type: area
title: Storage Growth
stacked: true
height: 250
---
month,Documents,Media
Jan,42,18
Feb,49,23
Mar,57,29
Apr,66,34
```

## Scatter

```plotly
type: scatter
title: Quality versus Speed
xLabel: Iteration
yLabel: Score
height: 260
---
iteration,Parser,Renderer
1,72,69
2,78,75
3,84,82
4,91,89
```

## Pie

```plotly
type: pie
title: Runtime Mix
height: 250
---
runtime,value
WASM,61
Native,28
Browser,11
```

## Heatmap

```plotly
type: heatmap
title: Review Coverage
height: 270
---
area,Security,Correctness,Visual
Markdown,94,96,91
Charts,90,93,95
Templates,88,92,94
```

