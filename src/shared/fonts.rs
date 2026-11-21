//! Discover fonts alongside the project's images, imports, and other assets.

use super::{normalized_flow_path, Bytes, FlowPath, Font};
use std::collections::HashSet;
use std::path::Path;

#[derive(Default)]
pub(super) struct DiscoveredFonts {
    pub fonts: Vec<Font>,
    pub warnings: Vec<String>,
}

pub(super) fn list(root: &FlowPath) -> Result<Vec<FlowPath>, String> {
    let json = serde_json::to_string(root).map_err(|error| error.to_string())?;
    let entries = flow_like_wasm_sdk::host::storage_list(&json)
        .ok_or_else(|| "storage listing is unavailable".to_string())?;
    entries
        .into_iter()
        .map(|entry| serde_json::from_value(entry).map_err(|error| error.to_string()))
        .collect()
}

pub(super) fn read(file: &FlowPath) -> Option<Vec<u8>> {
    flow_like_wasm_sdk::host::storage_read(&serde_json::to_string(file).ok()?)
}

/// Listing failures and broken incidental font assets should not prevent a
/// document from using its explicit or bundled fonts. Surface them in the same
/// report as Typst's unknown-family warnings instead.
pub(super) fn discover(
    source_folder: Option<&FlowPath>,
    asset_roots: &[FlowPath],
    mut list: impl FnMut(&FlowPath) -> Result<Vec<FlowPath>, String>,
    mut read: impl FnMut(&FlowPath) -> Option<Vec<u8>>,
) -> DiscoveredFonts {
    let mut result = DiscoveredFonts::default();
    let mut seen_roots = HashSet::new();
    let mut seen_files = HashSet::new();
    let mut seen_data = HashSet::new();
    let roots = source_folder
        .map(|root| ("source_folder".to_string(), root))
        .into_iter()
        .chain(
            asset_roots
                .iter()
                .enumerate()
                .map(|(index, root)| (format!("asset_roots[{index}]"), root)),
        );

    for (label, root) in roots {
        let mut root = root.clone();
        match normalized_flow_path(&root.path, &label, true) {
            Ok(path) => root.path = path,
            Err(error) => {
                result.warnings.push(format!(
                    "warning: automatic font discovery skipped {label}: {error}"
                ));
                continue;
            }
        }
        if !seen_roots.insert(key(&root)) {
            continue;
        }

        // The host lists files recursively beneath this object-store prefix.
        // Entries are complete FlowPaths, not paths relative to the root. The
        // host also handles path encoding (e.g. spaces in the supplied prefix).
        let mut entries = match list(&root) {
            Ok(entries) => entries,
            Err(error) => {
                result.warnings.push(format!(
                    "warning: could not discover fonts in {label} at FlowLike path {:?}: {error}; connect font_files explicitly if listing is unavailable",
                    root.path,
                ));
                continue;
            }
        };
        // Object stores need not return a stable order. Preserve root priority
        // while making ties between fonts in the same root deterministic.
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        for mut file in entries {
            if !is_font_file(&file.path) {
                continue;
            }
            let location = format!("{label} font at FlowLike path {:?}", file.path);
            match normalized_flow_path(&file.path, &location, false) {
                Ok(path) => file.path = path,
                Err(error) => {
                    result.warnings.push(format!("warning: {error}"));
                    continue;
                }
            }
            if !seen_files.insert(key(&file)) {
                continue;
            }
            let Some(data) = read(&file) else {
                result
                    .warnings
                    .push(format!("warning: could not read {location}"));
                continue;
            };
            let data = Bytes::new(data);
            if seen_data.contains(&data) {
                continue;
            }
            let fonts: Vec<_> = Font::iter(data.clone()).collect();
            if fonts.is_empty() {
                result.warnings.push(format!(
                    "warning: {location} contains no supported fonts; supply a valid TTF, OTF, or TTC file"
                ));
            } else {
                seen_data.insert(data);
            }
            result.fonts.extend(fonts);
        }
    }
    result
}

fn key(file: &FlowPath) -> (String, Option<String>, String) {
    (
        file.store_ref.clone(),
        file.cache_store_ref.clone(),
        file.path.clone(),
    )
}

fn is_font_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["ttf", "otf", "ttc"]
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}

/// FontBook keeps the first equally matching variant. Exact duplicates are
/// removed by data AND collection index, preserving distinct TTC faces, font
/// variants, and same-family fonts with different glyph coverage.
pub(super) fn combine(explicit: Vec<Font>, discovered: Vec<Font>) -> Vec<Font> {
    let mut seen = HashSet::new();
    explicit
        .into_iter()
        .chain(discovered)
        .chain(typst_assets::fonts().flat_map(|data| Font::iter(Bytes::new(data))))
        .filter(|font| seen.insert(font.clone()))
        .collect()
}
