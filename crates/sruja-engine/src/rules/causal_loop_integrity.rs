//! Causal loop integrity validation rule
//!
//! Validates that causal loops are properly formed with valid variables and relationships.

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::ast::TopLevelItem;
use sruja_language::Program;

use crate::validator::Rule;

/// Rule that validates causal loop integrity
pub struct CausalLoopIntegrityRule;

impl Rule for CausalLoopIntegrityRule {
    fn name(&self) -> &str {
        "Causal Loop Integrity"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check each causal loop
        for item in &program.items {
            if let TopLevelItem::CausalLoop(cl) = item {
                // Collect all variable IDs defined in this causal loop
                let mut variable_ids: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for var in &cl.variables {
                    variable_ids.insert(var.id.clone());
                }

                // Validate: Causal loop should have at least one variable
                if cl.variables.is_empty() && cl.relationships.is_empty() {
                    diagnostics.push(Diagnostic::new(
                        sruja_diagnostics::codes::CODE_MISSING_FIELD,
                        Severity::Error,
                        format!(
                            "Causal loop '{}' must have at least one variable or relationship",
                            cl.id
                        ),
                        cl.location.clone(),
                    ).with_suggestions(vec![
                        "Add variables to the causal loop body".to_string(),
                        "Example: causal_loop Loop1 { variable X; variable Y; X -> Y positive }".to_string(),
                    ]));
                }

                // Validate: All variables have unique IDs
                let mut seen_ids: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for var in &cl.variables {
                    if seen_ids.contains(&var.id) {
                        diagnostics.push(
                            Diagnostic::new(
                                sruja_diagnostics::codes::CODE_DUPLICATE_ID,
                                Severity::Error,
                                format!(
                                    "Duplicate variable ID '{}' in causal loop '{}'",
                                    var.id, cl.id
                                ),
                                cl.location.clone(),
                            )
                            .with_suggestions(vec![
                                "Use unique IDs for each variable in the causal loop".to_string(),
                            ]),
                        );
                    }
                    seen_ids.insert(var.id.clone());
                }

                // Validate: All relationships reference valid variables
                for rel in &cl.relationships {
                    if !variable_ids.contains(&rel.from) {
                        diagnostics.push(
                            Diagnostic::new(
                                sruja_diagnostics::codes::CODE_UNDEFINED_REF,
                                Severity::Error,
                                format!(
                                "Causal loop '{}' relationship references undefined variable '{}'",
                                cl.id, rel.from
                            ),
                                cl.location.clone(),
                            )
                            .with_suggestions(vec![
                                format!(
                                    "Define the variable '{}' in the causal loop body",
                                    rel.from
                                ),
                                "Check for typos in the variable name".to_string(),
                            ]),
                        );
                    }

                    if !variable_ids.contains(&rel.to) {
                        diagnostics.push(
                            Diagnostic::new(
                                sruja_diagnostics::codes::CODE_UNDEFINED_REF,
                                Severity::Error,
                                format!(
                                    "Causal loop '{}' relationship references undefined variable '{}'",
                                    cl.id, rel.to
                                ),
                                cl.location.clone(),
                            )
                            .with_suggestions(vec![
                                format!("Define the variable '{}' in the causal loop body", rel.to),
                                "Check for typos in the variable name".to_string(),
                            ]),
                        );
                    }

                    // Validate: Relationships have valid polarities
                    // Note: polarity is an enum, so it's always valid structurally,
                    // but we can warn if it's missing or if certain patterns are detected
                    if let Some(effect) = &rel.effect {
                        if effect.trim().is_empty() {
                            diagnostics.push(
                                Diagnostic::new(
                                    sruja_diagnostics::codes::CODE_MISSING_FIELD,
                                    Severity::Warning,
                                    format!(
                                    "Causal loop '{}' relationship has empty effect description",
                                    cl.id
                                ),
                                    cl.location.clone(),
                                )
                                .with_suggestions(vec![
                                    "Add a description of the causal effect".to_string(),
                                    "Example: X -> Y positive 'increases'".to_string(),
                                ]),
                            );
                        }
                    }
                }

                // Validate: Causal loop with relationships should form a connected graph
                if !cl.relationships.is_empty() && cl.variables.is_empty() {
                    diagnostics.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_MISSING_FIELD,
                            Severity::Warning,
                            format!(
                                "Causal loop '{}' has relationships but no variables defined",
                                cl.id
                            ),
                            cl.location.clone(),
                        )
                        .with_suggestions(vec![
                            "Define variables for each node in the causal loop".to_string(),
                            "Example: causal_loop Loop1 { variable X; X -> X positive }"
                                .to_string(),
                        ]),
                    );
                }
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_loop_with_valid_structure() {
        let input = r#"
causal_loop Loop1 balancing "Test Loop" {
    variable Stock "Stock Variable"
    variable Flow "Flow Variable"
    Stock -> Flow "increases" polarity + delay "1s"
    Flow -> Stock "decreases" polarity - delay "1s"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = CausalLoopIntegrityRule;
        let diagnostics = rule.validate(&program);

        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for valid causal loop: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_causal_loop_missing_variables_and_relationships() {
        let input = r#"
causal_loop Loop1 reinforcing "Empty Loop" {
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = CausalLoopIntegrityRule;
        let diagnostics = rule.validate(&program);

        assert!(
            diagnostics.iter().any(|d| d
                .message
                .contains("must have at least one variable or relationship")),
            "Expected error for empty causal loop: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_causal_loop_undefined_variable_reference() {
        let input = r#"
causal_loop Loop1 balancing "Invalid Ref Loop" {
    variable Stock "Stock Variable"
    Stock -> Flow "increases" polarity +
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = CausalLoopIntegrityRule;
        let diagnostics = rule.validate(&program);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("undefined variable") && d.message.contains("Flow")),
            "Expected error for undefined variable reference: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_causal_loop_duplicate_variable_ids() {
        let input = r#"
causal_loop Loop1 reinforcing "Duplicate Variables" {
    variable Stock "Stock 1"
    variable Stock "Stock 2"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = CausalLoopIntegrityRule;
        let diagnostics = rule.validate(&program);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("Duplicate variable ID") && d.message.contains("Stock")),
            "Expected error for duplicate variable ID: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_causal_loop_relationships_without_variables() {
        let input = r#"
causal_loop Loop1 reinforcing "No Variables" {
    A -> B "effect" polarity +
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = CausalLoopIntegrityRule;
        let diagnostics = rule.validate(&program);

        // Should warn about having relationships but no variables
        assert!(
            diagnostics.iter().any(|d| d
                .message
                .contains("has relationships but no variables defined")),
            "Expected warning for relationships without variables: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_causal_loop_single_variable_self_loop() {
        let input = r#"
causal_loop Loop1 reinforcing "Self Loop" {
    variable Population "Population"
    Population -> Population "grows" polarity +
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = CausalLoopIntegrityRule;
        let diagnostics = rule.validate(&program);

        // Self-referencing loops are valid in systems thinking
        assert!(
            diagnostics.is_empty(),
            "Self-referencing loops should be valid: {:?}",
            diagnostics
        );
    }
}
