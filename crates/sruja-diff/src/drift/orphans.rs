use super::helpers::{is_likely_doc_or_tool_path, is_likely_entry_point, is_likely_framework_consumed};
use sruja_scan::{Graph, NodeKind};
use std::collections::HashSet;

/// Find modules with no incoming or outgoing dependency edges.
pub fn find_orphan_modules(graph: &Graph) -> Vec<String> {
    find_orphan_modules_with_config(graph, true)
}

/// Find orphan modules with configurable barrel file exclusion.
pub fn find_orphan_modules_with_config(graph: &Graph, exclude_barrel_files: bool) -> Vec<String> {
    let mut has_incoming: HashSet<&str> = HashSet::new();
    let mut has_outgoing: HashSet<&str> = HashSet::new();

    for edge in &graph.edges {
        if edge.source.starts_with("module:") || edge.kind.as_str() == "defines" {
            continue;
        }
        has_outgoing.insert(edge.source.as_str());
        has_incoming.insert(edge.target.as_str());
    }

    graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::MODULE)
        .filter(|n| !n.id.starts_with("module:") && !n.id.contains('#'))
        .filter(|n| !has_incoming.contains(n.id.as_str()) && !has_outgoing.contains(n.id.as_str()))
        .filter(|n| {
            let path = n.path.as_deref().unwrap_or("");
            !is_likely_doc_or_tool_path(path, &n.id)
        })
        .filter(|n| {
            let path = n.path.as_deref().unwrap_or("");
            !is_likely_entry_point(path, &n.id)
        })
        .filter(|n| {
            let path = n.path.as_deref().unwrap_or("");
            !is_likely_framework_consumed(path, &n.id)
        })
        .filter(|n| {
            if exclude_barrel_files {
                let path = n.path.as_deref().unwrap_or("");
                !sruja_scan::is_barrel_file(std::path::Path::new(path))
            } else {
                true
            }
        })
        .map(|n| n.id.clone())
        .collect()
}
