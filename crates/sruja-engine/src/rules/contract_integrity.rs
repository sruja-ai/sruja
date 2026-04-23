//! API contract integrity validation rule

use crate::DomainSchema;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{ElementDefBodyItem, Program, TopLevelItem, Contract};

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
            format!("Contract '{}' is empty (no inputs, outputs, or errors).", c.name),
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
            format!("Contract '{}' has no behavioral constraints defined.", c.name),
            c.location.clone(),
        ));
    }
}
