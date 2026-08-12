use flow_like_typst_nodes as _;
use flow_like_wasm_sdk::*;
use std::collections::{BTreeSet, HashSet};

const FLOW_LIKE_TOML: &str = include_str!("../flow-like.toml");

/// Auto-discovers all `#[register_node]` nodes via `inventory`.
fn all_package_nodes() -> Vec<NodeDefinition> {
    inventory::iter::<WasmNodeEntry>
        .into_iter()
        .map(|entry| (entry.get_node)())
        .collect()
}

fn manifest_node_ids() -> Vec<String> {
    let mut in_node = false;
    let mut ids = Vec::new();

    for line in FLOW_LIKE_TOML.lines().map(str::trim) {
        if line.starts_with("[[") {
            in_node = line == "[[nodes]]";
            continue;
        }

        if in_node {
            if let Some(id) = line
                .strip_prefix("id = \"")
                .and_then(|value| value.strip_suffix('"'))
            {
                ids.push(id.to_string());
                in_node = false;
            }
        }
    }

    ids
}

#[test]
fn lint_inventory_matches_manifest() {
    let nodes = all_package_nodes();
    assert!(
        !nodes.is_empty(),
        "no package nodes were linked into the lint test"
    );

    let inventory_ids: BTreeSet<_> = nodes.iter().map(|node| node.name.as_str()).collect();
    assert_eq!(
        inventory_ids.len(),
        nodes.len(),
        "inventory contains duplicate node ids"
    );

    let manifest_ids = manifest_node_ids();
    let unique_manifest_ids: BTreeSet<_> = manifest_ids.iter().map(String::as_str).collect();
    assert_eq!(
        unique_manifest_ids.len(),
        manifest_ids.len(),
        "flow-like.toml contains duplicate node ids"
    );
    assert_eq!(inventory_ids, unique_manifest_ids);
}

#[test]
fn lint_no_duplicate_input_output_pin_names() {
    for node in all_package_nodes() {
        let input_names: HashSet<&str> = node
            .pins
            .iter()
            .filter(|p| p.pin_type == PinType::Input)
            .map(|p| p.name.as_str())
            .collect();

        for pin in node.pins.iter().filter(|p| p.pin_type == PinType::Output) {
            assert!(
                !input_names.contains(pin.name.as_str()),
                "[{}] Input and output pin share the name \"{}\"",
                node.name,
                pin.name
            );
        }
    }
}

#[test]
fn lint_impure_nodes_have_both_exec_sides() {
    for node in all_package_nodes() {
        let has_input_exec = node
            .pins
            .iter()
            .any(|p| p.pin_type == PinType::Input && p.data_type == VariableType::Execution);
        let has_output_exec = node
            .pins
            .iter()
            .any(|p| p.pin_type == PinType::Output && p.data_type == VariableType::Execution);

        if has_input_exec || has_output_exec {
            assert!(
                has_input_exec && has_output_exec,
                "[{}] Impure node must have both input and output exec pins",
                node.name
            );
        }
    }
}

#[test]
fn lint_struct_pins_have_schema() {
    for node in all_package_nodes() {
        for pin in &node.pins {
            if pin.data_type == VariableType::Struct {
                assert!(
                    pin.schema
                        .as_ref()
                        .is_some_and(|schema| !schema.trim().is_empty()),
                    "[{}] Struct pin \"{}\" ({:?}) has no schema",
                    node.name,
                    pin.name,
                    pin.pin_type
                );
            }
        }
    }
}

#[test]
fn lint_no_root_array_schemas() {
    for node in all_package_nodes() {
        for pin in &node.pins {
            if let Some(schema_str) = &pin.schema {
                let schema: serde_json::Value =
                    serde_json::from_str(schema_str).expect("schema must be valid JSON");
                assert_ne!(
                    schema.get("type").and_then(|t| t.as_str()),
                    Some("array"),
                    "[{}] Pin \"{}\" has a root-level array schema — use ValueType::Array",
                    node.name,
                    pin.name
                );
            }
        }
    }
}

#[test]
fn lint_every_node_has_description_and_category() {
    for node in all_package_nodes() {
        assert!(
            !node.description.trim().is_empty(),
            "[{}] Missing description",
            node.name
        );
        assert!(
            !node.category.trim().is_empty(),
            "[{}] Missing category",
            node.name
        );
    }
}

#[test]
fn lint_no_generic_pins() {
    for node in all_package_nodes() {
        for pin in &node.pins {
            assert_ne!(
                pin.data_type,
                VariableType::Generic,
                "[{}] Pin \"{}\" uses Generic type — use a specific type",
                node.name,
                pin.name
            );
        }
    }
}

#[test]
fn lint_no_pathbuf_pins() {
    for node in all_package_nodes() {
        for pin in &node.pins {
            assert_ne!(
                pin.data_type,
                VariableType::PathBuf,
                "[{}] Pin \"{}\" uses PathBuf type — use FlowPath (Struct) instead",
                node.name,
                pin.name
            );
        }
    }
}
