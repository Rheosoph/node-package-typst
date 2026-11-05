use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use binstall_tar::Archive;
use ecow::eco_format;
use flate2::read::GzDecoder;
use flow_like_wasm_sdk::*;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    io::Read,
    path::{Component, Path},
    sync::{Arc, Mutex},
};
use tiny_skia as sk;
use typst::diag::{FileError, FileResult, PackageError, Severity, SourceDiagnostic};
use typst::foundations::{Array, Bytes, Dict, Str, Value};
use typst::layout::{
    Abs, Frame, FrameItem, FrameKind, GroupItem, Page, PagedDocument, Point, Transform,
};
use typst::syntax::{
    is_ident as is_typst_syntax_identifier, is_newline, package::PackageSpec, parse_code, FileId,
    Lines, Source, Span, SyntaxKind, VirtualPath,
};
use typst::text::Font;
use typst::visualize::{Paint, RelativeTo, Tiling};
use typst_as_lib::{
    cached_file_resolver::CachedFileResolver, file_resolver::FileResolver, TypstAsLibError,
    TypstEngine,
};

pub const CATEGORY: &str = "Document/Typst";
pub const MAIN_FILE: &str = "main.typ";
pub const DEFAULT_TEMPLATE_FUNCTION: &str = "template";
const TEMPLATE_ENTRY_FILE: &str = "__flowlike_entry.typ";
const TEMPLATE_SHOW_RULE: &str = "#show: __flowlike_template.with(..sys.inputs)";
/// Lines `apply_template_source` puts in front of the document source.
const TEMPLATE_WRAPPER_LINES: usize = 3;
pub const DEFAULT_PNG_PPI: f64 = 144.0;
/// Memory a PNG export may use: half of the 128 MiB "heavy" tier in
/// flow-like.toml, leaving the rest for fonts and the compiled document.
const PNG_MEMORY_BUDGET: u64 = 64 * 1024 * 1024;
/// Estimated bytes per rendered pixel: the RGBA canvas, the copy encode_png
/// makes, a page-sized gradient texture, and the encoded output.
const PNG_BYTES_PER_PIXEL: u64 = 13;
const TYPST_PACKAGE_REPOSITORY_URL: &str = "https://packages.typst.org";
const MAX_UNIVERSE_PACKAGE_BYTES: usize = 128 * 1024 * 1024;
pub const DEFAULT_SOURCE: &str = r#"#set page(paper: "a4", margin: 2cm)
#set text(font: "Libertinus Serif", size: 11pt)

= FlowLike Typst Document

This document was compiled from Typst source inside a FlowLike WASM node.
"#;

pub struct TypstProject {
    pub main_path: String,
    pub source: String,
    pub source_folder: Option<FlowPath>,
    /// Additional project roots searched in order after `source_folder`.
    pub asset_roots: Vec<FlowPath>,
    pub allow_universe_packages: bool,
    /// Additional fonts explicitly supplied through `font_files`.
    pub fonts: Vec<Font>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TypstCompileReport {
    pub success: bool,
    pub format: String,
    pub page_count: u64,
    pub warning_count: u64,
    pub warnings: Vec<String>,
    pub output_bytes: u64,
}

pub struct CompiledTypstDocument {
    pub document: PagedDocument,
    pub warnings: Vec<String>,
}

pub fn add_source_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "source",
        "Source",
        "Main Typst document source",
        VariableType::String,
    )
    .set_default_value(json!(DEFAULT_SOURCE));
}

pub fn add_source_file_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "source_file",
        "Source File",
        "FlowPath to the main .typ source file",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>();
}

pub fn add_source_folder_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "source_folder",
        "Source Folder",
        "FlowPath folder used as the root for relative imports and assets",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>();
}

pub fn add_font_files_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "font_files",
        "Font Files",
        "Optional font file FlowPaths (TTF, OTF, or TTC); select fonts by family name in Typst",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>()
    .set_value_type(ValueType::Array)
    .set_default_value(json!([]));
}

pub fn add_asset_roots_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "asset_roots",
        "Asset Roots",
        "Optional folders searched after Source Folder for imports and assets; first readable match wins",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>()
    .set_value_type(ValueType::Array)
    .set_default_value(json!([]));
}

pub fn add_package_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "allow_universe_packages",
        "Packages",
        "Allow @preview Typst Universe packages via packages.typst.org",
        VariableType::Boolean,
    )
    .set_default_value(json!(true));
}

pub fn add_template_data_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "template_data",
        "Template Data (JSON)",
        "JSON object exposed as template named arguments through sys.inputs",
        VariableType::String,
    )
    .set_default_value(json!("{}"));
}

pub fn add_template_file_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "template_file",
        "Template File",
        "Optional .typ module exporting the template function to import and apply",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>();
}

pub fn add_template_function_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "template_function",
        "Template Function",
        "Exported function name to import when template_file is connected",
        VariableType::String,
    )
    .set_default_value(json!(DEFAULT_TEMPLATE_FUNCTION));
}

pub fn add_output_file_pin(node: &mut NodeDefinition, description: &str) {
    node.add_input_pin("output_file", "Output", description, VariableType::Struct)
        .set_schema::<FlowPath>();
}

