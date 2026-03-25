//! Architecture Quality Score (AQS) calculation
//!
//! Multi-dimensional score that measures how well selected components capture
//! the architectural essence of the system.

use sruja_scan::{Graph, Node};
use std::collections::HashMap;

use super::risk::DependencyRisk;
use super::roles::ArchitecturalRole;

/// Architecture Quality Score breakdown
#[derive(Debug, Clone, Default)]
pub struct ArchitectureQualityScore {
    /// Overall score (0-1)
    pub overall: f64,
    /// Coverage of critical roles (entry points, APIs, data stores)
    pub critical_role_coverage: f64,
    /// Coverage of high-centrality nodes
    pub centrality_coverage: f64,
    /// Coverage of high-risk components
    pub risk_coverage: f64,
    /// Coverage of bounded contexts (if available)
    pub context_coverage: f64,
    /// Diversity of selection (spread across domains)
    pub diversity_score: f64,
    /// Compression ratio (selected / total)
    pub compression_ratio: f64,
    /// Detailed breakdown by role
    pub role_breakdown: HashMap<ArchitecturalRole, f64>,
}

impl ArchitectureQualityScore {
    /// Grade based on overall score
    pub fn grade(&self) -> &'static str {
        if self.overall >= 0.9 {
            "A"
        } else if self.overall >= 0.8 {
            "B"
        } else if self.overall >= 0.7 {
            "C"
        } else if self.overall >= 0.6 {
            "D"
        } else {
            "F"
        }
    }

    /// Summary for display
    pub fn summary(&self) -> String {
        format!(
            "AQS: {:.0}% (Grade {}) - Critical: {:.0}% | Centrality: {:.0}% | Risk: {:.0}% | Diversity: {:.0}%",
            self.overall * 100.0,
            self.grade(),
            self.critical_role_coverage * 100.0,
            self.centrality_coverage * 100.0,
            self.risk_coverage * 100.0,
            self.diversity_score * 100.0,
        )
    }
}

/// Compute Architecture Quality Score for a selection
pub fn compute_aqs(
    selected_nodes: &[Node],
    graph: &Graph,
    role_map: &HashMap<String, ArchitecturalRole>,
    risk_scores: &HashMap<String, DependencyRisk>,
) -> ArchitectureQualityScore {
    let total_nodes = graph.nodes.len();
    if total_nodes == 0 {
        return ArchitectureQualityScore::default();
    }

    let selected_ids: HashMap<&str, &Node> =
        selected_nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let critical_role_coverage = compute_critical_role_coverage(&selected_ids, graph, role_map);
    let centrality_coverage = compute_centrality_coverage(&selected_ids, graph);
    let risk_coverage = compute_risk_coverage(&selected_ids, graph, risk_scores);
    let context_coverage = compute_context_coverage(&selected_ids, graph);
    let diversity_score = compute_diversity_score(&selected_ids, graph);
    let compression_ratio = selected_nodes.len() as f64 / total_nodes as f64;

    let role_breakdown = compute_role_breakdown(&selected_ids, graph, role_map);

    let overall = 0.30 * critical_role_coverage
        + 0.25 * centrality_coverage
        + 0.20 * risk_coverage
        + 0.10 * context_coverage
        + 0.15 * diversity_score;

    ArchitectureQualityScore {
        overall,
        critical_role_coverage,
        centrality_coverage,
        risk_coverage,
        context_coverage,
        diversity_score,
        compression_ratio,
        role_breakdown,
    }
}

fn compute_critical_role_coverage(
    selected_ids: &HashMap<&str, &Node>,
    graph: &Graph,
    role_map: &HashMap<String, ArchitecturalRole>,
) -> f64 {
    let critical_roles = [
        ArchitecturalRole::EntryPoint,
        ArchitecturalRole::ApiSurface,
        ArchitecturalRole::DataStore,
    ];

    let mut total_critical = 0usize;
    let mut covered_critical = 0usize;

    for node in &graph.nodes {
        let role = role_map
            .get(&node.id)
            .copied()
            .unwrap_or(ArchitecturalRole::Unknown);
        if critical_roles.contains(&role) {
            total_critical += 1;
            if selected_ids.contains_key(node.id.as_str()) {
                covered_critical += 1;
            }
        }
    }

    if total_critical == 0 {
        return 1.0;
    }

    covered_critical as f64 / total_critical as f64
}

