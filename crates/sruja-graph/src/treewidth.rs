//! Treewidth Analysis via Tree Decomposition
//!
//! Treewidth measures structural complexity of dependency graphs. Lower treewidth
//! indicates a more tree-like (maintainable) structure. Higher treewidth suggests
//! cyclic interdependencies that may need refactoring.
//!
//! Uses the min-fill heuristic for fast O(n²) approximation.

use std::collections::{BTreeSet, HashMap, HashSet};

pub struct TreewidthAnalyzer {
    algorithm: TreewidthAlgorithm,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TreewidthAlgorithm {
    #[default]
    MinFill,
}

pub struct TreewidthResult {
    pub treewidth: usize,
    pub decomposition: Vec<TreeBag>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub complexity_rating: ComplexityRating,
    pub elimination_order: Vec<String>,
}

pub struct TreeBag {
    pub id: usize,
    pub nodes: BTreeSet<String>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

pub struct ComplexityHotspot {
    pub nodes: Vec<String>,
    pub treewidth: usize,
    pub suggested_refactor: RefactorSuggestion,
}

pub struct RefactorSuggestion {
    pub description: String,
    pub pattern: RefactorPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorPattern {
    ExtractInterface,
    EventDriven,
    FacadePattern,
    SplitModule,
    IntroduceAbstraction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityRating {
    Low,
    Moderate,
    High,
    Critical,
}

impl std::fmt::Display for ComplexityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityRating::Low => write!(f, "Low"),
            ComplexityRating::Moderate => write!(f, "Moderate"),
            ComplexityRating::High => write!(f, "High"),
            ComplexityRating::Critical => write!(f, "Critical"),
        }
    }
}

impl Default for TreewidthAnalyzer {
    fn default() -> Self {
        Self {
            algorithm: TreewidthAlgorithm::MinFill,
        }
    }
}

impl TreewidthAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_algorithm(algorithm: TreewidthAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn analyze(&self, nodes: &[String], edges: &[(String, String)]) -> TreewidthResult {
        match self.algorithm {
            TreewidthAlgorithm::MinFill => self.min_fill_analysis(nodes, edges),
        }
    }

    fn min_fill_analysis(&self, nodes: &[String], edges: &[(String, String)]) -> TreewidthResult {
        let mut graph = Graph::from_edges(nodes, edges);
        let mut elimination_order: Vec<String> = Vec::new();
        let mut bags: Vec<BTreeSet<String>> = Vec::new();
        let mut max_bag_size = 0;

        while !graph.nodes.is_empty() {
            let node_to_eliminate = self.select_min_fill_node(&graph);
            let neighbors = graph.get_neighbors(&node_to_eliminate);

            let mut bag: BTreeSet<String> = BTreeSet::new();
            bag.insert(node_to_eliminate.clone());
            for neighbor in &neighbors {
                bag.insert(neighbor.clone());
            }

            max_bag_size = max_bag_size.max(bag.len());
            bags.push(bag);

            graph.make_clique(&neighbors);
            graph.remove_node(&node_to_eliminate);
            elimination_order.push(node_to_eliminate);
        }

        let treewidth = max_bag_size.saturating_sub(1);
        let tree_bags = self.build_tree_bags(&bags);
        let hotspots = self.detect_hotspots(&bags, treewidth);
        let rating = self.rate_complexity(treewidth);

        TreewidthResult {
            treewidth,
            decomposition: tree_bags,
            hotspots,
            complexity_rating: rating,
            elimination_order,
        }
    }

    fn select_min_fill_node(&self, graph: &Graph) -> String {
        let mut best_node = String::new();
        let mut min_fill = usize::MAX;

        for node in &graph.nodes {
            let neighbors = graph.get_neighbors(node);
            let fill = self.compute_fill_in(graph, &neighbors);

            if fill < min_fill {
                min_fill = fill;
                best_node = node.clone();
            }
        }

        best_node
    }

    fn compute_fill_in(&self, graph: &Graph, neighbors: &BTreeSet<String>) -> usize {
        let mut fill = 0;
        let neighbors_vec: Vec<&String> = neighbors.iter().collect();

        for i in 0..neighbors_vec.len() {
            for j in (i + 1)..neighbors_vec.len() {
                let u = neighbors_vec[i];
                let v = neighbors_vec[j];
                if !graph.has_edge(u, v) {
                    fill += 1;
                }
            }
        }

        fill
    }

    fn build_tree_bags(&self, bags: &[BTreeSet<String>]) -> Vec<TreeBag> {
        let mut tree_bags: Vec<TreeBag> = bags
            .iter()
            .enumerate()
            .map(|(i, nodes)| TreeBag {
                id: i,
                nodes: nodes.clone(),
                parent: None,
                children: Vec::new(),
            })
            .collect();

        for i in 1..tree_bags.len() {
            let current_bag = &bags[i];

            for j in (0..i).rev() {
                let prev_bag = &bags[j];
                let intersection: BTreeSet<String> =
                    current_bag.intersection(prev_bag).cloned().collect();

                if !intersection.is_empty() {
                    tree_bags[i].parent = Some(j);
                    tree_bags[j].children.push(i);
                    break;
                }
            }
        }

        tree_bags
    }

    fn detect_hotspots(
        &self,
        bags: &[BTreeSet<String>],
        treewidth: usize,
    ) -> Vec<ComplexityHotspot> {
        let threshold = (treewidth as f32 * 0.8) as usize;

        bags.iter()
            .filter(|bag| bag.len() > threshold + 1)
            .map(|bag| {
                let bag_treewidth = bag.len().saturating_sub(1);
                let nodes: Vec<String> = bag.iter().cloned().collect();
                let pattern = self.suggest_refactor_pattern(&nodes, bag_treewidth);

                ComplexityHotspot {
                    nodes: nodes.clone(),
                    treewidth: bag_treewidth,
                    suggested_refactor: RefactorSuggestion {
                        description: self.generate_refactor_description(&nodes, pattern),
                        pattern,
                    },
                }
            })
            .collect()
    }

    fn suggest_refactor_pattern(&self, nodes: &[String], treewidth: usize) -> RefactorPattern {
        if treewidth > 8 {
            RefactorPattern::SplitModule
        } else if nodes.len() > 5 {
            RefactorPattern::EventDriven
        } else {
            RefactorPattern::ExtractInterface
        }
    }

    fn generate_refactor_description(&self, nodes: &[String], pattern: RefactorPattern) -> String {
        let node_list = nodes.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        let suffix = if nodes.len() > 3 {
            format!(" and {} others", nodes.len() - 3)
        } else {
            String::new()
        };

        match pattern {
            RefactorPattern::ExtractInterface => {
                format!(
                    "Extract interface from {}{} to reduce coupling",
                    node_list, suffix
                )
            }
            RefactorPattern::EventDriven => {
                format!(
                    "Introduce event-driven communication between {}{}",
                    node_list, suffix
                )
            }
            RefactorPattern::FacadePattern => {
                format!(
                    "Add facade pattern to simplify interactions among {}{}",
                    node_list, suffix
                )
            }
            RefactorPattern::SplitModule => {
                format!(
                    "Split tightly coupled group {}{} into separate modules",
                    node_list, suffix
                )
            }
            RefactorPattern::IntroduceAbstraction => {
                format!("Introduce abstraction layer for {}{}", node_list, suffix)
            }
        }
    }

    fn rate_complexity(&self, treewidth: usize) -> ComplexityRating {
        match treewidth {
            0..=2 => ComplexityRating::Low,
            3..=5 => ComplexityRating::Moderate,
            6..=10 => ComplexityRating::High,
            _ => ComplexityRating::Critical,
        }
    }
}

struct Graph {
    nodes: HashSet<String>,
    edges: HashSet<(String, String)>,
    adjacency: HashMap<String, BTreeSet<String>>,
}

impl Graph {
    fn from_edges(nodes: &[String], edges: &[(String, String)]) -> Self {
        let mut graph = Graph {
            nodes: nodes.iter().cloned().collect(),
            edges: HashSet::new(),
            adjacency: HashMap::new(),
        };

        for node in nodes {
            graph.adjacency.entry(node.clone()).or_default();
        }

        for (source, target) in edges {
            graph.edges.insert((source.clone(), target.clone()));
            graph.edges.insert((target.clone(), source.clone()));
            graph
                .adjacency
                .entry(source.clone())
                .or_default()
                .insert(target.clone());
            graph
                .adjacency
                .entry(target.clone())
                .or_default()
                .insert(source.clone());
        }

        graph
    }

