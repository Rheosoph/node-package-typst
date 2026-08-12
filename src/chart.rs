//! Static rendering for Flow-Like's `nivo` and `plotly` Markdown fences.
//!
//! Flow-Like deliberately accepts two small interchange formats for charts:
//! native JSON and a light-weight `key: value` header followed by a `---` line
//! and simple comma-separated data.  This module turns both forms into an SVG
//! before embedding it in package-free Typst markup.  Keeping the renderer here
//! avoids making generated documents depend on a Typst chart package (or on a
//! JavaScript runtime at compile time).

use serde_json::{Map, Value};
use std::fmt::Write as _;

const SVG_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 360.0;
const MIN_HEIGHT: f64 = 220.0;
const MAX_HEIGHT: f64 = 900.0;
const MAX_CHART_SOURCE_BYTES: usize = 512 * 1024;
const MAX_STATIC_SERIES: usize = 32;
const MAX_STATIC_DATA_POINTS: usize = 4_000;
const FONT_TICK: f64 = 12.0;
const FONT_LEGEND: f64 = 12.5;
const FONT_AXIS: f64 = 13.0;
const FONT_LABEL: f64 = 12.0;
const DEFAULT_COLORS: &[&str] = &[
    "#fb562d", "#8b61e3", "#09aeae", "#eba42c", "#2b7ad6", "#42aa60", "#da529c", "#62778d",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Bar,
    Line,
    Area,
    Scatter,
    Pie,
    Radar,
    Heatmap,
    Funnel,
    Waffle,
    Calendar,
    Fallback,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum LegendPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Debug, Default)]
struct Config {
    chart_type: Option<String>,
    title: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
    colors: Vec<String>,
    height: Option<f64>,
    show_legend: Option<bool>,
    legend_position: LegendPosition,
    stacked: bool,
    stacked_explicit: bool,
    horizontal: bool,
}

#[derive(Clone, Debug, Default)]
struct TableData {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct Point {
    label: String,
    x: Option<f64>,
    y: Option<f64>,
}

#[derive(Clone, Debug)]
struct Series {
    name: String,
    points: Vec<Point>,
    color: Option<String>,
}

#[derive(Clone, Debug)]
struct HeatmapData {
    columns: Vec<String>,
    rows: Vec<String>,
    values: Vec<Vec<Option<f64>>>,
}

#[derive(Clone, Debug)]
struct CalendarCell {
    day: String,
    value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IsoDate {
    year: i32,
    month: u32,
    day: u32,
}

impl IsoDate {
    fn serial_day(self) -> i64 {
        let mut year = self.year;
        if self.month <= 2 {
            year -= 1;
        }
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let shifted_month = self.month as i32 + if self.month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + self.day as i32 - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        (era * 146_097 + day_of_era - 719_468) as i64
    }

    fn weekday_from_monday(self) -> usize {
        // 1970-01-01 was Thursday, index 3 in a Monday-first week.
        (self.serial_day() + 3).rem_euclid(7) as usize
    }
}

#[derive(Clone, Debug)]
struct Chart {
    kind: Kind,
    source_type: String,
    source_name: &'static str,
    title: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
    colors: Vec<String>,
    height: f64,
    show_legend: bool,
    legend_position: LegendPosition,
    stacked: bool,
    horizontal: bool,
    categories: Vec<String>,
    series: Vec<Series>,
    slices: Vec<(String, f64)>,
    heatmap: Option<HeatmapData>,
    calendar: Vec<CalendarCell>,
    fallback: TableData,
    fallback_reason: Option<String>,
}

impl Chart {
    fn from_config(source_name: &'static str, source_type: String, config: Config) -> Self {
        Self {
            kind: Kind::Fallback,
            source_type,
            source_name,
            title: config.title,
            x_label: config.x_label,
            y_label: config.y_label,
            colors: resolve_colors(&config.colors),
            height: config
                .height
                .unwrap_or(DEFAULT_HEIGHT)
                .clamp(MIN_HEIGHT, MAX_HEIGHT),
            show_legend: config.show_legend.unwrap_or(true),
            legend_position: config.legend_position,
            stacked: config.stacked,
            horizontal: config.horizontal,
            categories: Vec::new(),
            series: Vec::new(),
            slices: Vec::new(),
            heatmap: None,
            calendar: Vec::new(),
            fallback: TableData::default(),
            fallback_reason: None,
        }
    }

    fn color(&self, index: usize) -> &str {
        self.colors
            .get(index % self.colors.len())
            .map(String::as_str)
            .unwrap_or(DEFAULT_COLORS[index % DEFAULT_COLORS.len()])
    }

    fn finish(mut self) -> Self {
        let series_points: usize = self.series.iter().map(|series| series.points.len()).sum();
        let heatmap_points: usize = self
            .heatmap
            .as_ref()
            .map_or(0, |heatmap| heatmap.values.iter().map(Vec::len).sum());
        let data_points = series_points
            .max(self.slices.len())
            .max(heatmap_points)
            .max(self.calendar.len());
        if self.series.len() > MAX_STATIC_SERIES || data_points > MAX_STATIC_DATA_POINTS {
            self.kind = Kind::Fallback;
            self.fallback_reason = Some(format!(
                "The chart is too large for a static renderer (maximum {MAX_STATIC_SERIES} series and {MAX_STATIC_DATA_POINTS} data points)."
            ));
        }

        let usable = match self.kind {
            Kind::Bar | Kind::Line | Kind::Area | Kind::Scatter => self
                .series
                .iter()
                .any(|series| series.points.iter().any(|point| point.y.is_some())),
            Kind::Pie | Kind::Funnel | Kind::Waffle => {
                self.slices.iter().any(|(_, value)| value.is_finite())
            }
            Kind::Radar => {
                self.categories.len() >= 3
                    && self
                        .series
                        .iter()
                        .any(|series| series.points.iter().any(|point| point.y.is_some()))
            }
            Kind::Heatmap => self.heatmap.as_ref().is_some_and(|heatmap| {
                heatmap.values.iter().flatten().any(|value| value.is_some())
            }),
            Kind::Calendar => !self.calendar.is_empty(),
            Kind::Fallback => false,
        };

        if !usable {
            if self.kind != Kind::Fallback && self.fallback_reason.is_none() {
                self.fallback_reason = Some(format!(
                    "The {} data could not be represented as a static chart.",
                    self.source_type
                ));
            }
            self.kind = Kind::Fallback;
        }

        if self.series.len() <= 1 && !matches!(self.kind, Kind::Pie | Kind::Waffle) {
            // An explicit `showLegend: true` remains harmless, but a one-item
            // legend carries no information and consumes a surprising amount
            // of printable space.
            self.show_legend = false;
        }
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct PlotBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Convert the body of a Flow-Like `nivo` or `plotly` fence to standalone Typst.
///
/// Only the exact lower-case language identifiers are accepted.  Returned
/// markup has no package imports: it embeds a generated SVG using Typst's
/// built-in `image` and `bytes` functions.
pub(crate) fn chart_to_typst(language: &str, body: &str) -> Result<String, String> {
    let source_name = match language {
        "nivo" => "Nivo",
        "plotly" => "Plotly",
        _ => {
            return Err(format!(
                "unsupported chart fence language {language:?}; expected exactly \"nivo\" or \"plotly\""
            ))
        }
    };

    if body.len() > MAX_CHART_SOURCE_BYTES {
        return Err(format!(
            "chart block is too large (maximum {} KiB)",
            MAX_CHART_SOURCE_BYTES / 1024
        ));
    }

    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Err("chart block is empty".into());
    }

    let chart = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        let value: Value = serde_json::from_str(trimmed)
            .map_err(|error| format!("invalid JSON in {language} chart: {error}"))?;
        match language {
            "nivo" => chart_from_nivo_json(value)?,
            "plotly" => chart_from_plotly_json(value)?,
            _ => unreachable!(),
        }
    } else {
        let (config, table) = parse_csv_mode(trimmed)?;
        chart_from_csv(source_name, language, config, table)
    };

    let chart = chart.finish();
    let svg = render_svg(&chart);
    let alt = chart
        .title
        .clone()
        .unwrap_or_else(|| format!("{} {} chart", chart.source_name, chart.source_type));
    Ok(format!(
        "#image(bytes({}), format: \"svg\", width: 100%, alt: {})",
        typst_string_literal(&svg),
        typst_string_literal(&alt)
    ))
}

fn parse_csv_mode(input: &str) -> Result<(Config, TableData), String> {
    let lines: Vec<&str> = input.lines().collect();
    let delimiter = lines.iter().position(|line| *line == "---");
    let (config, csv_lines) = match delimiter {
        Some(index) => (parse_config(&lines[..index])?, &lines[index + 1..]),
        None => (Config::default(), lines.as_slice()),
    };

    let table = parse_simple_csv(csv_lines);
    if table.headers.is_empty() {
        return Err("chart CSV is empty".into());
    }
    Ok((config, table))
}

fn parse_config(lines: &[&str]) -> Result<Config, String> {
    let mut config = Config::default();
    for (line_index, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let Some(colon) = line.find(':') else {
            return Err(format!(
                "malformed chart config on line {}: expected key: value",
                line_index + 1
            ));
        };
        let key = line[..colon].trim();
        let value = line[colon + 1..].trim();
        if key.is_empty() {
            return Err(format!(
                "malformed chart config on line {}: key is empty",
                line_index + 1
            ));
        }

        match key {
            "type" => {
                if value.is_empty() {
                    return Err("chart config `type` cannot be empty".into());
                }
                config.chart_type = Some(value.to_string());
            }
            "title" => config.title = nonempty_owned(value),
            "xLabel" => config.x_label = nonempty_owned(value),
            "yLabel" => config.y_label = nonempty_owned(value),
            "colors" => config.colors = parse_color_config(value)?,
            "height" => {
                let height = value
                    .parse::<f64>()
                    .map_err(|_| "chart config `height` must be a number".to_string())?;
                if !height.is_finite() || height <= 0.0 {
                    return Err("chart config `height` must be a positive finite number".into());
                }
                config.height = Some(height);
            }
            "showLegend" => config.show_legend = Some(parse_bool("showLegend", value)?),
            "stacked" => {
                config.stacked = parse_bool("stacked", value)?;
                config.stacked_explicit = true;
            }
            "animate" => {
                // Animation has no meaning in print, but validating it catches
                // the same accidental malformed front matter as Flow-Like.
                let _ = parse_bool("animate", value)?;
            }
            "layout" => match value {
                "vertical" => config.horizontal = false,
                "horizontal" => config.horizontal = true,
                _ => return Err("chart config `layout` must be `vertical` or `horizontal`".into()),
            },
            "legendPosition" => {
                config.legend_position = parse_legend_position(value)?;
            }
            // Flow-Like's parser is intentionally forward-compatible: it
            // stores arbitrary scalar keys and lets the target chart ignore
            // them.  Do the same here after verifying key/value syntax.
            _ => {}
        }
    }
    Ok(config)
}

fn parse_bool(key: &str, value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("chart config `{key}` must be `true` or `false`")),
    }
}

fn parse_legend_position(value: &str) -> Result<LegendPosition, String> {
    match value {
        "top" => Ok(LegendPosition::Top),
        "bottom" => Ok(LegendPosition::Bottom),
        "left" => Ok(LegendPosition::Left),
        "right" => Ok(LegendPosition::Right),
        _ => Err("chart config `legendPosition` must be top, bottom, left, or right".into()),
    }
}

fn parse_color_config(value: &str) -> Result<Vec<String>, String> {
    if value.is_empty() {
        return Err("chart config `colors` cannot be empty".into());
    }
    if value.starts_with('[') || value.ends_with(']') {
        if !(value.starts_with('[') && value.ends_with(']')) {
            return Err("chart config `colors` has an unterminated array".into());
        }
        let colors: Vec<String> = value[1..value.len() - 1]
            .split(',')
            .map(|entry| {
                entry
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
            })
            .filter(|entry| !entry.is_empty())
            .collect();
        if colors.is_empty() {
            return Err("chart config `colors` array cannot be empty".into());
        }
        Ok(colors)
    } else {
        Ok(vec![value.trim_matches('"').trim_matches('\'').to_string()])
    }
}

fn parse_simple_csv(lines: &[&str]) -> TableData {
    let mut meaningful = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !is_leaked_fence(line));
    let Some(header) = meaningful.next() else {
        return TableData::default();
    };
    let headers = header
        .split(',')
        .map(|cell| cell.trim().to_string())
        .collect();
    let rows = meaningful
        .map(|line| {
            line.split(',')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect();
    TableData { headers, rows }
}

fn is_leaked_fence(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.len() < 3 || (bytes[0] != b'`' && bytes[0] != b'~') {
        return false;
    }
    let marker = bytes[0];
    let count = bytes.iter().take_while(|byte| **byte == marker).count();
    if count < 3 {
        return false;
    }
    line[count..]
        .trim()
        .chars()
        .all(|character| character.is_alphanumeric() || character == '_' || character == '-')
}

fn nonempty_owned(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

fn finite_number(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}

fn scalar_label(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => format!("[{} items]", values.len()),
        Value::Object(values) => format!("{{{} fields}}", values.len()),
    }
}

fn json_number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(value) => value.as_f64().filter(|number| number.is_finite()),
        Value::String(value) => finite_number(value),
        _ => None,
    }
}

fn json_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn chart_from_csv(
    source_name: &'static str,
    language: &str,
    mut config: Config,
    table: TableData,
) -> Chart {
    let chart_type = config
        .chart_type
        .clone()
        .unwrap_or_else(|| auto_detect_type(&table));
    let (kind, alias_stacked, alias_horizontal) = csv_kind(language, &chart_type);
    if alias_stacked && !config.stacked_explicit {
        config.stacked = true;
    }
    if alias_horizontal {
        config.horizontal = true;
    }

    let mut chart = Chart::from_config(source_name, chart_type.clone(), config);
    chart.kind = kind;
    chart.fallback = table.clone();

    match kind {
        Kind::Bar | Kind::Line | Kind::Area | Kind::Radar => {
            populate_regular_csv(&mut chart, &table);
        }
        Kind::Scatter => populate_scatter_csv(&mut chart, &table, language),
        Kind::Pie | Kind::Funnel | Kind::Waffle => {
            chart.slices = slices_from_csv(&table);
        }
        Kind::Heatmap => chart.heatmap = heatmap_from_csv(&table),
        Kind::Calendar => chart.calendar = calendar_from_csv(&table),
        Kind::Fallback => {
            chart.fallback_reason = Some(format!(
                "{} chart type `{}` has no static renderer.",
                source_name, chart_type
            ));
        }
    }
    chart
}

fn csv_kind(language: &str, chart_type: &str) -> (Kind, bool, bool) {
    match chart_type {
        "bar" => (Kind::Bar, false, false),
        "line" => (Kind::Line, false, false),
        "area" => (Kind::Area, false, false),
        "scatter" | "scatterplot" => (Kind::Scatter, false, false),
        "pie" => (Kind::Pie, false, false),
        "radar" => (Kind::Radar, false, false),
        "heatmap" => (Kind::Heatmap, false, false),
        "funnel" => (Kind::Funnel, false, false),
        "bump" if language == "nivo" => (Kind::Line, false, false),
        "areaBump" if language == "nivo" => (Kind::Area, true, false),
        "stream" if language == "nivo" => (Kind::Area, true, false),
        "radialBar" if language == "nivo" => (Kind::Bar, true, true),
        "waffle" if language == "nivo" => (Kind::Waffle, false, false),
        "calendar" if language == "nivo" => (Kind::Calendar, false, false),
        _ => (Kind::Fallback, false, false),
    }
}

