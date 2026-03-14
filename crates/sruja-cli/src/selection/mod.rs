//! Smart Component Coverage Selection
//!
//! Selects components based on architectural importance rather than simple percentage.
//! Goal: Enable answering architecture questions with minimal component selection.

mod centrality;
pub mod question_coverage;
mod risk;
mod roles;
mod score;
pub mod summarize;

pub use centrality::{compute_all_centrality, ComponentImportance};
pub use question_coverage::{evaluate_question_coverage, refine_for_questions};
pub use risk::{compute_dependency_risk, DependencyRisk};
pub use roles::{detect_architectural_role, ArchitecturalRole};
pub use score::{compute_aqs, ArchitectureQualityScore};
pub use summarize::{summarize_large_component, ComponentSummary};

use sruja_scan::{Graph, Node};
use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_config_default() {
        let config = SelectionConfig::default();
        assert_eq!(config.target_ratio, 0.15);
        assert_eq!(config.target_question_coverage, 0.80);
        assert!(config.critical_roles.len() >= 2);
        assert!(config.include_high_risk);
        assert!(!config.enable_llm);
    }
}

/// Configuration for smart selection
#[derive(Debug, Clone)]
pub struct SelectionConfig {
    /// Target compression ratio (0.1 = 10% of nodes)
    pub target_ratio: f64,
    /// Minimum question coverage score to achieve
    pub target_question_coverage: f64,
    /// Always include these architectural roles
    pub critical_roles: Vec<ArchitecturalRole>,
    /// Include high-risk components
    pub include_high_risk: bool,
    /// Minimum centrality percentile to include
    pub min_centrality_percentile: f64,
    /// Enable LLM-enhanced selection
    pub enable_llm: bool,
    /// Include bounded context representatives
    pub include_context_boundaries: bool,
    /// Minimum representatives per bounded context
    pub min_context_representatives: usize,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            target_ratio: 0.15,
            target_question_coverage: 0.80,
            critical_roles: vec![
                ArchitecturalRole::EntryPoint,
                ArchitecturalRole::ApiSurface,
                ArchitecturalRole::DataStore,
            ],
            include_high_risk: true,
            min_centrality_percentile: 90.0,
            enable_llm: false,
            include_context_boundaries: true,
            min_context_representatives: 3,
        }
    }
}

/// Result of smart selection
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Selected nodes
    pub nodes: Vec<Node>,
    /// Architecture Quality Score
    pub quality_score: ArchitectureQualityScore,
    /// Role coverage breakdown
    pub role_coverage: HashMap<ArchitecturalRole, RoleCoverage>,
    /// Selection reasons for each node
    pub selection_reasons: HashMap<String, Vec<SelectionReason>>,
}

#[derive(Debug, Clone)]
pub struct RoleCoverage {
    pub total: usize,
    pub selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionReason {
    CriticalRole,
    HighCentrality,
    HighRisk,
    BridgeNode,
    ContextBoundary,
    DiverseSample,
    UserRequested,
}

/// Perform smart component selection
pub fn smart_select(graph: &Graph, config: &SelectionConfig) -> SelectionResult {
    let mut selector = Selector::new(graph, config);
    selector.select();
    selector.into_result()
}

struct Selector<'a> {
    graph: &'a Graph,
    config: &'a SelectionConfig,
    selected_ids: HashSet<String>,
    selected_nodes: Vec<Node>,
    selection_reasons: HashMap<String, Vec<SelectionReason>>,
    centrality_scores: HashMap<String, ComponentImportance>,
    role_map: HashMap<String, ArchitecturalRole>,
    risk_scores: HashMap<String, DependencyRisk>,
}

impl<'a> Selector<'a> {
    fn new(graph: &'a Graph, config: &'a SelectionConfig) -> Self {
        Self {
            graph,
            config,
            selected_ids: HashSet::new(),
            selected_nodes: Vec::new(),
            selection_reasons: HashMap::new(),
            centrality_scores: HashMap::new(),
            role_map: HashMap::new(),
            risk_scores: HashMap::new(),
        }
    }

    fn select(&mut self) {
        eprintln!("   🎯 Computing component importance metrics...");

        self.compute_metrics();
        self.select_critical_roles();
        self.select_high_centrality();
        self.select_high_risk();
        self.select_bridge_nodes();
        self.select_context_representatives();
        self.select_diverse_sample();
    }

    fn compute_metrics(&mut self) {
        self.centrality_scores = centrality::compute_all_centrality(self.graph);

        for node in &self.graph.nodes {
            let role = roles::detect_architectural_role(node, self.graph);
            self.role_map.insert(node.id.clone(), role);

            let risk = risk::compute_dependency_risk(node, self.graph);
            self.risk_scores.insert(node.id.clone(), risk);
        }
    }

    fn select_critical_roles(&mut self) {
        for node in &self.graph.nodes {
            if let Some(role) = self.role_map.get(&node.id) {
                if self.config.critical_roles.contains(role) {
                    self.add_node(node, SelectionReason::CriticalRole);
                }
            }
        }
    }

    fn select_high_centrality(&mut self) {
        let threshold = self.compute_centrality_threshold();

        let mut candidates: Vec<_> = self
            .graph
            .nodes
            .iter()
            .filter(|n| !self.selected_ids.contains(&n.id))
            .filter_map(|n| {
                let importance = self.centrality_scores.get(&n.id)?;
                let max_score = importance
                    .degree_centrality
                    .max(importance.betweenness_centrality)
                    .max(importance.pagerank);
                Some((n, max_score))
            })
            .filter(|(_, score)| *score >= threshold)
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let budget = self.compute_budget_for_category(0.15);
        for (node, _) in candidates.into_iter().take(budget) {
            self.add_node(node, SelectionReason::HighCentrality);
        }
    }

