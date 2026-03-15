//! Shared types for Sruja architecture DSL.
//!
//! This crate provides common type definitions used across multiple Sruja crates
//! to ensure consistency and reduce duplication.
//!
//! ## Node and edge kinds
//!
//! [`NodeKind`] and [`EdgeKind`] are the **inferred/graph** kind set: they describe
//! nodes and edges as produced by [`sruja-scan`](https://docs.rs/sruja-scan) and
//! stored in [`sruja-graph`](https://docs.rs/sruja-graph). The DSL (`.sruja`) allows
//! user-defined kinds via `kind "Label"`; when converting from DSL to a graph (e.g.
//! in sruja-diff's `program_to_graph`), DSL kinds are mapped to these enum values
//! or to a suitable default. Consumers (scan, graph, diff, report) should use these
//! types as the single source of truth for kind identity in architecture intelligence.

use serde::{Deserialize, Serialize};

pub mod severity;

pub use severity::Severity;

/// Unique identifier for a node in the architecture graph.
pub type NodeId = String;

/// Unique identifier for a decision in the architecture.
pub type DecisionId = String;

/// Unique identifier for a policy in the architecture.
pub type PolicyId = String;

/// Unique identifier for a requirement in the architecture.
pub type RequirementId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A top-level system or subsystem that groups related services.
    System,
    /// A deployable unit of functionality that can run independently.
    Service,
    /// A grouping of components within a service.
    Container,
    /// A modular unit of code with a specific responsibility.
    Component,
    /// A data storage system.
    Database,
    /// A message queue or streaming platform.
    Queue,
    /// An external API or service outside the system boundary.
    ExternalApi,
    /// A user-facing application or interface.
    Frontend,
    /// A generic module or package.
    Module,
}

