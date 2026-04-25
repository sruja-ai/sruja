//! Helpers to collect source references from graph evidence.

use crate::types::SourceRef;
use sruja_scan::{EdgeEvidence, Graph};
use std::collections::HashSet;

pub(super) fn edge_evidence_to_source_refs(evidence: &[EdgeEvidence]) -> Vec<SourceRef> {
    let mut refs = Vec::new();
    for ev in evidence {
        if ev.file.is_some() || ev.detail.is_some() {
            refs.push(SourceRef {
                file: ev.file.clone(),
                line: ev.line,
                detail: ev.detail.clone(),
            });
        }
    }
    refs
}

/// Collect unique source refs for edges that form a cycle (consecutive pairs in cycle).
pub(super) fn collect_cycle_sources(graph: &Graph, cycle: &[String]) -> Vec<SourceRef> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for w in cycle.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        for edge in &graph.edges {
            if edge.source == *a && edge.target == *b {
                for ev in &edge.evidence {
                    let key = (ev.file.clone(), ev.line);
                    if seen.insert(key) {
                        out.push(SourceRef {
                            file: ev.file.clone(),
                            line: ev.line,
                            detail: ev.detail.clone(),
                        });
                    }
                }
                break;
            }
        }
    }
    if cycle.len() > 1 {
        let (last, first) = (cycle.last().unwrap(), cycle.first().unwrap());
        for edge in &graph.edges {
            if edge.source == *last && edge.target == *first {
                for ev in &edge.evidence {
                    let key = (ev.file.clone(), ev.line);
                    if seen.insert(key) {
                        out.push(SourceRef {
                            file: ev.file.clone(),
                            line: ev.line,
                            detail: ev.detail.clone(),
                        });
                    }
                }
                break;
            }
        }
    }
    out
}

pub(super) fn collect_edge_sources(graph: &Graph, source: &str, target: &str) -> Vec<SourceRef> {
    for edge in &graph.edges {
        if edge.source == source && edge.target == target {
            return edge_evidence_to_source_refs(&edge.evidence);
        }
    }
    Vec::new()
}

pub(super) fn collect_node_path_source(graph: &Graph, node_id: &str) -> Vec<SourceRef> {
    for node in &graph.nodes {
        if node.id == node_id {
            if let Some(ref path) = node.path {
                return vec![SourceRef {
                    file: Some(path.clone()),
                    line: None,
                    detail: None,
                }];
            }
            return Vec::new();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
    use std::collections::HashMap;

    fn node(id: &str, path: Option<&str>) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            kind: NodeKind::Module,
            path: path.map(|p| p.to_string()),
            ..Node::default()
        }
    }

    fn edge(source: &str, target: &str, evidence: Vec<EdgeEvidence>) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind: EdgeKind::Calls,
            evidence,
        }
    }

    #[test]
    fn edge_evidence_to_source_refs_filters_entries_with_no_file_and_no_detail() {
        let refs = edge_evidence_to_source_refs(&[
            EdgeEvidence {
                rule: "r".to_string(),
                file: None,
                line: Some(10),
                detail: None,
            },
            EdgeEvidence {
                rule: "r".to_string(),
                file: Some("a.ts".to_string()),
                line: Some(1),
                detail: None,
            },
            EdgeEvidence {
                rule: "r".to_string(),
                file: None,
                line: None,
                detail: Some("something".to_string()),
            },
        ]);

        assert_eq!(refs.len(), 2);
        assert_eq!(
            refs[0],
            SourceRef {
                file: Some("a.ts".to_string()),
                line: Some(1),
                detail: None,
            }
        );
        assert_eq!(
            refs[1],
            SourceRef {
                file: None,
                line: None,
                detail: Some("something".to_string()),
            }
        );
    }

    #[test]
    fn collect_edge_sources_returns_source_refs_for_matching_edge() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("A", None), node("B", None)],
            edges: vec![edge(
                "A",
                "B",
                vec![EdgeEvidence {
                    rule: "rule".to_string(),
                    file: Some("src/a.ts".to_string()),
                    line: Some(42),
                    detail: Some("import".to_string()),
                }],
            )],
            incidents: Vec::new(),
            confidence: None,
        };

        let refs = collect_edge_sources(&graph, "A", "B");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].file.as_deref(), Some("src/a.ts"));
        assert_eq!(refs[0].line, Some(42));
        assert_eq!(refs[0].detail.as_deref(), Some("import"));
    }

    #[test]
    fn collect_edge_sources_returns_empty_when_edge_missing() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("A", None), node("B", None)],
            edges: vec![],
            incidents: Vec::new(),
            confidence: None,
        };

        let refs = collect_edge_sources(&graph, "A", "B");
        assert!(refs.is_empty());
    }

    #[test]
    fn collect_node_path_source_returns_path_as_source_ref() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("A", Some("src/a.ts"))],
            edges: vec![],
            incidents: Vec::new(),
            confidence: None,
        };

        let refs = collect_node_path_source(&graph, "A");
        assert_eq!(
            refs,
            vec![SourceRef {
                file: Some("src/a.ts".to_string()),
                line: None,
                detail: None,
            }]
        );
    }

    #[test]
    fn collect_cycle_sources_includes_last_to_first_edge_and_dedups_by_file_and_line() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("A", None), node("B", None), node("C", None)],
            edges: vec![
                edge(
                    "A",
                    "B",
                    vec![EdgeEvidence {
                        rule: "r".to_string(),
                        file: Some("src/shared.ts".to_string()),
                        line: Some(1),
                        detail: Some("a->b".to_string()),
                    }],
                ),
                edge(
                    "B",
                    "C",
                    vec![EdgeEvidence {
                        rule: "r".to_string(),
                        file: Some("src/shared.ts".to_string()),
                        line: Some(1),
                        detail: Some("b->c".to_string()),
                    }],
                ),
                edge(
                    "C",
                    "A",
                    vec![EdgeEvidence {
                        rule: "r".to_string(),
                        file: Some("src/c.ts".to_string()),
                        line: Some(3),
                        detail: Some("c->a".to_string()),
                    }],
                ),
            ],
            incidents: Vec::new(),
            confidence: None,
        };

        let cycle = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let refs = collect_cycle_sources(&graph, &cycle);
        assert_eq!(refs.len(), 2);
        assert!(refs
            .iter()
            .any(|r| r.file.as_deref() == Some("src/shared.ts") && r.line == Some(1)));
        assert!(refs
            .iter()
            .any(|r| r.file.as_deref() == Some("src/c.ts") && r.line == Some(3)));
    }
}
