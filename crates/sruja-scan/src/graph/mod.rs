//! Inferred architecture graph schema.
//!
//! This is the minimal, repo-scoped graph used for deterministic diffing and review grounding.
//! NodeKind and EdgeKind are from sruja-language; scan only uses a subset of variants.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub use sruja_language::ast::{Criticality, EdgeKind, NodeKind, SourceBinding};

pub mod centrality;
pub use centrality::{compute_all_centrality, ComponentImportance};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    /// Canonical ID for cross-system reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_id: Option<String>,
    /// Aliases for this element
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Owner team or individual
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Business domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Criticality level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criticality: Option<Criticality>,
    /// Source bindings to external resources
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceBinding>,
    /// Gotchas/tribal knowledge about this element
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gotchas: Vec<String>,
    /// Operational constraints
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operational_constraints: Vec<String>,
    /// Paths to runbooks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runbooks: Vec<String>,
    pub confidence: Option<u8>,
    /// State machine definitions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state_machines: Vec<ResolvedStateMachine>,
    /// API contract definitions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contracts: Vec<ResolvedContract>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: NodeKind::Module,
            label: String::new(),
            technology: None,
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            gotchas: Vec::new(),
            operational_constraints: Vec::new(),
            runbooks: Vec::new(),
            confidence: None,
            state_machines: Vec::new(),
            contracts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeEvidence {
    pub rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EdgeEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// Operational incidents
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidents: Vec<Incident>,
    /// Overall graph discovery confidence (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected: Vec<String>, // FQNs of affected elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lesson: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlastRadiusDirection {
    Upstream,
    Downstream,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlastRadiusNode {
    pub id: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlastRadiusResult {
    pub target: String,
    pub max_depth: usize,
    pub upstream: Vec<BlastRadiusNode>,
    pub downstream: Vec<BlastRadiusNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedStateMachine {
    pub name: String,
    pub states: Vec<String>, // All unique states
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub transitions: Vec<ResolvedTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedTransition {
    pub from: String,
    pub to: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedContract {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<ResolvedField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<ResolvedField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ResolvedError>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedField {
    pub name: String,
    pub spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedError {
    pub code: String,
    pub description: String,
}

type Adjacency<'a> = HashMap<&'a str, Vec<&'a str>>;

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            incidents: Vec::new(),
            confidence: None,
        }
    }

    pub fn merge(&mut self, other: Graph) {
        self.nodes.extend(other.nodes);
        self.edges.extend(other.edges);
        self.incidents.extend(other.incidents);
        for (k, v) in other.metadata {
            self.metadata.entry(k).or_insert(v);
        }
        self.canonicalize();
    }

    pub fn canonicalize(&mut self) {
        fn node_kind_rank(kind: &NodeKind) -> u8 {
            match kind {
                NodeKind::System => 100,
                NodeKind::Service => 90,
                NodeKind::Container => 80,
                NodeKind::Component => 70,
                NodeKind::Frontend => 60,
                NodeKind::ExternalApi => 50,
                NodeKind::Database => 40,
                NodeKind::Queue => 30,
                NodeKind::Module => 10,
                NodeKind::Custom(_) => 10,
            }
        }

        fn merge_node(base: &mut Node, other: Node) {
            if node_kind_rank(&other.kind) > node_kind_rank(&base.kind) {
                base.kind = other.kind.clone();
            }
            if (base.label.is_empty() || base.label == base.id)
                && !other.label.is_empty()
                && other.label != other.id
            {
                base.label = other.label;
            }
            if base.technology.is_none() {
                base.technology = other.technology;
            }
            if base.path.is_none() {
                base.path = other.path;
            }
            for (k, v) in other.metadata {
                base.metadata.entry(k).or_insert(v);
            }

            // Merge architecture index fields
            if base.canonical_id.is_none() {
                base.canonical_id = other.canonical_id;
            }
            if !other.aliases.is_empty() {
                for alias in other.aliases {
                    if !base.aliases.contains(&alias) {
                        base.aliases.push(alias);
                    }
                }
            }
            if base.owner.is_none() {
                base.owner = other.owner;
            }
            if base.domain.is_none() {
                base.domain = other.domain;
            }
            if base.criticality.is_none() {
                base.criticality = other.criticality;
            }
            if !other.sources.is_empty() {
                for source in other.sources {
                    if !base.sources.contains(&source) {
                        base.sources.push(source);
                    }
                }
            }
            if !other.gotchas.is_empty() {
                for gotcha in other.gotchas {
                    if !base.gotchas.contains(&gotcha) {
                        base.gotchas.push(gotcha);
                    }
                }
            }
            if !other.operational_constraints.is_empty() {
                for constraint in other.operational_constraints {
                    if !base.operational_constraints.contains(&constraint) {
                        base.operational_constraints.push(constraint);
                    }
                }
            }
            if !other.runbooks.is_empty() {
                for runbook in other.runbooks {
                    if !base.runbooks.contains(&runbook) {
                        base.runbooks.push(runbook);
                    }
                }
            }
            if base.confidence.is_none() {
                base.confidence = other.confidence;
            }
        }

        let mut nodes_by_id: BTreeMap<String, Node> = BTreeMap::new();
        self.nodes.sort_by(|a, b| {
            (a.id.as_str(), a.kind.kind_str(), a.label.as_str()).cmp(&(
                b.id.as_str(),
                b.kind.kind_str(),
                b.label.as_str(),
            ))
        });
        for node in self.nodes.drain(..) {
            nodes_by_id
                .entry(node.id.clone())
                .and_modify(|existing| merge_node(existing, node.clone()))
                .or_insert(node);
        }
        self.nodes = nodes_by_id.into_values().collect();

        let node_set: HashSet<String> = self.nodes.iter().map(|n| n.id.clone()).collect();

        fn evidence_sort_key(
            e: &EdgeEvidence,
        ) -> (&str, &Option<String>, &Option<u32>, &Option<String>) {
            (e.rule.as_str(), &e.file, &e.line, &e.detail)
        }

        let mut merged: BTreeMap<(String, String, String), Vec<EdgeEvidence>> = BTreeMap::new();
        for mut edge in self.edges.drain(..) {
            if !node_set.contains(&edge.source) || !node_set.contains(&edge.target) {
                continue;
            }
            edge.evidence
                .sort_by(|a, b| evidence_sort_key(a).cmp(&evidence_sort_key(b)));
            edge.evidence.dedup();

            let key = (
                edge.source.clone(),
                edge.target.clone(),
                edge.kind.as_str().to_string(),
            );
            merged.entry(key).or_default().extend(edge.evidence);
        }

        self.edges = Vec::with_capacity(merged.len());
        for ((source, target, kind_str), mut evidence) in merged {
            let Ok(kind) = kind_str.parse::<EdgeKind>() else {
                continue;
            };
            evidence.sort_by(|a, b| evidence_sort_key(a).cmp(&evidence_sort_key(b)));
            evidence.dedup();
            self.edges.push(Edge {
                source,
                target,
                kind,
                evidence,
            });
        }

        self.incidents.sort_by(|a, b| a.id.cmp(&b.id));
        self.incidents.dedup_by(|a, b| a.id == b.id);
    }

    #[must_use]
    pub fn blast_radius(&self, target_id: &str, max_depth: usize) -> BlastRadiusResult {
        let (outgoing, incoming) = self.build_adjacency();
        let downstream = self.walk_blast_radius(&outgoing, target_id, max_depth);
        let upstream = self.walk_blast_radius(&incoming, target_id, max_depth);

        BlastRadiusResult {
            target: target_id.to_string(),
            max_depth,
            upstream,
            downstream,
        }
    }

    fn build_adjacency(&self) -> (Adjacency<'_>, Adjacency<'_>) {
        let mut outgoing: Adjacency<'_> = HashMap::new();
        let mut incoming: Adjacency<'_> = HashMap::new();

        for edge in &self.edges {
            outgoing
                .entry(edge.source.as_str())
                .or_default()
                .push(edge.target.as_str());
            incoming
                .entry(edge.target.as_str())
                .or_default()
                .push(edge.source.as_str());
        }

        for targets in outgoing.values_mut() {
            targets.sort_unstable();
            targets.dedup();
        }
        for sources in incoming.values_mut() {
            sources.sort_unstable();
            sources.dedup();
        }

        (outgoing, incoming)
    }

    fn walk_blast_radius(
        &self,
        adjacency: &HashMap<&str, Vec<&str>>,
        target_id: &str,
        max_depth: usize,
    ) -> Vec<BlastRadiusNode> {
        if max_depth == 0 {
            return Vec::new();
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
        let mut out: Vec<BlastRadiusNode> = Vec::new();

        visited.insert(target_id);
        queue.push_back((target_id, 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(neighbors) = adjacency.get(current) {
                for &next in neighbors {
                    if visited.insert(next) {
                        let next_depth = depth + 1;
                        out.push(BlastRadiusNode {
                            id: next.to_string(),
                            depth: next_depth,
                        });
                        queue.push_back((next, next_depth));
                    }
                }
            }
        }

        out.sort_by(|a, b| (a.depth, a.id.as_str()).cmp(&(b.depth, b.id.as_str())));
        out
    }

    pub fn find_path(&self, source_id: &str, target_id: &str) -> Option<Vec<String>> {
        let (outgoing, _) = self.build_adjacency();
        if !outgoing.contains_key(source_id) && !self.nodes.iter().any(|n| n.id == source_id) {
            return None;
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<(&str, Vec<String>)> = VecDeque::new();

        visited.insert(source_id);
        queue.push_back((source_id, vec![source_id.to_string()]));

        while let Some((current, path)) = queue.pop_front() {
            if current == target_id {
                return Some(path);
            }

            if let Some(neighbors) = outgoing.get(current) {
                for &next in neighbors {
                    if visited.insert(next) {
                        let mut next_path = path.clone();
                        next_path.push(next.to_string());
                        queue.push_back((next, next_path));
                    }
                }
            }
        }

        None
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> Node {
        let mut node = Node::default();
        node.id = id.to_string();
        node.label = id.to_string();
        node
    }

    fn edge(source: &str, target: &str) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind: EdgeKind::Calls,
            evidence: Vec::new(),
        }
    }

    #[test]
    fn canonicalize_drops_dangling_edges() {
        let mut graph = Graph::default();
        graph.nodes = vec![node("a")];
        graph.edges = vec![edge("a", "missing")];

        graph.canonicalize();
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn canonicalize_merges_duplicate_edges_and_evidence() {
        let mut graph = Graph::default();
        graph.nodes = vec![node("a"), node("b")];
        graph.edges = vec![
            Edge {
                source: "a".into(),
                target: "b".into(),
                kind: EdgeKind::Calls,
                evidence: vec![EdgeEvidence {
                    rule: "r2".into(),
                    file: None,
                    line: None,
                    detail: None,
                }],
            },
            Edge {
                source: "a".into(),
                target: "b".into(),
                kind: EdgeKind::Calls,
                evidence: vec![
                    EdgeEvidence {
                        rule: "r1".into(),
                        file: None,
                        line: None,
                        detail: None,
                    },
                    EdgeEvidence {
                        rule: "r2".into(),
                        file: None,
                        line: None,
                        detail: None,
                    },
                ],
            },
        ];

        graph.canonicalize();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].evidence.len(), 2);
        assert_eq!(graph.edges[0].evidence[0].rule, "r1");
        assert_eq!(graph.edges[0].evidence[1].rule, "r2");
    }

    #[test]
    fn blast_radius_returns_upstream_and_downstream() {
        let mut graph = Graph::default();
        graph.nodes = vec![node("a"), node("b"), node("c"), node("d")];
        graph.edges = vec![edge("a", "b"), edge("b", "c"), edge("d", "b")];

        let res = graph.blast_radius("b", 2);
        assert_eq!(res.target, "b");

        assert_eq!(
            res.downstream,
            vec![BlastRadiusNode {
                id: "c".to_string(),
                depth: 1
            }]
        );
        assert_eq!(
            res.upstream,
            vec![
                BlastRadiusNode {
                    id: "a".to_string(),
                    depth: 1
                },
                BlastRadiusNode {
                    id: "d".to_string(),
                    depth: 1
                }
            ]
        );
    }

    #[test]
    fn blast_radius_depth_zero_is_empty() {
        let mut graph = Graph::default();
        graph.nodes = vec![node("a"), node("b")];
        graph.edges = vec![edge("a", "b")];

        let res = graph.blast_radius("a", 0);
        assert!(res.upstream.is_empty());
        assert!(res.downstream.is_empty());
    }

    #[test]
    fn find_path_returns_correct_sequence() {
        let mut graph = Graph::default();
        graph.nodes = vec![node("a"), node("b"), node("c"), node("d")];
        graph.edges = vec![
            edge("a", "b"),
            edge("b", "c"),
            edge("c", "d"),
            edge("a", "c"),
        ];

        let path = graph.find_path("a", "d").expect("path should exist");
        assert_eq!(path, vec!["a", "c", "d"]);

        let no_path = graph.find_path("d", "a");
        assert!(no_path.is_none());
    }
}