pub fn add_pdf_svg_output_file_pins(node: &mut NodeDefinition) {
    node.add_input_pin(
        "pdf_output_file",
        "PDF Output",
        "FlowPath file where the PDF will be written",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>();

    node.add_input_pin(
        "svg_output_file",
        "SVG Output",
        "FlowPath file where the SVG will be written",
        VariableType::Struct,
    )
    .set_schema::<FlowPath>();
}

pub fn add_svg_padding_pin(node: &mut NodeDefinition) {
    node.add_input_pin(
        "padding_pt",
        "Padding",
        "SVG padding in points",
        VariableType::Float,
    )
    .set_default_value(json!(0.0))
    .set_range(0.0, 144.0)
    .set_step(1.0);
}

pub fn add_png_pins(node: &mut NodeDefinition) {
    node.add_input_pin(
        "ppi",
        "PPI",
        "Pixels per inch; 72 renders one pixel per point",
        VariableType::Float,
    )
    .set_default_value(json!(DEFAULT_PNG_PPI))
    .set_range(1.0, 300.0)
    .set_step(1.0);

    node.add_input_pin(
        "page",
        "Page",
        "1-based page to render; 0 stacks all pages vertically",
        VariableType::Integer,
    )
    .set_default_value(json!(1));
}

pub fn add_report_output(node: &mut NodeDefinition) {
    node.add_output_pin(
        "report",
        "Report",
        "Compilation result metadata and warnings",
        VariableType::Struct,
    )
    .set_schema::<TypstCompileReport>()
    .set_enforce_schema(true);
}

pub fn typst_scores() -> NodeScores {
    NodeScores {
        privacy: 10,
        security: 8,
        performance: 6,
        governance: 8,
        reliability: 7,
        cost: 10,
    }
}

pub fn read_source_project(ctx: &Context) -> Result<TypstProject, String> {
    let source = read_source_text(ctx);

    Ok(TypstProject {
        main_path: MAIN_FILE.to_string(),
        source,
        source_folder: read_optional_flow_path(ctx, "source_folder")?,
        asset_roots: read_asset_roots(ctx)?,
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
        fonts: read_font_files(ctx)?,
    })
}

pub fn read_file_project(ctx: &Context) -> Result<TypstProject, String> {
    let source_file: FlowPath = ctx.require_input_as("source_file")?;
    let source_folder = read_optional_flow_path(ctx, "source_folder")?;
    let (main_path, source_folder) = file_project_location(&source_file, source_folder)?;
    let source = source_file
        .get_string(ctx)
        .ok_or_else(|| format!("Failed to read Typst source from {}", source_file.path))?;

    Ok(TypstProject {
        main_path,
        source,
        source_folder: Some(source_folder),
        asset_roots: read_asset_roots(ctx)?,
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
        fonts: read_font_files(ctx)?,
    })
}

/// A project compiled with `template_data`, possibly wrapped in the generated
/// template import and show rule.
pub struct TemplateProject {
    pub project: TypstProject,
    main_file: MainFileLabel,
}

impl From<TypstProject> for TemplateProject {
    fn from(project: TypstProject) -> Self {
        Self {
            project,
            main_file: MainFileLabel::default(),
        }
    }
}

/// How diagnostics name and number lines of the main file.
#[derive(Default)]
struct MainFileLabel {
    /// Shown instead of a generated main file path.
    name: Option<&'static str>,
    /// Generated lines in front of the caller's source.
    generated_lines: usize,
}

pub fn read_template_source_project(ctx: &Context) -> Result<TemplateProject, String> {
    let source = read_template_source_text(ctx);
    let source_folder = read_optional_flow_path(ctx, "source_folder")?;
    let template_file = read_optional_flow_path(ctx, "template_file")?;
    let Some(template_file) = template_file else {
        return Ok(TypstProject {
            main_path: MAIN_FILE.to_string(),
            source,
            source_folder,
            asset_roots: read_asset_roots(ctx)?,
            allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
            fonts: read_font_files(ctx)?,
        }
        .into());
    };

    let source_folder = source_folder
        .or_else(|| template_file.parent())
        .unwrap_or_else(|| empty_root_like(&template_file));

    read_template_project(ctx, source, None, source_folder, template_file)
}

pub fn read_template_file_project(ctx: &Context) -> Result<TemplateProject, String> {
    let template_file = read_optional_flow_path(ctx, "template_file")?;
    let Some(template_file) = template_file else {
        return read_file_project(ctx).map(TemplateProject::from);
    };

    let source_file: FlowPath = ctx.require_input_as("source_file")?;
    let source_folder = read_optional_flow_path(ctx, "source_folder")?
        .or_else(|| source_file.parent())
        .or_else(|| template_file.parent())
        .unwrap_or_else(|| empty_root_like(&source_file));
    let main_path = flow_path_relative_to_root(&source_file, &source_folder, "source_file")?;
    let source = source_file.get_string(ctx).ok_or_else(|| {
        format!(
            "Failed to read Typst document source from {}",
            source_file.path
        )
    })?;

    read_template_project(ctx, source, Some(main_path), source_folder, template_file)
}

fn read_template_project(
    ctx: &Context,
    source: String,
    main_path: Option<String>,
    source_folder: FlowPath,
    template_file: FlowPath,
) -> Result<TemplateProject, String> {
    let template_function = read_template_function(ctx)?;
    let template_path =
        flow_path_relative_to_root(&template_file, &source_folder, "template_file")?;
    let generated_entry = main_path.is_none();
    let main_path = main_path.unwrap_or_else(|| generated_template_entry_path(&template_path));
    if main_path == template_path {
        return Err("source_file and template_file must refer to different files".to_string());
    }
    let import_path = relative_import_path(&main_path, &template_path)?;
    let source = apply_template_source(&import_path, &template_function, &source)?;

    Ok(TemplateProject {
        project: TypstProject {
            main_path,
            source,
            source_folder: Some(source_folder),
            asset_roots: read_asset_roots(ctx)?,
            allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
            fonts: read_font_files(ctx)?,
        },
        main_file: MainFileLabel {
            name: generated_entry.then_some("source"),
            generated_lines: TEMPLATE_WRAPPER_LINES,
        },
    })
}

pub fn compile_project(project: TypstProject) -> Result<CompiledTypstDocument, String> {
    compile(project, None, MainFileLabel::default())
}

/// Compiles with `inputs` exposed as `sys.inputs`.
pub fn compile_template_project(
    template: TemplateProject,
    inputs: Dict,
) -> Result<CompiledTypstDocument, String> {
    compile(template.project, Some(inputs), template.main_file)
}

fn compile(
    project: TypstProject,
    inputs: Option<Dict>,
    main_file: MainFileLabel,
) -> Result<CompiledTypstDocument, String> {
    let main_path = project.main_path;
    let main_id = file_id(&main_path);
    let loaded_files = LoadedFiles::default();
    let diagnostics = DiagnosticSources {
        main_name: main_file
            .name
            .map_or_else(|| main_path.clone(), str::to_string),
        main_line_offset: main_file.generated_lines,
        main: Source::new(main_id, project.source),
        loaded_files: loaded_files.clone(),
    };

    // typst-as-lib reports the error of the last resolver that failed, so the
    // resolver that owns every non-main file must come last.
    let engine = TypstEngine::builder()
        .fonts(
            project
                .fonts
                .into_iter()
                .chain(typst_assets::fonts().flat_map(|bytes| Font::iter(Bytes::new(bytes)))),
        )
        .with_static_source_file_resolver([diagnostics.main.clone()])
        .add_file_resolver(ProjectFileResolver::new(
            &diagnostics.main,
            project.source_folder,
            project.asset_roots,
            project.allow_universe_packages,
            loaded_files,
        ))
        .build();

    let warned = match inputs {
        Some(inputs) => engine.compile_with_input(main_path.as_str(), inputs),
        None => engine.compile(main_path.as_str()),
    };
    let document: PagedDocument = warned
        .output
        .map_err(|error| diagnostics.compile_error(&error))?;
    let warnings = warned
        .warnings
        .iter()
        .map(|warning| diagnostics.format(warning))
        .collect();

    Ok(CompiledTypstDocument { document, warnings })
}

/// Sources needed to render diagnostic spans as `path:line:column`.
struct DiagnosticSources {
    main: Source,
    main_name: String,
    main_line_offset: usize,
    loaded_files: LoadedFiles,
}

impl DiagnosticSources {
    fn compile_error(&self, error: &TypstAsLibError) -> String {
        let details = match error {
            TypstAsLibError::TypstSource(diagnostics) => diagnostics
                .iter()
                .map(|diagnostic| self.format(diagnostic))
                .collect::<Vec<_>>()
                .join("\n"),
            TypstAsLibError::HintedString(hinted) => {
                let mut text = format!("error: {}", hinted.message());
                push_hints(&mut text, hinted.hints());
                text
            }
            error => format!("error: {error}"),
        };

        format!("Typst compilation failed:\n{details}")
    }

    fn format(&self, diagnostic: &SourceDiagnostic) -> String {
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let mut text = match self.location(diagnostic.span) {
            Some(location) => format!("{location}: {severity}: {}", diagnostic.message),
            None => format!("{severity}: {}", diagnostic.message),
        };

        for point in &diagnostic.trace {
            match self.location(point.span) {
                Some(location) => text.push_str(&format!("\n  {location}: {}", point.v)),
                None => text.push_str(&format!("\n  {}", point.v)),
            }
        }
        push_hints(&mut text, &diagnostic.hints);
        text
    }

    fn location(&self, span: Span) -> Option<String> {
        let id = span.id()?;
        let is_main = id == self.main.id();
        let path = if is_main {
            self.main_name.clone()
        } else {
            diagnostic_path(id)
        };

        let position = if let Some(range) = span.range() {
            // Data files (JSON, TOML, SVG, ...) report byte ranges into the
            // loaded bytes, which include any byte order mark.
            self.loaded_files.lock().ok().and_then(|files| {
                let text = std::str::from_utf8(files.get(&id)?).ok()?;
                Lines::new(text.to_string()).byte_to_line_column(range.start)
            })
        } else if is_main {
            match line_and_column(&self.main, span) {
                Some((line, column)) => match line.checked_sub(self.main_line_offset) {
                    Some(line) => Some((line, column)),
                    None => return Some(format!("{path} (generated template import)")),
                },
                None => None,
            }
        } else {
            self.loaded_files
                .lock()
                .ok()
                .and_then(|files| source_from_bytes(id, files.get(&id)?).ok())
                .and_then(|source| line_and_column(&source, span))
        };

        Some(match position {
            Some((line, column)) => format!("{path}:{}:{}", line + 1, column + 1),
            None => path,
        })
    }
}

fn push_hints(text: &mut String, hints: &[ecow::EcoString]) {
    for hint in hints {
        text.push_str(&format!("\n  hint: {hint}"));
    }
}

/// Zero-based line and column where `span` starts in `source`.
fn line_and_column(source: &Source, span: Span) -> Option<(usize, usize)> {
    let range = source.range(span)?;
    source.lines().byte_to_line_column(range.start)
}

fn diagnostic_path(id: FileId) -> String {
    let path = id.vpath().as_rootless_path().display();
    match id.package() {
        Some(package) => format!("{package}/{path}"),
        None => path.to_string(),
    }
}

pub fn read_template_inputs(ctx: &Context) -> Result<Dict, String> {
    let raw = ctx.get_string("template_data").unwrap_or_default();
    let raw = if raw.trim().is_empty() {
        "{}"
    } else {
        raw.trim()
    };

    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|error| format!("template_data must be valid JSON: {error}"))?;

    let serde_json::Value::Object(object) = value else {
        return Err("template_data must be a JSON object".to_string());
    };

    let mut inputs = Dict::new();
    for (key, value) in object {
        inputs.insert(Str::from(key), json_value_to_typst(value)?);
    }

    Ok(inputs)
}

fn json_value_to_typst(value: serde_json::Value) -> Result<Value, String> {
    match value {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(value) => Ok(Value::Bool(value)),
        serde_json::Value::Number(value) => json_number_to_typst(value),
        serde_json::Value::String(value) => Ok(Value::Str(Str::from(value))),
        serde_json::Value::Array(values) => {
            let mut array = Array::new();
            for value in values {
                array.push(json_value_to_typst(value)?);
            }
            Ok(Value::Array(array))
        }
        serde_json::Value::Object(values) => {
            let mut dict = Dict::new();
            for (key, value) in values {
                dict.insert(Str::from(key), json_value_to_typst(value)?);
            }
            Ok(Value::Dict(dict))
        }
    }
}

fn json_number_to_typst(value: serde_json::Number) -> Result<Value, String> {
    if let Some(value) = value.as_i64() {
        return Ok(Value::Int(value));
    }

    if let Some(unsigned_value) = value.as_u64() {
        return i64::try_from(unsigned_value)
            .map(Value::Int)
            .or_else(|_| value.as_f64().map(Value::Float).ok_or_else(number_error));
    }

    value.as_f64().map(Value::Float).ok_or_else(number_error)
}

fn number_error() -> String {
    "template_data contains a number that cannot be represented in Typst".to_string()
}

fn read_source_text(ctx: &Context) -> String {
    let source = ctx.get_string("source").unwrap_or_default();
    if source.trim().is_empty() {
        DEFAULT_SOURCE.to_string()
    } else {
        source
    }
}

fn read_template_source_text(ctx: &Context) -> String {
    ctx.get_string("source")
        .unwrap_or_else(|| DEFAULT_SOURCE.to_string())
}

fn read_optional_flow_path(ctx: &Context, pin_name: &str) -> Result<Option<FlowPath>, String> {
    match ctx.get_input(pin_name) {
        None => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|error| format!("Failed to deserialize input '{pin_name}': {error}")),
    }
}

fn read_asset_roots(ctx: &Context) -> Result<Vec<FlowPath>, String> {
    let Some(value) = ctx.get_input("asset_roots") else {
        return Ok(Vec::new());
    };
    let mut roots: Vec<FlowPath> = serde_json::from_value(value.clone())
        .map_err(|error| format!("Failed to deserialize input 'asset_roots': {error}"))?;
    for (index, root) in roots.iter_mut().enumerate() {
        root.path = normalized_flow_path(&root.path, &format!("asset_roots[{index}]"), true)?;
    }
    Ok(roots)
}

fn read_font_files(ctx: &Context) -> Result<Vec<Font>, String> {
    let Some(value) = ctx.get_input("font_files") else {
        return Ok(Vec::new());
    };
    let files: Vec<FlowPath> = serde_json::from_value(value.clone())
        .map_err(|error| format!("Failed to deserialize input 'font_files': {error}"))?;
    load_font_files(&files, |file| file.read(ctx))
}

