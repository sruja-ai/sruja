//! Graph comparison: node/edge diffs, violations, and suggestions.

use crate::health::calculate_health_score_from_violations;
use crate::source_ref::{collect_edge_sources, collect_node_path_source};
use crate::types::HealthScorePenalties;
use crate::types::{
    DiffEdge, DiffNode, DiffResult, DiffSummary, EdgeDiff, NodeDiff, NodeMatch, Severity,
    Violation, ViolationKind,
};
use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};
use std::collections::HashSet;

pub fn compare_graphs(actual: &Graph, proposed: &Graph) -> DiffResult {
    let node_diff = compare_nodes(&actual.nodes, &proposed.nodes);
    let edge_diff = compare_edges(&actual.edges, &proposed.edges);
    let violations = detect_violations(actual, proposed, &node_diff, &edge_diff);
    let suggestions = generate_suggestions(&node_diff, &edge_diff, &violations);

    let summary = DiffSummary {
        proposed_components: proposed.nodes.len(),
        existing_components: actual.nodes.len(),
        new_components: node_diff.added.len(),
        missing_components: node_diff.removed.len(),
        new_dependencies: edge_diff.added.len(),
        removed_dependencies: edge_diff.removed.len(),
        health_score: calculate_health_score(&node_diff, &edge_diff, &violations),
    };

    DiffResult {
        proposal_title: "Architecture Comparison".to_string(),
        node_diff,
        edge_diff,
        violations,
        suggestions,
        summary,
    }
}

fn compare_nodes(actual: &[Node], proposed: &[Node]) -> NodeDiff {
    let actual_ids: HashSet<&str> = actual.iter().map(|n| n.id.as_str()).collect();
    let proposed_ids: HashSet<&str> = proposed.iter().map(|n| n.id.as_str()).collect();

    let added: Vec<DiffNode> = proposed
        .iter()
        .filter(|n| !actual_ids.contains(n.id.as_str()))
        .map(|n| DiffNode {
            id: n.id.clone(),
            kind: n.kind,
            label: n.label.clone(),
            technology: n.technology.clone(),
            description: None,
        })
        .collect();

    let removed: Vec<DiffNode> = actual
        .iter()
        .filter(|n| !proposed_ids.contains(n.id.as_str()))
        .map(|n| DiffNode {
            id: n.id.clone(),
            kind: n.kind,
            label: n.label.clone(),
            technology: n.technology.clone(),
            description: None,
        })
        .collect();

    let matched: Vec<NodeMatch> = proposed
        .iter()
        .filter(|n| actual_ids.contains(n.id.as_str()))
        .map(|pn| {
            let actual_node = actual.iter().find(|an| an.id == pn.id).unwrap();
            NodeMatch {
                proposal_id: pn.id.clone(),
                actual_id: actual_node.id.clone(),
                similarity: calculate_similarity(&pn.label, &actual_node.label),
                kind_match: pn.kind == actual_node.kind,
            }
        })
        .collect();

    NodeDiff {
        added,
        removed,
        matched,
    }
}

fn compare_edges(actual: &[Edge], proposed: &[Edge]) -> EdgeDiff {
    let actual_set: HashSet<(String, String, EdgeKind)> = actual
        .iter()
        .map(|e| (e.source.clone(), e.target.clone(), e.kind))
        .collect();

    let proposed_set: HashSet<(String, String, EdgeKind)> = proposed
        .iter()
        .map(|e| (e.source.clone(), e.target.clone(), e.kind))
        .collect();

    let added: Vec<DiffEdge> = proposed
        .iter()
        .filter(|e| !actual_set.contains(&(e.source.clone(), e.target.clone(), e.kind)))
        .map(|e| DiffEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: e.kind,
            label: None,
        })
        .collect();

    let removed: Vec<DiffEdge> = actual
        .iter()
        .filter(|e| !proposed_set.contains(&(e.source.clone(), e.target.clone(), e.kind)))
        .map(|e| DiffEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: e.kind,
            label: None,
        })
        .collect();

    EdgeDiff { added, removed }
}

