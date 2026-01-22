//! Orphan element detection rule
//!
//! Detects elements that are not referenced by any relations.

use std::collections::HashSet;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that detects orphan elements (elements not referenced by relations)
pub struct OrphanDetectionRule;

impl Rule for OrphanDetectionRule {
    fn name(&self) -> &str {
        "Orphan Detection"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect all elements and relations
        let (elements, relations) = sruja_language::collect_elements(program);
        
        // Build set of referenced elements from relations
        let mut referenced: HashSet<String> = HashSet::new();
        for rel in &relations {
            referenced.insert(rel.from.as_string());
            referenced.insert(rel.to.as_string());
        }

        // Find orphan elements (defined but not referenced)
        for (fqn, elem) in &elements {
            if !referenced.contains(fqn) {
                diagnostics.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_ORPHAN_ELEMENT,
                    Severity::Warning,
                    format!("Element '{}' is defined but not referenced by any relation", fqn),
                    elem.location.clone(),
                ).with_suggestions(vec![
                    "Orphan elements may indicate incomplete architecture".to_string(),
                    "Consider adding relations or removing unused elements".to_string(),
                ]));
            }
        }

        diagnostics
    }
}
