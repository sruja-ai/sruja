//! Domain Schema representation and logic.

use crate::ast::SchemaBlock;
use std::collections::{HashMap, HashSet};

/// Represents a domain schema with allowed node kinds, edge kinds, and nesting rules.
#[derive(Debug, Clone, Default)]
pub struct DomainSchema {
    /// Name of the schema (e.g. "architecture", "compliance").
    pub name: String,
    /// Set of allowed node kinds. If empty, all kinds are allowed.
    pub node_kinds: HashSet<String>,
    /// Set of allowed edge kinds. If empty, all kinds are allowed.
    pub edge_kinds: HashSet<String>,
    /// Maps a parent kind to the set of allowed child kinds.
    /// If empty, all nesting is allowed.
    pub nesting_rules: HashMap<String, HashSet<String>>,
}

impl DomainSchema {
    /// Create a new domain schema.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            node_kinds: HashSet::new(),
            edge_kinds: HashSet::new(),
            nesting_rules: HashMap::new(),
        }
    }

    /// Create a domain schema from an AST SchemaBlock.
    pub fn from_ast(schema: &SchemaBlock) -> Self {
        let mut base = match schema.name.as_str() {
            "architecture" => Self::architecture(),
            "compliance" => Self::compliance(),
            "business_process" | "business-process" => Self::business_process(),
            "knowledge" => Self::knowledge(),
            _ => Self::new(&schema.name),
        };

        // Merge any explicitly provided definitions from the schema block
        for kind in &schema.node_kinds {
            base.node_kinds.insert(kind.clone());
        }

        for kind in &schema.edge_kinds {
            base.edge_kinds.insert(kind.clone());
        }

        for rule in &schema.nesting {
            base.nesting_rules
                .entry(rule.parent.clone())
                .or_default()
                .insert(rule.child.clone());
        }

        base
    }

    /// Returns the default architecture schema (C4-inspired).
    pub fn architecture() -> Self {
        let mut schema = Self::new("architecture");
        schema.node_kinds = [
            "person",
            "system",
            "container",
            "component",
            "database",
            "queue",
            "service",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        schema.edge_kinds = [
            "depends_on",
            "calls",
            "reads_from",
            "writes_to",
            "publishes_to",
            "subscribes_to",
            "owns",
            "contains",
            "uses",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        // C4 nesting rules
        let mut nesting = HashMap::new();
        nesting.insert(
            "system".to_string(),
            [
                "container".to_string(),
                "database".to_string(),
                "queue".to_string(),
                "component".to_string(),
            ]
            .iter()
            .cloned()
            .collect(),
        );
        nesting.insert(
            "container".to_string(),
            ["component".to_string()].iter().cloned().collect(),
        );

        schema.nesting_rules = nesting;
        schema
    }

    /// Returns a built-in schema pack for regulatory compliance.
    pub fn compliance() -> Self {
        let mut schema = Self::new("compliance");
        schema.node_kinds = [
            "domain",
            "regulation",
            "control",
            "evidence",
            "risk",
            "asset",
            "audit",
            "owner",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        schema.edge_kinds = [
            "mandates",
            "mitigates",
            "audits",
            "owns",
            "applies_to",
            "violates",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let mut nesting = HashMap::new();
        nesting.insert(
            "domain".to_string(),
            ["regulation".to_string()].iter().cloned().collect(),
        );
        nesting.insert(
            "regulation".to_string(),
            ["control".to_string(), "evidence".to_string()]
                .iter()
                .cloned()
                .collect(),
        );
        schema.nesting_rules = nesting;
        schema
    }

    /// Returns a built-in schema pack for business process modeling.
    pub fn business_process() -> Self {
        let mut schema = Self::new("business_process");
        schema.node_kinds = [
            "process", "activity", "decision", "actor", "system", "artifact",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        schema.edge_kinds = [
            "triggers",
            "performs",
            "decides",
            "escalates_to",
            "uses",
            "produces",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let mut nesting = HashMap::new();
        nesting.insert(
            "process".to_string(),
            ["activity".to_string(), "decision".to_string()]
                .iter()
                .cloned()
                .collect(),
        );
        schema.nesting_rules = nesting;
        schema
    }

    /// Returns a built-in schema pack for knowledge intelligence mapping.
    pub fn knowledge() -> Self {
        let mut schema = Self::new("knowledge");
        schema.node_kinds = ["concept", "citation", "claim", "evidence", "source"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        schema.edge_kinds = [
            "cites",
            "supports",
            "contradicts",
            "derived_from",
            "relates_to",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        schema
    }

    /// Check if a node kind is allowed in this schema.
    pub fn is_node_kind_allowed(&self, kind: &str) -> bool {
        self.node_kinds.is_empty() || self.node_kinds.contains(kind)
    }

    /// Check if an edge kind is allowed in this schema.
    pub fn is_edge_kind_allowed(&self, kind: &str) -> bool {
        self.edge_kinds.is_empty() || self.edge_kinds.contains(kind)
    }

    /// Check if a child kind can be nested under a parent kind.
    pub fn is_nesting_allowed(&self, parent_kind: &str, child_kind: &str) -> bool {
        if self.nesting_rules.is_empty() {
            return true;
        }
        if let Some(allowed_children) = self.nesting_rules.get(parent_kind) {
            allowed_children.contains(child_kind)
        } else {
            false
        }
    }
}