impl NodeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::System => "system",
            NodeKind::Service => "service",
            NodeKind::Container => "container",
            NodeKind::Component => "component",
            NodeKind::Database => "database",
            NodeKind::Queue => "queue",
            NodeKind::ExternalApi => "external_api",
            NodeKind::Frontend => "frontend",
            NodeKind::Module => "module",
        }
    }
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for NodeKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(NodeKind::System),
            "service" => Ok(NodeKind::Service),
            "container" => Ok(NodeKind::Container),
            "component" => Ok(NodeKind::Component),
            "database" => Ok(NodeKind::Database),
            "queue" => Ok(NodeKind::Queue),
            "external_api" | "externalapi" => Ok(NodeKind::ExternalApi),
            "frontend" => Ok(NodeKind::Frontend),
            "module" => Ok(NodeKind::Module),
            _ => Err(format!("Unknown NodeKind: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Indicates that the source node depends on the target node for functionality.
    DependsOn,
    /// Indicates that the source node makes calls to the target node.
    Calls,
    /// Indicates that the source node reads data from the target node.
    ReadsFrom,
    /// Indicates that the source node writes data to the target node.
    WritesTo,
    /// Indicates that the source node publishes events to the target node.
    PublishesTo,
    /// Indicates that the source node subscribes to events from the target node.
    SubscribesTo,
    /// Indicates that the source node owns or manages the target node.
    Owns,
    /// Indicates that the source node contains or encompasses the target node.
    Contains,
    /// Indicates that the source node uses the target node in some way.
    Uses,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::DependsOn => "depends_on",
            EdgeKind::Calls => "calls",
            EdgeKind::ReadsFrom => "reads_from",
            EdgeKind::WritesTo => "writes_to",
            EdgeKind::PublishesTo => "publishes_to",
            EdgeKind::SubscribesTo => "subscribes_to",
            EdgeKind::Owns => "owns",
            EdgeKind::Contains => "contains",
            EdgeKind::Uses => "uses",
        }
    }
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EdgeKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "depends_on" => Ok(EdgeKind::DependsOn),
            "calls" => Ok(EdgeKind::Calls),
            "reads_from" => Ok(EdgeKind::ReadsFrom),
            "writes_to" => Ok(EdgeKind::WritesTo),
            "publishes_to" => Ok(EdgeKind::PublishesTo),
            "subscribes_to" => Ok(EdgeKind::SubscribesTo),
            "owns" => Ok(EdgeKind::Owns),
            "contains" => Ok(EdgeKind::Contains),
            "uses" => Ok(EdgeKind::Uses),
            _ => Err(format!("Unknown EdgeKind: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodekind_display() {
        assert_eq!(NodeKind::ExternalApi.to_string(), "external_api");
        assert_eq!(NodeKind::Database.to_string(), "database");
    }

    #[test]
    fn test_nodekind_from_str() {
        assert_eq!("service".parse::<NodeKind>(), Ok(NodeKind::Service));
        assert_eq!("module".parse::<NodeKind>(), Ok(NodeKind::Module));
        assert_eq!(
            "external_api".parse::<NodeKind>(),
            Ok(NodeKind::ExternalApi)
        );
        assert_eq!("externalapi".parse::<NodeKind>(), Ok(NodeKind::ExternalApi));
        assert!("unknown".parse::<NodeKind>().is_err());
    }

    #[test]
    fn test_nodekind_as_str() {
        assert_eq!(NodeKind::System.as_str(), "system");
        assert_eq!(NodeKind::ExternalApi.as_str(), "external_api");
        assert_eq!(NodeKind::Queue.as_str(), "queue");
    }

    #[test]
    fn test_edgekind_display() {
        assert_eq!(EdgeKind::DependsOn.to_string(), "depends_on");
        assert_eq!(EdgeKind::Calls.to_string(), "calls");
    }

    #[test]
    fn test_edgekind_from_str() {
        assert_eq!("calls".parse::<EdgeKind>(), Ok(EdgeKind::Calls));
        assert_eq!("reads_from".parse::<EdgeKind>(), Ok(EdgeKind::ReadsFrom));
        assert_eq!(
            "publishes_to".parse::<EdgeKind>(),
            Ok(EdgeKind::PublishesTo)
        );
        assert_eq!("owns".parse::<EdgeKind>(), Ok(EdgeKind::Owns));
        assert!("unknown".parse::<EdgeKind>().is_err());
    }

    #[test]
    fn test_edgekind_serde() {
        let kind = EdgeKind::ReadsFrom;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"reads_from\"");
        let parsed: EdgeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kind);
    }

    #[test]
    fn test_edgekind_as_str() {
        assert_eq!(EdgeKind::DependsOn.as_str(), "depends_on");
        assert_eq!(EdgeKind::SubscribesTo.as_str(), "subscribes_to");
    }

    #[test]
    fn test_nodekind_serde() {
        let kind = NodeKind::ExternalApi;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"external_api\"");
        let parsed: NodeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kind);
    }

    // Tests for type aliases
    #[test]
    fn test_type_aliases() {
        let node_id: NodeId = "node123".to_string();
        let decision_id: DecisionId = "dec456".to_string();
        let policy_id: PolicyId = "pol789".to_string();
        let requirement_id: RequirementId = "req000".to_string();

        assert_eq!(node_id, "node123");
        assert_eq!(decision_id, "dec456");
        assert_eq!(policy_id, "pol789");
        assert_eq!(requirement_id, "req000");
    }

    // Add FromStr test for Severity (missing implementation)
    #[test]
    fn test_severity_from_str() {
        assert_eq!("error".parse::<Severity>(), Ok(Severity::Error));
        assert_eq!("warning".parse::<Severity>(), Ok(Severity::Warning));
        assert_eq!("info".parse::<Severity>(), Ok(Severity::Info));
        assert_eq!("hint".parse::<Severity>(), Ok(Severity::Hint));
        assert_eq!("ERROR".parse::<Severity>(), Ok(Severity::Error)); // Case insensitive
        assert_eq!("WARNING".parse::<Severity>(), Ok(Severity::Warning));
        assert_eq!("INFO".parse::<Severity>(), Ok(Severity::Info));
        assert_eq!("HINT".parse::<Severity>(), Ok(Severity::Hint));
        assert!("unknown".parse::<Severity>().is_err());
    }
}
