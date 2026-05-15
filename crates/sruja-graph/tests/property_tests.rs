//! Property-based tests for graph operations using proptest

use proptest::prelude::*;
use sruja_graph::*;

fn arb_node_kind() -> impl Strategy<Value = NodeKind> {
    prop_oneof![
        Just(NodeKind::new(NodeKind::SYSTEM)),
        Just(NodeKind::new(NodeKind::CONTAINER)),
        Just(NodeKind::new(NodeKind::COMPONENT)),
        Just(NodeKind::new(NodeKind::DATABASE)),
        Just(NodeKind::new(NodeKind::QUEUE)),
        Just(NodeKind::new(NodeKind::SERVICE)),
        Just(NodeKind::new(NodeKind::FRONTEND)),
        Just(NodeKind::new(NodeKind::MODULE)),
    ]
}

fn arb_node_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

fn make_node(id: &str, kind: NodeKind) -> ArchitectureNode {
    ArchitectureNode {
        id: id.to_string(),
        kind,
        label: id.to_string(),
        ..Default::default()
    }
}

fn make_edge(id: &str, source: &str, target: &str, kind: EdgeKind) -> ArchitectureEdge {
    ArchitectureEdge {
        id: id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        kind,
        label: None,
        description: None,
        source_ref: SourceReference::manual(),
    }
}

proptest! {
    #[test]
    fn prop_add_node_then_get_node(id in arb_node_id()) {
        let mut graph = KnowledgeGraph::new();
        let node = make_node(&id, NodeKind::new(NodeKind::SERVICE));

        graph.add_node(node).unwrap();
        let retrieved = graph.get_node(&id);

        prop_assert!(retrieved.is_some());
        prop_assert_eq!(&retrieved.unwrap().id, &id);
    }

    #[test]
    fn prop_duplicate_node_fails(id in arb_node_id()) {
        let mut graph = KnowledgeGraph::new();
        let node = make_node(&id, NodeKind::new(NodeKind::SERVICE));

        graph.add_node(node.clone()).unwrap();
        let result = graph.add_node(node);

        prop_assert!(result.is_err());
    }

    #[test]
    fn prop_add_remove_node_roundtrip(id in arb_node_id()) {
        let mut graph = KnowledgeGraph::new();
        let node = make_node(&id, NodeKind::new(NodeKind::SERVICE));

        graph.add_node(node).unwrap();
        let removed = graph.remove_node(&id);

        prop_assert!(removed.is_ok());
        prop_assert!(graph.get_node(&id).is_none());
    }

    #[test]
    fn prop_merge_node_idempotent(id in arb_node_id()) {
        let mut graph = KnowledgeGraph::new();
        let node = make_node(&id, NodeKind::new(NodeKind::SERVICE));

        graph.merge_node(node.clone());
        graph.merge_node(node);

        prop_assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn prop_edge_requires_both_nodes(source_id in arb_node_id(), target_id in arb_node_id()) {
        prop_assume!(source_id != target_id);

        let mut graph = KnowledgeGraph::new();

        graph.add_node(make_node(&source_id, NodeKind::new(NodeKind::SERVICE))).unwrap();

        let edge = make_edge("e1", &source_id, &target_id, EdgeKind::new(EdgeKind::CALLS));

        let result = graph.add_edge(edge);
        prop_assert!(result.is_err());
    }

    #[test]
    fn prop_json_roundtrip(num_nodes in 0usize..10) {
        let mut graph = KnowledgeGraph::with_name("TestGraph");

        for i in 0..num_nodes {
            let id = format!("node_{}", i);
            graph.add_node(make_node(&id, NodeKind::new(NodeKind::SERVICE))).unwrap();
        }

        let json = graph.to_json().unwrap();
        let restored = KnowledgeGraph::from_json(&json).unwrap();

        prop_assert_eq!(restored.nodes.len(), num_nodes);
        prop_assert_eq!(&restored.metadata.name, "TestGraph");
    }

    #[test]
    fn prop_stats_consistency(num_nodes in 0usize..10, num_edges in 0usize..10) {
        let mut graph = KnowledgeGraph::new();

        let actual_nodes = num_nodes.min(num_edges + 1);

        for i in 0..actual_nodes {
            let id = format!("n{}", i);
            graph.add_node(make_node(&id, NodeKind::new(NodeKind::SERVICE))).unwrap();
        }

        let mut edges_added = 0;
        for i in 0..num_edges {
            if i + 1 < actual_nodes {
                let edge = make_edge(
                    &format!("e{}", i),
                    &format!("n{}", i),
                    &format!("n{}", i + 1),
                    EdgeKind::new(EdgeKind::CALLS),
                );
                if graph.add_edge(edge).is_ok() {
                    edges_added += 1;
                }
            }
        }

        let stats = graph.stats();
        prop_assert_eq!(stats.total_nodes, actual_nodes);
        prop_assert_eq!(stats.total_edges, edges_added);
    }

    #[test]
    fn prop_remove_node_removes_incident_edges(
        a_id in arb_node_id(),
        b_id in arb_node_id(),
        c_id in arb_node_id()
    ) {
        prop_assume!(a_id != b_id);
        prop_assume!(b_id != c_id);
        prop_assume!(a_id != c_id);

        let mut graph = KnowledgeGraph::new();

        graph.add_node(make_node(&a_id, NodeKind::new(NodeKind::SERVICE))).unwrap();
        graph.add_node(make_node(&b_id, NodeKind::new(NodeKind::SERVICE))).unwrap();
        graph.add_node(make_node(&c_id, NodeKind::new(NodeKind::SERVICE))).unwrap();

        graph.add_edge(make_edge("e1", &a_id, &b_id, EdgeKind::new(EdgeKind::CALLS))).unwrap();
        graph.add_edge(make_edge("e2", &b_id, &c_id, EdgeKind::new(EdgeKind::CALLS))).unwrap();

        prop_assert_eq!(graph.edges.len(), 2);

        graph.remove_node(&b_id).unwrap();

        prop_assert!(graph.edges.is_empty());
    }

    #[test]
    fn prop_find_nodes_by_kind(kind in arb_node_kind()) {
        let mut graph = KnowledgeGraph::new();
        let id = format!("node_{:?}", kind);

        graph.add_node(make_node(&id, kind.clone())).unwrap();
        let found = graph.find_nodes_by_kind(kind);

        prop_assert_eq!(found.len(), 1);
        prop_assert_eq!(&found[0].id, &id);
    }
}
