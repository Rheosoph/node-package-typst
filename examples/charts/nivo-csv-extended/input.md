# Nivo CSV: Extended Types and Configuration

This gallery includes Flow-Like aliases and print-friendly approximations for interactive chart types.

## Stacked horizontal bars with every config field

```nivo
type: bar
title: Workload by Team
xLabel: Hours
yLabel: Team
colors: [#7c3aed, #06b6d4, #f59e0b]
height: 300
showLegend: true
legendPosition: top
stacked: true
layout: horizontal
animate: true
---
team,Planned,Unplanned,Support
Design,80,14,20
Engineering,130,31,28
Operations,95,22,46
```

## Bump as a static line chart

```nivo
type: bump
title: Search Rank over Time
height: 250
---
week,Alpha,Beta,Gamma
W1,3,1,2
W2,2,1,3
W3,1,3,2
W4,1,2,3
```

## Area bump

```nivo
type: areaBump
title: Segment Share
height: 250
---
quarter,Enterprise,SMB,Consumer
Q1,44,31,25
Q2,48,30,22
Q3,52,29,19
Q4,55,28,17
```

## Stream

```nivo
type: stream
title: Topic Volume
colors: spectral
height: 250
---
month,Automation,Analytics,Documents
Jan,18,14,9
Feb,24,16,12
Mar,28,21,16
Apr,35,25,19
```

## Radial bar approximation

```nivo
type: radialBar
title: Goal Completion
height: 250
---
goal,Current,Remaining
Reliability,84,16
Coverage,72,28
Adoption,63,37
```

## Waffle

```nivo
type: waffle
title: Deployment Outcomes
height: 260
---
result,value
Successful,82
Rolled back,11
Cancelled,7
```

## Calendar

```nivo
type: calendar
title: Daily Runs
height: 250
---
day,value
2026-07-01,4
2026-07-02,8
2026-07-03,3
2026-07-04,10
2026-07-05,7
2026-07-06,12
2026-07-07,6
2026-07-08,9
2026-07-09,5
2026-07-10,11
2026-07-11,2
2026-07-12,7
2026-07-13,13
2026-07-14,8
2026-07-15,6
2026-07-16,10
2026-07-17,4
2026-07-18,9
2026-07-19,3
2026-07-20,12
2026-07-21,7
2026-07-22,11
2026-07-23,5
2026-07-24,8
2026-07-25,4
2026-07-26,10
2026-07-27,6
2026-07-28,13
2026-07-29,9
2026-07-30,7
2026-07-31,11
```

## Automatic pie detection

With two columns and six or fewer numeric rows, Flow-Like detects a pie chart.

```nivo
channel,value
Direct,45
Search,32
Partner,23
```
