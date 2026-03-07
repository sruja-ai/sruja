//! Graph persistence and management for CLI
//!
//! Provides knowledge graph storage and retrieval for architecture intelligence.

use sruja_graph::KnowledgeGraph;
use sruja_scan::ScanGraph;
use std::path::Path;
use std::process::Command;

use crate::commands::CliError;

const GRAPH_FILE: &str = ".sruja/graph.json";

/// Load existing graph or build a new one from repository scan
pub fn load_or_build_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let graph_path = repo.join(GRAPH_FILE);

    if graph_path.exists() {
        match load_graph(repo) {
            Ok(graph) => {
                // Check if stale (code changed since last build)
                if is_graph_stale(repo, &graph)? {
                    eprintln!("💡 Code changed since last analysis, rebuilding graph...");
                    return build_and_save_graph(repo);
                }
                return Ok(graph);
            }
            Err(e) => {
                eprintln!("⚠️  Failed to load existing graph: {}. Rebuilding...", e);
                return build_and_save_graph(repo);
            }
        }
    }

    build_and_save_graph(repo)
}

/// Load graph from disk
pub fn load_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let graph_path = repo.join(GRAPH_FILE);

    let json = std::fs::read_to_string(&graph_path).map_err(|e| {
        CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to read graph file: {}", e),
        ))
    })?;

    let graph: KnowledgeGraph = serde_json::from_str(&json).map_err(|e| CliError::Json(e))?;

    Ok(graph)
}

/// Build knowledge graph from repository scan and save to disk
pub fn build_and_save_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let scan_graph = sruja_scan::scan_repo(repo)
        .map_err(|e| CliError::Validation(format!("Scan failed: {}", e)))?;

    let mut kg = KnowledgeGraph::new();
    sruja_graph::merge_scan_into_graph(&mut kg, &scan_graph, &repo.display().to_string());

    save_graph(repo, &kg)?;

    Ok(kg)
}

/// Save knowledge graph to disk
pub fn save_graph(repo: &Path, graph: &KnowledgeGraph) -> Result<(), CliError> {
    let sruja_dir = repo.join(".sruja");
    std::fs::create_dir_all(&sruja_dir).map_err(|e| CliError::Io(e))?;

    let json = serde_json::to_string_pretty(graph).map_err(|e| CliError::Json(e))?;

    std::fs::write(sruja_dir.join("graph.json"), json).map_err(|e| CliError::Io(e))?;

    Ok(())
}

/// Check if graph is stale (code changed since last build)
fn is_graph_stale(repo: &Path, _graph: &KnowledgeGraph) -> Result<bool, CliError> {
    // Get current commit SHA
    let current_sha = get_current_commit_sha(repo);

    // Get last analyzed commit from graph metadata
    // For now, just check if graph file is older than any source file
    let graph_path = repo.join(GRAPH_FILE);

    if !graph_path.exists() {
        return Ok(true);
    }

    let graph_metadata = std::fs::metadata(&graph_path).map_err(|e| CliError::Io(e))?;
    let graph_modified = graph_metadata.modified().map_err(|e| CliError::Io(e))?;

    // Check if any source files are newer than graph
    // Simple heuristic: check if last commit is newer than graph file
    if let Some(sha) = current_sha {
        // If we have commit info, store it in graph metadata later
        // For now, always rebuild to ensure freshness
        // TODO: Implement proper staleness check with commit SHA tracking
        return Ok(true);
    }

    Ok(false)
}

/// Get current commit short SHA
fn get_current_commit_sha(repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            repo.as_os_str().to_str().unwrap_or("."),
            "rev-parse",
            "--short=7",
            "HEAD",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_and_save_graph() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create a simple JS file
        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(
            repo_path.join("src/index.js"),
            "const helper = require('./helper');",
        )
        .unwrap();
        std::fs::write(repo_path.join("src/helper.js"), "module.exports = {};").unwrap();

        let result = build_and_save_graph(repo_path);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert!(!graph.nodes.is_empty());

        // Check file was created
        assert!(repo_path.join(".sruja/graph.json").exists());
    }

    #[test]
    fn test_load_graph() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Build and save first
        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/index.js"), "").unwrap();

        build_and_save_graph(repo_path).unwrap();

        // Then load
        let loaded = load_graph(repo_path);
        assert!(loaded.is_ok());
    }
}
