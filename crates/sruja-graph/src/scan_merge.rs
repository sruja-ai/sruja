//! Consolidated scan-to-graph merge logic
//!
//! This module provides functionality to merge architecture graphs from code scans
//! into the KnowledgeGraph. This eliminates duplication between CLI and Chat modules.

use crate::{
    ArchitectureEdge, ArchitectureNode, EdgeKind, KnowledgeGraph, NodeKind, SourceReference,
};
use chrono::Utc;
use sruja_scan::{EdgeKind as ScanEdgeKind, Graph, NodeKind as ScanNodeKind};

/// Merge a scanned architecture graph into a KnowledgeGraph.
///
/// This function converts nodes and edges from a code scan into architecture
/// nodes and edges, and merges them into the provided KnowledgeGraph.
///
/// # Arguments
///
/// * `graph` - The KnowledgeGraph to merge into
/// * `scan_graph` - The scanned architecture graph from sruja-scan
/// * `repo_path` - The repository path for source references
///
/// # Returns
///
/// The number of items (nodes + edges) merged into the graph
///
/// # Example
///
/// ```ignore
/// use sruja_graph::KnowledgeGraph;
/// use sruja_scan::scan_repo;
/// use std::path::Path;
///
/// let mut kg = KnowledgeGraph::new();
/// let scan_graph = scan_repo(Path::new("."))?;
/// let count = merge_scan_into_graph(&mut kg, &scan_graph, ".");
/// println!("Merged {} items", count);
/// ```
pub fn merge_scan_into_graph(
    graph: &mut KnowledgeGraph,
    scan_graph: &Graph,
    repo_path: &str,
) -> usize {
    let now = Utc::now();
    let source = SourceReference::scanned_repo(repo_path);
    let mut count = 0;

    // Merge nodes
    for node in &scan_graph.nodes {
        let kind = match node.kind {
            ScanNodeKind::Service => NodeKind::Service,
            ScanNodeKind::Module => NodeKind::Module,
            ScanNodeKind::Database => NodeKind::Database,
            ScanNodeKind::ExternalApi => NodeKind::ExternalApi,
        };

        let arch_node = ArchitectureNode {
            id: node.id.clone(),
            kind,
            label: node.label.clone(),
            technology: node.technology.clone(),
            description: node.path.clone(),
            metadata: node.metadata.clone(),
            source: source.clone(),
            created_at: now,
            updated_at: now,
        };

        graph.merge_node(arch_node);
        count += 1;
    }

    // Merge edges
    for edge in &scan_graph.edges {
        let kind = match edge.kind {
            ScanEdgeKind::Calls => EdgeKind::Calls,
            ScanEdgeKind::ReadsFrom => EdgeKind::ReadsFrom,
            ScanEdgeKind::WritesTo => EdgeKind::WritesTo,
        };

        let edge_id = format!("{}-{}-{:?}", edge.source, edge.target, edge.kind);
        let arch_edge = ArchitectureEdge {
            id: edge_id,
            source: edge.source.clone(),
            target: edge.target.clone(),
            kind,
            label: None,
            description: None,
            source_ref: source.clone(),
        };

        graph.merge_edge(arch_edge);
        count += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeEvidence, Node};
    use std::collections::HashMap;

    fn create_test_scan_graph() -> Graph {
        Graph {
            metadata: HashMap::new(),
            nodes: vec![
                Node {
                    id: "module.main".to_string(),
                    kind: ScanNodeKind::Module,
                    label: "main".to_string(),
                    path: Some("src/main.ts".to_string()),
                    technology: Some("TypeScript".to_string()),
                    metadata: HashMap::new(),
                },
                Node {
                    id: "module.utils".to_string(),
                    kind: ScanNodeKind::Module,
                    label: "utils".to_string(),
                    path: Some("src/utils.ts".to_string()),
                    technology: Some("TypeScript".to_string()),
                    metadata: HashMap::new(),
                },
            ],
            edges: vec![Edge {
                source: "module.main".to_string(),
                target: "module.utils".to_string(),
                kind: ScanEdgeKind::Calls,
                evidence: vec![EdgeEvidence {
                    rule: "import".to_string(),
                    file: Some("src/main.ts".to_string()),
                    line: Some(1),
                    detail: Some("import { utils } from './utils'".to_string()),
                }],
            }],
        }
    }

    #[test]
    fn test_merge_empty_graph() {
        let mut kg = KnowledgeGraph::new();
        let scan_graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![],
            edges: vec![],
        };

        let count = merge_scan_into_graph(&mut kg, &scan_graph, ".");
        assert_eq!(count, 0);
        assert_eq!(kg.nodes.len(), 0);
        assert_eq!(kg.edges.len(), 0);
    }

    #[test]
    fn test_merge_nodes_and_edges() {
        let mut kg = KnowledgeGraph::new();
        let scan_graph = create_test_scan_graph();

        let count = merge_scan_into_graph(&mut kg, &scan_graph, "/test/repo");

        // Should have merged 2 nodes + 1 edge = 3 items
        assert_eq!(count, 3);
        assert_eq!(kg.nodes.len(), 2);
        assert_eq!(kg.edges.len(), 1);

        // Verify node conversion
        let main_node = kg.nodes.get("module.main").expect("main node should exist");
        assert_eq!(main_node.kind, NodeKind::Module);
        assert_eq!(main_node.label, "main");
        assert_eq!(main_node.technology, Some("TypeScript".to_string()));
        assert_eq!(main_node.description, Some("src/main.ts".to_string()));

        // Verify edge conversion
        let edge = kg.edges.iter().next().expect("edge should exist");
        assert_eq!(edge.source, "module.main");
        assert_eq!(edge.target, "module.utils");
        assert_eq!(edge.kind, EdgeKind::Calls);
    }

    #[test]
    fn test_merge_is_idempotent() {
        let mut kg = KnowledgeGraph::new();
        let scan_graph = create_test_scan_graph();

        // Merge twice
        let count1 = merge_scan_into_graph(&mut kg, &scan_graph, "/test/repo");
        let count2 = merge_scan_into_graph(&mut kg, &scan_graph, "/test/repo");

        // Both should report the same count
        assert_eq!(count1, 3);
        assert_eq!(count2, 3);

        // But graph should still have same number of unique nodes/edges
        assert_eq!(kg.nodes.len(), 2);
        assert_eq!(kg.edges.len(), 1);
    }

    #[test]
    fn test_node_kind_conversion() {
        let mut kg = KnowledgeGraph::new();

        // Test all node kind conversions
        let test_cases = vec![
            (ScanNodeKind::Service, NodeKind::Service),
            (ScanNodeKind::Module, NodeKind::Module),
            (ScanNodeKind::Database, NodeKind::Database),
            (ScanNodeKind::ExternalApi, NodeKind::ExternalApi),
        ];

        for (i, (scan_kind, expected_kind)) in test_cases.into_iter().enumerate() {
            let scan_graph = Graph {
                metadata: HashMap::new(),
                nodes: vec![Node {
                    id: format!("test.{}", i),
                    kind: scan_kind,
                    label: format!("test{}", i),
                    path: Some(format!("test{}.ts", i)),
                    technology: None,
                    metadata: HashMap::new(),
                }],
                edges: vec![],
            };

            let count = merge_scan_into_graph(&mut kg, &scan_graph, ".");
            assert_eq!(count, 1);

            let node = kg
                .nodes
                .get(&format!("test.{}", i))
                .expect("node should exist");
            assert_eq!(node.kind, expected_kind);
        }
    }

    #[test]
    fn test_edge_kind_conversion() {
        let mut kg = KnowledgeGraph::new();

        // Test all edge kind conversions
        let test_cases = vec![
            (ScanEdgeKind::Calls, EdgeKind::Calls),
            (ScanEdgeKind::ReadsFrom, EdgeKind::ReadsFrom),
            (ScanEdgeKind::WritesTo, EdgeKind::WritesTo),
        ];

        for (i, (scan_kind, expected_kind)) in test_cases.into_iter().enumerate() {
            let scan_graph = Graph {
                metadata: HashMap::new(),
                nodes: vec![
                    Node {
                        id: format!("source.{}", i),
                        kind: ScanNodeKind::Module,
                        label: format!("source{}", i),
                        path: Some(format!("source{}.ts", i)),
                        technology: None,
                        metadata: HashMap::new(),
                    },
                    Node {
                        id: format!("target.{}", i),
                        kind: ScanNodeKind::Module,
                        label: format!("target{}", i),
                        path: Some(format!("target{}.ts", i)),
                        technology: None,
                        metadata: HashMap::new(),
                    },
                ],
                edges: vec![Edge {
                    source: format!("source.{}", i),
                    target: format!("target.{}", i),
                    kind: scan_kind,
                    evidence: vec![],
                }],
            };

            let count = merge_scan_into_graph(&mut kg, &scan_graph, ".");
            assert_eq!(count, 3); // 2 nodes + 1 edge

            let edge = kg
                .edges
                .iter()
                .find(|e| e.source == format!("source.{}", i))
                .expect("edge should exist");
            assert_eq!(edge.kind, expected_kind);
        }
    }

    #[test]
    fn test_source_reference() {
        let mut kg = KnowledgeGraph::new();
        let scan_graph = create_test_scan_graph();

        merge_scan_into_graph(&mut kg, &scan_graph, "/my/repo/path");

        // All nodes should have the correct source reference
        for node in kg.nodes.values() {
            match &node.source {
                SourceReference::ScannedRepo { path } => {
                    assert!(path.contains("/my/repo/path"));
                }
                _ => panic!("Expected ScannedRepo source reference"),
            }
        }

        // All edges should have the correct source reference
        for edge in kg.edges.iter() {
            match &edge.source_ref {
                SourceReference::ScannedRepo { path } => {
                    assert!(path.contains("/my/repo/path"));
                }
                _ => panic!("Expected ScannedRepo source reference"),
            }
        }
    }
}
