//! Consolidated scan-to-graph merge logic
//!
//! This module provides functionality to merge architecture graphs from code scans
//! into the KnowledgeGraph. This eliminates duplication between CLI and Chat modules.

use crate::{ArchitectureEdge, ArchitectureNode, KnowledgeGraph, SourceReference};
use chrono::Utc;
use sruja_scan::Graph;

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
        let arch_node = ArchitectureNode {
            id: node.id.clone(),
            kind: node.kind.clone(),
            label: node.label.clone(),
            technology: node.technology.clone(),
            description: node.path.clone(),
            metadata: node.metadata.clone(),
            source: source.clone(),
            created_at: now,
            updated_at: now,
            gotchas: node.gotchas.clone(),
            operational_constraints: node.operational_constraints.clone(),
            runbooks: node.runbooks.clone(),
        };

        graph.merge_node(arch_node);
        count += 1;
    }

    // Merge incidents
    for inc in &scan_graph.incidents {
        let incident = crate::Incident {
            id: inc.id.clone(),
            title: inc.title.clone(),
            date: inc.date.clone(),
            severity: inc.severity.clone(),
            affected: inc.affected.clone(),
            cause: inc.cause.clone(),
            resolution: inc.resolution.clone(),
            lesson: inc.lesson.clone(),
            source: source.clone(),
        };
        graph.add_incident(incident).ok();
        count += 1;
    }

    // Merge edges
    for edge in &scan_graph.edges {
        let edge_id = format!("{}-{}-{:?}", edge.source, edge.target, edge.kind);
        let arch_edge = ArchitectureEdge {
            id: edge_id,
            source: edge.source.clone(),
            target: edge.target.clone(),
            kind: edge.kind.clone(),
            label: None,
            description: None,
            source_ref: source.clone(),
        };

        graph.merge_edge(arch_edge);
        count += 1;
    }

    count
}

