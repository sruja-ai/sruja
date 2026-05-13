//! Relation integrity checks (duplicate edges, self-references).

use crate::DomainSchema;
use std::collections::HashSet;

use sruja_diagnostics::{codes, Diagnostic, Severity};
use sruja_language::{collect_elements, Program};

use crate::validator::Rule;

/// Detects duplicate relations between the same endpoints and self-referential edges.
pub struct RelationIntegrityRule;

impl Rule for RelationIntegrityRule {
    fn name(&self) -> &str {
        "Relation Integrity"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let (_elements, relations) = collect_elements(program);
        let mut diags = Vec::new();
        let mut seen_pairs: HashSet<(String, String)> = HashSet::new();

        for rel in &relations {
            let from = rel.from.as_string();
            let to = rel.to.as_string();

            if from == to {
                diags.push(
                    Diagnostic::new(
                        codes::CODE_INVALID_RELATION,
                        Severity::Error,
                        format!("Self-referential relation `{from} -> {to}` has no effect"),
                        rel.location.clone(),
                    )
                    .with_suggestions(vec![
                        "Remove the relation or connect to a different element".to_string(),
                        "If you meant recursion or internal calls, describe that in documentation instead of a self-edge".to_string(),
                    ]),
                );
            }

            let key = (from, to);
            if !seen_pairs.insert(key.clone()) {
                let (from, to) = key;
                diags.push(
                    Diagnostic::new(
                        codes::CODE_INVALID_RELATION,
                        Severity::Error,
                        format!("Duplicate relation between `{from}` and `{to}`"),
                        rel.location.clone(),
                    )
                    .with_suggestions(vec![
                        "Keep a single relation between these elements".to_string(),
                        "If the edges differ, merge the labels into one quoted label".to_string(),
                    ]),
                );
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    fn validate(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja");
        let program = parser.parse(input).expect("parse");
        RelationIntegrityRule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn flags_self_relation() {
        let input = r#"
A = system "A"
A -> A "loops"
"#;
        let diags = validate(input);
        assert!(diags.iter().any(|d| d.code == codes::CODE_INVALID_RELATION));
        assert!(diags.iter().any(|d| d.message.contains("Self-referential")));
    }

    #[test]
    fn flags_duplicate_pair() {
        let input = r#"
A = system "A"
B = system "B"
A -> B "x"
A -> B "y"
"#;
        let diags = validate(input);
        assert!(diags.iter().any(|d| d.code == codes::CODE_INVALID_RELATION));
        assert!(diags
            .iter()
            .any(|d| d.message.contains("Duplicate relation")));
    }
}
