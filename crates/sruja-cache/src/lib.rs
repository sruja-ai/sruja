//! Disk caching for Sruja scan results and graph computations.
//!
//! Provides cached scan results (keyed by git commit) and centrality scores
//! (keyed by graph hash) to avoid redundant re-computation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during cache operations.
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Scan error: {0}")]
    Scan(#[from] sruja_scan::ScanError),
}

/// Path to the scan cache file relative to repo root.
pub const SCAN_CACHE_PATH: &str = ".sruja/cache/scan.json";

/// Path to the centrality cache file relative to repo root.
const CENTRALITY_CACHE_PATH: &str = ".sruja/cache/centrality.json";

/// Cached scan result with git commit for staleness detection.
#[derive(Serialize, Deserialize)]
pub struct ScanCache {
    #[serde(default)]
    pub git_commit: String,
    pub graph: sruja_scan::Graph,
}

/// Cached centrality scores keyed by graph hash.
#[derive(Serialize, Deserialize)]
struct CentralityCache {
    graph_hash: u64,
    scores: HashMap<String, sruja_scan::graph::ComponentImportance>,
}

/// Read git HEAD commit (short) if repo is a git work tree; otherwise None.
pub fn git_commit_short(repo_path: &Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Load or compute a cached scan graph.
///
/// Uses git commit hash for staleness detection. If the cache is stale or
/// missing, runs a full or incremental scan and writes the result to disk.
pub fn scan_repo_cached(repo_path: &Path) -> Result<sruja_scan::Graph, CacheError> {
    scan_repo_cached_with_opts(repo_path, false)
}

/// Load or compute a cached scan graph with options.
///
/// If `incremental` is true, always runs an incremental scan (but still writes
/// the result to cache for next time).
pub fn scan_repo_cached_with_opts(
    repo_path: &Path,
    incremental: bool,
) -> Result<sruja_scan::Graph, CacheError> {
    let cache_path = repo_path.join(SCAN_CACHE_PATH);
    let current_commit = git_commit_short(repo_path).unwrap_or_default();

    if !incremental && cache_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cache_path) {
            // Try new ScanCache format with git_commit staleness check
            if let Ok(cache) = serde_json::from_str::<ScanCache>(&content) {
                // If no git repo, skip staleness check (treat as fresh)
                if current_commit.is_empty() || cache.git_commit == current_commit {
                    return Ok(cache.graph);
                }
                // Cache is stale — fall through to use incremental rescan
            } else if let Ok(graph) = serde_json::from_str::<sruja_scan::Graph>(&content) {
                // Legacy format (raw Graph, no commit tracking)
                return Ok(graph);
            }
        }
    }

    // Try legacy path for backward compat
    if !incremental {
        let legacy = repo_path.join(".sruja/graph.json");
        if legacy.exists() {
            if let Ok(content) = std::fs::read_to_string(&legacy) {
                if let Ok(graph) = serde_json::from_str::<sruja_scan::Graph>(&content) {
                    return Ok(graph);
                }
            }
        }
    }

    let do_incremental = incremental || cache_path.exists();
    let graph = if do_incremental {
        sruja_scan::scan_repo_incremental(repo_path)?
    } else {
        sruja_scan::scan_repo(repo_path)?
    };

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cache = ScanCache {
        git_commit: current_commit,
        graph: graph.clone(),
    };
    let content = serde_json::to_string(&cache)?;
    let _ = std::fs::write(cache_path, content);

    Ok(graph)
}

/// Compute centrality with disk caching.
///
/// Results are cached keyed by graph hash. If the hash matches, returns cached
/// scores without recomputation.
pub fn compute_all_centrality_cached(
    repo_path: &Path,
    graph: &sruja_scan::Graph,
    quiet: bool,
) -> Result<HashMap<String, sruja_scan::graph::ComponentImportance>, CacheError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Compute a hash of the graph for cache key.
    let graph_json = serde_json::to_string(graph)?;
    let mut hasher = DefaultHasher::new();
    graph_json.hash(&mut hasher);
    let graph_hash = hasher.finish();

    let cache_path = repo_path.join(CENTRALITY_CACHE_PATH);

    // Try to load from cache.
    if cache_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cache_path) {
            if let Ok(cached) = serde_json::from_str::<CentralityCache>(&content) {
                if cached.graph_hash == graph_hash {
                    return Ok(cached.scores);
                }
            }
        }
    }

    // Compute fresh.
    let scores = if quiet {
        sruja_scan::graph::centrality::compute_all_centrality_quiet(graph, true)
    } else {
        sruja_scan::graph::centrality::compute_all_centrality(graph)
    };

    // Write cache.
    let cache = CentralityCache {
        graph_hash,
        scores: scores.clone(),
    };
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string(&cache)?;
    let _ = std::fs::write(cache_path, content);

    Ok(scores)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_commit_short_returns_none_outside_repo() {
        let dir = tempfile::tempdir().expect("temp");
        assert!(git_commit_short(dir.path()).is_none());
    }

    #[test]
    fn scan_cache_path_is_correct() {
        assert_eq!(SCAN_CACHE_PATH, ".sruja/cache/scan.json");
    }
}
