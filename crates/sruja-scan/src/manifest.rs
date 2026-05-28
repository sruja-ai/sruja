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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn new_manifest_is_empty_version_one() {
        let m = ScanManifest::new();
        assert_eq!(m.version, 1);
        assert!(m.entries.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut manifest = ScanManifest::new();
        manifest.entries.insert(
            "src/lib.rs".to_string(),
            ManifestEntry {
                size_bytes: 12,
                blake3_hash: "abc".to_string(),
            },
        );
        manifest.save(dir.path()).expect("save");
        let loaded = ScanManifest::load(dir.path()).expect("load after save");
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries["src/lib.rs"].size_bytes, 12);
    }

    #[test]
    fn load_returns_none_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(ScanManifest::load(dir.path()).is_none());
    }

    #[test]
    fn hash_file_matches_blake3_of_contents() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sample.txt");
        fs::write(&path, b"hello scan").expect("write");
        let hash = ScanManifest::hash_file(&path).expect("hash");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, blake3::hash(b"hello scan").to_hex().to_string());
    }
}
