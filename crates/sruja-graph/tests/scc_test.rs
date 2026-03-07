//! Tests for SCC (strongly connected components) analysis.

use sruja_graph::SccAnalyzer;

fn analyze(nodes: &[&str], edges: &[(&str, &str)]) -> sruja_graph::SccResult {
    let nodes: Vec<String> = nodes.iter().map(|s| (*s).to_string()).collect();
    let edges: Vec<(String, String)> = edges
        .iter()
        .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
        .collect();
    let analyzer = SccAnalyzer::new();
    analyzer.analyze(&nodes, &edges)
}

#[test]
fn empty_graph_has_no_cyclic_sccs() {
    let result = analyze(&[], &[]);
    assert_eq!(result.total_sccs, 0);
    assert_eq!(result.cyclic_sccs, 0);
}

#[test]
fn single_node_has_one_scc_non_cyclic() {
    let result = analyze(&["a"], &[]);
    assert_eq!(result.total_sccs, 1);
    assert_eq!(result.cyclic_sccs, 0);
    assert_eq!(result.components[0].nodes, vec!["a"]);
    assert!(!result.components[0].is_cyclic);
}

#[test]
fn two_node_cycle_is_one_cyclic_scc() {
    let result = analyze(&["a", "b"], &[("a", "b"), ("b", "a")]);
    assert_eq!(result.total_sccs, 1);
    assert_eq!(result.cyclic_sccs, 1);
    assert!(result.components[0].is_cyclic);
    assert_eq!(result.largest_scc_size, 2);
}

#[test]
fn dag_produces_no_cyclic_sccs() {
    let result = analyze(&["a", "b", "c"], &[("a", "b"), ("b", "c"), ("a", "c")]);
    assert_eq!(result.cyclic_sccs, 0);
    assert_eq!(result.total_sccs, 3);
}
