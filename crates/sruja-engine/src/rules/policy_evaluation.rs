use sruja_diagnostics::Diagnostic;
use sruja_language::{collect_elements, ElementDef, Program, TopLevelItem};

use crate::utils::{
    edge_is_excepted, element_has_metadata, enforcement_to_severity, has_tag, normalize_tag,
    selector_matches_element,
};
use crate::validator::Rule;

pub struct PolicyEvaluationRule;

impl Rule for PolicyEvaluationRule {
    fn name(&self) -> &str {
        "Policy Evaluation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, relations) = collect_elements(program);
        let mut diags: Vec<Diagnostic> = Vec::new();

        for item in &program.items {
            let TopLevelItem::Policy(policy) = item else {
                continue;
            };

            let severity = enforcement_to_severity(policy.enforcement.as_str());

            for rule in &policy.rules {
                match rule {
                    sruja_language::PolicyRuleAst::DenyEdge {
                        from,
                        to,
                        except,
                        message,
                        suggestions,
                    } => {
                        for rel in &relations {
                            let from_fqn = rel.from.as_string();
                            let to_fqn = rel.to.as_string();
                            let (Some(from_elem), Some(to_elem)) =
                                (elements.get(&from_fqn), elements.get(&to_fqn))
                            else {
                                continue;
                            };

                            if !selector_matches_element(from, from_fqn.as_str(), from_elem) {
                                continue;
                            }
                            if !selector_matches_element(to, to_fqn.as_str(), to_elem) {
                                continue;
                            }
                            if edge_is_excepted(
                                except,
                                from_fqn.as_str(),
                                from_elem,
                                to_fqn.as_str(),
                                to_elem,
                            ) {
                                continue;
                            }

                            {
                                let msg = message.clone().unwrap_or_else(|| {
                                    format!(
                                        "Policy '{}' violated: {} must not connect to {}",
                                        policy.id, from_fqn, to_fqn
                                    )
                                });
                                let suggs = if suggestions.is_empty() {
                                    vec![
                                        "Remove the relation or route through an allowed intermediary"
                                            .to_string(),
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
                                        rel.location.clone(),
                                    )
                                    .with_suggestions(suggs),
                                );
                            }
                        }
                    }
                    sruja_language::PolicyRuleAst::RequireTags {
                        selector,
                        tags,
                        except,
                        message,
                        suggestions,
                    } => {
                        let required: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();

                        for (fqn, elem) in &elements {
                            if !selector_matches_element(selector, fqn.as_str(), elem) {
                                continue;
                            }
                            if selector_matches_any(except, fqn.as_str(), elem) {
                                continue;
                            }

                            let missing: Vec<String> = required
                                .iter()
                                .filter(|t| !has_tag(elem, t.as_str()))
                                .cloned()
                                .collect();
                            if missing.is_empty() {
                                continue;
                            }

                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Policy '{}' violated: '{}' missing required tags: {}",
                                    policy.id,
                                    fqn,
                                    missing.join(", ")
                                )
                            });
                            let suggs = if suggestions.is_empty() {
                                vec![
                                    "Add tags to the element using `tags [\"...\"]` or `#tag`"
                                        .to_string(),
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
                                    elem.location.clone(),
                                )
                                .with_suggestions(suggs),
                            );
                        }
                    }
                    sruja_language::PolicyRuleAst::RequireMetadata {
                        selector,
                        key,
                        value,
                        except,
                        message,
                        suggestions,
                    } => {
                        for (fqn, elem) in &elements {
                            if !selector_matches_element(selector, fqn.as_str(), elem) {
                                continue;
                            }
                            if selector_matches_any(except, fqn.as_str(), elem) {
                                continue;
                            }

                            if element_has_metadata(elem, key.as_str(), value.as_deref()) {
                                continue;
                            }

                            let msg = message.clone().unwrap_or_else(|| {
                                if let Some(value) = value {
                                    format!(
                                        "Policy '{}' violated: '{}' metadata '{}' must be '{}'",
                                        policy.id, fqn, key, value
                                    )
                                } else {
                                    format!(
                                        "Policy '{}' violated: '{}' missing metadata key '{}'",
                                        policy.id, fqn, key
                                    )
                                }
                            });
                            let suggs = if suggestions.is_empty() {
                                vec![
                                    format!("Add `metadata {{ {} \"...\" }}` to the element", key),
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
                                    elem.location.clone(),
                                )
                                .with_suggestions(suggs),
                            );
                        }
                    }
                    sruja_language::PolicyRuleAst::RequireSlo {
                        selector,
                        except,
                        message,
                        suggestions,
                    } => {
                        for (fqn, elem) in &elements {
                            if !selector_matches_element(selector, fqn.as_str(), elem) {
                                continue;
                            }
                            if selector_matches_any(except, fqn.as_str(), elem) {
                                continue;
                            }

                            let has_slo = elem
                                .assignment
                                .body
                                .as_ref()
                                .is_some_and(|b| b.slo.is_some());
                            if has_slo {
                                continue;
                            }

                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Policy '{}' violated: '{}' missing SLO definition",
                                    policy.id, fqn
                                )
                            });
                            let suggs = if suggestions.is_empty() {
                                vec![
                                    "Add an `slo { ... }` block to the element".to_string(),
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
                                    elem.location.clone(),
                                )
                                .with_suggestions(suggs),
                            );
                        }
                    }
                }
            }
        }

        diags
    }
}

