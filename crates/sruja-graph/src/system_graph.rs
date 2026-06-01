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
