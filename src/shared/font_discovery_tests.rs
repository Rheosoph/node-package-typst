use super::*;

const CUSTOM_FONT: &[u8] = include_bytes!("../../tests/fixtures/fonts/texgyrecursor-regular.otf");

fn path(path: &str, store: &str) -> FlowPath {
    FlowPath::new(path.to_string(), store.to_string(), None)
}

fn project(source: &str) -> TypstProject {
    TypstProject {
        main_path: MAIN_FILE.to_string(),
        source: source.to_string(),
        source_folder: None,
        asset_roots: Vec::new(),
        allow_universe_packages: false,
        fonts: Vec::new(),
    }
}

fn text_fonts(frame: &Frame, fonts: &mut Vec<Font>) {
    for (_, item) in frame.items() {
        match item {
            FrameItem::Text(text) => fonts.push(text.font.clone()),
            FrameItem::Group(group) => text_fonts(&group.frame, fonts),
            _ => {}
        }
    }
}

// Trailing bytes leave the font's tables intact but distinguish its data from
// an otherwise equally matching face in another source.
fn marked_font(marker: u8) -> Font {
    let mut data = CUSTOM_FONT.to_vec();
    data.push(marker);
    Font::new(Bytes::new(data), 0).unwrap()
}

#[test]
fn nested_fonts_are_discovered_without_reading_other_assets() {
    let source = path("project/", "documents");
    let asset_root = FlowPath::new(
        "My Fonts".to_string(),
        "brand".to_string(),
        Some("brand-cache".to_string()),
    );
    let ttf = typst_assets::fonts()
        .find(|data| data.starts_with(&[0, 1, 0, 0]))
        .unwrap();
    let mut listed = Vec::new();
    let mut read_paths = Vec::new();
    let found = fonts::discover(
        Some(&source),
        &[asset_root.clone()],
        |root| {
            listed.push(root.path.clone());
            match root.store_ref.as_str() {
                "documents" => Ok(vec![
                    root.child("logo.svg"),
                    root.child("fonts/deep/custom.OtF"),
                    root.child("document.typ"),
                    root.child("fonts/license.txt"),
                    root.child("font.ttf.backup"),
                ]),
                "brand" => Ok(vec![FlowPath::new(
                    "My%20Fonts/nested/mono.TTF".to_string(),
                    root.store_ref.clone(),
                    root.cache_store_ref.clone(),
                )]),
                _ => panic!("unexpected store"),
            }
        },
        |file| {
            read_paths.push(file.path.clone());
            match file.store_ref.as_str() {
                "documents" => {
                    assert_eq!(file.path, "project/fonts/deep/custom.OtF");
                    Some(CUSTOM_FONT.to_vec())
                }
                "brand" => {
                    assert_eq!(file.path, "My%20Fonts/nested/mono.TTF");
                    assert_eq!(file.cache_store_ref, asset_root.cache_store_ref);
                    Some(ttf.to_vec())
                }
                _ => panic!("unexpected store"),
            }
        },
    );
    assert_eq!(listed, ["project", "My Fonts"]);
    assert_eq!(read_paths.len(), 2);
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    assert_eq!(found.fonts.len(), 2);
    assert_eq!(found.fonts[0].info().family, "TeX Gyre Cursor");
    assert_eq!(found.fonts[1].info().family, "DejaVu Sans Mono");
}