fn auto_detect_type(table: &TableData) -> String {
    if table.headers.len() == 2
        && table.rows.len() <= 6
        && !table.rows.is_empty()
        && table
            .rows
            .iter()
            .all(|row| row.get(1).and_then(|value| finite_number(value)).is_some())
    {
        return "pie".into();
    }
    if table.headers.len() >= 3
        && table.rows.iter().any(|row| {
            let label = row.first().map(String::as_str).unwrap_or("").to_lowercase();
            [
                "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
                "q1", "q2", "q3", "q4",
            ]
            .iter()
            .any(|needle| label.contains(needle))
                || label.get(..4).is_some_and(|prefix| {
                    prefix.chars().all(|character| character.is_ascii_digit())
                })
        })
    {
        return "line".into();
    }
    "bar".into()
}

fn populate_regular_csv(chart: &mut Chart, table: &TableData) {
    chart.categories = table
        .rows
        .iter()
        .map(|row| row.first().cloned().unwrap_or_default())
        .collect();

    for (series_index, name) in table.headers.iter().skip(1).enumerate() {
        let points = table
            .rows
            .iter()
            .map(|row| {
                let label = row.first().cloned().unwrap_or_default();
                Point {
                    x: finite_number(&label),
                    label,
                    y: row
                        .get(series_index + 1)
                        .and_then(|value| finite_number(value)),
                }
            })
            .collect();
        chart.series.push(Series {
            name: name.clone(),
            points,
            color: None,
        });
    }
}

fn populate_scatter_csv(chart: &mut Chart, table: &TableData, language: &str) {
    if language == "nivo" && table.headers.len() >= 3 {
        // Flow-Like's Nivo CSV convention is group,x,y.
        for row in &table.rows {
            let name = row.first().cloned().unwrap_or_else(|| "Series".into());
            let x_label = row.get(1).cloned().unwrap_or_default();
            let point = Point {
                x: finite_number(&x_label),
                label: x_label,
                y: row.get(2).and_then(|value| finite_number(value)),
            };
            if let Some(series) = chart.series.iter_mut().find(|series| series.name == name) {
                series.points.push(point);
            } else {
                chart.series.push(Series {
                    name,
                    points: vec![point],
                    color: None,
                });
            }
        }
        chart.categories = chart
            .series
            .first()
            .map(|series| {
                series
                    .points
                    .iter()
                    .map(|point| point.label.clone())
                    .collect()
            })
            .unwrap_or_default();
    } else {
        populate_regular_csv(chart, table);
    }
}

fn slices_from_csv(table: &TableData) -> Vec<(String, f64)> {
    table
        .rows
        .iter()
        .filter_map(|row| {
            Some((
                row.first().cloned().unwrap_or_default(),
                row.get(1).and_then(|value| finite_number(value))?,
            ))
        })
        .collect()
}

fn heatmap_from_csv(table: &TableData) -> Option<HeatmapData> {
    if table.headers.len() < 2 || table.rows.is_empty() {
        return None;
    }
    Some(HeatmapData {
        columns: table.headers.iter().skip(1).cloned().collect(),
        rows: table
            .rows
            .iter()
            .map(|row| row.first().cloned().unwrap_or_default())
            .collect(),
        values: table
            .rows
            .iter()
            .map(|row| {
                (1..table.headers.len())
                    .map(|index| row.get(index).and_then(|value| finite_number(value)))
                    .collect()
            })
            .collect(),
    })
}

fn calendar_from_csv(table: &TableData) -> Vec<CalendarCell> {
    table
        .rows
        .iter()
        .filter_map(|row| {
            Some(CalendarCell {
                day: row.first()?.clone(),
                value: row.get(1).and_then(|value| finite_number(value))?,
            })
        })
        .collect()
}

fn resolve_colors(requested: &[String]) -> Vec<String> {
    if requested.is_empty() {
        return DEFAULT_COLORS.iter().map(|color| (*color).into()).collect();
    }
    if requested.len() == 1 {
        if let Some(scheme) = color_scheme(&requested[0]) {
            return scheme.iter().map(|color| (*color).into()).collect();
        }
    }
    let colors: Vec<String> = requested
        .iter()
        .filter_map(|color| sanitize_color(color))
        .collect();
    if colors.is_empty() {
        DEFAULT_COLORS.iter().map(|color| (*color).into()).collect()
    } else {
        colors
    }
}

fn sanitize_color(color: &str) -> Option<String> {
    let trimmed = color.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if matches!(hex.len(), 3 | 6 | 8) && hex.chars().all(|character| character.is_ascii_hexdigit())
    {
        return Some(format!("#{hex}"));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "red" => Some("#dc2626".into()),
        "orange" => Some("#ea580c".into()),
        "yellow" => Some("#ca8a04".into()),
        "green" => Some("#16a34a".into()),
        "blue" => Some("#2563eb".into()),
        "purple" => Some("#9333ea".into()),
        "pink" => Some("#db2777".into()),
        "gray" | "grey" => Some("#6b7280".into()),
        "black" => Some("#111827".into()),
        "white" => Some("#ffffff".into()),
        _ => None,
    }
}

fn color_scheme(name: &str) -> Option<&'static [&'static str]> {
    match name.to_ascii_lowercase().as_str() {
        "nivo" => Some(DEFAULT_COLORS),
        "paired" => Some(&[
            "#a6cee3", "#1f78b4", "#b2df8a", "#33a02c", "#fb9a99", "#e31a1c", "#fdbf6f", "#ff7f00",
            "#cab2d6", "#6a3d9a",
        ]),
        "category10" => Some(&[
            "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
            "#bcbd22", "#17becf",
        ]),
        "accent" => Some(&[
            "#7fc97f", "#beaed4", "#fdc086", "#ffff99", "#386cb0", "#f0027f", "#bf5b17", "#666666",
        ]),
        "dark2" => Some(&[
            "#1b9e77", "#d95f02", "#7570b3", "#e7298a", "#66a61e", "#e6ab02", "#a6761d", "#666666",
        ]),
        "set1" => Some(&[
            "#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#ffff33", "#a65628", "#f781bf",
            "#999999",
        ]),
        "set2" => Some(&[
            "#66c2a5", "#fc8d62", "#8da0cb", "#e78ac3", "#a6d854", "#ffd92f", "#e5c494", "#b3b3b3",
        ]),
        "set3" => Some(&[
            "#8dd3c7", "#ffffb3", "#bebada", "#fb8072", "#80b1d3", "#fdb462", "#b3de69", "#fccde5",
        ]),
        "pastel1" => Some(&[
            "#fbb4ae", "#b3cde3", "#ccebc5", "#decbe4", "#fed9a6", "#ffffcc",
        ]),
        "pastel2" => Some(&[
            "#b3e2cd", "#fdcdac", "#cbd5e8", "#f4cae4", "#e6f5c9", "#fff2ae",
        ]),
        "spectral" => Some(&[
            "#d53e4f", "#f46d43", "#fdae61", "#fee08b", "#e6f598", "#abdda4", "#66c2a5", "#3288bd",
        ]),
        "blues" => Some(&["#eff6ff", "#bfdbfe", "#60a5fa", "#2563eb", "#1e3a8a"]),
        "reds" => Some(&["#fef2f2", "#fecaca", "#f87171", "#dc2626", "#7f1d1d"]),
        "greens" => Some(&["#f0fdf4", "#bbf7d0", "#4ade80", "#16a34a", "#14532d"]),
        _ => None,
    }
}

fn chart_from_nivo_json(value: Value) -> Result<Chart, String> {
    let (mut config, data, chart_type, index_by, keys) = match &value {
        Value::Object(object) => {
            let config = nivo_config_from_json(object)?;
            let chart_type = config.chart_type.clone().unwrap_or_else(|| "bar".into());
            let index_by = optional_string_field(object, "indexBy")?;
            let keys = optional_string_array_field(object, "keys")?;
            let data = object.get("data").cloned().unwrap_or(Value::Null);
            (config, data, chart_type, index_by, keys)
        }
        Value::Array(_) => (Config::default(), value.clone(), "bar".into(), None, None),
        _ => return Err("Nivo JSON chart must be an object or array".into()),
    };

    let (kind, alias_stacked, alias_horizontal) = csv_kind("nivo", &chart_type);
    if alias_stacked && !config.stacked_explicit {
        config.stacked = true;
    }
    if alias_horizontal {
        config.horizontal = true;
    }
    let mut chart = Chart::from_config("Nivo", chart_type.clone(), config);
    chart.kind = kind;
    chart.fallback = table_from_json(&data);

    match kind {
        Kind::Bar | Kind::Radar => {
            if let Some((categories, series)) =
                flat_object_series(&data, index_by.as_deref(), keys.as_deref())
            {
                chart.categories = categories;
                chart.series = series;
            } else if let Some((categories, series)) = nested_json_series(&data) {
                chart.categories = categories;
                chart.series = series;
            }
        }
        Kind::Line | Kind::Area | Kind::Scatter => {
            if let Some((categories, series)) = nested_json_series(&data) {
                chart.categories = categories;
                chart.series = series;
            } else if let Some((categories, series)) =
                flat_object_series(&data, index_by.as_deref(), keys.as_deref())
            {
                chart.categories = categories;
                chart.series = series;
            }
        }
        Kind::Pie | Kind::Funnel | Kind::Waffle => {
            chart.slices = slices_from_json(&data);
        }
        Kind::Heatmap => chart.heatmap = heatmap_from_json(&data),
        Kind::Calendar => chart.calendar = calendar_from_json(&data),
        Kind::Fallback => {
            chart.fallback_reason = Some(format!(
                "Nivo chart type `{chart_type}` has no static renderer."
            ));
        }
    }
    Ok(chart)
}

fn nivo_config_from_json(object: &Map<String, Value>) -> Result<Config, String> {
    let mut config = Config {
        chart_type: optional_string_field(object, "chartType")?
            .or(optional_string_field(object, "type")?),
        title: optional_title_field(object, "title")?,
        x_label: optional_string_field(object, "xLabel")?,
        y_label: optional_string_field(object, "yLabel")?,
        ..Config::default()
    };

    if config.x_label.is_none() {
        config.x_label = axis_legend(object.get("axisBottom"));
    }
    if config.y_label.is_none() {
        config.y_label = axis_legend(object.get("axisLeft"));
    }
    if let Some(colors) = object.get("colors") {
        config.colors = colors_from_json(colors)?;
    }
    if let Some(height) = object.get("height") {
        config.height = Some(positive_json_number("height", height)?);
    }
    if let Some(show) = object.get("showLegend") {
        config.show_legend = Some(
            show.as_bool()
                .ok_or_else(|| "Nivo JSON `showLegend` must be a boolean".to_string())?,
        );
    } else if let Some(legends) = object.get("legends") {
        let legends = legends
            .as_array()
            .ok_or_else(|| "Nivo JSON `legends` must be an array".to_string())?;
        config.show_legend = Some(!legends.is_empty());
        if let Some(anchor) = legends
            .first()
            .and_then(Value::as_object)
            .and_then(|legend| legend.get("anchor"))
            .and_then(Value::as_str)
        {
            config.legend_position = match anchor {
                "top" | "top-left" | "top-right" => LegendPosition::Top,
                "left" | "left-top" | "left-bottom" => LegendPosition::Left,
                "right" | "right-top" | "right-bottom" => LegendPosition::Right,
                _ => LegendPosition::Bottom,
            };
        }
    }
    if let Some(position) = object.get("legendPosition") {
        let position = position.as_str().ok_or_else(|| {
            "Nivo JSON `legendPosition` must be top, bottom, left, or right".to_string()
        })?;
        config.legend_position = parse_legend_position(position)?;
    }
    if let Some(stacked) = object.get("stacked") {
        config.stacked = stacked
            .as_bool()
            .ok_or_else(|| "Nivo JSON `stacked` must be a boolean".to_string())?;
        config.stacked_explicit = true;
    }
    if let Some(group_mode) = object.get("groupMode") {
        let group_mode = group_mode
            .as_str()
            .ok_or_else(|| "Nivo JSON `groupMode` must be a string".to_string())?;
        match group_mode {
            "stacked" => config.stacked = true,
            "grouped" => config.stacked = false,
            _ => return Err("Nivo JSON `groupMode` must be `stacked` or `grouped`".into()),
        }
        config.stacked_explicit = true;
    }
    if let Some(layout) = object.get("layout") {
        let layout = layout
            .as_str()
            .ok_or_else(|| "Nivo JSON `layout` must be a string".to_string())?;
        match layout {
            "vertical" => config.horizontal = false,
            "horizontal" => config.horizontal = true,
            _ => return Err("Nivo JSON `layout` must be `vertical` or `horizontal`".into()),
        }
    }
    Ok(config)
}

fn optional_string_field(object: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    value
        .as_str()
        .map(|value| Some(value.to_string()))
        .ok_or_else(|| format!("JSON chart `{key}` must be a string"))
}

fn optional_string_array_field(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or_else(|| format!("JSON chart `{key}` must be an array of strings"))?;
    let mut output = Vec::with_capacity(values.len());
    for value in values {
        output.push(
            value
                .as_str()
                .ok_or_else(|| format!("JSON chart `{key}` must contain only strings"))?
                .to_string(),
        );
    }
    Ok(Some(output))
}

fn optional_title_field(object: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    match value {
        Value::String(value) => Ok(nonempty_owned(value)),
        Value::Object(value) => match value.get("text") {
            Some(Value::String(value)) => Ok(nonempty_owned(value)),
            Some(_) => Err(format!("JSON chart `{key}.text` must be a string")),
            None => Ok(None),
        },
        Value::Null => Ok(None),
        _ => Err(format!(
            "JSON chart `{key}` must be a string or title object"
        )),
    }
}

fn positive_json_number(key: &str, value: &Value) -> Result<f64, String> {
    let number = value
        .as_f64()
        .ok_or_else(|| format!("JSON chart `{key}` must be a number"))?;
    if !number.is_finite() || number <= 0.0 {
        return Err(format!(
            "JSON chart `{key}` must be a positive finite number"
        ));
    }
    Ok(number)
}

fn axis_legend(value: Option<&Value>) -> Option<String> {
    let object = value?.as_object()?;
    json_string(object.get("legend")).or_else(|| {
        object.get("title").and_then(|title| match title {
            Value::String(value) => Some(value.clone()),
            Value::Object(value) => json_string(value.get("text")),
            _ => None,
        })
    })
}

fn colors_from_json(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "JSON chart `colors` array must contain only strings".into())
            })
            .collect(),
        Value::Object(object) => {
            if let Some(scheme) = object.get("scheme") {
                return scheme
                    .as_str()
                    .map(|scheme| vec![scheme.to_string()])
                    .ok_or_else(|| "JSON chart `colors.scheme` must be a string".into());
            }
            if let Some(colors) = object.get("colors").or_else(|| object.get("range")) {
                return colors_from_json(colors);
            }
            // Nivo also accepts callback/config objects that cannot be carried
            // into a static renderer.  They are valid advanced configuration,
            // so use the deterministic default palette rather than failing.
            Ok(Vec::new())
        }
        _ => Err("JSON chart `colors` must be a string, array, or object".into()),
    }
}

fn flat_object_series(
    data: &Value,
    requested_index: Option<&str>,
    requested_keys: Option<&[String]>,
) -> Option<(Vec<String>, Vec<Series>)> {
    let rows = data.as_array()?;
    let objects: Vec<&Map<String, Value>> = rows.iter().filter_map(Value::as_object).collect();
    if objects.is_empty() {
        return None;
    }

    let mut all_keys: Vec<String> = objects
        .iter()
        .flat_map(|object| object.keys().cloned())
        .collect();
    all_keys.sort();
    all_keys.dedup();
    let index_key = requested_index
        .map(str::to_string)
        .or_else(|| {
            [
                "index", "id", "label", "name", "category", "country", "date", "day",
            ]
            .iter()
            .find(|key| objects.iter().any(|object| object.contains_key(**key)))
            .map(|key| (*key).to_string())
        })
        .or_else(|| {
            all_keys
                .iter()
                .find(|key| {
                    objects
                        .iter()
                        .any(|object| object.get(*key).is_some_and(|value| !value.is_number()))
                })
                .cloned()
        })
        .or_else(|| all_keys.first().cloned())?;

    let value_keys: Vec<String> = if let Some(keys) = requested_keys {
        keys.to_vec()
    } else {
        all_keys
            .into_iter()
            .filter(|key| key != &index_key)
            .filter(|key| {
                objects
                    .iter()
                    .any(|object| json_number(object.get(key)).is_some())
            })
            .collect()
    };
    if value_keys.is_empty() {
        return None;
    }

    let categories: Vec<String> = objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            object
                .get(&index_key)
                .map(scalar_label)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| (index + 1).to_string())
        })
        .collect();
    let series = value_keys
        .iter()
        .map(|key| Series {
            name: key.clone(),
            points: objects
                .iter()
                .zip(&categories)
                .map(|(object, label)| Point {
                    label: label.clone(),
                    x: finite_number(label),
                    y: json_number(object.get(key)),
                })
                .collect(),
            color: None,
        })
        .collect();
    Some((categories, series))
}

