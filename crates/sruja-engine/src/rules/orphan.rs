//! Orphan element detection rule
//!
//! Detects elements that are not referenced by any relations.

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
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

        // TODO: Collect all elements and relations from program
        // TODO: Build set of used elements from relations
        // TODO: Report elements that are defined but not used

        // For now, this is a placeholder
        diagnostics
    }
}