    fn get_neighbors(&self, node: &str) -> BTreeSet<String> {
        self.adjacency.get(node).cloned().unwrap_or_default()
    }

    fn has_edge(&self, u: &str, v: &str) -> bool {
        self.edges.contains(&(u.to_string(), v.to_string()))
            || self.edges.contains(&(v.to_string(), u.to_string()))
    }

    fn make_clique(&mut self, nodes: &BTreeSet<String>) {
        let nodes_vec: Vec<&String> = nodes.iter().collect();

        for i in 0..nodes_vec.len() {
            for j in (i + 1)..nodes_vec.len() {
                let u = nodes_vec[i];
                let v = nodes_vec[j];

                if !self.has_edge(u, v) {
                    self.edges.insert((u.clone(), v.clone()));
                    self.edges.insert((v.clone(), u.clone()));
                    self.adjacency
                        .entry(u.clone())
                        .or_default()
                        .insert(v.clone());
                    self.adjacency
                        .entry(v.clone())
                        .or_default()
                        .insert(u.clone());
                }
            }
        }
    }

    fn remove_node(&mut self, node: &str) {
        self.nodes.remove(node);
        self.adjacency.remove(node);

        for neighbors in self.adjacency.values_mut() {
            neighbors.remove(node);
        }

        self.edges.retain(|(u, v)| u != node && v != node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let analyzer = TreewidthAnalyzer::new();
        let result = analyzer.analyze(&[], &[]);
        assert_eq!(result.treewidth, 0);
        assert_eq!(result.complexity_rating, ComplexityRating::Low);
    }

    #[test]
    fn test_single_node() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let result = analyzer.analyze(&nodes, &[]);
        assert_eq!(result.treewidth, 0);
    }

