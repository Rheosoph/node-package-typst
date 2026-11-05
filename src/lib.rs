//! FlowLike Typst WASM nodes.
//!
//! Node definitions live in `src/nodes/*`; shared Typst compile/export code
//! lives in `src/shared.rs`.

mod chart;
mod markdown;
mod nodes;
mod shared;

use flow_like_wasm_sdk::wasm_main;

wasm_main!();

#[cfg(test)]
mod tests {
    use super::nodes::{
        CompileTypstFileToPdfNode, CompileTypstFileToPdfSvgNode, CompileTypstFileToPngNode,
        CompileTypstFileToSvgNode, CompileTypstSourceToPdfNode, CompileTypstSourceToPdfSvgNode,
        CompileTypstSourceToPngNode, CompileTypstSourceToSvgNode,
        CompileTypstTemplateFileToPdfNode, CompileTypstTemplateFileToPdfSvgNode,
        CompileTypstTemplateFileToPngNode, CompileTypstTemplateFileToSvgNode,
        CompileTypstTemplateToPdfNode, CompileTypstTemplateToPdfSvgNode,
        CompileTypstTemplateToPngNode, CompileTypstTemplateToSvgNode, MarkdownToTypstNode,
        ValidateTypstFileNode, ValidateTypstSourceNode,
    };
    use flow_like_wasm_sdk::*;

