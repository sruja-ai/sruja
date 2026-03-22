//! Graph persistence and management for CLI
//!
//! Provides knowledge graph storage and retrieval for context engineering.

use sruja_graph::KnowledgeGraph;
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
            e.kind(),
            format!("Failed to read graph file {}: {}", graph_path.display(), e),
        ))
    })?;

    let graph: KnowledgeGraph = serde_json::from_str(&json)?;

    Ok(graph)
}

/// Build knowledge graph from repository scan and save to disk
pub fn build_and_save_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let scan_graph = sruja_scan::scan_repo(repo)
        .map_err(|e| CliError::Validation(format!("Scan failed: {}", e)))?;

    let mut kg = KnowledgeGraph::new();
    sruja_graph::merge_scan_into_graph(&mut kg, &scan_graph, &repo.display().to_string());

    let commit_sha = get_current_commit_sha(repo);
    kg.metadata.commit_sha = commit_sha;

    save_graph(repo, &kg)?;

    Ok(kg)
}

/// Save knowledge graph to disk
pub fn save_graph(repo: &Path, graph: &KnowledgeGraph) -> Result<(), CliError> {
    let sruja_dir = repo.join(".sruja");
    std::fs::create_dir_all(&sruja_dir)?;

    let json = serde_json::to_string_pretty(graph)?;

    std::fs::write(sruja_dir.join("graph.json"), json)?;

    Ok(())
}

/// Check if graph is stale (code changed since last build)
fn is_graph_stale(repo: &Path, graph: &KnowledgeGraph) -> Result<bool, CliError> {
    let graph_path = repo.join(GRAPH_FILE);

    if !graph_path.exists() {
        return Ok(true);
    }

    let current_sha = get_current_commit_sha(repo);

    match (&current_sha, &graph.metadata.commit_sha) {
        (Some(current), Some(stored)) => {
            if current != stored {
                return Ok(true);
            }
            Ok(false)
        }
        (Some(_), None) => Ok(true),
        (None, _) => {
            let graph_metadata = std::fs::metadata(&graph_path)?;
            let graph_modified = graph_metadata.modified()?;

            check_source_files_newer(repo, graph_modified)
        }
    }
}

fn check_source_files_newer(
    repo: &Path,
    graph_modified: std::time::SystemTime,
) -> Result<bool, CliError> {
    let source_dirs = ["src", "lib", "app", "packages", "crates"];

    for dir in source_dirs {
        let dir_path = repo.join(dir);
        if dir_path.exists() && dir_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified > graph_modified {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

/// Get current commit short SHA
fn get_current_commit_sha(repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "--short=7", "HEAD"])
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

        assert!(repo_path.join(".sruja/graph.json").exists());
    }

    #[test]
    fn test_load_graph() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/index.js"), "").unwrap();

        build_and_save_graph(repo_path).unwrap();

        let loaded = load_graph(repo_path);
        assert!(loaded.is_ok());
    }

    #[test]
    fn test_load_graph_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_graph(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_graph_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".sruja")).unwrap();
        std::fs::write(repo_path.join(".sruja/graph.json"), "not valid json").unwrap();

        let result = load_graph(repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_graph_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let graph = KnowledgeGraph::new();

        let result = save_graph(repo_path, &graph);
        assert!(result.is_ok());
        assert!(repo_path.join(".sruja/graph.json").exists());
    }

    #[test]
    fn test_save_graph_pretty_json() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let mut graph = KnowledgeGraph::with_name("TestGraph");

        let node = sruja_graph::ArchitectureNode {
            id: "test_node".to_string(),
            kind: sruja_graph::NodeKind::Service,
            label: "Test".to_string(),
            technology: Some("Rust".to_string()),
            description: Some("Test node".to_string()),
            metadata: std::collections::HashMap::new(),
            source: sruja_graph::SourceReference::manual(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        graph.add_node(node).unwrap();

        save_graph(repo_path, &graph).unwrap();

        let json = std::fs::read_to_string(repo_path.join(".sruja/graph.json")).unwrap();
        assert!(json.contains("\"TestGraph\""));
        assert!(json.contains("test_node"));
    }

    #[test]
    fn test_load_or_build_graph_creates_new_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/app.js"), "console.log('hello')").unwrap();

        let result = load_or_build_graph(repo_path);
        assert!(result.is_ok());
        assert!(repo_path.join(".sruja/graph.json").exists());
    }

    #[test]
    fn test_load_or_build_graph_loads_existing() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/app.js"), "console.log('hello')").unwrap();

        let first = build_and_save_graph(repo_path).unwrap();
        let second = load_or_build_graph(repo_path);

        assert!(second.is_ok());
    }

    #[test]
    fn test_load_or_build_graph_rebuilds_on_corruption() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::fs::create_dir_all(repo_path.join(".sruja")).unwrap();
        std::fs::write(repo_path.join(".sruja/graph.json"), "corrupted").unwrap();

        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/app.js"), "console.log('hello')").unwrap();

        let result = load_or_build_graph(repo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_current_commit_sha_in_git_repo() {
        let sha = get_current_commit_sha(std::path::Path::new("."));
        if std::path::Path::new(".git").exists() {
            assert!(sha.is_some());
            let sha = sha.unwrap();
            assert_eq!(sha.len(), 7);
            assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_get_current_commit_sha_outside_git() {
        let temp_dir = TempDir::new().unwrap();
        let sha = get_current_commit_sha(temp_dir.path());
        assert!(sha.is_none());
    }

    #[test]
    fn test_is_graph_stale_no_graph_file() {
        let temp_dir = TempDir::new().unwrap();
        let graph = KnowledgeGraph::new();
        let result = is_graph_stale(temp_dir.path(), &graph);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_build_and_save_graph_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let result = build_and_save_graph(repo_path);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_save_graph_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let graph1 = KnowledgeGraph::with_name("First");
        save_graph(repo_path, &graph1).unwrap();

        let graph2 = KnowledgeGraph::with_name("Second");
        save_graph(repo_path, &graph2).unwrap();

        let loaded = load_graph(repo_path).unwrap();
        assert_eq!(loaded.metadata.name, "Second");
    }

    #[test]
    fn test_load_graph_roundtrip_preserves_data() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let mut original = KnowledgeGraph::with_name("TestArchitecture");
        let node = sruja_graph::ArchitectureNode {
            id: "svc_api".to_string(),
            kind: sruja_graph::NodeKind::Service,
            label: "API Service".to_string(),
            technology: Some("Node.js".to_string()),
            description: Some("Main API".to_string()),
            metadata: std::collections::HashMap::new(),
            source: sruja_graph::SourceReference::manual(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        original.add_node(node).unwrap();

        save_graph(repo_path, &original).unwrap();
        let loaded = load_graph(repo_path).unwrap();

        assert_eq!(loaded.metadata.name, "TestArchitecture");
        assert!(loaded.get_node("svc_api").is_some());
    }
}
