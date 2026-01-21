//! Unique ID validation rule
//!
//! Ensures all element IDs are unique within the architecture.

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that checks for duplicate element IDs
pub struct UniqueIdRule;

impl Rule for UniqueIdRule {
    fn name(&self) -> &str {
        "Unique IDs"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut seen_ids: HashMap<String, SourceLocation> = HashMap::with_capacity(100);
        let mut diagnostics = Vec::new();

        // Helper to check if an ID is duplicate
        let mut check_id = |id: &str, loc: &SourceLocation| {
            if id.is_empty() {
                return;
            }

            if let Some(existing) = seen_ids.get(id) {
                let msg = format!(
                    "Duplicate identifier '{}'. First defined at line {}:{}",
                    id, existing.line, existing.column
                );

                let suggestions = vec![
                    format!("Rename this element to a unique identifier (e.g., '{}2' or '{}_v2')", id, id),
                    "Element IDs must be unique within the architecture".to_string(),
                ];

                diagnostics.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_DUPLICATE_ID,
                    Severity::Error,
                    msg,
                    loc.clone(),
                ).with_suggestions(suggestions));
            } else {
                seen_ids.insert(id.to_string(), loc.clone());
            }
        };

        // Collect all elements and check for duplicates
        let (elements, _relations) = sruja_language::collect_elements(program);
        
        for (fqn, elem) in &elements {
            check_id(fqn, &elem.location);
        }

        diagnostics
    }
}
