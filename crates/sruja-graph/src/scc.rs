//! Strongly Connected Components (SCC) Analysis
//!
//! Uses Tarjan's algorithm to find maximal cyclic subgraphs in dependency graphs.
//! SCCs reveal tightly coupled clusters and potential domain boundaries.

use std::collections::{HashMap, HashSet};

pub type SccId = String;

pub struct SccAnalyzer {
    min_size: usize,
}

pub struct SccResult {
    pub components: Vec<Scc>,
    pub condensation_dag: Vec<CondensationEdge>,
    pub total_sccs: usize,
    pub cyclic_sccs: usize,
    pub largest_scc_size: usize,
}

pub struct Scc {
    pub id: SccId,
    pub nodes: Vec<String>,
    pub is_cyclic: bool,
    pub internal_density: f32,
    pub suggested_boundary: Option<String>,
}

pub struct CondensationEdge {
    pub from_scc: SccId,
    pub to_scc: SccId,
}

impl Default for SccAnalyzer {
    fn default() -> Self {
        Self { min_size: 1 }
    }
}

impl SccAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_size(min_size: usize) -> Self {
        Self { min_size }
    }

    pub fn analyze(&self, nodes: &[String], edges: &[(String, String)]) -> SccResult {
        let adj = build_adjacency_list(nodes, edges);
        let raw_sccs = tarjan_scc(&adj);

        let components: Vec<Scc> = raw_sccs
            .into_iter()
            .enumerate()
            .filter(|(_, nodes)| nodes.len() >= self.min_size)
            .map(|(i, scc_nodes)| {
                let is_cyclic = scc_nodes.len() > 1 || has_self_loop(&scc_nodes, edges);
                let internal_density = calculate_internal_density(&scc_nodes, edges);
                let suggested_boundary = if is_cyclic && scc_nodes.len() > 2 {
                    Some(suggest_boundary(&scc_nodes))
                } else {
                    None
                };

                Scc {
                    id: format!("scc_{}", i),
                    nodes: scc_nodes,
                    is_cyclic,
                    internal_density,
                    suggested_boundary,
                }
            })
            .collect();

        let cyclic_count = components.iter().filter(|c| c.is_cyclic).count();
        let largest_size = components.iter().map(|c| c.nodes.len()).max().unwrap_or(0);

        let condensation_dag = build_condensation_dag(&components, edges);

        SccResult {
            total_sccs: components.len(),
            cyclic_sccs: cyclic_count,
            largest_scc_size: largest_size,
            components,
            condensation_dag,
        }
    }
}

fn build_adjacency_list(
    nodes: &[String],
    edges: &[(String, String)],
) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for node in nodes {
        adj.entry(node.clone()).or_default();
    }

    for (source, target) in edges {
        adj.entry(source.clone()).or_default().push(target.clone());
    }

    adj
}

fn tarjan_scc(adj: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut index_counter: usize = 0;
    let mut indices: HashMap<String, usize> = HashMap::new();
    let mut lowlinks: HashMap<String, usize> = HashMap::new();
    let mut on_stack: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    let mut sccs: Vec<Vec<String>> = Vec::new();

    for node in adj.keys() {
        if !indices.contains_key(node) {
            strongconnect(
                node,
                adj,
                &mut index_counter,
                &mut indices,
                &mut lowlinks,
                &mut on_stack,
                &mut stack,
                &mut sccs,
            );
        }
    }

    sccs
}

#[allow(clippy::too_many_arguments)]
fn strongconnect(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    index_counter: &mut usize,
    indices: &mut HashMap<String, usize>,
    lowlinks: &mut HashMap<String, usize>,
    on_stack: &mut HashSet<String>,
    stack: &mut Vec<String>,
    sccs: &mut Vec<Vec<String>>,
) {
    let index = *index_counter;
    *index_counter += 1;

    indices.insert(node.to_string(), index);
    lowlinks.insert(node.to_string(), index);
    on_stack.insert(node.to_string());
    stack.push(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !indices.contains_key(neighbor) {
                strongconnect(
                    neighbor,
                    adj,
                    index_counter,
                    indices,
                    lowlinks,
                    on_stack,
                    stack,
                    sccs,
                );

                let neighbor_lowlink = *lowlinks.get(neighbor).unwrap();
                let node_lowlink = *lowlinks.get(node).unwrap();
                lowlinks.insert(node.to_string(), node_lowlink.min(neighbor_lowlink));
            } else if on_stack.contains(neighbor) {
                let neighbor_index = *indices.get(neighbor).unwrap();
                let node_lowlink = *lowlinks.get(node).unwrap();
                lowlinks.insert(node.to_string(), node_lowlink.min(neighbor_index));
            }
        }
    }

    let node_lowlink = *lowlinks.get(node).unwrap();
    let node_index = *indices.get(node).unwrap();

    if node_lowlink == node_index {
        let mut scc: Vec<String> = Vec::new();
        loop {
            let w = stack.pop().unwrap();
            on_stack.remove(&w);
            scc.push(w.clone());
            if w == node {
                break;
            }
        }
        sccs.push(scc);
    }
}