    #[test]
    fn test_two_nodes_no_edge() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let result = analyzer.analyze(&nodes, &[]);
        assert_eq!(result.treewidth, 0);
    }

    #[test]
    fn test_two_nodes_with_edge() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![("a".to_string(), "b".to_string())];
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.treewidth, 1);
        assert_eq!(result.complexity_rating, ComplexityRating::Low);
    }

    #[test]
    fn test_tree_structure() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec![
            "root".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let edges = vec![
            ("root".to_string(), "a".to_string()),
            ("root".to_string(), "b".to_string()),
            ("root".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.treewidth, 1);
        assert_eq!(result.complexity_rating, ComplexityRating::Low);
    }

    #[test]
    fn test_cycle() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
            ("c".to_string(), "a".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);
        assert!(result.treewidth >= 2);
    }

    #[test]
    fn test_clique() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let mut edges = Vec::new();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                edges.push((nodes[i].clone(), nodes[j].clone()));
            }
        }
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.treewidth, 3);
        assert_eq!(result.complexity_rating, ComplexityRating::Moderate);
    }

    #[test]
    fn test_large_clique_critical() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes: Vec<String> = (0..12).map(|i| format!("n{}", i)).collect();
        let mut edges = Vec::new();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                edges.push((nodes[i].clone(), nodes[j].clone()));
            }
        }
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.treewidth, 11);
        assert_eq!(result.complexity_rating, ComplexityRating::Critical);
    }

    #[test]
    fn test_hotspots_detected() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes: Vec<String> = (0..8).map(|i| format!("n{}", i)).collect();
        let mut edges = Vec::new();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                edges.push((nodes[i].clone(), nodes[j].clone()));
            }
        }
        let result = analyzer.analyze(&nodes, &edges);
        assert!(!result.hotspots.is_empty());
    }

    #[test]
    fn test_elimination_order_complete() {
        let analyzer = TreewidthAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);
        assert_eq!(result.elimination_order.len(), 3);
    }
}
