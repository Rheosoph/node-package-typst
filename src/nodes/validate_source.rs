use crate::shared::*;
use flow_like_wasm_sdk::*;

#[register_node]
#[derive(Default)]
pub struct ValidateTypstSourceNode;

impl WasmNode for ValidateTypstSourceNode {
    fn get_node(&self) -> NodeDefinition {
        let mut node = NodeDefinition::new(
            "typst_validate_source",
            "Validate Typst Source",
            "Compiles Typst source text without export and returns diagnostics metadata",
            CATEGORY,
        );
        node.add_input_pin("exec", "Exec", "Trigger pin", VariableType::Execution);
        add_source_pin(&mut node);
        add_source_folder_pin(&mut node);
        add_package_pin(&mut node);
        node.add_output_pin(
            "exec_out",
            "Done",
            "Execution continues",
            VariableType::Execution,
        );
        node.add_output_pin(
            "valid",
            "Valid",
            "Whether Typst compilation succeeded",
            VariableType::Boolean,
        );
        add_report_output(&mut node);
        node.set_scores(typst_scores());
        node.set_long_running(true);
        node.add_permission(NodePermission::NetworkHttp);
        node.add_permission(NodePermission::StorageRead);
        node
    }

    fn run(&self, mut ctx: Context) -> ExecutionResult {
        let project = read_source_project(&ctx);
        match compile_project(project) {
            Ok(compiled) => {
                ctx.set_output("valid", true);
                ctx.set_output_json("report", &report(&compiled, "typst", 0));
                ctx.activate_exec("exec_out");
                ctx.success()
            }
            Err(error) => {
                ctx.set_output("valid", false);
                ctx.set_output_json("report", &failure_report("typst", &error));
                ctx.activate_exec("exec_out");
                ctx.success()
            }
        }
    }
}
