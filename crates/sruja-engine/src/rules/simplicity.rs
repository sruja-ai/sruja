//! Simplicity validation rule
//!
//! Validates that users are using the right perspective for their modeling goals.

use crate::DomainSchema;
use sruja_diagnostics::Diagnostic;
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that validates simplicity guidance
pub struct SimplicityRule;

impl Rule for SimplicityRule {
    fn name(&self) -> &str {
        "SimplicityGuidance"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        // Currently returns empty diagnostics as DDD features are deferred
        // This matches the Go implementation which also has commented-out logic
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    #[test]
    fn simplicity_rule_name() {
        let rule = SimplicityRule;
        assert_eq!(rule.name(), "SimplicityGuidance");
    }

    #[test]
    fn simplicity_rule_empty_program_returns_no_diagnostics() {
        let rule = SimplicityRule;
        let program = Program::default();
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(diags.is_empty());
    }

    #[test]
    fn simplicity_rule_non_empty_program_returns_empty_diagnostics_current_behavior() {
        let rule = SimplicityRule;
        let program = Parser::new("test.sruja".to_string())
            .parse("S = system \"My System\" {}")
            .expect("parse");
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(
            diags.is_empty(),
            "SimplicityRule currently defers DDD logic"
        );
    }
}
