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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_sitter::{Language, ParsedFile};
    use std::fs;

    #[test]
    fn new_cache_is_empty() {
        let cache = AstCache::new();
        assert!(cache.files.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut cache = AstCache::new();
        cache.files.insert(
            "src/lib.rs".to_string(),
            (
                Language::Rust,
                ParsedFile {
                    name: "lib".to_string(),
                    path: "src/lib.rs".to_string(),
                    imports: vec!["std::io".to_string()],
                    exports: vec!["foo".to_string()],
                    definitions: vec![],
                },
            ),
        );
        cache.save(dir.path()).expect("save");
        assert!(dir.path().join(".sruja/ast_cache.json").exists());
        let loaded = AstCache::load(dir.path()).expect("load");
        assert_eq!(loaded.files.len(), 1);
        let (lang, parsed) = loaded.files.get("src/lib.rs").expect("entry");
        assert_eq!(*lang, Language::Rust);
        assert_eq!(parsed.imports, vec!["std::io"]);
    }

    #[test]
    fn load_returns_none_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(AstCache::load(dir.path()).is_none());
    }

    #[test]
    fn load_returns_none_for_invalid_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache_dir = dir.path().join(".sruja");
        fs::create_dir_all(&cache_dir).expect("mkdir");
        fs::write(cache_dir.join("ast_cache.json"), b"{not json").expect("write");
        assert!(AstCache::load(dir.path()).is_none());
    }
}
