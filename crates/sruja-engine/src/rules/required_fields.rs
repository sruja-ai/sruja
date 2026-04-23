use crate::DomainSchema;
use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, ElementKind, Program};

use crate::validator::Rule;

pub struct RequiredFieldsRule;

impl Rule for RequiredFieldsRule {
    fn name(&self) -> &str {
        "Required Fields"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);
        let mut diags: Vec<Diagnostic> = Vec::with_capacity(16);

        for (id, elem) in &elements {
            let kind = &elem.assignment.kind;
            if !kind_requires_description(kind) && !kind_requires_technology(kind) {
                continue;
            }

            let (desc_ok, tech_ok) = {
                let body = elem.assignment.body.as_ref();
                let desc_ok = body
                    .and_then(|b| b.description.as_deref())
                    .is_some_and(|s| !s.trim().is_empty());
                let tech_ok = body
                    .and_then(|b| b.technology.as_deref())
                    .is_some_and(|s| !s.trim().is_empty());
                (desc_ok, tech_ok)
            };

            if kind_requires_description(kind) && !desc_ok {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_MISSING_FIELD,
                        Severity::Error,
                        format!("Missing required field `description` on {} `{}`", kind, id),
                        elem.assignment.location.clone(),
                    )
                    .with_suggestions(vec![r#"Add: description "..." "#.trim().to_string()]),
                );
            }

            if kind_requires_technology(kind) && !tech_ok {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_MISSING_FIELD,
                        Severity::Error,
                        format!("Missing required field `technology` on {} `{}`", kind, id),
                        elem.assignment.location.clone(),
                    )
                    .with_suggestions(vec![r#"Add: technology "..." "#.trim().to_string()]),
                );
            }
        }

        diags
    }
}

fn kind_requires_description(kind: &ElementKind) -> bool {
    matches!(
        kind,
        ElementKind::Container
            | ElementKind::Component
            | ElementKind::Database
            | ElementKind::DataStore
    )
}

fn kind_requires_technology(kind: &ElementKind) -> bool {
    matches!(
        kind,
        ElementKind::Container | ElementKind::Database | ElementKind::DataStore
    )
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    fn validate(input: &str) -> Vec<Diagnostic> {
        let program = Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse");
        RequiredFieldsRule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn reports_missing_description_for_container() {
        let diags = validate(
            r#"
A = container "A" {
  technology "Rust"
}
"#,
        );
        assert!(diags
            .iter()
            .any(|d| d.code == sruja_diagnostics::codes::CODE_MISSING_FIELD));
    }

    #[test]
    fn reports_missing_technology_for_database() {
        let diags = validate(
            r#"
DB = database "DB" {
  description "Data store"
}
"#,
        );
        assert!(diags
            .iter()
            .any(|d| d.message.contains("technology") && d.severity == Severity::Error));
    }
}