/// Reads explicit file handles independently of the Typst project root.
fn load_font_files(
    files: &[FlowPath],
    mut read: impl FnMut(&FlowPath) -> Option<Vec<u8>>,
) -> Result<Vec<Font>, String> {
    let mut fonts = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let pin_name = format!("font_files[{index}]");
        normalized_flow_path(&file.path, &pin_name, false)?;
        let bytes = read(file).ok_or_else(|| {
            format!(
                "{pin_name}: failed to read font at FlowLike path {:?}",
                file.path
            )
        })?;
        let parsed: Vec<_> = Font::iter(Bytes::new(bytes)).collect();
        if parsed.is_empty() {
            return Err(format!(
                "{pin_name}: {:?} contains no supported fonts; supply a valid TTF, OTF, or TTC file",
                file.path
            ));
        }
        fonts.extend(parsed);
    }
    Ok(fonts)
}

fn file_project_location(
    source_file: &FlowPath,
    source_folder: Option<FlowPath>,
) -> Result<(String, FlowPath), String> {
    let source_folder = source_folder
        .or_else(|| source_file.parent())
        .unwrap_or_else(|| empty_root_like(source_file));
    let main_path = flow_path_relative_to_root(source_file, &source_folder, "source_file")?;
    Ok((main_path, source_folder))
}

fn read_template_function(ctx: &Context) -> Result<String, String> {
    let function = ctx
        .get_string("template_function")
        .unwrap_or_else(|| DEFAULT_TEMPLATE_FUNCTION.to_string());
    let function = function.trim();
    if function.is_empty() {
        return Ok(DEFAULT_TEMPLATE_FUNCTION.to_string());
    }

    if is_typst_identifier(function) {
        Ok(function.to_string())
    } else {
        Err(format!(
            "template_function must be a Typst identifier, got {function:?}"
        ))
    }
}

fn is_typst_identifier(name: &str) -> bool {
    if !is_typst_syntax_identifier(name) {
        return false;
    }

    let parsed = parse_code(name);
    let mut children = parsed.children();
    matches!(
        children.next(),
        Some(child) if child.kind() == SyntaxKind::Ident && child.text().as_str() == name
    ) && children.next().is_none()
}

fn apply_template_source(
    template_path: &str,
    template_function: &str,
    source: &str,
) -> Result<String, String> {
    let template_path = typst_string_literal(template_path)?;

    Ok(format!(
        "#import {template_path}: {template_function} as __flowlike_template\n\
         {TEMPLATE_SHOW_RULE}\n\n\
         {source}"
    ))
}

fn typst_string_literal(value: &str) -> Result<String, String> {
    // Line breaks would also shift every reported line of the wrapped source.
    if value
        .chars()
        .any(|char| char.is_control() || is_newline(char))
    {
        return Err(
            "Typst import paths cannot contain control characters or line breaks".to_string(),
        );
    }

    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!("\"{escaped}\""))
}

fn flow_path_relative_to_root(
    file: &FlowPath,
    root: &FlowPath,
    pin_name: &str,
) -> Result<String, String> {
    if file.store_ref != root.store_ref || file.cache_store_ref != root.cache_store_ref {
        return Err(format!(
            "{pin_name} must use the same FlowPath store as source_folder"
        ));
    }

    let file_path = normalized_flow_path(&file.path, pin_name, false)?;
    let root_path = normalized_flow_path(&root.path, "source_folder", true)?;

    if root_path.is_empty() {
        return Ok(file_path);
    }

    let prefix = format!("{root_path}/");
    file_path
        .strip_prefix(&prefix)
        .map(|relative| relative.to_string())
        .filter(|relative| !relative.is_empty())
        .ok_or_else(|| {
            format!(
                "{pin_name} ({}) must be inside source_folder ({})",
                file.path, root.path
            )
        })
}

fn normalized_flow_path(path: &str, pin_name: &str, allow_empty: bool) -> Result<String, String> {
    if Path::new(path).is_absolute() {
        return Err(format!("{pin_name} must be a relative FlowPath"));
    }

    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return if allow_empty {
            Ok(String::new())
        } else {
            Err(format!("{pin_name} must not be empty"))
        };
    }

    let path = Path::new(trimmed);

    let mut normalized = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(component) => {
                let component = component.to_str().ok_or_else(|| {
                    format!("{pin_name} contains a path component that is not valid UTF-8")
                })?;
                normalized.push(component);
            }
            _ => {
                return Err(format!(
                    "{pin_name} must not contain . or .. path components"
                ));
            }
        }
    }

    Ok(normalized.join("/"))
}

fn relative_import_path(from_main_path: &str, to_file_path: &str) -> Result<String, String> {
    let from = path_components(from_main_path, "main Typst source path")?;
    let to = path_components(to_file_path, "template_file")?;
    if to.is_empty() {
        return Err("template_file must not be empty".to_string());
    }

    let from_dir_len = from.len().saturating_sub(1);
    let from_dir = &from[..from_dir_len];
    let mut common = 0;
    while common < from_dir.len() && common < to.len() && from_dir[common] == to[common] {
        common += 1;
    }

    let mut relative = Vec::new();
    for _ in common..from_dir.len() {
        relative.push("..".to_string());
    }
    relative.extend(to[common..].iter().cloned());

    if relative.is_empty() {
        Err("template import path must not be empty".to_string())
    } else {
        Ok(relative.join("/"))
    }
}

fn generated_template_entry_path(template_path: &str) -> String {
    if template_path != TEMPLATE_ENTRY_FILE {
        return TEMPLATE_ENTRY_FILE.to_string();
    }

    "__flowlike_entry_1.typ".to_string()
}

fn path_components(path: &str, label: &str) -> Result<Vec<String>, String> {
    normalized_flow_path(path, label, false).map(|path| {
        path.split('/')
            .filter(|component| !component.is_empty())
            .map(|component| component.to_string())
            .collect()
    })
}

fn empty_root_like(path: &FlowPath) -> FlowPath {
    FlowPath {
        path: String::new(),
        store_ref: path.store_ref.clone(),
        cache_store_ref: path.cache_store_ref.clone(),
    }
}

pub fn export_pdf(compiled: &CompiledTypstDocument) -> Result<Vec<u8>, String> {
    let options = typst_pdf::PdfOptions::default();
    typst_pdf::pdf(&compiled.document, &options)
        .map_err(|error| format!("Typst PDF export failed: {error:?}"))
}

pub fn export_svg(compiled: &CompiledTypstDocument, padding_pt: f64) -> String {
    typst_svg::svg_merged(&compiled.document, Abs::pt(padding_pt.max(0.0)))
}

pub struct PngOptions {
    pub ppi: f64,
    /// 1-based page number, or `None` to stack every page vertically.
    pub page: Option<u64>,
}

pub fn read_png_options(ctx: &Context) -> Result<PngOptions, String> {
    let ppi = ctx.get_f64("ppi").unwrap_or(DEFAULT_PNG_PPI);
    if !ppi.is_finite() || ppi < 1.0 {
        return Err(format!("ppi must be at least 1, got {ppi}"));
    }

    let page = match ctx.get_i64("page").unwrap_or(1) {
        0 => None,
        page if page > 0 => Some(page.unsigned_abs()),
        page => return Err(format!("page must be 0 or a page number, got {page}")),
    };

    Ok(PngOptions { ppi, page })
}

pub fn export_png(
    compiled: &CompiledTypstDocument,
    options: &PngOptions,
) -> Result<Vec<u8>, String> {
    let pages = &compiled.document.pages;
    let pixel_per_pt = (options.ppi / 72.0) as f32;
    let selected = match options.page {
        Some(page) => usize::try_from(page - 1)
            .ok()
            .and_then(|index| pages.get(index..=index))
            .ok_or_else(|| {
                format!(
                    "page {page} does not exist; the document has {} page(s)",
                    pages.len()
                )
            })?,
        None => pages.as_slice(),
    };
    if selected.is_empty() {
        return Err("the document has no pages to render".to_string());
    }

    check_png_memory(selected, options.ppi, pixel_per_pt)?;
    // typst-render panics instead of returning errors, and a panic aborts the
    // whole WASM instance, so reject what it cannot draw before rendering.
    for page in selected {
        let check = RenderCheck::new(pixel_per_pt);
        if let Some(fill) = page.fill_or_white() {
            check.paint(&fill, false)?;
        }
        check.frame(&page.frame)?;
    }

    let pixmap = match selected {
        [page] => typst_render::render(page, pixel_per_pt),
        _ => typst_render::render_merged(&compiled.document, pixel_per_pt, Abs::zero(), None),
    };
    pixmap
        .encode_png()
        .map_err(|error| format!("Typst PNG export failed: {error}"))
}

fn check_png_memory(pages: &[Page], ppi: f64, pixel_per_pt: f32) -> Result<(), String> {
    let sizes: Vec<(u64, u64)> = pages
        .iter()
        .map(|page| {
            let size = page.frame.size();
            (
                png_pixels(size.x, pixel_per_pt),
                png_pixels(size.y, pixel_per_pt),
            )
        })
        .collect();
    let mut pixels = sizes.iter().fold(0u64, |total, (width, height)| {
        total.saturating_add(width.saturating_mul(*height))
    });
    let mut area_pt = pages.iter().map(|page| page_area_pt(page)).sum::<f64>();
    if pages.len() > 1 {
        // Stacking keeps every page image while it copies them onto one canvas.
        let width = sizes.iter().map(|(width, _)| *width).max().unwrap_or(0);
        let height = sizes
            .iter()
            .fold(0u64, |total, (_, height)| total.saturating_add(*height));
        pixels = pixels.saturating_add(width.saturating_mul(height));
        let width_pt = pages
            .iter()
            .map(|page| page.frame.width().to_pt())
            .fold(0.0, f64::max);
        let height_pt = pages
            .iter()
            .map(|page| page.frame.height().to_pt())
            .sum::<f64>();
        area_pt += width_pt * height_pt;
    }

    let bytes = pixels.saturating_mul(PNG_BYTES_PER_PIXEL);
    if bytes <= PNG_MEMORY_BUDGET {
        return Ok(());
    }

    let max_ppi = 72.0 * (PNG_MEMORY_BUDGET as f64 / (PNG_BYTES_PER_PIXEL as f64 * area_pt)).sqrt();
    let advice = if pages.len() > 1 {
        format!(
            "render a single page or use at most {} ppi",
            max_ppi.floor()
        )
    } else {
        format!("use at most {} ppi", max_ppi.floor())
    };
    Err(format!(
        "a PNG at {ppi} ppi needs about {} MB, more than the {} MB available; {advice}",
        bytes / (1024 * 1024),
        PNG_MEMORY_BUDGET / (1024 * 1024)
    ))
}

