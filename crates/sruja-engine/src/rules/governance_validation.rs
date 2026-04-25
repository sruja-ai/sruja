//! Governance validation rule
//!
//! Mirrors Go `GovernanceValidationRule`:
//! - Ensures unique IDs for governance-related kinds:
//!   requirement, adr, policy, scenario, flow, contract
//! - Treats "story" as "scenario"

use crate::DomainSchema;
use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, ElementKind, Program};

use crate::validator::Rule;

pub struct GovernanceValidationRule;

impl Rule for GovernanceValidationRule {
    fn name(&self) -> &str {
        "Governance Validation"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);

        // kind -> id -> first definition
        let mut seen: HashMap<&'static str, HashMap<String, FirstSeen>> = HashMap::new();
        for k in [
            "requirement",
            "adr",
            "policy",
            "scenario",
            "flow",
            "contract",
        ] {
            seen.insert(k, HashMap::new());
        }

        let mut diags: Vec<Diagnostic> = Vec::new();

        for elem in elements.values() {
            let kind_key = normalize_governance_kind(&elem.assignment.kind);
            let Some(kind_key) = kind_key else { continue };

            let id = elem.assignment.name.clone();
            if id.is_empty() {
                continue;
            }

            let loc = elem.location.clone();
            let Some(entry) = seen.get_mut(kind_key) else {
                log::warn!(
                    "kind_key '{}' not in governance map - this is a bug",
                    kind_key
                );
                continue;
            };
            if let Some(existing) = entry.get(&id) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_DUPLICATE_IDENTIFIER,
                        Severity::Error,
                        format!("Duplicate {} ID '{}'", kind_key, id),
                        loc.clone(),
                    )
                    .with_suggestions(vec![
                        format!(
                            "{} '{}' is already defined at line {}",
                            kind_key, id, existing.line
                        ),
                        format!("Use a unique ID for each {}", kind_key),
                    ]),
                );
            } else {
                entry.insert(id, FirstSeen { line: loc.line });
            }
        }

        diags
    }
}

struct FirstSeen {
    line: u32,
}

fn normalize_governance_kind(kind: &ElementKind) -> Option<&'static str> {
    match kind {
        ElementKind::Requirement => Some("requirement"),
        ElementKind::Adr => Some("adr"),
        ElementKind::Policy => Some("policy"),
        ElementKind::Scenario => Some("scenario"),
        ElementKind::Story => Some("scenario"),
        ElementKind::Flow => Some("flow"),
        // Contract isn't in our DSL yet, but Go reserves it.
        ElementKind::Custom(k) if k == "contract" => Some("contract"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainSchema;
    use sruja_language::Parser;

    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        let rule = GovernanceValidationRule;
        rule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn empty_program_returns_no_diagnostics() {
        let diags = validate_program("");
        assert!(diags.is_empty());
    }

    #[test]
    fn unique_requirements_pass() {
        let input = r#"
R1 = requirement functional "Must support X"
R2 = requirement functional "Must support Y"
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn unique_adrs_pass() {
        let input = r#"
ADR001 = adr "Use Rust" { status "Accepted" }
ADR002 = adr "JSON Format" { status "Accepted" }
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn non_governance_elements_ignored() {
        let input = r#"
api = system "API"
api = container "API"
"#;
        let diags = validate_program(input);
        assert!(
            diags.is_empty(),
            "systems/containers don't require unique IDs for governance"
        );
    }
}
