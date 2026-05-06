//! Compliance: policy evaluation from intent and compliance report building.
//!
//! Evaluates DeclaredPolicy rules against the scan graph by converting them to
//! sruja-graph Policy/Constraint and using find_policy_violations. Structured
//! rules (e.g. ForbidEdge) are evaluated first; legacy phrase parsing is a
//! compatibility shim.

use sruja_graph::NodeKind;
use sruja_graph::{
    merge_scan_into_graph, Constraint, KnowledgeGraph, Policy, PolicyRule, PolicySeverity,
    SourceReference,
};
use sruja_intent::compare::Evidence;
use sruja_intent::model::PolicyRuleContent;
use sruja_intent::{Drift, DriftKind, IntentModel, Severity};
use sruja_scan::Graph;
use std::str::FromStr;

/// Parse a constraint string into (source_kind, target_kind, allowed).
/// Supports: "X must not call Y", "X cannot call Y" (case-insensitive).
/// Returns None if the pattern does not match.
pub fn parse_constraint_to_rule(
    s: &str,
) -> Option<(sruja_graph::NodeKind, sruja_graph::NodeKind, bool)> {
    let tokens: Vec<String> = s
        .split_whitespace()
        .map(|t| t.to_ascii_lowercase())
        .collect();

    let (src_tokens, tgt_tokens) =
        if let Some(pos) = tokens.windows(3).position(|w| w == ["must", "not", "call"]) {
            (&tokens[..pos], &tokens[(pos + 3)..])
        } else if let Some(pos) = tokens.windows(2).position(|w| w == ["cannot", "call"]) {
            (&tokens[..pos], &tokens[(pos + 2)..])
        } else {
            return None;
        };

    if src_tokens.is_empty() || tgt_tokens.is_empty() {
        return None;
    }

    let src_str = normalize_kind(&src_tokens.join(" "));
    let tgt_str = normalize_kind(&tgt_tokens.join(" "));

    let source_kind = NodeKind::from_str(&src_str).ok()?;
    let target_kind = NodeKind::from_str(&tgt_str).ok()?;
    Some((source_kind, target_kind, false))
}

/// Normalize kind string to match NodeKind::from_str (e.g. "external_api", "External API" -> "external_api").
fn normalize_kind(s: &str) -> String {
    s.replace([' ', '-'], "_").to_lowercase()
}

fn enforcement_to_policy_severity(enforcement: &str) -> PolicySeverity {
    match enforcement.to_lowercase().trim() {
        "required" | "error" => PolicySeverity::Error,
        "recommended" | "warn" | "warning" => PolicySeverity::Warning,
        "optional" | "info" => PolicySeverity::Info,
        _ => PolicySeverity::Error,
    }
}

/// Build graph Constraint from a DeclaredPolicy rule. Uses structured content first; falls back to phrase parsing.
fn rule_to_constraint(
    rule: &sruja_intent::model::PolicyRule,
) -> Option<(NodeKind, NodeKind, bool)> {
    match &rule.content {
        Some(PolicyRuleContent::DenyEdge {
            from, to, except, ..
        }) => {
            if !except.is_empty() {
                return None;
            }
            let source_kind = selector_to_node_kind(from)?;
            let target_kind = selector_to_node_kind(to)?;
            Some((source_kind, target_kind, false))
        }
        _ => parse_constraint_to_rule(rule.constraint.as_str()),
    }
}

fn selector_to_node_kind(selector: &sruja_intent::model::PolicySelector) -> Option<NodeKind> {
    if selector.id.is_some()
        || !selector.tags.is_empty()
        || selector.technology.is_some()
        || !selector.meta.is_empty()
    {
        return None;
    }
    let kind = selector.kind.as_deref()?;
    let kind = normalize_kind(kind);
    NodeKind::from_str(&kind).ok()
}

