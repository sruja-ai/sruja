//! Architecture Knowledge Graph
//!
//! This crate provides the core knowledge graph for Sruja - storing architecture
//! elements, decisions, policies, and their relationships.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

pub mod bm25;
pub mod centrality;
pub mod context_score;
pub mod coupling;
pub mod graph;
pub mod hybrid_retrieval;
pub mod learning;
pub mod query;
#[cfg(not(target_arch = "wasm32"))]
pub mod scan_merge;
pub mod scc;
pub mod snapshot;
pub mod system_graph;
pub mod treewidth;

pub use bm25::{Bm25Hit, SparseIndex};
pub use centrality::{
    ArchitecturalHotspot, BridgeNode, CentralityAnalyzer, CentralityResult, HotspotRole, HubNode,
};
pub use context_score::{
    compute_context_score, scan_external_context, ContextScore, DimensionScore,
    ExternalContextSummary, QuickWin,
};
pub use coupling::{
    CouplingAnalyzer, CouplingResult, CouplingSummary, CouplingViolation, CouplingViolationType,
    ModuleCoupling, Zone,
};
pub use graph::{ContextEventSummary, KnowledgeGraph};
pub use hybrid_retrieval::{
    classify_query, execute_graph_only, execute_hybrid, select_strategy, HybridResult,
    QueryComplexity, RetrievalStrategy, SemanticCandidate,
};
pub use learning::{ExperimentOutcome, LearningEntry, LearningKind, LearningPatch, MemoryError};
pub use query::{
    LlmGuidedWhyResult, LlmGuidedWhyStep, PolicyViolation, QueryError, QueryResult,
    ReasonedWhyResult, ReasonedWhyStep,
};
#[cfg(not(target_arch = "wasm32"))]
pub use scan_merge::merge_scan_into_graph;
pub use scc::{CondensationEdge, Scc, SccAnalyzer, SccResult};
pub use snapshot::{compute_deltas, GraphDelta, GraphSnapshot};
pub use system_graph::{
    BlastRadius, EdgeConfidence, SystemEdge, SystemGraph, SystemHubNode, SystemNode, SystemRepo,
    TraceHop, TraceResult,
};
pub use treewidth::{
    ComplexityHotspot, ComplexityRating, RefactorPattern, RefactorSuggestion, TreeBag,
    TreewidthAnalyzer, TreewidthResult,
};

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Decision not found: {0}")]
    DecisionNotFound(String),

    #[error("Duplicate node: {0}")]
    DuplicateNode(String),

    #[error("Invalid edge: {0}")]
    InvalidEdge(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type NodeId = String;
pub type DecisionId = String;
pub type PolicyId = String;
pub type RequirementId = String;
pub type SessionId = String;
pub type MessageId = String;

/// Re-export shared node/edge kinds from sruja-language for a single source of truth.
pub use sruja_language::{EdgeKind, NodeKind};

/// Generic knowledge graph node storing arbitrary domain context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub label: String,
    pub description: Option<String>,
    pub metadata: HashMap<String, String>,
    pub source: SourceReference,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Backward compatible alias for code migrations
pub type ArchitectureNode = GraphNode;

impl GraphNode {
    /// Helper constructor ensuring domain fields are cleanly initialized into metadata
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: NodeId,
        kind: NodeKind,
        label: String,
        description: Option<String>,
        metadata: HashMap<String, String>,
        source: SourceReference,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            kind,
            label,
            description,
            metadata,
            source,
            created_at,
            updated_at,
        }
    }

    // Domain specific getters storing data in metadata for generalization
    pub fn technology(&self) -> Option<&str> {
        self.metadata.get("technology").map(|s| s.as_str())
    }

    pub fn set_technology(&mut self, tech: Option<String>) {
        if let Some(t) = tech {
            self.metadata.insert("technology".to_string(), t);
        } else {
            self.metadata.remove("technology");
        }
    }

    pub fn gotchas(&self) -> Vec<String> {
        self.metadata
            .get("gotchas")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    pub fn set_gotchas(&mut self, gotchas: Vec<String>) {
        if !gotchas.is_empty() {
            if let Ok(s) = serde_json::to_string(&gotchas) {
                self.metadata.insert("gotchas".to_string(), s);
            }
        }
    }

    pub fn operational_constraints(&self) -> Vec<String> {
        self.metadata
            .get("operational_constraints")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    pub fn set_operational_constraints(&mut self, constraints: Vec<String>) {
        if !constraints.is_empty() {
            if let Ok(s) = serde_json::to_string(&constraints) {
                self.metadata
                    .insert("operational_constraints".to_string(), s);
            }
        }
    }

    pub fn runbooks(&self) -> Vec<String> {
        self.metadata
            .get("runbooks")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    pub fn set_runbooks(&mut self, runbooks: Vec<String>) {
        if !runbooks.is_empty() {
            if let Ok(s) = serde_json::to_string(&runbooks) {
                self.metadata.insert("runbooks".to_string(), s);
            }
        }
    }
}

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: NodeKind(NodeKind::MODULE.to_string()),
            label: String::new(),
            description: None,
            metadata: HashMap::new(),
            source: SourceReference::Manual,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub severity: Option<String>,
    pub affected: Vec<NodeId>,
    pub cause: Option<String>,
    pub resolution: Option<String>,
    pub lesson: Option<String>,
    pub source: SourceReference,
}

/// Generic knowledge graph edge storing relationships between domain nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
    pub label: Option<String>,
    pub description: Option<String>,
    pub source_ref: SourceReference,
}

