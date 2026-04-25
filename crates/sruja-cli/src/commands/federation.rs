//! Federation: publish repo.bundle.json and compose system.index.json.
//!
//! Phase 3 artifacts:
//! - repo.bundle.json: repo metadata, local DSL snapshot, latest context, truth state.
//! - system.index.json: composed graph across repos with canonical IDs and lineage.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::discover::discover_context_json_from_graph;
use super::CliError;
use crate::utils::architecture_path;
use sruja_scan::scan_repo;

const REPO_BUNDLE_SCHEMA_VERSION: u32 = 1;
const SYSTEM_INDEX_SCHEMA_VERSION: u32 = 1;

/// Repo bundle schema: published repo truth + evidence artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBundle {
    pub schema_version: u32,
    /// Repo identifier (dir name or git remote; may be empty).
    pub repo_id: String,
    /// Repository path as provided (e.g. "." or absolute).
    pub repo_path: String,
    /// Git HEAD short commit if available.
    pub git_commit: Option<String>,
    /// Path to baseline DSL file within repo (e.g. repo.sruja).
    pub baseline_path: Option<String>,
    /// Content of baseline DSL file, if present.
    pub baseline_dsl: Option<String>,
    /// Latest context (from .sruja/context.json): components, edges, truth_status, updated_at, etc.
    pub context: serde_json::Value,
    /// Truth state: "reviewed" | "drifted" | "unknown".
    pub truth_status: String,
    /// Optional intent refs (ADR paths, .sruja intent files).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intent_refs: Vec<String>,
    /// Optional contracts (exposed interfaces); reserved for future use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracts: Option<serde_json::Value>,
    /// Optional owners; reserved for future use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owners: Option<serde_json::Value>,
}

fn git_commit_short(repo_path: &Path) -> Option<String> {
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

/// Infer repo_id from directory name or git remote.
pub fn infer_repo_id(repo_path: &Path) -> String {
    if let Ok(canonical) = repo_path.canonicalize() {
        if let Some(name) = canonical.file_name().and_then(|n| n.to_str()) {
            if !name.is_empty() && name != "." {
                return name.to_string();
            }
        }
    }
    std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .and_then(|url| {
            url.trim()
                .rsplit('/')
                .next()
                .map(|s| s.trim_end_matches(".git").to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn publish(
    repo_root: &str,
    repo_id_override: Option<&str>,
    output_path: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path).map_err(|e| CliError::scan(e.to_string()))?;
    let ctx = discover_context_json_from_graph(repo_root, repo_path, &graph)?;
    let context_value =
        serde_json::to_value(&ctx).map_err(|e| CliError::validation(e.to_string()))?;

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let (baseline_path_str, baseline_dsl, truth_status) = if let Some(ref base) = baseline_path {
        let path_str = base.to_string_lossy().to_string();
        let content = fs::read_to_string(base).ok();
        let content_ref = content.as_deref().unwrap_or("");
        let parser = sruja_language::Parser::new(base.to_string_lossy().as_ref());
        let program = parser
            .parse(content_ref)
            .map_err(|diags| CliError::parse_with_diagnostics(path_str.clone(), diags))?;
        let proposed = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed);
        let truth = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };
        (Some(path_str), content, truth.to_string())
    } else {
        let _drift = sruja_diff::detect_architectural_drift(&graph);
        (None, None, "unknown".to_string())
    };

    let mut context = context_value;
    context["truth_status"] = serde_json::Value::String(truth_status.clone());
    context["git_commit"] = git_commit_short(repo_path)
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    context["baseline_path"] = baseline_path_str
        .as_ref()
        .map(|s| serde_json::Value::String(s.clone()))
        .unwrap_or(serde_json::Value::Null);
    context["graph"] =
        serde_json::to_value(&graph).map_err(|e| CliError::validation(e.to_string()))?;

    let repo_id = repo_id_override
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| infer_repo_id(repo_path));
    let bundle = RepoBundle {
        schema_version: REPO_BUNDLE_SCHEMA_VERSION,
        repo_id: repo_id.clone(),
        repo_path: repo_root.to_string(),
        git_commit: git_commit_short(repo_path),
        baseline_path: baseline_path_str,
        baseline_dsl: baseline_dsl.or_else(|| {
            baseline_path
                .as_ref()
                .and_then(|p| fs::read_to_string(p).ok())
        }),
        context,
        truth_status,
        intent_refs: vec![],
        contracts: None,
        owners: None,
    };

    let out_path = Path::new(output_path);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create output dir: {}", parent.display()),
            ))
        })?;
    }
    fs::write(
        out_path,
        serde_json::to_string_pretty(&bundle).map_err(|e| CliError::validation(e.to_string()))?,
    )
    .map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write {}: {}", output_path, e),
        ))
    })?;

    eprintln!(
        "Wrote {} (repo_id={}, truth_status={})",
        output_path, repo_id, bundle.truth_status
    );
    Ok(())
}

