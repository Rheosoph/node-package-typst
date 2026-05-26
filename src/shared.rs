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
use typst::foundations::Bytes;
use typst::layout::{Abs, PagedDocument};
use typst::syntax::{package::PackageSpec, FileId, Source, VirtualPath};
use typst_as_lib::{
    cached_file_resolver::CachedFileResolver, file_resolver::FileResolver, TypstEngine,
};

pub const CATEGORY: &str = "Document/Typst";
pub const MAIN_FILE: &str = "main.typ";
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
        "Typst source for main.typ",
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

pub fn add_output_file_pin(node: &mut NodeDefinition, description: &str) {
    node.add_input_pin("output_file", "Output", description, VariableType::Struct)
        .set_schema::<FlowPath>();
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

pub fn read_source_project(ctx: &Context) -> TypstProject {
    let source = read_source_text(ctx);

    TypstProject {
        main_path: MAIN_FILE.to_string(),
        source,
        source_folder: ctx.get_input_as("source_folder"),
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
    }
}

pub fn read_file_project(ctx: &Context) -> Result<TypstProject, String> {
    let source_file: FlowPath = ctx.require_input_as("source_file")?;
    let source = source_file
        .get_string(ctx)
        .ok_or_else(|| format!("Failed to read Typst source from {}", source_file.path))?;
    let source_folder = ctx
        .get_input_as("source_folder")
        .or_else(|| source_file.parent());

    Ok(TypstProject {
        main_path: source_file
            .file_name()
            .unwrap_or_else(|| MAIN_FILE.to_string()),
        source,
        source_folder,
        allow_universe_packages: ctx.get_bool("allow_universe_packages").unwrap_or(true),
    })
}

pub fn compile_project(project: TypstProject) -> Result<CompiledTypstDocument, String> {
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

    let warned = engine.compile(main_path.as_str());
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

fn read_source_text(ctx: &Context) -> String {
    let source = ctx.get_string("source").unwrap_or_default();
    if source.trim().is_empty() {
        DEFAULT_SOURCE.to_string()
    } else {
        source
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
    ctx.require_input_as("output_file")
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
