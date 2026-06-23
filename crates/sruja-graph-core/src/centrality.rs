//! Centrality Metrics for Architecture Analysis
//!
//! Identifies architecturally significant nodes based on their position in the
//! dependency graph. High centrality nodes are often critical architectural
//! components that require special attention during refactoring.

use crate::{ContextEdge, ContextGraph, ContextNode};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct CentralityAnalyzer {
    normalized: bool,
    max_sample_size: usize,
    quiet: bool,
}

const DEFAULT_MAX_SAMPLE_SIZE: usize = 1000;
const LARGE_GRAPH_THRESHOLD: usize = 5000;
const VERY_LARGE_GRAPH_THRESHOLD: usize = 50000;

pub struct CentralityResult {
    pub betweenness: HashMap<String, f64>,
    pub closeness: HashMap<String, f64>,
    pub degree: HashMap<String, f64>,
    pub pagerank: HashMap<String, f64>,
    pub eigenvector: HashMap<String, f64>,
    pub hotspots: Vec<ArchitecturalHotspot>,
    pub top_hubs: Vec<HubNode>,
    pub top_bridges: Vec<BridgeNode>,
}

pub struct ArchitecturalHotspot {
    pub node: String,
    pub betweenness: f64,
    pub closeness: f64,
    pub degree: f64,
    pub combined_score: f64,
    pub role: HotspotRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotspotRole {
    Hub,
    Bridge,
    Bottleneck,
    Peripheral,
}

pub struct HubNode {
    pub node: String,
    pub degree_centrality: f64,
    pub dependents: usize,
}

pub struct BridgeNode {
    pub node: String,
    pub betweenness: f64,
    pub connects_components: usize,
}

/// Result of BFS phase in Brandes' betweenness algorithm.
type BrandesBfsResult = (
    Vec<String>,
    HashMap<String, Vec<String>>,
    HashMap<String, f64>,
    HashMap<String, usize>,
);

impl Default for CentralityAnalyzer {
    fn default() -> Self {
        Self {
            normalized: true,
            max_sample_size: DEFAULT_MAX_SAMPLE_SIZE,
            quiet: false,
        }
    }
}

impl CentralityAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_normalized(normalized: bool) -> Self {
        Self {
            normalized,
            ..Self::default()
        }
    }

    pub fn with_max_sample_size(max_sample_size: usize) -> Self {
        Self {
            max_sample_size,
            ..Self::default()
        }
    }

    pub fn with_quiet(quiet: bool) -> Self {
        Self {
            quiet,
            ..Self::default()
        }
    }

    pub fn analyze_graph<G: ContextGraph>(&self, graph: &G) -> CentralityResult {
        let nodes: Vec<String> = graph.nodes().iter().map(|n| n.id().to_string()).collect();
        let edges: Vec<(String, String)> = graph
            .edges()
            .iter()
            .map(|e| (e.source().to_string(), e.target().to_string()))
            .collect();
        self.analyze(&nodes, &edges)
    }

    pub fn analyze(&self, nodes: &[String], edges: &[(String, String)]) -> CentralityResult {
        let graph = DirectedGraph::from_edges(nodes, edges);

        let skip_betweenness = graph.nodes.len() > VERY_LARGE_GRAPH_THRESHOLD;

        let betweenness = if skip_betweenness {
            eprintln!(
                "  ℹ️  Skipping betweenness centrality (graph too large: {} nodes)",
                graph.nodes.len()
            );
            eprintln!("     Using degree centrality only for large graph analysis");
            nodes.iter().map(|n| (n.clone(), 0.0)).collect()
        } else {
            self.compute_betweenness(&graph)
        };

        let closeness = self.compute_closeness(&graph);
        let degree = self.compute_degree(&graph);
        let pagerank = self.compute_pagerank(&graph);
        let eigenvector = self.compute_eigenvector(&graph);

        let hotspots = self.identify_hotspots(&betweenness, &closeness, &degree);
        let top_hubs = self.identify_hubs(&degree, &graph);
        let top_bridges = if skip_betweenness {
            vec![]
        } else {
            self.identify_bridges(&betweenness, &graph)
        };

        CentralityResult {
            betweenness,
            closeness,
            degree,
            pagerank,
            eigenvector,
            hotspots,
            top_hubs,
            top_bridges,
        }
    }