fn nested_json_series(data: &Value) -> Option<(Vec<String>, Vec<Series>)> {
    let values = data.as_array()?;
    let mut series = Vec::new();
    for (series_index, value) in values.iter().enumerate() {
        let object = value.as_object()?;
        let points = object.get("data")?.as_array()?;
        let mut output_points = Vec::new();
        for (point_index, point) in points.iter().enumerate() {
            let (label, x, y) = match point {
                Value::Object(point) => {
                    let x_value = point.get("x").or_else(|| point.get("label"));
                    let label = x_value
                        .map(scalar_label)
                        .filter(|label| !label.is_empty())
                        .unwrap_or_else(|| (point_index + 1).to_string());
                    (
                        label,
                        json_number(x_value),
                        json_number(point.get("y").or_else(|| point.get("value"))),
                    )
                }
                Value::Array(point) if point.len() >= 2 => {
                    let label = scalar_label(&point[0]);
                    (label, json_number(point.first()), json_number(point.get(1)))
                }
                _ => continue,
            };
            output_points.push(Point { label, x, y });
        }
        series.push(Series {
            name: json_string(object.get("id").or_else(|| object.get("name")))
                .unwrap_or_else(|| format!("Series {}", series_index + 1)),
            points: output_points,
            color: object
                .get("color")
                .and_then(Value::as_str)
                .and_then(sanitize_color),
        });
    }
    let categories = series
        .first()
        .map(|series| {
            series
                .points
                .iter()
                .map(|point| point.label.clone())
                .collect()
        })
        .unwrap_or_default();
    (!series.is_empty()).then_some((categories, series))
}

fn slices_from_json(data: &Value) -> Vec<(String, f64)> {
    let Some(values) = data.as_array() else {
        return Vec::new();
    };
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let object = value.as_object()?;
            let label = json_string(
                object
                    .get("label")
                    .or_else(|| object.get("id"))
                    .or_else(|| object.get("name"))
                    .or_else(|| object.get("day")),
            )
            .unwrap_or_else(|| format!("Item {}", index + 1));
            let value = json_number(
                object
                    .get("value")
                    .or_else(|| object.get("y"))
                    .or_else(|| object.get("amount")),
            )?;
            Some((label, value))
        })
        .collect()
}

fn heatmap_from_json(data: &Value) -> Option<HeatmapData> {
    let rows = data.as_array()?;
    // Native Nivo shape: [{ id, data: [{ x, y }] }].
    if rows
        .first()
        .and_then(Value::as_object)
        .and_then(|row| row.get("data"))
        .and_then(Value::as_array)
        .is_some()
    {
        let first_points = rows.first()?.as_object()?.get("data")?.as_array()?;
        let columns = first_points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                point
                    .as_object()
                    .and_then(|point| json_string(point.get("x")))
                    .unwrap_or_else(|| (index + 1).to_string())
            })
            .collect();
        let mut row_labels = Vec::new();
        let mut values = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let object = row.as_object()?;
            row_labels.push(
                json_string(object.get("id").or_else(|| object.get("label")))
                    .unwrap_or_else(|| (index + 1).to_string()),
            );
            values.push(
                object
                    .get("data")?
                    .as_array()?
                    .iter()
                    .map(|point| {
                        point.as_object().and_then(|point| {
                            json_number(point.get("y").or_else(|| point.get("value")))
                        })
                    })
                    .collect(),
            );
        }
        return Some(HeatmapData {
            columns,
            rows: row_labels,
            values,
        });
    }

    // A flat object matrix is also common in generated Nivo JSON.
    let (row_labels, series) = flat_object_series(data, None, None)?;
    let columns = series.iter().map(|series| series.name.clone()).collect();
    let values = (0..row_labels.len())
        .map(|row| {
            series
                .iter()
                .map(|series| series.points.get(row).and_then(|point| point.y))
                .collect()
        })
        .collect();
    Some(HeatmapData {
        columns,
        rows: row_labels,
        values,
    })
}

fn calendar_from_json(data: &Value) -> Vec<CalendarCell> {
    let Some(values) = data.as_array() else {
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|value| {
            let object = value.as_object()?;
            Some(CalendarCell {
                day: json_string(object.get("day").or_else(|| object.get("date")))?,
                value: json_number(object.get("value"))?,
            })
        })
        .collect()
}

fn table_from_json(value: &Value) -> TableData {
    match value {
        Value::Array(values) if values.iter().any(Value::is_object) => {
            let objects: Vec<&Map<String, Value>> =
                values.iter().filter_map(Value::as_object).collect();
            let mut headers: Vec<String> = objects
                .iter()
                .flat_map(|object| object.keys().cloned())
                .collect();
            headers.sort();
            headers.dedup();
            prioritize_headers(&mut headers);
            headers.truncate(7);
            let rows = objects
                .iter()
                .take(12)
                .map(|object| {
                    headers
                        .iter()
                        .map(|header| object.get(header).map(scalar_label).unwrap_or_default())
                        .collect()
                })
                .collect();
            TableData { headers, rows }
        }
        Value::Array(values) if values.iter().any(Value::is_array) => {
            let columns = values
                .iter()
                .filter_map(Value::as_array)
                .map(Vec::len)
                .max()
                .unwrap_or(0)
                .min(7);
            TableData {
                headers: (1..=columns)
                    .map(|index| format!("Column {index}"))
                    .collect(),
                rows: values
                    .iter()
                    .filter_map(Value::as_array)
                    .take(12)
                    .map(|row| row.iter().take(columns).map(scalar_label).collect())
                    .collect(),
            }
        }
        Value::Array(values) => TableData {
            headers: vec!["Value".into()],
            rows: values
                .iter()
                .take(12)
                .map(|value| vec![scalar_label(value)])
                .collect(),
        },
        Value::Object(object) => {
            if let Some(data) = object.get("data") {
                return table_from_json(data);
            }
            TableData {
                headers: vec!["Field".into(), "Value".into()],
                rows: object
                    .iter()
                    .take(12)
                    .map(|(key, value)| vec![key.clone(), scalar_label(value)])
                    .collect(),
            }
        }
        value => TableData {
            headers: vec!["Value".into()],
            rows: vec![vec![scalar_label(value)]],
        },
    }
}

fn prioritize_headers(headers: &mut [String]) {
    const PRIORITY: &[&str] = &["id", "name", "label", "day", "date", "x", "y", "value"];
    headers.sort_by_key(|header| {
        PRIORITY
            .iter()
            .position(|priority| header == priority)
            .unwrap_or(PRIORITY.len())
    });
}

fn chart_from_plotly_json(value: Value) -> Result<Chart, String> {
    let (traces_value, mut config) = match &value {
        Value::Array(_) => (value.clone(), Config::default()),
        Value::Object(object) => {
            let config = plotly_config_from_json(object)?;
            let traces = if let Some(data) = object.get("data") {
                data.clone()
            } else if object.contains_key("x")
                || object.contains_key("y")
                || object.contains_key("values")
                || object.contains_key("z")
            {
                Value::Array(vec![value.clone()])
            } else {
                Value::Null
            };
            (traces, config)
        }
        _ => return Err("Plotly JSON chart must be an object or trace array".into()),
    };

    let traces = traces_value.as_array();
    let mut kinds = Vec::new();
    let mut trace_type_names = Vec::new();
    let mut invalid_trace_shape = false;
    if let Some(traces) = traces {
        for trace in traces {
            let Some(object) = trace.as_object() else {
                invalid_trace_shape = true;
                continue;
            };
            match plotly_trace_kind(object)? {
                Some((kind, name)) => {
                    kinds.push(kind);
                    trace_type_names.push(name);
                }
                None => {
                    trace_type_names.push(
                        object
                            .get("type")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string(),
                    );
                    kinds.push(Kind::Fallback);
                }
            }
        }
    }

    let source_type = trace_type_names
        .first()
        .cloned()
        .unwrap_or_else(|| "chart".into());
    let kind = compatible_plotly_kind(&kinds);
    if kind == Kind::Area && !config.stacked_explicit {
        config.stacked = traces
            .into_iter()
            .flatten()
            .filter_map(Value::as_object)
            .any(|trace| trace.get("stackgroup").is_some());
    }
    let mut chart = Chart::from_config("Plotly", source_type.clone(), config);
    chart.kind = kind;
    chart.fallback = table_from_json(&traces_value);

    if invalid_trace_shape || traces.is_none() || kinds.contains(&Kind::Fallback) {
        chart.kind = Kind::Fallback;
        chart.fallback_reason = Some(format!(
            "Plotly chart type `{source_type}` or its data shape has no static renderer."
        ));
        return Ok(chart);
    }
    let traces = traces.expect("checked above");

    match kind {
        Kind::Bar => {
            if trace_type_names.iter().all(|name| name == "histogram") {
                populate_plotly_histogram(&mut chart, traces);
            } else {
                populate_plotly_cartesian(&mut chart, traces, true);
            }
        }
        Kind::Line | Kind::Area | Kind::Scatter => {
            populate_plotly_cartesian(&mut chart, traces, false);
        }
        Kind::Pie | Kind::Waffle => populate_plotly_pie(&mut chart, traces),
        Kind::Radar => populate_plotly_radar(&mut chart, traces),
        Kind::Heatmap => chart.heatmap = traces.first().and_then(plotly_heatmap),
        Kind::Funnel => populate_plotly_funnel(&mut chart, traces),
        Kind::Calendar | Kind::Fallback => {
            chart.kind = Kind::Fallback;
            chart.fallback_reason = Some(format!(
                "Plotly chart type `{source_type}` has no static renderer."
            ));
        }
    }
    Ok(chart)
}

fn plotly_config_from_json(object: &Map<String, Value>) -> Result<Config, String> {
    let mut config = Config::default();
    let layout = match object.get("layout") {
        Some(Value::Object(layout)) => Some(layout),
        Some(Value::Null) | None => None,
        Some(_) => return Err("Plotly JSON `layout` must be an object".into()),
    };

    if let Some(layout) = layout {
        config.title = optional_title_field(layout, "title")?;
        if let Some(x_axis) = layout.get("xaxis") {
            if !x_axis.is_object() {
                return Err("Plotly JSON `layout.xaxis` must be an object".into());
            }
            config.x_label = axis_legend(Some(x_axis));
        }
        if let Some(y_axis) = layout.get("yaxis") {
            if !y_axis.is_object() {
                return Err("Plotly JSON `layout.yaxis` must be an object".into());
            }
            config.y_label = axis_legend(Some(y_axis));
        }
        if let Some(height) = layout.get("height") {
            config.height = Some(positive_json_number("layout.height", height)?);
        }
        if let Some(show) = layout.get("showlegend") {
            config.show_legend =
                Some(show.as_bool().ok_or_else(|| {
                    "Plotly JSON `layout.showlegend` must be a boolean".to_string()
                })?);
        }
        if let Some(mode) = layout.get("barmode") {
            let mode = mode
                .as_str()
                .ok_or_else(|| "Plotly JSON `layout.barmode` must be a string".to_string())?;
            config.stacked = matches!(mode, "stack" | "relative");
            config.stacked_explicit = true;
        }
        if let Some(colorway) = layout.get("colorway") {
            config.colors = colors_from_json(colorway)?;
        }
        if let Some(legend) = layout.get("legend") {
            let legend = legend
                .as_object()
                .ok_or_else(|| "Plotly JSON `layout.legend` must be an object".to_string())?;
            let orientation = legend.get("orientation").and_then(Value::as_str);
            let x = legend.get("x").and_then(Value::as_f64);
            let y = legend.get("y").and_then(Value::as_f64);
            config.legend_position = if x.is_some_and(|x| x < 0.0) {
                LegendPosition::Left
            } else if x.is_some_and(|x| x > 1.0) {
                LegendPosition::Right
            } else if y.is_some_and(|y| y > 1.0) {
                LegendPosition::Top
            } else if y.is_some_and(|y| y < 0.0) || orientation == Some("h") {
                LegendPosition::Bottom
            } else {
                LegendPosition::Right
            };
        }
    }
    if config.title.is_none() {
        config.title = optional_title_field(object, "title")?;
    }
    Ok(config)
}

fn plotly_trace_kind(object: &Map<String, Value>) -> Result<Option<(Kind, String)>, String> {
    let chart_type = match object.get("type") {
        Some(Value::String(value)) => value.as_str(),
        Some(_) => return Err("Plotly trace `type` must be a string".into()),
        None => "scatter",
    };
    let lower = chart_type.to_ascii_lowercase();
    let kind = match lower.as_str() {
        "bar" | "histogram" => Kind::Bar,
        "line" => Kind::Line,
        "area" => Kind::Area,
        "pie" => Kind::Pie,
        "heatmap" | "heatmapgl" => Kind::Heatmap,
        "funnel" | "funnelarea" => Kind::Funnel,
        "scatterpolar" | "scatterpolargl" | "radar" => Kind::Radar,
        "scatter" | "scattergl" => {
            let mode = match object.get("mode") {
                Some(Value::String(mode)) => mode.as_str(),
                Some(_) => return Err("Plotly trace `mode` must be a string".into()),
                None => "lines",
            };
            let fill = object.get("fill").and_then(Value::as_str).unwrap_or("none");
            if fill != "none" || object.get("stackgroup").is_some() {
                Kind::Area
            } else if mode.contains("lines") {
                Kind::Line
            } else {
                Kind::Scatter
            }
        }
        _ => return Ok(None),
    };
    Ok(Some((kind, lower)))
}

fn compatible_plotly_kind(kinds: &[Kind]) -> Kind {
    let Some(first) = kinds.first().copied() else {
        return Kind::Fallback;
    };
    if kinds.contains(&Kind::Fallback) {
        return Kind::Fallback;
    }
    if kinds.iter().all(|kind| *kind == first) {
        return first;
    }
    if kinds
        .iter()
        .all(|kind| matches!(kind, Kind::Line | Kind::Area))
    {
        return if kinds.contains(&Kind::Area) {
            Kind::Area
        } else {
            Kind::Line
        };
    }
    if kinds
        .iter()
        .all(|kind| matches!(kind, Kind::Line | Kind::Scatter))
    {
        return Kind::Line;
    }
    Kind::Fallback
}

fn populate_plotly_cartesian(chart: &mut Chart, traces: &[Value], bars: bool) {
    for (trace_index, trace) in traces.iter().enumerate() {
        let Some(object) = trace.as_object() else {
            continue;
        };
        let horizontal = bars && object.get("orientation").and_then(Value::as_str) == Some("h");
        chart.horizontal |= horizontal;
        let (labels, values) = if horizontal {
            (
                object.get("y").and_then(Value::as_array),
                object.get("x").and_then(Value::as_array),
            )
        } else {
            (
                object.get("x").and_then(Value::as_array),
                object.get("y").and_then(Value::as_array),
            )
        };
        let Some(values) = values else {
            continue;
        };
        let points: Vec<Point> = values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let label_value = labels.and_then(|labels| labels.get(index));
                let label = label_value
                    .map(scalar_label)
                    .filter(|label| !label.is_empty())
                    .unwrap_or_else(|| (index + 1).to_string());
                Point {
                    x: if horizontal {
                        None
                    } else {
                        json_number(label_value)
                    },
                    label,
                    y: json_number(Some(value)),
                }
            })
            .collect();
        if chart.categories.is_empty() {
            chart.categories = points.iter().map(|point| point.label.clone()).collect();
        }
        chart.series.push(Series {
            name: json_string(object.get("name"))
                .unwrap_or_else(|| format!("Series {}", trace_index + 1)),
            points,
            color: plotly_trace_color(object),
        });
    }
}

