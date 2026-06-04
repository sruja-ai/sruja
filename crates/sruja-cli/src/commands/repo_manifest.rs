//! Repo manifest: `.sruja/repos.toml` for declaring inter-repo relationships.
//!
//! This module provides types and functions for loading, saving, and resolving
//! repo manifests that declare cross-repo dependencies and auto-discovery of
//! sibling repositories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::CliError;

/// Repo manifest schema: `.sruja/repos.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoManifest {
    /// Map of repo name to repo entry.
    #[serde(default)]
    pub repos: HashMap<String, RepoEntry>,
    /// Explicit cross-repo edges.
    #[serde(default)]
    pub edges: Vec<CrossRepoEdge>,
}

/// A single repo entry in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    /// Relative or absolute path to the repo.
    pub path: String,
    /// Optional repo_id override (otherwise inferred from directory name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<String>,
}

/// An explicit cross-repo edge declared in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRepoEdge {
    /// Source repo name (key in `[repos]`).
    pub source: String,
    /// Target repo name (key in `[repos]`).
    pub target: String,
    /// Edge kind (e.g., "calls", "depends_on", "publishes_to").
    pub kind: String,
    /// Optional human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Load manifest from `.sruja/repos.toml` relative to `repo_root`.
///
/// Returns a default empty manifest if the file does not exist.
pub fn load_manifest(repo_root: &Path) -> Result<RepoManifest, CliError> {
    let manifest_path = repo_root.join(".sruja/repos.toml");
    if !manifest_path.exists() {
        return Ok(RepoManifest::default());
    }

    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", manifest_path.display(), e),
        ))
    })?;

    let manifest: RepoManifest = toml::from_str(&content).map_err(|e| {
        CliError::validation(format!("Invalid {}: {}", manifest_path.display(), e))
    })?;

    Ok(manifest)
}

/// Save manifest to `.sruja/repos.toml` relative to `repo_root`.
#[allow(dead_code)]
pub fn save_manifest(repo_root: &Path, manifest: &RepoManifest) -> Result<(), CliError> {
    let manifest_path = repo_root.join(".sruja/repos.toml");
    if let Some(parent) = manifest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create dir {}: {}", parent.display(), e),
            ))
        })?;
    }

    let content = toml::to_string_pretty(manifest).map_err(|e| {
        CliError::validation(format!("Failed to serialize manifest: {}", e))
    })?;

    std::fs::write(&manifest_path, content).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write {}: {}", manifest_path.display(), e),
        ))
    })?;

    Ok(())
}

/// Resolve repo entries to absolute paths, filtering out missing repos.
pub fn resolve_repo_paths(
    repo_root: &Path,
    manifest: &RepoManifest,
) -> HashMap<String, PathBuf> {
    let mut resolved = HashMap::new();

    for (name, entry) in &manifest.repos {
        let path = Path::new(&entry.path);
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };

        if resolved_path.exists() {
            resolved.insert(name.clone(), resolved_path);
        }
    }

    resolved
}

/// Auto-discover sibling repos (directories with `.git` in parent of `repo_root`).
pub fn auto_discover_sibling_repos(repo_root: &Path) -> Vec<(String, PathBuf)> {
    let mut discovered = Vec::new();

    let canonical_root = repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf());

    if let Some(parent) = canonical_root.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path == canonical_root {
                    continue;
                }
                if path.is_dir() && path.join(".git").exists() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    discovered.push((name, path));
                }
            }
        }
    }

    discovered
}

/// Get resolved repo paths from manifest, including auto-discovered siblings
/// that are not already in the manifest.
pub fn get_all_repo_paths(
    repo_root: &Path,
    manifest: &RepoManifest,
) -> HashMap<String, PathBuf> {
    let mut all_paths = resolve_repo_paths(repo_root, manifest);

    // Add auto-discovered siblings not already in manifest
    for (name, path) in auto_discover_sibling_repos(repo_root) {
        all_paths.entry(name).or_insert(path);
    }

    all_paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_manifest_returns_default_when_missing() {
        let dir = tempdir().unwrap();
        let manifest = load_manifest(dir.path()).unwrap();
        assert!(manifest.repos.is_empty());
        assert!(manifest.edges.is_empty());
    }

    #[test]
    fn save_and_load_manifest_roundtrip() {
        let dir = tempdir().unwrap();
        let mut repos = HashMap::new();
        repos.insert(
            "api".to_string(),
            RepoEntry {
                path: "../api".to_string(),
                repo_id: Some("api-gateway".to_string()),
            },
        );
        let manifest = RepoManifest {
            repos,
            edges: vec![CrossRepoEdge {
                source: "api".to_string(),
                target: "user".to_string(),
                kind: "calls".to_string(),
                label: Some("REST".to_string()),
            }],
        };

        save_manifest(dir.path(), &manifest).unwrap();
        let loaded = load_manifest(dir.path()).unwrap();

        assert_eq!(loaded.repos.len(), 1);
        assert!(loaded.repos.contains_key("api"));
        assert_eq!(loaded.edges.len(), 1);
        assert_eq!(loaded.edges[0].kind, "calls");
    }

    #[test]
    fn resolve_repo_paths_filters_missing() {
        let dir = tempdir().unwrap();
        let mut repos = HashMap::new();
        repos.insert(
            "exists".to_string(),
            RepoEntry {
                path: ".".to_string(),
                repo_id: None,
            },
        );
        repos.insert(
            "missing".to_string(),
            RepoEntry {
                path: "../nonexistent".to_string(),
                repo_id: None,
            },
        );
        let manifest = RepoManifest {
            repos,
            edges: vec![],
        };

        let resolved = resolve_repo_paths(dir.path(), &manifest);
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("exists"));
    }
}
