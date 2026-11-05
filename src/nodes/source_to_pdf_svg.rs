use crate::nodes::read_pdf_svg_output_files;
use crate::shared::*;
use flow_like_wasm_sdk::*;

#[register_node]
#[derive(Default)]
pub struct CompileTypstSourceToPdfSvgNode;

impl WasmNode for CompileTypstSourceToPdfSvgNode {
    fn get_node(&self) -> NodeDefinition {
        let mut node = NodeDefinition::new(
            "typst_source_to_pdf_svg",
            "Typst Source to PDF + SVG",
            "Compiles Typst source text once and writes both PDF and SVG outputs",
            CATEGORY,
        );
        node.add_input_pin("exec", "Exec", "Trigger pin", VariableType::Execution);
        add_source_pin(&mut node);
        add_source_folder_pin(&mut node);
        add_asset_roots_pin(&mut node);
        add_font_files_pin(&mut node);
        add_package_pin(&mut node);
        add_svg_padding_pin(&mut node);
        add_pdf_svg_output_file_pins(&mut node);
        node.add_output_pin(
            "exec_out",
            "Done",
            "Execution continues",
            VariableType::Execution,
        );
        node.add_output_pin(
            "pdf_file",
            "PDF File",
            "Written PDF FlowPath",
            VariableType::Struct,
        )
        .set_schema::<FlowPath>();
        node.add_output_pin(
            "svg_file",
            "SVG File",
            "Written SVG FlowPath",
            VariableType::Struct,
        )
        .set_schema::<FlowPath>();
        add_report_output(&mut node);
        node.set_scores(typst_scores());
        node.set_long_running(true);
        node.add_permission(NodePermission::NetworkHttp);
        node.add_permission(NodePermission::StorageRead);
        node.add_permission(NodePermission::StorageWrite);
        node
    }

    fn run(&self, mut ctx: Context) -> ExecutionResult {
        let (pdf_output_file, svg_output_file) = match read_pdf_svg_output_files(&ctx) {
            Ok(output_files) => output_files,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf+svg", &error));
                return ctx.fail(error);
            }
        };

        let project = match read_source_project(&ctx) {
            Ok(project) => project,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf+svg", &error));
                return ctx.fail(error);
            }
        };
        let compiled = match compile_project(project) {
            Ok(compiled) => compiled,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf+svg", &error));
                return ctx.fail(error);
            }
        };

        let pdf = match export_pdf(&compiled) {
            Ok(pdf) => pdf,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf+svg", &error));
                return ctx.fail(error);
            }
        };
        let padding_pt = ctx.get_f64("padding_pt").unwrap_or(0.0);
        let svg = export_svg(&compiled, padding_pt);

        if let Err(error) = write_output(&ctx, &pdf_output_file, &pdf) {
            ctx.set_output_json("report", &failure_report("pdf+svg", &error));
            return ctx.fail(error);
        }
        if let Err(error) = write_output(&ctx, &svg_output_file, svg.as_bytes()) {
            ctx.set_output_json("report", &failure_report("pdf+svg", &error));
            return ctx.fail(error);
        }

        ctx.set_output_json("pdf_file", &pdf_output_file);
        ctx.set_output_json("svg_file", &svg_output_file);
        ctx.set_output_json(
            "report",
            &report(&compiled, "pdf+svg", pdf.len() + svg.len()),
        );
        ctx.success()
    }
}
