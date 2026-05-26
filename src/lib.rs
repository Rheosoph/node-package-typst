//! FlowLike Typst WASM nodes.
//!
//! Node definitions live in `src/nodes/*`; shared Typst compile/export code
//! lives in `src/shared.rs`.

mod nodes;
mod shared;

use flow_like_wasm_sdk::wasm_main;

wasm_main!();

#[cfg(test)]
mod tests {
    use super::nodes::{
        CompileTypstFileToPdfNode, CompileTypstFileToPdfSvgNode, CompileTypstFileToSvgNode,
        CompileTypstSourceToPdfNode, CompileTypstSourceToPdfSvgNode, CompileTypstSourceToSvgNode,
        CompileTypstTemplateFileToPdfNode, CompileTypstTemplateFileToPdfSvgNode,
        CompileTypstTemplateFileToSvgNode, CompileTypstTemplateToPdfNode,
        CompileTypstTemplateToPdfSvgNode, CompileTypstTemplateToSvgNode, ValidateTypstFileNode,
        ValidateTypstSourceNode,
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
        assert!(node.pins.iter().any(|pin| pin.name == "source_folder"));
        assert!(node.permissions.contains(&NodePermission::StorageRead));
        assert!(node.permissions.contains(&NodePermission::StorageWrite));
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
}
