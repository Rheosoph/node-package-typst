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
    sync::Mutex,
};
use typst::diag::{FileError, FileResult, PackageError};
use typst::foundations::{Array, Bytes, Dict, Str, Value};
use typst::layout::{Abs, PagedDocument};
use typst::syntax::{
    is_ident as is_typst_syntax_identifier, package::PackageSpec, parse_code, FileId, Source,
    SyntaxKind, VirtualPath,
};
use typst_as_lib::{
    cached_file_resolver::CachedFileResolver, file_resolver::FileResolver, TypstEngine,
};

pub const CATEGORY: &str = "Document/Typst";
pub const MAIN_FILE: &str = "main.typ";
pub const DEFAULT_TEMPLATE_FUNCTION: &str = "template";
const TEMPLATE_ENTRY_FILE: &str = "__flowlike_entry.typ";
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
    pub allow_universe_packages: bool,
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
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
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
        source_folder,
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
    })
}

pub fn read_template_source_project(ctx: &Context) -> Result<TypstProject, String> {
    let source = read_template_source_text(ctx);
    let source_folder = read_optional_flow_path(ctx, "source_folder")?;
    let template_file = read_optional_flow_path(ctx, "template_file")?;
    let Some(template_file) = template_file else {
        return Ok(TypstProject {
            main_path: MAIN_FILE.to_string(),
            source,
            source_folder,
            allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
        });
    };

    let source_folder = source_folder
        .or_else(|| template_file.parent())
        .unwrap_or_else(|| empty_root_like(&template_file));

    read_template_project(ctx, source, None, source_folder, template_file)
}

pub fn read_template_file_project(ctx: &Context) -> Result<TypstProject, String> {
    let template_file = read_optional_flow_path(ctx, "template_file")?;
    let Some(template_file) = template_file else {
        return read_file_project(ctx);
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
) -> Result<TypstProject, String> {
    let template_function = read_template_function(ctx)?;
    let template_path =
        flow_path_relative_to_root(&template_file, &source_folder, "template_file")?;
    let main_path = main_path.unwrap_or_else(|| generated_template_entry_path(&template_path));
    if main_path == template_path {
        return Err("source_file and template_file must refer to different files".to_string());
    }
    let import_path = relative_import_path(&main_path, &template_path)?;
    let source = apply_template_source(&import_path, &template_function, &source)?;

    Ok(TypstProject {
        main_path,
        source,
        source_folder: Some(source_folder),
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
    })
}

pub fn compile_project(project: TypstProject) -> Result<CompiledTypstDocument, String> {
    compile_project_with_optional_inputs(project, None)
}

pub fn compile_project_with_inputs(
    project: TypstProject,
    inputs: Dict,
) -> Result<CompiledTypstDocument, String> {
    compile_project_with_optional_inputs(project, Some(inputs))
}

fn compile_project_with_optional_inputs(
    project: TypstProject,
    inputs: Option<Dict>,
) -> Result<CompiledTypstDocument, String> {
    let main_path = project.main_path;
    let main_id = file_id(&main_path);

    let mut builder = TypstEngine::builder()
        .fonts(typst_assets::fonts())
        .with_static_source_file_resolver([(main_id, project.source)]);

    builder = match project.source_folder {
        Some(source_folder) => builder.add_file_resolver(
            CachedFileResolver::new(FlowPathProjectResolver::new(source_folder))
                .with_in_memory_source_cache()
                .with_in_memory_binary_cache(),
        ),
        None => builder.add_file_resolver(MissingSourceFolderResolver),
    };

    if project.allow_universe_packages {
        builder = builder.add_file_resolver(
            CachedFileResolver::new(FlowLikeUniversePackageResolver::default())
                .with_in_memory_source_cache()
                .with_in_memory_binary_cache(),
        );
    }

    let engine = builder.build();

    let warned = match inputs {
        Some(inputs) => engine.compile_with_input(main_path.as_str(), inputs),
        None => engine.compile(main_path.as_str()),
    };
    let document: PagedDocument = warned
        .output
        .map_err(|error| format!("Typst compilation failed: {error:?}"))?;
    let warnings = warned
        .warnings
        .iter()
        .map(|warning| format!("{warning:?}"))
        .collect();

    Ok(CompiledTypstDocument { document, warnings })
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

fn file_project_location(
    source_file: &FlowPath,
    source_folder: Option<FlowPath>,
) -> Result<(String, Option<FlowPath>), String> {
    if let Some(source_folder) = source_folder {
        let main_path = flow_path_relative_to_root(source_file, &source_folder, "source_file")?;
        return Ok((main_path, Some(source_folder)));
    }

    if let Some(source_folder) = source_file.parent() {
        let main_path = flow_path_relative_to_root(source_file, &source_folder, "source_file")?;
        return Ok((main_path, Some(source_folder)));
    }

    let main_path = normalized_flow_path(&source_file.path, "source_file", false)?;
    Ok((main_path, None))
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
         #show: __flowlike_template.with(..sys.inputs)\n\n\
         {source}"
    ))
}

