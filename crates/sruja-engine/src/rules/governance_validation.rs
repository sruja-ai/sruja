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

        let mut diags: Vec<Diagnostic> = Vec::new();

        // Validate requirement priorities directly from AST
        for item in &program.items {
            if let sruja_language::TopLevelItem::Requirement(req) = item {
                if let Some(priority) = &req.priority {
                    let p = priority.to_lowercase();
                    if p != "must" && p != "should" && p != "could" && p != "wont" {
                        diags.push(
                            Diagnostic::new(
                                sruja_diagnostics::codes::CODE_BEST_PRACTICE,
                                Severity::Warning,
                                format!(
                                    "Invalid priority '{}' for requirement '{}'. Expected: must, should, could, or wont",
                                    priority, req.id
                                ),
                                req.location.clone(),
                            )
                            .with_suggestions(vec![
                                "Valid priorities: must, should, could, wont (MoSCoW)".to_string(),
                            ]),
                        );
                    }
                }

                // Accepted requirements must have affects (traceability)
                let is_accepted = req
                    .status
                    .as_deref()
                    .is_some_and(|s| s.to_lowercase() == "accepted");
                if is_accepted && req.affects.is_empty() {
                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_BEST_PRACTICE,
                            Severity::Warning,
                            format!(
                                "Accepted requirement '{}' has no 'affects' — add traceability to architecture elements",
                                req.id
                            ),
                            req.location.clone(),
                        )
                        .with_suggestions(vec![
                            "Add 'affects System.Container' to link this requirement to code".to_string(),
                        ]),
                    );
                }
            }
        }

        let (elements, _relations) = collect_elements(program);

        // Validate requirement affects references exist in the architecture
        for item in &program.items {
            if let sruja_language::TopLevelItem::Requirement(req) = item {
                for affect in &req.affects {
                    let exists = elements.contains_key(affect)
                        || elements
                            .keys()
                            .any(|k| k.starts_with(&format!("{affect}.")));
                    if !exists {
                        diags.push(
                            Diagnostic::new(
                                sruja_diagnostics::codes::CODE_INVALID_PROPERTY,
                                Severity::Warning,
                                format!(
                                    "Requirement '{}' affects '{}' which is not found in the architecture",
                                    req.id, affect
                                ),
                                req.location.clone(),
                            )
                            .with_suggestions(vec![
                                format!("Ensure '{}' is defined in a .sruja file", affect),
                                "Check spelling and casing of the element ID".to_string(),
                            ]),
                        );
                    }
                }
            }
        }

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

        for elem in elements.values() {
            let kind_key = normalize_governance_kind(&elem.assignment.kind);
            let Some(kind_key) = kind_key else { continue };

            let id = elem.assignment.name.clone();
            if id.is_empty() {
                continue;
            }

            let loc = elem.location.clone();
            let Some(entry) = seen.get_mut(kind_key) else {
                tracing::warn!(
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

    #[test]
    fn accepted_requirement_without_affects_warns() {
        let input = r#"
R1 = requirement functional "Must login" {
    status "accepted"
}
"#;
        let diags = validate_program(input);
        assert!(diags.iter().any(|d| d.message.contains("no 'affects'")));
    }

    #[test]
    fn accepted_requirement_with_affects_passes() {
        let input = r#"
MySys = system "My System"
MySys.API = container "API"
R1 = requirement functional "Must login" {
    status "accepted"
    affects MySys.API
}
"#;
        let diags = validate_program(input);
        assert!(!diags.iter().any(|d| d.message.contains("no 'affects'")));
    }

    #[test]
    fn requirement_affects_missing_element_warns() {
        let input = r#"
R1 = requirement functional "Must login" {
    affects NonExistent
}
"#;
        let diags = validate_program(input);
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("not found in the architecture")),
            "Expected 'not found' warning, got: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn invalid_priority_warns() {
        let input = r#"
R1 = requirement functional "Must login" {
    priority "critical"
}
"#;
        let diags = validate_program(input);
        assert!(diags.iter().any(|d| d.message.contains("Invalid priority")));
    }

    #[test]
    fn valid_priority_passes() {
        let input = r#"
R1 = requirement functional "Must login" {
    priority "must"
}
"#;
        let diags = validate_program(input);
        assert!(!diags.iter().any(|d| d.message.contains("Invalid priority")));
    }
}
