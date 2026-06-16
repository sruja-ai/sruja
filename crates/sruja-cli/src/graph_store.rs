//! Graph persistence and management for CLI
//!
//! Provides knowledge graph storage and retrieval for context engineering.

use sruja_graph::{ContextEventSummary, GraphSnapshot, KnowledgeGraph, LearningEntry};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use crate::commands::CliError;

const GRAPH_FILE: &str = ".sruja/cache/kg.json";
const LEGACY_GRAPH_FILE: &str = ".sruja/graph.json";
pub(crate) const SNAPSHOTS_FILE: &str = ".sruja/graph_snapshots.jsonl";
const MAX_SNAPSHOTS: usize = 100;

fn atomic_write_file(path: &std::path::Path, contents: &[u8]) -> Result<(), CliError> {
    use std::io::Write;

    let parent = path.parent().ok_or_else(|| {
        CliError::validation(format!(
            "Invalid path (no parent directory): {}",
            path.display()
        ))
    })?;
    std::fs::create_dir_all(parent)?;

    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_path = parent.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        unique
    ));

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)?;
    f.write_all(contents)?;
    f.sync_all()?;

    #[cfg(windows)]
    {
        let _ = std::fs::remove_file(path);
    }

    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Load existing graph or build a new one from repository scan
pub fn load_or_build_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let graph_path = repo.join(GRAPH_FILE);

    // Try new cache path first, then legacy for backward compat
    let effective_path = if graph_path.exists() {
        graph_path
    } else {
        let legacy = repo.join(LEGACY_GRAPH_FILE);
        if legacy.exists() {
            legacy
        } else {
            graph_path
        }
    };

    if effective_path.exists() {
        match load_graph_from(&effective_path) {
            Ok(graph) => {
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

/// Load graph from disk (used by tests and as public API for callers that want read-only access)
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let graph_path = repo.join(GRAPH_FILE);
    if graph_path.exists() {
        return load_graph_from(&graph_path);
    }
    let legacy = repo.join(LEGACY_GRAPH_FILE);
    load_graph_from(&legacy)
}

fn load_graph_from(path: &Path) -> Result<KnowledgeGraph, CliError> {
    let json = std::fs::read_to_string(path).map_err(|_| CliError::ConfigCorrupted {
        message: format!(
            "Cannot read {}. Run `sruja sync` to rebuild.",
            path.display()
        ),
    })?;

    serde_json::from_str(&json).map_err(|_| CliError::ConfigCorrupted {
        message: format!(
            "{} is not valid JSON. Run `sruja sync` to rebuild.",
            path.display()
        ),
    })
}

/// Build knowledge graph from repository scan and save to disk
pub fn build_and_save_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let scan_graph = sruja_scan::scan_repo(repo)
        .map_err(|e| CliError::validation(format!("Scan failed: {}", e)))?;

    let mut kg = KnowledgeGraph::new();
    sruja_graph::merge_scan_into_graph(&mut kg, &scan_graph, &repo.display().to_string());
    merge_decision_records_from_repo(repo, &mut kg)?;
    merge_learnings_from_memory(repo, &mut kg);
    merge_recent_events(repo, &mut kg);

    let commit_sha = get_current_commit_sha(repo);
    kg.metadata.commit_sha = commit_sha.clone();

    // Compute deltas and append snapshot before saving
    let graph_path = repo.join(GRAPH_FILE);
    if graph_path.exists() {
        if let Ok(prev_kg) = load_graph_from(&graph_path) {
            let deltas = sruja_graph::snapshot::compute_deltas(&prev_kg, &kg);
            if !deltas.is_empty() {
                let snapshot = GraphSnapshot {
                    timestamp: chrono::Utc::now(),
                    commit_sha: commit_sha.clone().unwrap_or_default(),
                    deltas,
                };
                if let Err(e) = append_snapshot(repo, &snapshot) {
                    eprintln!("Warning: Failed to save graph snapshot: {}", e);
                }
            }
        }
    }

    save_graph(repo, &kg)?;

    Ok(kg)
}

/// Merge agent learnings from `.sruja/agent_memory.json` into the knowledge graph.
/// This makes learnings traversable alongside architecture elements.
fn merge_learnings_from_memory(repo: &Path, graph: &mut KnowledgeGraph) {
    let memory_path = repo.join(".sruja/agent_memory.json");
    if !memory_path.exists() {
        return;
    }

    let Ok(json) = std::fs::read_to_string(&memory_path) else {
        return;
    };

    let Ok(memory) = serde_json::from_str::<serde_json::Value>(&json) else {
        return;
    };

    let Some(entries) = memory.get("learnings").and_then(|e| e.as_array()) else {
        return;
    };

    for entry_value in entries {
        let Ok(learning) = serde_json::from_value::<LearningEntry>(entry_value.clone()) else {
            eprintln!("Warning: skipping learning entry that failed to deserialize");
            continue;
        };
        graph.learnings.insert(learning.id.clone(), learning);
    }

    if !graph.learnings.is_empty() {
        graph.touch();
    }
}

/// Merge recent context events from `.sruja/context_events.jsonl` into the graph.
/// Keeps only the last 50 events, compacted into summaries.
fn merge_recent_events(repo: &Path, graph: &mut KnowledgeGraph) {
    let events_path = repo.join(".sruja/context_events.jsonl");
    if !events_path.exists() {
        return;
    }

    let Ok(content) = std::fs::read_to_string(&events_path) else {
        return;
    };

    let max_events = 50;
    let cutoff_days = 30;
    let cutoff = chrono::Utc::now() - chrono::Duration::days(cutoff_days);

    let mut summaries: Vec<ContextEventSummary> = Vec::new();

    for line in content.lines().rev() {
        if summaries.len() >= max_events {
            break;
        }

        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        let timestamp = event
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        // Skip events older than cutoff
        if timestamp < cutoff {
            continue;
        }

        let kind = event
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let outcome = event
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let elements = event
            .get("elements")
            .or_else(|| event.get("details").and_then(|d| d.get("elements")))
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let summary = event
            .get("summary")
            .and_then(|v| v.as_str())
            .map(String::from);

        summaries.push(ContextEventSummary {
            timestamp,
            kind,
            elements,
            outcome,
            summary,
        });
    }

    // Reverse so oldest is first
    summaries.reverse();

    if !summaries.is_empty() {
        graph.set_recent_events(summaries);
    }
}

/// Merge Decision Records from `.sruja/decisions/*.md` into `graph` so `graph.json` stays aligned
/// with the markdown source of truth after `sruja sync` / rebuild.
pub fn merge_decision_records_from_repo(
    repo: &Path,
    graph: &mut KnowledgeGraph,
) -> Result<(), CliError> {
    let items = crate::commands::decision::list_decisions(repo)?;
    let mut merged = 0usize;
    for it in items {
        let md = repo.join(&it.path);
        if !md.is_file() {
            continue;
        }
        let (fm, body) = match crate::commands::decision::parse_decision_file(&md) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(target: "sruja", "skip decision file {:?}: {}", md, e);
                continue;
            }
        };
        let prev = graph.get_decision(&fm.id);
        match crate::commands::decision::build_graph_decision(repo, &md, &fm, &body, prev) {
            Ok(d) => {
                graph.decisions.insert(d.id.clone(), d);
                merged += 1;
            }
            Err(e) => {
                tracing::warn!(target: "sruja", "skip decision {}: {}", fm.id, e);
            }
        }
    }
    if merged > 0 {
        graph.touch();
    }
    Ok(())
}