fn page_area_pt(page: &Page) -> f64 {
    page.frame.width().to_pt() * page.frame.height().to_pt()
}

/// Pixel length typst-render uses for a page edge.
fn png_pixels(length: Abs, pixel_per_pt: f32) -> u64 {
    (pixel_per_pt * length.to_pt() as f32).round().max(1.0) as u64
}

/// Walks a frame the way typst-render 0.14 does and fails where the renderer
/// would unwrap a `None`: tiling textures smaller than a pixel, text tilings
/// sampled outside their texture, and transforms that cannot be inverted.
#[derive(Clone, Copy)]
struct RenderCheck {
    pixel_per_pt: f32,
    transform: sk::Transform,
    container: sk::Transform,
}

impl RenderCheck {
    fn new(pixel_per_pt: f32) -> Self {
        let transform = sk::Transform::from_scale(pixel_per_pt, pixel_per_pt);
        Self {
            pixel_per_pt,
            transform,
            container: transform,
        }
    }

    fn frame(self, frame: &Frame) -> Result<(), String> {
        for (pos, item) in frame.items() {
            match item {
                FrameItem::Group(group) => self.group(*pos, group)?,
                FrameItem::Shape(shape, _) => {
                    let check = self.translate(*pos);
                    if let Some(fill) = &shape.fill {
                        check.paint(fill, false)?;
                    }
                    if let Some(stroke) = &shape.stroke {
                        check.paint(&stroke.paint, false)?;
                    }
                }
                FrameItem::Text(text) => {
                    let check = self.translate(*pos);
                    check.paint(&text.fill, true)?;
                    if let Some(stroke) = &text.stroke {
                        check.paint(&stroke.paint, true)?;
                    }
                }
                FrameItem::Image(..) | FrameItem::Link(..) | FrameItem::Tag(_) => {}
            }
        }
        Ok(())
    }

    fn group(self, pos: Point, group: &GroupItem) -> Result<(), String> {
        let group_transform = sk_transform(&group.transform);
        let container = match group.frame.kind() {
            FrameKind::Soft => self.container,
            FrameKind::Hard => self
                .container
                .pre_concat(self.transform.post_concat(invert(self.container)?))
                .pre_concat(sk_transform(&Transform::translate(pos.x, pos.y)))
                .pre_concat(group_transform),
        };
        let check = Self {
            transform: self.translate(pos).transform.pre_concat(group_transform),
            container,
            ..self
        };
        check.frame(&group.frame)
    }

    fn translate(self, pos: Point) -> Self {
        Self {
            transform: self
                .transform
                .pre_translate(pos.x.to_pt() as f32, pos.y.to_pt() as f32),
            ..self
        }
    }

    fn paint(self, paint: &Paint, on_text: bool) -> Result<(), String> {
        let relative = match paint {
            Paint::Solid(_) => return Ok(()),
            Paint::Gradient(gradient) => gradient.unwrap_relative(on_text),
            Paint::Tiling(tiling) => tiling.unwrap_relative(on_text),
        };
        if relative == RelativeTo::Parent {
            invert(self.transform)?;
            invert(self.container)?;
        }
        match paint {
            Paint::Tiling(tiling) => self.tiling(tiling, on_text),
            _ => Ok(()),
        }
    }

    fn tiling(self, tiling: &Tiling, on_text: bool) -> Result<(), String> {
        let ppi = self.pixel_per_pt * 72.0;
        let size = tiling.size() + tiling.spacing();
        for extent in [size.x, size.y] {
            let texture = (extent.to_pt() as f32 * self.pixel_per_pt).round();
            if texture < 1.0 {
                return Err(format!(
                    "a tiling tile is smaller than one pixel at {ppi} ppi; raise ppi or enlarge the tile"
                ));
            }
            // Text samples the texture up to ceil(extent) - 1, but it is only
            // round(extent) pixels wide.
            let sampled = ((extent.to_pt() * self.pixel_per_pt as f64) as f32).ceil();
            if on_text && sampled > texture {
                return Err(format!(
                    "text filled with a tiling cannot be rendered at {ppi} ppi because the tile is not a whole number of pixels; change ppi or the tile size"
                ));
            }
        }
        // Tiles render their own frame from a fresh transform.
        Self::new(self.pixel_per_pt).frame(tiling.frame())
    }
}

fn invert(transform: sk::Transform) -> Result<sk::Transform, String> {
    transform.invert().ok_or_else(|| {
        "the document uses a transform the PNG renderer cannot invert, such as a skew or rotation that collapses content to zero size".to_string()
    })
}

/// Same conversion as typst-render's.
fn sk_transform(transform: &Transform) -> sk::Transform {
    let Transform {
        sx,
        ky,
        kx,
        sy,
        tx,
        ty,
    } = *transform;
    sk::Transform::from_row(
        sx.get() as f32,
        ky.get() as f32,
        kx.get() as f32,
        sy.get() as f32,
        tx.to_pt() as f32,
        ty.to_pt() as f32,
    )
}

pub fn report(
    compiled: &CompiledTypstDocument,
    format: &str,
    output_bytes: usize,
) -> TypstCompileReport {
    TypstCompileReport {
        success: true,
        format: format.to_string(),
        page_count: compiled.document.pages.len() as u64,
        warning_count: compiled.warnings.len() as u64,
        warnings: compiled.warnings.clone(),
        output_bytes: output_bytes as u64,
    }
}

pub fn failure_report(format: &str, error: &str) -> TypstCompileReport {
    TypstCompileReport {
        success: false,
        format: format.to_string(),
        page_count: 0,
        warning_count: 0,
        warnings: vec![error.to_string()],
        output_bytes: 0,
    }
}

pub fn read_output_file(ctx: &Context) -> Result<FlowPath, String> {
    read_flow_path_pin(ctx, "output_file")
}

pub fn read_flow_path_pin(ctx: &Context, pin_name: &str) -> Result<FlowPath, String> {
    ctx.require_input_as(pin_name)
}

pub fn write_output(ctx: &Context, output_file: &FlowPath, bytes: &[u8]) -> Result<(), String> {
    if output_file.put(ctx, bytes) {
        Ok(())
    } else {
        Err(format!(
            "Failed to write Typst output to {}",
            output_file.path
        ))
    }
}

fn file_id(path: &str) -> FileId {
    FileId::new(None, VirtualPath::new(path))
}

/// Raw bytes of every project and package file loaded during a compilation,
/// kept so diagnostics can be mapped back to lines and columns.
type LoadedFiles = Arc<Mutex<HashMap<FileId, Vec<u8>>>>;

/// Sends every file except the main source to the one resolver that owns it,
/// so its error is the one reported instead of a generic "file not found".
struct ProjectFileResolver {
    project: Option<CachedFileResolver<FlowPathProjectResolver>>,
    packages: Option<CachedFileResolver<FlowLikeUniversePackageResolver>>,
    /// The main source when it exists only in memory, for `read()` of itself.
    in_memory_main: Option<Source>,
}

impl ProjectFileResolver {
    fn new(
        main: &Source,
        source_folder: Option<FlowPath>,
        asset_roots: Vec<FlowPath>,
        allow_universe_packages: bool,
        loaded_files: LoadedFiles,
    ) -> Self {
        Self {
            in_memory_main: source_folder.is_none().then(|| main.clone()),
            project: (source_folder.is_some() || !asset_roots.is_empty()).then(|| {
                cached(FlowPathProjectResolver::new(
                    source_folder,
                    asset_roots,
                    loaded_files.clone(),
                ))
            }),
            packages: allow_universe_packages
                .then(|| cached(FlowLikeUniversePackageResolver::new(loaded_files))),
        }
    }

    fn resolver_for(&self, id: FileId) -> FileResult<&dyn FileResolver> {
        match id.package() {
            Some(package) if package.namespace != "preview" => {
                Err(unsupported_package_namespace(package))
            }
            Some(package) => self
                .packages
                .as_ref()
                .map(|resolver| resolver as &dyn FileResolver)
                .ok_or_else(|| universe_packages_disabled(package)),
            None => self
                .project
                .as_ref()
                .map(|resolver| resolver as &dyn FileResolver)
                .ok_or_else(|| missing_source_folder(id)),
        }
    }
}

impl FileResolver for ProjectFileResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        if let Some(main) = self.in_memory_main.as_ref().filter(|main| main.id() == id) {
            return Ok(Cow::Owned(Bytes::from_string(main.text().to_owned())));
        }
        self.resolver_for(id)?.resolve_binary(id)
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        self.resolver_for(id)?.resolve_source(id)
    }
}

fn cached<T>(resolver: T) -> CachedFileResolver<T> {
    CachedFileResolver::new(resolver)
        .with_in_memory_source_cache()
        .with_in_memory_binary_cache()
}

fn missing_source_folder(id: FileId) -> FileError {
    FileError::Other(Some(eco_format!(
        "source_folder is not connected, so {} cannot be loaded; connect source_folder or asset_roots to folders that contain it",
        id.vpath().as_rootless_path().display()
    )))
}

