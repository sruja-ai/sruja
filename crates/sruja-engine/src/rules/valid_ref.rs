//! Valid reference validation rule
//!
//! Ensures all references in relations point to valid elements.

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that validates all references point to existing elements
pub struct ValidRefRule;

impl Rule for ValidRefRule {
    fn name(&self) -> &str {
        "Valid References"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect all element IDs from program
        let (elements, relations) = sruja_language::collect_elements(program);
        let element_ids: HashSet<String> = elements.keys().cloned().collect();

        // Check all relation references point to valid elements
        for rel in &relations {
            let from = rel.from.as_string();
            let to = rel.to.as_string();

            if !element_ids.contains(&from) {
                diagnostics.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_UNDEFINED_REF,
                    Severity::Error,
                    format!("Reference '{}' in relation does not exist", from),
                    rel.location.clone(),
                ).with_suggestions(vec![
                    format!("Element '{}' must be defined before it can be referenced", from),
                    "Check for typos or missing element definitions".to_string(),
                ]));
            }

            if !element_ids.contains(&to) {
                diagnostics.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_UNDEFINED_REF,
                    Severity::Error,
                    format!("Reference '{}' in relation does not exist", to),
                    rel.location.clone(),
                ).with_suggestions(vec![
                    format!("Element '{}' must be defined before it can be referenced", to),
                    "Check for typos or missing element definitions".to_string(),
                ]));
            }
        }

        diagnostics
    }
}
