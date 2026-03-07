//! Shared types for Sruja architecture DSL.
//!
//! This crate provides common type definitions used across multiple Sruja crates
//! to ensure consistency and reduce duplication.

use serde::{Deserialize, Serialize};

pub mod severity;

pub use severity::Severity;

pub type NodeId = String;
pub type DecisionId = String;
pub type PolicyId = String;
pub type RequirementId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
        assert!("unknown".parse::<NodeKind>().is_err());
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
        assert!("unknown".parse::<EdgeKind>().is_err());
    }

    #[test]
    fn test_nodekind_serde() {
        let kind = NodeKind::ExternalApi;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"external_api\"");
        let parsed: NodeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kind);
    }
}