fn plotly_trace_color(object: &Map<String, Value>) -> Option<String> {
    object
        .get("line")
        .and_then(Value::as_object)
        .and_then(|line| line.get("color"))
        .and_then(Value::as_str)
        .and_then(sanitize_color)
        .or_else(|| {
            object
                .get("marker")
                .and_then(Value::as_object)
                .and_then(|marker| marker.get("color"))
                .and_then(Value::as_str)
                .and_then(sanitize_color)
        })
}

fn populate_plotly_pie(chart: &mut Chart, traces: &[Value]) {
    let Some(object) = traces.first().and_then(Value::as_object) else {
        return;
    };
    let Some(values) = object.get("values").and_then(Value::as_array) else {
        return;
    };
    let labels = object.get("labels").and_then(Value::as_array);
    chart.slices = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            Some((
                labels
                    .and_then(|labels| labels.get(index))
                    .map(scalar_label)
                    .unwrap_or_else(|| format!("Item {}", index + 1)),
                json_number(Some(value))?,
            ))
        })
        .collect();
    if let Some(colors) = object
        .get("marker")
        .and_then(Value::as_object)
        .and_then(|marker| marker.get("colors"))
        .and_then(Value::as_array)
    {
        let requested: Vec<String> = colors
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
        if !requested.is_empty() {
            chart.colors = resolve_colors(&requested);
        }
    }
}

fn populate_plotly_radar(chart: &mut Chart, traces: &[Value]) {
    for (index, trace) in traces.iter().enumerate() {
        let Some(object) = trace.as_object() else {
            continue;
        };
        let Some(values) = object.get("r").and_then(Value::as_array) else {
            continue;
        };
        let labels = object.get("theta").and_then(Value::as_array);
        let mut points: Vec<Point> = values
            .iter()
            .enumerate()
            .map(|(point_index, value)| Point {
                label: labels
                    .and_then(|labels| labels.get(point_index))
                    .map(scalar_label)
                    .unwrap_or_else(|| (point_index + 1).to_string()),
                x: None,
                y: json_number(Some(value)),
            })
            .collect();
        // Plotly commonly repeats the first polar point at the end to close
        // the shape. SVG polygons close themselves, so retaining that point
        // would create a duplicate axis and label.
        if points.len() > 1
            && points
                .first()
                .zip(points.last())
                .is_some_and(|(first, last)| first.label == last.label)
        {
            points.pop();
        }
        if chart.categories.is_empty() {
            chart.categories = points.iter().map(|point| point.label.clone()).collect();
        }
        chart.series.push(Series {
            name: json_string(object.get("name"))
                .unwrap_or_else(|| format!("Series {}", index + 1)),
            points,
            color: plotly_trace_color(object),
        });
    }
}

fn plotly_heatmap(trace: &Value) -> Option<HeatmapData> {
    let object = trace.as_object()?;
    let z = object.get("z")?.as_array()?;
    let rows = object.get("y").and_then(Value::as_array);
    let columns = object.get("x").and_then(Value::as_array);
    let width = z
        .iter()
        .filter_map(Value::as_array)
        .map(Vec::len)
        .max()
        .unwrap_or(0);
    Some(HeatmapData {
        columns: (0..width)
            .map(|index| {
                columns
                    .and_then(|columns| columns.get(index))
                    .map(scalar_label)
                    .unwrap_or_else(|| (index + 1).to_string())
            })
            .collect(),
        rows: z
            .iter()
            .enumerate()
            .map(|(index, _)| {
                rows.and_then(|rows| rows.get(index))
                    .map(scalar_label)
                    .unwrap_or_else(|| (index + 1).to_string())
            })
            .collect(),
        values: z
            .iter()
            .filter_map(Value::as_array)
            .map(|row| row.iter().map(|value| json_number(Some(value))).collect())
            .collect(),
    })
}

fn populate_plotly_funnel(chart: &mut Chart, traces: &[Value]) {
    let Some(object) = traces.first().and_then(Value::as_object) else {
        return;
    };
    let orientation = object
        .get("orientation")
        .and_then(Value::as_str)
        .unwrap_or("h");
    let (labels, values) = if orientation == "v" {
        (
            object.get("x").and_then(Value::as_array),
            object.get("y").and_then(Value::as_array),
        )
    } else {
        (
            object.get("y").and_then(Value::as_array),
            object.get("x").and_then(Value::as_array),
        )
    };
    let Some(values) = values else {
        return;
    };
    chart.slices = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            Some((
                labels
                    .and_then(|labels| labels.get(index))
                    .map(scalar_label)
                    .unwrap_or_else(|| format!("Stage {}", index + 1)),
                json_number(Some(value))?,
            ))
        })
        .collect();
}

fn populate_plotly_histogram(chart: &mut Chart, traces: &[Value]) {
    for (trace_index, trace) in traces.iter().enumerate() {
        let Some(object) = trace.as_object() else {
            continue;
        };
        let values: Vec<f64> = object
            .get("x")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|value| json_number(Some(value)))
            .collect();
        if values.is_empty() {
            continue;
        }
        let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
        let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bins = values.len().clamp(1, 10);
        let span = (maximum - minimum).max(1.0);
        let width = span / bins as f64;
        let mut counts = vec![0.0; bins];
        for value in values {
            let index = (((value - minimum) / width).floor() as usize).min(bins - 1);
            counts[index] += 1.0;
        }
        let points: Vec<Point> = counts
            .into_iter()
            .enumerate()
            .map(|(index, count)| {
                let start = minimum + index as f64 * width;
                let end = start + width;
                Point {
                    label: format!("{}–{}", format_tick(start), format_tick(end)),
                    x: None,
                    y: Some(count),
                }
            })
            .collect();
        if chart.categories.is_empty() {
            chart.categories = points.iter().map(|point| point.label.clone()).collect();
        }
        chart.series.push(Series {
            name: json_string(object.get("name"))
                .unwrap_or_else(|| format!("Series {}", trace_index + 1)),
            points,
            color: plotly_trace_color(object),
        });
    }
}