fn detect_violations(
    actual: &Graph,
    proposed: &Graph,
    node_diff: &NodeDiff,
    edge_diff: &EdgeDiff,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for edge in &edge_diff.added {
        let source = proposed.nodes.iter().find(|n| n.id == edge.source);
        let target = proposed.nodes.iter().find(|n| n.id == edge.target);

        if let (Some(src), Some(tgt)) = (source, target) {
            if src.kind == NodeKind::Module && tgt.kind == NodeKind::Database {
                let sources = collect_edge_sources(actual, &edge.source, &edge.target);
                violations.push(Violation {
                    kind: ViolationKind::LayerViolation,
                    severity: Severity::Warning,
                    message: format!(
                        "Direct database access from '{}' - consider adding a service layer",
                        src.label
                    ),
                    location: Some(format!("{} -> {}", edge.source, edge.target)),
                    suggestion: Some(format!(
                        "Add a data access service between {} and {}",
                        src.label, tgt.label
                    )),
                    sources,
                });
            }
        }
    }

    for node in &node_diff.added {
        let has_incoming = proposed.edges.iter().any(|e| e.target == node.id);
        let has_outgoing = proposed.edges.iter().any(|e| e.source == node.id);

        if !has_incoming && !has_outgoing {
            let sources = collect_node_path_source(actual, &node.id);
            violations.push(Violation {
                kind: ViolationKind::OrphanComponent,
                severity: Severity::Warning,
                message: format!("Component '{}' has no connections", node.label),
                location: Some(node.id.clone()),
                suggestion: Some(format!(
                    "Define how '{}' interacts with other components",
                    node.label
                )),
                sources,
            });
        }
    }

    for node in &node_diff.added {
        if node.kind == NodeKind::Service && node.technology.is_none() {
            let sources = collect_node_path_source(actual, &node.id);
            violations.push(Violation {
                kind: ViolationKind::UndocumentedComponent,
                severity: Severity::Info,
                message: format!("Service '{}' has no technology specified", node.label),
                location: Some(node.id.clone()),
                suggestion: Some(format!(
                    "Specify the technology for '{}' (e.g., Node.js, Go, Python)",
                    node.label
                )),
                sources,
            });
        }
    }

    violations
}

fn generate_suggestions(
    node_diff: &NodeDiff,
    edge_diff: &EdgeDiff,
    violations: &[Violation],
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if !node_diff.added.is_empty() {
        let db_added: Vec<_> = node_diff
            .added
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .collect();
        if !db_added.is_empty() {
            suggestions.push(format!(
                "Consider data migration strategy for new database(s): {}",
                db_added
                    .iter()
                    .map(|n| n.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    for violation in violations {
        if let Some(ref sugg) = violation.suggestion {
            suggestions.push(sugg.clone());
        }
    }

    if !edge_diff.added.is_empty() {
        let external_edges: Vec<_> = edge_diff
            .added
            .iter()
            .filter(|e| e.kind == EdgeKind::Calls)
            .collect();
        if !external_edges.is_empty() {
            suggestions.push(
                "Add error handling and retry logic for new synchronous dependencies".to_string(),
            );
        }
    }

    suggestions.sort();
    suggestions.dedup();
    suggestions
}

fn calculate_similarity(a: &str, b: &str) -> f32 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    let a_words: HashSet<&str> = a_lower.split_whitespace().collect();
    let b_words: HashSet<&str> = b_lower.split_whitespace().collect();

    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }

    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();

    intersection as f32 / union as f32
}

fn calculate_health_score(
    node_diff: &NodeDiff,
    _edge_diff: &EdgeDiff,
    violations: &[Violation],
) -> u8 {
    let mut score =
        calculate_health_score_from_violations(violations, HealthScorePenalties::default());

    let orphan_penalty = node_diff
        .added
        .iter()
        .filter(|n| {
            violations.iter().any(|v| {
                v.kind == ViolationKind::OrphanComponent && v.location.as_deref() == Some(&n.id)
            })
        })
        .count();
    score = score.saturating_sub((orphan_penalty as u8).saturating_mul(3));

    score
}
