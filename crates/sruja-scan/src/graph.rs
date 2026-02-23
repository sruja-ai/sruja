//! Inferred architecture graph schema.
//!
//! This is the minimal, repo-scoped graph used for deterministic diffing and review grounding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Service,
    Module,
    Database,
    ExternalApi,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EdgeKind {
    Calls,
    ReadsFrom,
    WritesTo,
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

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
