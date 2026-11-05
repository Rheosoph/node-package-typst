//! Regenerate the checked-in Markdown-to-Typst example artifacts.
//!
//! Run on the native host because the workspace's default Cargo target is WASM:
//!
//! ```text
//! cargo run --target $(rustc -vV | awk '/^host:/ {print $2}') --example render_examples
//! ```

#[allow(dead_code)]
#[path = "../src/chart.rs"]
mod chart;
#[allow(dead_code)]
#[path = "../src/markdown.rs"]
mod markdown;
#[allow(dead_code)]
#[path = "../src/shared.rs"]
mod shared;

use shared::{CompiledTypstDocument, TypstProject};
use std::{env, fs, path::PathBuf, process};
use typst::{
    foundations::{Array, Dict, Str, Value},
    layout::PagedDocument,
    syntax::{FileId, VirtualPath},
};
use typst_as_lib::TypstEngine;

const BODY_PLACEHOLDER: &str = "{markdown_body}";

#[derive(Clone, Copy)]
enum Mode {
    Standalone,
    FormatString,
    ImportedTemplate,
}

#[derive(Clone, Copy)]
struct Case {
    directory: &'static str,
    mode: Mode,
}

const CASES: &[Case] = &[
    Case {
        directory: "standalone/core-markdown",
        mode: Mode::Standalone,
    },
    Case {
        directory: "standalone/rich-markdown",
        mode: Mode::Standalone,
    },
    Case {
        directory: "standalone/directives",
        mode: Mode::Standalone,
    },
    Case {
        directory: "standalone/safety-and-degradation",
        mode: Mode::Standalone,
    },
    Case {
        directory: "charts/nivo-csv-core",
        mode: Mode::Standalone,
    },
    Case {
        directory: "charts/nivo-csv-extended",
        mode: Mode::Standalone,
    },
    Case {
        directory: "charts/nivo-json",
        mode: Mode::Standalone,
    },
    Case {
        directory: "charts/plotly-csv",
        mode: Mode::Standalone,
    },
    Case {
        directory: "charts/plotly-json",
        mode: Mode::Standalone,
    },
    Case {
        directory: "charts/fallbacks",
        mode: Mode::Standalone,
    },
    Case {
        directory: "format-string/executive-report",
        mode: Mode::FormatString,
    },
    Case {
        directory: "format-string/technical-brief",
        mode: Mode::FormatString,
    },
    Case {
        directory: "format-string/product-one-pager",
        mode: Mode::FormatString,
    },
    Case {
        directory: "template",
        mode: Mode::ImportedTemplate,
    },
];

struct Artifact {
    path: PathBuf,
    bytes: Vec<u8>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("example generation failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let check = match arguments.as_slice() {
        [] => false,
        [argument] if argument == "--check" => true,
        _ => return Err("usage: render_examples [--check]".to_string()),
    };

    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut artifacts = Vec::new();

    for case in CASES {
        let directory = examples.join(case.directory);
        let markdown = read_text(directory.join("input.md"))?;
        let body = markdown::markdown_to_typst(&markdown)
            .map_err(|error| format!("{}: {error}", case.directory))?;

        let (document, document_name, compiled) = match case.mode {
            Mode::Standalone => {
                let compiled = compile(&body)?;
                (body.clone(), "document.typ", compiled)
            }
            Mode::FormatString => {
                let layout = read_text(directory.join("layout.typ"))?;
                let document = insert_body(&layout, &body, case.directory)?;
                let compiled = compile(&document)?;
                (document, "document.typ", compiled)
            }
            Mode::ImportedTemplate => {
                let document = format!(
                    "#import \"report.typ\": report as __flowlike_template\n#show: __flowlike_template.with(..sys.inputs)\n\n{body}\n"
                );
                let report = read_text(directory.join("report.typ"))?;
                let data = read_text(directory.join("data.json"))?;
                let compiled = compile_imported_template(&document, &report, &data)?;
                (document, "resolved-document.typ", compiled)
            }
        };

        let pdf = shared::export_pdf(&compiled)
            .map_err(|error| format!("{} PDF: {error}", case.directory))?;
        let svg = shared::export_svg(&compiled, 0.0);
        artifacts.extend([
            Artifact {
                path: directory.join("generated-body.typ"),
                bytes: body.into_bytes(),
            },
            Artifact {
                path: directory.join(document_name),
                bytes: document.into_bytes(),
            },
            Artifact {
                path: directory.join("output.pdf"),
                bytes: pdf,
            },
            Artifact {
                path: directory.join("output.svg"),
                bytes: svg.into_bytes(),
            },
        ]);
        println!(
            "rendered {:<42} {:>2} page(s)",
            case.directory,
            compiled.document.pages.len()
        );
    }

