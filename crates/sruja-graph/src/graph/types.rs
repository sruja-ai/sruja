use crate::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: HashMap<NodeId, ArchitectureNode>,
    pub edges: Vec<ArchitectureEdge>,
    pub decisions: HashMap<DecisionId, Decision>,
    pub policies: HashMap<PolicyId, Policy>,
    pub requirements: HashMap<RequirementId, Requirement>,
    pub incidents: HashMap<String, Incident>,
    pub learnings: HashMap<String, LearningEntry>,
    pub recent_events: Vec<ContextEventSummary>,
    pub metadata: GraphMetadata,
}

/// A compact summary of a context event, embedded in the knowledge graph.
/// This enables temporal queries without reading the full JSONL log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEventSummary {
    pub timestamp: DateTime<Utc>,
    pub kind: String,
    pub elements: Vec<String>,
    pub outcome: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub commit_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_decisions: usize,
    pub accepted_decisions: usize,
    pub proposed_decisions: usize,
    pub total_policies: usize,
    pub total_requirements: usize,
    pub total_learnings: usize,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            name: "Architecture Graph".to_string(),
            description: None,
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            commit_sha: None,
        }
    }
}

impl sruja_graph_core::ContextGraph for KnowledgeGraph {
    type Node = ArchitectureNode;
    type Edge = ArchitectureEdge;

    fn nodes(&self) -> Vec<&Self::Node> {
        self.nodes.values().collect()
    }

    fn edges(&self) -> Vec<&Self::Edge> {
        self.edges.iter().collect()
    }

    fn get_node(&self, id: &str) -> Option<&Self::Node> {
        self.nodes.get(id)
    }

    fn get_edges_from(&self, node_id: &str) -> Vec<&Self::Edge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    fn get_edges_to(&self, node_id: &str) -> Vec<&Self::Edge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }
}
