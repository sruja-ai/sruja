//! Centrality algorithms for component importance scoring
//!
//! Implements multiple centrality measures to identify architecturally important components.

use crate::Graph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Multi-dimensional importance score for a component
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentImportance {
    /// Number of direct connections (in + out)
    pub degree_centrality: f64,
    /// How often node is on shortest paths (bridge nodes)
    pub betweenness_centrality: f64,
    /// Influence based on connected nodes' importance
    pub eigenvector_centrality: f64,
    /// How close to all other nodes
    pub closeness_centrality: f64,
    /// Importance based on incoming connections
    pub pagerank: f64,
}

/// Compute all centrality metrics for all nodes in the graph
pub fn compute_all_centrality(graph: &Graph) -> HashMap<String, ComponentImportance> {
    let mut scores: HashMap<String, ComponentImportance> = graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), ComponentImportance::default()))
        .collect();

    let node_indices = build_node_index(graph);
    let adj = build_adjacency(graph, &node_indices);
    let n = graph.nodes.len();

    if n == 0 {
        return scores;
    }

    compute_degree_centrality(graph, &mut scores);
    compute_betweenness_centrality(&adj, &node_indices, &mut scores, n);
    compute_closeness_centrality(&adj, &node_indices, &mut scores, n);
    compute_pagerank(&adj, &node_indices, &mut scores, n);
    compute_eigenvector_centrality(&adj, &node_indices, &mut scores, n);

    scores
}

fn build_node_index(graph: &Graph) -> HashMap<String, usize> {
    graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect()
}

fn build_adjacency(graph: &Graph, node_indices: &HashMap<String, usize>) -> Vec<Vec<usize>> {
    let n = graph.nodes.len();
    let mut adj = vec![vec![]; n];

    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) = (
            node_indices.get(&edge.source),
            node_indices.get(&edge.target),
        ) {
            adj[src].push(tgt);
        }
    }

    adj
}

fn compute_degree_centrality(graph: &Graph, scores: &mut HashMap<String, ComponentImportance>) {
    let n = graph.nodes.len();
    if n <= 1 {
        return;
    }

    let max_possible = (n - 1) as f64 * 2.0;

    for node in &graph.nodes {
        let in_degree = graph.edges.iter().filter(|e| e.target == node.id).count();
        let out_degree = graph.edges.iter().filter(|e| e.source == node.id).count();
        let total = (in_degree + out_degree) as f64;

        if let Some(importance) = scores.get_mut(&node.id) {
            importance.degree_centrality = total / max_possible;
        }
    }
}

fn compute_betweenness_centrality(
    adj: &[Vec<usize>],
    node_indices: &HashMap<String, usize>,
    scores: &mut HashMap<String, ComponentImportance>,
    n: usize,
) {
    if n <= 2 {
        return;
    }

    let id_to_idx = node_indices;
    let idx_to_id: HashMap<usize, String> =
        id_to_idx.iter().map(|(k, &v)| (v, k.clone())).collect();

    let mut betweenness = vec![0.0f64; n];

    for s in 0..n {
        let mut stack = Vec::new();
        let mut predecessors: Vec<Vec<usize>> = vec![vec![]; n];
        let mut sigma = vec![0f64; n];
        sigma[s] = 1.0;
        let mut distance = vec![-1i32; n];
        distance[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in &adj[v] {
                if distance[w] < 0 {
                    queue.push_back(w);
                    distance[w] = distance[v] + 1;
                }
                if distance[w] == distance[v] + 1 {
                    sigma[w] += sigma[v];
                    predecessors[w].push(v);
                }
            }
        }

        let mut delta = vec![0.0f64; n];
        while let Some(w) = stack.pop() {
            for &v in &predecessors[w] {
                delta[v] += sigma[v] / sigma[w] * (1.0 + delta[w]);
            }
            if w != s {
                betweenness[w] += delta[w];
            }
        }
    }

    let max_betweenness = betweenness.iter().cloned().fold(0.0f64, f64::max).max(1.0);

    for (idx, &b) in betweenness.iter().enumerate() {
        if let Some(id) = idx_to_id.get(&idx) {
            if let Some(importance) = scores.get_mut(id) {
                importance.betweenness_centrality = b / max_betweenness;
            }
        }
    }
}

fn compute_closeness_centrality(
    adj: &[Vec<usize>],
    node_indices: &HashMap<String, usize>,
    scores: &mut HashMap<String, ComponentImportance>,
    n: usize,
) {
    if n <= 1 {
        return;
    }

    let idx_to_id: HashMap<usize, String> =
        node_indices.iter().map(|(k, &v)| (v, k.clone())).collect();

    for s in 0..n {
        let mut visited = vec![false; n];
        let mut distance = vec![usize::MAX; n];
        let mut queue = std::collections::VecDeque::new();

        visited[s] = true;
        distance[s] = 0;
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            for &w in &adj[v] {
                if !visited[w] {
                    visited[w] = true;
                    distance[w] = distance[v] + 1;
                    queue.push_back(w);
                }
            }
        }

        let reachable: usize = distance
            .iter()
            .filter(|&&d| d != usize::MAX && d > 0)
            .count();
        let total_distance: usize = distance.iter().filter(|&&d| d != usize::MAX && d > 0).sum();

        if let Some(id) = idx_to_id.get(&s) {
            if let Some(importance) = scores.get_mut(id) {
                if total_distance > 0 && reachable > 0 {
                    importance.closeness_centrality = (reachable as f64) / (total_distance as f64);
                }
            }
        }
    }
}