    if check {
        check_artifacts(&artifacts)?;
        println!("all {} generated artifacts are current", artifacts.len());
    } else {
        write_artifacts(&artifacts)?;
        println!("wrote {} generated artifacts", artifacts.len());
    }

    Ok(())
}

fn read_text(path: PathBuf) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|error| format!("could not read {}: {error}", path.display()))
}

fn insert_body(layout: &str, body: &str, case: &str) -> Result<String, String> {
    let placeholder_count = layout.match_indices(BODY_PLACEHOLDER).count();
    if placeholder_count != 1 {
        return Err(format!(
            "{case}/layout.typ must contain exactly one {BODY_PLACEHOLDER} placeholder; found {placeholder_count}"
        ));
    }
    Ok(layout.replace(BODY_PLACEHOLDER, body))
}

fn compile(source: &str) -> Result<CompiledTypstDocument, String> {
    shared::compile_project(TypstProject {
        main_path: "document.typ".to_string(),
        source: source.to_string(),
        source_folder: None,
        asset_roots: Vec::new(),
        allow_universe_packages: false,
        fonts: Vec::new(),
    })
}

fn compile_imported_template(
    document: &str,
    report: &str,
    data: &str,
) -> Result<CompiledTypstDocument, String> {
    let data: serde_json::Value = serde_json::from_str(data)
        .map_err(|error| format!("invalid template/data.json: {error}"))?;
    let serde_json::Value::Object(values) = data else {
        return Err("template/data.json must contain a JSON object".to_string());
    };
    let mut inputs = Dict::new();
    for (key, value) in values {
        inputs.insert(Str::from(key), json_to_typst(value)?);
    }

    let main_path = "resolved-document.typ";
    let engine = TypstEngine::builder()
        .fonts(typst_assets::fonts())
        .with_static_source_file_resolver([
            (file_id(main_path), document.to_string()),
            (file_id("report.typ"), report.to_string()),
        ])
        .build();
    let warned = engine.compile_with_input(main_path, inputs);
    let document: PagedDocument = warned
        .output
        .map_err(|error| format!("Typst template compilation failed: {error:?}"))?;
    let warnings = warned
        .warnings
        .iter()
        .map(|warning| format!("{warning:?}"))
        .collect();
    Ok(CompiledTypstDocument { document, warnings })
}

fn file_id(path: &str) -> FileId {
    FileId::new(None, VirtualPath::new(path))
}

fn json_to_typst(value: serde_json::Value) -> Result<Value, String> {
    match value {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(value) => Ok(Value::Bool(value)),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(Value::Int(value))
            } else {
                value.as_f64().map(Value::Float).ok_or_else(|| {
                    "template data number cannot be represented in Typst".to_string()
                })
            }
        }
        serde_json::Value::String(value) => Ok(Value::Str(Str::from(value))),
        serde_json::Value::Array(values) => {
            let mut array = Array::new();
            for value in values {
                array.push(json_to_typst(value)?);
            }
            Ok(Value::Array(array))
        }
        serde_json::Value::Object(values) => {
            let mut dict = Dict::new();
            for (key, value) in values {
                dict.insert(Str::from(key), json_to_typst(value)?);
            }
            Ok(Value::Dict(dict))
        }
    }
}

fn write_artifacts(artifacts: &[Artifact]) -> Result<(), String> {
    for artifact in artifacts {
        if let Some(parent) = artifact.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
        }
        fs::write(&artifact.path, &artifact.bytes)
            .map_err(|error| format!("could not write {}: {error}", artifact.path.display()))?;
    }
    Ok(())
}

fn check_artifacts(artifacts: &[Artifact]) -> Result<(), String> {
    let mut stale = Vec::new();
    for artifact in artifacts {
        match fs::read(&artifact.path) {
            Ok(existing) if existing == artifact.bytes => {}
            Ok(_) => stale.push(format!("{} differs", artifact.path.display())),
            Err(error) => stale.push(format!("{} is missing: {error}", artifact.path.display())),
        }
    }
    if stale.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "generated examples are stale:\n{}",
            stale.join("\n")
        ))
    }
}
