//! Centrality algorithms for component importance scoring
//!
//! Delegates to `sruja-graph-core::CentralityAnalyzer` for the 3 shared measures
//! (degree, betweenness, closeness) and adds pagerank and eigenvector.

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
    use sruja_graph_core::centrality::CentralityAnalyzer;

    let analyzer = CentralityAnalyzer::new();
    let result = analyzer.analyze_graph(graph);

    // Merge all 5 measures into ComponentImportance
    let mut scores: HashMap<String, ComponentImportance> = HashMap::new();
    for node in &graph.nodes {
        scores.insert(
            node.id.clone(),
            ComponentImportance {
                degree_centrality: result.degree.get(&node.id).copied().unwrap_or(0.0),
                betweenness_centrality: result.betweenness.get(&node.id).copied().unwrap_or(0.0),
                eigenvector_centrality: result.eigenvector.get(&node.id).copied().unwrap_or(0.0),
                closeness_centrality: result.closeness.get(&node.id).copied().unwrap_or(0.0),
                pagerank: result.pagerank.get(&node.id).copied().unwrap_or(0.0),
            },
        );
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, EdgeKind, Graph, Node, NodeKind};

    fn make_test_graph() -> Graph {
        Graph {
            nodes: vec![
                Node {
                    id: "a".into(),
                    kind: NodeKind::new(NodeKind::MODULE),
                    label: "a".into(),
                    path: Some("a.rs".into()),
                    ..Node::default()
                },
                Node {
                    id: "b".into(),
                    kind: NodeKind::new(NodeKind::MODULE),
                    label: "b".into(),
                    path: Some("b.rs".into()),
                    ..Node::default()
                },
                Node {
                    id: "c".into(),
                    kind: NodeKind::new(NodeKind::MODULE),
                    label: "c".into(),
                    path: Some("c.rs".into()),
                    ..Node::default()
                },
            ],
            edges: vec![
                Edge {
                    source: "a".into(),
                    target: "b".into(),
                    kind: EdgeKind::new(EdgeKind::DEPENDS_ON),
                    evidence: vec![],
                    confidence: Default::default(),
                },
                Edge {
                    source: "b".into(),
                    target: "c".into(),
                    kind: EdgeKind::new(EdgeKind::DEPENDS_ON),
                    evidence: vec![],
                    confidence: Default::default(),
                },
            ],
            ..Graph::default()
        }
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
        assert!(
            b_score.betweenness_centrality > 0.0,
            "Bridge node b should have non-zero betweenness"
        );
    }

    #[test]
    fn test_pagerank_and_eigenvector() {
        let graph = make_test_graph();
        let scores = compute_all_centrality(&graph);

        for importance in scores.values() {
            assert!(
                importance.pagerank >= 0.0 && importance.pagerank <= 1.0,
                "PageRank should be normalized"
            );
            assert!(
                importance.eigenvector_centrality >= 0.0 && importance.eigenvector_centrality <= 1.0,
                "Eigenvector should be normalized"
            );
        }
    }
}