fn compute_centrality_coverage(selected_ids: &HashMap<&str, &Node>, graph: &Graph) -> f64 {
    let n = graph.nodes.len();
    if n < 10 {
        return 1.0;
    }

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut out_degree: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *in_degree.entry(&edge.target).or_default() += 1;
        *out_degree.entry(&edge.source).or_default() += 1;
    }

    let mut node_scores: Vec<(&str, usize)> = graph
        .nodes
        .iter()
        .map(|n| {
            let total = in_degree.get(n.id.as_str()).copied().unwrap_or(0)
                + out_degree.get(n.id.as_str()).copied().unwrap_or(0);
            (n.id.as_str(), total)
        })
        .collect();

    node_scores.sort_by(|a, b| b.1.cmp(&a.1));

    let top_10_pct = (n as f64 * 0.10).ceil() as usize;
    let top_nodes: Vec<&str> = node_scores
        .iter()
        .take(top_10_pct)
        .map(|(id, _)| *id)
        .collect();

    if top_nodes.is_empty() {
        return 1.0;
    }

    let covered = top_nodes
        .iter()
        .filter(|id| selected_ids.contains_key(*id))
        .count();

    covered as f64 / top_nodes.len() as f64
}

fn compute_risk_coverage(
    selected_ids: &HashMap<&str, &Node>,
    graph: &Graph,
    risk_scores: &HashMap<String, DependencyRisk>,
) -> f64 {
    let high_risk_threshold = 0.5;

    let high_risk_nodes: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            risk_scores
                .get(&n.id)
                .map(|r| r.risk_score > high_risk_threshold)
                .unwrap_or(false)
        })
        .map(|n| n.id.as_str())
        .collect();

    if high_risk_nodes.is_empty() {
        return 1.0;
    }

    let covered = high_risk_nodes
        .iter()
        .filter(|id| selected_ids.contains_key(*id))
        .count();

    covered as f64 / high_risk_nodes.len() as f64
}

fn compute_context_coverage(selected_ids: &HashMap<&str, &Node>, graph: &Graph) -> f64 {
    let contexts = extract_contexts(graph);

    if contexts.is_empty() {
        return 1.0;
    }

    let contexts_with_representation = contexts
        .values()
        .filter(|nodes| {
            nodes
                .iter()
                .any(|id| selected_ids.contains_key(id.as_str()))
        })
        .count();

    contexts_with_representation as f64 / contexts.len() as f64
}

fn extract_contexts(graph: &Graph) -> HashMap<String, Vec<String>> {
    let mut contexts: HashMap<String, Vec<String>> = HashMap::new();

    for node in &graph.nodes {
        let context = extract_node_context(node);
        contexts.entry(context).or_default().push(node.id.clone());
    }

    contexts
}

fn extract_node_context(node: &Node) -> String {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let parts: Vec<&str> = path.split('/').collect();

    for (i, part) in parts.iter().enumerate() {
        if (*part == "src"
            || *part == "lib"
            || *part == "crates"
            || *part == "packages"
            || *part == "services")
            && i + 1 < parts.len()
        {
            return parts[i + 1].to_string();
        }
    }

    if parts.len() >= 2 {
        return format!("{}_{}", parts[0], parts.get(1).unwrap_or(&""));
    }

    "root".to_string()
}

fn compute_diversity_score(selected_ids: &HashMap<&str, &Node>, graph: &Graph) -> f64 {
    let all_contexts = extract_contexts(graph);
    let selected_contexts: HashMap<String, usize> = selected_ids
        .values()
        .map(|n| (extract_node_context(n), ()))
        .filter(|(ctx, _)| ctx != "root")
        .map(|(ctx, _)| ctx)
        .fold(HashMap::new(), |mut acc, ctx| {
            *acc.entry(ctx).or_default() += 1;
            acc
        });

    let total_contexts = all_contexts.keys().filter(|c| *c != "root").count();
    if total_contexts == 0 {
        return 1.0;
    }

    let represented = selected_contexts.len();
    let ratio = represented as f64 / total_contexts as f64;

    let ideal_per_context = selected_ids.len() as f64 / total_contexts.max(1) as f64;
    let variance: f64 = selected_contexts
        .values()
        .map(|&count| {
            ((count as f64 - ideal_per_context).abs() / ideal_per_context.max(1.0)).powi(2)
        })
        .sum::<f64>()
        / selected_contexts.len().max(1) as f64;

    let balance_score = 1.0 / (1.0 + variance.sqrt());

    0.6 * ratio + 0.4 * balance_score
}