fn unsupported_package_namespace(package: &PackageSpec) -> FileError {
    FileError::Package(PackageError::Other(Some(eco_format!(
        "only @preview packages can be downloaded, so {package} is not available"
    ))))
}

fn universe_packages_disabled(package: &PackageSpec) -> FileError {
    FileError::Package(PackageError::Other(Some(eco_format!(
        "{package} needs Typst Universe, but allow_universe_packages is disabled"
    ))))
}

struct FlowPathProjectResolver {
    source_folder: Option<FlowPath>,
    asset_roots: Vec<FlowPath>,
    files: LoadedFiles,
}

impl FlowPathProjectResolver {
    fn new(
        source_folder: Option<FlowPath>,
        asset_roots: Vec<FlowPath>,
        files: LoadedFiles,
    ) -> Self {
        Self {
            source_folder,
            asset_roots,
            files,
        }
    }

    fn resolve_project_file(&self, id: FileId) -> FileResult<Vec<u8>> {
        self.resolve_project_file_with(id, |path| {
            let json = serde_json::to_string(path)
                .map_err(|error| FileError::Other(Some(eco_format!("{error}"))))?;
            Ok(flow_like_wasm_sdk::host::storage_read(&json))
        })
    }

    fn resolve_project_file_with(
        &self,
        id: FileId,
        mut read: impl FnMut(&FlowPath) -> FileResult<Option<Vec<u8>>>,
    ) -> FileResult<Vec<u8>> {
        if id.package().is_some() {
            return Err(file_not_found(id));
        }

        if let Some(cached) = self
            .files
            .lock()
            .map_err(|_| FileError::Other(Some(eco_format!("Could not lock source cache"))))?
            .get(&id)
            .cloned()
        {
            return Ok(cached);
        }

        let relative_path = flow_relative_path(id)?;
        let roots = self
            .source_folder
            .iter()
            .map(|root| ("source_folder".to_string(), root))
            .chain(
                self.asset_roots
                    .iter()
                    .enumerate()
                    .map(|(index, root)| (format!("asset_roots[{index}]"), root)),
            );
        let mut attempted = Vec::new();
        for (label, root) in roots {
            let path = root.child(&relative_path);
            if let Some(bytes) = read(&path)? {
                self.files
                    .lock()
                    .map_err(|_| {
                        FileError::Other(Some(eco_format!("Could not lock source cache")))
                    })?
                    .insert(id, bytes.clone());
                return Ok(bytes);
            }
            attempted.push(format!("{label} at FlowLike path {:?}", path.path));
        }
        if attempted.is_empty() {
            return Err(missing_source_folder(id));
        }
        Err(FileError::Other(Some(eco_format!(
            "{relative_path} could not be read from {}",
            attempted.join("; ")
        ))))
    }
}

impl FileResolver for FlowPathProjectResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        let bytes = self.resolve_project_file(id)?;
        Ok(Cow::Owned(Bytes::new(bytes)))
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        let bytes = self.resolve_project_file(id)?;
        let source = source_from_bytes(id, &bytes)?;
        Ok(Cow::Owned(source))
    }
}

fn flow_relative_path(id: FileId) -> FileResult<String> {
    let path = id.vpath().as_rootless_path();
    if !is_safe_package_path(path) {
        return Err(file_not_found(id));
    }

    path.to_str()
        .map(|path| path.to_string())
        .ok_or_else(|| FileError::Other(Some(eco_format!("FlowPath contains invalid UTF-8"))))
}

struct FlowLikeUniversePackageResolver {
    files: LoadedFiles,
}

impl FlowLikeUniversePackageResolver {
    fn new(files: LoadedFiles) -> Self {
        Self { files }
    }

    fn resolve_package_file(&self, id: FileId) -> FileResult<Vec<u8>> {
        let Some(package) = id.package() else {
            return Err(file_not_found(id));
        };

        if package.namespace != "preview" {
            return Err(unsupported_package_namespace(package));
        }

        if let Some(cached) = self
            .files
            .lock()
            .map_err(|_| FileError::Other(Some(eco_format!("Could not lock package cache"))))?
            .get(&id)
            .cloned()
        {
            return Ok(cached);
        }

        let archive = fetch_universe_package_archive(package)?;
        let extracted = unpack_universe_package(package, &archive)?;

        let mut cache = self
            .files
            .lock()
            .map_err(|_| FileError::Other(Some(eco_format!("Could not lock package cache"))))?;
        for (file_id, bytes) in extracted {
            cache.entry(file_id).or_insert(bytes);
        }

        cache.get(&id).cloned().ok_or_else(|| file_not_found(id))
    }
}

impl FileResolver for FlowLikeUniversePackageResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        let bytes = self.resolve_package_file(id)?;
        Ok(Cow::Owned(Bytes::new(bytes)))
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        let bytes = self.resolve_package_file(id)?;
        let source = source_from_bytes(id, &bytes)?;
        Ok(Cow::Owned(source))
    }
}

fn fetch_universe_package_archive(package: &PackageSpec) -> FileResult<Vec<u8>> {
    let url = format!(
        "{}/{}/{}-{}.tar.gz",
        TYPST_PACKAGE_REPOSITORY_URL, package.namespace, package.name, package.version,
    );
    let response = http_ns::http_request(0, &url, "{}", &[]).ok_or_else(|| {
        FileError::Package(PackageError::NetworkFailed(Some(eco_format!(
            "host returned no response for {url}"
        ))))
    })?;

    let response_json: serde_json::Value = serde_json::from_str(&response).map_err(|error| {
        FileError::Package(PackageError::NetworkFailed(Some(eco_format!(
            "invalid host response for {url}: {error}"
        ))))
    })?;

    let status = response_json
        .get("status")
        .and_then(|status| status.as_u64())
        .unwrap_or_default();
    if status != 200 {
        return Err(FileError::Package(PackageError::NetworkFailed(Some(
            eco_format!("response returned unsuccessful status code {status} for {url}"),
        ))));
    }

    let compressed = decode_http_response_body_bytes(&response_json)?;
    if compressed.len() > MAX_UNIVERSE_PACKAGE_BYTES {
        return Err(FileError::Package(PackageError::NetworkFailed(Some(
            eco_format!("package archive exceeds {MAX_UNIVERSE_PACKAGE_BYTES} bytes"),
        ))));
    }

    let decoder = GzDecoder::new(&compressed[..]);
    let mut limited = decoder.take((MAX_UNIVERSE_PACKAGE_BYTES + 1) as u64);
    let mut archive = Vec::new();
    limited.read_to_end(&mut archive).map_err(|error| {
        FileError::Package(PackageError::MalformedArchive(Some(eco_format!("{error}"))))
    })?;

    if archive.len() > MAX_UNIVERSE_PACKAGE_BYTES {
        return Err(FileError::Package(PackageError::MalformedArchive(Some(
            eco_format!("package expands beyond {MAX_UNIVERSE_PACKAGE_BYTES} bytes"),
        ))));
    }

    Ok(archive)
}

fn decode_http_response_body_bytes(response_json: &serde_json::Value) -> FileResult<Vec<u8>> {
    if let Some(body_base64) = response_json
        .get("body_base64")
        .and_then(|body| body.as_str())
    {
        return BASE64.decode(body_base64).map_err(|error| {
            FileError::Package(PackageError::NetworkFailed(Some(eco_format!(
                "invalid base64 response body: {error}"
            ))))
        });
    }

    response_json
        .get("body")
        .and_then(|body| body.as_str())
        .map(|body| body.as_bytes().to_vec())
        .ok_or_else(|| {
            FileError::Package(PackageError::NetworkFailed(Some(eco_format!(
                "host response did not contain body_base64"
            ))))
        })
}

fn unpack_universe_package(
    package: &PackageSpec,
    archive: &[u8],
) -> FileResult<Vec<(FileId, Vec<u8>)>> {
    let mut archive = Archive::new(archive);
    let entries = archive.entries().map_err(|error| {
        FileError::Package(PackageError::MalformedArchive(Some(eco_format!("{error}"))))
    })?;

    let mut files = Vec::new();
    for entry in entries {
        let mut file = entry.map_err(|error| {
            FileError::Package(PackageError::MalformedArchive(Some(eco_format!("{error}"))))
        })?;
        if !file.header().entry_type().is_file() {
            continue;
        }

        let path = file.path().map_err(|error| {
            FileError::Package(PackageError::MalformedArchive(Some(eco_format!("{error}"))))
        })?;

        if !is_safe_package_path(&path) {
            continue;
        }

        let file_id = FileId::new(Some(package.clone()), VirtualPath::new(path.as_ref()));
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|error| {
            FileError::Package(PackageError::MalformedArchive(Some(eco_format!("{error}"))))
        })?;
        files.push((file_id, bytes));
    }

    Ok(files)
}

fn is_safe_package_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn source_from_bytes(id: FileId, bytes: &[u8]) -> FileResult<Source> {
    let contents = std::str::from_utf8(bytes).map_err(|_| FileError::InvalidUtf8)?;
    Ok(Source::new(
        id,
        contents.trim_start_matches('\u{feff}').to_owned(),
    ))
}

fn file_not_found(id: FileId) -> FileError {
    FileError::NotFound(id.vpath().as_rootless_path().to_path_buf())
}

