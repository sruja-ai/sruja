//! Dependency risk scoring for components
//!
//! Identifies components with risky dependency patterns that should be prioritized for coverage.

use sruja_scan::{Graph, Node};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Risk factors for a component
#[derive(Debug, Clone, Default)]
pub struct DependencyRisk {
    /// Part of a circular dependency
    pub has_circular_dep: bool,
    /// Deep dependency chain (>5 levels)
    pub deep_dependency_chain: bool,
    /// Crosses domain/module boundaries
    pub cross_boundary_deps: bool,
    /// Many dependents (high blast radius)
    pub high_blast_radius: bool,
    /// Unstable (frequently changed) - placeholder for git integration
    pub unstable: bool,
    /// Frequently changed in git history (high churn)
    pub high_churn: bool,
    /// Overall risk score (0-1, higher = riskier)
    pub risk_score: f64,
}

/// Git change frequency data for a component
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct GitChangeFrequency {
    pub commit_count: usize,
    pub recent_commit_count: usize,
    pub churn_score: f64,
}

/// Compute dependency risk for a node
pub fn compute_dependency_risk(node: &Node, graph: &Graph) -> DependencyRisk {
    let mut risk = DependencyRisk::default();

    risk.has_circular_dep = is_in_cycle(&node.id, graph);
    risk.deep_dependency_chain = has_deep_chain(&node.id, graph, 5);
    risk.cross_boundary_deps = has_cross_boundary_deps(&node.id, graph);
    risk.high_blast_radius = has_high_blast_radius(&node.id, graph, 10);

    risk.risk_score = compute_risk_score(&risk);

    risk
}

/// Compute dependency risk with git history
#[allow(dead_code)]
pub fn compute_dependency_risk_with_git(
    node: &Node,
    graph: &Graph,
    git_freq: Option<&GitChangeFrequency>,
) -> DependencyRisk {
    let mut risk = compute_dependency_risk(node, graph);

    if let Some(freq) = git_freq {
        risk.high_churn = freq.churn_score > 0.7;
        risk.unstable = freq.recent_commit_count > 5;
    }

    risk.risk_score = compute_risk_score(&risk);
    risk
}

/// Compute all dependency risks for a graph
#[allow(dead_code)]
pub fn compute_all_risks(graph: &Graph) -> HashMap<String, DependencyRisk> {
    graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), compute_dependency_risk(n, graph)))
        .collect()
}

fn compute_risk_score(risk: &DependencyRisk) -> f64 {
    let mut score: f64 = 0.0;

    if risk.has_circular_dep {
        score += 0.30;
    }
    if risk.deep_dependency_chain {
        score += 0.15;
    }
    if risk.cross_boundary_deps {
        score += 0.10;
    }
    if risk.high_blast_radius {
        score += 0.15;
    }
    if risk.unstable {
        score += 0.10;
    }
    if risk.high_churn {
        score += 0.20;
    }

    score.min(1.0)
}

/// Compute git change frequency for all files in a repository
#[allow(dead_code)]
pub fn compute_git_change_frequencies(
    repo_path: &Path,
    nodes: &[Node],
) -> HashMap<String, GitChangeFrequency> {
    let mut frequencies = HashMap::new();

    let git_check = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_path)
        .output();

    if git_check.is_err() || !git_check.map(|o| o.status.success()).unwrap_or(false) {
        return frequencies;
    }

    let log_output = std::process::Command::new("git")
        .args([
            "log",
            "--name-only",
            "--pretty=format:",
            "--since=3.months.ago",
        ])
        .current_dir(repo_path)
        .output();

    let log_lines: Vec<String> = match log_output {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect(),
        Err(_) => return frequencies,
    };

    let mut file_counts: HashMap<String, usize> = HashMap::new();
    for line in &log_lines {
        *file_counts.entry(line.clone()).or_default() += 1;
    }

    let _total_commits = log_lines.len().max(1);
    let max_count = file_counts.values().copied().max().unwrap_or(1);

    for node in nodes {
        if let Some(ref node_path) = node.path {
            let count = file_counts.get(node_path).copied().unwrap_or(0);

            let churn_score = if max_count > 0 {
                count as f64 / max_count as f64
            } else {
                0.0
            };

            frequencies.insert(
                node.id.clone(),
                GitChangeFrequency {
                    commit_count: count,
                    recent_commit_count: count,
                    churn_score,
                },
            );
        }
    }

    frequencies
}

/// Compute git change frequency for a single file
#[allow(dead_code)]
pub fn compute_change_frequency(repo_path: &Path, node: &Node) -> f64 {
    let file_path = match &node.path {
        Some(p) => p,
        None => return 0.0,
    };

    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "--", file_path])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(o) => {
            let count = String::from_utf8_lossy(&o.stdout).lines().count();
            (count as f64 / 50.0).min(1.0)
        }
        Err(_) => 0.0,
    }
}