/// Evaluate intent policies against the scan graph using the knowledge graph policy engine.
/// Returns drifts for each policy violation found.
pub fn evaluate_policy_violations(intent: &IntentModel, scan_graph: &Graph) -> Vec<Drift> {
    let mut kg = KnowledgeGraph::new();
    merge_scan_into_graph(&mut kg, scan_graph, "");

    for policy in &intent.policies {
        let mut rules: Vec<PolicyRule> = Vec::new();
        for rule in &policy.rules {
            if let Some((source_kind, target_kind, allowed)) = rule_to_constraint(rule) {
                rules.push(PolicyRule {
                    description: rule.description.clone(),
                    constraint: Constraint {
                        source_kind: Some(source_kind),
                        target_kind: Some(target_kind),
                        allowed,
                        message: rule.constraint.clone(),
                    },
                });
            }
        }
        if !rules.is_empty() {
            let graph_policy = Policy {
                id: policy.name.clone(),
                name: policy.name.clone(),
                description: policy.description.clone(),
                rules,
                severity: enforcement_to_policy_severity(policy.enforcement.as_str()),
                source: SourceReference::Manual,
            };
            let _ = kg.add_policy(graph_policy);
        }
    }

    let violations = kg.find_policy_violations();
    violations
        .into_iter()
        .map(|v| Drift {
            kind: DriftKind::PolicyViolation,
            severity: match v.severity {
                PolicySeverity::Error => Severity::High,
                PolicySeverity::Warning => Severity::Medium,
                PolicySeverity::Info => Severity::Low,
            },
            description: v.message.clone(),
            evidence: vec![Evidence {
                source: "policy".to_string(),
                location: Some(format!("{} -> {}", v.source, v.target)),
                detail: format!("Policy '{}' violated", v.policy_name),
            }],
            intent_ref: Some(v.policy_id),
            suggestion: Some(format!(
                "Remove or change the dependency from {} to {} to comply with policy",
                v.source, v.target
            )),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_graph::NodeKind;

    #[test]
    fn test_parse_constraint_must_not_call() {
        let (src, tgt, allowed) =
            parse_constraint_to_rule("external_api must not call database").unwrap();
        assert_eq!(src, NodeKind::ExternalApi);
        assert_eq!(tgt, NodeKind::Database);
        assert!(!allowed);
    }

    #[test]
    fn test_parse_constraint_cannot_call() {
        let (src, tgt, allowed) =
            parse_constraint_to_rule("service cannot call external_api").unwrap();
        assert_eq!(src, NodeKind::Service);
        assert_eq!(tgt, NodeKind::ExternalApi);
        assert!(!allowed);
    }

    #[test]
    fn test_parse_constraint_is_case_insensitive_and_whitespace_tolerant() {
        let (src, tgt, allowed) =
            parse_constraint_to_rule("  External   API   MUST   NOT   CALL   DataBase  ").unwrap();
        assert_eq!(src, NodeKind::ExternalApi);
        assert_eq!(tgt, NodeKind::Database);
        assert!(!allowed);
    }

    #[test]
    fn test_parse_constraint_supports_hyphens_and_spaces_in_kinds() {
        let (src, tgt, allowed) =
            parse_constraint_to_rule("external-api cannot call external api").unwrap();
        assert_eq!(src, NodeKind::ExternalApi);
        assert_eq!(tgt, NodeKind::ExternalApi);
        assert!(!allowed);
    }

    #[test]
    fn test_parse_constraint_invalid_kind_returns_none() {
        assert!(parse_constraint_to_rule("madeup_kind must not call database").is_none());
    }

    #[test]
    fn test_parse_constraint_missing_target_returns_none() {
        assert!(parse_constraint_to_rule("service cannot call").is_none());
        assert!(parse_constraint_to_rule("service must not call").is_none());
    }

    #[test]
    fn test_parse_constraint_unrecognized_returns_none() {
        assert!(parse_constraint_to_rule("random text").is_none());
    }

    #[test]
    fn test_selector_to_node_kind_accepts_simple_kind_and_rejects_extra_filters() {
        use sruja_intent::model::PolicySelector;

        let selector = PolicySelector {
            kind: Some("External API".to_string()),
            ..Default::default()
        };
        assert_eq!(
            selector_to_node_kind(&selector),
            Some(NodeKind::ExternalApi)
        );

        let selector_with_id = PolicySelector {
            kind: Some("external_api".to_string()),
            id: Some("x".to_string()),
            ..Default::default()
        };
        assert_eq!(selector_to_node_kind(&selector_with_id), None);

        let selector_with_tag = PolicySelector {
            kind: Some("external_api".to_string()),
            tags: vec!["pci".to_string()],
            ..Default::default()
        };
        assert_eq!(selector_to_node_kind(&selector_with_tag), None);
    }

    #[test]
    fn test_enforcement_to_policy_severity() {
        assert_eq!(
            enforcement_to_policy_severity("required"),
            PolicySeverity::Error
        );
        assert_eq!(
            enforcement_to_policy_severity("warn"),
            PolicySeverity::Warning
        );
        assert_eq!(enforcement_to_policy_severity("INFO"), PolicySeverity::Info);
        assert_eq!(
            enforcement_to_policy_severity("unknown"),
            PolicySeverity::Error
        );
    }

    #[test]
    fn test_rule_to_constraint_deny_edge_with_exceptions_is_ignored() {
        use sruja_intent::model::{
            PolicyEdgeException, PolicyRule, PolicyRuleContent, PolicySelector,
        };

        let rule = PolicyRule {
            description: "deny edge with exception".to_string(),
            constraint: "deny edge".to_string(),
            content: Some(PolicyRuleContent::DenyEdge {
                from: PolicySelector {
                    kind: Some("service".to_string()),
                    ..Default::default()
                },
                to: PolicySelector {
                    kind: Some("database".to_string()),
                    ..Default::default()
                },
                except: vec![PolicyEdgeException {
                    from: PolicySelector {
                        kind: Some("service".to_string()),
                        ..Default::default()
                    },
                    to: PolicySelector {
                        kind: Some("database".to_string()),
                        ..Default::default()
                    },
                }],
                message: None,
                suggestions: vec![],
            }),
        };

        assert_eq!(rule_to_constraint(&rule), None);
    }

    #[test]
    fn test_structured_forbid_edge_rule_produces_violation() {
        use sruja_intent::model::PolicyRuleContent;
        use sruja_intent::IntentModel;
        use std::path::PathBuf;

        let sruja = r#"
Security = policy "No external API to database" {
  category "security"
  rule deny edge from { kind "external_api" } to { kind "database" }
}
"#;
        let path = PathBuf::from("test.sruja");
        let model = IntentModel::from_sruja_content(sruja, &path).expect("parse");
        assert_eq!(model.policies.len(), 1);
        assert_eq!(model.policies[0].rules.len(), 1);
        assert!(matches!(
            model.policies[0].rules[0].content,
            Some(PolicyRuleContent::DenyEdge { .. })
        ));

        let mut scan_graph = sruja_scan::Graph::new();
        scan_graph.nodes.push(sruja_scan::Node {
            id: "ext".to_string(),
            kind: sruja_scan::NodeKind::ExternalApi,
            label: "External API".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.nodes.push(sruja_scan::Node {
            id: "db".to_string(),
            kind: sruja_scan::NodeKind::Database,
            label: "DB".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.edges.push(sruja_scan::Edge {
            source: "ext".to_string(),
            target: "db".to_string(),
            kind: sruja_scan::EdgeKind::Calls,
            evidence: vec![],
            confidence: Default::default(),
        });

        let drifts = evaluate_policy_violations(&model, &scan_graph);
        assert!(!drifts.is_empty(), "should report policy violation");
    }
}
