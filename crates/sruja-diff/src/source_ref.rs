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
