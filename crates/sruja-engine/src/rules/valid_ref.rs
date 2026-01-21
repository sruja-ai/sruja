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

        // TODO: Collect all element IDs from program
        // TODO: Check all relation references point to valid elements
        // TODO: Report undefined references

        // For now, this is a placeholder
        diagnostics
    }
}
