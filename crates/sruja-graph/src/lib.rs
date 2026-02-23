//! Architecture Knowledge Graph
//!
//! This crate provides the core knowledge graph for Sruja - storing architecture
//! elements, decisions, policies, and their relationships.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

pub mod graph;
pub mod query;
pub mod scan_merge;

pub use graph::KnowledgeGraph;
pub use query::{PolicyViolation, QueryError, QueryResult};
pub use scan_merge::merge_scan_into_graph;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    System,
    Service,
    Container,
    Component,
    Database,
    Queue,
    ExternalApi,
    Frontend,
    Module,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::System => write!(f, "system"),
            NodeKind::Service => write!(f, "service"),
            NodeKind::Container => write!(f, "container"),
            NodeKind::Component => write!(f, "component"),
            NodeKind::Database => write!(f, "database"),
            NodeKind::Queue => write!(f, "queue"),
            NodeKind::ExternalApi => write!(f, "external_api"),
            NodeKind::Frontend => write!(f, "frontend"),
            NodeKind::Module => write!(f, "module"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    DependsOn,
    Calls,
    ReadsFrom,
    WritesTo,
    PublishesTo,
    SubscribesTo,
    Owns,
    Contains,
    Uses,
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeKind::DependsOn => write!(f, "depends_on"),
            EdgeKind::Calls => write!(f, "calls"),
            EdgeKind::ReadsFrom => write!(f, "reads_from"),
            EdgeKind::WritesTo => write!(f, "writes_to"),
            EdgeKind::PublishesTo => write!(f, "publishes_to"),
            EdgeKind::SubscribesTo => write!(f, "subscribes_to"),
            EdgeKind::Owns => write!(f, "owns"),
            EdgeKind::Contains => write!(f, "contains"),
            EdgeKind::Uses => write!(f, "uses"),
        }
    }
}

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
}

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}
