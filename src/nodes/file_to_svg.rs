use crate::shared::*;
use flow_like_wasm_sdk::*;

#[register_node]
#[derive(Default)]
pub struct CompileTypstFileToSvgNode;

impl WasmNode for CompileTypstFileToSvgNode {
    fn get_node(&self) -> NodeDefinition {
        let mut node = NodeDefinition::new(
            "typst_file_to_svg",
            "Typst File to SVG",
            "Reads a Typst source FlowPath, compiles it into SVG, and writes it to storage",
            CATEGORY,
        );
        node.add_input_pin("exec", "Exec", "Trigger pin", VariableType::Execution);
        add_source_file_pin(&mut node);
        add_source_folder_pin(&mut node);
        add_package_pin(&mut node);
        add_svg_padding_pin(&mut node);
        add_output_file_pin(&mut node, "FlowPath file where the SVG will be written");
        node.add_output_pin(
            "exec_out",
            "Done",
            "Execution continues",
            VariableType::Execution,
        );
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
        let project = match read_file_project(&ctx) {
            Ok(project) => project,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("svg", &error));
                return ctx.fail(error);
            }
        };
        let compiled = match compile_project(project) {
            Ok(compiled) => compiled,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("svg", &error));
                return ctx.fail(error);
            }
        };

        let padding_pt = ctx.get_f64("padding_pt").unwrap_or(0.0);
        let svg = export_svg(&compiled, padding_pt);

        let output_file = match read_output_file(&ctx) {
            Ok(output_file) => output_file,
            Err(error) => {
                ctx.set_output_json("report", &failure_report("svg", &error));
                return ctx.fail(error);
            }
        };

        match write_output(&ctx, &output_file, svg.as_bytes()) {
            Ok(()) => ctx.set_output_json("svg_file", &output_file),
            Err(error) => {
                ctx.set_output_json("report", &failure_report("svg", &error));
                return ctx.fail(error);
            }
        }

        ctx.set_output_json("report", &report(&compiled, "svg", svg.len()));
        ctx.activate_exec("exec_out");
        ctx.success()
    }
}