/// Merge a parsed DSL program into a KnowledgeGraph.
pub fn merge_program_into_graph(
    graph: &mut KnowledgeGraph,
    program: &sruja_language::ast::Program,
    source_file: &str,
) -> usize {
    let now = Utc::now();
    let source = SourceReference::dsl_file(source_file, 1);
    let mut count = 0;

    // Collect elements from program
    let (elements, _relations) = sruja_language::collect_elements(program);

    // Merge elements as nodes
    for (fqn, elem) in elements {
        let node_kind = match elem.assignment.kind {
            sruja_language::ast::ElementKind::Person => sruja_scan::NodeKind::Module,
            sruja_language::ast::ElementKind::System => sruja_scan::NodeKind::System,
            sruja_language::ast::ElementKind::Container => sruja_scan::NodeKind::Container,
            sruja_language::ast::ElementKind::Component => sruja_scan::NodeKind::Component,
            sruja_language::ast::ElementKind::Database => sruja_scan::NodeKind::Database,
            sruja_language::ast::ElementKind::Queue => sruja_scan::NodeKind::Queue,
            sruja_language::ast::ElementKind::Requirement => sruja_scan::NodeKind::Module,
            sruja_language::ast::ElementKind::Custom(ref s) => {
                sruja_scan::NodeKind::Custom(s.clone())
            }
            _ => sruja_scan::NodeKind::Module,
        };

        let arch_node = ArchitectureNode {
            id: fqn,
            kind: node_kind,
            label: elem.assignment.title.unwrap_or(elem.assignment.name),
            technology: elem
                .assignment
                .body
                .as_ref()
                .and_then(|b| b.technology.clone()),
            description: elem
                .assignment
                .body
                .as_ref()
                .and_then(|b| b.description.clone()),
            metadata: std::collections::HashMap::new(),
            source: source.clone(),
            created_at: now,
            updated_at: now,
            gotchas: elem
                .assignment
                .body
                .as_ref()
                .map(|b| b.gotchas.clone())
                .unwrap_or_default(),
            operational_constraints: elem
                .assignment
                .body
                .as_ref()
                .map(|b| b.operational_constraints.clone())
                .unwrap_or_default(),
            runbooks: elem
                .assignment
                .body
                .as_ref()
                .map(|b| b.runbooks.clone())
                .unwrap_or_default(),
        };
        graph.merge_node(arch_node);
        count += 1;
    }

    // Collect ADRs
    for item in &program.items {
        if let sruja_language::ast::TopLevelItem::Adr(adr) = item {
            let status = match adr.status.as_deref().unwrap_or("proposed") {
                "accepted" | "Accepted" => crate::DecisionStatus::Accepted,
                "rejected" | "Rejected" => crate::DecisionStatus::Rejected,
                "superseded" | "Superseded" => crate::DecisionStatus::Superseded,
                _ => crate::DecisionStatus::Proposed,
            };

            let decision = crate::Decision {
                id: adr.id.clone(),
                title: adr.title.clone(),
                status,
                context: adr.context.clone().unwrap_or_default(),
                decision: adr.decision.clone().unwrap_or_default(),
                consequences: adr.consequences.clone().unwrap_or_default(),
                alternatives: vec![],
                created_at: now,
                updated_at: now,
                ratified_at: if status == crate::DecisionStatus::Accepted {
                    Some(now)
                } else {
                    None
                },
                author: None,
                source: source.clone(),
                affects: adr.affects.clone(),
            };

            graph.add_decision(decision).ok();
            count += 1;
        } else if let sruja_language::ast::TopLevelItem::Incident(inc) = item {
            let incident = crate::Incident {
                id: inc.id.clone(),
                title: inc.title.clone(),
                date: inc.date.clone(),
                severity: inc.severity.clone(),
                affected: inc.affected.iter().map(|id| id.as_string()).collect(),
                cause: inc.cause.clone(),
                resolution: inc.resolution.clone(),
                lesson: inc.lesson.clone(),
                source: source.clone(),
            };
            graph.add_incident(incident).ok();
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeKind, NodeKind};
    use sruja_scan::{Edge, EdgeEvidence, Graph, Node};
    use std::collections::HashMap;

    fn create_test_scan_graph() -> Graph {
        let mut graph = Graph::default();
        graph.nodes = vec![
            {
                let mut node = Node::default();
                node.id = "module.main".to_string();
                node.kind = NodeKind::Module;
                node.label = "main".to_string();
                node.path = Some("src/main.ts".to_string());
                node.technology = Some("TypeScript".to_string());
                node
            },
            {
                let mut node = Node::default();
                node.id = "module.utils".to_string();
                node.kind = NodeKind::Module;
                node.label = "utils".to_string();
                node.path = Some("src/utils.ts".to_string());
                node.technology = Some("TypeScript".to_string());
                node
            },
        ];
        graph.edges = vec![Edge {
            source: "module.main".to_string(),
            target: "module.utils".to_string(),
            kind: EdgeKind::Calls,
            evidence: vec![EdgeEvidence {
                rule: "import".to_string(),
                file: Some("src/main.ts".to_string()),
                line: Some(1),
                detail: Some("import { utils } from './utils'".to_string()),
            }],
        }];
        graph
    }

    #[test]
    fn test_merge_empty_graph() {
        let mut kg = KnowledgeGraph::new();
        let scan_graph = Graph::default();

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
        let edge = kg.edges.first().expect("edge should exist");
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

    fn test_node(id: &str, kind: NodeKind) -> Node {
        let mut node = Node::default();
        node.id = id.to_string();
        node.kind = kind;
        node.label = id.to_string();
        node.path = Some(format!("{}.ts", id.trim_start_matches("module.")));
        node.technology = Some("TypeScript".to_string());
        node
    }

    fn test_edge(source: &str, target: &str, kind: EdgeKind) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind,
            evidence: vec![],
        }
    }

    #[test]
    fn test_node_kind_conversion() {
        let mut kg = KnowledgeGraph::new();

        // Test all node kind conversions
        let test_cases = vec![
            NodeKind::Service,
            NodeKind::Module,
            NodeKind::Database,
            NodeKind::ExternalApi,
        ];

        for (i, kind) in test_cases.into_iter().enumerate() {
            let scan_graph = Graph {
                metadata: HashMap::new(),
                nodes: vec![test_node(&format!("test.{}", i), kind.clone())],
                edges: vec![],
                incidents: vec![],
                confidence: None,
            };

            let count = merge_scan_into_graph(&mut kg, &scan_graph, ".");
            assert_eq!(count, 1);

            let node = kg
                .nodes
                .get(&format!("test.{}", i))
                .expect("node should exist");
            assert_eq!(node.kind, kind);
        }
    }

    #[test]
    fn test_edge_kind_conversion() {
        let mut kg = KnowledgeGraph::new();

        let test_cases = vec![EdgeKind::Calls, EdgeKind::ReadsFrom, EdgeKind::WritesTo];

        for (i, kind) in test_cases.into_iter().enumerate() {
            let mut scan_graph = Graph::default();
            scan_graph.nodes = vec![
                {
                    let mut node = Node::default();
                    node.id = format!("source.{}", i);
                    node.kind = NodeKind::Module;
                    node.label = format!("source{}", i);
                    node.path = Some(format!("source{}.ts", i));
                    node
                },
                {
                    let mut node = Node::default();
                    node.id = format!("target.{}", i);
                    node.kind = NodeKind::Module;
                    node.label = format!("target{}", i);
                    node.path = Some(format!("target{}.ts", i));
                    node
                },
            ];
            scan_graph.edges = vec![Edge {
                source: format!("source.{}", i),
                target: format!("target.{}", i),
                kind: kind.clone(),
                evidence: vec![],
            }];

            let count = merge_scan_into_graph(&mut kg, &scan_graph, ".");
            assert_eq!(count, 3); // 2 nodes + 1 edge

            let edge = kg
                .edges
                .iter()
                .find(|e| e.source == format!("source.{}", i))
                .expect("edge should exist");
            assert_eq!(edge.kind, kind);
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
