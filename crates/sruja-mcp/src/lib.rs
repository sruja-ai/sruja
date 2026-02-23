//! MCP Server for Sruja
//!
//! Exposes architecture knowledge via HTTP API and MCP protocol.

pub mod error;
pub mod handlers;
pub mod server;
pub mod tools;

pub use error::McpError;
pub use server::McpServer;
pub use tools::{SrujaTool, ToolResponse};

use serde::{Deserialize, Serialize};
use sruja_graph::{KnowledgeGraph, PolicyViolation, QueryResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_decisions: usize,
    pub accepted_decisions: usize,
    pub total_policies: usize,
    pub nodes_by_kind: std::collections::HashMap<String, usize>,
}

impl From<&KnowledgeGraph> for ArchitectureSummary {
    fn from(graph: &KnowledgeGraph) -> Self {
        let stats = graph.stats();
        let mut nodes_by_kind = std::collections::HashMap::new();

        for node in graph.nodes.values() {
            *nodes_by_kind.entry(node.kind.to_string()).or_insert(0) += 1;
        }

        Self {
            total_nodes: stats.total_nodes,
            total_edges: stats.total_edges,
            total_decisions: stats.total_decisions,
            accepted_decisions: stats.accepted_decisions,
            total_policies: stats.total_policies,
            nodes_by_kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub decision: String,
    pub context: String,
    pub consequences: String,
    pub alternatives: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&sruja_graph::Decision> for DecisionResponse {
    fn from(d: &sruja_graph::Decision) -> Self {
        Self {
            id: d.id.clone(),
            title: d.title.clone(),
            status: d.status.to_string(),
            decision: d.decision.clone(),
            context: d.context.clone(),
            consequences: d.consequences.clone(),
            alternatives: d.alternatives.clone(),
            created_at: d.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub question: String,
    pub answer: String,
    pub confidence: f32,
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub kind: String,
    pub reference: String,
    pub excerpt: String,
}

impl From<QueryResult> for QueryResponse {
    fn from(result: QueryResult) -> Self {
        Self {
            question: result.question,
            answer: result.answer,
            confidence: result.confidence,
            evidence: result
                .evidence
                .into_iter()
                .map(|e| EvidenceItem {
                    kind: format!("{:?}", e.kind),
                    reference: e.reference,
                    excerpt: e.excerpt,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationResponse {
    pub policy_id: String,
    pub policy_name: String,
    pub source: String,
    pub target: String,
    pub message: String,
    pub severity: String,
}

impl From<PolicyViolation> for PolicyViolationResponse {
    fn from(v: PolicyViolation) -> Self {
        Self {
            policy_id: v.policy_id,
            policy_name: v.policy_name,
            source: v.source,
            target: v.target,
            message: v.message,
            severity: format!("{:?}", v.severity),
        }
    }
}
