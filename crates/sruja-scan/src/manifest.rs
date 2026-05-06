use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanManifest {
    pub version: u32,
    pub entries: BTreeMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub size_bytes: u64,
    pub blake3_hash: String,
}

impl ScanManifest {
    pub fn new() -> Self {
        Self {
            version: 1,
            entries: BTreeMap::new(),
        }
    }
}

impl Default for ScanManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanManifest {
    pub fn load(repo_root: &Path) -> Option<Self> {
        let path = repo_root.join(".sruja").join("scan_manifest.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(manifest) = serde_json::from_str::<Self>(&content) {
                    return Some(manifest);
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
        let path = dir.join("scan_manifest.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn hash_file(path: &Path) -> io::Result<String> {
        let bytes = fs::read(path)?;
        let hash = blake3::hash(&bytes);
        Ok(hash.to_hex().to_string())
    }
}
