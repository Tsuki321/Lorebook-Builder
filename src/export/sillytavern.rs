use std::collections::BTreeMap;

use crate::model::{WorldInfo, WorldInfoEntry};

/// Serialize a `WorldInfo` to the exact SillyTavern v3 lorebook JSON format
/// matching `Example World Info.json` (uid as string keys, all ST quirks).
pub fn write(wiki: &WorldInfo) -> serde_json::Value {
    let entries: BTreeMap<String, &WorldInfoEntry> =
        wiki.entries.iter().map(|(k, v)| (k.clone(), v)).collect();

    serde_json::json!({
        "name": wiki.name,
        "description": wiki.description,
        "is_creation": wiki.is_creation,
        "scan_depth": wiki.scan_depth,
        "token_budget": wiki.token_budget,
        "recursive_scanning": wiki.recursive_scanning,
        "extensions": wiki.extensions,
        "entries": entries,
    })
}

/// Convenience: write straight to a string.
pub fn to_string_pretty(wiki: &WorldInfo) -> String {
    let v = write(wiki);
    serde_json::to_string_pretty(&v).expect("serialization of plain data never fails")
}
