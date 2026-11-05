use super::*;

fn root(path: &str, store: &str) -> FlowPath {
    FlowPath::new(path.to_string(), store.to_string(), None)
}

#[test]
fn first_readable_asset_root_wins_and_is_cached_without_a_source_folder() {
    let resolver = FlowPathProjectResolver::new(
        None,
        vec![root("preferred", "store"), root("fallback", "store")],
        LoadedFiles::default(),
    );
    let id = file_id("images/logo.svg");
    let bytes = resolver
        .resolve_project_file_with(id, |path| {
            assert_eq!(path.path, "preferred/images/logo.svg");
            Ok(Some(b"preferred image".to_vec()))
        })
        .unwrap();
    assert_eq!(bytes, b"preferred image");
    assert_eq!(
        resolver
            .resolve_project_file_with(id, |_| panic!("the winner must stay cached"))
            .unwrap(),
        bytes
    );
}

#[test]
fn source_folder_keeps_priority_over_asset_roots() {
    let resolver = FlowPathProjectResolver::new(
        Some(root("project", "source-store")),
        vec![root("assets", "asset-store")],
        LoadedFiles::default(),
    );
    assert_eq!(
        resolver
            .resolve_project_file_with(file_id("logo.svg"), |path| {
                assert_eq!(path.path, "project/logo.svg");
                assert_eq!(path.store_ref, "source-store");
                Ok(Some(b"existing image".to_vec()))
            })
            .unwrap(),
        b"existing image"
    );
}

#[test]
fn missing_files_fall_through_in_order_and_keep_each_stores_handles() {
    let resolver = FlowPathProjectResolver::new(
        Some(root("project", "source-store")),
        vec![
            root("brand", "brand-store"),
            FlowPath::new(
                "shared".to_string(),
                "shared-store".to_string(),
                Some("shared-cache".to_string()),
            ),
            root("unused", "unused-store"),
        ],
        LoadedFiles::default(),
    );
    let mut attempts = Vec::new();
    let bytes = resolver
        .resolve_project_file_with(file_id("images/logo.svg"), |path| {
            attempts.push((
                path.path.clone(),
                path.store_ref.clone(),
                path.cache_store_ref.clone(),
            ));
            Ok((path.store_ref == "shared-store").then(|| b"shared image".to_vec()))
        })
        .unwrap();
    assert_eq!(bytes, b"shared image");
    assert_eq!(
        attempts,
        vec![
            (
                "project/images/logo.svg".to_string(),
                "source-store".to_string(),
                None
            ),
            (
                "brand/images/logo.svg".to_string(),
                "brand-store".to_string(),
                None
            ),
            (
                "shared/images/logo.svg".to_string(),
                "shared-store".to_string(),
                Some("shared-cache".to_string())
            ),
        ]
    );
}

#[test]
fn empty_files_are_successful_matches() {
    let resolver = FlowPathProjectResolver::new(
        None,
        vec![root("", "empty-store"), root("fallback", "other-store")],
        LoadedFiles::default(),
    );
    assert!(resolver
        .resolve_project_file_with(file_id("data.txt"), |path| {
            assert_eq!(path.path, "data.txt");
            assert_eq!(path.store_ref, "empty-store");
            Ok(Some(Vec::new()))
        })
        .unwrap()
        .is_empty());
}

#[test]
fn missing_asset_diagnostic_lists_every_attempt_in_search_order() {
    let resolver = FlowPathProjectResolver::new(
        Some(root("project", "store")),
        vec![root("brand", "store"), root("shared", "other-store")],
        LoadedFiles::default(),
    );
    let error = resolver
        .resolve_project_file_with(file_id("images/missing.svg"), |_| Ok(None))
        .unwrap_err()
        .to_string();
    assert_eq!(
        error,
        "failed to load file (images/missing.svg could not be read from source_folder at FlowLike path \"project/images/missing.svg\"; asset_roots[0] at FlowLike path \"brand/images/missing.svg\"; asset_roots[1] at FlowLike path \"shared/images/missing.svg\")"
    );
}

#[test]
fn normalized_virtual_paths_keep_their_project_relative_directories() {
    let resolver =
        FlowPathProjectResolver::new(None, vec![root("shared", "store")], LoadedFiles::default());
    for path in ["/templates/../images/logo.svg", "images/./logo.svg"] {
        assert_eq!(
            resolver
                .resolve_project_file_with(file_id(path), |path| {
                    assert_eq!(path.path, "shared/images/logo.svg");
                    Ok(Some(b"image".to_vec()))
                })
                .unwrap(),
            b"image"
        );
    }
}