/// Upsert one Decision Record into the KG cache after a file change (`accept`, `link`, …).
pub fn upsert_decision_record_in_stored_graph(repo: &Path, md_path: &Path) -> Result<(), CliError> {
    let mut kg = load_or_build_graph(repo)?;
    let (fm, body) = crate::commands::decision::parse_decision_file(md_path)?;
    let prev = kg.get_decision(&fm.id);
    let d = crate::commands::decision::build_graph_decision(repo, md_path, &fm, &body, prev)?;
    kg.decisions.insert(d.id.clone(), d);
    kg.touch();
    save_graph(repo, &kg)
}

/// Save knowledge graph to disk
pub fn save_graph(repo: &Path, graph: &KnowledgeGraph) -> Result<(), CliError> {
    let cache_dir = repo.join(".sruja/cache");
    std::fs::create_dir_all(&cache_dir)?;

    let json = serde_json::to_string(graph)?;
    atomic_write_file(&cache_dir.join("kg.json"), json.as_bytes())?;

    Ok(())
}

/// Append a snapshot to the JSONL file and trim to MAX_SNAPSHOTS
fn append_snapshot(repo: &Path, snapshot: &GraphSnapshot) -> Result<(), CliError> {
    let path = repo.join(SNAPSHOTS_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{}", serde_json::to_string(snapshot)?)?;

    // Trim to last MAX_SNAPSHOTS
    trim_snapshots(repo, MAX_SNAPSHOTS)?;
    Ok(())
}

/// Keep only the last N snapshots in the JSONL file
fn trim_snapshots(repo: &Path, max_count: usize) -> Result<(), CliError> {
    let path = repo.join(SNAPSHOTS_FILE);
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() <= max_count {
        return Ok(());
    }

    let keep = &lines[lines.len() - max_count..];
    std::fs::write(&path, keep.join("\n") + "\n")?;

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

    if let Ok(entries) = std::fs::read_dir(repo) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "sruja" {
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

    for dir in source_dirs {
        let dir_path = repo.join(dir);
        if dir_path.exists()
            && dir_path.is_dir()
            && any_path_modified_after(&dir_path, graph_modified)
        {
            return Ok(true);
        }
    }

    let decisions_dir = repo.join(".sruja/decisions");
    if decisions_dir.is_dir() && any_path_modified_after(&decisions_dir, graph_modified) {
        return Ok(true);
    }

    Ok(false)
}

fn any_path_modified_after(root: &Path, cutoff: std::time::SystemTime) -> bool {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_file() {
            if let Ok(modified) = metadata.modified() {
                if modified > cutoff {
                    return true;
                }
            }
            continue;
        }

        if !metadata.is_dir() {
            continue;
        }

        if should_skip_dir(&path) {
            continue;
        }

        let entries = match std::fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            stack.push(entry.path());
        }
    }

    false
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(name, ".git" | "target" | "node_modules")
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

        assert!(repo_path.join(GRAPH_FILE).exists());
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
        let cache_dir = repo_path.join(".sruja/cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("kg.json"), "not valid json").unwrap();

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
        assert!(repo_path.join(GRAPH_FILE).exists());
    }

    #[test]
    fn test_save_graph_pretty_json() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let mut graph = KnowledgeGraph::with_name("TestGraph");

        let mut node = sruja_graph::ArchitectureNode {
            id: "test_node".to_string(),
            kind: sruja_graph::NodeKind::new(sruja_graph::NodeKind::SERVICE),
            label: "Test".to_string(),
            description: Some("Test node".to_string()),
            ..sruja_graph::ArchitectureNode::default()
        };
        node.set_technology(Some("Rust".to_string()));
        graph.add_node(node).unwrap();

        save_graph(repo_path, &graph).unwrap();

        let json = std::fs::read_to_string(repo_path.join(GRAPH_FILE)).unwrap();
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
        assert!(repo_path.join(GRAPH_FILE).exists());
    }

    #[test]
    fn test_load_or_build_graph_loads_existing() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/app.js"), "console.log('hello')").unwrap();

        let _first = build_and_save_graph(repo_path).unwrap();
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
        let mut node = sruja_graph::ArchitectureNode {
            id: "svc_api".to_string(),
            kind: sruja_graph::NodeKind::new(sruja_graph::NodeKind::SERVICE),
            label: "API Service".to_string(),
            description: Some("Main API".to_string()),
            ..sruja_graph::ArchitectureNode::default()
        };
        node.set_technology(Some("Node.js".to_string()));
        original.add_node(node).unwrap();

        save_graph(repo_path, &original).unwrap();
        let loaded = load_graph(repo_path).unwrap();

        assert_eq!(loaded.metadata.name, "TestArchitecture");
        assert!(loaded.get_node("svc_api").is_some());
    }

    #[test]
    fn decision_record_merged_into_saved_graph() {
        use crate::commands::decision::create_decision_record;
        use sruja_graph::DecisionStatus;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join("src")).unwrap();
        std::fs::write(repo_path.join("src/app.js"), "console.log(1)").unwrap();

        build_and_save_graph(repo_path).unwrap();
        let id = create_decision_record(
            repo_path,
            "Use event sourcing",
            "adr",
            None,
            "test_tool",
            "human",
            "test",
        )
        .unwrap();

        let loaded = load_graph(repo_path).unwrap();
        let d = loaded
            .get_decision(&id)
            .expect("decision merged into graph");
        assert_eq!(d.title, "Use event sourcing");
        assert!(matches!(d.status, DecisionStatus::Proposed));
    }
}