fn compute_pagerank(
    adj: &[Vec<usize>],
    node_indices: &HashMap<String, usize>,
    scores: &mut HashMap<String, ComponentImportance>,
    n: usize,
) {
    if n == 0 {
        return;
    }

    let idx_to_id: HashMap<usize, String> =
        node_indices.iter().map(|(k, &v)| (v, k.clone())).collect();

    let damping = 0.85;
    let tolerance = 1e-6;
    let max_iterations = 100;

    let mut out_degree = vec![0usize; n];
    for (v, neighbors) in adj.iter().enumerate() {
        out_degree[v] = neighbors.len();
    }

    let mut in_neighbors: Vec<Vec<usize>> = vec![vec![]; n];
    for (v, neighbors) in adj.iter().enumerate() {
        for &w in neighbors {
            in_neighbors[w].push(v);
        }
    }

    let mut rank = vec![1.0 / n as f64; n];

    for _ in 0..max_iterations {
        let mut new_rank = vec![(1.0 - damping) / n as f64; n];

        for v in 0..n {
            for &u in &in_neighbors[v] {
                if out_degree[u] > 0 {
                    new_rank[v] += damping * rank[u] / out_degree[u] as f64;
                }
            }
        }

        let diff: f64 = new_rank
            .iter()
            .zip(rank.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        rank = new_rank;

        if diff < tolerance {
            break;
        }
    }

    let max_rank = rank.iter().cloned().fold(0.0f64, f64::max).max(1e-10);

    for (idx, &r) in rank.iter().enumerate() {
        if let Some(id) = idx_to_id.get(&idx) {
            if let Some(importance) = scores.get_mut(id) {
                importance.pagerank = r / max_rank;
            }
        }
    }
}

fn compute_eigenvector_centrality(
    adj: &[Vec<usize>],
    node_indices: &HashMap<String, usize>,
    scores: &mut HashMap<String, ComponentImportance>,
    n: usize,
) {
    if n == 0 {
        return;
    }

    let idx_to_id: HashMap<usize, String> =
        node_indices.iter().map(|(k, &v)| (v, k.clone())).collect();

    let mut in_neighbors: Vec<Vec<usize>> = vec![vec![]; n];
    for (v, neighbors) in adj.iter().enumerate() {
        for &w in neighbors {
            in_neighbors[w].push(v);
        }
    }

    let mut centrality = vec![1.0; n];
    let max_iterations = 100;
    let tolerance = 1e-6;

    for _ in 0..max_iterations {
        let mut new_centrality = vec![0.0; n];

        for v in 0..n {
            for &u in &in_neighbors[v] {
                new_centrality[v] += centrality[u];
            }
        }

        let norm: f64 = new_centrality
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt()
            .max(1e-10);
        for c in &mut new_centrality {
            *c /= norm;
        }

        let diff: f64 = new_centrality
            .iter()
            .zip(centrality.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        centrality = new_centrality;

        if diff < tolerance {
            break;
        }
    }

    let max_c = centrality.iter().cloned().fold(0.0f64, f64::max).max(1e-10);

    for (idx, &c) in centrality.iter().enumerate() {
        if let Some(id) = idx_to_id.get(&idx) {
            if let Some(importance) = scores.get_mut(id) {
                importance.eigenvector_centrality = c / max_c;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, EdgeKind, Graph, Node, NodeKind};

    fn make_test_graph() -> Graph {
        let mut graph = Graph::default();
        graph.nodes = vec![
            {
                let mut node = Node::default();
                node.id = "a".into();
                node.kind = NodeKind::Module;
                node.label = "a".into();
                node.path = Some("a.rs".into());
                node
            },
            {
                let mut node = Node::default();
                node.id = "b".into();
                node.kind = NodeKind::Module;
                node.label = "b".into();
                node.path = Some("b.rs".into());
                node
            },
            {
                let mut node = Node::default();
                node.id = "c".into();
                node.kind = NodeKind::Module;
                node.label = "c".into();
                node.path = Some("c.rs".into());
                node
            },
        ];
        graph.edges = vec![
            Edge {
                source: "a".into(),
                target: "b".into(),
                kind: EdgeKind::DependsOn,
                evidence: vec![],
            },
            Edge {
                source: "b".into(),
                target: "c".into(),
                kind: EdgeKind::DependsOn,
                evidence: vec![],
            },
        ];
        graph
    }

    #[test]
    fn test_centrality_computation() {
        let graph = make_test_graph();
        let scores = compute_all_centrality(&graph);

        assert!(scores.contains_key("a"));
        assert!(scores.contains_key("b"));
        assert!(scores.contains_key("c"));

        let b_score = &scores["b"];
        assert!(b_score.degree_centrality > 0.0);
    }

    #[test]
    fn test_bridge_node_high_betweenness() {
        let graph = make_test_graph();
        let scores = compute_all_centrality(&graph);

        let b_score = &scores["b"];
        assert!(b_score.betweenness_centrality > 0.0);
    }
}
