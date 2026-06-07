use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// Serialized theme name (e.g. "Mocha", "Latte", "Frappe", "Macchiato").
    #[serde(default)]
    pub theme: Option<String>,
    /// Last wiki URL the user typed into the Crawl tab.
    #[serde(default)]
    pub last_wiki_url: Option<String>,
    /// Last export output path.
    #[serde(default)]
    pub last_export_path: Option<String>,
    /// Last set of enabled seed-category names (in display order).
    #[serde(default)]
    pub enabled_seeds: Vec<String>,
}

impl UserConfig {
    pub fn config_path() -> PathBuf {
        crate::app::config_path().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<UserConfig>(&bytes) {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::warn!(error = %e, path = %path.display(), "failed to parse config.json, using defaults");
                    Self::default()
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "failed to read config.json, using defaults");
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .context("serializing UserConfig")?;
        std::fs::write(&path, bytes)
            .with_context(|| format!("writing config to {}", path.display()))?;
        Ok(())
    }
}