    fn compute_betweenness(&self, graph: &DirectedGraph) -> HashMap<String, f64> {
        let mut betweenness: HashMap<String, f64> = HashMap::new();

        for node in &graph.nodes {
            betweenness.insert(node.clone(), 0.0);
        }

        let sources: Vec<String> = if graph.nodes.len() > LARGE_GRAPH_THRESHOLD {
            let degree = self.compute_degree(graph);
            let mut nodes_with_degree: Vec<(String, f64)> = degree.into_iter().collect();
            nodes_with_degree
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            nodes_with_degree
                .into_iter()
                .take(self.max_sample_size)
                .map(|(node, _)| node)
                .collect()
        } else {
            graph.nodes.clone()
        };

        let total_sources = sources.len();
        let progress_interval = if total_sources > 5000 { 1000 } else { 500 };
        for (idx, source) in sources.iter().enumerate() {
            if !self.quiet && total_sources > 100 && idx % progress_interval == 0 {
                eprintln!(
                    "  Computing betweenness: {}/{} sources processed",
                    idx, total_sources
                );
            }

            let (mut stack, predecessors, sigma, _) = self.brandes_bfs(graph, source);

            let mut delta: HashMap<String, f64> = HashMap::new();
            for v in &graph.nodes {
                delta.insert(v.clone(), 0.0);
            }

            while let Some(w) = stack.pop() {
                let sigma_w = sigma.get(&w).copied().unwrap_or(0.0);
                let delta_w = delta.get(&w).copied().unwrap_or(0.0);

                if let Some(preds) = predecessors.get(&w) {
                    for v in preds {
                        let sigma_v = sigma.get(v).copied().unwrap_or(1.0);
                        if let Some(delta_v) = delta.get_mut(v) {
                            *delta_v += (sigma_v / sigma_w.max(1.0)) * (1.0 + delta_w);
                        }
                    }
                }

                if w != *source {
                    if let Some(bw) = betweenness.get_mut(&w) {
                        *bw += delta_w;
                    }
                }
            }
        }

        if graph.nodes.len() > LARGE_GRAPH_THRESHOLD {
            let scale = graph.nodes.len() as f64 / sources.len() as f64;
            for value in betweenness.values_mut() {
                *value *= scale;
            }
        }

        if self.normalized && graph.nodes.len() > 2 {
            let n = graph.nodes.len() as f64;
            let norm = (n - 1.0) * (n - 2.0);
            if norm > 0.0 {
                for value in betweenness.values_mut() {
                    *value /= norm;
                }
            }
        }

        betweenness
    }

    fn brandes_bfs(&self, graph: &DirectedGraph, source: &str) -> BrandesBfsResult {
        let mut stack: Vec<String> = Vec::new();
        let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
        let mut sigma: HashMap<String, f64> = HashMap::new();
        let mut dist: HashMap<String, usize> = HashMap::new();

        for node in &graph.nodes {
            sigma.insert(node.clone(), 0.0);
            dist.insert(node.clone(), usize::MAX);
            predecessors.insert(node.clone(), Vec::new());
        }

        sigma.insert(source.to_string(), 1.0);
        dist.insert(source.to_string(), 0);

        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(source.to_string());

        while let Some(v) = queue.pop_front() {
            stack.push(v.clone());
            let dist_v = dist.get(&v).copied().unwrap_or(usize::MAX);

            if let Some(neighbors) = graph.successors.get(&v) {
                for w in neighbors {
                    let dist_w = dist.get(w).copied().unwrap_or(usize::MAX);

                    if dist_w == usize::MAX {
                        dist.insert(w.clone(), dist_v + 1);
                        queue.push_back(w.clone());
                    }

                    if dist_w == usize::MAX || dist_w == dist_v + 1 {
                        let sigma_v = sigma.get(&v).copied().unwrap_or(0.0);
                        if let Some(sigma_w) = sigma.get_mut(w) {
                            *sigma_w += sigma_v;
                        }
                        if let Some(preds) = predecessors.get_mut(w) {
                            preds.push(v.clone());
                        }
                    }
                }
            }
        }

        (stack, predecessors, sigma, dist)
    }

