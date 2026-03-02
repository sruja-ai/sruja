//! Tests for graph comparison and drift detection.

#[cfg(test)]
mod tests {
    use crate::{compare_graphs, detect_architectural_drift, ViolationKind};
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};
    use std::collections::HashMap;

    fn make_node(id: &str, kind: NodeKind, label: &str) -> Node {
        Node {
            id: id.to_string(),
            kind,
            label: label.to_string(),
            technology: None,
            path: None,
            metadata: HashMap::new(),
        }
    }

    fn make_edge(source: &str, target: &str, kind: EdgeKind) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind,
            evidence: Vec::new(),
        }
    }

    #[test]
    fn test_compare_empty_graphs() {
        let actual = Graph::new();
        let proposed = Graph::new();
        let result = compare_graphs(&actual, &proposed);
        assert!(result.is_empty());
    }

    #[test]
    fn test_detect_added_node() {
        let actual = Graph::new();
        let mut proposed = Graph::new();
        proposed
            .nodes
            .push(make_node("api", NodeKind::Service, "API"));

        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.added.len(), 1);
        assert_eq!(result.node_diff.added[0].id, "api");
    }

    #[test]
    fn test_detect_removed_node() {
        let mut actual = Graph::new();
        actual
            .nodes
            .push(make_node("old", NodeKind::Service, "Old Service"));
        let proposed = Graph::new();

        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.removed.len(), 1);
    }

    #[test]
    fn test_detect_layer_violation() {
        let actual = Graph::new();
        let mut proposed = Graph::new();

        proposed
            .nodes
            .push(make_node("frontend", NodeKind::Module, "Frontend"));
        proposed
            .nodes
            .push(make_node("db", NodeKind::Database, "Database"));
        proposed
            .edges
            .push(make_edge("frontend", "db", EdgeKind::ReadsFrom));

        let result = compare_graphs(&actual, &proposed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::LayerViolation));
    }

    #[test]
    fn test_detect_architectural_drift_cycle_and_orphan() {
        let mut graph = Graph::new();
        graph.nodes.push(make_node("a", NodeKind::Module, "A"));
        graph.nodes.push(make_node("b", NodeKind::Module, "B"));
        graph.nodes.push(make_node("c", NodeKind::Module, "C"));
        graph
            .nodes
            .push(make_node("orphan", NodeKind::Module, "Orphan"));
        graph.edges.push(make_edge("a", "b", EdgeKind::Calls));
        graph.edges.push(make_edge("b", "c", EdgeKind::Calls));
        graph.edges.push(make_edge("c", "a", EdgeKind::Calls));

        let report = detect_architectural_drift(&graph);

        assert!(report
            .violations
            .iter()
            .any(|v| { v.kind == ViolationKind::CircularDependency }));
        assert!(report
            .violations
            .iter()
            .any(|v| { v.kind == ViolationKind::OrphanComponent }));
        assert!(report.health_score <= 100);
        assert_eq!(report.total_modules, 4);
        assert!(!report.suggestions.is_empty());
    }
}
