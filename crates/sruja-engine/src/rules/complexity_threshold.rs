//! Complexity Threshold Rule
//!
//! Validates that architecture graph complexity is within acceptable thresholds.
//! Based on talks: "Cyclomatic complexity above a threshold triggers mandatory human review"

use crate::validator::Rule;
use crate::DomainSchema;
use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::Program;

const COMPLEXITY_THRESHOLD: usize = 6;
const CRITICAL_COMPLEXITY_THRESHOLD: usize = 10;

pub struct ComplexityThresholdRule;

impl Rule for ComplexityThresholdRule {
    fn name(&self) -> &str {
        "ComplexityThreshold"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let item_count = program.items.len();
        if item_count < 2 {
            return vec![];
        }

        let mut diagnostics = Vec::new();
        let loc = SourceLocation::new("architecture".to_string(), 1, 1);

        if item_count >= CRITICAL_COMPLEXITY_THRESHOLD {
            diagnostics.push(
                Diagnostic::new(
                    "complexity.critical",
                    Severity::Warning,
                    format!(
                        "Architecture has {} elements (critical complexity). Mandatory human review required before merging.",
                        item_count
                    ),
                    loc,
                )
                .with_suggestions(vec!["Consider breaking into smaller systems or containers.".to_string()]),
            );
        } else if item_count >= COMPLEXITY_THRESHOLD {
            diagnostics.push(
                Diagnostic::new(
                    "complexity.high",
                    Severity::Warning,
                    format!(
                        "Architecture has {} elements (high complexity). Human review recommended.",
                        item_count
                    ),
                    loc,
                )
                .with_suggestions(vec!["Review dependencies between components.".to_string()]),
            );
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainSchema;

    #[test]
    fn low_complexity_no_warning() {
        let rule = ComplexityThresholdRule;
        let program = sruja_language::Parser::new("test.sruja".to_string())
            .parse("S = system \"System\" {}")
            .expect("parse");
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(diags.is_empty(), "Low complexity should not warn");
    }

    #[test]
    fn empty_program_no_warning() {
        let rule = ComplexityThresholdRule;
        let program = Program::default();
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(diags.is_empty());
    }
}