/// Node in system index with lineage (which repo owns it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIndexNode {
    /// Canonical ID: repo_id::local_id to avoid cross-repo collisions.
    pub canonical_id: String,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    /// Repo that owns this element.
    pub repo_id: String,
    /// Local element ID within that repo.
    pub local_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criticality: Option<sruja_language::ast::Criticality>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<sruja_language::ast::SourceBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

/// Edge in system index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIndexEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Repo that contributed this edge (source repo if edge is between repos).
    pub repo_id: String,
}

/// Conflict: same logical element or key from multiple repos with different definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIndexConflict {
    pub key: String,
    pub repos: Vec<String>,
    pub message: String,
}

/// Composed multi-repo system index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIndex {
    pub schema_version: u32,
    /// Repos that were composed.
    pub repos: Vec<RepoEntry>,
    pub nodes: Vec<SystemIndexNode>,
    pub edges: Vec<SystemIndexEdge>,
    /// Conflicts (duplicate keys or incompatible definitions).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<SystemIndexConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub repo_id: String,
    pub repo_path: String,
    pub truth_status: String,
    pub git_commit: Option<String>,
}

fn is_bundle_filename(name: &str) -> bool {
    name == "repo.bundle.json" || name.ends_with(".repo.bundle.json")
}

fn collect_bundle_paths_in_dir(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>, CliError> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read dir {}: {}", dir.display(), e),
        ))
    })? {
        let e = e.map_err(CliError::Io)?;
        let path = e.path();
        if path.is_dir() && recursive {
            out.extend(collect_bundle_paths_in_dir(&path, recursive)?);
            continue;
        }
        if path.is_file()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_bundle_filename)
        {
            out.push(path);
        }
    }
    Ok(out)
}

fn collect_bundle_paths(inputs: &[String], recursive: bool) -> Result<Vec<PathBuf>, CliError> {
    if inputs.is_empty() {
        return Err(CliError::validation(
            "No inputs provided. Pass at least one -i <bundle-or-dir>.".to_string(),
        ));
    }

    let mut out = Vec::new();
    for input in inputs {
        let p = Path::new(input);
        if !p.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Input not found: {}", input),
            )));
        }
        if p.is_file() {
            out.push(p.to_path_buf());
            continue;
        }
        out.extend(collect_bundle_paths_in_dir(p, recursive)?);
    }
    out.sort(); // stable order for compose
    out.dedup();
    Ok(out)
}

