//! Domain Schema representation and validation logic.

use std::collections::{HashMap, HashSet};
use sruja_language::ast::SchemaBlock;

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
        let mut node_kinds = HashSet::new();
        for kind in &schema.node_kinds {
            node_kinds.insert(kind.clone());
        }

        let mut edge_kinds = HashSet::new();
        for kind in &schema.edge_kinds {
            edge_kinds.insert(kind.clone());
        }

        let mut nesting_rules: HashMap<String, HashSet<String>> = HashMap::new();
        for rule in &schema.nesting {
            nesting_rules
                .entry(rule.parent.clone())
                .or_default()
                .insert(rule.child.clone());
        }

        Self {
            name: schema.name.clone(),
            node_kinds,
            edge_kinds,
            nesting_rules,
        }
    }

    /// Returns the default architecture schema (C4-inspired).
    pub fn architecture() -> Self {
        let mut schema = Self::new("architecture");
        schema.node_kinds = [
            "person", "system", "container", "component", "database", "queue", "service"
        ].iter().map(|s| s.to_string()).collect();
        
        schema.edge_kinds = [
            "depends_on", "calls", "reads_from", "writes_to", "publishes_to", "subscribes_to", "owns", "contains", "uses"
        ].iter().map(|s| s.to_string()).collect();

        // C4 nesting rules
        let mut nesting = HashMap::new();
        nesting.insert("system".to_string(), [
            "container".to_string(), "database".to_string(), "queue".to_string(), "component".to_string()
        ].iter().cloned().collect());
        nesting.insert("container".to_string(), [
            "component".to_string()
        ].iter().cloned().collect());
        
        schema.nesting_rules = nesting;
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

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::ast::NestingRule;
    use sruja_diagnostics::SourceLocation;

    #[test]
    fn test_schema_from_ast() {
        let schema_block = SchemaBlock {
            location: SourceLocation::new("test.sruja".to_string(), 1, 1),
            name: "compliance".to_string(),
            node_kinds: vec!["regulation".to_string(), "policy".to_string()],
            edge_kinds: vec!["mandates".to_string()],
            nesting: vec![NestingRule {
                parent: "regulation".to_string(),
                child: "policy".to_string(),
            }],
        };

        let schema = DomainSchema::from_ast(&schema_block);
        assert_eq!(schema.name, "compliance");
        assert!(schema.is_node_kind_allowed("regulation"));
        assert!(schema.is_node_kind_allowed("policy"));
        assert!(!schema.is_node_kind_allowed("system"));
        assert!(schema.is_edge_kind_allowed("mandates"));
        assert!(!schema.is_edge_kind_allowed("calls"));
        assert!(schema.is_nesting_allowed("regulation", "policy"));
        assert!(!schema.is_nesting_allowed("policy", "regulation"));
    }

    #[test]
    fn test_architecture_schema() {
        let schema = DomainSchema::architecture();
        assert_eq!(schema.name, "architecture");
        assert!(schema.is_node_kind_allowed("system"));
        assert!(schema.is_nesting_allowed("system", "container"));
        assert!(!schema.is_nesting_allowed("component", "system"));
    }
}