fn is_in_cycle(node_id: &str, graph: &Graph) -> bool {
    let node_indices: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let n = graph.nodes.len();
    let mut adj = vec![vec![]; n];

    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) = (
            node_indices.get(edge.source.as_str()),
            node_indices.get(edge.target.as_str()),
        ) {
            adj[src].push(tgt);
        }
    }

    let start_idx = match node_indices.get(node_id) {
        Some(&idx) => idx,
        None => return false,
    };

    let mut visited = vec![false; n];
    let mut rec_stack = vec![false; n];

    fn dfs_cycle(
        v: usize,
        adj: &[Vec<usize>],
        visited: &mut [bool],
        rec_stack: &mut [bool],
        start: usize,
        path_len: usize,
    ) -> bool {
        visited[v] = true;
        rec_stack[v] = true;

        for &neighbor in &adj[v] {
            if neighbor == start && path_len > 0 {
                return true;
            }
            if !visited[neighbor] {
                if dfs_cycle(neighbor, adj, visited, rec_stack, start, path_len + 1) {
                    return true;
                }
            } else if rec_stack[neighbor] {
                return true;
            }
        }

        rec_stack[v] = false;
        false
    }

    dfs_cycle(start_idx, &adj, &mut visited, &mut rec_stack, start_idx, 0)
}

fn has_deep_chain(node_id: &str, graph: &Graph, threshold: usize) -> bool {
    let node_indices: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let n = graph.nodes.len();
    let mut adj = vec![vec![]; n];

    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) = (
            node_indices.get(edge.source.as_str()),
            node_indices.get(edge.target.as_str()),
        ) {
            adj[src].push(tgt);
        }
    }

    let start_idx = match node_indices.get(node_id) {
        Some(&idx) => idx,
        None => return false,
    };

    let mut max_depth = 0;
    let mut visited = vec![false; n];

    fn dfs_depth(
        v: usize,
        adj: &[Vec<usize>],
        visited: &mut [bool],
        depth: usize,
        max_depth: &mut usize,
    ) {
        *max_depth = (*max_depth).max(depth);
        visited[v] = true;

        for &neighbor in &adj[v] {
            if !visited[neighbor] {
                dfs_depth(neighbor, adj, visited, depth + 1, max_depth);
            }
        }
    }

    dfs_depth(start_idx, &adj, &mut visited, 0, &mut max_depth);

    max_depth >= threshold
}

fn has_cross_boundary_deps(node_id: &str, graph: &Graph) -> bool {
    let node = match graph.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return false,
    };

    let node_domain = extract_domain(node);

    for edge in &graph.edges {
        if edge.source == node_id {
            if let Some(target) = graph.nodes.iter().find(|n| n.id == edge.target) {
                let target_domain = extract_domain(target);
                if node_domain != target_domain
                    && !node_domain.is_empty()
                    && !target_domain.is_empty()
                {
                    return true;
                }
            }
        }
    }

    false
}

fn has_high_blast_radius(node_id: &str, graph: &Graph, threshold: usize) -> bool {
    let dependents = graph.edges.iter().filter(|e| e.target == node_id).count();

    dependents >= threshold
}

fn extract_domain(node: &Node) -> String {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let parts: Vec<&str> = path.split('/').collect();

    for (i, part) in parts.iter().enumerate() {
        if (*part == "src" || *part == "lib" || *part == "crates" || *part == "packages")
            && i + 1 < parts.len()
        {
            return parts[i + 1].to_string();
        }
    }

    if parts.len() >= 2 {
        return parts[0].to_string();
    }

    String::new()
}

