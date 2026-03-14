//! Resolve default architecture file path when `-a` / `--architecture` is not set.
//! Order: repo.sruja (repo root) → architecture.sruja (repo root) → docs/architecture.sruja.

use std::path::{Path, PathBuf};

const REPO_SURUJA: &str = "repo.sruja";
const ARCHITECTURE_SURUJA: &str = "architecture.sruja";
const DOCS_ARCHITECTURE_SURUJA: &str = "docs/architecture.sruja";

/// Returns the path to the first existing default architecture file under `repo_root`, or None.
pub fn resolve_architecture_path(repo_root: &Path) -> Option<PathBuf> {
    let candidates: [PathBuf; 3] = [
        repo_root.join(REPO_SURUJA),
        repo_root.join(ARCHITECTURE_SURUJA),
        repo_root.join(DOCS_ARCHITECTURE_SURUJA),
    ];
    for p in &candidates {
        if p.exists() {
            return Some(p.clone());
        }
    }
    None
}