/// Backward compatible alias for code migrations
pub type ArchitectureEdge = GraphEdge;

impl sruja_graph_core::ContextNode for GraphNode {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        self.kind.kind_str()
    }
    fn label(&self) -> &str {
        &self.label
    }
    fn technology(&self) -> Option<&str> {
        self.technology()
    }
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl sruja_graph_core::ContextEdge for GraphEdge {
    fn id(&self) -> &str {
        &self.id
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn target(&self) -> &str {
        &self.target
    }
    fn kind(&self) -> &str {
        self.kind.kind_str()
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    pub title: String,
    pub status: DecisionStatus,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub alternatives: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ratified_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub source: SourceReference,
    pub affects: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
    Rejected,
}

impl std::fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionStatus::Proposed => write!(f, "proposed"),
            DecisionStatus::Accepted => write!(f, "accepted"),
            DecisionStatus::Deprecated => write!(f, "deprecated"),
            DecisionStatus::Superseded => write!(f, "superseded"),
            DecisionStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub name: String,
    pub description: String,
    pub rules: Vec<PolicyRule>,
    pub severity: PolicySeverity,
    pub source: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub description: String,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub source_kind: Option<NodeKind>,
    pub target_kind: Option<NodeKind>,
    pub allowed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicySeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: RequirementId,
    pub title: String,
    pub description: String,
    pub priority: RequirementPriority,
    pub source: SourceReference,
    pub satisfied_by: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementPriority {
    Must,
    Should,
    Could,
    Wont,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceReference {
    Conversation {
        session_id: SessionId,
        message_ids: Vec<MessageId>,
    },
    AdrFile {
        path: String,
    },
    DslFile {
        path: String,
        line: u32,
    },
    ScannedRepo {
        path: String,
    },
    Manual,
}

impl SourceReference {
    pub fn conversation(session_id: impl Into<String>, message_ids: Vec<String>) -> Self {
        SourceReference::Conversation {
            session_id: session_id.into(),
            message_ids,
        }
    }

    pub fn dsl_file(path: impl Into<String>, line: u32) -> Self {
        SourceReference::DslFile {
            path: path.into(),
            line,
        }
    }

    pub fn adr_file(path: impl Into<String>) -> Self {
        SourceReference::AdrFile { path: path.into() }
    }

    pub fn scanned_repo(path: impl Into<String>) -> Self {
        SourceReference::ScannedRepo { path: path.into() }
    }

    pub fn manual() -> Self {
        SourceReference::Manual
    }

    /// Short summary for evidence display (deterministic, no LLM).
    pub fn summary(&self) -> String {
        match self {
            SourceReference::ScannedRepo { path } => format!("scanned: {}", path),
            SourceReference::AdrFile { path } => format!("ADR: {}", path),
            SourceReference::DslFile { path, line } => format!("{}:{}", path, line),
            SourceReference::Conversation { .. } => "conversation".to_string(),
            SourceReference::Manual => "manual".to_string(),
        }
    }
}

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_reference_summary_scanned_repo() {
        let r = SourceReference::scanned_repo("/path/to/repo");
        assert_eq!(r.summary(), "scanned: /path/to/repo");
    }

    #[test]
    fn source_reference_summary_adr_file() {
        let r = SourceReference::adr_file("docs/adr/001.md");
        assert_eq!(r.summary(), "ADR: docs/adr/001.md");
    }

    #[test]
    fn source_reference_summary_dsl_file() {
        let r = SourceReference::dsl_file("arch.sruja", 10);
        assert_eq!(r.summary(), "arch.sruja:10");
    }

    #[test]
    fn source_reference_summary_manual() {
        let r = SourceReference::manual();
        assert_eq!(r.summary(), "manual");
    }

    #[test]
    fn source_reference_summary_conversation() {
        let r = SourceReference::conversation("sess-1", vec!["msg-1".to_string()]);
        assert_eq!(r.summary(), "conversation");
    }

    #[test]
    fn generate_id_returns_non_empty_uuid_like() {
        let id = generate_id();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 36);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }

    #[test]
    fn decision_status_display() {
        assert_eq!(DecisionStatus::Accepted.to_string(), "accepted");
        assert_eq!(DecisionStatus::Proposed.to_string(), "proposed");
    }

    #[test]
    fn graph_node_technology_roundtrip() {
        let mut node = GraphNode::default();
        assert!(node.technology().is_none());
        node.set_technology(Some("Rust".to_string()));
        assert_eq!(node.technology(), Some("Rust"));
        node.set_technology(None);
        assert!(node.technology().is_none());
    }

    #[test]
    fn graph_node_gotchas_and_constraints_serialize_in_metadata() {
        let mut node = GraphNode::default();
        node.set_gotchas(vec!["avoid sync IO".to_string()]);
        node.set_operational_constraints(vec!["read-only".to_string()]);
        node.set_runbooks(vec!["restart pod".to_string()]);

        assert_eq!(node.gotchas(), vec!["avoid sync IO".to_string()]);
        assert_eq!(
            node.operational_constraints(),
            vec!["read-only".to_string()]
        );
        assert_eq!(node.runbooks(), vec!["restart pod".to_string()]);
    }

    #[test]
    fn context_node_trait_delegates_to_graph_node() {
        let mut node = GraphNode {
            id: "svc".to_string(),
            label: "Service".to_string(),
            ..Default::default()
        };
        node.set_technology(Some("Go".to_string()));
        node.description = Some("API service".to_string());

        assert_eq!(sruja_graph_core::ContextNode::id(&node), "svc");
        assert_eq!(sruja_graph_core::ContextNode::label(&node), "Service");
        assert_eq!(sruja_graph_core::ContextNode::technology(&node), Some("Go"));
        assert_eq!(
            sruja_graph_core::ContextNode::description(&node),
            Some("API service")
        );
    }

    #[test]
    fn context_edge_trait_delegates_to_graph_edge() {
        let edge = GraphEdge {
            id: "e1".to_string(),
            source: "a".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            label: Some("calls".to_string()),
            description: Some("dependency".to_string()),
            source_ref: SourceReference::manual(),
        };
        assert_eq!(sruja_graph_core::ContextEdge::source(&edge), "a");
        assert_eq!(sruja_graph_core::ContextEdge::target(&edge), "b");
        assert_eq!(sruja_graph_core::ContextEdge::label(&edge), Some("calls"));
    }
}
