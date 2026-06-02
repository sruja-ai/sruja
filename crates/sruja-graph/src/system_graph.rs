use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNode {
    pub canonical_id: String,
    pub kind: String,
    pub label: String,
    pub technology: Option<String>,
    pub repo_id: String,
    pub local_id: String,
    pub owner: Option<String>,
    pub domain: Option<String>,
    pub criticality: Option<String>,
    pub aliases: Vec<String>,
    pub gotchas: Vec<String>,
    pub operational_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub label: Option<String>,
    pub repo_id: String,
    pub confidence: EdgeConfidence,
    pub is_cross_repo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeConfidence {
    #[default]
    Extracted,
    Inferred,
    Ambiguous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRepo {
    pub repo_id: String,
    pub repo_path: String,
    pub truth_status: String,
    pub git_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemGraph {
    pub repos: Vec<SystemRepo>,
    pub nodes: HashMap<String, SystemNode>,
    pub edges: Vec<SystemEdge>,
    outgoing: HashMap<String, Vec<String>>,
    incoming: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    pub target: String,
    pub max_depth: usize,
    pub upstream: Vec<BlastRadiusHit>,
    pub downstream: Vec<BlastRadiusHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusHit {
    pub canonical_id: String,
    pub depth: usize,
    pub repo_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceHop {
    pub node_id: String,
    pub label: String,
    pub kind: String,
    pub repo_id: String,
    pub technology: Option<String>,
    pub edge_kind: Option<String>,
    pub edge_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResult {
    pub query: String,
    pub hops: Vec<TraceHop>,
    pub repos_touched: usize,
    pub teams: HashSet<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHubNode {
    pub canonical_id: String,
    pub label: String,
    pub repo_id: String,
    pub edge_count: usize,
    pub is_cross_repo: bool,
}

impl SystemGraph {
    pub fn new(repos: Vec<SystemRepo>, nodes: Vec<SystemNode>, edges: Vec<SystemEdge>) -> Self {
        let node_map: HashMap<String, SystemNode> = nodes
            .into_iter()
            .map(|n| (n.canonical_id.clone(), n))
            .collect();

        let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();

        for edge in &edges {
            outgoing
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            incoming
                .entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }

        for targets in outgoing.values_mut() {
            targets.sort_unstable();
            targets.dedup();
        }
        for sources in incoming.values_mut() {
            sources.sort_unstable();
            sources.dedup();
        }

        Self {
            repos,
            nodes: node_map,
            edges,
            outgoing,
            incoming,
        }
    }

    pub fn get_node(&self, id: &str) -> Option<&SystemNode> {
        self.nodes.get(id)
    }

    pub fn neighbors_out(&self, id: &str) -> &[String] {
        self.outgoing.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn neighbors_in(&self, id: &str) -> &[String] {
        self.incoming.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn blast_radius(&self, target_id: &str, max_depth: usize) -> BlastRadius {
        let downstream = self.walk(&self.outgoing, target_id, max_depth);
        let upstream = self.walk(&self.incoming, target_id, max_depth);

        BlastRadius {
            target: target_id.to_string(),
            max_depth,
            upstream,
            downstream,
        }
    }

    pub fn find_path(&self, source_id: &str, target_id: &str) -> Option<Vec<String>> {
        if !self.nodes.contains_key(source_id) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(source_id);
        queue.push_back((source_id, vec![source_id.to_string()]));

        while let Some((current, path)) = queue.pop_front() {
            if current == target_id {
                return Some(path);
            }

            if let Some(neighbors) = self.outgoing.get(current) {
                for next in neighbors {
                    if visited.insert(next) {
                        let mut next_path = path.clone();
                        next_path.push(next.clone());
                        queue.push_back((next, next_path));
                    }
                }
            }
        }

        None
    }

    pub fn entrypoints(&self) -> Vec<&SystemNode> {
        self.nodes
            .values()
            .filter(|n| {
                matches!(n.kind.as_str(), "service" | "frontend" | "container")
                    && self
                        .incoming
                        .get(&n.canonical_id)
                        .is_none_or(|v| v.is_empty())
            })
            .collect()
    }

    pub fn data_stores(&self) -> Vec<&SystemNode> {
        self.nodes
            .values()
            .filter(|n| matches!(n.kind.as_str(), "database" | "queue"))
            .collect()
    }

    pub fn hubs(&self, min_edges: usize) -> Vec<SystemHubNode> {
        self.nodes
            .values()
            .filter_map(|n| {
                let out_count = self.outgoing.get(&n.canonical_id).map_or(0, |v| v.len());
                let in_count = self.incoming.get(&n.canonical_id).map_or(0, |v| v.len());
                let total = out_count + in_count;

                if total >= min_edges {
                    let is_cross_repo = self.edges.iter().any(|e| {
                        (e.source == n.canonical_id || e.target == n.canonical_id)
                            && e.is_cross_repo
                    });
                    Some(SystemHubNode {
                        canonical_id: n.canonical_id.clone(),
                        label: n.label.clone(),
                        repo_id: n.repo_id.clone(),
                        edge_count: total,
                        is_cross_repo,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn teams(&self) -> HashMap<String, Vec<&SystemNode>> {
        let mut teams: HashMap<String, Vec<&SystemNode>> = HashMap::new();
        for node in self.nodes.values() {
            let team = node.owner.as_deref().unwrap_or("unowned");
            teams.entry(team.to_string()).or_default().push(node);
        }
        teams
    }

    pub fn by_repo(&self) -> HashMap<&str, Vec<&SystemNode>> {
        let mut by_repo: HashMap<&str, Vec<&SystemNode>> = HashMap::new();
        for node in self.nodes.values() {
            by_repo.entry(&node.repo_id).or_default().push(node);
        }
        by_repo
    }

    pub fn trace_flow(&self, start_id: &str, max_depth: usize) -> TraceResult {
        let mut hops = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut repos_touched: HashSet<String> = HashSet::new();
        let mut teams: HashSet<String> = HashSet::new();
        let mut warnings = Vec::new();

        if let Some(start_node) = self.nodes.get(start_id) {
            hops.push(TraceHop {
                node_id: start_node.canonical_id.clone(),
                label: start_node.label.clone(),
                kind: start_node.kind.clone(),
                repo_id: start_node.repo_id.clone(),
                technology: start_node.technology.clone(),
                edge_kind: None,
                edge_label: None,
            });
            repos_touched.insert(start_node.repo_id.clone());
            if let Some(ref owner) = start_node.owner {
                teams.insert(owner.clone());
            }
        } else {
            warnings.push(format!("Start node '{}' not found", start_id));
            return TraceResult {
                query: start_id.to_string(),
                hops,
                repos_touched: 0,
                teams,
                warnings,
            };
        }

        visited.insert(start_id.to_string());
        queue.push_back((start_id.to_string(), 0usize));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(neighbors) = self.outgoing.get(&current) {
                for next in neighbors {
                    if visited.insert(next.clone()) {
                        let edge = self
                            .edges
                            .iter()
                            .find(|e| e.source == current && e.target == *next);

                        if let Some(next_node) = self.nodes.get(next) {
                            hops.push(TraceHop {
                                node_id: next.clone(),
                                label: next_node.label.clone(),
                                kind: next_node.kind.clone(),
                                repo_id: next_node.repo_id.clone(),
                                technology: next_node.technology.clone(),
                                edge_kind: edge.map(|e| e.kind.clone()),
                                edge_label: edge.and_then(|e| e.label.clone()),
                            });
                            repos_touched.insert(next_node.repo_id.clone());
                            if let Some(ref owner) = next_node.owner {
                                teams.insert(owner.clone());
                            }

                            if let Some(incoming) = self.incoming.get(next) {
                                if incoming.len() == 1
                                    && matches!(next_node.kind.as_str(), "service" | "container")
                                {
                                    warnings.push(format!(
                                        "SPOF: {} (single incoming path)",
                                        next_node.label
                                    ));
                                }
                            }
                        }

                        queue.push_back((next.clone(), depth + 1));
                    }
                }
            }
        }

        TraceResult {
            query: start_id.to_string(),
            hops,
            repos_touched: repos_touched.len(),
            teams,
            warnings,
        }
    }

    pub fn resolve_entity(&self, query: &str) -> Option<String> {
        let q_lower = query.to_lowercase();

        if self.nodes.contains_key(query) {
            return Some(query.to_string());
        }

        for node in self.nodes.values() {
            if node.label.to_lowercase() == q_lower {
                return Some(node.canonical_id.clone());
            }
        }

        for node in self.nodes.values() {
            if node.aliases.iter().any(|a| a.to_lowercase() == q_lower) {
                return Some(node.canonical_id.clone());
            }
        }

        for node in self.nodes.values() {
            if node.label.to_lowercase().contains(&q_lower)
                || node.local_id.to_lowercase().contains(&q_lower)
            {
                return Some(node.canonical_id.clone());
            }
        }

        None
    }

    fn walk(
        &self,
        adjacency: &HashMap<String, Vec<String>>,
        start: &str,
        max_depth: usize,
    ) -> Vec<BlastRadiusHit> {
        if max_depth == 0 {
            return Vec::new();
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(start);
        queue.push_back((start, 0usize));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(neighbors) = adjacency.get(current) {
                for next in neighbors {
                    if visited.insert(next) {
                        let repo_id = self
                            .nodes
                            .get(next)
                            .map(|n| n.repo_id.clone())
                            .unwrap_or_default();
                        result.push(BlastRadiusHit {
                            canonical_id: next.clone(),
                            depth: depth + 1,
                            repo_id,
                        });
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }

        result.sort_by(|a, b| {
            a.depth
                .cmp(&b.depth)
                .then_with(|| a.canonical_id.cmp(&b.canonical_id))
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, kind: &str, label: &str, repo_id: &str) -> SystemNode {
        SystemNode {
            canonical_id: id.to_string(),
            kind: kind.to_string(),
            label: label.to_string(),
            technology: None,
            repo_id: repo_id.to_string(),
            local_id: id.to_string(),
            owner: None,
            domain: None,
            criticality: None,
            aliases: Vec::new(),
            gotchas: Vec::new(),
            operational_constraints: Vec::new(),
        }
    }

    fn create_test_edge(source: &str, target: &str, kind: &str) -> SystemEdge {
        SystemEdge {
            source: source.to_string(),
            target: target.to_string(),
            kind: kind.to_string(),
            label: None,
            repo_id: "test-repo".to_string(),
            confidence: EdgeConfidence::Extracted,
            is_cross_repo: false,
        }
    }

    fn create_test_graph() -> SystemGraph {
        let repos = vec![SystemRepo {
            repo_id: "test-repo".to_string(),
            repo_path: "/test".to_string(),
            truth_status: "verified".to_string(),
            git_commit: Some("abc123".to_string()),
        }];

        let nodes = vec![
            create_test_node("frontend", "service", "Frontend", "test-repo"),
            create_test_node("api", "container", "API Service", "test-repo"),
            create_test_node("db", "database", "Database", "test-repo"),
            create_test_node("auth", "service", "Auth Service", "test-repo"),
        ];

        let edges = vec![
            create_test_edge("frontend", "api", "calls"),
            create_test_edge("api", "db", "reads"),
            create_test_edge("api", "auth", "calls"),
        ];

        SystemGraph::new(repos, nodes, edges)
    }

    #[test]
    fn test_system_graph_new() {
        let graph = create_test_graph();
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
        assert_eq!(graph.repos.len(), 1);
    }

    #[test]
    fn test_get_node() {
        let graph = create_test_graph();
        assert!(graph.get_node("frontend").is_some());
        assert!(graph.get_node("nonexistent").is_none());
        assert_eq!(graph.get_node("frontend").unwrap().label, "Frontend");
    }

    #[test]
    fn test_neighbors_out() {
        let graph = create_test_graph();
        let api_neighbors = graph.neighbors_out("api");
        assert_eq!(api_neighbors.len(), 2);
        assert!(api_neighbors.contains(&"db".to_string()));
        assert!(api_neighbors.contains(&"auth".to_string()));

        let db_neighbors = graph.neighbors_out("db");
        assert!(db_neighbors.is_empty());

        let nonexistent = graph.neighbors_out("nonexistent");
        assert!(nonexistent.is_empty());
    }

    #[test]
    fn test_neighbors_in() {
        let graph = create_test_graph();
        let api_incoming = graph.neighbors_in("api");
        assert_eq!(api_incoming.len(), 1);
        assert!(api_incoming.contains(&"frontend".to_string()));

        let frontend_incoming = graph.neighbors_in("frontend");
        assert!(frontend_incoming.is_empty());
    }

    #[test]
    fn test_blast_radius() {
        let graph = create_test_graph();
        let radius = graph.blast_radius("api", 2);

        assert_eq!(radius.target, "api");
        assert_eq!(radius.max_depth, 2);
        assert_eq!(radius.downstream.len(), 2);
        assert_eq!(radius.upstream.len(), 1);

        assert!(radius.downstream.iter().any(|h| h.canonical_id == "db"));
        assert!(radius.downstream.iter().any(|h| h.canonical_id == "auth"));
        assert!(radius.upstream.iter().any(|h| h.canonical_id == "frontend"));
    }

    #[test]
    fn test_blast_radius_zero_depth() {
        let graph = create_test_graph();
        let radius = graph.blast_radius("api", 0);
        assert!(radius.downstream.is_empty());
        assert!(radius.upstream.is_empty());
    }

    #[test]
    fn test_find_path() {
        let graph = create_test_graph();
        let path = graph.find_path("frontend", "db");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path[0], "frontend");
        assert_eq!(path[1], "api");
        assert_eq!(path[2], "db");
    }

    #[test]
    fn test_find_path_no_path() {
        let graph = create_test_graph();
        let path = graph.find_path("db", "frontend");
        assert!(path.is_none());
    }

    #[test]
    fn test_find_path_nonexistent_source() {
        let graph = create_test_graph();
        let path = graph.find_path("nonexistent", "db");
        assert!(path.is_none());
    }

    #[test]
    fn test_find_path_same_node() {
        let graph = create_test_graph();
        let path = graph.find_path("api", "api");
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_entrypoints() {
        let graph = create_test_graph();
        let entrypoints = graph.entrypoints();
        assert_eq!(entrypoints.len(), 1);
        assert_eq!(entrypoints[0].canonical_id, "frontend");
    }

    #[test]
    fn test_data_stores() {
        let graph = create_test_graph();
        let stores = graph.data_stores();
        assert_eq!(stores.len(), 1);
        assert_eq!(stores[0].canonical_id, "db");
    }

    #[test]
    fn test_hubs() {
        let graph = create_test_graph();
        let hubs = graph.hubs(2);
        assert!(hubs.iter().any(|h| h.canonical_id == "api"));

        let no_hubs = graph.hubs(10);
        assert!(no_hubs.is_empty());
    }

    #[test]
    fn test_teams() {
        let mut graph = create_test_graph();
        graph.nodes.get_mut("api").unwrap().owner = Some("platform-team".to_string());
        graph.nodes.get_mut("db").unwrap().owner = Some("data-team".to_string());

        let teams = graph.teams();
        assert!(teams.contains_key("platform-team"));
        assert!(teams.contains_key("data-team"));
        assert!(teams.contains_key("unowned"));
    }

    #[test]
    fn test_by_repo() {
        let graph = create_test_graph();
        let by_repo = graph.by_repo();
        assert!(by_repo.contains_key("test-repo"));
        assert_eq!(by_repo.get("test-repo").unwrap().len(), 4);
    }

    #[test]
    fn test_trace_flow() {
        let graph = create_test_graph();
        let result = graph.trace_flow("frontend", 3);

        assert_eq!(result.query, "frontend");
        assert!(!result.hops.is_empty());
        assert!(result.hops[0].node_id == "frontend");
        assert!(result.repos_touched >= 1);
    }

    #[test]
    fn test_trace_flow_nonexistent() {
        let graph = create_test_graph();
        let result = graph.trace_flow("nonexistent", 3);

        assert_eq!(result.query, "nonexistent");
        assert!(result.hops.is_empty());
        assert_eq!(result.repos_touched, 0);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_trace_flow_spof_detection() {
        let graph = create_test_graph();
        let result = graph.trace_flow("frontend", 2);

        let spof_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("SPOF"))
            .collect();
        assert!(!spof_warnings.is_empty());
    }

    #[test]
    fn test_resolve_entity_by_id() {
        let graph = create_test_graph();
        let result = graph.resolve_entity("api");
        assert_eq!(result, Some("api".to_string()));
    }

    #[test]
    fn test_resolve_entity_by_label() {
        let graph = create_test_graph();
        let result = graph.resolve_entity("API Service");
        assert_eq!(result, Some("api".to_string()));
    }

    #[test]
    fn test_resolve_entity_case_insensitive() {
        let graph = create_test_graph();
        let result = graph.resolve_entity("api service");
        assert_eq!(result, Some("api".to_string()));
    }

    #[test]
    fn test_resolve_entity_partial_match() {
        let graph = create_test_graph();
        let result = graph.resolve_entity("front");
        assert_eq!(result, Some("frontend".to_string()));
    }

    #[test]
    fn test_resolve_entity_not_found() {
        let graph = create_test_graph();
        let result = graph.resolve_entity("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_entity_with_aliases() {
        let mut graph = create_test_graph();
        graph
            .nodes
            .get_mut("api")
            .unwrap()
            .aliases
            .push("backend".to_string());

        let result = graph.resolve_entity("backend");
        assert_eq!(result, Some("api".to_string()));
    }

    #[test]
    fn test_edge_confidence_default() {
        let confidence = EdgeConfidence::default();
        assert_eq!(confidence, EdgeConfidence::Extracted);
    }

    #[test]
    fn test_edge_confidence_variants() {
        assert_ne!(EdgeConfidence::Extracted, EdgeConfidence::Inferred);
        assert_ne!(EdgeConfidence::Extracted, EdgeConfidence::Ambiguous);
        assert_ne!(EdgeConfidence::Inferred, EdgeConfidence::Ambiguous);
    }

    #[test]
    fn test_system_graph_with_cross_repo_edges() {
        let repos = vec![
            SystemRepo {
                repo_id: "repo1".to_string(),
                repo_path: "/repo1".to_string(),
                truth_status: "verified".to_string(),
                git_commit: None,
            },
            SystemRepo {
                repo_id: "repo2".to_string(),
                repo_path: "/repo2".to_string(),
                truth_status: "verified".to_string(),
                git_commit: None,
            },
        ];

        let nodes = vec![
            create_test_node("svc1", "service", "Service 1", "repo1"),
            create_test_node("svc2", "service", "Service 2", "repo2"),
        ];

        let edges = vec![SystemEdge {
            source: "svc1".to_string(),
            target: "svc2".to_string(),
            kind: "calls".to_string(),
            label: Some("API call".to_string()),
            repo_id: "repo1".to_string(),
            confidence: EdgeConfidence::Inferred,
            is_cross_repo: true,
        }];

        let graph = SystemGraph::new(repos, nodes, edges);

        let hubs = graph.hubs(1);
        assert!(hubs.iter().any(|h| h.is_cross_repo));
    }

    #[test]
    fn test_system_graph_dedup_edges() {
        let repos = vec![];
        let nodes = vec![
            create_test_node("a", "service", "A", "repo"),
            create_test_node("b", "service", "B", "repo"),
        ];

        let edges = vec![
            create_test_edge("a", "b", "calls"),
            create_test_edge("a", "b", "calls"),
        ];

        let graph = SystemGraph::new(repos, nodes, edges);
        let neighbors = graph.neighbors_out("a");
        assert_eq!(neighbors.len(), 1);
    }

    #[test]
    fn test_blast_radius_sorting() {
        let repos = vec![];
        let nodes = vec![
            create_test_node("root", "service", "Root", "repo"),
            create_test_node("child1", "service", "Child 1", "repo"),
            create_test_node("child2", "service", "Child 2", "repo"),
            create_test_node("grandchild", "service", "Grandchild", "repo"),
        ];

        let edges = vec![
            create_test_edge("root", "child1", "calls"),
            create_test_edge("root", "child2", "calls"),
            create_test_edge("child1", "grandchild", "calls"),
        ];

        let graph = SystemGraph::new(repos, nodes, edges);
        let radius = graph.blast_radius("root", 3);

        assert_eq!(radius.downstream[0].depth, 1);
        assert_eq!(radius.downstream[1].depth, 1);
        assert_eq!(radius.downstream[2].depth, 2);
    }
}
