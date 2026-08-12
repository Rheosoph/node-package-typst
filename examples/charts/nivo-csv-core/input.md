# Nivo CSV: Core Static Charts

Each fence uses Flow-Like's lightweight configuration header and simple CSV data.

## Grouped bar

```nivo
type: bar
title: Revenue and Profit by Quarter
xLabel: Quarter
yLabel: EUR thousands
colors: paired
height: 280
showLegend: true
legendPosition: bottom
stacked: false
layout: vertical
animate: false
---
quarter,revenue,profit
Q1,120,32
Q2,148,41
Q3,171,49
Q4,205,63
```

## Line

```nivo
type: line
title: Weekly Active Users
xLabel: Week
yLabel: Users
height: 260
---
week,Web,Desktop
W1,82,44
W2,96,51
W3,110,58
W4,137,72
W5,151,80
```

## Area

```nivo
type: area
title: Requests by Region
colors: [#2563eb, #14b8a6]
height: 260
---
month,Europe,Americas
Jan,42,35
Feb,51,40
Mar,57,48
Apr,69,53
```

## Scatter

```nivo
type: scatter
title: Latency versus Throughput
xLabel: Requests per second
yLabel: Latency in milliseconds
height: 280
---
group,x,y
Stable,110,42
Stable,145,50
Stable,180,61
Candidate,120,36
Candidate,160,44
Candidate,205,55
```

## Pie

```nivo
type: pie
title: Acquisition Mix
height: 260
---
channel,value
Organic,42
Partner,26
Paid,19
Referral,13
```

## Radar

```nivo
type: radar
title: Capability Profile
height: 280
---
capability,Current,Target
Quality,72,90
Speed,81,88
Coverage,65,85
Usability,76,92
```

## Heatmap

```nivo
type: heatmap
title: Activity by Team and Day
height: 280
---
team,Mon,Tue,Wed,Thu,Fri
Design,3,5,4,7,6
Engineering,8,9,7,10,8
Research,4,3,6,5,7
```

## Funnel

```nivo
type: funnel
title: Conversion Funnel
height: 280
---
stage,value
Visitors,1200
Signups,620
Activated,390
Customers,145
```

