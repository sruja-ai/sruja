//! Scenario validation rule
//!
//! Validates behavioral flows (scenarios, stories, flows) similar to the Go engine:
//! - Each step must reference existing elements
//! - Enforces simple tag-based policies (e.g., external -> database is forbidden)

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use sruja_language::{collect_elements, ElementDef, Program, ScenarioStep, TopLevelItem};

use crate::validator::Rule;

/// Validates scenarios and flows.
pub struct ScenarioValidationRule;

impl Rule for ScenarioValidationRule {
    fn name(&self) -> &str {
        "Scenario Validation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);
        let runner = ScenarioRunner::new(&elements);

        let mut diags: Vec<Diagnostic> = Vec::new();

        for item in &program.items {
            match item {
                TopLevelItem::Scenario(s) => {
                    diags.extend(runner.validate_steps(&s.steps, &s.location));
                }
                TopLevelItem::Flow(f) => {
                    diags.extend(runner.validate_steps(&f.steps, &f.location));
                }
                // TODO(parity): Go also validates inline steps inside element bodies.
                _ => {}
            }
        }

        diags
    }
}

struct ScenarioRunner<'a> {
    elements: &'a HashMap<String, ElementDef>,
}

impl<'a> ScenarioRunner<'a> {
    fn new(elements: &'a HashMap<String, ElementDef>) -> Self {
        Self { elements }
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

            let from_exists = self.element_exists(&from_fqn);
            let to_exists = self.element_exists(&to_fqn);

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

    fn element_exists(&self, fqn: &str) -> bool {
        if fqn.is_empty() {
            return false;
        }
        if self.elements.contains_key(fqn) {
            return true;
        }
        // Allow leaf-id match as fallback (parity-ish; Go uses exact FQN from parts)
        let suffix = format!(".{}", fqn);
        self.elements
            .keys()
            .any(|k| k == fqn || k.ends_with(&suffix))
    }

    fn check_policies(
        &self,
        _step: &ScenarioStep,
        from_fqn: &str,
        to_fqn: &str,
        loc: &SourceLocation,
    ) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();

        let from_elem = self.find_element(from_fqn);
        let to_elem = self.find_element(to_fqn);
        if from_elem.is_none() || to_elem.is_none() {
            return diags;
        }

        let from_tags = self.get_tags(from_elem.unwrap());
        let to_tags = self.get_tags(to_elem.unwrap());

        if has_tag(&from_tags, "external") && has_tag(&to_tags, "database") {
            diags.push(
                Diagnostic::new(
                    sruja_diagnostics::codes::CODE_POLICY_VIOLATION,
                    Severity::Error,
                    format!(
                        "Security Policy Violation: External node '{}' cannot talk directly to database '{}'",
                        from_fqn, to_fqn
                    ),
                    loc.clone(),
                )
                .with_suggestions(vec![
                    "Route this request through an API Gateway or Service layer".to_string(),
                    "Ensure the database is not publicly accessible".to_string(),
                ]),
            );
        }

        diags
    }

    fn get_tags(&self, elem: &ElementDef) -> Vec<String> {
        let mut tags: Vec<String> = Vec::new();

        // 1) Tag refs on assignment
        for t in &elem.assignment.tag_refs {
            tags.push(t.trim_start_matches('#').to_string());
        }

        // 2) Metadata tags: metadata { tag "a,b" } or tags "a,b"
        if let Some(body) = &elem.assignment.body {
            for m in &body.metadata {
                if m.key == "tags" || m.key == "tag" {
                    if let Some(val) = m.value.as_ref() {
                        let v = val.trim().trim_matches('"');
                        tags.extend(
                            v.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty()),
                        );
                    }
                }
            }
        }

        tags
    }

    fn find_element(&self, fqn: &str) -> Option<&ElementDef> {
        if let Some(e) = self.elements.get(fqn) {
            return Some(e);
        }
        let suffix = format!(".{}", fqn);
        self.elements.iter().find_map(|(k, e)| {
            if k == fqn || k.ends_with(&suffix) {
                Some(e)
            } else {
                None
            }
        })
    }
}

fn has_tag(tags: &[String], target: &str) -> bool {
    let target = target.to_lowercase();
    tags.iter().any(|t| t.trim().to_lowercase() == target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_language::Parser;

    fn validate_program(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = match parser.parse(input) {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        let rule = ScenarioValidationRule;
        rule.validate(&program)
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
        assert!(diags.iter().any(|d| d.message.contains("Policy Violation")));
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
}