pub async fn compose(
    inputs: &[String],
    recursive: bool,
    output_path: &str,
) -> Result<(), CliError> {
    let paths = collect_bundle_paths(inputs, recursive)?;
    if paths.is_empty() {
        return Err(CliError::validation(
            "No bundle files found. Provide a path to a repo.bundle.json (or *.repo.bundle.json) file, or a directory containing such files. Tip: use --recursive when bundles are nested in subdirectories.".to_string(),
        ));
    }

    let mut repos = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut conflicts = Vec::new();

    for path in &paths {
        let content = fs::read_to_string(path).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read {}: {}", path.display(), e),
            ))
        })?;
        let bundle: RepoBundle = serde_json::from_str(&content).map_err(|e| {
            CliError::validation(format!("Invalid bundle {}: {}", path.display(), e))
        })?;

        let repo_id = bundle.repo_id.as_str();
        repos.push(RepoEntry {
            repo_id: bundle.repo_id.clone(),
            repo_path: bundle.repo_path.clone(),
            truth_status: bundle.truth_status.clone(),
            git_commit: bundle.git_commit.clone(),
        });

        // Load and merge nodes/edges: DSL wins, Scan fills the gaps.
        let mut bundle_nodes = BTreeMap::new();
        let mut bundle_edges = Vec::new();

        // 1. Load from baseline DSL (if present)
        if let Some(ref dsl) = bundle.baseline_dsl {
            let parser = sruja_language::Parser::new("repo.sruja");
            if let Ok(program) = parser.parse(dsl) {
                let g = sruja_diff::program_to_graph(&program);
                for n in g.nodes {
                    bundle_nodes.insert(
                        n.id.clone(),
                        SystemIndexNode {
                            canonical_id: format!("{}::{}", repo_id, n.id),
                            kind: n.kind.as_str().to_string(),
                            label: n.label.clone(),
                            technology: n.technology.clone(),
                            repo_id: repo_id.to_string(),
                            local_id: n.id.clone(),
                            owner: n.owner.clone(),
                            domain: n.domain.clone(),
                            criticality: n.criticality,
                            sources: n.sources.clone(),
                            logical_id: n.canonical_id.clone(),
                            aliases: n.aliases.clone(),
                        },
                    );
                }
                for e in g.edges {
                    bundle_edges.push(SystemIndexEdge {
                        source: format!("{}::{}", repo_id, e.source),
                        target: format!("{}::{}", repo_id, e.target),
                        kind: e.kind.as_str().to_string(),
                        label: e.evidence.first().and_then(|ev| ev.detail.clone()),
                        repo_id: repo_id.to_string(),
                    });
                }
            }
        }

        // 2. Load from scanned context (graph) to fill gaps
        if let Some(g_val) = bundle.context.get("graph") {
            if let Ok(g) = serde_json::from_value::<sruja_scan::Graph>(g_val.clone()) {
                // Precompute centrality to find important modules
                let centrality = sruja_scan::graph::centrality::compute_all_centrality(&g);
                let mut high_centrality_nodes = std::collections::HashSet::new();

                // Keep top 15% of modules by pagerank or degree
                let mut module_scores: Vec<(&String, f64)> = centrality
                    .iter()
                    .filter(|(id, _)| {
                        g.nodes
                            .iter()
                            .any(|n| &n.id == *id && n.kind == sruja_scan::NodeKind::Module)
                    })
                    .map(|(id, c)| (id, c.pagerank + c.degree_centrality))
                    .collect();

                module_scores
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let top_n = std::cmp::max(1, module_scores.len() * 15 / 100);
                for (id, _) in module_scores.into_iter().take(top_n) {
                    high_centrality_nodes.insert(id.clone());
                }

                for n in g.nodes {
                    // Only add if not already in DSL (DSL is reviewed truth)
                    if !bundle_nodes.contains_key(&n.id) {
                        // Filter: include anything high-level OR module if it has specific metadata
                        let is_high_level = matches!(
                            n.kind,
                            sruja_scan::NodeKind::System
                                | sruja_scan::NodeKind::Container
                                | sruja_scan::NodeKind::Service
                                | sruja_scan::NodeKind::Database
                                | sruja_scan::NodeKind::ExternalApi
                                | sruja_scan::NodeKind::Queue
                                | sruja_scan::NodeKind::Frontend
                        );

                        let is_important_module = n.kind == sruja_scan::NodeKind::Module
                            && (high_centrality_nodes.contains(&n.id)
                                || sruja_diff::drift::is_likely_entry_point(
                                    n.path.as_deref().unwrap_or(""),
                                    &n.id,
                                ));

                        if is_high_level || is_important_module {
                            bundle_nodes.insert(
                                n.id.clone(),
                                SystemIndexNode {
                                    canonical_id: format!("{}::{}", repo_id, n.id),
                                    kind: n.kind.as_str().to_string(),
                                    label: n.label.clone(),
                                    technology: n.technology.clone(),
                                    repo_id: repo_id.to_string(),
                                    local_id: n.id.clone(),
                                    owner: n.owner.clone(),
                                    domain: n.domain.clone(),
                                    criticality: n.criticality,
                                    sources: n.sources.clone(),
                                    logical_id: n.canonical_id.clone(), // This comes from ElementDefBody.canonical_id
                                    aliases: n.aliases.clone(),
                                },
                            );
                        }
                    }
                }
                for e in g.edges {
                    // Only include edges where both ends are in our filtered nodes
                    let src_exists = bundle_nodes.contains_key(&e.source);
                    let dst_exists = bundle_nodes.contains_key(&e.target);
                    if src_exists && dst_exists {
                        bundle_edges.push(SystemIndexEdge {
                            source: format!("{}::{}", repo_id, e.source),
                            target: format!("{}::{}", repo_id, e.target),
                            kind: e.kind.as_str().to_string(),
                            label: e.evidence.first().and_then(|ev| ev.detail.clone()),
                            repo_id: repo_id.to_string(),
                        });
                    }
                }
            }
        }

        nodes.extend(bundle_nodes.into_values());
        edges.extend(bundle_edges);
    }

    // Detect conflicts: same local_id in multiple repos (possible shared element)
    let mut local_by_key: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for n in &nodes {
        local_by_key
            .entry(format!("{}::{}", n.kind, n.label.to_lowercase()))
            .or_default()
            .push(n.repo_id.clone());
    }
    for (key, repo_list) in local_by_key {
        let unique: Vec<_> = repo_list
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        if unique.len() > 1 {
            conflicts.push(SystemIndexConflict {
                key: key.clone(),
                repos: unique,
                message:
                    "Same kind+label in multiple repos; resolve canonical ownership or rename."
                        .to_string(),
            });
        }
    }

    let index = SystemIndex {
        schema_version: SYSTEM_INDEX_SCHEMA_VERSION,
        repos,
        nodes,
        edges,
        conflicts,
    };

    let out_path = Path::new(output_path);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create output dir: {}", parent.display()),
            ))
        })?;
    }
    fs::write(
        out_path,
        serde_json::to_string_pretty(&index).map_err(|e| CliError::validation(e.to_string()))?,
    )
    .map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write {}: {}", output_path, e),
        ))
    })?;

    eprintln!(
        "Wrote {} ({} repos, {} nodes, {} edges, {} conflict(s))",
        output_path,
        index.repos.len(),
        index.nodes.len(),
        index.edges.len(),
        index.conflicts.len()
    );
    Ok(())
}