    fn compute_closeness(&self, graph: &DirectedGraph) -> HashMap<String, f64> {
        let mut closeness: HashMap<String, f64> = HashMap::new();
        let n = graph.nodes.len();

        let sources: Vec<String> = if graph.nodes.len() > LARGE_GRAPH_THRESHOLD {
            let degree = self.compute_degree(graph);
            let mut nodes_with_degree: Vec<(String, f64)> = degree.into_iter().collect();
            nodes_with_degree
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            nodes_with_degree
                .into_iter()
                .take(self.max_sample_size)
                .map(|(node, _)| node)
                .collect()
        } else {
            graph.nodes.clone()
        };

        for source in &sources {
            let distances = self.bfs_distances(graph, source);
            let reachable: usize = distances.values().filter(|&&d| d < usize::MAX).count();
            let total_dist: usize = distances.values().filter(|&&d| d < usize::MAX).sum();

            if total_dist > 0 && reachable > 1 {
                let cc = (reachable - 1) as f64 / total_dist as f64;

                if self.normalized && n > 1 {
                    let normalized_cc = cc * (reachable - 1) as f64 / (n - 1) as f64;
                    closeness.insert(source.clone(), normalized_cc);
                } else {
                    closeness.insert(source.clone(), cc);
                }
            } else {
                closeness.insert(source.clone(), 0.0);
            }
        }

        for node in &graph.nodes {
            if !closeness.contains_key(node) {
                closeness.insert(node.clone(), 0.0);
            }
        }

        closeness
    }

    fn bfs_distances(&self, graph: &DirectedGraph, source: &str) -> HashMap<String, usize> {
        let mut dist: HashMap<String, usize> = HashMap::new();

        for node in &graph.nodes {
            dist.insert(node.clone(), usize::MAX);
        }
        dist.insert(source.to_string(), 0);

        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(source.to_string());

        while let Some(v) = queue.pop_front() {
            let dist_v = dist.get(&v).copied().unwrap_or(usize::MAX);

            if let Some(neighbors) = graph.successors.get(&v) {
                for w in neighbors {
                    let dist_w = dist.get(w).copied().unwrap_or(usize::MAX);
                    if dist_w == usize::MAX {
                        dist.insert(w.clone(), dist_v + 1);
                        queue.push_back(w.clone());
                    }
                }
            }
        }

        dist
    }

    fn compute_degree(&self, graph: &DirectedGraph) -> HashMap<String, f64> {
        let n = graph.nodes.len().max(1);
        let mut degree: HashMap<String, f64> = HashMap::new();

        for node in &graph.nodes {
            let out_degree = graph.successors.get(node).map(|s| s.len()).unwrap_or(0);
            let in_degree = graph.predecessors.get(node).map(|p| p.len()).unwrap_or(0);

            let total = (in_degree + out_degree) as f64;

            if self.normalized {
                degree.insert(node.clone(), total / (n - 1).max(1) as f64);
            } else {
                degree.insert(node.clone(), total);
            }
        }

        degree
    }

    fn compute_pagerank(&self, graph: &DirectedGraph) -> HashMap<String, f64> {
        let n = graph.nodes.len();
        if n == 0 {
            return HashMap::new();
        }

        let damping = 0.85;
        let tolerance = 1e-6;
        let max_iterations = 100;

        let mut out_degree: HashMap<&str, usize> = HashMap::new();
        for node in &graph.nodes {
            out_degree.insert(
                node.as_str(),
                graph.successors.get(node).map(|s| s.len()).unwrap_or(0),
            );
        }

        let mut rank: HashMap<String, f64> = HashMap::new();
        let initial = 1.0 / n as f64;
        for node in &graph.nodes {
            rank.insert(node.clone(), initial);
        }

        for _ in 0..max_iterations {
            let mut new_rank: HashMap<String, f64> = HashMap::new();
            for node in &graph.nodes {
                new_rank.insert(node.clone(), (1.0 - damping) / n as f64);
            }

            for node in &graph.nodes {
                if let Some(preds) = graph.predecessors.get(node) {
                    for pred in preds {
                        let pred_out = out_degree.get(pred.as_str()).copied().unwrap_or(0);
                        if pred_out > 0 {
                            let pred_rank = rank.get(pred).copied().unwrap_or(0.0);
                            if let Some(r) = new_rank.get_mut(node) {
                                *r += damping * pred_rank / pred_out as f64;
                            }
                        }
                    }
                }
            }

            let diff: f64 = new_rank
                .iter()
                .map(|(k, v)| (v - rank.get(k).copied().unwrap_or(0.0)).abs())
                .sum();

            rank = new_rank;

            if diff < tolerance {
                break;
            }
        }

        let max_rank = rank.values().cloned().fold(0.0f64, f64::max).max(1e-10);
        for value in rank.values_mut() {
            *value /= max_rank;
        }

        rank
    }