#[cfg(test)]
#[path = "shared/asset_roots_tests.rs"]
mod asset_roots_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context(inputs: serde_json::Map<String, serde_json::Value>) -> Context {
        Context::from_input(ExecutionInput {
            inputs,
            node_id: "node".to_string(),
            run_id: "run".to_string(),
            app_id: "app".to_string(),
            board_id: "board".to_string(),
            user_id: "user".to_string(),
            stream_state: false,
            log_level: 0,
            node_name: "test".to_string(),
        })
    }

    fn flow_path(path: &str, store_ref: &str) -> FlowPath {
        FlowPath::new(path.to_string(), store_ref.to_string(), None)
    }

    #[test]
    fn asset_roots_are_optional_and_keep_their_order_and_stores() {
        let empty = read_source_project(&test_context(serde_json::Map::new())).unwrap();
        assert!(empty.asset_roots.is_empty());

        for roots in [
            Vec::new(),
            vec![
                flow_path("brand/assets/", "brand-store"),
                flow_path("", "shared-store"),
            ],
        ] {
            let mut inputs = serde_json::Map::new();
            inputs.insert("asset_roots".into(), json!(roots));
            let ctx = test_context(inputs);
            let projects = [
                read_source_project(&ctx).unwrap(),
                read_template_source_project(&ctx).unwrap().project,
            ];
            for project in projects {
                assert_eq!(project.asset_roots.len(), roots.len());
                for (actual, expected) in project.asset_roots.iter().zip(&roots) {
                    assert_eq!(actual.path, expected.path.trim_end_matches('/'));
                    assert_eq!(actual.store_ref, expected.store_ref);
                }
            }
        }
    }

    #[test]
    fn malformed_asset_roots_report_the_pin_and_bad_path_index() {
        for value in [json!(null), json!("assets"), json!(["assets"])] {
            let mut inputs = serde_json::Map::new();
            inputs.insert("asset_roots".into(), value);
            let error = read_source_project(&test_context(inputs)).err().unwrap();
            assert!(
                error.contains("Failed to deserialize input 'asset_roots'"),
                "{error}"
            );
        }
        for path in ["/assets", "../assets", "assets/../../private", "."] {
            let mut inputs = serde_json::Map::new();
            inputs.insert(
                "asset_roots".into(),
                json!([flow_path("valid", "store"), flow_path(path, "store")]),
            );
            let error = read_source_project(&test_context(inputs)).err().unwrap();
            assert!(error.contains("asset_roots[1]"), "{error}");
        }
    }

    #[test]
    fn imported_templates_receive_asset_roots() {
        let mut ctx_inputs = serde_json::Map::new();
        ctx_inputs.insert(
            "template_file".into(),
            json!(flow_path("project/report.typ", "store")),
        );
        ctx_inputs.insert(
            "asset_roots".into(),
            json!([flow_path("shared", "assets-store")]),
        );
        let project = read_template_source_project(&test_context(ctx_inputs))
            .unwrap()
            .project;
        assert_eq!(project.source_folder.unwrap().path, "project");
        assert_eq!(project.asset_roots[0].path, "shared");
        assert_eq!(project.asset_roots[0].store_ref, "assets-store");
    }

    #[test]
    fn asset_roots_are_used_without_source_folder_and_keep_in_memory_source() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".into(), json!("#read(\"main.typ\").len()"));
        inputs.insert("asset_roots".into(), json!([flow_path("assets", "store")]));
        let ctx = test_context(inputs);
        let mut project = read_source_project(&ctx).unwrap();
        project.source = "#image(\"logo.svg\")".into();
        let error = compile_project(project).err().unwrap();
        assert!(
            error.contains("asset_roots[0] at FlowLike path \"assets/logo.svg\""),
            "{error}"
        );
        assert!(!error.contains("source_folder is not connected"), "{error}");
        let compiled = compile_project(read_source_project(&ctx).unwrap()).unwrap();
        assert_eq!(compiled.document.pages.len(), 1);
    }

    #[test]
    fn font_files_are_optional_but_malformed_input_is_reported() {
        assert!(read_font_files(&test_context(serde_json::Map::new()))
            .unwrap()
            .is_empty());

        let mut inputs = serde_json::Map::new();
        inputs.insert("font_files".to_string(), json!([]));
        assert!(read_font_files(&test_context(inputs)).unwrap().is_empty());

        for value in [json!(null), json!("font.otf"), json!(["font.otf"])] {
            let mut inputs = serde_json::Map::new();
            inputs.insert("font_files".to_string(), value);
            let error = read_source_project(&test_context(inputs)).err().unwrap();
            assert!(
                error.contains("Failed to deserialize input 'font_files'"),
                "{error}"
            );
        }
    }

    #[test]
    fn unreadable_font_errors_identify_the_input_and_storage_path() {
        let mut inputs = serde_json::Map::new();
        inputs.insert(
            "font_files".to_string(),
            json!([flow_path("uploads/fonts/missing.otf", "font-store")]),
        );
        let ctx = test_context(inputs);
        let expected =
            "font_files[0]: failed to read font at FlowLike path \"uploads/fonts/missing.otf\"";
        assert_eq!(read_source_project(&ctx).err().unwrap(), expected);
        assert_eq!(read_template_source_project(&ctx).err().unwrap(), expected);
        assert_eq!(
            read_template_project(
                &ctx,
                "Body".to_string(),
                None,
                flow_path("project", "document-store"),
                flow_path("project/template.typ", "document-store"),
            )
            .err()
            .unwrap(),
            expected,
        );
    }

    #[test]
    fn invalid_fonts_do_not_silently_fall_back() {
        for bytes in [Vec::new(), b"not a font".to_vec()] {
            let error = load_font_files(&[flow_path("fonts/broken.ttf", "store")], |_| {
                Some(bytes.clone())
            })
            .unwrap_err();
            assert!(error.contains("font_files[0]"), "{error}");
            assert!(error.contains("fonts/broken.ttf"), "{error}");
            assert!(error.contains("contains no supported fonts"), "{error}");
        }
    }

    #[test]
    fn invalid_font_paths_are_rejected_before_storage_reads() {
        for path in ["", "/fonts/font.otf", "../font.otf"] {
            let error = load_font_files(&[flow_path(path, "store")], |_| {
                panic!("invalid font paths must not be read")
            })
            .unwrap_err();
            assert!(error.contains("font_files[0]"), "{error}");
        }
    }

    const CUSTOM_FONT: &[u8] = include_bytes!("../tests/fixtures/fonts/texgyrecursor-regular.otf");

    #[test]
    fn font_files_load_otf_and_ttf_from_explicit_storage_handles() {
        let ttf = typst_assets::fonts()
            .find(|bytes| bytes.starts_with(&[0, 1, 0, 0]))
            .expect("bundled TTF fixture");
        let files = [
            FlowPath::new(
                "uploads/custom.otf".to_string(),
                "custom-store".to_string(),
                Some("cache".to_string()),
            ),
            flow_path("shared/bundled.ttf", "other-store"),
        ];
        let mut read_count = 0;
        let fonts = load_font_files(&files, |file| {
            read_count += 1;
            match file.store_ref.as_str() {
                "custom-store" => {
                    assert_eq!(file.path, "uploads/custom.otf");
                    assert_eq!(file.cache_store_ref.as_deref(), Some("cache"));
                    Some(CUSTOM_FONT.to_vec())
                }
                "other-store" => Some(ttf.to_vec()),
                _ => panic!("unexpected storage handle"),
            }
        })
        .unwrap();
        assert_eq!(read_count, 2);
        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[0].info().family, "TeX Gyre Cursor");
        assert_eq!(fonts[1].info().family, "DejaVu Sans Mono");
    }

    fn frame_font_families(frame: &Frame, families: &mut Vec<String>) {
        for (_, item) in frame.items() {
            match item {
                FrameItem::Text(text) => families.push(text.font.info().family.to_string()),
                FrameItem::Group(group) => frame_font_families(&group.frame, families),
                _ => {}
            }
        }
    }

    #[test]
    fn custom_fonts_are_used_in_source_and_template_compilation() {
        let source = r#"#set text(font: "TeX Gyre Cursor")
Custom font
#text(font: "Libertinus Serif")[Bundled font]
"#;
        let project = |fonts| TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: source.to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts,
        };
        let without_fonts = compile_project(project(Vec::new())).unwrap();
        assert!(without_fonts
            .warnings
            .iter()
            .any(|warning| { warning.contains("unknown font family: tex gyre cursor") }));

        let fonts = load_font_files(&[flow_path("fonts/custom.otf", "store")], |_| {
            Some(CUSTOM_FONT.to_vec())
        })
        .unwrap();
        let mut template = project(fonts.clone());
        template.source = "#text(font: sys.inputs.font)[Custom font]\n#text(font: \"Libertinus Serif\")[Bundled font]".to_string();
        let mut inputs = Dict::new();
        inputs.insert(Str::from("font"), Value::Str(Str::from("TeX Gyre Cursor")));
        let documents = [
            compile_project(project(fonts)).unwrap(),
            compile_template_project(template.into(), inputs).unwrap(),
        ];
        for compiled in documents {
            assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
            let mut families = Vec::new();
            for page in &compiled.document.pages {
                frame_font_families(&page.frame, &mut families);
            }
            assert!(
                families.iter().any(|name| name == "TeX Gyre Cursor"),
                "{families:?}"
            );
            assert!(
                families.iter().any(|name| name == "Libertinus Serif"),
                "{families:?}"
            );
        }
    }

    #[test]
    fn json_template_data_converts_to_typst_values() {
        let value = serde_json::json!({
            "title": "Quarterly Briefing",
            "count": 3,
            "draft": true,
            "items": ["Data", null],
            "meta": { "client": "Rheosoph" }
        });

        let Value::Dict(dict) = json_value_to_typst(value).expect("json should convert") else {
            panic!("root object should convert to dictionary");
        };

        assert!(
            matches!(dict.get("title").unwrap(), Value::Str(value) if value.as_str() == "Quarterly Briefing")
        );
        assert!(matches!(dict.get("count").unwrap(), Value::Int(3)));
        assert!(matches!(dict.get("draft").unwrap(), Value::Bool(true)));
        assert!(matches!(dict.get("items").unwrap(), Value::Array(_)));
        assert!(matches!(dict.get("meta").unwrap(), Value::Dict(_)));
    }

    #[test]
    fn compile_project_accepts_template_inputs() {
        let mut inputs = Dict::new();
        inputs.insert(Str::from("title"), Value::Str(Str::from("Template Title")));

        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: "#import sys: inputs\n= #inputs.title".to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts: Vec::new(),
        };

        let compiled =
            compile_template_project(project.into(), inputs).expect("template should compile");
        assert_eq!(compiled.document.pages.len(), 1);
    }

    #[test]
    fn compile_project_applies_function_with_template_inputs() {
        let mut inputs = Dict::new();
        inputs.insert(Str::from("title"), Value::Str(Str::from("Template Title")));

        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: r#"#let template(title: "", doc) = [
= #title
#doc
]
#show: template.with(..sys.inputs)

