//! Custom Constraints validation rule
//!
//! Evaluates natural-language-like constraints defined in the `constraints` block.
//!
//! Supported patterns:
//! - `<source_regex> -> <target_regex> forbidden`

use crate::DomainSchema;
use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, Program, TopLevelItem};

use crate::validator::Rule;

pub struct CustomConstraintsRule;

impl Rule for CustomConstraintsRule {
    fn name(&self) -> &str {
        "Custom Constraints"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut constraints = Vec::new();

        for item in &program.items {
            if let TopLevelItem::Constraints(block) = item {
                for entry in &block.entries {
                    constraints.push((entry.value.clone(), block.location.clone()));
                }
            }
        }

        if constraints.is_empty() {
            return vec![];
        }

        let (elements, relations) = collect_elements(program);

        for (constraint_str, _loc) in constraints {
            let s = constraint_str.trim().to_lowercase();

            // Simple pattern: "<source> -> <target> forbidden"
            if s.contains("->") && s.ends_with("forbidden") {
                let parts: Vec<&str> = s.split("->").collect();
                if parts.len() == 2 {
                    let source_pattern = parts[0].trim();
                    let target_part = parts[1].replace("forbidden", "").trim().to_string();
                    let target_pattern = target_part.as_str();

                    for relation in &relations {
                        let from_id = relation.from.as_string().to_lowercase();
                        let to_id = relation.to.as_string().to_lowercase();

                        let from_kind = elements
                            .get(&relation.from.as_string())
                            .map_or(String::new(), |e| {
                                e.assignment.kind.to_string().to_lowercase()
                            });

                        let to_kind = elements
                            .get(&relation.to.as_string())
                            .map_or(String::new(), |e| {
                                e.assignment.kind.to_string().to_lowercase()
                            });

                        let from_match =
                            from_id.contains(source_pattern) || from_kind.contains(source_pattern);
                        let to_match =
                            to_id.contains(target_pattern) || to_kind.contains(target_pattern);

                        if from_match && to_match {
                            diagnostics.push(
                                Diagnostic::new(
                                    sruja_diagnostics::codes::CODE_CUSTOM_CONSTRAINT,
                                    Severity::Error,
                                    format!("Custom constraint violated: {}", constraint_str),
                                    relation.location.clone(),
                                )
                                .with_suggestions(vec![format!(
                                    "Remove the relation {} -> {}",
                                    relation.from.as_string(),
                                    relation.to.as_string()
                                )]),
                            );
                        }
                    }
                }
            }
        }

        diagnostics
    }
}