#[test]
fn root_and_filename_order_is_stable_and_explicit_fonts_take_priority() {
    let source = path("source", "store");
    let roots = [path("brand", "store"), path("shared", "store")];
    let explicit = marked_font(0);
    let expected = (1..=4).map(marked_font).collect::<Vec<_>>();
    let mut listed = Vec::new();
    let found = fonts::discover(
        Some(&source),
        &roots,
        |root| {
            listed.push(root.path.clone());
            Ok(if root.path == "source" {
                vec![root.child("z.otf"), root.child("a.otf")]
            } else {
                vec![root.child("font.otf")]
            })
        },
        |file| {
            let marker = match file.path.as_str() {
                "source/a.otf" => 1,
                "source/z.otf" => 2,
                "brand/font.otf" => 3,
                "shared/font.otf" => 4,
                other => panic!("unexpected path: {other}"),
            };
            Some(marked_font(marker).data().to_vec())
        },
    );
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    assert_eq!(listed, ["source", "brand", "shared"]);
    assert_eq!(found.fonts, expected);

    // Verify Typst's actual selected text font, not just vector insertion order.
    for include_explicit in [false, true] {
        let mut document = project("#set text(font: \"TeX Gyre Cursor\")\nHello");
        document.source_folder = Some(source.clone());
        document.asset_roots = roots.to_vec();
        if include_explicit {
            document.fonts.push(explicit.clone());
        }
        let compiled = compile_with_font_io(
            document,
            None,
            MainFileLabel::default(),
            |root| Ok(vec![root.child("font.otf")]),
            |file| {
                Some(
                    marked_font(match file.path.as_str() {
                        "source/font.otf" => 1,
                        "brand/font.otf" => 3,
                        "shared/font.otf" => 4,
                        _ => unreachable!(),
                    })
                    .data()
                    .to_vec(),
                )
            },
        )
        .unwrap();
        assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
        let mut selected = Vec::new();
        text_fonts(&compiled.document.pages[0].frame, &mut selected);
        assert!(!selected.is_empty());
        let expected = if include_explicit {
            &explicit
        } else {
            &expected[0]
        };
        assert!(selected.iter().all(|font| font == expected));
    }
}

#[test]
fn duplicate_roots_files_and_contents_are_registered_once() {
    let root = path("project", "store");
    let mut lists = 0;
    let mut reads = Vec::new();
    let found = fonts::discover(
        Some(&root),
        &[path("project/", "store"), root.child("fonts")],
        |folder| {
            lists += 1;
            Ok(if folder.path == "project" {
                vec![folder.child("fonts/a.otf"), folder.child("fonts/a.otf")]
            } else {
                vec![folder.child("copy.otf"), folder.child("a.otf")]
            })
        },
        |file| {
            reads.push(file.path.clone());
            Some(CUSTOM_FONT.to_vec())
        },
    );
    assert_eq!(lists, 2);
    assert_eq!(reads, ["project/fonts/a.otf", "project/fonts/copy.otf"]);
    assert_eq!(found.fonts.len(), 1);
    let font = found.fonts[0].clone();
    let combined = fonts::combine(vec![font.clone()], found.fonts);
    assert_eq!(
        combined
            .iter()
            .filter(|candidate| **candidate == font)
            .count(),
        1
    );
}

#[test]
fn collection_faces_survive_font_deduplication() {
    // Construct a two-face TTC that shares one valid OpenType table directory.
    // Table offsets in a collection are absolute from the collection start.
    let offset = 20u32;
    let mut collection = b"ttcf\0\x01\0\0".to_vec();
    collection.extend_from_slice(&2u32.to_be_bytes());
    collection.extend_from_slice(&offset.to_be_bytes());
    collection.extend_from_slice(&offset.to_be_bytes());
    collection.extend_from_slice(CUSTOM_FONT);
    let table_count = u16::from_be_bytes([CUSTOM_FONT[4], CUSTOM_FONT[5]]) as usize;
    for table in 0..table_count {
        let index = offset as usize + 12 + table * 16 + 8;
        let table_offset = u32::from_be_bytes(collection[index..index + 4].try_into().unwrap());
        collection[index..index + 4].copy_from_slice(&(table_offset + offset).to_be_bytes());
    }
    let found = fonts::discover(
        None,
        &[path("assets", "store")],
        |root| Ok(vec![root.child("family.TtC")]),
        |_| Some(collection.clone()),
    );
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    assert_eq!(found.fonts.len(), 2);
    assert_eq!(found.fonts[0].index(), 0);
    assert_eq!(found.fonts[1].index(), 1);
    let combined = fonts::combine(found.fonts.clone(), found.fonts.clone());
    for face in found.fonts {
        assert_eq!(combined.iter().filter(|font| **font == face).count(), 1);
    }
}

#[test]
fn inferred_file_and_template_folders_are_font_sources() {
    let (_, file_root) = file_project_location(&path("project/body.typ", "store"), None).unwrap();
    let (_, storage_root) = file_project_location(&path("body.typ", "store"), None).unwrap();
    let context = Context::from_input(ExecutionInput {
        inputs: serde_json::from_value(json!({
            "template_file": path("templates/report.typ", "templates-store"),
            "source": "Body",
        }))
        .unwrap(),
        node_id: "node".into(),
        run_id: "run".into(),
        app_id: "app".into(),
        board_id: "board".into(),
        user_id: "user".into(),
        stream_state: false,
        log_level: 0,
        node_name: "test".into(),
    });
    let template = read_template_source_project(&context).unwrap();
    let template_root = template.project.source_folder.unwrap();
    for (root, expected) in [
        (file_root, "project"),
        (storage_root, ""),
        (template_root, "templates"),
    ] {
        let found = fonts::discover(
            Some(&root),
            &[],
            |folder| {
                assert_eq!(folder.path, expected);
                Ok(vec![folder.child("fonts/custom.otf")])
            },
            |_| Some(CUSTOM_FONT.to_vec()),
        );
        assert!(found.warnings.is_empty(), "{:?}", found.warnings);
        assert_eq!(found.fonts.len(), 1);
    }
}