    fn assert_common_typst_node(node: &NodeDefinition) {
        assert!(
            node.category.starts_with("Document/Typst"),
            "{} should live under Document/Typst",
            node.name
        );
        assert!(!node.description.trim().is_empty());
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.pin_type == PinType::Input && pin.data_type == VariableType::Execution));
        assert!(
            node.pins
                .iter()
                .any(|pin| pin.pin_type == PinType::Output
                    && pin.data_type == VariableType::Execution)
        );
    }

    fn assert_template_node(node: &NodeDefinition) {
        assert_common_typst_node(node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "template_data" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "template_file"));
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "template_function" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn markdown_to_typst_node_definition() {
        let node = MarkdownToTypstNode.get_node();
        assert_eq!(node.name, "typst_markdown_to_typst");
        assert_eq!(node.category, "Document/Typst");
        assert!(!node.description.trim().is_empty());
        assert!(node.pins.iter().any(|pin| {
            pin.name == "markdown"
                && pin.pin_type == PinType::Input
                && pin.data_type == VariableType::String
        }));
        assert!(node.pins.iter().any(|pin| {
            pin.name == "typst_source"
                && pin.pin_type == PinType::Output
                && pin.data_type == VariableType::String
        }));
        assert!(!node
            .pins
            .iter()
            .any(|pin| pin.data_type == VariableType::Execution));
        assert!(node.permissions.is_empty());
    }

    #[test]
    fn source_to_pdf_node_definition() {
        let node = CompileTypstSourceToPdfNode.get_node();
        assert_eq!(node.name, "typst_source_to_pdf");
        assert_common_typst_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
        assert!(!node.pins.iter().any(|pin| pin.name == "pdf_base64"));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn file_to_pdf_node_definition() {
        let node = CompileTypstFileToPdfNode.get_node();
        assert_eq!(node.name, "typst_file_to_pdf");
        assert_common_typst_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn file_to_svg_node_definition() {
        let node = CompileTypstFileToSvgNode.get_node();
        assert_eq!(node.name, "typst_file_to_svg");
        assert_common_typst_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "padding_pt"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn source_to_svg_node_definition() {
        let node = CompileTypstSourceToSvgNode.get_node();
        assert_eq!(node.name, "typst_source_to_svg");
        assert_common_typst_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
        assert!(!node.pins.iter().any(|pin| pin.name == "svg_text"));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn source_to_pdf_svg_node_definition() {
        let node = CompileTypstSourceToPdfSvgNode.get_node();
        assert_eq!(node.name, "typst_source_to_pdf_svg");
        assert_common_typst_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "pdf_output_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "svg_output_file"));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn file_to_pdf_svg_node_definition() {
        let node = CompileTypstFileToPdfSvgNode.get_node();
        assert_eq!(node.name, "typst_file_to_pdf_svg");
        assert_common_typst_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "pdf_output_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "svg_output_file"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    fn assert_png_pins(node: &NodeDefinition) {
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "ppi" && pin.data_type == VariableType::Float));
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "page" && pin.data_type == VariableType::Integer));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "png_file" && pin.pin_type == PinType::Output));
    }

    #[test]
    fn source_to_png_node_definition() {
        let node = CompileTypstSourceToPngNode.get_node();
        assert_eq!(node.name, "typst_source_to_png");
        assert_common_typst_node(&node);
        assert_png_pins(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn file_to_png_node_definition() {
        let node = CompileTypstFileToPngNode.get_node();
        assert_eq!(node.name, "typst_file_to_png");
        assert_common_typst_node(&node);
        assert_png_pins(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn template_to_png_node_definition() {
        let node = CompileTypstTemplateToPngNode.get_node();
        assert_eq!(node.name, "typst_template_to_png");
        assert_template_node(&node);
        assert_png_pins(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
    }

    #[test]
    fn template_file_to_png_node_definition() {
        let node = CompileTypstTemplateFileToPngNode.get_node();
        assert_eq!(node.name, "typst_template_file_to_png");
        assert_template_node(&node);
        assert_png_pins(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
    }

    #[test]
    fn template_to_pdf_node_definition() {
        let node = CompileTypstTemplateToPdfNode.get_node();
        assert_eq!(node.name, "typst_template_to_pdf");
        assert_template_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
    }

    #[test]
    fn template_file_to_pdf_node_definition() {
        let node = CompileTypstTemplateFileToPdfNode.get_node();
        assert_eq!(node.name, "typst_template_file_to_pdf");
        assert_template_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
    }

    #[test]
    fn template_to_svg_node_definition() {
        let node = CompileTypstTemplateToSvgNode.get_node();
        assert_eq!(node.name, "typst_template_to_svg");
        assert_template_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "padding_pt"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
    }

    #[test]
    fn template_file_to_svg_node_definition() {
        let node = CompileTypstTemplateFileToSvgNode.get_node();
        assert_eq!(node.name, "typst_template_file_to_svg");
        assert_template_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "padding_pt"));
        assert!(node.pins.iter().any(|pin| pin.name == "output_file"));
    }

    #[test]
    fn template_to_pdf_svg_node_definition() {
        let node = CompileTypstTemplateToPdfSvgNode.get_node();
        assert_eq!(node.name, "typst_template_to_pdf_svg");
        assert_template_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "pdf_output_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "svg_output_file"));
    }

    #[test]
    fn template_file_to_pdf_svg_node_definition() {
        let node = CompileTypstTemplateFileToPdfSvgNode.get_node();
        assert_eq!(node.name, "typst_template_file_to_pdf_svg");
        assert_template_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "pdf_output_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "svg_output_file"));
    }

    #[test]
    fn validate_source_node_definition() {
        let node = ValidateTypstSourceNode.get_node();
        assert_eq!(node.name, "typst_validate_source");
        assert_common_typst_node(&node);
        assert!(node
            .pins
            .iter()
            .any(|pin| pin.name == "source" && pin.data_type == VariableType::String));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "valid"));
        assert!(!node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn validate_file_node_definition() {
        let node = ValidateTypstFileNode.get_node();
        assert_eq!(node.name, "typst_validate_file");
        assert_common_typst_node(&node);
        assert!(node.pins.iter().any(|pin| pin.name == "source_file"));
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.pins.iter().any(|pin| pin.name == "valid"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(!node.permissions.contains(&NodePermission::StorageWrite));
    }

    #[test]
    fn successful_node_execution_activates_exec_out_once() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("source".to_string(), json!("Hello from Typst"));
        inputs.insert("allow_universe_packages".to_string(), json!(false));
        let ctx = Context::from_input(ExecutionInput {
            inputs,
            node_id: "test-node".to_string(),
            run_id: "test-run".to_string(),
            app_id: "test-app".to_string(),
            board_id: "test-board".to_string(),
            user_id: "test-user".to_string(),
            stream_state: false,
            log_level: 0,
            node_name: "typst_validate_source".to_string(),
        });

        let result = ValidateTypstSourceNode.run(ctx);

        assert!(result.error.is_none());
        assert_eq!(result.activate_exec, vec!["exec_out"]);
        assert_eq!(result.outputs.get("valid"), Some(&json!(true)));
    }
}
