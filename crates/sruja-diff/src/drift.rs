//! Architectural drift detection: cycles, orphans, layer violations, god modules.

use crate::health::calculate_health_score_from_violations;
use crate::source_ref::{collect_cycle_sources, collect_edge_sources, collect_node_path_source};
use crate::types::HealthScorePenalties;
use crate::types::{DriftConfig, DriftReport, Severity, Violation, ViolationKind};
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
            severity: Severity::Warning,
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
            severity: Severity::Info,
            message: format!(
                "Module '{}' has {} dependencies (threshold: {})",
                module.name, module.dependency_count, config.god_module_threshold
            ),
            location: Some(module.name.clone()),
            suggestion: Some(
                "Consider splitting this module into smaller, focused components".to_string(),
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

    let health_score = calculate_drift_health_score(&violations);

    DriftReport {
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
        health_score,
    }
}

/// Find circular dependencies in the graph using DFS.
/// Returns deduplicated cycles (canonicalized by lexicographically smallest rotation).
pub fn find_circular_dependencies(graph: &Graph) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    let mut raw_cycles: Vec<Vec<&str>> = Vec::new();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut rec_stack: HashSet<&str> = HashSet::new();
    let mut path: Vec<&str> = Vec::new();

    for node in &graph.nodes {
        if !visited.contains(node.id.as_str()) {
            dfs_cycles(
                node.id.as_str(),
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut raw_cycles,
            );
        }
    }

    let mut seen: HashSet<Vec<String>> = HashSet::new();
    let mut result = Vec::new();
    for cycle in raw_cycles {
        let canonical = canonicalize_cycle(&cycle);
        if seen.insert(canonical.clone()) {
            result.push(canonical);
        }
    }
    result
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

fn dfs_cycles<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
    cycles: &mut Vec<Vec<&'a str>>,
) {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_cycles(neighbor, adj, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(neighbor) {
                if let Some(cycle_start) = path.iter().position(|n| *n == *neighbor) {
                    let cycle: Vec<&'a str> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
}

/// Path patterns that usually indicate doc, test, or tooling code rather than main product.
/// Orphans and god modules in these paths are excluded so the score reflects product code only.
fn is_likely_doc_or_tool_path(path: &str, id: &str) -> bool {
    let p = path.replace('\\', "/").to_lowercase();
    let id_lower = id.to_lowercase();
    // Path-based: doc, tests, tools, vendor, examples (we look at source, not tests/examples)
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
        || p.contains("/examples/")
        || p.contains("/fixtures/")
        || p.contains("/sample")
        // Id often encodes path (e.g. server_embed_doc_go)
        || id_lower.contains("_doc_go")
        || id_lower.ends_with("_test_go")
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

fn calculate_drift_health_score(violations: &[Violation]) -> u8 {
    calculate_health_score_from_violations(violations, HealthScorePenalties::default())
}
