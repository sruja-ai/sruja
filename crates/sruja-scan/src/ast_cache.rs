use crate::tree_sitter::{Language, ParsedFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AstCache {
    pub files: HashMap<String, (Language, ParsedFile)>,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn load(repo_root: &Path) -> Option<Self> {
        let path = repo_root.join(".sruja").join("ast_cache.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cache) = serde_json::from_str::<Self>(&content) {
                    return Some(cache);
                }
            }
        }
        None
    }

    pub fn save(&self, repo_root: &Path) -> io::Result<()> {
        let dir = repo_root.join(".sruja");
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let path = dir.join("ast_cache.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