fn render_svg(chart: &Chart) -> String {
    let height = chart.height;
    let mut svg = String::with_capacity(16_384);
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="{height:.0}" viewBox="0 0 800 {height:.0}" role="img">"#
    );
    let _ = write!(
        svg,
        "<title>{}</title>",
        xml_escape(chart.title.as_deref().unwrap_or(chart.source_type.as_str()))
    );
    let _ = write!(
        svg,
        r##"<rect x="0.5" y="0.5" width="799" height="{:.1}" rx="12" fill="#ffffff" stroke="#d8dee9"/>"##,
        height - 1.0
    );
    // SVG presentation attributes are used instead of a <style> element so
    // the image remains friendly to conservative Typst/SVG backends.
    let mut top = 27.0;
    if let Some(title) = &chart.title {
        svg_text(&mut svg, 28.0, top, 19.0, "#172033", "start", title, true);
        top += 30.0;
    }

    let legend_items = legend_items(chart);
    let show_legend = chart.show_legend && !legend_items.is_empty();
    let (left, right, bottom) = match chart.kind {
        Kind::Bar | Kind::Line | Kind::Area | Kind::Scatter | Kind::Fallback => (74.0, 30.0, 68.0),
        Kind::Heatmap => (36.0, 28.0, 34.0),
        Kind::Calendar => (58.0, 28.0, 30.0),
        Kind::Pie | Kind::Radar | Kind::Funnel | Kind::Waffle => (28.0, 28.0, 30.0),
    };
    let mut plot = PlotBox {
        x: left,
        y: top + 10.0,
        width: SVG_WIDTH - left - right,
        height: (height - top - bottom).max(24.0),
    };
    let side_legend_reserve = if chart.kind == Kind::Pie {
        232.0
    } else {
        120.0
    };
    if show_legend {
        match chart.legend_position {
            LegendPosition::Top => {
                render_legend(
                    &mut svg,
                    chart,
                    &legend_items,
                    LegendLayout::horizontal(28.0, top, 744.0),
                );
                plot.y += 28.0;
                plot.height -= 28.0;
            }
            LegendPosition::Bottom => plot.height -= 26.0,
            LegendPosition::Left => {
                plot.x += side_legend_reserve;
                plot.width -= side_legend_reserve;
            }
            LegendPosition::Right => plot.width -= side_legend_reserve,
        }
    }

    match chart.kind {
        Kind::Bar => render_bar(&mut svg, chart, plot),
        Kind::Line => render_line_like(&mut svg, chart, plot, false, false),
        Kind::Area => render_line_like(&mut svg, chart, plot, true, false),
        Kind::Scatter => render_line_like(&mut svg, chart, plot, false, true),
        Kind::Pie => render_pie(&mut svg, chart, plot),
        Kind::Radar => render_radar(&mut svg, chart, plot),
        Kind::Heatmap => render_heatmap(&mut svg, chart, plot),
        Kind::Funnel => render_funnel(&mut svg, chart, plot),
        Kind::Waffle => render_waffle(&mut svg, chart, plot),
        Kind::Calendar => render_calendar(&mut svg, chart, plot),
        Kind::Fallback => render_fallback(&mut svg, chart, plot),
    }

    if show_legend {
        match chart.legend_position {
            LegendPosition::Top => {}
            LegendPosition::Bottom => render_legend(
                &mut svg,
                chart,
                &legend_items,
                LegendLayout::horizontal(28.0, height - 29.0, 744.0),
            ),
            LegendPosition::Left => {
                let (legend_y, row_gap) = side_legend_layout(chart, plot, legend_items.len());
                render_legend(
                    &mut svg,
                    chart,
                    &legend_items,
                    LegendLayout::vertical(24.0, legend_y, side_legend_reserve - 38.0, row_gap),
                );
            }
            LegendPosition::Right => {
                let (legend_y, row_gap) = side_legend_layout(chart, plot, legend_items.len());
                render_legend(
                    &mut svg,
                    chart,
                    &legend_items,
                    LegendLayout::vertical(
                        plot.x + plot.width + 18.0,
                        legend_y,
                        side_legend_reserve - 38.0,
                        row_gap,
                    ),
                );
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

fn side_legend_layout(chart: &Chart, plot: PlotBox, item_count: usize) -> (f64, f64) {
    if chart.kind != Kind::Pie {
        return (plot.y + 12.0, 21.0);
    }

    // A side legend can contain up to ten slices. Keep the relaxed spacing
    // when it fits, but compact dense legends inside the plot's vertical
    // bounds instead of letting their final rows run beyond a short chart.
    let visible = item_count.min(10);
    let top_extent = 10.0;
    let bottom_extent = 4.0;
    let available_span = (plot.height - top_extent - bottom_extent).max(0.0);
    let row_gap = if visible > 1 {
        (available_span / (visible - 1) as f64).min(27.0)
    } else {
        27.0
    };
    let used_span = row_gap * visible.saturating_sub(1) as f64;
    let y = plot.y + top_extent + (available_span - used_span) / 2.0;
    (y, row_gap)
}

fn legend_items(chart: &Chart) -> Vec<String> {
    match chart.kind {
        Kind::Pie => {
            let total: f64 = chart.slices.iter().map(|(_, value)| value.max(0.0)).sum();
            chart
                .slices
                .iter()
                .map(|(name, value)| {
                    let percentage = if total > 0.0 {
                        value.max(0.0) / total * 100.0
                    } else {
                        0.0
                    };
                    format!("{name} · {percentage:.0}%")
                })
                .collect()
        }
        Kind::Waffle => chart.slices.iter().map(|(name, _)| name.clone()).collect(),
        _ => chart
            .series
            .iter()
            .map(|series| series.name.clone())
            .collect(),
    }
}

#[derive(Clone, Copy)]
struct LegendLayout {
    x: f64,
    y: f64,
    width: f64,
    horizontal: bool,
    vertical_gap: f64,
}

impl LegendLayout {
    fn horizontal(x: f64, y: f64, width: f64) -> Self {
        Self {
            x,
            y,
            width,
            horizontal: true,
            vertical_gap: 21.0,
        }
    }

    fn vertical(x: f64, y: f64, width: f64, vertical_gap: f64) -> Self {
        Self {
            x,
            y,
            width,
            horizontal: false,
            vertical_gap,
        }
    }
}

fn render_legend(svg: &mut String, chart: &Chart, items: &[String], layout: LegendLayout) {
    let LegendLayout {
        x,
        y,
        width,
        horizontal,
        vertical_gap,
    } = layout;
    if horizontal {
        let visible = items.len().clamp(1, 8);
        let item_width = (width / visible as f64).clamp(82.0, 180.0);
        let used_width = item_width * visible as f64;
        let start_x = x + ((width - used_width) / 2.0).max(0.0);
        for (index, item) in items.iter().take(visible).enumerate() {
            let item_x = start_x + index as f64 * item_width;
            render_legend_symbol(svg, chart, index, item_x, y - 9.0);
            svg_text(
                svg,
                item_x + 15.0,
                y,
                FONT_LEGEND,
                "#4b5563",
                "start",
                &truncate_label(item, 22),
                false,
            );
        }
    } else {
        if chart.kind == Kind::Pie {
            let total: f64 = chart.slices.iter().map(|(_, value)| value.max(0.0)).sum();
            for (index, (name, value)) in chart.slices.iter().take(10).enumerate() {
                let item_y = y + index as f64 * vertical_gap;
                render_legend_symbol(svg, chart, index, x, item_y - 9.0);
                svg_text(
                    svg,
                    x + 18.0,
                    item_y,
                    FONT_LEGEND,
                    "#334155",
                    "start",
                    &truncate_label(name, ((width - 52.0) / 6.0).max(6.0) as usize),
                    false,
                );
                let percentage = if total > 0.0 {
                    value.max(0.0) / total * 100.0
                } else {
                    0.0
                };
                svg_text(
                    svg,
                    x + width,
                    item_y,
                    FONT_LEGEND,
                    "#172033",
                    "end",
                    &format!("{percentage:.0}%"),
                    true,
                );
            }
            return;
        }
        for (index, item) in items.iter().take(10).enumerate() {
            let item_y = y + index as f64 * vertical_gap;
            render_legend_symbol(svg, chart, index, x, item_y - 9.0);
            svg_text(
                svg,
                x + 15.0,
                item_y,
                FONT_LEGEND,
                "#4b5563",
                "start",
                &truncate_label(item, (width / 6.0) as usize),
                false,
            );
        }
    }
}

fn render_legend_symbol(svg: &mut String, chart: &Chart, index: usize, x: f64, y: f64) {
    let color = legend_color(chart, index);
    if chart.kind == Kind::Pie {
        let _ = write!(
            svg,
            r##"<circle cx="{:.1}" cy="{:.1}" r="5" fill="{color}"/>"##,
            x + 5.0,
            y + 5.0
        );
    } else {
        let _ = write!(
            svg,
            r##"<rect x="{x:.1}" y="{y:.1}" width="10" height="10" rx="2" fill="{color}"/>"##
        );
    }
}

fn legend_color(chart: &Chart, index: usize) -> &str {
    if matches!(chart.kind, Kind::Pie | Kind::Waffle) {
        return chart.color(index);
    }
    chart
        .series
        .get(index)
        .and_then(|series| series.color.as_deref())
        .unwrap_or_else(|| chart.color(index))
}

fn render_bar(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let values: Vec<f64> = chart
        .series
        .iter()
        .flat_map(|series| series.points.iter().filter_map(|point| point.y))
        .collect();
    if values.is_empty() {
        return;
    }
    let (minimum, maximum) = value_domain(&values, chart.stacked, chart);
    if chart.horizontal {
        render_horizontal_bar(svg, chart, plot, minimum, maximum);
        return;
    }
    draw_axes(svg, chart, plot, minimum, maximum, true, false);
    let categories = chart.categories.len().max(1);
    let series_count = chart.series.len().max(1);
    let band = plot.width / categories as f64;
    let slot = band * 0.74;
    let bar_width = if chart.stacked {
        slot
    } else {
        slot / series_count as f64
    };
    let zero = value_to_y(0.0, minimum, maximum, plot);
    for category in 0..categories {
        let mut positive = 0.0;
        let mut negative = 0.0;
        for (series_index, series) in chart.series.iter().enumerate() {
            let value = series
                .points
                .get(category)
                .and_then(|point| point.y)
                .unwrap_or(0.0);
            let (start, end) = if chart.stacked {
                if value >= 0.0 {
                    let start = positive;
                    positive += value;
                    (start, positive)
                } else {
                    let start = negative;
                    negative += value;
                    (start, negative)
                }
            } else {
                (0.0, value)
            };
            let y1 = value_to_y(start, minimum, maximum, plot);
            let y2 = value_to_y(end, minimum, maximum, plot);
            let x = plot.x
                + category as f64 * band
                + (band - slot) / 2.0
                + if chart.stacked {
                    0.0
                } else {
                    series_index as f64 * bar_width
                };
            let y = y1.min(y2);
            let height = (y1 - y2).abs().max(0.8);
            let color = series
                .color
                .as_deref()
                .unwrap_or_else(|| chart.color(series_index));
            let _ = write!(
                svg,
                r##"<rect x="{x:.2}" y="{y:.2}" width="{:.2}" height="{height:.2}" rx="2" fill="{}"/>"##,
                (bar_width * 0.9).max(0.8),
                color
            );
        }
        let _ = zero;
        let label = chart
            .categories
            .get(category)
            .map(String::as_str)
            .unwrap_or("");
        svg_text(
            svg,
            plot.x + (category as f64 + 0.5) * band,
            plot.y + plot.height + 17.0,
            FONT_TICK,
            "#64748b",
            "middle",
            &truncate_label(label, (band / 6.0).max(4.0) as usize),
            false,
        );
    }
}

fn render_horizontal_bar(
    svg: &mut String,
    chart: &Chart,
    mut plot: PlotBox,
    minimum: f64,
    maximum: f64,
) {
    plot.x += 34.0;
    plot.width -= 34.0;
    draw_axes(svg, chart, plot, minimum, maximum, false, false);
    let categories = chart.categories.len().max(1);
    let series_count = chart.series.len().max(1);
    let band = plot.height / categories as f64;
    let slot = band * 0.72;
    let bar_height = if chart.stacked {
        slot
    } else {
        slot / series_count as f64
    };
    let value_to_x = |value: f64| {
        plot.x + ((value - minimum) / (maximum - minimum).max(f64::EPSILON)) * plot.width
    };
    for category in 0..categories {
        let mut positive = 0.0;
        let mut negative = 0.0;
        for (series_index, series) in chart.series.iter().enumerate() {
            let value = series
                .points
                .get(category)
                .and_then(|point| point.y)
                .unwrap_or(0.0);
            let (start, end) = if chart.stacked {
                if value >= 0.0 {
                    let start = positive;
                    positive += value;
                    (start, positive)
                } else {
                    let start = negative;
                    negative += value;
                    (start, negative)
                }
            } else {
                (0.0, value)
            };
            let x1 = value_to_x(start);
            let x2 = value_to_x(end);
            let y = plot.y
                + category as f64 * band
                + (band - slot) / 2.0
                + if chart.stacked {
                    0.0
                } else {
                    series_index as f64 * bar_height
                };
            let color = series
                .color
                .as_deref()
                .unwrap_or_else(|| chart.color(series_index));
            let _ = write!(
                svg,
                r##"<rect x="{:.2}" y="{y:.2}" width="{:.2}" height="{:.2}" rx="2" fill="{}"/>"##,
                x1.min(x2),
                (x1 - x2).abs().max(0.8),
                (bar_height * 0.88).max(0.8),
                color
            );
        }
        let label = chart
            .categories
            .get(category)
            .map(String::as_str)
            .unwrap_or("");
        svg_text(
            svg,
            plot.x - 7.0,
            plot.y + (category as f64 + 0.55) * band,
            FONT_TICK,
            "#64748b",
            "end",
            &truncate_label(label, 14),
            false,
        );
    }
}

fn render_line_like(svg: &mut String, chart: &Chart, plot: PlotBox, area: bool, scatter: bool) {
    let values: Vec<f64> = chart
        .series
        .iter()
        .flat_map(|series| series.points.iter().filter_map(|point| point.y))
        .collect();
    if values.is_empty() {
        return;
    }
    let stacked_area = area && chart.stacked;
    let rank_axis = chart.source_name == "Nivo" && chart.source_type == "bump";
    let (minimum, maximum) = if rank_axis {
        let minimum = values.iter().copied().fold(f64::INFINITY, f64::min).floor();
        let mut maximum = values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil();
        if maximum <= minimum {
            maximum = minimum + 1.0;
        }
        (minimum, maximum)
    } else {
        value_domain(&values, stacked_area, chart)
    };
    let count = chart
        .series
        .iter()
        .map(|series| series.points.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let numeric_x = chart
        .series
        .iter()
        .flat_map(|series| &series.points)
        .all(|point| point.x.is_some());
    let x_values: Vec<f64> = chart
        .series
        .iter()
        .flat_map(|series| series.points.iter().filter_map(|point| point.x))
        .collect();
    let x_min = x_values.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = x_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let x_at = |point: &Point, index: usize| {
        if numeric_x && x_min.is_finite() && x_max > x_min {
            plot.x + (point.x.unwrap_or(x_min) - x_min) / (x_max - x_min) * plot.width
        } else if count <= 1 {
            plot.x + plot.width / 2.0
        } else {
            plot.x + index as f64 / (count - 1) as f64 * plot.width
        }
    };
    let value_y = |value: f64| {
        if rank_axis {
            plot.y + ((value - minimum) / (maximum - minimum).max(f64::EPSILON)) * plot.height
        } else {
            value_to_y(value, minimum, maximum, plot)
        }
    };
    draw_axes(svg, chart, plot, minimum, maximum, true, rank_axis);
    if numeric_x && x_min.is_finite() && x_max.is_finite() {
        render_numeric_x_ticks(svg, plot, x_min, x_max);
    }

    if stacked_area {
        let mut positive = vec![0.0; count];
        let mut negative = vec![0.0; count];
        for (series_index, series) in chart.series.iter().enumerate() {
            let color = series
                .color
                .as_deref()
                .unwrap_or_else(|| chart.color(series_index));
            let mut tops = Vec::new();
            let mut bases = Vec::new();
            for (index, point) in series.points.iter().enumerate() {
                let Some(value) = point.y else {
                    continue;
                };
                let baseline = if value >= 0.0 {
                    &mut positive[index]
                } else {
                    &mut negative[index]
                };
                let start = *baseline;
                *baseline += value;
                let x = x_at(point, index);
                bases.push((x, value_y(start)));
                tops.push((x, value_y(*baseline)));
            }
            if tops.len() < 2 {
                continue;
            }
            let mut fill_path = format!("M {:.2} {:.2}", tops[0].0, tops[0].1);
            for (x, y) in tops.iter().skip(1) {
                let _ = write!(fill_path, " L {x:.2} {y:.2}");
            }
            for (x, y) in bases.iter().rev() {
                let _ = write!(fill_path, " L {x:.2} {y:.2}");
            }
            fill_path.push_str(" Z");
            let _ = write!(
                svg,
                r##"<path d="{fill_path}" fill="{color}" fill-opacity="0.46"/>"##
            );
            let mut line_path = format!("M {:.2} {:.2}", tops[0].0, tops[0].1);
            for (x, y) in tops.iter().skip(1) {
                let _ = write!(line_path, " L {x:.2} {y:.2}");
            }
            let _ = write!(
                svg,
                r##"<path d="{line_path}" fill="none" stroke="{color}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"##
            );
        }
    } else {
        for (series_index, series) in chart.series.iter().enumerate() {
            let color = series
                .color
                .as_deref()
                .unwrap_or_else(|| chart.color(series_index));
            let mut segments: Vec<Vec<(f64, f64)>> = vec![Vec::new()];
            for (index, point) in series.points.iter().enumerate() {
                if let Some(value) = point.y {
                    segments
                        .last_mut()
                        .expect("one segment exists")
                        .push((x_at(point, index), value_y(value)));
                } else if !segments.last().is_some_and(Vec::is_empty) {
                    segments.push(Vec::new());
                }
            }
            for points in segments.iter().filter(|points| !points.is_empty()) {
                if area && points.len() >= 2 {
                    let baseline =
                        value_to_y(0.0_f64.clamp(minimum, maximum), minimum, maximum, plot);
                    let mut path = format!("M {:.2} {baseline:.2}", points[0].0);
                    for (x, y) in points {
                        let _ = write!(path, " L {x:.2} {y:.2}");
                    }
                    let _ = write!(path, " L {:.2} {baseline:.2} Z", points.last().unwrap().0);
                    let _ = write!(
                        svg,
                        r##"<path d="{path}" fill="{}" fill-opacity="0.18"/>"##,
                        color
                    );
                }
                if !scatter && points.len() >= 2 {
                    let mut path = format!("M {:.2} {:.2}", points[0].0, points[0].1);
                    for (x, y) in points.iter().skip(1) {
                        let _ = write!(path, " L {x:.2} {y:.2}");
                    }
                    let _ = write!(
                        svg,
                        r##"<path d="{path}" fill="none" stroke="{}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"##,
                        color
                    );
                }
                for (x, y) in points {
                    let radius = if scatter { 4.5 } else { 3.5 };
                    let _ = write!(
                        svg,
                        r##"<circle cx="{x:.2}" cy="{y:.2}" r="{radius:.1}" fill="{}" stroke="#ffffff" stroke-width="1"/>"##,
                        color
                    );
                }
            }
        }
    }

    if !numeric_x {
        let band = plot.width / count.max(1) as f64;
        let labels = chart
            .series
            .first()
            .map(|series| &series.points)
            .into_iter()
            .flatten();
        for (index, point) in labels.enumerate() {
            if index >= count {
                break;
            }
            svg_text(
                svg,
                x_at(point, index),
                plot.y + plot.height + 17.0,
                FONT_TICK,
                "#64748b",
                "middle",
                &truncate_label(&point.label, (band / 6.0).max(4.0) as usize),
                false,
            );
        }
    }
}

fn render_numeric_x_ticks(svg: &mut String, plot: PlotBox, minimum: f64, maximum: f64) {
    if !minimum.is_finite() || !maximum.is_finite() {
        return;
    }
    if (maximum - minimum).abs() < f64::EPSILON {
        svg_text(
            svg,
            plot.x + plot.width / 2.0,
            plot.y + plot.height + 18.0,
            FONT_TICK,
            "#64748b",
            "middle",
            &format_tick(minimum),
            false,
        );
        return;
    }
    let step = nice_step((maximum - minimum) / 4.0);
    let first = (minimum / step).ceil() * step;
    let mut value = first;
    let mut rendered = 0;
    while value <= maximum + step * 1e-9 && rendered < 8 {
        let ratio = (value - minimum) / (maximum - minimum);
        let x = plot.x + ratio * plot.width;
        let _ = write!(
            svg,
            r##"<line x1="{x:.1}" y1="{:.1}" x2="{x:.1}" y2="{:.1}" stroke="#eef1f5" stroke-width="1"/>"##,
            plot.y,
            plot.y + plot.height
        );
        svg_text(
            svg,
            x,
            plot.y + plot.height + 18.0,
            FONT_TICK,
            "#64748b",
            "middle",
            &format_axis_tick(value, step),
            false,
        );
        value += step;
        rendered += 1;
    }
}

fn draw_axes(
    svg: &mut String,
    chart: &Chart,
    plot: PlotBox,
    minimum: f64,
    maximum: f64,
    y_ticks: bool,
    invert_y: bool,
) {
    let ticks = if invert_y {
        integer_axis_tick_values(minimum, maximum)
    } else {
        axis_tick_values(minimum, maximum)
    };
    let tick_step = ticks
        .windows(2)
        .next()
        .map(|pair| pair[1] - pair[0])
        .unwrap_or(maximum - minimum);
    if y_ticks {
        for value in &ticks {
            let ratio = (*value - minimum) / (maximum - minimum).max(f64::EPSILON);
            let y = if invert_y {
                plot.y + ratio * plot.height
            } else {
                plot.y + plot.height - ratio * plot.height
            };
            let _ = write!(
                svg,
                r##"<line x1="{:.1}" y1="{y:.1}" x2="{:.1}" y2="{y:.1}" stroke="#e7ebf0" stroke-width="1"/>"##,
                plot.x,
                plot.x + plot.width
            );
            svg_text(
                svg,
                plot.x - 8.0,
                y + 3.5,
                FONT_TICK,
                "#64748b",
                "end",
                &format_axis_tick(*value, tick_step),
                false,
            );
        }
    } else {
        for value in &ticks {
            let ratio = (*value - minimum) / (maximum - minimum).max(f64::EPSILON);
            let x = plot.x + ratio * plot.width;
            let _ = write!(
                svg,
                r##"<line x1="{x:.1}" y1="{:.1}" x2="{x:.1}" y2="{:.1}" stroke="#e7ebf0" stroke-width="1"/>"##,
                plot.y,
                plot.y + plot.height
            );
            svg_text(
                svg,
                x,
                plot.y + plot.height + 17.0,
                FONT_TICK,
                "#64748b",
                "middle",
                &format_axis_tick(*value, tick_step),
                false,
            );
        }
    }
    let _ = write!(
        svg,
        r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="#aeb8c5"/>"##,
        plot.x,
        plot.y + plot.height,
        plot.x + plot.width,
        plot.y + plot.height
    );
    if let Some(label) = &chart.x_label {
        svg_text(
            svg,
            plot.x + plot.width / 2.0,
            plot.y + plot.height + 38.0,
            FONT_AXIS,
            "#475569",
            "middle",
            label,
            true,
        );
    }
    if let Some(label) = &chart.y_label {
        let x = 18.0;
        let y = plot.y + plot.height / 2.0;
        let _ = write!(
            svg,
            r##"<text x="{x:.1}" y="{y:.1}" font-family="Arial, sans-serif" font-size="{FONT_AXIS:.1}" font-weight="600" fill="#475569" text-anchor="middle" transform="rotate(-90 {x:.1} {y:.1})">{}</text>"##,
            xml_escape(label)
        );
    }
}

fn value_domain(values: &[f64], stacked: bool, chart: &Chart) -> (f64, f64) {
    let mut minimum = values.iter().copied().fold(0.0_f64, f64::min);
    let mut maximum = values.iter().copied().fold(0.0_f64, f64::max);
    if stacked && !chart.series.is_empty() {
        let categories = chart
            .series
            .iter()
            .map(|series| series.points.len())
            .max()
            .unwrap_or(0);
        for category in 0..categories {
            let positive: f64 = chart
                .series
                .iter()
                .filter_map(|series| series.points.get(category).and_then(|point| point.y))
                .filter(|value| *value > 0.0)
                .sum();
            let negative: f64 = chart
                .series
                .iter()
                .filter_map(|series| series.points.get(category).and_then(|point| point.y))
                .filter(|value| *value < 0.0)
                .sum();
            minimum = minimum.min(negative);
            maximum = maximum.max(positive);
        }
        let normalized_progress = minimum >= 0.0
            && (0..categories).all(|category| {
                let total: f64 = chart
                    .series
                    .iter()
                    .filter_map(|series| series.points.get(category).and_then(|point| point.y))
                    .sum();
                (total - 100.0).abs() < 1e-6
            });
        if normalized_progress {
            return (0.0, 100.0);
        }
    }
    if (maximum - minimum).abs() < f64::EPSILON {
        if maximum == 0.0 {
            maximum = 1.0;
        } else {
            minimum = minimum.min(0.0);
            maximum = maximum.max(0.0) * 1.1;
        }
    } else {
        let padding = (maximum - minimum) * 0.08;
        if minimum < 0.0 {
            minimum -= padding;
        }
        maximum += padding;
    }
    let target_step = (maximum - minimum) / 4.0;
    let mut step = nice_step(target_step);
    if values
        .iter()
        .all(|value| (value - value.round()).abs() < 1e-9)
    {
        step = step.max(1.0);
    }
    let nice_minimum = (minimum / step).floor() * step;
    let mut nice_maximum = (maximum / step).ceil() * step;
    if nice_maximum <= nice_minimum {
        nice_maximum = nice_minimum + step;
    }
    (nice_minimum, nice_maximum)
}

fn nice_step(value: f64) -> f64 {
    if !value.is_finite() || value <= 0.0 {
        return 1.0;
    }
    let exponent = value.log10().floor();
    let power = 10_f64.powf(exponent);
    let fraction = value / power;
    let nice_fraction = if fraction < 1.5 {
        1.0
    } else if fraction < 3.0 {
        2.0
    } else if fraction < 7.0 {
        5.0
    } else {
        10.0
    };
    nice_fraction * power
}

fn axis_tick_values(minimum: f64, maximum: f64) -> Vec<f64> {
    let range = maximum - minimum;
    if !range.is_finite() || range <= 0.0 {
        return vec![minimum];
    }
    let mut step = nice_step(range / 4.0);
    let mut intervals = (range / step).round() as usize;
    if intervals > 6 {
        step = nice_step(range / 5.0);
        intervals = (range / step).round() as usize;
    }
    intervals = intervals.clamp(1, 8);
    (0..=intervals)
        .map(|index| {
            if index == intervals {
                maximum
            } else {
                minimum + index as f64 * step
            }
        })
        .filter(|value| *value <= maximum + step * 1e-9)
        .collect()
}

fn integer_axis_tick_values(minimum: f64, maximum: f64) -> Vec<f64> {
    let minimum = minimum.floor();
    let maximum = maximum.ceil();
    let range = maximum - minimum;
    if !range.is_finite() || range <= 0.0 {
        return vec![minimum];
    }
    let step = nice_step(range / 5.0).ceil().max(1.0);
    let mut ticks = Vec::new();
    let mut value = minimum;
    while value <= maximum + f64::EPSILON && ticks.len() < 8 {
        ticks.push(value);
        value += step;
    }
    if ticks
        .last()
        .is_none_or(|last| (*last - maximum).abs() > f64::EPSILON)
    {
        ticks.push(maximum);
    }
    ticks
}

fn value_to_y(value: f64, minimum: f64, maximum: f64, plot: PlotBox) -> f64 {
    plot.y + plot.height - ((value - minimum) / (maximum - minimum).max(f64::EPSILON)) * plot.height
}

fn render_pie(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let total: f64 = chart.slices.iter().map(|(_, value)| value.max(0.0)).sum();
    if total <= 0.0 {
        render_fallback(svg, chart, plot);
        return;
    }
    let cx = plot.x + plot.width / 2.0;
    let cy = plot.y + plot.height / 2.0;
    let radius = (plot.height * 0.47).min(plot.width * 0.31);
    let inner = radius * 0.48;
    let ring_radius = (radius + inner) / 2.0;
    let ring_width = radius - inner;
    let _ = write!(
        svg,
        r##"<circle cx="{cx:.2}" cy="{cy:.2}" r="{ring_radius:.2}" fill="none" stroke="#eef2f7" stroke-width="{ring_width:.2}"/>"##
    );
    let mut angle = -std::f64::consts::FRAC_PI_2;
    let visible_slices = chart
        .slices
        .iter()
        .filter(|(_, value)| value.is_finite() && *value > 0.0)
        .count();
    for (index, (_, value)) in chart.slices.iter().enumerate() {
        let fraction = value.max(0.0) / total;
        if fraction <= 0.0 {
            continue;
        }
        if visible_slices == 1 {
            // An SVG arc whose start and end points coincide is degenerate.
            // A stroked circle is the exact, backend-friendly full-ring form.
            let _ = write!(
                svg,
                r##"<circle cx="{cx:.2}" cy="{cy:.2}" r="{ring_radius:.2}" fill="none" stroke="{}" stroke-width="{ring_width:.2}"/>"##,
                chart.color(index)
            );
            continue;
        }
        let end = angle + fraction * std::f64::consts::TAU;
        let large = if end - angle > std::f64::consts::PI {
            1
        } else {
            0
        };
        let x1 = cx + radius * angle.cos();
        let y1 = cy + radius * angle.sin();
        let x2 = cx + radius * end.cos();
        let y2 = cy + radius * end.sin();
        let ix2 = cx + inner * end.cos();
        let iy2 = cy + inner * end.sin();
        let ix1 = cx + inner * angle.cos();
        let iy1 = cy + inner * angle.sin();
        let path = format!(
            "M {x1:.2} {y1:.2} A {radius:.2} {radius:.2} 0 {large} 1 {x2:.2} {y2:.2} L {ix2:.2} {iy2:.2} A {inner:.2} {inner:.2} 0 {large} 0 {ix1:.2} {iy1:.2} Z"
        );
        let _ = write!(
            svg,
            r##"<path d="{path}" fill="{}" stroke="#ffffff" stroke-width="3" stroke-linejoin="round"/>"##,
            chart.color(index)
        );
        let middle_angle = angle + (end - angle) / 2.0;
        let label_radius = (radius + inner) / 2.0;
        let percentage = fraction * 100.0;
        let percentage_label = format!("{percentage:.0}%");
        let available_arc = label_radius * (end - angle);
        let estimated_label_width = percentage_label.chars().count() as f64 * 7.0 + 6.0;
        let side_legend = chart.show_legend
            && matches!(
                chart.legend_position,
                LegendPosition::Left | LegendPosition::Right
            );
        if !side_legend
            && radius >= 42.0
            && fraction >= 0.07
            && available_arc >= estimated_label_width
        {
            svg_text(
                svg,
                cx + label_radius * middle_angle.cos(),
                cy + label_radius * middle_angle.sin() + 4.0,
                FONT_LABEL,
                contrast_text_color(chart.color(index)),
                "middle",
                &percentage_label,
                true,
            );
        }
        angle = end;
    }
    let _ = write!(
        svg,
        r##"<circle cx="{cx:.2}" cy="{cy:.2}" r="{:.2}" fill="#ffffff" stroke="#e2e8f0" stroke-width="1.4"/>"##,
        (inner - 0.8).max(1.0)
    );
    svg_text(
        svg,
        cx,
        cy - 2.0,
        20.0,
        "#172033",
        "middle",
        &format_tick(total),
        true,
    );
    svg_text(
        svg,
        cx,
        cy + 16.0,
        FONT_LABEL,
        "#64748b",
        "middle",
        "total",
        false,
    );
}

fn render_radar(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let axes = chart.categories.len();
    if axes < 3 {
        render_fallback(svg, chart, plot);
        return;
    }
    let cx = plot.x + plot.width / 2.0;
    let cy = plot.y + plot.height / 2.0;
    let radius = (plot.width.min(plot.height) / 2.0 - 20.0).max(24.0);
    let observed_maximum = chart
        .series
        .iter()
        .flat_map(|series| series.points.iter().filter_map(|point| point.y))
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let maximum = if (50.0..=100.0).contains(&observed_maximum) {
        100.0
    } else {
        observed_maximum
    };
    let angle_at = |index: usize| {
        -std::f64::consts::FRAC_PI_2 + index as f64 / axes as f64 * std::f64::consts::TAU
    };
    for ring in 1..=4 {
        let ring_radius = radius * ring as f64 / 4.0;
        let points = (0..axes)
            .map(|index| {
                let angle = angle_at(index);
                format!(
                    "{:.2},{:.2}",
                    cx + ring_radius * angle.cos(),
                    cy + ring_radius * angle.sin()
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let _ = write!(
            svg,
            r##"<polygon points="{points}" fill="none" stroke="#dce2e9" stroke-width="1"/>"##
        );
    }
    for index in 0..axes {
        let angle = angle_at(index);
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        let _ = write!(
            svg,
            r##"<line x1="{cx:.2}" y1="{cy:.2}" x2="{x:.2}" y2="{y:.2}" stroke="#dce2e9"/>"##
        );
        let label_x = cx + (radius + 16.0) * angle.cos();
        let label_y = cy + (radius + 16.0) * angle.sin() + 3.0;
        svg_text(
            svg,
            label_x,
            label_y,
            FONT_TICK,
            "#64748b",
            if angle.cos() > 0.25 {
                "start"
            } else if angle.cos() < -0.25 {
                "end"
            } else {
                "middle"
            },
            &truncate_label(&chart.categories[index], 14),
            false,
        );
    }
    for (series_index, series) in chart.series.iter().enumerate() {
        let points = (0..axes)
            .map(|index| {
                let value = series
                    .points
                    .get(index)
                    .and_then(|point| point.y)
                    .unwrap_or(0.0)
                    .max(0.0);
                let r = value / maximum * radius;
                let angle = angle_at(index);
                format!("{:.2},{:.2}", cx + r * angle.cos(), cy + r * angle.sin())
            })
            .collect::<Vec<_>>()
            .join(" ");
        let color = series
            .color
            .as_deref()
            .unwrap_or_else(|| chart.color(series_index));
        let _ = write!(
            svg,
            r##"<polygon points="{points}" fill="{color}" fill-opacity="0.12" stroke="{color}" stroke-width="2" stroke-linejoin="round"/>"##
        );
    }
}

fn render_heatmap(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let Some(heatmap) = &chart.heatmap else {
        render_fallback(svg, chart, plot);
        return;
    };
    let rows = heatmap.rows.len().max(1);
    let columns = heatmap.columns.len().max(1);
    let label_width = 72.0;
    let cell_width = (plot.width - label_width) / columns as f64;
    let cell_height = (plot.height - 24.0) / rows as f64;
    let values: Vec<f64> = heatmap.values.iter().flatten().flatten().copied().collect();
    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    for (column, label) in heatmap.columns.iter().enumerate() {
        svg_text(
            svg,
            plot.x + label_width + (column as f64 + 0.5) * cell_width,
            plot.y + 11.0,
            FONT_TICK,
            "#64748b",
            "middle",
            &truncate_label(label, (cell_width / 6.0).max(4.0) as usize),
            false,
        );
    }
    for (row, label) in heatmap.rows.iter().enumerate() {
        let y = plot.y + 22.0 + row as f64 * cell_height;
        svg_text(
            svg,
            plot.x + label_width - 7.0,
            y + cell_height / 2.0 + 3.5,
            FONT_TICK,
            "#64748b",
            "end",
            &truncate_label(label, 12),
            false,
        );
        for column in 0..columns {
            let value = heatmap
                .values
                .get(row)
                .and_then(|row| row.get(column))
                .copied()
                .flatten();
            let color = value
                .map(|value| heat_color(value, minimum, maximum))
                .unwrap_or_else(|| "#eef1f5".into());
            let x = plot.x + label_width + column as f64 * cell_width;
            let _ = write!(
                svg,
                r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="2" fill="{color}" stroke="#ffffff" stroke-width="2"/>"##,
                x + 1.0,
                y + 1.0,
                (cell_width - 2.0).max(1.0),
                (cell_height - 2.0).max(1.0)
            );
            if let Some(value) = value {
                if cell_width >= 36.0 && cell_height >= 20.0 {
                    svg_text(
                        svg,
                        x + cell_width / 2.0,
                        y + cell_height / 2.0 + 3.0,
                        10.5,
                        if normalized(value, minimum, maximum) > 0.58 {
                            "#ffffff"
                        } else {
                            "#263449"
                        },
                        "middle",
                        &format_tick(value),
                        false,
                    );
                }
            }
        }
    }
}

fn heat_color(value: f64, minimum: f64, maximum: f64) -> String {
    let ratio = normalized(value, minimum, maximum);
    let (start, end) = ((232.0, 241.0, 252.0), (37.0, 99.0, 235.0));
    format!(
        "#{:02x}{:02x}{:02x}",
        (start.0 + (end.0 - start.0) * ratio).round() as u8,
        (start.1 + (end.1 - start.1) * ratio).round() as u8,
        (start.2 + (end.2 - start.2) * ratio).round() as u8
    )
}

fn normalized(value: f64, minimum: f64, maximum: f64) -> f64 {
    if !minimum.is_finite() || !maximum.is_finite() || (maximum - minimum).abs() < f64::EPSILON {
        0.55
    } else {
        ((value - minimum) / (maximum - minimum)).clamp(0.0, 1.0)
    }
}

fn render_funnel(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let maximum = chart
        .slices
        .iter()
        .map(|(_, value)| value.max(0.0))
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let count = chart.slices.len().max(1);
    let step = plot.height / count as f64;
    let center = plot.x + plot.width / 2.0;
    for (index, (label, value)) in chart.slices.iter().enumerate() {
        let next = chart
            .slices
            .get(index + 1)
            .map(|(_, value)| value.max(0.0))
            .unwrap_or(value.max(0.0) * 0.72);
        let top_width = (value.max(0.0) / maximum * plot.width * 0.82).max(8.0);
        let bottom_width = (next / maximum * plot.width * 0.82).max(8.0);
        let y1 = plot.y + index as f64 * step + 2.0;
        let y2 = y1 + step - 4.0;
        let points = format!(
            "{:.2},{y1:.2} {:.2},{y1:.2} {:.2},{y2:.2} {:.2},{y2:.2}",
            center - top_width / 2.0,
            center + top_width / 2.0,
            center + bottom_width / 2.0,
            center - bottom_width / 2.0
        );
        let _ = write!(
            svg,
            r##"<polygon points="{points}" fill="{}" stroke="#ffffff" stroke-width="2"/>"##,
            chart.color(index)
        );
        let text = format!("{} · {}", truncate_label(label, 22), format_tick(*value));
        let available_width = top_width.min(bottom_width);
        let estimated_text_width = text.chars().count() as f64 * 6.4 + 12.0;
        let middle_y = (y1 + y2) / 2.0;
        if available_width >= estimated_text_width {
            svg_text(
                svg,
                center,
                middle_y + 4.0,
                FONT_LABEL,
                contrast_text_color(chart.color(index)),
                "middle",
                &text,
                true,
            );
        } else {
            let edge = center + top_width.max(bottom_width) / 2.0;
            let label_x = (edge + 20.0).min(plot.x + plot.width - estimated_text_width);
            let _ = write!(
                svg,
                r##"<line x1="{edge:.2}" y1="{middle_y:.2}" x2="{:.2}" y2="{middle_y:.2}" stroke="#94a3b8" stroke-width="1"/>"##,
                label_x - 5.0
            );
            svg_text(
                svg,
                label_x,
                middle_y + 4.0,
                FONT_LABEL,
                "#334155",
                "start",
                &text,
                true,
            );
        }
    }
}

fn render_waffle(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let total: f64 = chart.slices.iter().map(|(_, value)| value.max(0.0)).sum();
    if total <= 0.0 {
        render_fallback(svg, chart, plot);
        return;
    }
    let columns = 10;
    let rows = 10;
    let size = (plot.width / columns as f64)
        .min(plot.height / rows as f64)
        .min(24.0);
    let grid_width = columns as f64 * size;
    let start_x = plot.x + (plot.width - grid_width) / 2.0;
    let start_y = plot.y + (plot.height - rows as f64 * size) / 2.0;
    let mut cutoffs = Vec::new();
    let mut cumulative = 0.0;
    for (_, value) in &chart.slices {
        cumulative += value.max(0.0) / total * (columns * rows) as f64;
        cutoffs.push(cumulative);
    }
    for index in 0..columns * rows {
        let series = cutoffs
            .iter()
            .position(|cutoff| index as f64 + 0.5 <= *cutoff)
            .unwrap_or(cutoffs.len().saturating_sub(1));
        let column = index % columns;
        let row = rows - 1 - index / columns;
        let x = start_x + column as f64 * size;
        let y = start_y + row as f64 * size;
        let _ = write!(
            svg,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="2" fill="{}"/>"##,
            x + 1.0,
            y + 1.0,
            size - 2.0,
            size - 2.0,
            chart.color(series)
        );
    }
}

fn contrast_text_color(color: &str) -> &'static str {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() < 6 {
        return "#ffffff";
    }
    let red = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as u32;
    let green = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as u32;
    let blue = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as u32;
    let brightness = (red * 299 + green * 587 + blue * 114) / 1000;
    if brightness > 120 {
        "#172033"
    } else {
        "#ffffff"
    }
}

fn render_calendar(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let mut cells = chart.calendar.clone();
    cells.sort_by(|a, b| a.day.cmp(&b.day));
    let minimum = cells
        .iter()
        .map(|cell| cell.value)
        .fold(f64::INFINITY, f64::min);
    let maximum = cells
        .iter()
        .map(|cell| cell.value)
        .fold(f64::NEG_INFINITY, f64::max);
    let dated: Option<Vec<(CalendarCell, IsoDate, i64)>> = cells
        .iter()
        .cloned()
        .map(|cell| {
            let date = parse_iso_date(&cell.day)?;
            Some((cell, date, date.serial_day()))
        })
        .collect();
    let Some(mut dated) = dated else {
        render_calendar_sequence(svg, &cells, plot, minimum, maximum);
        return;
    };
    if dated.is_empty() {
        return;
    }
    dated.sort_by_key(|(_, _, serial)| *serial);
    let first_serial = dated[0].2;
    let last_serial = dated.last().map(|entry| entry.2).unwrap_or(first_serial);
    let calendar_start = first_serial - dated[0].1.weekday_from_monday() as i64;
    let weeks = ((last_serial - calendar_start).div_euclid(7) + 1).max(1) as usize;

    let gap = 3.0;
    let day_label_width = 37.0;
    let header_height = 24.0;
    let width_for_cells =
        (plot.width - day_label_width - gap * weeks.saturating_sub(1) as f64).max(8.0);
    let height_for_cells = (plot.height - header_height - gap * 6.0).max(8.0);
    let cell_size = (width_for_cells / weeks as f64)
        .min(height_for_cells / 7.0)
        .clamp(7.0, 24.0);
    let grid_width = cell_size * weeks as f64 + gap * weeks.saturating_sub(1) as f64;
    let grid_height = cell_size * 7.0 + gap * 6.0;
    let total_width = day_label_width + grid_width;
    let grid_x = plot.x + (plot.width - total_width).max(0.0) / 2.0 + day_label_width;
    let grid_y =
        plot.y + (plot.height - header_height - grid_height).max(0.0) / 2.0 + header_height;

    let mut month_positions: Vec<(i32, u32, usize)> = Vec::new();
    for (_, date, serial) in &dated {
        let column = ((*serial - calendar_start).div_euclid(7)) as usize;
        if let Some((_, _, existing_column)) = month_positions
            .iter_mut()
            .find(|(year, month, _)| *year == date.year && *month == date.month)
        {
            *existing_column = (*existing_column).min(column);
        } else {
            month_positions.push((date.year, date.month, column));
        }
    }
    for (year, month, column) in month_positions {
        svg_text(
            svg,
            grid_x + column as f64 * (cell_size + gap),
            grid_y - 9.0,
            FONT_TICK,
            "#64748b",
            "start",
            &format!("{} {year}", month_name(month)),
            false,
        );
    }

    for (calendar, date, serial) in &dated {
        let column = ((*serial - calendar_start).div_euclid(7)) as usize;
        let row = date.weekday_from_monday();
        let x = grid_x + column as f64 * (cell_size + gap);
        let y = grid_y + row as f64 * (cell_size + gap);
        let color = heat_color(calendar.value, minimum, maximum);
        let _ = write!(
            svg,
            r##"<rect x="{x:.1}" y="{y:.1}" width="{cell_size:.1}" height="{cell_size:.1}" rx="2" fill="{color}"><title>{}: {}</title></rect>"##,
            xml_escape(&calendar.day),
            xml_escape(&format_tick(calendar.value))
        );
    }
    for (index, day) in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .iter()
        .enumerate()
    {
        svg_text(
            svg,
            grid_x - 8.0,
            grid_y + (index as f64 + 0.5) * cell_size + index as f64 * gap + 4.0,
            FONT_TICK,
            "#64748b",
            "end",
            day,
            false,
        );
    }
}

fn render_calendar_sequence(
    svg: &mut String,
    cells: &[CalendarCell],
    plot: PlotBox,
    minimum: f64,
    maximum: f64,
) {
    let rows = 7;
    let columns = cells.len().div_ceil(rows).max(1);
    let gap = 3.0;
    let cell_size = ((plot.width - gap * columns.saturating_sub(1) as f64) / columns as f64)
        .min((plot.height - 22.0 - gap * 6.0) / rows as f64)
        .clamp(7.0, 24.0);
    let grid_width = cell_size * columns as f64 + gap * columns.saturating_sub(1) as f64;
    let grid_height = cell_size * rows as f64 + gap * 6.0;
    let start_x = plot.x + (plot.width - grid_width).max(0.0) / 2.0;
    let start_y = plot.y + 22.0 + (plot.height - 22.0 - grid_height).max(0.0) / 2.0;
    for (index, calendar) in cells.iter().enumerate() {
        let column = index / rows;
        let row = index % rows;
        let x = start_x + column as f64 * (cell_size + gap);
        let y = start_y + row as f64 * (cell_size + gap);
        let color = heat_color(calendar.value, minimum, maximum);
        let _ = write!(
            svg,
            r##"<rect x="{x:.1}" y="{y:.1}" width="{cell_size:.1}" height="{cell_size:.1}" rx="2" fill="{color}"><title>{}: {}</title></rect>"##,
            xml_escape(&calendar.day),
            xml_escape(&format_tick(calendar.value))
        );
        if row == 0 {
            svg_text(
                svg,
                x + cell_size / 2.0,
                start_y - 9.0,
                FONT_TICK,
                "#64748b",
                "middle",
                &truncate_label(&calendar.day, 8),
                false,
            );
        }
    }
}

fn parse_iso_date(value: &str) -> Option<IsoDate> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = value.get(0..4)?.parse::<i32>().ok()?;
    let month = value.get(5..7)?.parse::<u32>().ok()?;
    let day = value.get(8..10)?.parse::<u32>().ok()?;
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days_in_month).contains(&day) {
        return None;
    }
    Some(IsoDate { year, month, day })
}

fn month_name(month: u32) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .get(month.saturating_sub(1) as usize)
    .copied()
    .unwrap_or("")
}

fn render_fallback(svg: &mut String, chart: &Chart, plot: PlotBox) {
    let reason = chart
        .fallback_reason
        .as_deref()
        .unwrap_or("This chart could not be represented statically.");
    let _ = write!(
        svg,
        r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="8" fill="#f7f9fc" stroke="#d8dee9"/>"##,
        plot.x, plot.y, plot.width, plot.height
    );
    svg_text(
        svg,
        plot.x + 16.0,
        plot.y + 23.0,
        FONT_LABEL,
        "#64748b",
        "start",
        &truncate_label(reason, 96),
        false,
    );
    let table = &chart.fallback;
    let columns = table.headers.len().clamp(1, 7);
    let column_width = (plot.width - 24.0) / columns as f64;
    let row_height = 24.0;
    let table_x = plot.x + 12.0;
    let table_y = plot.y + 38.0;
    // `table_y` already accounts for the explanatory line and the first
    // 24-pixel slot is the header. Keep every complete data-row slot that
    // remains; subtracting another row here made compact fallback cards show
    // headers with no inspectable data at all.
    let visible_rows = (((plot.height - 52.0) / row_height).floor() as usize).min(10);
    for column in 0..columns {
        let x = table_x + column as f64 * column_width;
        let _ = write!(
            svg,
            r##"<rect x="{x:.1}" y="{table_y:.1}" width="{column_width:.1}" height="{row_height:.1}" fill="#e9eef5" stroke="#d8dee9"/>"##
        );
        svg_text(
            svg,
            x + 7.0,
            table_y + 16.0,
            10.5,
            "#334155",
            "start",
            &truncate_label(
                table
                    .headers
                    .get(column)
                    .map(String::as_str)
                    .unwrap_or("Value"),
                (column_width / 6.0).max(5.0) as usize,
            ),
            true,
        );
    }
    for (row_index, row) in table.rows.iter().take(visible_rows).enumerate() {
        let y = table_y + (row_index + 1) as f64 * row_height;
        for column in 0..columns {
            let x = table_x + column as f64 * column_width;
            let _ = write!(
                svg,
                r##"<rect x="{x:.1}" y="{y:.1}" width="{column_width:.1}" height="{row_height:.1}" fill="{}" stroke="#e2e7ee"/>"##,
                if row_index % 2 == 0 {
                    "#ffffff"
                } else {
                    "#f9fafb"
                }
            );
            svg_text(
                svg,
                x + 7.0,
                y + 16.0,
                10.5,
                "#475569",
                "start",
                &truncate_label(
                    row.get(column).map(String::as_str).unwrap_or(""),
                    (column_width / 6.0).max(5.0) as usize,
                ),
                false,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn svg_text(
    svg: &mut String,
    x: f64,
    y: f64,
    size: f64,
    fill: &str,
    anchor: &str,
    text: &str,
    bold: bool,
) {
    let _ = write!(
        svg,
        r#"<text x="{x:.2}" y="{y:.2}" font-family="Arial, sans-serif" font-size="{size:.1}" font-weight="{}" fill="{}" text-anchor="{}">{}</text>"#,
        if bold { "600" } else { "400" },
        fill,
        anchor,
        xml_escape(text)
    );
}

fn format_tick(value: f64) -> String {
    let magnitude = value.abs();
    if magnitude >= 1_000_000_000.0 {
        format!("{:.1}B", value / 1_000_000_000.0)
    } else if magnitude >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if magnitude >= 1_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else if value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        trim_decimal(format!("{value:.2}"))
    }
}

fn format_axis_tick(value: f64, step: f64) -> String {
    let magnitude = value.abs();
    if magnitude >= 1_000.0 {
        return format_tick(value);
    }
    let decimals = if step.abs() >= 1.0 {
        if (step - step.round()).abs() < 1e-9 {
            0
        } else {
            1
        }
    } else if step.abs() >= 0.1 {
        1
    } else if step.abs() >= 0.01 {
        2
    } else {
        3
    };
    trim_decimal(format!("{value:.decimals$}"))
}

fn trim_decimal(mut value: String) -> String {
    if value.contains('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
    }
    if value == "-0" {
        "0".into()
    } else {
        value
    }
}

fn truncate_label(label: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = label.chars().count();
    if count <= max_chars {
        return label.to_string();
    }
    if max_chars <= 1 {
        return "…".into();
    }
    let mut output: String = label.chars().take(max_chars - 1).collect();
    output.push('…');
    output
}

fn xml_escape(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            character if character.is_control() && !matches!(character, '\t' | '\n') => {
                let _ = write!(output, "&#x{:X};", character as u32);
            }
            character => output.push(character),
        }
    }
    output
}

fn typst_string_literal(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\0' => output.push_str("\\u{0}"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{{{:x}}}", character as u32);
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svg_from_typst(output: &str) -> String {
        assert!(output.starts_with("#image(bytes(\""));
        assert!(output.ends_with(')'));
        let start = "#image(bytes(\"".len();
        let end = output
            .find("\"), format:")
            .expect("bytes literal terminator");
        let encoded = &output[start..end];
        let mut decoded = String::new();
        let mut chars = encoded.chars();
        while let Some(character) = chars.next() {
            if character != '\\' {
                decoded.push(character);
                continue;
            }
            match chars.next().expect("escaped character") {
                '\\' => decoded.push('\\'),
                '"' => decoded.push('"'),
                'n' => decoded.push('\n'),
                'r' => decoded.push('\r'),
                't' => decoded.push('\t'),
                other => panic!("unexpected escape {other:?}"),
            }
        }
        decoded
    }

    #[test]
    fn rejects_non_exact_languages() {
        assert!(chart_to_typst("Nivo", "a,b\nx,1").is_err());
        assert!(chart_to_typst("plotly ", "a,b\nx,1").is_err());
        assert!(chart_to_typst("vega", "a,b\nx,1").is_err());
    }

    #[test]
    fn rejects_empty_and_malformed_json() {
        assert_eq!(
            chart_to_typst("nivo", "  ").unwrap_err(),
            "chart block is empty"
        );
        assert!(chart_to_typst("plotly", "{oops")
            .unwrap_err()
            .contains("invalid JSON"));
        assert!(
            chart_to_typst("nivo", "42").is_ok(),
            "non-JSON text is valid CSV"
        );
    }

    #[test]
    fn bounds_model_generated_chart_input_and_output() {
        let oversized = "x".repeat(MAX_CHART_SOURCE_BYTES + 1);
        assert!(chart_to_typst("nivo", &oversized)
            .unwrap_err()
            .contains("chart block is too large"));

        let mut dense = String::from("type: line\n---\nlabel,value\n");
        for index in 0..=MAX_STATIC_DATA_POINTS {
            let _ = writeln!(dense, "{index},{index}");
        }
        let output = chart_to_typst("nivo", &dense).expect("dense input should degrade safely");
        assert!(svg_from_typst(&output).contains("too large for a static renderer"));
    }

    #[test]
    fn csv_bar_supports_config_crlf_and_escaping() {
        let output = chart_to_typst(
            "nivo",
            "type: bar\r\ntitle: R&D <Q1> \\\"plan\\\"\r\nxLabel: Quarter\r\nyLabel: Revenue\r\ncolors: [#112233, #abcdef]\r\nheight: 410\r\nshowLegend: true\r\nlegendPosition: top\r\nstacked: true\r\nlayout: vertical\r\n---\r\nquarter,A,B\r\nQ1,12,8\r\nQ2,14,9",
        )
        .unwrap();
        assert!(output.contains("format: \"svg\", width: 100%, alt:"));
        assert!(output.contains("R&amp;D &lt;Q1&gt;"));
        assert!(output.contains("#112233"));
        assert!(output.contains("height=\\\"410\\\""));
        assert!(output.contains("Quarter"));
        assert!(output.contains("Revenue"));
        assert!(!output.contains("\r"));
    }

    #[test]
    fn delimiter_must_be_an_exact_line() {
        let parsed = parse_csv_mode("type: bar\n ---\na,b\nx,1").unwrap();
        assert_eq!(parsed.0.chart_type, None);
        assert_eq!(parsed.1.headers[0], "type: bar");
        let parsed = parse_csv_mode("type: bar\n---\na,b\nx,1").unwrap();
        assert_eq!(parsed.0.chart_type.as_deref(), Some("bar"));
        assert_eq!(parsed.1.headers, ["a", "b"]);
    }

    #[test]
    fn config_errors_are_reported() {
        assert!(chart_to_typst("nivo", "type bar\n---\na,b\nx,1").is_err());
        assert!(chart_to_typst("nivo", "height: tall\n---\na,b\nx,1").is_err());
        assert!(chart_to_typst("nivo", "stacked: yes\n---\na,b\nx,1").is_err());
        assert!(chart_to_typst("nivo", "layout: diagonal\n---\na,b\nx,1").is_err());
        assert!(chart_to_typst("nivo", "colors: [red, blue\n---\na,b\nx,1").is_err());
    }

    #[test]
    fn simplistic_csv_does_not_parse_quoted_commas() {
        let (_, table) = parse_csv_mode("name,value\n\"New York, NY\",12").unwrap();
        assert_eq!(table.rows[0], ["\"New York", "NY\"", "12"]);
    }

    #[test]
    fn leaked_fence_lines_are_not_data() {
        let (_, table) = parse_csv_mode("name,value\nA,1\n```\n~~~plotly").unwrap();
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn auto_detects_pie_line_and_bar() {
        let pie = chart_to_typst("nivo", "label,value\nA,2\nB,3").unwrap();
        assert!(svg_from_typst(&pie).contains("<path d="));
        let line = chart_to_typst("plotly", "month,a,b\nJan,1,2\nFeb,2,3").unwrap();
        assert!(svg_from_typst(&line).contains("stroke-linecap=\"round\""));
        let bar = chart_to_typst("nivo", "group,a,b\nEast,1,2\nWest,2,4").unwrap();
        assert!(svg_from_typst(&bar).contains("<rect"));
    }

    #[test]
    fn renders_all_core_csv_chart_types() {
        let cases = [
            ("bar", "label,a,b\nA,1,2\nB,3,4", "<rect"),
            ("line", "label,a,b\nA,1,2\nB,3,4", "<path"),
            ("area", "label,a,b\nA,1,2\nB,3,4", "fill-opacity=\"0.18\""),
            ("pie", "label,value\nA,1\nB,3", "<path"),
            ("radar", "label,a,b\nA,1,2\nB,3,4\nC,2,1", "<polygon"),
            ("heatmap", "label,a,b\nA,1,2\nB,3,4", "#2563eb"),
            ("funnel", "label,value\nLead,100\nWon,30", "<polygon"),
        ];
        for (chart_type, data, needle) in cases {
            let output =
                chart_to_typst("nivo", &format!("type: {chart_type}\n---\n{data}")).unwrap();
            assert!(
                svg_from_typst(&output).contains(needle),
                "{chart_type} missing {needle}"
            );
        }
        let scatter =
            chart_to_typst("nivo", "type: scatter\n---\ngroup,x,y\nA,1,2\nA,2,3\nB,1,4").unwrap();
        assert!(svg_from_typst(&scatter).contains("<circle"));
    }

    #[test]
    fn nivo_aliases_have_static_equivalents() {
        for chart_type in ["bump", "areaBump", "stream", "radialBar"] {
            let output = chart_to_typst(
                "nivo",
                &format!("type: {chart_type}\n---\nlabel,a,b\nA,1,2\nB,3,4"),
            )
            .unwrap();
            let svg = svg_from_typst(&output);
            assert!(!svg.contains("no static renderer"), "{chart_type}");
        }
        let waffle = chart_to_typst("nivo", "type: waffle\n---\nlabel,value\nA,30\nB,70").unwrap();
        assert!(svg_from_typst(&waffle).matches("<rect").count() > 80);
        let calendar = chart_to_typst(
            "nivo",
            "type: calendar\n---\nday,value\n2026-01-01,3\n2026-01-02,8",
        )
        .unwrap();
        assert!(svg_from_typst(&calendar).contains("Jan 2026"));
    }

    #[test]
    fn pie_has_one_configurable_legend_and_a_visible_full_ring() {
        let visible = chart_to_typst(
            "nivo",
            "type: pie\nshowLegend: true\nlegendPosition: right\nheight: 260\n---\nlabel,value\nOrganic,60\nPaid,40",
        )
        .unwrap();
        let visible = svg_from_typst(&visible);
        assert_eq!(visible.matches(">Organic</text>").count(), 1);
        assert_eq!(visible.matches(">60%</text>").count(), 1);
        assert_eq!(visible.matches(">40%</text>").count(), 1);
        assert!(visible.contains("r=\"5\" fill=\"#fb562d\""));
        assert!(visible.contains("fill=\"#ffffff\" stroke=\"#e2e8f0\""));

        let hidden = chart_to_typst(
            "nivo",
            "type: pie\nshowLegend: false\nheight: 260\n---\nlabel,value\nOrganic,60\nPaid,40",
        )
        .unwrap();
        let hidden = svg_from_typst(&hidden);
        assert!(!hidden.contains(">Organic</text>"));
        assert_eq!(hidden.matches(">60%</text>").count(), 1);
        assert_eq!(hidden.matches(">40%</text>").count(), 1);

        let full = chart_to_typst(
            "plotly",
            "type: pie\nshowLegend: false\n---\nlabel,value\nOnly,9",
        )
        .unwrap();
        let full = svg_from_typst(&full);
        assert!(full.contains("fill=\"none\" stroke=\"#fb562d\" stroke-width="));
    }

    #[test]
    fn pie_side_legends_split_long_labels_from_single_percentages() {
        for (position, marker_x) in [("left", "29.0"), ("right", "563.0")] {
            let output = chart_to_typst(
                "nivo",
                &format!(
                    "type: pie\nshowLegend: true\nlegendPosition: {position}\nheight: 260\n---\nlabel,value\nInternational Enterprise Accounts,65\nOther,35"
                ),
            )
            .unwrap();
            let svg = svg_from_typst(&output);
            assert!(
                svg.contains(">International Enterpri…</text>"),
                "{position}"
            );
            assert_eq!(svg.matches(">65%</text>").count(), 1, "{position}");
            assert_eq!(svg.matches(">35%</text>").count(), 1, "{position}");
            assert!(!svg.contains(" · "), "{position}");
            assert!(
                svg.contains(&format!("<circle cx=\"{marker_x}\"")),
                "{position}"
            );
        }
    }

    #[test]
    fn dense_pie_side_legend_compacts_inside_a_short_plot() {
        let plot = PlotBox {
            x: 260.0,
            y: 67.0,
            width: 512.0,
            height: 133.0,
        };
        let mut chart = Chart::from_config("Nivo", "pie".into(), Config::default());
        chart.kind = Kind::Pie;
        let visible = 10;
        let (first_baseline, row_gap) = side_legend_layout(&chart, plot, visible);
        let last_baseline = first_baseline + (visible - 1) as f64 * row_gap;

        assert!(row_gap < 27.0, "dense rows should compact");
        assert!(first_baseline - 10.0 >= plot.y - f64::EPSILON);
        assert!(last_baseline + 4.0 <= plot.y + plot.height + f64::EPSILON);
    }

    #[test]
    fn pie_arc_labels_skip_slices_that_cannot_hold_text() {
        let output = chart_to_typst(
            "nivo",
            "type: pie\nshowLegend: false\nheight: 260\n---\nlabel,value\nLarge,95\nTiny,5",
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert!(svg.contains(">95%</text>"));
        assert!(!svg.contains(">5%</text>"));
    }

    #[test]
    fn legend_uses_effective_plotly_series_color() {
        let output = chart_to_typst(
            "plotly",
            r##"{"data":[{"type":"bar","name":"Plan","x":["Q1","Q2"],"y":[2,3],"marker":{"color":"#7c3aed"}},{"type":"bar","name":"Actual","x":["Q1","Q2"],"y":[3,4],"marker":{"color":"#06b6d4"}}],"layout":{"showlegend":true}}"##,
        )
        .unwrap();
        assert!(
            svg_from_typst(&output).matches("fill=\"#7c3aed\"").count() >= 3,
            "both bars and their legend swatch should use the trace color"
        );
    }

    #[test]
    fn numeric_x_axes_have_readable_ticks() {
        let output = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"scatter","mode":"markers","x":[100,150,200],"y":[2,4,3]}]}"#,
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert!(svg.contains(">100</text>"));
        assert!(svg.contains(">200</text>"));
    }

    #[test]
    fn plotly_radar_drops_the_repeated_closing_axis() {
        let output = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"scatterpolar","r":[72,84,68,79,72],"theta":["Quality","Speed","Coverage","Usability","Quality"]}]}"#,
        )
        .unwrap();
        assert_eq!(
            svg_from_typst(&output).matches(">Quality</text>").count(),
            1
        );
    }

    #[test]
    fn iso_calendar_uses_real_weekdays_and_month_labels() {
        let date = parse_iso_date("2026-07-01").expect("valid date");
        assert_eq!(date.weekday_from_monday(), 2, "2026-07-01 is Wednesday");
        assert!(parse_iso_date("2026-02-29").is_none());
        assert!(parse_iso_date("2024-02-29").is_some());

        let output = chart_to_typst(
            "nivo",
            "type: calendar\nheight: 250\n---\nday,value\n2026-07-01,4\n2026-07-02,8",
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert!(svg.contains("Jul 2026"));
        assert!(svg.contains("2026-07-01: 4"));
    }

    #[test]
    fn funnel_moves_narrow_labels_outside_and_radial_bar_stacks() {
        let funnel = chart_to_typst(
            "nivo",
            "type: funnel\nshowLegend: false\n---\nstage,value\nVisitors,1000\nCustomers,1",
        )
        .unwrap();
        let funnel = svg_from_typst(&funnel);
        assert!(funnel.contains("Customers · 1"));
        assert!(funnel.contains("stroke=\"#94a3b8\""));

        let (config, table) =
            parse_csv_mode("type: radialBar\n---\ngoal,Current,Remaining\nReliability,84,16")
                .unwrap();
        let radial = chart_from_csv("Nivo", "nivo", config, table);
        assert!(radial.horizontal);
        assert!(radial.stacked);

        let radial = chart_to_typst(
            "nivo",
            "type: radialBar\n---\ngoal,Current,Remaining\nReliability,84,16\nCoverage,72,28",
        )
        .unwrap();
        let radial = svg_from_typst(&radial);
        assert!(radial.contains(">100</text>"));
        assert!(!radial.contains(">120</text>"));
    }

    #[test]
    fn bump_uses_an_inverted_integer_rank_axis() {
        let output = chart_to_typst(
            "nivo",
            "type: bump\n---\nweek,Alpha,Beta\nW1,3,1\nW2,2,3\nW3,1,2",
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert!(svg.contains(">1</text>"));
        assert!(svg.contains(">2</text>"));
        assert!(svg.contains(">3</text>"));
        assert!(!svg.contains(">0</text>"));
        assert!(!svg.contains(">1.5</text>"));
    }

    #[test]
    fn stacked_areas_render_distinct_cumulative_bands() {
        let output = chart_to_typst(
            "plotly",
            "type: area\nstacked: true\n---\nmonth,Documents,Media\nJan,24,12\nFeb,31,15",
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert_eq!(svg.matches("fill-opacity=\"0.46\"").count(), 2);

        let normalized = chart_to_typst(
            "nivo",
            "type: areaBump\n---\nmonth,Enterprise,SMB,Consumer\nJan,44,31,25\nFeb,48,30,22",
        )
        .unwrap();
        let normalized = svg_from_typst(&normalized);
        assert!(normalized.contains(">100</text>"));
        assert!(!normalized.contains(">120</text>"));
    }

    #[test]
    fn unsupported_csv_type_becomes_a_table_card() {
        let output = chart_to_typst(
            "nivo",
            "type: sankey\ntitle: Flow\nheight: 220\n---\nsource,target,value\nA,B,12",
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert!(svg.contains("no static renderer"));
        assert!(svg.contains("source"));
        assert!(svg.contains("target"));
        assert!(svg.contains(">A</text>"));
        assert!(svg.contains(">12</text>"));
        assert!(!svg.contains("source,target,value"));
    }

    #[test]
    fn renders_native_nivo_json_shapes() {
        let bar = chart_to_typst(
            "nivo",
            r##"{"chartType":"bar","title":"Countries","indexBy":"country","keys":["sales","cost"],"colors":["#123456","#abcdef"],"data":[{"country":"USA","sales":12,"cost":5},{"country":"UK","sales":8,"cost":4}]}"##,
        )
        .unwrap();
        assert!(svg_from_typst(&bar).contains("#123456"));

        let line = chart_to_typst(
            "nivo",
            r#"{"chartType":"line","data":[{"id":"A","data":[{"x":"Jan","y":2},{"x":"Feb","y":4}]}]}"#,
        )
        .unwrap();
        assert!(svg_from_typst(&line).contains("stroke-linecap=\"round\""));

        let heatmap = chart_to_typst(
            "nivo",
            r#"{"chartType":"heatmap","data":[{"id":"North","data":[{"x":"Q1","y":12},{"x":"Q2","y":20}]}]}"#,
        )
        .unwrap();
        assert!(svg_from_typst(&heatmap).contains("North"));
    }

    #[test]
    fn renders_native_plotly_trace_shapes() {
        let line = chart_to_typst(
            "plotly",
            r#"{"data":[{"x":["Jan","Feb"],"y":[10,15],"type":"scatter","mode":"lines+markers","name":"Sales"}],"layout":{"title":{"text":"Trend"},"xaxis":{"title":{"text":"Month"}},"yaxis":{"title":"Amount"},"height":420}}"#,
        )
        .unwrap();
        let svg = svg_from_typst(&line);
        assert!(svg.contains("Trend"));
        assert!(svg.contains("Month"));
        assert!(svg.contains("height=\"420\""));

        let pie = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"pie","labels":["A","B"],"values":[3,7]}]}"#,
        )
        .unwrap();
        assert!(svg_from_typst(&pie).contains("70%"));

        let heatmap = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"heatmap","x":["Q1","Q2"],"y":["A","B"],"z":[[1,2],[3,4]]}]}"#,
        )
        .unwrap();
        assert!(svg_from_typst(&heatmap).contains("Q1"));

        let radar = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"scatterpolar","r":[3,5,2],"theta":["A","B","C"],"fill":"toself"}]}"#,
        )
        .unwrap();
        assert!(svg_from_typst(&radar).contains("<polygon"));
    }

    #[test]
    fn unsupported_plotly_trace_becomes_a_card() {
        let output = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"sankey","node":{"label":["A","B"]},"link":{"source":[0],"target":[1],"value":[4]}}],"layout":{"title":"Flow"}}"#,
        )
        .unwrap();
        let svg = svg_from_typst(&output);
        assert!(svg.contains("no static renderer"));
        assert!(svg.contains("sankey"));
    }

    #[test]
    fn malformed_json_config_is_an_error() {
        assert!(chart_to_typst("nivo", r#"{"chartType":3,"data":[]}"#).is_err());
        assert!(chart_to_typst("plotly", r#"{"data":[],"layout":{"showlegend":"yes"}}"#).is_err());
        assert!(chart_to_typst("plotly", r#"{"data":[{"type":5,"x":[1],"y":[2]}]}"#).is_err());
    }

    #[test]
    fn xml_and_typst_literals_are_safe() {
        let output = chart_to_typst(
            "plotly",
            concat!(
                "type: pie\n",
                "title: a \"quote\" & <tag> #[] $ \\\n",
                "---\nname,value\nA&B,2\n<C>,3"
            ),
        )
        .unwrap();
        assert!(output.contains("&amp;"));
        assert!(output.contains("&lt;tag&gt;"));
        assert!(output.contains("\\\"quote\\\""));
        assert!(output.contains("\\\\"));
        let svg = svg_from_typst(&output);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("A&amp;B"));
    }

    #[test]
    fn output_is_deterministic() {
        let source = "type: bar\ncolors: paired\n---\nlabel,a,b\nA,1,2\nB,3,4";
        let first = chart_to_typst("nivo", source).unwrap();
        let second = chart_to_typst("nivo", source).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn emitted_markup_compiles_without_packages() {
        let source = chart_to_typst(
            "plotly",
            r#"{"data":[{"type":"scatter","mode":"lines+markers","x":["Jan","Feb"],"y":[2,5],"name":"R&D"}],"layout":{"title":"A \"safe\" chart"}}"#,
        )
        .unwrap();
        let compiled = crate::shared::compile_project(crate::shared::TypstProject {
            main_path: crate::shared::MAIN_FILE.to_string(),
            source,
            source_folder: None,
            allow_universe_packages: false,
        })
        .expect("generated chart Typst must compile");
        assert_eq!(compiled.document.pages.len(), 1);
    }
}