fn compute_role_breakdown(
    selected_ids: &HashMap<&str, &Node>,
    graph: &Graph,
    role_map: &HashMap<String, ArchitecturalRole>,
) -> HashMap<ArchitecturalRole, f64> {
    let mut totals: HashMap<ArchitecturalRole, usize> = HashMap::new();
    let mut covered: HashMap<ArchitecturalRole, usize> = HashMap::new();

    for node in &graph.nodes {
        let role = role_map
            .get(&node.id)
            .copied()
            .unwrap_or(ArchitecturalRole::Unknown);
        *totals.entry(role).or_default() += 1;

        if selected_ids.contains_key(node.id.as_str()) {
            *covered.entry(role).or_default() += 1;
        }
    }

    totals
        .into_iter()
        .map(|(role, total)| {
            let cov = covered.get(&role).copied().unwrap_or(0);
            let score = if total == 0 {
                1.0
            } else {
                cov as f64 / total as f64
            };
            (role, score)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, NodeKind};

    fn make_graph(nodes: usize) -> Graph {
        let nodes: Vec<Node> = (0..nodes)
            .map(|i| Node {
                id: format!("n{}", i),
                kind: NodeKind::Module,
                label: format!("n{}", i),
                technology: None,
                path: Some(format!("src/domain{}/mod.rs", i % 3)),
                metadata: Default::default(),
                canonical_id: None,
                aliases: Vec::new(),
                owner: None,
                domain: None,
                criticality: None,
                sources: Vec::new(),
            })
            .collect();

        let edges: Vec<Edge> = (0..nodes.len().saturating_sub(1))
            .map(|i| Edge {
                source: format!("n{}", i),
                target: format!("n{}", i + 1),
                kind: EdgeKind::DependsOn,
                evidence: vec![],
            })
            .collect();

        Graph {
            nodes,
            edges,
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_aqs_empty_selection() {
        let graph = make_graph(10);
        let role_map = HashMap::new();
        let risk_scores = HashMap::new();

        let aqs = compute_aqs(&[], &graph, &role_map, &risk_scores);
        // Empty selection should have low diversity score, affecting overall
        assert!(aqs.diversity_score < 0.5);
        assert!(aqs.overall < 1.0);
    }

    #[test]
    fn test_aqs_full_selection() {
        let graph = make_graph(10);
        let role_map: HashMap<String, ArchitecturalRole> = graph
            .nodes
            .iter()
            .map(|n| (n.id.clone(), ArchitecturalRole::CoreDomain))
            .collect();
        let risk_scores: HashMap<String, DependencyRisk> = graph
            .nodes
            .iter()
            .map(|n| (n.id.clone(), DependencyRisk::default()))
            .collect();

        let aqs = compute_aqs(&graph.nodes, &graph, &role_map, &risk_scores);
        assert!(aqs.overall > 0.8);
    }

    #[test]
    fn test_aqs_grade() {
        let aqs = ArchitectureQualityScore {
            overall: 0.95,
            ..Default::default()
        };
        assert_eq!(aqs.grade(), "A");

        let aqs = ArchitectureQualityScore {
            overall: 0.85,
            ..Default::default()
        };
        assert_eq!(aqs.grade(), "B");

        let aqs = ArchitectureQualityScore {
            overall: 0.75,
            ..Default::default()
        };
        assert_eq!(aqs.grade(), "C");

        let aqs = ArchitectureQualityScore {
            overall: 0.55,
            ..Default::default()
        };
        assert_eq!(aqs.grade(), "F");
    }

    #[test]
    fn test_compression_ratio() {
        let graph = make_graph(100);
        let role_map = HashMap::new();
        let risk_scores = HashMap::new();

        let selected: Vec<Node> = graph.nodes.iter().take(15).cloned().collect();
        let aqs = compute_aqs(&selected, &graph, &role_map, &risk_scores);

        assert!((aqs.compression_ratio - 0.15).abs() < 0.01);
    }
}
