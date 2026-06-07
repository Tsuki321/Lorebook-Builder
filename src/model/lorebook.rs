use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::entry::WorldInfoEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "is_creation")]
    pub is_creation: bool,
    #[serde(rename = "scan_depth")]
    pub scan_depth: u32,
    #[serde(rename = "token_budget")]
    pub token_budget: u32,
    #[serde(rename = "recursive_scanning")]
    pub recursive_scanning: bool,
    pub extensions: serde_json::Value,
    /// Entries are keyed by stringified UID in SillyTavern's format.
    pub entries: BTreeMap<String, WorldInfoEntry>,
}

impl WorldInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            is_creation: false,
            scan_depth: 50,
            token_budget: 2000,
            recursive_scanning: false,
            extensions: serde_json::json!({}),
            entries: BTreeMap::new(),
        }
    }
}
