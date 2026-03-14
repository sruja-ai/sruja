//! Architectural drift detection: cycles, orphans, layer violations, god modules.

use crate::health::calculate_health_score_with_breakdown;
use crate::source_ref::{collect_cycle_sources, collect_edge_sources, collect_node_path_source};
use crate::types::HealthScorePenalties;
use crate::types::{
    DriftConfig, DriftReport, HealthScoreBreakdown, Severity, Violation, ViolationKind,
};
use sruja_scan::{Graph, NodeKind};
use std::collections::{HashMap, HashSet};

/// Detect architectural drift in a codebase by analyzing the scanned graph
/// for common architectural issues like circular dependencies, god modules,
/// layer violations, and orphan components.
pub fn detect_architectural_drift(graph: &Graph) -> DriftReport {
    detect_architectural_drift_with_config(graph, &DriftConfig::default())
}

/// Detect architectural drift with custom configuration.
pub fn detect_architectural_drift_with_config(graph: &Graph, config: &DriftConfig) -> DriftReport {
    let mut violations = Vec::new();
    let mut suggestions = Vec::new();

    let circular = find_circular_dependencies(graph);
    for cycle in &circular {
        let sources = collect_cycle_sources(graph, cycle);
        violations.push(Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: format!("Circular dependency detected: {}", cycle.join(" -> ")),
            location: Some(cycle.first().cloned().unwrap_or_default()),
            suggestion: Some(
                "Consider introducing an interface or event-based communication to break the cycle"
                    .to_string(),
            ),
            sources,
        });
    }

    let orphans = find_orphan_modules(graph);
    for orphan in &orphans {
        let sources = collect_node_path_source(graph, orphan);
        violations.push(Violation {
            kind: ViolationKind::OrphanComponent,
            severity: Severity::Info,
            message: format!("Module '{}' has no incoming or outgoing dependencies", orphan),
            location: Some(orphan.clone()),
            suggestion: Some(
                "Consider if this module is still needed or if it should be connected to the rest of the system".to_string(),
            ),
            sources,
        });
    }

    let layer_violations = find_layer_violations_advanced(graph);
    for violation in &layer_violations {
        let sources = collect_edge_sources(graph, &violation.source, &violation.target);
        violations.push(Violation {
            kind: ViolationKind::LayerViolation,
            severity: Severity::Warning,
            message: format!(
                "Layer violation: '{}' directly accesses '{}'",
                violation.source, violation.target
            ),
            location: Some(format!("{} -> {}", violation.source, violation.target)),
            suggestion: Some(
                "Consider adding a service layer to abstract this dependency".to_string(),
            ),
            sources,
        });
    }

    let god_modules = find_god_modules(graph, config.god_module_threshold);
    for module in &god_modules {
        let sources = collect_node_path_source(graph, &module.name);
        violations.push(Violation {
            kind: ViolationKind::GodModule,
            severity: Severity::Warning,
            message: format!(
                "Bottleneck Detected: Module '{}' acts as a 'God Module' with {} dependencies (threshold: {})",
                module.name, module.dependency_count, config.god_module_threshold
            ),
            location: Some(module.name.clone()),
            suggestion: Some(
                "Consider splitting this module into smaller, focused components to reduce regression risk".to_string(),
            ),
            sources,
        });
    }

    if !circular.is_empty() {
        suggestions.push("Fix circular dependencies to improve maintainability".to_string());
    }
    if !orphans.is_empty() {
        suggestions
            .push("Review orphan modules - they may be dead code or need integration".to_string());
    }
    if !layer_violations.is_empty() {
        suggestions.push("Introduce proper layering to reduce coupling".to_string());
    }
    if !god_modules.is_empty() {
        suggestions.push("Refactor god modules into smaller components".to_string());
    }

    let penalties = HealthScorePenalties::default();
    let breakdown = calculate_health_score_with_breakdown(&violations, penalties);
    let health_breakdown = Some(HealthScoreBreakdown {
        cycle_penalty: breakdown.cycle_penalty,
        layer_penalty: breakdown.zone_of_pain_penalty,
        god_module_penalty: breakdown.god_module_penalty,
        orphan_penalty: breakdown.orphan_penalty,
        other_penalty: breakdown.other_penalty,
    });

    DriftReport {
        scan_scope: sruja_scan::scan_scope::ScanScope::default(),
        total_modules: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Module)
            .count(),
        total_services: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Service)
            .count(),
        total_databases: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .count(),
        total_dependencies: graph.edges.len(),
        circular_dependencies: circular.len(),
        orphan_modules: orphans.len(),
        layer_violations: layer_violations.len(),
        violations,
        suggestions,
        health_score: breakdown.score,
        health_breakdown,
    }
}

