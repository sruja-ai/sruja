//! API contract integrity validation rule

use crate::DomainSchema;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{Contract, ElementDefBodyItem, Program, TopLevelItem};

use crate::validator::Rule;

pub struct ContractIntegrityRule;

impl Rule for ContractIntegrityRule {
    fn name(&self) -> &str {
        "Contract Integrity"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &program.items {
            if let TopLevelItem::ElementDef(elem) = item {
                if let Some(body) = &elem.assignment.body {
                    validate_body(body, &mut diagnostics);
                }
            }
        }

        diagnostics
    }
}

fn validate_body(body: &sruja_language::ElementDefBody, diagnostics: &mut Vec<Diagnostic>) {
    for c in &body.contracts {
        validate_contract(c, diagnostics);
    }

    for item in &body.items {
        if let ElementDefBodyItem::ElementDef(elem) = item {
            if let Some(nested_body) = &elem.assignment.body {
                validate_body(nested_body, diagnostics);
            }
        }
    }
}

fn validate_contract(c: &Contract, diagnostics: &mut Vec<Diagnostic>) {
    // E321: Contract empty
    if c.inputs.is_empty() && c.outputs.is_empty() && c.errors.is_empty() {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_CONTRACT_EMPTY,
            Severity::Error,
            format!(
                "Contract '{}' is empty (no inputs, outputs, or errors).",
                c.name
            ),
            c.location.clone(),
        ));
    }

    // W321: No input fields
    if c.inputs.is_empty() {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_CONTRACT_NO_INPUTS,
            Severity::Warning,
            format!("Contract '{}' has no input fields defined.", c.name),
            c.location.clone(),
        ));
    }

    // W322: No error definitions
    if c.errors.is_empty() {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_CONTRACT_NO_ERRORS,
            Severity::Warning,
            format!("Contract '{}' has no error responses defined.", c.name),
            c.location.clone(),
        ));
    }

    // W323: No constraints
    if c.constraints.is_empty() {
        diagnostics.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_CONTRACT_NO_CONSTRAINTS,
            Severity::Info,
            format!(
                "Contract '{}' has no behavioral constraints defined.",
                c.name
            ),
            c.location.clone(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainSchema;
    use sruja_diagnostics::codes::{
        CODE_CONTRACT_EMPTY, CODE_CONTRACT_NO_CONSTRAINTS, CODE_CONTRACT_NO_ERRORS,
        CODE_CONTRACT_NO_INPUTS,
    };
    use sruja_language::Parser;

    fn parse_program(input: &str) -> Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse")
    }

    fn codes(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn rule_name_is_contract_integrity() {
        assert_eq!(ContractIntegrityRule.name(), "Contract Integrity");
    }

    #[test]
    fn empty_contract_emits_error_and_warnings() {
        let program = parse_program(
            r#"
Api = component "API" {
  contract "Empty" {
    description "No fields defined"
  }
}
"#,
        );
        let rule = ContractIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        let found = codes(&diags);
        assert!(found.contains(&CODE_CONTRACT_EMPTY));
        assert!(found.contains(&CODE_CONTRACT_NO_INPUTS));
        assert!(found.contains(&CODE_CONTRACT_NO_ERRORS));
        assert!(found.contains(&CODE_CONTRACT_NO_CONSTRAINTS));
    }

    #[test]
    fn complete_contract_has_no_diagnostics() {
        let program = parse_program(
            r#"
Api = component "API" {
  contract "GetUser" {
    description "Fetch user"
    input { user_id "uuid" }
    output { name "string" }
    error { "NOT_FOUND" "missing user" }
    constraint "idempotent"
  }
}
"#,
        );
        let rule = ContractIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        assert!(diags.is_empty(), "complete contract should pass: {diags:?}");
    }

    #[test]
    fn contract_with_outputs_but_no_inputs_only_warns_on_inputs_and_errors() {
        let program = parse_program(
            r#"
Api = component "API" {
  contract "Notify" {
    output { status "string" }
    constraint "at-most-once"
  }
}
"#,
        );
        let rule = ContractIntegrityRule;
        let diags = rule.validate(&program, &DomainSchema::architecture());
        let found = codes(&diags);
        assert!(!found.contains(&CODE_CONTRACT_EMPTY));
        assert!(found.contains(&CODE_CONTRACT_NO_INPUTS));
        assert!(found.contains(&CODE_CONTRACT_NO_ERRORS));
    }
}