#[test]
fn package_files_never_resolve_from_asset_roots_or_the_project_cache() {
    let package: PackageSpec = "@preview/example:0.1.0".parse().unwrap();
    let id = FileId::new(Some(package), VirtualPath::new("images/logo.svg"));
    let files = LoadedFiles::default();
    files.lock().unwrap().insert(id, b"cached package".to_vec());
    let resolver = FlowPathProjectResolver::new(
        Some(root("project", "store")),
        vec![root("shared", "store")],
        files.clone(),
    );
    assert!(matches!(
        resolver.resolve_project_file_with(id, |_| panic!("packages have their own resolver")),
        Err(FileError::NotFound(_))
    ));

    let main = Source::new(file_id(MAIN_FILE), "Body".to_string());
    let router = ProjectFileResolver::new(&main, None, vec![root("shared", "store")], false, files);
    let error = router.resolve_binary(id).unwrap_err().to_string();
    assert!(
        error.contains("allow_universe_packages is disabled"),
        "{error}"
    );
}

/// Uses the production search and caching logic with an in-memory storage host.
struct MemoryStorageResolver {
    project: FlowPathProjectResolver,
    storage: HashMap<(String, String), Vec<u8>>,
}

impl MemoryStorageResolver {
    fn read(&self, id: FileId) -> FileResult<Vec<u8>> {
        self.project.resolve_project_file_with(id, |path| {
            Ok(self
                .storage
                .get(&(path.store_ref.clone(), path.path.clone()))
                .cloned())
        })
    }
}

impl FileResolver for MemoryStorageResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        Ok(Cow::Owned(Bytes::new(self.read(id)?)))
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        Ok(Cow::Owned(source_from_bytes(id, &self.read(id)?)?))
    }
}

#[test]
fn nested_imports_data_and_images_from_asset_roots_export_in_all_formats() {
    let files = LoadedFiles::default();
    let storage = [
        (
            "brand-store",
            "brand/templates/report.typ",
            "#import \"helper.typ\": title\n#let report(body) = [#title\n#image(\"../images/logo.svg\", width: 12pt)\n#body]",
        ),
        (
            "shared-store",
            "shared/templates/report.typ",
            "#panic(\"the first matching template must win\")",
        ),
        (
            "shared-store",
            "shared/templates/helper.typ",
            "#let title = read(\"../data/title.txt\")",
        ),
        ("shared-store", "shared/data/title.txt", "Shared title"),
        (
            "shared-store",
            "shared/images/logo.svg",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>"#,
        ),
    ]
    .into_iter()
    .map(|(store, path, text)| ((store.to_string(), path.to_string()), text.as_bytes().to_vec()))
    .collect();
    let resolver = MemoryStorageResolver {
        project: FlowPathProjectResolver::new(
            Some(root("project", "project-store")),
            vec![root("brand", "brand-store"), root("shared", "shared-store")],
            files.clone(),
        ),
        storage,
    };
    let main_path = "documents/main.typ";
    let source = format!(
        "#set page(width: 100pt, height: 80pt, margin: 4pt)\n{}",
        apply_template_source("../templates/report.typ", "report", "Body").unwrap()
    );
    let engine = TypstEngine::builder()
        .fonts(typst_assets::fonts())
        .with_static_source_file_resolver([(file_id(main_path), source)])
        .add_file_resolver(resolver)
        .build();
    let warned = engine.compile_with_input(main_path, Dict::new());
    let compiled = CompiledTypstDocument {
        document: warned.output.expect("nested project should compile"),
        warnings: Vec::new(),
    };
    assert!(warned.warnings.is_empty(), "{:?}", warned.warnings);
    assert_eq!(compiled.document.pages.len(), 1);
    assert!(export_pdf(&compiled).unwrap().starts_with(b"%PDF-"));
    assert!(export_svg(&compiled, 0.0).contains("<svg"));
    assert!(export_png(
        &compiled,
        &PngOptions {
            ppi: 72.0,
            page: Some(1)
        }
    )
    .unwrap()
    .starts_with(b"\x89PNG\r\n\x1a\n"));
    let loaded = files.lock().unwrap();
    for path in [
        "templates/report.typ",
        "templates/helper.typ",
        "data/title.txt",
        "images/logo.svg",
    ] {
        assert!(loaded.contains_key(&file_id(path)), "{path} was not loaded");
    }
}