/// Find circular dependencies in the graph using Tarjan's SCC algorithm.
/// Returns deduplicated cycles (canonicalized by lexicographically smallest rotation).
pub fn find_circular_dependencies(graph: &Graph) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    // Find all strongly connected components (SCCs)
    let sccs = tarjan_scc(&graph.nodes, &adj);

    // Convert SCCs with more than 1 node to cycles
    let mut result = Vec::new();
    for scc in sccs {
        if scc.len() > 1 {
            // For SCCs, we need to find an actual cycle path
            if let Some(cycle) = find_cycle_in_scc(&scc, &adj) {
                let canonical = canonicalize_cycle(&cycle);
                result.push(canonical);
            }
        }
    }

    result
}

/// Tarjan's algorithm for finding strongly connected components
fn tarjan_scc<'a>(
    nodes: &'a [sruja_scan::Node],
    adj: &HashMap<&'a str, Vec<&'a str>>,
) -> Vec<Vec<&'a str>> {
    let mut index_counter = 0usize;
    let mut stack: Vec<&'a str> = Vec::new();
    let mut indices: HashMap<&'a str, usize> = HashMap::new();
    let mut lowlinks: HashMap<&'a str, usize> = HashMap::new();
    let mut on_stack: HashSet<&'a str> = HashSet::new();
    let mut sccs: Vec<Vec<&'a str>> = Vec::new();

    for node in nodes {
        if !indices.contains_key(node.id.as_str()) {
            tarjan_strongconnect(
                node.id.as_str(),
                adj,
                &mut index_counter,
                &mut stack,
                &mut indices,
                &mut lowlinks,
                &mut on_stack,
                &mut sccs,
            );
        }
    }

    sccs
}

#[allow(clippy::too_many_arguments)]
fn tarjan_strongconnect<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    index_counter: &mut usize,
    stack: &mut Vec<&'a str>,
    indices: &mut HashMap<&'a str, usize>,
    lowlinks: &mut HashMap<&'a str, usize>,
    on_stack: &mut HashSet<&'a str>,
    sccs: &mut Vec<Vec<&'a str>>,
) {
    indices.insert(node, *index_counter);
    lowlinks.insert(node, *index_counter);
    *index_counter += 1;
    stack.push(node);
    on_stack.insert(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !indices.contains_key(neighbor) {
                tarjan_strongconnect(
                    neighbor,
                    adj,
                    index_counter,
                    stack,
                    indices,
                    lowlinks,
                    on_stack,
                    sccs,
                );
                let node_lowlink = *lowlinks.get(node).unwrap_or(&usize::MAX);
                let neighbor_lowlink = *lowlinks.get(neighbor).unwrap_or(&usize::MAX);
                lowlinks.insert(node, node_lowlink.min(neighbor_lowlink));
            } else if on_stack.contains(neighbor) {
                let node_lowlink = *lowlinks.get(node).unwrap_or(&usize::MAX);
                let neighbor_index = *indices.get(neighbor).unwrap_or(&usize::MAX);
                lowlinks.insert(node, node_lowlink.min(neighbor_index));
            }
        }
    }

    if lowlinks.get(node) == indices.get(node) {
        let mut scc = Vec::new();
        loop {
            let top = stack.pop().unwrap();
            on_stack.remove(top);
            scc.push(top);
            if top == node {
                break;
            }
        }
        sccs.push(scc);
    }
}

/// Find an actual cycle path within a strongly connected component
fn find_cycle_in_scc<'a>(
    scc: &[&'a str],
    adj: &HashMap<&'a str, Vec<&'a str>>,
) -> Option<Vec<&'a str>> {
    if scc.is_empty() {
        return None;
    }

    let scc_set: HashSet<&str> = scc.iter().copied().collect();

    // Start from any node and do a DFS within the SCC to find a cycle
    let start = scc[0];
    let mut visited: HashSet<&str> = HashSet::new();
    let mut path: Vec<&'a str> = Vec::new();

    dfs_find_cycle(start, &scc_set, adj, &mut visited, &mut path)
}