fn selector_matches_any(
    except: &[sruja_language::PolicySelectorAst],
    fqn: &str,
    elem: &ElementDef,
) -> bool {
    except
        .iter()
        .any(|selector| selector_matches_element(selector, fqn, elem))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::Severity;
    use sruja_language::Parser;

    fn validate(input: &str) -> Vec<Diagnostic> {
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("parse");
        PolicyEvaluationRule.validate(&program)
    }

    #[test]
    fn require_tags_reports_missing_tag() {
        let input = r#"
EncryptionPolicy = policy "DBs must be encrypted" {
  enforcement "required"
  rule require tags on { kind "database" } tags ["encrypted"]
}

DB = database "DB"
"#;
        let diags = validate(input);
        assert!(diags
            .iter()
            .any(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION));
        assert!(diags.iter().any(|d| d.severity == Severity::Error));
    }

    #[test]
    fn require_tags_passes_when_tags_present_in_body() {
        let input = r#"
EncryptionPolicy = policy "DBs must be encrypted" {
  enforcement "required"
  rule require tags on { kind "database" } tags ["encrypted"]
}

DB = database "DB" {
  tags ["encrypted"]
}
"#;
        let diags = validate(input);
        assert!(!diags
            .iter()
            .any(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION));
    }

    #[test]
    fn require_metadata_reports_missing_key() {
        let input = r#"
OwnershipPolicy = policy "Containers must declare owners" {
  enforcement "recommended"
  rule require metadata on { kind "container" } key "owner"
}

API = container "API"
"#;
        let diags = validate(input);
        assert!(diags
            .iter()
            .any(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION));
        assert!(diags.iter().any(|d| d.severity == Severity::Warning));
    }

    #[test]
    fn require_slo_with_tag_filter_only_applies_to_matching_elements() {
        let input = r#"
SloPolicy = policy "Production containers must have SLOs" {
  enforcement "required"
  rule require slo on { kind "container" tag "production" }
}

Prod = container "Prod" {
  tags ["production"]
}

Dev = container "Dev"
"#;
        let diags = validate(input);
        let violations: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION)
            .collect();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Prod"));
    }

    #[test]
    fn deny_edge_respects_edge_exceptions() {
        let input = r#"
NoServiceToDb = policy "Services must not call DBs" {
  enforcement "required"
  rule deny edge from { kind "service" } to { kind "database" } except from { id "Checkout" } to { id "PaymentsDb" }
}

Checkout = service "Checkout"
PaymentsDb = database "Payments DB"
Checkout -> PaymentsDb "SQL"
"#;
        let diags = validate(input);
        assert!(!diags
            .iter()
            .any(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION));
    }

    #[test]
    fn deny_edge_uses_custom_message_and_suggestions() {
        let input = r#"
NoServiceToDb = policy "Services must not call DBs" {
  enforcement "required"
  rule deny edge from { kind "service" } to { kind "database" } message "Direct DB access is forbidden" suggest "Use the Repository service"
}

Checkout = service "Checkout"
PaymentsDb = database "Payments DB"
Checkout -> PaymentsDb "SQL"
"#;
        let diags = validate(input);
        let violations: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION)
            .collect();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].message, "Direct DB access is forbidden");
        assert!(violations[0]
            .suggestions
            .iter()
            .any(|s| s == "Use the Repository service"));
    }

    #[test]
    fn require_tags_respects_element_exceptions() {
        let input = r#"
EncryptionPolicy = policy "DBs must be encrypted" {
  enforcement "required"
  rule require tags on { kind "database" } tags ["encrypted"] except { id "DevDb" }
}

DevDb = database "Dev DB"
ProdDb = database "Prod DB"
"#;
        let diags = validate(input);
        let violations: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.code == sruja_diagnostics::codes::CODE_POLICY_VIOLATION)
            .collect();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ProdDb"));
    }
}