Body
"#
            .to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts: Vec::new(),
        };

        let compiled =
            compile_template_project(project.into(), inputs).expect("template should compile");
        assert_eq!(compiled.document.pages.len(), 1);
    }

    #[test]
    fn checked_example_compiles_through_an_imported_template() {
        let source = apply_template_source(
            "report.typ",
            "report",
            include_str!("../examples/template/document.typ"),
        )
        .expect("example wrapper should be valid");
        let mut context_inputs = serde_json::Map::new();
        context_inputs.insert(
            "template_data".to_string(),
            serde_json::json!(include_str!("../examples/template/data.json")),
        );
        let inputs =
            read_template_inputs(&test_context(context_inputs)).expect("example data should parse");

        let engine = TypstEngine::builder()
            .fonts(typst_assets::fonts())
            .with_static_source_file_resolver([
                (file_id(TEMPLATE_ENTRY_FILE), source),
                (
                    file_id("report.typ"),
                    include_str!("../examples/template/report.typ").to_string(),
                ),
            ])
            .build();
        let warned = engine.compile_with_input(TEMPLATE_ENTRY_FILE, inputs);
        let document: PagedDocument = warned
            .output
            .expect("checked template example should compile through its import");

        assert_eq!(document.pages.len(), 1);
    }

    #[test]
    fn apply_template_source_imports_template_function() {
        let source =
            apply_template_source("templates/report.typ", "report", "= Body").expect("source");

        assert!(source.contains(r#"#import "templates/report.typ": report as __flowlike_template"#));
        assert!(source.contains("#show: __flowlike_template.with(..sys.inputs)"));
        assert!(source.ends_with("= Body"));
    }

    #[test]
    fn relative_import_path_is_from_main_file_directory() {
        let path =
            relative_import_path("docs/report.typ", "templates/report.typ").expect("relative path");

        assert_eq!(path, "../templates/report.typ");
    }

    #[test]
    fn generated_template_entry_does_not_shadow_template() {
        assert_eq!(
            generated_template_entry_path("main.typ"),
            "__flowlike_entry.typ"
        );
        assert_eq!(
            generated_template_entry_path("__flowlike_entry.typ"),
            "__flowlike_entry_1.typ"
        );
    }

    #[test]
    fn source_template_named_main_uses_a_distinct_entry_path() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".to_string(), serde_json::json!(""));
        inputs.insert(
            "source_folder".to_string(),
            serde_json::to_value(flow_path("project", "store")).unwrap(),
        );
        inputs.insert(
            "template_file".to_string(),
            serde_json::to_value(flow_path("project/main.typ", "store")).unwrap(),
        );
        let ctx = test_context(inputs);

        let project = read_template_source_project(&ctx)
            .expect("project should be valid")
            .project;

        assert_eq!(project.main_path, "__flowlike_entry.typ");
        assert!(project
            .source
            .starts_with(r#"#import "main.typ": template as __flowlike_template"#));
    }

    #[test]
    fn template_source_preserves_explicit_empty_body() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".to_string(), serde_json::json!(""));
        let ctx = test_context(inputs);

        let project = read_template_source_project(&ctx)
            .expect("project should be valid")
            .project;

        assert!(project.source.is_empty());
    }

    #[test]
    fn malformed_optional_flow_path_is_an_error() {
        let mut inputs = serde_json::Map::new();
        inputs.insert(
            "template_file".to_string(),
            serde_json::json!("templates/report.typ"),
        );
        let ctx = test_context(inputs);

        let error = read_template_source_project(&ctx)
            .err()
            .expect("input should be rejected");

        assert!(error.contains("template_file"));
        assert!(error.contains("deserialize"));
    }

    #[test]
    fn malformed_source_folder_is_not_treated_as_absent() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".to_string(), serde_json::json!("= Body"));
        inputs.insert("source_folder".to_string(), serde_json::json!("project"));
        let ctx = test_context(inputs);

        let error = read_template_source_project(&ctx)
            .err()
            .expect("input should be rejected");

        assert!(error.contains("source_folder"));
        assert!(error.contains("deserialize"));
    }

    #[test]
    fn source_project_rejects_a_malformed_source_folder() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".to_string(), serde_json::json!("= Body"));
        inputs.insert("source_folder".to_string(), serde_json::json!("project"));
        let ctx = test_context(inputs);

        let error = read_source_project(&ctx)
            .err()
            .expect("input should be rejected");

        assert!(error.contains("source_folder"));
        assert!(error.contains("deserialize"));
    }

    #[test]
    fn explicit_source_folder_preserves_nested_main_path() {
        let source_file = flow_path("project/docs/main.typ", "store");
        let source_folder = flow_path("project", "store");

        let (main_path, resolved_folder) =
            file_project_location(&source_file, Some(source_folder)).expect("valid location");

        assert_eq!(main_path, "docs/main.typ");
        assert_eq!(resolved_folder.path, "project");
    }

    #[test]
    fn root_level_source_file_resolves_assets_from_the_store_root() {
        let source_file = flow_path("thumbnail.typ", "store");

        let (main_path, resolved_folder) =
            file_project_location(&source_file, None).expect("valid location");

        assert_eq!(main_path, "thumbnail.typ");
        assert_eq!(resolved_folder.path, "");
        assert_eq!(resolved_folder.store_ref, "store");
    }

    fn compile_error(source: &str, source_folder: Option<FlowPath>) -> String {
        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: source.to_string(),
            source_folder,
            asset_roots: Vec::new(),
            allow_universe_packages: true,
            fonts: Vec::new(),
        };

        compile_project(project)
            .err()
            .expect("compilation should fail")
    }

    #[test]
    fn missing_asset_error_names_the_source_folder_path() {
        let error = compile_error(
            "#set page(width: 10pt, height: 10pt)\n#image(\"assets/logo.webp\")",
            Some(flow_path("projects/thumbnail", "store")),
        );

        assert!(error.contains(
            "main.typ:2:8: error: failed to load file (assets/logo.webp could not be read from source_folder at FlowLike path \"projects/thumbnail/assets/logo.webp\")"
        ), "{error}");
        assert!(!error.contains("searched at"), "{error}");
        assert!(!error.contains("SourceDiagnostic"), "{error}");
    }

    #[test]
    fn missing_source_folder_is_not_masked_by_the_package_resolver() {
        let error = compile_error(
            "#set page(background: image(\"background.svg\"))\nBody",
            None,
        );

        assert!(error.contains("main.typ:1:29: error:"), "{error}");
        assert!(
            error.contains("source_folder is not connected, so background.svg cannot be loaded"),
            "{error}"
        );
    }

    #[test]
    fn disabled_universe_packages_are_reported_as_such() {
        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: "#import \"@preview/cetz:0.3.0\": canvas".to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts: Vec::new(),
        };

        let error = compile_project(project)
            .err()
            .expect("compilation should fail");

        assert!(
            error.contains(
                "@preview/cetz:0.3.0 needs Typst Universe, but allow_universe_packages is disabled"
            ),
            "{error}"
        );
    }

    #[test]
    fn warnings_are_readable_and_located() {
        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: "#set text(font: \"Inter\")\nBody".to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts: Vec::new(),
        };

        let compiled = compile_project(project).expect("unknown fonts only warn");

        assert!(
            compiled
                .warnings
                .iter()
                .any(|warning| warning
                    .starts_with("main.typ:1:17: warning: unknown font family: inter")),
            "{:?}",
            compiled.warnings
        );
    }

    fn template_source_context(source: &str) -> Context {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".to_string(), serde_json::json!(source));
        inputs.insert(
            "template_file".to_string(),
            serde_json::to_value(flow_path("project/template.typ", "store")).unwrap(),
        );
        inputs.insert(
            "allow_universe_packages".to_string(),
            serde_json::json!(false),
        );
        test_context(inputs)
    }

    #[test]
    fn generated_template_import_errors_name_the_source_pin() {
        let project = read_template_source_project(&template_source_context("= Title"))
            .expect("project should be valid");

        let error = compile_template_project(project, Dict::new())
            .err()
            .expect("the template cannot be read");

        assert!(
            error.contains("source (generated template import): error: failed to load file (template.typ could not be read"),
            "{error}"
        );
        assert!(!error.contains("__flowlike_entry"), "{error}");
    }

    #[test]
    fn files_that_start_like_the_wrapper_keep_their_line_numbers() {
        let source = apply_template_source("report.typ", "report", "= Title\n#undefined")
            .expect("wrapped source");

        let error = compile_error(&source, None);

        assert!(
            error.contains("main.typ:1:9: error: failed to load file (source_folder is not connected, so report.typ cannot be loaded"),
            "{error}"
        );
    }

    #[test]
    fn template_import_paths_reject_typst_line_breaks() {
        assert!(apply_template_source("a\u{2028}b.typ", "template", "").is_err());
        assert!(apply_template_source("a\u{85}b.typ", "template", "").is_err());
        assert!(apply_template_source("a b/ü.typ", "template", "").is_ok());
    }

    #[test]
    fn data_file_errors_are_located_by_byte_range() {
        let data_id = file_id("data.toml");
        let loaded_files = LoadedFiles::default();
        loaded_files
            .lock()
            .unwrap()
            .insert(data_id, "\u{feff}a = 1\nb = \nc = 2\n".as_bytes().to_vec());
        let diagnostics = DiagnosticSources {
            main: Source::new(file_id(MAIN_FILE), "#toml(\"data.toml\")".to_string()),
            main_name: MAIN_FILE.to_string(),
            main_line_offset: 0,
            loaded_files,
        };

        // The byte order mark is 3 bytes, so byte 13 is the end of "b = ".
        let span = Span::from_range(data_id, 13..14);

        assert_eq!(diagnostics.location(span).as_deref(), Some("data.toml:2:5"));
    }

    #[test]
    fn source_nodes_can_read_their_own_in_memory_source() {
        let compiled = compile_source("#let me = read(\"main.typ\")\n#me.len()");

        assert_eq!(compiled.document.pages.len(), 1);
    }

    #[test]
    fn non_preview_packages_are_unsupported_even_when_downloads_are_off() {
        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: "#import \"@local/mytpl:0.1.0\": *".to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts: Vec::new(),
        };

        let error = compile_project(project)
            .err()
            .expect("package is unavailable");

        assert!(
            error.contains(
                "only @preview packages can be downloaded, so @local/mytpl:0.1.0 is not available"
            ),
            "{error}"
        );
    }

    fn compile_source(source: &str) -> CompiledTypstDocument {
        compile_project(TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: source.to_string(),
            source_folder: None,
            asset_roots: Vec::new(),
            allow_universe_packages: false,
            fonts: Vec::new(),
        })
        .expect("source should compile")
    }

    fn png_size(png: &[u8]) -> (u32, u32) {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "output should be a PNG");
        let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
        (width, height)
    }

    #[test]
    fn png_export_at_72_ppi_maps_points_to_pixels() {
        let compiled = compile_source("#set page(width: 1280pt, height: 720pt, fill: black)\nHi");

        let png = export_png(
            &compiled,
            &PngOptions {
                ppi: 72.0,
                page: Some(1),
            },
        )
        .expect("png");

        assert_eq!(png_size(&png), (1280, 720));
    }

    #[test]
    fn png_export_selects_or_stacks_pages() {
        let compiled = compile_source(
            "#set page(width: 100pt, height: 50pt)\nOne\n#pagebreak()\n#set page(width: 80pt, height: 40pt)\nTwo",
        );

        let second = export_png(
            &compiled,
            &PngOptions {
                ppi: 144.0,
                page: Some(2),
            },
        )
        .expect("png");
        let stacked = export_png(
            &compiled,
            &PngOptions {
                ppi: 144.0,
                page: None,
            },
        )
        .expect("png");

        assert_eq!(png_size(&second), (160, 80));
        assert_eq!(png_size(&stacked), (200, 180));
    }

    fn png(
        compiled: &CompiledTypstDocument,
        ppi: f64,
        page: Option<u64>,
    ) -> Result<Vec<u8>, String> {
        export_png(compiled, &PngOptions { ppi, page })
    }

    #[test]
    fn png_export_rejects_missing_pages() {
        let compiled = compile_source("#set page(width: 100pt, height: 100pt)\nOne");

        let missing = png(&compiled, 72.0, Some(2)).expect_err("page 2 does not exist");

        assert!(missing.contains("page 2 does not exist; the document has 1 page(s)"));
    }

    #[test]
    fn png_export_stays_within_the_memory_tier() {
        let thumbnail = compile_source("#set page(width: 1280pt, height: 720pt)\nOne");
        let two_pages =
            compile_source("#set page(width: 1280pt, height: 720pt)\nOne\n#pagebreak()\nTwo");

        let too_large = png(&thumbnail, 600.0, Some(1)).expect_err("600 ppi does not fit");
        let stacked = png(&two_pages, 144.0, None).expect_err("two stacked pages do not fit");

        assert!(
            too_large.contains("a PNG at 600 ppi needs about 793 MB, more than the 64 MB available; use at most 170 ppi"),
            "{too_large}"
        );
        assert!(png(&thumbnail, 170.0, Some(1)).is_ok());
        assert!(
            stacked.contains("render a single page or use at most"),
            "{stacked}"
        );
        assert!(png(&two_pages, 144.0, Some(2)).is_ok());
    }

    /// Runs typst-render directly, as export_png would without its checks.
    fn render_panics(compiled: &CompiledTypstDocument, ppi: f64) -> bool {
        let page = &compiled.document.pages[0];
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            typst_render::render(page, (ppi / 72.0) as f32)
        }))
        .is_err()
    }

    #[test]
    fn png_export_rejects_tilings_smaller_than_a_pixel() {
        let compiled = compile_source(
            "#set page(width: 100pt, height: 100pt, fill: tiling(size: (2pt, 2pt))[#place(circle(radius: 0.5pt))])",
        );

        assert!(render_panics(&compiled, 12.0));
        let error = png(&compiled, 12.0, Some(1)).expect_err("tile is under a pixel");
        assert!(
            error.contains("a tiling tile is smaller than one pixel at 12 ppi"),
            "{error}"
        );
        assert!(png(&compiled, 144.0, Some(1)).is_ok());
    }

    #[test]
    fn png_export_rejects_text_tilings_sampled_outside_their_texture() {
        let compiled = compile_source(
            "#set page(width: 200pt, height: 100pt)\n#text(size: 40pt, fill: tiling(size: (10.3pt, 10.3pt))[#rect(width: 10.3pt, height: 10.3pt, fill: red)])[Tiled]",
        );

        assert!(render_panics(&compiled, 72.0));
        let error = png(&compiled, 72.0, Some(1)).expect_err("tile is 10.3 px wide");
        assert!(
            error.contains("text filled with a tiling cannot be rendered at 72 ppi"),
            "{error}"
        );
        assert!(png(&compiled, 144.0, Some(1)).is_ok());
    }

    #[test]
    fn png_export_rejects_transforms_the_renderer_cannot_invert() {
        let compiled = compile_source(
            "#set page(width: 100pt, height: 100pt)\n#skew(ax: 45deg, ay: 45deg)[#block[#block(fill: red)[x]]]",
        );

        assert!(render_panics(&compiled, 72.0));
        let error = png(&compiled, 72.0, Some(1)).expect_err("singular transform");
        assert!(error.contains("cannot invert"), "{error}");
    }

    #[test]
    fn png_export_renders_gradients_and_tilings_that_fit() {
        let compiled = compile_source(
            "#set page(width: 300pt, height: 200pt, fill: gradient.linear(red, blue))\n#text(fill: gradient.linear(red, blue, relative: \"parent\"))[Text]\n#rect(width: 50pt, height: 50pt, fill: tiling(size: (10pt, 10pt))[#circle(radius: 3pt)])\n#rotate(30deg)[#block[#text(fill: tiling(size: (8pt, 8pt))[#rect(width: 4pt, height: 4pt)])[Hi]]]",
        );

        assert!(png(&compiled, 72.0, Some(1)).is_ok());
        assert!(png(&compiled, 144.0, None).is_ok());
    }

    #[test]
    fn png_options_validate_ppi_and_page() {
        let options = |ppi: serde_json::Value, page: serde_json::Value| {
            let mut inputs = serde_json::Map::new();
            inputs.insert("ppi".to_string(), ppi);
            inputs.insert("page".to_string(), page);
            read_png_options(&test_context(inputs))
        };

        let defaults = read_png_options(&test_context(serde_json::Map::new())).expect("defaults");
        assert_eq!(defaults.ppi, DEFAULT_PNG_PPI);
        assert_eq!(defaults.page, Some(1));
        assert_eq!(
            options(serde_json::json!(72), serde_json::json!(0))
                .unwrap()
                .page,
            None
        );
        assert!(options(serde_json::json!(0.0), serde_json::json!(1)).is_err());
        assert!(options(serde_json::json!(0.5), serde_json::json!(1)).is_err());
        assert!(options(serde_json::json!(-72.0), serde_json::json!(1)).is_err());
        assert!(options(serde_json::json!(72.0), serde_json::json!(-1)).is_err());
    }

    #[test]
    fn user_lines_after_the_template_wrapper_keep_their_numbers() {
        let wrapper = format!(
            "#import \"template.typ\": template as __flowlike_template\n{TEMPLATE_SHOW_RULE}\n\n"
        );
        let diagnostics = DiagnosticSources {
            main: Source::new(
                file_id(MAIN_FILE),
                format!("{wrapper}= Title\n#image(\"x.svg\")"),
            ),
            main_name: MAIN_FILE.to_string(),
            main_line_offset: TEMPLATE_WRAPPER_LINES,
            loaded_files: LoadedFiles::default(),
        };
        let image_call = diagnostics
            .main
            .root()
            .children()
            .find(|node| node.kind() == SyntaxKind::FuncCall)
            .expect("image call");

        assert_eq!(
            diagnostics.location(image_call.span()).as_deref(),
            Some("main.typ:2:2")
        );
    }

    #[test]
    fn invalid_source_file_folder_relationship_is_an_error() {
        let source_file = flow_path("project/docs/main.typ", "store");
        let source_folder = flow_path("another-project", "store");

        let error = file_project_location(&source_file, Some(source_folder))
            .expect_err("outside-root source should be rejected");

        assert!(error.contains("must be inside source_folder"));
    }

    #[test]
    fn source_file_and_folder_must_share_a_store() {
        let source_file = flow_path("project/main.typ", "source-store");
        let source_folder = flow_path("project", "other-store");

        let error = file_project_location(&source_file, Some(source_folder))
            .expect_err("store mismatch should be rejected");

        assert!(error.contains("same FlowPath store"));
    }

    #[test]
    fn absolute_flow_paths_are_rejected() {
        let error = normalized_flow_path("/project/main.typ", "source_file", false)
            .expect_err("absolute FlowPath should be rejected");

        assert!(error.contains("relative FlowPath"));
    }

    #[test]
    fn typst_identifier_validation_matches_typst_syntax() {
        assert!(is_typst_identifier("report"));
        assert!(is_typst_identifier("résumé"));
        assert!(is_typst_identifier("report-template"));
        assert!(!is_typst_identifier("_"));
        assert!(!is_typst_identifier("if"));
        assert!(!is_typst_identifier("two words"));
    }
}
