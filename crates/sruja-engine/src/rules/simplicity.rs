//! Simplicity validation rule
//!
//! Validates that users are using the right perspective for their modeling goals.

use sruja_diagnostics::Diagnostic;
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that validates simplicity guidance
pub struct SimplicityRule;

impl Rule for SimplicityRule {
    fn name(&self) -> &str {
        "SimplicityGuidance"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        // Currently returns empty diagnostics as DDD features are deferred
        // This matches the Go implementation which also has commented-out logic
        vec![]
    }
}
