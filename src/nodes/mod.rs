mod file_to_pdf;
mod file_to_pdf_svg;
mod file_to_png;
mod file_to_svg;
mod markdown_to_typst;
mod source_to_pdf;
mod source_to_pdf_svg;
mod source_to_png;
mod source_to_svg;
mod template_file_to_pdf;
mod template_file_to_pdf_svg;
mod template_file_to_png;
mod template_file_to_svg;
mod template_to_pdf;
mod template_to_pdf_svg;
mod template_to_png;
mod template_to_svg;
mod validate_file;
mod validate_source;

use flow_like_wasm_sdk::{Context, FlowPath};

pub(crate) fn read_pdf_svg_output_files(ctx: &Context) -> Result<(FlowPath, FlowPath), String> {
    let pdf_output_file = crate::shared::read_flow_path_pin(ctx, "pdf_output_file")?;
    let svg_output_file = crate::shared::read_flow_path_pin(ctx, "svg_output_file")?;

    if pdf_output_file.store_ref == svg_output_file.store_ref
        && pdf_output_file.path == svg_output_file.path
        && pdf_output_file.cache_store_ref == svg_output_file.cache_store_ref
    {
        return Err(
            "pdf_output_file and svg_output_file must refer to different files".to_string(),
        );
    }

    Ok((pdf_output_file, svg_output_file))
}

#[cfg(test)]
pub(crate) use file_to_pdf::CompileTypstFileToPdfNode;
#[cfg(test)]
pub(crate) use file_to_pdf_svg::CompileTypstFileToPdfSvgNode;
#[cfg(test)]
pub(crate) use file_to_png::CompileTypstFileToPngNode;
#[cfg(test)]
pub(crate) use file_to_svg::CompileTypstFileToSvgNode;
#[cfg(test)]
pub(crate) use markdown_to_typst::MarkdownToTypstNode;
#[cfg(test)]
pub(crate) use source_to_pdf::CompileTypstSourceToPdfNode;
#[cfg(test)]
pub(crate) use source_to_pdf_svg::CompileTypstSourceToPdfSvgNode;
#[cfg(test)]
pub(crate) use source_to_png::CompileTypstSourceToPngNode;
#[cfg(test)]
pub(crate) use source_to_svg::CompileTypstSourceToSvgNode;
#[cfg(test)]
pub(crate) use template_file_to_pdf::CompileTypstTemplateFileToPdfNode;
#[cfg(test)]
pub(crate) use template_file_to_pdf_svg::CompileTypstTemplateFileToPdfSvgNode;
#[cfg(test)]
pub(crate) use template_file_to_png::CompileTypstTemplateFileToPngNode;
#[cfg(test)]
pub(crate) use template_file_to_svg::CompileTypstTemplateFileToSvgNode;
#[cfg(test)]
pub(crate) use template_to_pdf::CompileTypstTemplateToPdfNode;
#[cfg(test)]
pub(crate) use template_to_pdf_svg::CompileTypstTemplateToPdfSvgNode;
#[cfg(test)]
pub(crate) use template_to_png::CompileTypstTemplateToPngNode;
#[cfg(test)]
pub(crate) use template_to_svg::CompileTypstTemplateToSvgNode;
#[cfg(test)]
pub(crate) use validate_file::ValidateTypstFileNode;
#[cfg(test)]
pub(crate) use validate_source::ValidateTypstSourceNode;

#[cfg(test)]
mod output_file_tests {
    use super::*;
    use flow_like_wasm_sdk::ExecutionInput;
    use serde_json::{json, Map};

    fn context(pdf_output_file: FlowPath, svg_output_file: FlowPath) -> Context {
        let mut inputs = Map::new();
        inputs.insert("pdf_output_file".to_string(), json!(pdf_output_file));
        inputs.insert("svg_output_file".to_string(), json!(svg_output_file));
        Context::from_input(ExecutionInput {
            inputs,
            node_id: "test-node".to_string(),
            run_id: "test-run".to_string(),
            app_id: "test-app".to_string(),
            board_id: "test-board".to_string(),
            user_id: "test-user".to_string(),
            stream_state: false,
            log_level: 0,
            node_name: "test-node".to_string(),
        })
    }

    fn flow_path(path: &str, cache_store_ref: Option<&str>) -> FlowPath {
        FlowPath::new(
            path.to_string(),
            "store".to_string(),
            cache_store_ref.map(str::to_string),
        )
    }

    #[test]
    fn combined_outputs_reject_the_same_flow_path() {
        let path = flow_path("reports/result.pdf", Some("cache"));
        let error = read_pdf_svg_output_files(&context(path.clone(), path)).unwrap_err();

        assert!(error.contains("must refer to different files"));
    }

    #[test]
    fn combined_outputs_allow_distinct_flow_paths() {
        let result = read_pdf_svg_output_files(&context(
            flow_path("reports/result.pdf", Some("cache")),
            flow_path("reports/result.svg", Some("cache")),
        ));

        assert!(result.is_ok());
    }

    #[test]
    fn combined_outputs_include_cache_store_in_identity() {
        let result = read_pdf_svg_output_files(&context(
            flow_path("reports/result", Some("pdf-cache")),
            flow_path("reports/result", Some("svg-cache")),
        ));

        assert!(result.is_ok());
    }
}