    fn compute_eigenvector(&self, graph: &DirectedGraph) -> HashMap<String, f64> {
        let n = graph.nodes.len();
        if n == 0 {
            return HashMap::new();
        }

        let mut centrality: HashMap<String, f64> = HashMap::new();
        for node in &graph.nodes {
            centrality.insert(node.clone(), 1.0);
        }

        let max_iterations = 100;
        let tolerance = 1e-6;

        for _ in 0..max_iterations {
            let mut new_centrality: HashMap<String, f64> = HashMap::new();
            for node in &graph.nodes {
                new_centrality.insert(node.clone(), 0.0);
            }

            for node in &graph.nodes {
                if let Some(preds) = graph.predecessors.get(node) {
                    for pred in preds {
                        let pred_c = centrality.get(pred).copied().unwrap_or(0.0);
                        if let Some(c) = new_centrality.get_mut(node) {
                            *c += pred_c;
                        }
                    }
                }
            }

            let norm: f64 = new_centrality
                .values()
                .map(|x| x * x)
                .sum::<f64>()
                .sqrt()
                .max(1e-10);
            for value in new_centrality.values_mut() {
                *value /= norm;
            }

            let diff: f64 = new_centrality
                .iter()
                .map(|(k, v)| (v - centrality.get(k).copied().unwrap_or(0.0)).abs())
                .sum();

            centrality = new_centrality;

            if diff < tolerance {
                break;
            }
        }

        let max_c = centrality
            .values()
            .cloned()
            .fold(0.0f64, f64::max)
            .max(1e-10);
        for value in centrality.values_mut() {
            *value /= max_c;
        }

        centrality
    }

    fn identify_hotspots(
        &self,
        betweenness: &HashMap<String, f64>,
        closeness: &HashMap<String, f64>,
        degree: &HashMap<String, f64>,
    ) -> Vec<ArchitecturalHotspot> {
        let max_bw = betweenness.values().cloned().fold(0.0, f64::max).max(1.0);
        let max_cc = closeness.values().cloned().fold(0.0, f64::max).max(1.0);
        let max_deg = degree.values().cloned().fold(0.0, f64::max).max(1.0);

        let mut hotspots: Vec<ArchitecturalHotspot> = betweenness
            .keys()
            .map(|node| {
                let bw = betweenness.get(node).copied().unwrap_or(0.0);
                let cc = closeness.get(node).copied().unwrap_or(0.0);
                let deg = degree.get(node).copied().unwrap_or(0.0);

                let norm_bw = bw / max_bw;
                let norm_cc = cc / max_cc;
                let norm_deg = deg / max_deg;

                let combined = (norm_bw + norm_cc + norm_deg) / 3.0;

                let role = if norm_deg > 0.7 {
                    HotspotRole::Hub
                } else if norm_bw > 0.5 {
                    HotspotRole::Bridge
                } else if norm_bw > 0.3 && norm_deg > 0.3 {
                    HotspotRole::Bottleneck
                } else {
                    HotspotRole::Peripheral
                };

                ArchitecturalHotspot {
                    node: node.clone(),
                    betweenness: bw,
                    closeness: cc,
                    degree: deg,
                    combined_score: combined,
                    role,
                }
            })
            .collect();

        hotspots.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        hotspots
    }

