//! Feedback loop integrity validation rule
//!
//! Validates that feedback loops are properly formed and have valid relationships.

use crate::DomainSchema;
use sruja_diagnostics::Diagnostic;
use sruja_language::{ast::TopLevelItem, collect_elements, Program};

use crate::utils::element_exists;
use crate::validator::Rule;

/// Rule that validates feedback loop integrity
pub struct FeedbackLoopIntegrityRule;

impl Rule for FeedbackLoopIntegrityRule {
    fn name(&self) -> &str {
        "Feedback Loop Integrity"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect all elements to check for valid references
        let (elements_map, _) = collect_elements(program);

        // Check each feedback loop
        for item in &program.items {
            if let TopLevelItem::FeedbackLoop(fl) = item {
                // Validate: Feedback loop must have at least one relationship
                if fl.relationships.is_empty() {
                    diagnostics.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_MISSING_FIELD,
                            sruja_diagnostics::Severity::Error,
                            format!(
                                "Feedback loop '{}' must have at least one relationship",
                                fl.id
                            ),
                            fl.location.clone(),
                        )
                        .with_suggestions(vec![
                            "Add relationships to the feedback loop body".to_string(),
                            "Example: feedback Loop1 { A -> B; B -> A }".to_string(),
                        ]),
                    );
                }

                // Validate: All relationships in feedback loop must reference valid elements
                for rel in &fl.relationships {
                    let from_name = rel.from.as_string();
                    let to_name = rel.to.as_string();

                    if !element_exists(&elements_map, &from_name) {
                        diagnostics.push(Diagnostic::new(
                            sruja_diagnostics::codes::CODE_UNDEFINED_REF,
                            sruja_diagnostics::Severity::Error,
                            format!(
                                "Feedback loop '{}' references undefined element '{}' in 'from' field",
                                fl.id, from_name
                            ),
                            rel.location.clone(),
                        ).with_suggestions(vec![
                            format!("Define the element '{}' before using it in the feedback loop", from_name),
                            "Check for typos in the element name".to_string(),
                            "If the element is nested, try referencing it using a fully qualified name (e.g., Parent.Child)".to_string(),
                        ]));
                    }

                    if !element_exists(&elements_map, &to_name) {
                        diagnostics.push(Diagnostic::new(
                            sruja_diagnostics::codes::CODE_UNDEFINED_REF,
                            sruja_diagnostics::Severity::Error,
                            format!(
                                "Feedback loop '{}' references undefined element '{}' in 'to' field",
                                fl.id, to_name
                            ),
                            rel.location.clone(),
                        ).with_suggestions(vec![
                            format!("Define the element '{}' before using it in the feedback loop", to_name),
                            "Check for typos in the element name".to_string(),
                            "If the element is nested, try referencing it using a fully qualified name (e.g., Parent.Child)".to_string(),
                        ]));
                    }
                }

                // Validate: Feedback loop should form a cycle (at least one path from start back to start)
                if !fl.relationships.is_empty() {
                    if !forms_cycle(&fl.relationships) {
                        diagnostics.push(
                            Diagnostic::new(
                                sruja_diagnostics::codes::CODE_CYCLE_DETECTED,
                                sruja_diagnostics::Severity::Warning,
                                format!("Feedback loop '{}' may not form a complete cycle", fl.id),
                                fl.location.clone(),
                            )
                            .with_suggestions(vec![
                                "Ensure the relationships form a closed loop".to_string(),
                                "Example: A -> B -> C -> A forms a cycle".to_string(),
                            ]),
                        );
                    }
                }
            }
        }

        diagnostics
    }
}

/// Check if the relationships form at least one cycle
fn forms_cycle(relations: &[sruja_language::ast::Relation]) -> bool {
    use std::collections::{HashMap, HashSet};

    // Build adjacency list
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for rel in relations {
        let from = rel.from.as_string();
        let to = rel.to.as_string();
        adj.entry(from).or_insert_with(Vec::new).push(to);
    }

    // Use DFS to find cycles
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for node in adj.keys() {
        if !visited.contains(node) {
            if dfs_has_cycle(node, &adj, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }

    false
}

fn dfs_has_cycle(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if dfs_has_cycle(neighbor, adj, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(neighbor) {
                return true;
            }
        }
    }

    rec_stack.remove(node);
    false
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;

    #[test]
    fn test_feedback_loop_with_valid_cycle() {
        let input = r#"
A = system "System A"
B = system "System B"

feedback Loop1 reinforcing "Test Loop" {
    A -> B "calls"
    B -> A "returns"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = FeedbackLoopIntegrityRule;
        let diagnostics = rule.validate(&program, &DomainSchema::architecture());

        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for valid feedback loop: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_feedback_loop_missing_relationships() {
        let input = r#"
A = system "System A"

feedback Loop1 reinforcing "Empty Loop" {
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = FeedbackLoopIntegrityRule;
        let diagnostics = rule.validate(&program, &DomainSchema::architecture());

        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("must have at least one relationship")),
            "Expected error for feedback loop without relationships: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_feedback_loop_invalid_element_reference() {
        let input = r#"
A = system "System A"

feedback Loop1 reinforcing "Invalid Ref Loop" {
    A -> B "calls"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = FeedbackLoopIntegrityRule;
        let diagnostics = rule.validate(&program, &DomainSchema::architecture());

        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("undefined element") && d.message.contains("B")),
            "Expected error for undefined element reference: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_feedback_loop_no_cycle_warning() {
        let input = r#"
A = system "System A"
B = system "System B"

feedback Loop1 reinforcing "Non-Cyclic Loop" {
    A -> B "calls"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = FeedbackLoopIntegrityRule;
        let diagnostics = rule.validate(&program, &DomainSchema::architecture());

        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("may not form a complete cycle")),
            "Expected warning for non-cyclic feedback loop: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_feedback_loop_qualified_element_references() {
        let input = r#"
Shop = system "Shop" {
    API = container "API"
    DB = database "Database"
}

feedback Loop1 reinforcing "Nested Loop" {
    Shop.API -> Shop.DB "reads"
    Shop.DB -> Shop.API "updates"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = FeedbackLoopIntegrityRule;
        let diagnostics = rule.validate(&program, &DomainSchema::architecture());

        // Qualified references should work
        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
            .count();
        assert!(
            error_count == 0,
            "Qualified references should be valid, got {} errors: {:?}",
            error_count,
            diagnostics
        );
    }

    #[test]
    fn test_feedback_loop_allows_leaf_references_for_nested_elements() {
        let input = r#"
Shop = system "Shop" {
    API = container "API"
    DB = database "Database"
}

feedback Loop1 reinforcing "Nested Loop (Leaf)" {
    API -> DB "reads"
    DB -> API "updates"
}
"#;
        let parser = sruja_language::Parser::new("test.sruja".to_string());
        let program = parser.parse(input).unwrap();

        let rule = FeedbackLoopIntegrityRule;
        let diagnostics = rule.validate(&program, &DomainSchema::architecture());

        assert!(
            diagnostics.is_empty(),
            "Leaf references should be valid for nested elements: {:?}",
            diagnostics
        );
    }
}