fn typst_string_literal(value: &str) -> Result<String, String> {
    if value.chars().any(|char| char.is_control()) {
        return Err("Typst import paths cannot contain control characters".to_string());
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

struct FlowPathProjectResolver {
    root: FlowPath,
    files: Mutex<HashMap<FileId, Vec<u8>>>,
}

impl FlowPathProjectResolver {
    fn new(root: FlowPath) -> Self {
        Self {
            root,
            files: Mutex::new(HashMap::new()),
        }
    }

    fn resolve_project_file(&self, id: FileId) -> FileResult<Vec<u8>> {
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
        let flow_path = self.root.child(&relative_path);
        let flow_path_json = serde_json::to_string(&flow_path)
            .map_err(|error| FileError::Other(Some(eco_format!("{error}"))))?;
        let bytes = flow_like_wasm_sdk::host::storage_read(&flow_path_json).ok_or_else(|| {
            FileError::Other(Some(eco_format!(
                "source folder file not found (tried {} for Typst path {})",
                flow_path.path,
                relative_path
            )))
        })?;

        self.files
            .lock()
            .map_err(|_| FileError::Other(Some(eco_format!("Could not lock source cache"))))?
            .insert(id, bytes.clone());
        Ok(bytes)
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

struct MissingSourceFolderResolver;

impl FileResolver for MissingSourceFolderResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        missing_source_folder(id)?;
        Err(file_not_found(id))
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        missing_source_folder(id)?;
        Err(file_not_found(id))
    }
}

fn missing_source_folder(id: FileId) -> FileResult<()> {
    if id.package().is_some() {
        return Ok(());
    }

    let path = id.vpath().as_rootless_path();
    if path == Path::new(MAIN_FILE) {
        return Ok(());
    }

    Err(FileError::Other(Some(eco_format!(
        "Typst referenced {}, but source_folder is not connected",
        path.display()
    ))))
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

#[derive(Default)]
struct FlowLikeUniversePackageResolver {
    files: Mutex<HashMap<FileId, Vec<u8>>>,
}

impl FlowLikeUniversePackageResolver {
    fn resolve_package_file(&self, id: FileId) -> FileResult<Vec<u8>> {
        let Some(package) = id.package() else {
            return Err(file_not_found(id));
        };

        if package.namespace != "preview" {
            return Err(file_not_found(id));
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
            allow_universe_packages: false,
        };

        let compiled =
            compile_project_with_inputs(project, inputs).expect("template should compile");
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
            allow_universe_packages: false,
        };

        let compiled =
            compile_project_with_inputs(project, inputs).expect("template should compile");
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

        let project = read_template_source_project(&ctx).expect("project should be valid");

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

        let project = read_template_source_project(&ctx).expect("project should be valid");

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
        assert_eq!(resolved_folder.unwrap().path, "project");
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
