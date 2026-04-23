//! Evidence Mapper
//!
//! Responsible for correlating scanned nodes/edges (fresh evidence) to
//! defined architecture elements and relationships (reviewed truth).

use sruja_language::DomainSchema;
use sruja_scan::Node;

/// Maps raw scanner output to schema-aligned elements.
pub struct EvidenceMapper<'a> {
    schema: &'a DomainSchema,
}

impl<'a> EvidenceMapper<'a> {
    pub fn new(schema: &'a DomainSchema) -> Self {
        Self { schema }
    }

    /// Correlates a scanned node to a schema node kind.
    pub fn map_node_kind(&self, node: &Node) -> String {
        let kind = node.kind.to_string_kind().to_lowercase();
        if self.schema.is_node_kind_allowed(&kind) {
            kind
        } else {
            // Fallback: try to map common scanner kinds to schema kinds
            match kind.as_str() {
                "class" | "module" | "function" => {
                    if self.schema.is_node_kind_allowed("component") {
                        "component".to_string()
                    } else if self.schema.is_node_kind_allowed("service") {
                        "service".to_string()
                    } else {
                        kind
                    }
                }
                "database" | "table" | "collection" => {
                    if self.schema.is_node_kind_allowed("database") {
                        "database".to_string()
                    } else {
                        kind
                    }
                }
                _ => kind,
            }
        }
    }

    /// Identifies schema violations in relationships.
    pub fn check_relationship_violation(
        &self,
        from_kind: &str,
        to_kind: &str,
        _label: &str,
    ) -> Option<String> {
        // If nesting rules are defined, they act as a proxy for valid relationships
        // for this simplified Phase 3 implementation.
        // Future: Add explicit relationship_rules to DomainSchema.
        
        // Example: if a 'component' is talking to a 'regulation', that might be weird.
        // For now, we just ensure both kinds are allowed.
        if !self.schema.is_node_kind_allowed(from_kind) {
            return Some(format!("Source kind '{}' is not defined in schema", from_kind));
        }
        if !self.schema.is_node_kind_allowed(to_kind) {
            return Some(format!("Target kind '{}' is not defined in schema", to_kind));
        }
        
        None
    }
}
