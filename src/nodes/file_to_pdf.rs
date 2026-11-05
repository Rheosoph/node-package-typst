use crate::shared::*;
use flow_like_wasm_sdk::*;

#[register_node]
#[derive(Default)]
pub struct CompileTypstFileToPdfNode;

impl WasmNode for CompileTypstFileToPdfNode {
    fn get_node(&self) -> NodeDefinition {
        let mut node = NodeDefinition::new(
            "typst_file_to_pdf",
            "Typst File to PDF",
            "Reads a Typst source FlowPath, compiles it into PDF, and writes the PDF to storage",
            CATEGORY,
        );
        node.add_input_pin("exec", "Exec", "Trigger pin", VariableType::Execution);
        add_source_file_pin(&mut node);
        add_source_folder_pin(&mut node);
        add_asset_roots_pin(&mut node);
        add_font_files_pin(&mut node);
        add_package_pin(&mut node);
        add_output_file_pin(&mut node, "FlowPath file where the PDF will be written");
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
        add_report_output(&mut node);
        node.set_scores(typst_scores());
        node.set_long_running(true);
        node.add_permission(NodePermission::NetworkHttp);
        node.add_permission(NodePermission::StorageRead);
        node.add_permission(NodePermission::StorageWrite);
        node
    }

    fn run(&self, mut ctx: Context) -> ExecutionResult {
        let project = match read_file_project(&ctx) {
            Ok(project) => project,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf", &error));
                return ctx.fail(error);
            }
        };
        let compiled = match compile_project(project) {
            Ok(compiled) => compiled,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf", &error));
                return ctx.fail(error);
            }
        };

        let pdf = match export_pdf(&compiled) {
            Ok(pdf) => pdf,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf", &error));
                return ctx.fail(error);
            }
        };

        let output_file = match read_output_file(&ctx) {
            Ok(output_file) => output_file,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf", &error));
                return ctx.fail(error);
            }
        };

        match write_output(&ctx, &output_file, &pdf) {
            Ok(()) => ctx.set_output_json("pdf_file", &output_file),
            Err(error) => {
                ctx.set_output_json("report", &failure_report("pdf", &error));
                return ctx.fail(error);
            }
        }

        ctx.set_output_json("report", &report(&compiled, "pdf", pdf.len()));
        ctx.success()
    }
}