fn dfs_find_cycle<'a>(
    node: &'a str,
    scc: &HashSet<&str>,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<&'a str>> {
    visited.insert(node);
    path.push(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            // Only consider neighbors within the SCC
            if !scc.contains(neighbor) {
                continue;
            }

            if path.contains(neighbor) {
                // Found a cycle
                if let Some(cycle_start) = path.iter().position(|&n| n == *neighbor) {
                    return Some(path[cycle_start..].to_vec());
                }
            } else if !visited.contains(neighbor) {
                if let Some(cycle) = dfs_find_cycle(neighbor, scc, adj, visited, path) {
                    return Some(cycle);
                }
            }
        }
    }

    path.pop();
    None
}

fn canonicalize_cycle(cycle: &[&str]) -> Vec<String> {
    if cycle.is_empty() {
        return Vec::new();
    }
    let n = cycle.len();
    let mut best_start = 0;
    for start in 1..n {
        for i in 0..n {
            let a = cycle[(best_start + i) % n];
            let b = cycle[(start + i) % n];
            match a.cmp(b) {
                std::cmp::Ordering::Less => break,
                std::cmp::Ordering::Greater => {
                    best_start = start;
                    break;
                }
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    (0..n)
        .map(|i| cycle[(best_start + i) % n].to_string())
        .collect()
}

/// Path patterns that usually indicate doc, test, or tooling code rather than main product.
/// Orphans and god modules in these paths are excluded so the score reflects product code only.
fn is_likely_doc_or_tool_path(path: &str, id: &str) -> bool {
    let p = path.replace('\\', "/").to_lowercase();
    let id_lower = id.to_lowercase();
    // Path-based: doc, tests, tools, vendor, examples, configs, mocks, migrations
    p.ends_with("doc.go")
        || p.contains("/doc/")
        || p.contains("_test.go")
        || p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("__tests__")
        || p.contains(".spec.")
        || p.contains(".test.")
        || p.contains("/tools/")
        || p.contains("/vendor/")
        || p.contains("/third_party/")
        || p.contains("/deps/")
        || p.contains("node_modules/")
        || p.contains("/stories/")
        || p.contains(".stories.")
        || p.contains("/examples/")
        || p.contains("/fixtures/")
        || p.contains("/sample")
        || p.contains("/mocks/")
        || p.contains("/mock/")
        || p.contains(".config.")
        || p.contains("config/")
        || p.contains("/scripts/")
        || p.contains("/build/")
        || p.contains("/migrations/")
        || p.contains("/setup/")
        // Configuration files and typical non-product entry points
        || p.ends_with("webpack.config.js")
        || p.ends_with("vite.config.ts")
        || p.ends_with("jest.config.js")
        || p.ends_with(".eslintrc.js")
        || p.ends_with("tailwind.config.js")
        // Id often encodes path (e.g. server_embed_doc_go)
        || id_lower.contains("_doc_go")
        || id_lower.ends_with("_test_go")
        || id_lower.contains("_config_")
        || id_lower.contains("_mock_")
        || id_lower.contains("_stories_")
}

/// Paths that are commonly entry points or re-export hubs; reporting them as orphans
/// is usually a false positive (scanner may not see dynamic requires or re-exports).
fn is_likely_entry_point(path: &str, _id: &str) -> bool {
    let p = path.replace('\\', "/");
    let p_lower = p.to_lowercase();
    // Common JS/TS entry file names (often no static imports in the file itself)
    if p_lower.ends_with("index.js")
        || p_lower.ends_with("index.ts")
        || p_lower.ends_with("index.jsx")
        || p_lower.ends_with("index.tsx")
        || p_lower.ends_with("main.js")
        || p_lower.ends_with("main.ts")
        || p_lower.ends_with("app.js")
        || p_lower.ends_with("app.ts")
    {
        return !p_lower.contains("/examples/")
            && !p_lower.contains("/tests/")
            && !p_lower.contains("/test/")
            && !p_lower.contains("_test.");
    }
    // Node package core: top-level lib/*.js (e.g. Express lib/request.js) are often
    // required via require() which the scanner may not extract; exclude to reduce noise.
    if p_lower.ends_with(".js") && p_lower.contains("/lib/") {
        let after_lib = p_lower.split("/lib/").last().unwrap_or("");
        if !after_lib.contains('/') {
            return true;
        }
    }
    // Rust crate root lib.rs: often only re-exports, so few or no edges to other crates
    if p_lower.ends_with("/src/lib.rs") {
        return true;
    }
    false
}

/// Find modules with no incoming or outgoing dependency edges.
/// Excludes containment edges (module:->file) and nodes that look like doc/tools/tests
/// so health score is not dominated by false positives in Go/JS/Python repos.
pub fn find_orphan_modules(graph: &Graph) -> Vec<String> {
    let mut has_incoming: HashSet<&str> = HashSet::new();
    let mut has_outgoing: HashSet<&str> = HashSet::new();

    for edge in &graph.edges {
        if !edge.source.starts_with("module:") {
            has_outgoing.insert(edge.source.as_str());
            has_incoming.insert(edge.target.as_str());
        }
    }

    graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
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
        .map(|n| n.id.clone())
        .collect()
}

struct LayerViolationInfo {
    source: String,
    target: String,
}

fn find_layer_violations_advanced(graph: &Graph) -> Vec<LayerViolationInfo> {
    let mut violations = Vec::new();

    let db_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Database)
        .map(|n| n.id.as_str())
        .collect();

    let frontend_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.label.contains("frontend") || n.label.contains("ui") || n.label.contains("web")
        })
        .map(|n| n.id.as_str())
        .collect();

    for edge in &graph.edges {
        if frontend_nodes.contains(edge.source.as_str()) && db_nodes.contains(edge.target.as_str())
        {
            violations.push(LayerViolationInfo {
                source: edge.source.clone(),
                target: edge.target.clone(),
            });
        }
    }

    violations
}