    fn identify_hubs(&self, degree: &HashMap<String, f64>, graph: &DirectedGraph) -> Vec<HubNode> {
        let mut hubs: Vec<HubNode> = degree
            .iter()
            .map(|(node, &deg)| HubNode {
                node: node.clone(),
                degree_centrality: deg,
                dependents: graph.predecessors.get(node).map(|p| p.len()).unwrap_or(0),
            })
            .collect();

        hubs.sort_by(|a, b| {
            b.degree_centrality
                .partial_cmp(&a.degree_centrality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        hubs.truncate(10);
        hubs
    }

    fn identify_bridges(
        &self,
        betweenness: &HashMap<String, f64>,
        graph: &DirectedGraph,
    ) -> Vec<BridgeNode> {
        let mut bridges: Vec<BridgeNode> = betweenness
            .iter()
            .map(|(node, &bw)| {
                let connects = graph.successors.get(node).map(|s| s.len()).unwrap_or(0)
                    + graph.predecessors.get(node).map(|p| p.len()).unwrap_or(0);
                BridgeNode {
                    node: node.clone(),
                    betweenness: bw,
                    connects_components: connects,
                }
            })
            .collect();

        bridges.sort_by(|a, b| {
            b.betweenness
                .partial_cmp(&a.betweenness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        bridges.truncate(10);
        bridges
    }
}

struct DirectedGraph {
    nodes: Vec<String>,
    successors: HashMap<String, HashSet<String>>,
    predecessors: HashMap<String, HashSet<String>>,
}

impl DirectedGraph {
    fn from_edges(nodes: &[String], edges: &[(String, String)]) -> Self {
        let mut graph = DirectedGraph {
            nodes: nodes.to_vec(),
            successors: HashMap::new(),
            predecessors: HashMap::new(),
        };

        for node in nodes {
            graph.successors.entry(node.clone()).or_default();
            graph.predecessors.entry(node.clone()).or_default();
        }

        for (source, target) in edges {
            graph
                .successors
                .entry(source.clone())
                .or_default()
                .insert(target.clone());
            graph
                .predecessors
                .entry(target.clone())
                .or_default()
                .insert(source.clone());
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let analyzer = CentralityAnalyzer::new();
        let result = analyzer.analyze(&[], &[]);
        assert!(result.betweenness.is_empty());
        assert!(result.hotspots.is_empty());
    }

    #[test]
    fn test_single_node() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let result = analyzer.analyze(&nodes, &[]);
        assert_eq!(result.betweenness.len(), 1);
        assert_eq!(result.betweenness["a"], 0.0);
    }

    #[test]
    fn test_two_nodes_connected() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![("a".to_string(), "b".to_string())];
        let result = analyzer.analyze(&nodes, &edges);

        assert_eq!(result.betweenness.len(), 2);
        assert!(result.closeness["a"] >= 0.0);
        assert!(result.degree["a"] > 0.0);
    }

    #[test]
    fn test_star_topology() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec![
            "center".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let edges = vec![
            ("center".to_string(), "a".to_string()),
            ("center".to_string(), "b".to_string()),
            ("center".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        let center_degree = result.degree["center"];
        let leaf_degree = result.degree["a"];
        assert!(center_degree > leaf_degree);
    }

    #[test]
    fn test_hub_identification() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec![
            "hub".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let edges = vec![
            ("hub".to_string(), "a".to_string()),
            ("hub".to_string(), "b".to_string()),
            ("hub".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        assert!(!result.top_hubs.is_empty());
        assert_eq!(result.top_hubs[0].node, "hub");
    }

    #[test]
    fn test_hotspot_roles() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec![
            "hub".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let edges = vec![
            ("hub".to_string(), "a".to_string()),
            ("hub".to_string(), "b".to_string()),
            ("hub".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        let hub_hotspot = result.hotspots.iter().find(|h| h.node == "hub").unwrap();
        assert_eq!(hub_hotspot.role, HotspotRole::Hub);
    }

    #[test]
    fn test_normalized_centrality() {
        let analyzer = CentralityAnalyzer::with_normalized(true);
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        for value in result.degree.values() {
            assert!(*value <= 1.0, "Normalized degree should be <= 1.0");
        }
        for value in result.betweenness.values() {
            assert!(*value <= 1.0, "Normalized betweenness should be <= 1.0");
        }
    }

    #[test]
    fn test_pagerank() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        assert_eq!(result.pagerank.len(), 3);
        for value in result.pagerank.values() {
            assert!(
                *value >= 0.0 && *value <= 1.0,
                "PageRank should be normalized"
            );
        }
    }

    #[test]
    fn test_eigenvector() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        assert_eq!(result.eigenvector.len(), 3);
        for value in result.eigenvector.values() {
            assert!(
                *value >= 0.0 && *value <= 1.0,
                "Eigenvector should be normalized"
            );
        }
    }

    #[test]
    fn test_bridge_betweenness() {
        let analyzer = CentralityAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        assert!(
            result.betweenness["b"] > 0.0,
            "Bridge node b should have non-zero betweenness, got {}",
            result.betweenness["b"]
        );
    }
}
