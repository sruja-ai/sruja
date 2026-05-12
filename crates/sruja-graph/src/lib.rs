//! Architecture Knowledge Graph
//!
//! This crate provides the core knowledge graph for Sruja - storing architecture
//! elements, decisions, policies, and their relationships.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

pub mod centrality;
pub mod context_score;
pub mod coupling;
pub mod graph;
pub mod query;
#[cfg(not(target_arch = "wasm32"))]
pub mod scan_merge;
pub mod scc;
pub mod treewidth;

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
pub use graph::KnowledgeGraph;
pub use query::{
    LlmGuidedWhyResult, LlmGuidedWhyStep, PolicyViolation, QueryError, QueryResult,
    ReasonedWhyResult, ReasonedWhyStep,
};
#[cfg(not(target_arch = "wasm32"))]
pub use scan_merge::merge_scan_into_graph;
pub use scc::{CondensationEdge, Scc, SccAnalyzer, SccResult};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub label: String,
    pub technology: Option<String>,
    pub description: Option<String>,
    pub metadata: HashMap<String, String>,
    pub source: SourceReference,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gotchas: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operational_constraints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runbooks: Vec<String>,
}

impl Default for ArchitectureNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: NodeKind::Module,
            label: String::new(),
            technology: None,
            description: None,
            metadata: HashMap::new(),
            source: SourceReference::Manual,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            gotchas: Vec::new(),
            operational_constraints: Vec::new(),
            runbooks: Vec::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureEdge {
    pub id: String,
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
    pub label: Option<String>,
    pub description: Option<String>,
    pub source_ref: SourceReference,
}

impl sruja_graph_core::ContextNode for ArchitectureNode {
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
        self.technology.as_deref()
    }
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl sruja_graph_core::ContextEdge for ArchitectureEdge {
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
}
