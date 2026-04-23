//! Scenario validation rule
//!
//! Validates behavioral flows (scenarios, stories, flows) similar to the Go engine:
//! - Each step must reference existing elements
//! - Enforces simple tag-based policies (e.g., external -> database is forbidden)

use crate::DomainSchema;
use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{
    collect_elements, ElementDef, ElementDefBodyItem, PolicyRuleAst, Program, ScenarioStep,
    TopLevelItem,
};

use crate::utils::{
    edge_is_excepted, element_exists, enforcement_to_severity, find_element,
    selector_matches_element,
};
use crate::validator::Rule;

/// Validates scenarios and flows.
pub struct ScenarioValidationRule;

impl Rule for ScenarioValidationRule {
    fn name(&self) -> &str {
        "Scenario Validation"
    }

    fn validate(&self, program: &Program, _schema: &DomainSchema) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);
        let policies: Vec<&sruja_language::Policy> = program
            .items
            .iter()
            .filter_map(|i| match i {
                TopLevelItem::Policy(p) => Some(p),
                _ => None,
            })
            .collect();
        let runner = ScenarioRunner::new(&elements, policies);

        let mut diags: Vec<Diagnostic> = Vec::new();

        for item in &program.items {
            match item {
                TopLevelItem::Scenario(s) => {
                    diags.extend(runner.validate_steps(&s.steps, &s.location));
                }
                TopLevelItem::Flow(f) => {
                    diags.extend(runner.validate_steps(&f.steps, &f.location));
                }
                TopLevelItem::ElementDef(elem) => {
                    diags.extend(validate_inline_steps_in_element(&runner, elem));
                }
                _ => {}
            }
        }

        diags
    }
}

fn validate_inline_steps_in_element(
    runner: &ScenarioRunner<'_>,
    elem: &ElementDef,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Some(body) = &elem.assignment.body else {
        return diags;
    };

    let inline_steps: Vec<ScenarioStep> = body
        .items
        .iter()
        .filter_map(|i| match i {
            ElementDefBodyItem::Step(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    if !inline_steps.is_empty() {
        diags.extend(runner.validate_steps(&inline_steps, &elem.location));
    }

    for item in &body.items {
        let ElementDefBodyItem::ElementDef(nested) = item else {
            continue;
        };
        diags.extend(validate_inline_steps_in_element(runner, nested));
    }

    diags
}

struct ScenarioRunner<'a> {
    elements: &'a HashMap<String, ElementDef>,
    policies: Vec<&'a sruja_language::Policy>,
}

impl<'a> ScenarioRunner<'a> {
    fn new(
        elements: &'a HashMap<String, ElementDef>,
        policies: Vec<&'a sruja_language::Policy>,
    ) -> Self {
        Self { elements, policies }
    }

    fn validate_steps(
        &self,
        steps: &[ScenarioStep],
        fallback_loc: &SourceLocation,
    ) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();

        for step in steps {
            let from_fqn = step
                .from
                .as_ref()
                .map(|q| q.as_string())
                .unwrap_or_default();
            let to_fqn = step.to.as_ref().map(|q| q.as_string()).unwrap_or_default();

            let from_exists = element_exists(self.elements, &from_fqn);
            let to_exists = element_exists(self.elements, &to_fqn);

            if !from_exists {
                diags.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                    Severity::Error,
                    format!("Step source '{}' not found in model", from_fqn),
                    fallback_loc.clone(),
                ));
            }

            if !to_exists {
                diags.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                    Severity::Error,
                    format!("Step target '{}' not found in model", to_fqn),
                    fallback_loc.clone(),
                ));
            }

            if !from_exists || !to_exists {
                continue;
            }

            // Policy enforcement (tag-based) - match Go example policy.
            diags.extend(self.check_policies(step, &from_fqn, &to_fqn, fallback_loc));
        }

        diags
    }

    fn check_policies(
        &self,
        _step: &ScenarioStep,
        from_fqn: &str,
        to_fqn: &str,
        loc: &SourceLocation,
    ) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();

        let Some(from_elem) = find_element(self.elements, from_fqn) else {
            return diags;
        };
        let Some(to_elem) = find_element(self.elements, to_fqn) else {
            return diags;
        };

        for policy in &self.policies {
            let severity = enforcement_to_severity(policy.enforcement.as_str());
            for rule in &policy.rules {
                let PolicyRuleAst::DenyEdge {
                    from,
                    to,
                    except,
                    message,
                    suggestions,
                } = rule
                else {
                    continue;
                };

                if !selector_matches_element(from, from_fqn, from_elem) {
                    continue;
                }
                if !selector_matches_element(to, to_fqn, to_elem) {
                    continue;
                }
                if edge_is_excepted(except, from_fqn, from_elem, to_fqn, to_elem) {
                    continue;
                }

                let msg = message.clone().unwrap_or_else(|| {
                    format!(
                        "Policy '{}' violated: {} must not connect to {}",
                        policy.id, from_fqn, to_fqn
                    )
                });
                let suggs = if suggestions.is_empty() {
                    vec![
                        "Remove the step or route through an allowed intermediary".to_string(),
                        "If intentional, add an exception".to_string(),
                    ]
                } else {
                    suggestions.clone()
                };

                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_POLICY_VIOLATION,
                        severity,
                        msg,
                        loc.clone(),
                    )
                    .with_suggestions(suggs),
                );
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use crate::DomainSchema;
    use super::*;
    use sruja_language::Parser;

    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        let rule = ScenarioValidationRule;
        rule.validate(&program, &DomainSchema::architecture())
    }

    #[test]
    fn empty_program_returns_no_diagnostics() {
        let diags = validate_program("");
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_scenario_steps_pass() {
        let input = r#"
user = person "User"
api = system "API"
db = datastore "DB"

user -> api "calls"
api -> db "queries"

scenario LoginFlow "Login" {
    step user -> api "submits"
    step api -> db "validates"
}
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_step_source_fails() {
        let input = r#"
api = system "API"
db = datastore "DB"

scenario Flow1 "Flow" {
    step nonexistent -> api "calls"
}
"#;
        let diags = validate_program(input);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("source")));
    }

    #[test]
    fn invalid_step_target_fails() {
        let input = r#"
user = person "User"
api = system "API"

scenario Flow1 "Flow" {
    step user -> nonexistent "calls"
}
"#;
        let diags = validate_program(input);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("target")));
    }

    #[test]
    fn external_to_database_policy_violation() {
        let input = r#"
NoExternalToDb = policy "No external to db" {
    enforcement "required"
    rule deny edge from { kind "container" tag "external" } to { kind "datastore" tag "database" }
}

ext = container "External" {
    metadata { tags ["external"] }
}
db = datastore "DB" {
    metadata { tags ["database"] }
}

scenario BadFlow "Bad Flow" {
    step ext -> db "direct access"
}
"#;
        let diags = validate_program(input);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("NoExternalToDb")));
    }

    #[test]
    fn flow_with_valid_steps_passes() {
        let input = r#"
user = person "User"
api = system "API"

LoginFlow = flow "Login" {
    step user -> api "submits"
}
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn inline_steps_inside_element_body_pass() {
        let input = r#"
user = person "User"
api = system "API" {
    step user -> api "calls"
}
db = datastore "DB"

api -> db "queries"
"#;
        let diags = validate_program(input);
        assert!(diags.is_empty());
    }

    #[test]
    fn inline_steps_inside_element_body_invalid_source_fails() {
        let input = r#"
api = system "API" {
    step nonexistent -> api "calls"
}
"#;
        let diags = validate_program(input);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.message.contains("source")));
    }
}