fn has_self_loop(nodes: &[String], edges: &[(String, String)]) -> bool {
    if nodes.len() != 1 {
        return false;
    }
    let node = &nodes[0];
    edges.iter().any(|(s, t)| s == node && t == node)
}

fn calculate_internal_density(nodes: &[String], edges: &[(String, String)]) -> f32 {
    if nodes.len() <= 1 {
        return 0.0;
    }

    let node_set: HashSet<&str> = nodes.iter().map(|s| s.as_str()).collect();

    let internal_edges = edges
        .iter()
        .filter(|(s, t)| node_set.contains(s.as_str()) && node_set.contains(t.as_str()))
        .count();

    let max_possible = nodes.len() * (nodes.len() - 1);

    if max_possible == 0 {
        return 0.0;
    }

    internal_edges as f32 / max_possible as f32
}

fn suggest_boundary(nodes: &[String]) -> String {
    let common_prefix = find_common_prefix(nodes);
    if !common_prefix.is_empty() {
        format!("{}_context", common_prefix)
    } else {
        "shared_context".to_string()
    }
}

fn find_common_prefix(nodes: &[String]) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let parts: Vec<&str> = nodes[0]
        .split(['.', '/', '_'])
        .collect();

    for prefix_len in (1..=parts.len()).rev() {
        let prefix = parts[..prefix_len].join("_");
        if nodes.iter().all(|n| {
            n.split(['.', '/', '_'])
                .take(prefix_len)
                .collect::<Vec<_>>()
                .join("_")
                == prefix
        }) {
            return prefix;
        }
    }

    String::new()
}

fn build_condensation_dag(sccs: &[Scc], edges: &[(String, String)]) -> Vec<CondensationEdge> {
    let mut node_to_scc: HashMap<String, SccId> = HashMap::new();

    for scc in sccs {
        for node in &scc.nodes {
            node_to_scc.insert(node.clone(), scc.id.clone());
        }
    }

    let mut dag_edges: HashSet<(SccId, SccId)> = HashSet::new();

    for (source, target) in edges {
        if let (Some(from_scc), Some(to_scc)) = (node_to_scc.get(source), node_to_scc.get(target)) {
            if from_scc != to_scc {
                dag_edges.insert((from_scc.clone(), to_scc.clone()));
            }
        }
    }

    dag_edges
        .into_iter()
        .map(|(from_scc, to_scc)| CondensationEdge { from_scc, to_scc })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let analyzer = SccAnalyzer::new();
        let result = analyzer.analyze(&[], &[]);
        assert_eq!(result.total_sccs, 0);
    }

    #[test]
    fn test_single_node() {
        let analyzer = SccAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let result = analyzer.analyze(&nodes, &[]);
        assert_eq!(result.total_sccs, 1);
        assert!(!result.components[0].is_cyclic);
    }

    #[test]
    fn test_self_loop() {
        let analyzer = SccAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let edges = vec![("a".to_string(), "a".to_string())];
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.total_sccs, 1);
        assert!(result.components[0].is_cyclic);
    }

    #[test]
    fn test_simple_cycle() {
        let analyzer = SccAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.total_sccs, 1);
        assert!(result.components[0].is_cyclic);
        assert_eq!(result.components[0].nodes.len(), 2);
    }

    #[test]
    fn test_dag_no_cycles() {
        let analyzer = SccAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.total_sccs, 3);
        assert_eq!(result.cyclic_sccs, 0);
    }

    #[test]
    fn test_condensation_dag() {
        let analyzer = SccAnalyzer::new();
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
            ("b".to_string(), "c".to_string()),
            ("c".to_string(), "d".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        assert!(result.condensation_dag.len() > 0);
    }

    #[test]
    fn test_internal_density() {
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
        ];
        let density = calculate_internal_density(&nodes, &edges);
        assert!(density > 0.0);
    }
}