/// Find system.index.json by walking up from a start directory.
///
/// Checks the start directory and each parent for `system.index.json` or
/// `.sruja/system.index.json`. Returns the first match found.
pub fn find_system_index(start_path: &Path) -> Option<PathBuf> {
    let start = if start_path.is_file() {
        start_path.parent()?
    } else {
        start_path
    };

    let mut current = start;
    loop {
        let index_path = current.join("system.index.json");
        if index_path.is_file() {
            return Some(index_path);
        }
        let sruja_index = current.join(".sruja").join("system.index.json");
        if sruja_index.is_file() {
            return Some(sruja_index);
        }
        match current.parent() {
            Some(p) if p != current => current = p,
            _ => break,
        }
    }
    None
}

/// Load and deserialize a system index from a file path.
pub fn load_system_index(path: &Path) -> Result<SystemIndex, CliError> {
    let content = fs::read_to_string(path).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read system index {}: {}", path.display(), e),
        ))
    })?;
    serde_json::from_str(&content).map_err(|e| {
        CliError::validation(format!("Invalid system index {}: {}", path.display(), e))
    })
}

/// Extract a filtered slice of the system index: only elements of matched kinds.
pub fn filter_system_index_by_kind(index: &SystemIndex, kind: &str) -> SystemIndex {
    let kind_lower = kind.to_lowercase();
    let nodes: Vec<SystemIndexNode> = index
        .nodes
        .iter()
        .filter(|n| n.kind.to_lowercase() == kind_lower)
        .cloned()
        .collect();
    let node_ids: std::collections::HashSet<&str> =
        nodes.iter().map(|n| n.canonical_id.as_str()).collect();
    let edges: Vec<SystemIndexEdge> = index
        .edges
        .iter()
        .filter(|e| node_ids.contains(e.source.as_str()) || node_ids.contains(e.target.as_str()))
        .cloned()
        .collect();
    SystemIndex {
        schema_version: index.schema_version,
        repos: index.repos.clone(),
        nodes,
        edges,
        conflicts: index.conflicts.clone(),
    }
}
