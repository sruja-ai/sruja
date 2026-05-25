//! Interactive Architecture Explorer data model.
//!
//! Defines the `ExplorerModel` JSON contract consumed by the VS Code
//! webview (D3.js renderer).  The builder composes data from the
//! knowledge graph and optional analytical overlays (coupling,
//! centrality, SCC, communities, drift) into a single serializable
//! structure.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use sruja_graph::{
    CentralityResult, CouplingResult, KnowledgeGraph, SccResult, Zone,
};
use sruja_scan::graph::community::CommunityInfo;

// ---------------------------------------------------------------------------
// Model types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerModel {
    pub schema_version: String,
    pub nodes: Vec<ExplorerNode>,
    pub edges: Vec<ExplorerEdge>,
    pub communities: Vec<ExpCommunity>,
    pub cycles: Vec<CycleGroup>,
    pub summary: ExplorerSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub children_count: usize,
    pub metrics: NodeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub centrality: f64,
    pub instability: f64,
    pub coupling_zone: String,
    pub drift_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_severity_max: Option<String>,
    pub health: String,
    pub is_hotspot: bool,
    pub is_in_cycle: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub community_id: Option<u32>,
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self {
            centrality: 0.0,
            instability: 0.0,
            coupling_zone: "unknown".into(),
            drift_count: 0,
            drift_severity_max: None,
            health: "healthy".into(),
            is_hotspot: false,
            is_in_cycle: false,
            community_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub is_cycle_edge: bool,
    pub has_drift: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpCommunity {
    pub id: u32,
    pub label: String,
    pub member_ids: Vec<String>,
    pub cohesion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleGroup {
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub drift_score: u8,
    pub health: String,
    pub hotspot_count: usize,
    pub cycle_count: usize,
    pub community_count: usize,
}

// ---------------------------------------------------------------------------
// Lightweight drift overlay (avoids depending on sruja-intent)
// ---------------------------------------------------------------------------

/// Per-node drift info injected by the CLI layer.
#[derive(Debug, Clone)]
pub struct NodeDriftInfo {
    pub count: usize,
    pub severity_max: Option<String>,
    pub health: String,
}

/// Edge-level drift flag.
#[derive(Debug, Clone)]
pub struct EdgeDriftInfo {
    pub source: String,
    pub target: String,
}

/// Aggregate drift summary injected by the caller.
#[derive(Debug, Clone, Default)]
pub struct DriftOverlay {
    pub score: u8,
    pub health: String,
    pub nodes: HashMap<String, NodeDriftInfo>,
    pub edges: Vec<EdgeDriftInfo>,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

pub struct ExplorerBuilder {
    graph: KnowledgeGraph,
    coupling: Option<CouplingResult>,
    centrality: Option<CentralityResult>,
    scc: Option<SccResult>,
    communities: Option<Vec<CommunityInfo>>,
    drift: Option<DriftOverlay>,
}

impl ExplorerBuilder {
    pub fn new(graph: KnowledgeGraph) -> Self {
        Self {
            graph,
            coupling: None,
            centrality: None,
            scc: None,
            communities: None,
            drift: None,
        }
    }

    pub fn coupling(mut self, c: CouplingResult) -> Self {
        self.coupling = Some(c);
        self
    }

    pub fn centrality(mut self, c: CentralityResult) -> Self {
        self.centrality = Some(c);
        self
    }

    pub fn scc(mut self, s: SccResult) -> Self {
        self.scc = Some(s);
        self
    }

    pub fn communities(mut self, c: Vec<CommunityInfo>) -> Self {
        self.communities = Some(c);
        self
    }

    pub fn drift(mut self, d: DriftOverlay) -> Self {
        self.drift = Some(d);
        self
    }

    pub fn build(self) -> ExplorerModel {
        let hotspot_ids: HashSet<String> = self
            .centrality
            .as_ref()
            .map(|c| c.hotspots.iter().map(|h| h.node.clone()).collect())
            .unwrap_or_default();

        let cycle_node_set: HashSet<String> = self
            .scc
            .as_ref()
            .map(|s| {
                s.components
                    .iter()
                    .filter(|c| c.is_cyclic)
                    .flat_map(|c| c.nodes.iter().cloned())
                    .collect()
            })
            .unwrap_or_default();

        let cycle_edge_set: HashSet<(String, String)> = self
            .scc
            .as_ref()
            .map(|s| build_cycle_edge_set(s, &self.graph))
            .unwrap_or_default();

        let community_map: HashMap<String, u32> = self
            .communities
            .as_ref()
            .map(|cs| {
                cs.iter()
                    .flat_map(|c| c.members.iter().map(move |m| (m.clone(), c.id)))
                    .collect()
            })
            .unwrap_or_default();

        // Children count: count nodes whose id starts with `parent.`
        let all_ids: Vec<String> = self.graph.nodes.keys().cloned().collect();
        let children_count_map: HashMap<String, usize> = {
            let mut m: HashMap<String, usize> = HashMap::new();
            for id in &all_ids {
                if let Some(dot_pos) = id.rfind('.') {
                    let parent = &id[..dot_pos];
                    *m.entry(parent.to_string()).or_default() += 1;
                }
            }
            m
        };

        let parent_id_map: HashMap<String, String> = {
            let mut m = HashMap::new();
            for id in &all_ids {
                if let Some(dot_pos) = id.rfind('.') {
                    let parent = id[..dot_pos].to_string();
                    if self.graph.nodes.contains_key(&parent) {
                        m.insert(id.clone(), parent);
                    }
                }
            }
            m
        };

        let drift_edge_set: HashSet<(String, String)> = self
            .drift
            .as_ref()
            .map(|d| {
                d.edges
                    .iter()
                    .map(|e| (e.source.clone(), e.target.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let mut nodes: Vec<ExplorerNode> = self
            .graph
            .nodes
            .values()
            .map(|n| {
                let mut metrics = NodeMetrics::default();

                if let Some(ref cent) = self.centrality {
                    let b = cent.betweenness.get(&n.id).copied().unwrap_or(0.0);
                    let c = cent.closeness.get(&n.id).copied().unwrap_or(0.0);
                    let d = cent.degree.get(&n.id).copied().unwrap_or(0.0);
                    metrics.centrality = (b + c + d) / 3.0;
                }

                if let Some(ref coup) = self.coupling {
                    metrics.instability =
                        coup.instability.get(&n.id).copied().unwrap_or(0.0);
                    let abs = coup.abstractness.get(&n.id).copied().unwrap_or(0.0);
                    let dist = coup.distance.get(&n.id).copied().unwrap_or(0.0);
                    metrics.coupling_zone = classify_zone(metrics.instability, abs, dist);
                }

                if let Some(ref drift) = self.drift {
                    if let Some(info) = drift.nodes.get(&n.id) {
                        metrics.drift_count = info.count;
                        metrics.drift_severity_max = info.severity_max.clone();
                        metrics.health = info.health.clone();
                    }
                }

                metrics.is_hotspot = hotspot_ids.contains(&n.id);
                metrics.is_in_cycle = cycle_node_set.contains(&n.id);
                metrics.community_id = community_map.get(&n.id).copied();

                ExplorerNode {
                    id: n.id.clone(),
                    kind: n.kind.kind_str().to_string(),
                    label: n.label.clone(),
                    description: n.description.clone(),
                    technology: n.technology().map(|s| s.to_string()),
                    parent_id: parent_id_map.get(&n.id).cloned(),
                    children_count: children_count_map.get(&n.id).copied().unwrap_or(0),
                    metrics,
                }
            })
            .collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        let mut edges: Vec<ExplorerEdge> = self
            .graph
            .edges
            .iter()
            .map(|e| {
                let pair = (e.source.clone(), e.target.clone());
                ExplorerEdge {
                    source: e.source.clone(),
                    target: e.target.clone(),
                    kind: e.kind.kind_str().to_string(),
                    label: e.label.clone(),
                    is_cycle_edge: cycle_edge_set.contains(&pair),
                    has_drift: drift_edge_set.contains(&pair),
                }
            })
            .collect();
        edges.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));

        let communities_out: Vec<ExpCommunity> = self
            .communities
            .as_ref()
            .map(|cs| {
                cs.iter()
                    .map(|c| ExpCommunity {
                        id: c.id,
                        label: c.suggested_label.clone(),
                        member_ids: c.members.clone(),
                        cohesion: c.cohesion,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let cycles: Vec<CycleGroup> = self
            .scc
            .as_ref()
            .map(|s| {
                s.components
                    .iter()
                    .filter(|c| c.is_cyclic)
                    .map(|c| CycleGroup {
                        nodes: c.nodes.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let hotspot_count = hotspot_ids.len();
        let cycle_count = cycles.len();
        let community_count = communities_out.len();

        let (drift_score, drift_health) = self
            .drift
            .as_ref()
            .map(|d| (d.score, d.health.clone()))
            .unwrap_or((0, "healthy".into()));

        ExplorerModel {
            schema_version: "explorer/v1".into(),
            summary: ExplorerSummary {
                total_nodes: nodes.len(),
                total_edges: edges.len(),
                drift_score,
                health: drift_health,
                hotspot_count,
                cycle_count,
                community_count,
            },
            nodes,
            edges,
            communities: communities_out,
            cycles,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn classify_zone(instability: f64, abstractness: f64, distance: f64) -> String {
    let zone = if distance <= 0.3 {
        Zone::MainSequence
    } else if instability < 0.5 && abstractness < 0.5 {
        Zone::ZoneOfPain
    } else {
        Zone::ZoneOfUselessness
    };
    match zone {
        Zone::MainSequence => "main_sequence",
        Zone::ZoneOfPain => "zone_of_pain",
        Zone::ZoneOfUselessness => "zone_of_uselessness",
    }
    .into()
}

fn build_cycle_edge_set(scc: &SccResult, graph: &KnowledgeGraph) -> HashSet<(String, String)> {
    let mut set = HashSet::new();
    for comp in &scc.components {
        if !comp.is_cyclic {
            continue;
        }
        let members: HashSet<&str> = comp.nodes.iter().map(|s| s.as_str()).collect();
        for edge in &graph.edges {
            if members.contains(edge.source.as_str()) && members.contains(edge.target.as_str()) {
                set.insert((edge.source.clone(), edge.target.clone()));
            }
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_graph::{ArchitectureNode, GraphEdge, NodeKind, SourceReference};

    fn test_graph() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::with_name("test");
        kg.add_node(ArchitectureNode {
            id: "SysA".into(),
            kind: NodeKind::new(NodeKind::SYSTEM),
            label: "System A".into(),
            description: Some("Primary system".into()),
            ..Default::default()
        })
        .unwrap();
        kg.add_node(ArchitectureNode {
            id: "SysA.API".into(),
            kind: NodeKind::new(NodeKind::CONTAINER),
            label: "API".into(),
            ..Default::default()
        })
        .unwrap();
        kg.add_node(ArchitectureNode {
            id: "SysB".into(),
            kind: NodeKind::new(NodeKind::SYSTEM),
            label: "System B".into(),
            ..Default::default()
        })
        .unwrap();
        kg.edges.push(GraphEdge {
            id: "e1".into(),
            source: "SysA".into(),
            target: "SysB".into(),
            kind: sruja_graph::EdgeKind::new(sruja_graph::EdgeKind::CALLS),
            label: Some("REST".into()),
            description: None,
            source_ref: SourceReference::Manual,
        });
        kg
    }

    #[test]
    fn build_minimal() {
        let model = ExplorerBuilder::new(test_graph()).build();
        assert_eq!(model.schema_version, "explorer/v1");
        assert_eq!(model.nodes.len(), 3);
        assert_eq!(model.edges.len(), 1);
        assert_eq!(model.summary.total_nodes, 3);
        assert_eq!(model.summary.total_edges, 1);
    }

    #[test]
    fn parent_child_hierarchy() {
        let model = ExplorerBuilder::new(test_graph()).build();
        let api = model.nodes.iter().find(|n| n.id == "SysA.API").unwrap();
        assert_eq!(api.parent_id.as_deref(), Some("SysA"));

        let sys_a = model.nodes.iter().find(|n| n.id == "SysA").unwrap();
        assert_eq!(sys_a.children_count, 1);
        assert!(sys_a.parent_id.is_none());
    }

    #[test]
    fn drift_overlay_applied() {
        let mut drift = DriftOverlay::default();
        drift.score = 45;
        drift.health = "minor_drift".into();
        drift.nodes.insert(
            "SysA".into(),
            NodeDriftInfo {
                count: 2,
                severity_max: Some("high".into()),
                health: "significant_drift".into(),
            },
        );
        drift.edges.push(EdgeDriftInfo {
            source: "SysA".into(),
            target: "SysB".into(),
        });

        let model = ExplorerBuilder::new(test_graph()).drift(drift).build();
        assert_eq!(model.summary.drift_score, 45);
        assert_eq!(model.summary.health, "minor_drift");

        let sys_a = model.nodes.iter().find(|n| n.id == "SysA").unwrap();
        assert_eq!(sys_a.metrics.drift_count, 2);
        assert_eq!(sys_a.metrics.health, "significant_drift");

        let edge = &model.edges[0];
        assert!(edge.has_drift);
    }

    #[test]
    fn json_roundtrip() {
        let model = ExplorerBuilder::new(test_graph()).build();
        let json = serde_json::to_string(&model).unwrap();
        let parsed: ExplorerModel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), model.nodes.len());
        assert_eq!(parsed.schema_version, "explorer/v1");
    }
}