/// Find all cycles in the graph
/// Returns a list of cycles, where each cycle is a list of node IDs
#[allow(dead_code)]
pub fn find_all_cycles(graph: &Graph) -> Vec<Vec<String>> {
    let node_indices: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let idx_to_id: HashMap<usize, &str> = node_indices.iter().map(|(&k, &v)| (v, k)).collect();

    let n = graph.nodes.len();
    let mut adj = vec![vec![]; n];

    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) = (
            node_indices.get(edge.source.as_str()),
            node_indices.get(edge.target.as_str()),
        ) {
            adj[src].push(tgt);
        }
    }

    let mut cycles = Vec::new();
    let mut visited = vec![false; n];
    let mut rec_stack = vec![false; n];
    let mut path = Vec::new();

    fn dfs_cycles(
        v: usize,
        adj: &[Vec<usize>],
        visited: &mut [bool],
        rec_stack: &mut [bool],
        path: &mut Vec<usize>,
        cycles: &mut Vec<Vec<String>>,
        idx_to_id: &HashMap<usize, &str>,
    ) {
        visited[v] = true;
        rec_stack[v] = true;
        path.push(v);

        for &neighbor in &adj[v] {
            if rec_stack[neighbor] {
                if let Some(cycle_start) = path.iter().position(|&x| x == neighbor) {
                    let cycle: Vec<String> = path[cycle_start..]
                        .iter()
                        .filter_map(|&idx| idx_to_id.get(&idx).map(|s| s.to_string()))
                        .collect();
                    if cycle.len() > 1 {
                        cycles.push(cycle);
                    }
                }
            } else if !visited[neighbor] {
                dfs_cycles(neighbor, adj, visited, rec_stack, path, cycles, idx_to_id);
            }
        }

        path.pop();
        rec_stack[v] = false;
    }

    for i in 0..n {
        if !visited[i] {
            dfs_cycles(
                i,
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
                &idx_to_id,
            );
        }
    }

    cycles.sort_by_key(|a| a.len());
    cycles.dedup_by(|a, b| {
        if a.len() != b.len() {
            return false;
        }
        let a_set: HashSet<_> = a.iter().collect();
        let b_set: HashSet<_> = b.iter().collect();
        a_set == b_set
    });

    cycles
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, NodeKind};

    fn make_graph(nodes: Vec<(&str, &str)>, edges: Vec<(&str, &str)>) -> Graph {
        Graph {
            nodes: nodes
                .into_iter()
                .map(|(id, path)| Node {
                    id: id.into(),
                    kind: NodeKind::Module,
                    label: id.into(),
                    technology: None,
                    path: Some(path.into()),
                    metadata: Default::default(),
                })
                .collect(),
            edges: edges
                .into_iter()
                .map(|(s, t)| Edge {
                    source: s.into(),
                    target: t.into(),
                    kind: EdgeKind::DependsOn,
                    evidence: vec![],
                })
                .collect(),
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_no_cycle() {
        let graph = make_graph(vec![("a", "src/a"), ("b", "src/b")], vec![("a", "b")]);

        let risk = compute_dependency_risk(&graph.nodes[0], &graph);
        assert!(!risk.has_circular_dep);
    }

    #[test]
    fn test_cycle_detection() {
        let graph = make_graph(
            vec![("a", "src/a"), ("b", "src/b"), ("c", "src/c")],
            vec![("a", "b"), ("b", "c"), ("c", "a")],
        );

        let risk = compute_dependency_risk(&graph.nodes[0], &graph);
        assert!(risk.has_circular_dep);
    }

    #[test]
    fn test_deep_chain() {
        let graph = make_graph(
            vec![
                ("a", "src/a"),
                ("b", "src/b"),
                ("c", "src/c"),
                ("d", "src/d"),
                ("e", "src/e"),
                ("f", "src/f"),
            ],
            vec![("a", "b"), ("b", "c"), ("c", "d"), ("d", "e"), ("e", "f")],
        );

        let risk = compute_dependency_risk(&graph.nodes[0], &graph);
        assert!(risk.deep_dependency_chain);
    }

    #[test]
    fn test_high_blast_radius() {
        let mut node_specs: Vec<(String, String)> = vec![("target".into(), "src/target".into())];
        let mut edge_specs: Vec<(String, String)> = Vec::new();
        for i in 0..15 {
            node_specs.push((format!("dep_{}", i), format!("src/dep_{}", i)));
            edge_specs.push((format!("dep_{}", i), "target".into()));
        }

        let graph = Graph {
            nodes: node_specs
                .into_iter()
                .map(|(id, path)| {
                    let label = id.clone();
                    Node {
                        id,
                        kind: NodeKind::Module,
                        label,
                        technology: None,
                        path: Some(path),
                        metadata: Default::default(),
                    }
                })
                .collect(),
            edges: edge_specs
                .into_iter()
                .map(|(s, t)| Edge {
                    source: s,
                    target: t,
                    kind: EdgeKind::DependsOn,
                    evidence: vec![],
                })
                .collect(),
            metadata: Default::default(),
        };

        let target_node = graph.nodes.iter().find(|n| n.id == "target").unwrap();
        let risk = compute_dependency_risk(target_node, &graph);
        assert!(risk.high_blast_radius);
    }

    #[test]
    fn test_risk_score_calculation() {
        let graph = make_graph(
            vec![("a", "src/a"), ("b", "src/b"), ("c", "src/c")],
            vec![("a", "b"), ("b", "c"), ("c", "a")],
        );

        let risk = compute_dependency_risk(&graph.nodes[0], &graph);
        assert!(risk.risk_score > 0.0);
        assert!(risk.risk_score <= 1.0);
    }

    #[test]
    fn test_find_all_cycles() {
        let graph = make_graph(
            vec![("a", "src/a"), ("b", "src/b"), ("c", "src/c")],
            vec![("a", "b"), ("b", "c"), ("c", "a")],
        );

        let cycles = find_all_cycles(&graph);
        assert!(!cycles.is_empty());
    }
}
