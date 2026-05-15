//! Graph comparison: node/edge diffs, violations, and suggestions.

use crate::health::calculate_health_score_from_violations;
use crate::source_ref::{collect_edge_sources, collect_node_path_source};
use crate::types::HealthScorePenalties;
use crate::types::{
    BaselineMode, DiffEdge, DiffNode, DiffResult, DiffSummary, EdgeDiff, NodeDiff, NodeMatch,
    Severity, TruthStatus, Violation, ViolationKind,
};
use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
pub struct CompareOptions {
    pub baseline_mode: BaselineMode,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            baseline_mode: BaselineMode::Auto,
        }
    }
}

pub fn compare_graphs(actual: &Graph, proposed: &Graph) -> DiffResult {
    compare_graphs_with_options(actual, proposed, CompareOptions::default())
}

pub fn compare_graphs_with_options(
    actual: &Graph,
    proposed: &Graph,
    opts: CompareOptions,
) -> DiffResult {
    let node_diff = compare_nodes(&actual.nodes, &proposed.nodes);
    let edge_diff = compare_edges(&actual.edges, &proposed.edges);
    let violations = detect_violations(actual, proposed, &node_diff, &edge_diff);
    let suggestions = generate_suggestions(&node_diff, &edge_diff, &violations);

    let mode = match opts.baseline_mode {
        BaselineMode::Auto => {
            let proposed_n = proposed.nodes.len();
            let actual_n = actual.nodes.len();
            let overview = proposed_n > 0
                && actual_n >= proposed_n.saturating_mul(10)
                && actual_n.saturating_sub(proposed_n) >= 500;
            if overview {
                BaselineMode::Overview
            } else {
                BaselineMode::Exhaustive
            }
        }
        m => m,
    };
    let coverage = if actual.nodes.is_empty() {
        None
    } else {
        Some(((proposed.nodes.len() as f64 / actual.nodes.len() as f64) * 100.0).clamp(0.0, 100.0))
    };

    let summary = DiffSummary {
        proposed_components: proposed.nodes.len(),
        existing_components: actual.nodes.len(),
        new_components: node_diff.added.len(),
        missing_components: node_diff.removed.len(),
        new_dependencies: edge_diff.added.len(),
        removed_dependencies: edge_diff.removed.len(),
        health_score: calculate_health_score(&node_diff, &edge_diff, &violations),
        baseline_mode: Some(mode),
        baseline_coverage_percent: coverage,
    };

    let truth_status = if violations.is_empty() {
        TruthStatus::Reviewed
    } else {
        TruthStatus::Drifted
    };

    DiffResult {
        proposal_title: "Architecture Comparison".to_string(),
        node_diff,
        edge_diff,
        violations,
        suggestions,
        summary,
        truth_status,
    }
}

