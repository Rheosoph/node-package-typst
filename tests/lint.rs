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
fn lint_compiler_nodes_accept_font_files() {
    let flow_path_schema = serde_json::to_value(schemars::schema_for!(FlowPath))
        .expect("FlowPath schema must serialize");

    for node in all_package_nodes() {
        let font_pins: Vec<_> = node
            .pins
            .iter()
            .filter(|pin| pin.name == "font_files")
            .collect();

        if node.name == "typst_markdown_to_typst" {
            assert!(
                font_pins.is_empty(),
                "Markdown conversion does not load fonts"
            );
            continue;
        }

        assert_eq!(
            font_pins.len(),
            1,
            "[{}] must expose one font_files pin",
            node.name
        );
        let pin = font_pins[0];
        assert_eq!(pin.pin_type, PinType::Input, "[{}]", node.name);
        assert_eq!(pin.data_type, VariableType::Struct, "[{}]", node.name);
        assert_eq!(pin.value_type, Some(ValueType::Array), "[{}]", node.name);
        assert_eq!(
            pin.default_value,
            Some(json!([])),
            "[{}] font_files must be optional",
            node.name
        );
        let schema: serde_json::Value = serde_json::from_str(
            pin.schema
                .as_deref()
                .expect("font_files must have a FlowPath schema"),
        )
        .expect("font_files schema must be valid JSON");
        assert_eq!(
            schema, flow_path_schema,
            "[{}] font_files schema must describe one FlowPath",
            node.name
        );
        assert!(
            node.permissions.contains(&NodePermission::StorageRead),
            "[{}] must be allowed to read font files",
            node.name
        );
    }
}

#[test]
fn lint_compiler_nodes_accept_asset_roots() {
    let flow_path_schema = serde_json::to_value(schemars::schema_for!(FlowPath))
        .expect("FlowPath schema must serialize");

    for node in all_package_nodes() {
        let asset_pins: Vec<_> = node
            .pins
            .iter()
            .filter(|pin| pin.name == "asset_roots")
            .collect();

        if node.name == "typst_markdown_to_typst" {
            assert!(
                asset_pins.is_empty(),
                "Markdown conversion does not resolve assets"
            );
            continue;
        }

        assert_eq!(
            asset_pins.len(),
            1,
            "[{}] must expose one asset_roots pin",
            node.name
        );
        let pin = asset_pins[0];
        assert_eq!(pin.pin_type, PinType::Input, "[{}]", node.name);
        assert_eq!(pin.data_type, VariableType::Struct, "[{}]", node.name);
        assert_eq!(pin.value_type, Some(ValueType::Array), "[{}]", node.name);
        assert_eq!(
            pin.default_value,
            Some(json!([])),
            "[{}] asset_roots must be optional",
            node.name
        );
        let schema: serde_json::Value = serde_json::from_str(
            pin.schema
                .as_deref()
                .expect("asset_roots must have a FlowPath schema"),
        )
        .expect("asset_roots schema must be valid JSON");
        assert_eq!(
            schema, flow_path_schema,
            "[{}] asset_roots schema must describe one FlowPath",
            node.name
        );
        assert!(
            node.permissions.contains(&NodePermission::StorageRead),
            "[{}] must be allowed to read assets",
            node.name
        );
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

/// Whether the manifest declares a node permission. TEMPORARY, together with
/// the capability flags in flow-like.toml: FlowLike Studio 0.1.12 strips node
/// permissions the manifest does not repeat, and older registries list
/// capabilities from the manifest. Newer hosts derive both from the nodes.
fn manifest_grants(permissions: &toml::Value, permission: &NodePermission) -> bool {
    let flag = |section: &str, key: &str| {
        permissions
            .get(section)
            .and_then(|section| section.get(key))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
    };

    match permission {
        NodePermission::NetworkHttp => flag("network", "http_enabled"),
        NodePermission::StorageRead | NodePermission::StorageWrite => {
            flag("filesystem", "node_storage") || flag("filesystem", "user_storage")
        }
        other => panic!("add the flow-like.toml permission that grants {other:?}"),
    }
}

#[test]
fn lint_manifest_declares_every_node_permission() {
    let manifest: toml::Table = FLOW_LIKE_TOML
        .parse()
        .expect("flow-like.toml must be valid TOML");
    let permissions = manifest
        .get("permissions")
        .expect("flow-like.toml needs a [permissions] table");

    for node in all_package_nodes() {
        for permission in &node.permissions {
            assert!(
                manifest_grants(permissions, permission),
                "[{}] declares {permission:?}, but flow-like.toml does not declare it, so the store listing would hide it",
                node.name
            );
        }
    }
}
