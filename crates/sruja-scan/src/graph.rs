//! Inferred architecture graph schema.
//!
//! This is the minimal, repo-scoped graph used for deterministic diffing and review grounding.
//! NodeKind and EdgeKind are from sruja_types; scan only uses a subset of variants.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub use sruja_types::{EdgeKind, NodeKind};

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

type Adjacency<'a> = HashMap<&'a str, Vec<&'a str>>;

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn canonicalize(&mut self) {
        fn node_kind_rank(kind: NodeKind) -> u8 {
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
            }
        }

        fn merge_node(base: &mut Node, other: Node) {
            if node_kind_rank(other.kind) > node_kind_rank(base.kind) {
                base.kind = other.kind;
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
        }

        let mut nodes_by_id: BTreeMap<String, Node> = BTreeMap::new();
        self.nodes.sort_by(|a, b| {
            (a.id.as_str(), a.kind.as_str(), a.label.as_str()).cmp(&(
                b.id.as_str(),
                b.kind.as_str(),
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
        Node {
            id: id.to_string(),
            kind: NodeKind::Module,
            label: id.to_string(),
            technology: None,
            path: None,
            metadata: HashMap::new(),
        }
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
        let mut graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("a")],
            edges: vec![edge("a", "missing")],
        };

        graph.canonicalize();
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn canonicalize_merges_duplicate_edges_and_evidence() {
        let mut graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("a"), node("b")],
            edges: vec![
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
            ],
        };

        graph.canonicalize();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].evidence.len(), 2);
        assert_eq!(graph.edges[0].evidence[0].rule, "r1");
        assert_eq!(graph.edges[0].evidence[1].rule, "r2");
    }

    #[test]
    fn blast_radius_returns_upstream_and_downstream() {
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("a"), node("b"), node("c"), node("d")],
            edges: vec![edge("a", "b"), edge("b", "c"), edge("d", "b")],
        };

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
        let graph = Graph {
            metadata: HashMap::new(),
            nodes: vec![node("a"), node("b")],
            edges: vec![edge("a", "b")],
        };

        let res = graph.blast_radius("a", 0);
        assert!(res.upstream.is_empty());
        assert!(res.downstream.is_empty());
    }
}
