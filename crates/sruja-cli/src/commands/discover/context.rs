use std::collections::{HashMap, HashSet};
use std::path::Path;

use sruja_scan::scan_scope::resolve_scan_scope;
use sruja_scan::Graph;

use super::models::DiscoverContextJson;
use crate::commands::CliError;
use crate::context_detection::build_repo_context;
use crate::commands::scan_repo_cached;

/// Build repo context summary as a string (for prompts or discovery).
pub fn discover_context_string(repo: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;
    discover_context_string_from_graph(repo, repo_path, &graph)
}

/// Build repo context summary from a pre-scanned graph (includes actual structure).
pub fn discover_context_string_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<String, CliError> {
    let context = build_repo_context(repo_path, graph);

    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();

    let mut areas: HashSet<String> = HashSet::new();
    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            let normalized = path.replace('\\', "/");
            let rel = if let Some(ref prefix) = repo_prefix {
                normalized
                    .strip_prefix(prefix)
                    .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else if !repo_arg_norm.is_empty() {
                normalized
                    .strip_prefix(&format!("{}/", repo_arg_norm))
                    .or_else(|| normalized.strip_prefix(&repo_arg_norm))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else {
                normalized.as_str()
            };

            if rel.is_empty() {
                continue;
            }

            let first = rel.split('/').next().unwrap_or("");
            if !first.is_empty()
                && !first.starts_with('.')
                && first != "test-repos"
                && first != "evaluation"
            {
                areas.insert(first.to_string());
            }
        }
    }
    let mut areas: Vec<String> = areas.into_iter().collect();
    areas.sort();

    let arch_style = if context.is_microservices {
        "microservices"
    } else if context.is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    };

    let mut out = String::new();
    out.push_str("# Repo context (for contextual discovery questions)\n\n");
    out.push_str(&format!("**Repo:** {}\n", repo));
    out.push_str(&format!("**Components (scan):** {}\n", graph.nodes.len()));
    out.push_str(&format!("**Edges:** {}\n", graph.edges.len()));
    out.push_str(&format!(
        "**Primary language:** {}\n",
        context.primary_language
    ));
    if let Some(ref fw) = context.framework {
        out.push_str(&format!("**Framework:** {}\n", fw));
    }
    out.push_str(&format!("**Architecture style:** {}\n", arch_style));
    if let Some(ref domain) = context.domain {
        out.push_str(&format!("**Domain (inferred):** {}\n", domain));
    }
    let areas_str = if areas.is_empty() {
        "(none — single directory or flat structure)".to_string()
    } else {
        areas.join(", ")
    };
    out.push_str(&format!(
        "**Suggested areas (from paths):** {}\n",
        areas_str
    ));

    out.push_str("\n## Key Components\n\n");

    let mut file_nodes: Vec<_> = graph.nodes.iter().filter(|n| n.path.is_some()).collect();

    let mut incoming_count: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *incoming_count.entry(&edge.target).or_default() += 1;
    }

    file_nodes.sort_by(|a, b| {
        let a_count = incoming_count.get(a.id.as_str()).copied().unwrap_or(0);
        let b_count = incoming_count.get(b.id.as_str()).copied().unwrap_or(0);
        b_count.cmp(&a_count)
    });

    let top_files = file_nodes.iter().take(20);
    for node in top_files {
        if let Some(ref path) = node.path {
            let rel_path = if let Some(ref prefix) = repo_prefix {
                path.replace('\\', "/")
                    .strip_prefix(prefix)
                    .or_else(|| path.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(path.as_str())
                    .trim_start_matches('/')
                    .to_string()
            } else {
                path.clone()
            };
            let import_count = incoming_count.get(node.id.as_str()).copied().unwrap_or(0);
            out.push_str(&format!(
                "- `{}` ({}, {} imports)\n",
                rel_path,
                node.kind.as_str(),
                import_count
            ));
        }
    }

    out.push_str("\n## Exported Interfaces\n\n");

    let export_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.id.contains('#'))
        .take(30)
        .collect();

    for node in export_nodes {
        out.push_str(&format!("- `{}` ({})\n", node.label, node.kind.as_str()));
    }

    out.push_str("\n## Classification Signals\n\n");
    let mut ambiguous = graph
        .nodes
        .iter()
        .filter(|n| n.confidence.unwrap_or(100) < 70)
        .collect::<Vec<_>>();
    ambiguous.sort_by_key(|n| n.confidence.unwrap_or(100));

    if ambiguous.is_empty() {
        out.push_str("- All nodes classified with high confidence (>70%).\n");
    } else {
        for node in ambiguous.iter().take(10) {
            let signals = node
                .metadata
                .get("classification.signals")
                .cloned()
                .unwrap_or_else(|| "none".to_string());
            out.push_str(&format!(
                "- `{}` (kind={}, confidence={}%, signals=[{}])\n",
                node.id,
                node.kind.as_str(),
                node.confidence.unwrap_or(0),
                signals
            ));
        }
    }

    out.push_str("\nUse this context to derive 2–5 questions tailored to this repo (see skill: contextual discovery).\n");
    Ok(out)
}

/// Build repo context as JSON for machine-readable consumption by agents.
pub fn discover_context_json(repo: &str) -> Result<DiscoverContextJson, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }
    let graph = scan_repo_cached(repo_path)?;
    discover_context_json_from_graph(repo, repo_path, &graph)
}

/// Build repo context as JSON from a pre-scanned graph (avoids rescanning).
pub fn discover_context_json_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<DiscoverContextJson, CliError> {
    let context = build_repo_context(repo_path, graph);
    let architecture_style = if context.is_microservices {
        "microservices"
    } else if context.is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    }
    .to_string();

    let (_, scan_scope) = resolve_scan_scope(repo_path);
    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();
    let mut areas: HashSet<String> = HashSet::new();
    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            let normalized = path.replace('\\', "/");
            let rel = if let Some(ref prefix) = repo_prefix {
                normalized
                    .strip_prefix(prefix)
                    .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else if !repo_arg_norm.is_empty() {
                normalized
                    .strip_prefix(&format!("{}/", repo_arg_norm))
                    .or_else(|| normalized.strip_prefix(&repo_arg_norm))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else {
                normalized.as_str()
            };
            if rel.is_empty() {
                continue;
            }
            let first = rel.split('/').next().unwrap_or("");
            if !first.is_empty()
                && !first.starts_with('.')
                && first != "test-repos"
                && first != "evaluation"
            {
                areas.insert(first.to_string());
            }
        }
    }
    let mut suggested_areas: Vec<String> = areas.into_iter().collect();
    suggested_areas.sort();
    Ok(DiscoverContextJson {
        repo: repo.to_string(),
        scan_scope,
        components: graph.nodes.len(),
        edges: graph.edges.len(),
        primary_language: context.primary_language.clone(),
        framework: context.framework.clone(),
        architecture_style,
        domain: context.domain.clone(),
        suggested_areas,
    })
}
