//! Optional `.sruja/extensions.toml` registry (policy enablement metadata).

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExtensionsFile {
    #[serde(default)]
    pub extensions: HashMap<String, ExtensionEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionEntry {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub params: HashMap<String, toml::Value>,
}

pub fn load_extensions(repo: &Path) -> ExtensionsFile {
    let path = repo.join(".sruja").join("extensions.toml");
    if !path.is_file() {
        return ExtensionsFile::default();
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    toml::from_str(&text).unwrap_or_default()
}

pub fn enabled_extension_ids(repo: &Path) -> Vec<String> {
    load_extensions(repo)
        .extensions
        .into_iter()
        .filter(|(_, e)| e.enabled)
        .map(|(k, _)| k)
        .collect()
}
