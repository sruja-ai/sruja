//! Unique ID validation rule
//!
//! Ensures all element IDs are unique within architecture.

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{ElementDefBodyItem, Program, TopLevelItem};

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
                    format!(
                        "Rename this element to a unique identifier (e.g., '{}2' or '{}_v2')",
                        id, id
                    ),
                    "Element IDs must be unique within the architecture".to_string(),
                ];

                diagnostics.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_DUPLICATE_ID,
                        Severity::Error,
                        msg,
                        loc.clone(),
                    )
                    .with_suggestions(suggestions),
                );
            } else {
                seen_ids.insert(id.to_string(), loc.clone());
            }
        };

        // Check top-level elements for duplicates
        for item in &program.items {
            if let TopLevelItem::ElementDef(elem) = item {
                check_id(&elem.assignment.name, &elem.location);

                // Check nested elements
                if let Some(body) = &elem.assignment.body {
                    check_nested_elements(body, &elem.assignment.name, &mut check_id);
                }
            }
        }

        diagnostics
    }
}

/// Recursively check nested elements for duplicate IDs
fn check_nested_elements<F>(
    body: &sruja_language::ElementDefBody,
    parent_fqn: &str,
    check_id: &mut F,
) where
    F: FnMut(&str, &SourceLocation),
{
    for item in &body.items {
        match item {
            ElementDefBodyItem::ElementDef(elem) => {
                let fqn = format!("{}.{}", parent_fqn, elem.assignment.name);
                check_id(&fqn, &elem.location);

                // Recursively check deeper nested elements
                if let Some(nested_body) = &elem.assignment.body {
                    check_nested_elements(nested_body, &fqn, check_id);
                }
            }
            _ => {}
        }
    }
}
