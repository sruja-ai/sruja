//! Sruja Graph Core Traits
//!
//! This crate defines domain-agnostic traits for graph nodes, edges, and the graph itself.
//! These traits allow the Sruja graph analyzers to operate on any directed graph,
//! facilitating generalization beyond software architecture.

use std::collections::HashMap;

/// A generic node in the context graph.
pub trait ContextNode {
    /// Unique identifier for the node.
    fn id(&self) -> &str;

    /// The kind of the node (e.g., "system", "policy", "regulation").
    fn kind(&self) -> &str;

    /// Human-readable label for the node.
    fn label(&self) -> &str;

    /// Optional technology associated with the node.
    fn technology(&self) -> Option<&str>;

    /// Optional description of the node.
    fn description(&self) -> Option<&str>;

    /// Arbitrary metadata associated with the node.
    fn metadata(&self) -> &HashMap<String, String>;
}

/// A generic directed edge in the context graph.
pub trait ContextEdge {
    /// Unique identifier for the edge.
    fn id(&self) -> &str;

    /// Identifier of the source node.
    fn source(&self) -> &str;

    /// Identifier of the target node.
    fn target(&self) -> &str;

    /// The kind of relationship (e.g., "depends_on", "implements", "mandates").
    fn kind(&self) -> &str;

    /// Optional human-readable label for the relationship.
    fn label(&self) -> Option<&str>;

    /// Optional description of the relationship.
    fn description(&self) -> Option<&str>;
}

/// A generic directed graph representing a context.
pub trait ContextGraph {
    type Node: ContextNode;
    type Edge: ContextEdge;

    /// Returns all nodes in the graph.
    fn nodes(&self) -> Vec<&Self::Node>;

    /// Returns all edges in the graph.
    fn edges(&self) -> Vec<&Self::Edge>;

    /// Retrieves a node by its identifier.
    fn get_node(&self, id: &str) -> Option<&Self::Node>;

    /// Returns all edges originating from the specified node.
    fn get_edges_from(&self, node_id: &str) -> Vec<&Self::Edge>;

    /// Returns all edges pointing to the specified node.
    fn get_edges_to(&self, node_id: &str) -> Vec<&Self::Edge>;
}

/// Result of a blast radius analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlastRadiusResult {
    pub target_id: String,
    pub upstream: Vec<BlastRadiusNode>,
    pub downstream: Vec<BlastRadiusNode>,
}

/// A node in the blast radius analysis result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlastRadiusNode {
    pub id: String,
    pub depth: usize,
}