#[test]
fn discovered_fonts_render_source_and_template_inputs_in_every_format() {
    for from_assets in [false, true] {
        let root = path("referenzmappe", "template-store");
        let mut document = project(
            "#set page(width: 180pt, height: 90pt, margin: 6pt)\n#set text(font: \"TeX Gyre Cursor\")\nAsset font",
        );
        let inputs = if from_assets {
            document.asset_roots.push(root.clone());
            document.source = document
                .source
                .replace("\"TeX Gyre Cursor\"", "sys.inputs.font");
            let mut inputs = Dict::new();
            inputs.insert(Str::from("font"), Value::Str(Str::from("TeX Gyre Cursor")));
            Some(inputs)
        } else {
            document.source_folder = Some(root.clone());
            None
        };
        let compiled = compile_with_font_io(
            document,
            inputs,
            MainFileLabel::default(),
            |folder| Ok(vec![folder.child("assets/fonts/custom.otf")]),
            |file| {
                assert_eq!(file.path, "referenzmappe/assets/fonts/custom.otf");
                Some(CUSTOM_FONT.to_vec())
            },
        )
        .unwrap();
        assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
        let mut selected = Vec::new();
        text_fonts(&compiled.document.pages[0].frame, &mut selected);
        assert!(!selected.is_empty());
        assert!(selected
            .iter()
            .all(|font| font.info().family == "TeX Gyre Cursor"));
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
    }
}

#[test]
fn discovery_failures_are_reported_without_blocking_other_fonts() {
    let mut document = project("#set text(font: \"TeX Gyre Cursor\")\nStill renders");
    document.source_folder = Some(path("unavailable", "store"));
    document.asset_roots = vec![path("assets", "store")];
    let compiled = compile_with_font_io(
        document,
        None,
        MainFileLabel::default(),
        |root| {
            if root.path == "unavailable" {
                Err("listing failed".to_string())
            } else {
                Ok(vec![
                    root.child("broken.ttf"),
                    root.child("missing.otf"),
                    root.child("valid.otf"),
                ])
            }
        },
        |file| match file.path.as_str() {
            "assets/broken.ttf" => Some(b"not a font".to_vec()),
            "assets/missing.otf" => None,
            "assets/valid.otf" => Some(CUSTOM_FONT.to_vec()),
            other => panic!("unexpected read: {other}"),
        },
    )
    .unwrap();
    assert_eq!(compiled.warnings.len(), 3, "{:?}", compiled.warnings);
    assert!(compiled
        .warnings
        .iter()
        .any(|warning| warning.contains("source_folder") && warning.contains("listing failed")));
    assert!(compiled
        .warnings
        .iter()
        .any(|warning| warning.contains("assets/broken.ttf")
            && warning.contains("no supported fonts")));
    assert!(compiled.warnings.iter().any(
        |warning| warning.contains("assets/missing.otf") && warning.contains("could not read")
    ));
    let report = report(&compiled, "pdf", export_pdf(&compiled).unwrap().len());
    assert!(report.success);
    assert_eq!(report.warning_count, 3);
    assert_eq!(report.warnings, compiled.warnings);
}

#[test]
fn projects_without_roots_keep_bundled_fonts_without_storage_calls() {
    let compiled = compile_with_font_io(
        project("#set text(font: \"Libertinus Serif\")\nDefault font"),
        None,
        MainFileLabel::default(),
        |_| panic!("no roots should be listed"),
        |_| panic!("no fonts should be read"),
    )
    .unwrap();
    assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
    let mut selected = Vec::new();
    text_fonts(&compiled.document.pages[0].frame, &mut selected);
    assert!(!selected.is_empty());
    assert!(selected
        .iter()
        .all(|font| font.info().family == "Libertinus Serif"));
}