fn compare_nodes(actual: &[Node], proposed: &[Node]) -> NodeDiff {
    let mut actual_by_id: HashMap<&str, &Node> = HashMap::new();
    let mut actual_by_canonical: HashMap<&str, &Node> = HashMap::new();
    let mut actual_by_alias: HashMap<&str, &Node> = HashMap::new();
    let mut alias_dupes: HashSet<&str> = HashSet::new();
    let mut actual_by_source: HashMap<String, &Node> = HashMap::new();
    let mut source_dupes: HashSet<String> = HashSet::new();

    for n in actual {
        actual_by_id.insert(n.id.as_str(), n);
        if let Some(ref cid) = n.canonical_id {
            actual_by_canonical.insert(cid.as_str(), n);
        }
    }
    for n in actual {
        for a in &n.aliases {
            let key = a.as_str();
            if actual_by_alias.contains_key(key) {
                alias_dupes.insert(key);
            } else {
                actual_by_alias.insert(key, n);
            }
        }
        for s in &n.sources {
            let key = format!(
                "{}:{}",
                s.kind.as_str().to_lowercase(),
                s.path.to_lowercase()
            );
            if actual_by_source.contains_key(&key) {
                source_dupes.insert(key.clone());
            } else {
                actual_by_source.insert(key.clone(), n);
            }
        }
    }

    let mut used_actual: HashSet<String> = HashSet::new();
    let mut matches: Vec<(String, String)> = Vec::new();

    for pn in proposed {
        let mut matched_id: Option<String> = None;
        if let Some(ref cid) = pn.canonical_id {
            if let Some(an) = actual_by_canonical.get(cid.as_str()) {
                if !used_actual.contains(&an.id) {
                    matched_id = Some(an.id.clone());
                }
            }
        }
        if matched_id.is_none() {
            for alias in &pn.aliases {
                let key = alias.as_str();
                if !alias_dupes.contains(key) {
                    if let Some(an) = actual_by_alias.get(key) {
                        if !used_actual.contains(&an.id) {
                            matched_id = Some(an.id.clone());
                            break;
                        }
                    }
                }
            }
        }
        if matched_id.is_none() {
            for s in &pn.sources {
                let key = format!(
                    "{}:{}",
                    s.kind.as_str().to_lowercase(),
                    s.path.to_lowercase()
                );
                if !source_dupes.contains(&key) {
                    if let Some(an) = actual_by_source.get(&key) {
                        if !used_actual.contains(&an.id) {
                            matched_id = Some(an.id.clone());
                            break;
                        }
                    }
                }
            }
        }
        if matched_id.is_none() {
            if let Some(an) = actual_by_id.get(pn.id.as_str()) {
                if !used_actual.contains(&an.id) {
                    matched_id = Some(an.id.clone());
                }
            }
        }

        if let Some(aid) = matched_id {
            used_actual.insert(aid.clone());
            matches.push((pn.id.clone(), aid));
        }
    }

    let matched: Vec<NodeMatch> = matches
        .iter()
        .filter_map(|(pid, aid)| {
            let pn = proposed.iter().find(|n| n.id == *pid)?;
            let an = actual.iter().find(|n| n.id == *aid)?;
            Some(NodeMatch {
                proposal_id: pid.clone(),
                actual_id: aid.clone(),
                similarity: calculate_similarity(&pn.label, &an.label),
                kind_match: pn.kind == an.kind,
            })
        })
        .collect();

    let matched_proposed_ids: HashSet<&str> = matches.iter().map(|(p, _)| p.as_str()).collect();
    let matched_actual_ids: HashSet<&str> = matches.iter().map(|(_, a)| a.as_str()).collect();

    let added: Vec<DiffNode> = proposed
        .iter()
        .filter(|n| !matched_proposed_ids.contains(n.id.as_str()))
        .map(|n| DiffNode {
            id: n.id.clone(),
            kind: n.kind.clone(),
            label: n.label.clone(),
            technology: n.technology.clone(),
            description: None,
        })
        .collect();

    let removed: Vec<DiffNode> = actual
        .iter()
        .filter(|n| !matched_actual_ids.contains(n.id.as_str()))
        .map(|n| DiffNode {
            id: n.id.clone(),
            kind: n.kind.clone(),
            label: n.label.clone(),
            technology: n.technology.clone(),
            description: None,
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
        .map(|e| (e.source.clone(), e.target.clone(), e.kind.clone()))
        .collect();

    let proposed_set: HashSet<(String, String, EdgeKind)> = proposed
        .iter()
        .map(|e| (e.source.clone(), e.target.clone(), e.kind.clone()))
        .collect();

    let added: Vec<DiffEdge> = proposed
        .iter()
        .filter(|e| !actual_set.contains(&(e.source.clone(), e.target.clone(), e.kind.clone())))
        .map(|e| DiffEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: e.kind.clone(),
            label: None,
        })
        .collect();

    let removed: Vec<DiffEdge> = actual
        .iter()
        .filter(|e| !proposed_set.contains(&(e.source.clone(), e.target.clone(), e.kind.clone())))
        .map(|e| DiffEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: e.kind.clone(),
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
            if src.kind == NodeKind::MODULE && tgt.kind == NodeKind::DATABASE {
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
                    sources: sources.clone(),
                    confidence: None,
                    evidence_count: Some(sources.len()),
                    production_relevant: None,
                    baseline_delta: None,
                    suppressed: None,
                    rule_id: None,
                    rationale: None,
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
                sources: sources.clone(),
                confidence: None,
                evidence_count: Some(sources.len()),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            });
        }
    }

    for node in &node_diff.added {
        if node.kind == NodeKind::SERVICE && node.technology.is_none() {
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
                sources: sources.clone(),
                confidence: None,
                evidence_count: Some(sources.len()),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
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
            .filter(|n| n.kind == NodeKind::DATABASE)
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
            .filter(|e| e.kind == EdgeKind::CALLS)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};

    fn node(id: &str, label: &str, kind: NodeKind) -> Node {
        Node {
            id: id.to_string(),
            label: label.to_string(),
            kind,
            ..Node::default()
        }
    }

    fn edge(source: &str, target: &str, kind: EdgeKind) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind,
            evidence: vec![],
            confidence: Default::default(),
        }
    }

    #[test]
    fn compare_empty_graphs() {
        let actual = Graph::default();
        let proposed = Graph::default();
        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.added.len(), 0);
        assert_eq!(result.node_diff.removed.len(), 0);
        assert_eq!(result.node_diff.matched.len(), 0);
        assert_eq!(result.edge_diff.added.len(), 0);
        assert_eq!(result.edge_diff.removed.len(), 0);
        assert_eq!(result.summary.proposed_components, 0);
        assert_eq!(result.summary.existing_components, 0);
    }

    #[test]
    fn compare_added_node() {
        let actual = Graph {
            nodes: vec![node("a", "A", NodeKind::new(NodeKind::MODULE))],
            edges: vec![],
            ..Default::default()
        };
        let proposed = Graph {
            nodes: vec![
                node("a", "A", NodeKind::new(NodeKind::MODULE)),
                node("b", "B", NodeKind::new(NodeKind::SERVICE)),
            ],
            edges: vec![],
            ..Default::default()
        };
        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.added.len(), 1);
        assert_eq!(result.node_diff.added[0].id, "b");
        assert_eq!(result.node_diff.removed.len(), 0);
        assert_eq!(result.node_diff.matched.len(), 1);
        assert_eq!(result.summary.new_components, 1);
    }

    #[test]
    fn compare_removed_node() {
        let actual = Graph {
            nodes: vec![
                node("a", "A", NodeKind::new(NodeKind::MODULE)),
                node("b", "B", NodeKind::new(NodeKind::MODULE)),
            ],
            edges: vec![],
            ..Default::default()
        };
        let proposed = Graph {
            nodes: vec![node("a", "A", NodeKind::new(NodeKind::MODULE))],
            edges: vec![],
            ..Default::default()
        };
        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.added.len(), 0);
        assert_eq!(result.node_diff.removed.len(), 1);
        assert_eq!(result.node_diff.removed[0].id, "b");
        assert_eq!(result.summary.missing_components, 1);
    }

    #[test]
    fn compare_added_and_removed_edges() {
        let actual = Graph {
            nodes: vec![
                node("a", "A", NodeKind::new(NodeKind::MODULE)),
                node("b", "B", NodeKind::new(NodeKind::MODULE)),
            ],
            edges: vec![edge("a", "b", EdgeKind::new(EdgeKind::CALLS))],
            ..Default::default()
        };
        let proposed = Graph {
            nodes: vec![
                node("a", "A", NodeKind::new(NodeKind::MODULE)),
                node("b", "B", NodeKind::new(NodeKind::MODULE)),
            ],
            edges: vec![
                edge("a", "b", EdgeKind::new(EdgeKind::CALLS)),
                edge("b", "a", EdgeKind::new(EdgeKind::CALLS)),
            ],
            ..Default::default()
        };
        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.edge_diff.added.len(), 1);
        assert_eq!(result.edge_diff.added[0].source, "b");
        assert_eq!(result.edge_diff.added[0].target, "a");
        assert_eq!(result.edge_diff.removed.len(), 0);
        assert_eq!(result.summary.new_dependencies, 1);
    }

    #[test]
    fn compare_identical_graphs_matched_only() {
        let g = Graph {
            nodes: vec![
                node("x", "X", NodeKind::new(NodeKind::MODULE)),
                node("y", "Y", NodeKind::new(NodeKind::SERVICE)),
            ],
            edges: vec![edge("x", "y", EdgeKind::new(EdgeKind::CALLS))],
            ..Default::default()
        };
        let result = compare_graphs(&g, &g);
        assert_eq!(result.node_diff.added.len(), 0);
        assert_eq!(result.node_diff.removed.len(), 0);
        assert_eq!(result.node_diff.matched.len(), 2);
        assert_eq!(result.edge_diff.added.len(), 0);
        assert_eq!(result.edge_diff.removed.len(), 0);
    }

    #[test]
    fn compare_matched_similarity_same_label() {
        let actual = Graph {
            nodes: vec![node("id", "UserService", NodeKind::new(NodeKind::SERVICE))],
            edges: vec![],
            ..Default::default()
        };
        let proposed = Graph {
            nodes: vec![node("id", "UserService", NodeKind::new(NodeKind::SERVICE))],
            edges: vec![],
            ..Default::default()
        };
        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.matched.len(), 1);
        assert!((result.node_diff.matched[0].similarity - 1.0).abs() < 1e-5);
        assert!(result.node_diff.matched[0].kind_match);
    }
}