struct GodModuleInfo {
    name: String,
    dependency_count: usize,
}

fn find_god_modules(graph: &Graph, threshold: usize) -> Vec<GodModuleInfo> {
    let mut dep_counts: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *dep_counts.entry(edge.source.as_str()).or_default() += 1;
    }

    graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .filter(|n| {
            let path = n.path.as_deref().unwrap_or("");
            !is_likely_doc_or_tool_path(path, &n.id)
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

#[cfg(test)]
mod tests {
    use super::{find_circular_dependencies, find_orphan_modules};
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};
    use std::collections::HashMap;

    fn node(id: &str, kind: NodeKind, path: Option<&str>) -> Node {
        Node {
            id: id.to_string(),
            kind,
            label: id.to_string(),
            path: path.map(String::from),
            technology: None,
            metadata: HashMap::new(),
        }
    }

    fn edge(source: &str, target: &str) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind: EdgeKind::Calls,
            evidence: vec![],
        }
    }

    #[test]
    fn find_circular_dependencies_detects_simple_cycle() {
        let mut g = Graph::default();
        g.nodes.push(node("a", NodeKind::Module, None));
        g.nodes.push(node("b", NodeKind::Module, None));
        g.edges.push(edge("a", "b"));
        g.edges.push(edge("b", "a"));

        let cycles = find_circular_dependencies(&g);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 2);
    }

    #[test]
    fn find_circular_dependencies_no_cycle_returns_empty() {
        let mut g = Graph::default();
        g.nodes.push(node("a", NodeKind::Module, None));
        g.nodes.push(node("b", NodeKind::Module, None));
        g.edges.push(edge("a", "b"));

        let cycles = find_circular_dependencies(&g);
        assert!(cycles.is_empty());
    }

    #[test]
    fn find_orphan_modules_detects_isolated_node() {
        let mut g = Graph::default();
        g.nodes.push(node("a", NodeKind::Module, Some("src/a.rs")));
        g.nodes.push(node("b", NodeKind::Module, Some("src/b.rs")));
        g.edges.push(edge("a", "b"));

        let orphans = find_orphan_modules(&g);
        assert!(orphans.is_empty(), "a and b are connected");

        g.nodes
            .push(node("orphan", NodeKind::Module, Some("src/orphan.rs")));
        let orphans = find_orphan_modules(&g);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0], "orphan");
    }
}