    fn select_high_risk(&mut self) {
        if !self.config.include_high_risk {
            return;
        }

        let mut high_risk: Vec<_> = self
            .graph
            .nodes
            .iter()
            .filter(|n| !self.selected_ids.contains(&n.id))
            .filter_map(|n| {
                let risk = self.risk_scores.get(&n.id)?;
                if risk.risk_score > 0.5 {
                    Some((n, risk.risk_score))
                } else {
                    None
                }
            })
            .collect();

        high_risk.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let budget = self.compute_budget_for_category(0.10);
        for (node, _) in high_risk.into_iter().take(budget) {
            self.add_node(node, SelectionReason::HighRisk);
        }
    }

    fn select_bridge_nodes(&mut self) {
        let mut bridge_nodes: Vec<_> = self
            .graph
            .nodes
            .iter()
            .filter(|n| !self.selected_ids.contains(&n.id))
            .filter_map(|n| {
                let importance = self.centrality_scores.get(&n.id)?;
                if importance.betweenness_centrality > 0.1 {
                    Some((n, importance.betweenness_centrality))
                } else {
                    None
                }
            })
            .collect();

        bridge_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let budget = self.compute_budget_for_category(0.10);
        for (node, _) in bridge_nodes.into_iter().take(budget) {
            self.add_node(node, SelectionReason::BridgeNode);
        }
    }

    fn select_context_representatives(&mut self) {
        // Semantic analysis removed - context boundary detection disabled
        // This feature previously used sruja-semantic (embedding-based clustering)
        // but was experimental and had no production usage.
        // To re-enable: add sruja-semantic back as optional dependency.
    }

    #[allow(dead_code)]
    fn sample_by_centrality_ids(&self, nodes: &[Node], count: usize) -> Vec<String> {
        let mut scored: Vec<_> = nodes
            .iter()
            .filter_map(|n| {
                let importance = self.centrality_scores.get(&n.id)?;
                let score = importance
                    .degree_centrality
                    .max(importance.betweenness_centrality)
                    .max(importance.pagerank);
                Some((n.id.clone(), score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(count).map(|(id, _)| id).collect()
    }

    #[allow(dead_code)]
    fn find_boundary_node_ids(
        &self,
        context_ids: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        self.graph
            .edges
            .iter()
            .filter(|e| {
                let source_in = context_ids.contains(&e.source);
                let target_in = context_ids.contains(&e.target);
                source_in != target_in
            })
            .map(|e| {
                if context_ids.contains(&e.source) {
                    e.source.clone()
                } else {
                    e.target.clone()
                }
            })
            .filter(|id| !self.selected_ids.contains(id))
            .collect()
    }

    fn select_diverse_sample(&mut self) {
        let target = (self.graph.nodes.len() as f64 * self.config.target_ratio) as usize;
        let remaining = target.saturating_sub(self.selected_nodes.len());

        if remaining == 0 {
            return;
        }

        let candidates: Vec<_> = self
            .graph
            .nodes
            .iter()
            .filter(|n| !self.selected_ids.contains(&n.id))
            .filter(|n| !is_excluded(n))
            .collect();

        let stride = (candidates.len() / remaining).max(1);
        for (i, node) in candidates.iter().enumerate() {
            if i % stride == 0 && self.selected_nodes.len() < target {
                self.add_node(node, SelectionReason::DiverseSample);
            }
        }
    }

    fn add_node(&mut self, node: &Node, reason: SelectionReason) {
        if self.selected_ids.insert(node.id.clone()) {
            self.selected_nodes.push(node.clone());
        }
        self.selection_reasons
            .entry(node.id.clone())
            .or_default()
            .push(reason);
    }

    fn compute_centrality_threshold(&self) -> f64 {
        let mut scores: Vec<_> = self
            .centrality_scores
            .values()
            .flat_map(|i| [i.degree_centrality, i.betweenness_centrality, i.pagerank])
            .collect();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let idx = (scores.len() as f64 * self.config.min_centrality_percentile / 100.0) as usize;
        scores.get(idx).copied().unwrap_or(0.5)
    }

    fn compute_budget_for_category(&self, pct: f64) -> usize {
        let target = (self.graph.nodes.len() as f64 * self.config.target_ratio) as usize;
        (target as f64 * pct) as usize
    }

    fn into_result(self) -> SelectionResult {
        let quality_score = score::compute_aqs(
            &self.selected_nodes,
            self.graph,
            &self.role_map,
            &self.risk_scores,
        );

        let mut role_coverage = HashMap::new();
        for role in ArchitecturalRole::all() {
            let total = self.role_map.values().filter(|r| **r == role).count();
            let selected = self
                .selected_nodes
                .iter()
                .filter(|n| self.role_map.get(&n.id) == Some(&role))
                .count();
            role_coverage.insert(role, RoleCoverage { total, selected });
        }

        SelectionResult {
            nodes: self.selected_nodes,
            quality_score,
            role_coverage,
            selection_reasons: self.selection_reasons,
        }
    }
}

fn is_excluded(node: &Node) -> bool {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let path_lower = path.to_lowercase();

    path_lower.contains("/test")
        || path_lower.contains("/tests")
        || path_lower.contains("/__tests__")
        || path_lower.contains("/spec")
        || path_lower.contains("/specs")
        || path_lower.contains("/example")
        || path_lower.contains("/examples")
        || path_lower.contains("/docs/")
        || path_lower.contains("/documentation")
        || path_lower.contains("/.github")
        || path_lower.contains("/node_modules")
}
