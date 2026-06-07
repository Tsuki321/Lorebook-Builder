use serde::{Deserialize, Serialize};

/// A single SillyTavern world-info entry.
///
/// Field names are deliberately identical to the on-disk format used by
/// SillyTavern, including the `key` / `keys` / `keysecondary` / `secondary_keys`
/// duplication that ST itself writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfoEntry {
    pub uid: u64,
    pub key: Vec<String>,
    pub keysecondary: Vec<String>,
    pub comment: String,
    pub content: String,
    pub constant: bool,
    pub selective: bool,
    #[serde(rename = "selectiveLogic")]
    pub selective_logic: u8,
    pub order: u32,
    pub position: u8,
    pub disable: bool,
    #[serde(rename = "addMemo")]
    pub add_memo: bool,
    #[serde(rename = "excludeRecursion")]
    pub exclude_recursion: bool,
    pub probability: u8,
    #[serde(rename = "displayIndex")]
    pub display_index: u64,
    #[serde(rename = "useProbability")]
    pub use_probability: bool,
    #[serde(rename = "secondary_keys")]
    pub secondary_keys: Vec<String>,
    pub keys: Vec<String>,
    pub id: u64,
    pub priority: u32,
    #[serde(rename = "insertion_order")]
    pub insertion_order: u32,
    pub enabled: bool,
    pub name: String,
    pub extensions: EntryExtensions,
    pub case_sensitive: bool,
    pub depth: u32,
    #[serde(rename = "characterFilter")]
    pub character_filter: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryExtensions {
    pub depth: u32,
    pub weight: u32,
    #[serde(rename = "addMemo")]
    pub add_memo: bool,
    pub probability: u8,
    #[serde(rename = "displayIndex")]
    pub display_index: u64,
    #[serde(rename = "selectiveLogic")]
    pub selective_logic: u8,
    #[serde(rename = "useProbability")]
    pub use_probability: bool,
    #[serde(rename = "characterFilter")]
    pub character_filter: Option<serde_json::Value>,
    #[serde(rename = "excludeRecursion")]
    pub exclude_recursion: bool,
}

impl WorldInfoEntry {
    /// Build a fresh entry with sensible defaults.
    ///
    /// `uid` and `id` are populated from the supplied counter value. The
    /// caller is responsible for tracking it.
    pub fn new(
        uid: u64,
        name: impl Into<String>,
        keys: Vec<String>,
        content: impl Into<String>,
        comment: impl Into<String>,
        priority: u32,
        position: u8,
    ) -> Self {
        let name = name.into();
        let secondary = Vec::new();
        Self {
            uid,
            key: keys.clone(),
            keysecondary: secondary.clone(),
            comment: comment.into(),
            content: content.into(),
            constant: false,
            selective: true,
            selective_logic: 0,
            order: priority,
            position,
            disable: false,
            add_memo: true,
            exclude_recursion: true,
            probability: 100,
            display_index: uid,
            use_probability: true,
            secondary_keys: secondary,
            keys,
            id: uid,
            priority,
            insertion_order: priority,
            enabled: true,
            name,
            extensions: EntryExtensions {
                depth: 4,
                weight: priority,
                add_memo: true,
                probability: 100,
                display_index: uid,
                selective_logic: 0,
                use_probability: true,
                character_filter: None,
                exclude_recursion: true,
            },
            case_sensitive: false,
            depth: 4,
            character_filter: None,
        }
    }
}
