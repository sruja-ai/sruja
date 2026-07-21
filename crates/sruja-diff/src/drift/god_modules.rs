use super::helpers::{is_likely_doc_or_tool_path, is_likely_entry_point};
use sruja_scan::{Graph, NodeKind};
use std::collections::HashMap;

pub struct GodModuleInfo {
    pub name: String,
    pub dependency_count: usize,
}

pub fn find_god_modules_with_config(
    graph: &Graph,
    threshold: usize,
    exclude_barrel_files: bool,
) -> Vec<GodModuleInfo> {
    let mut dep_counts: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *dep_counts.entry(edge.source.as_str()).or_default() += 1;
    }

    graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::MODULE)
        .filter(|n| {
            let path = n.path.as_deref().unwrap_or("");
            !is_likely_doc_or_tool_path(path, &n.id) && !is_likely_entry_point(path, &n.id)
        })
        .filter(|n| {
            if exclude_barrel_files {
                let path = n.path.as_deref().unwrap_or("");
                !sruja_scan::is_barrel_file(std::path::Path::new(path))
            } else {
                true
            }
        })
        .filter_map(|n| {
            let count = dep_counts.get(n.id.as_str()).copied().unwrap_or(0);
            if count >= threshold {
                Some(GodModuleInfo {
                    name: n.id.clone(),
                    dependency_count: count,
                })
            } else {
                None
            }
        })
        .collect()
}
