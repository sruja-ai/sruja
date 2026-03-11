//! Compliance: policy evaluation from intent and compliance report building.
//!
//! Evaluates DeclaredPolicy rules against the scan graph by converting them to
//! sruja-graph Policy/Constraint and using find_policy_violations.

use sruja_graph::NodeKind;
use sruja_graph::{
    merge_scan_into_graph, Constraint, KnowledgeGraph, Policy, PolicyRule, PolicySeverity,
    SourceReference,
};
use sruja_intent::compare::Evidence;
use sruja_intent::{Drift, DriftKind, IntentModel, Severity};
use sruja_scan::Graph;
use std::str::FromStr;

/// Parse a constraint string into (source_kind, target_kind, allowed).
/// Supports: "X must not call Y", "X cannot call Y" (case-insensitive).
/// Returns None if the pattern does not match.
pub fn parse_constraint_to_rule(
    s: &str,
) -> Option<(sruja_graph::NodeKind, sruja_graph::NodeKind, bool)> {
    let lower = s.to_lowercase().trim().to_string();
    let lower = lower.as_str();

    // "X must not call Y" or "X cannot call Y"
    let (source, target) = if lower.contains(" must not call ") {
        let parts: Vec<&str> = lower.splitn(2, " must not call ").collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0].trim(), parts[1].trim())
    } else if lower.contains(" cannot call ") {
        let parts: Vec<&str> = lower.splitn(2, " cannot call ").collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0].trim(), parts[1].trim())
    } else {
        return None;
    };

    let norm = |t: &str| t.replace([' ', '-'], "_").to_lowercase();
    let src_str = norm(source);
    let tgt_str = norm(target);

    let source_kind = NodeKind::from_str(&src_str).ok()?;
    let target_kind = NodeKind::from_str(&tgt_str).ok()?;
    Some((source_kind, target_kind, false))
}

/// Evaluate intent policies against the scan graph using the knowledge graph policy engine.
/// Returns drifts for each policy violation found.
pub fn evaluate_policy_violations(intent: &IntentModel, scan_graph: &Graph) -> Vec<Drift> {
    let mut kg = KnowledgeGraph::new();
    merge_scan_into_graph(&mut kg, scan_graph, "");

    for policy in &intent.policies {
        let mut rules: Vec<PolicyRule> = Vec::new();
        for rule in &policy.rules {
            if let Some((source_kind, target_kind, allowed)) =
                parse_constraint_to_rule(rule.constraint.as_str())
            {
                rules.push(PolicyRule {
                    description: rule.description.clone(),
                    constraint: Constraint {
                        source_kind: Some(source_kind),
                        target_kind: Some(target_kind),
                        allowed,
                        message: rule.description.clone(),
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
                severity: PolicySeverity::Error,
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
        let r = parse_constraint_to_rule("external_api must not call database");
        assert!(r.is_some());
        let (src, tgt, allowed) = r.unwrap();
        assert_eq!(src, NodeKind::ExternalApi);
        assert_eq!(tgt, NodeKind::Database);
        assert!(!allowed);
    }

    #[test]
    fn test_parse_constraint_cannot_call() {
        let r = parse_constraint_to_rule("service cannot call external_api");
        assert!(r.is_some());
        let (src, tgt, allowed) = r.unwrap();
        assert_eq!(src, NodeKind::Service);
        assert_eq!(tgt, NodeKind::ExternalApi);
        assert!(!allowed);
    }

    #[test]
    fn test_parse_constraint_unrecognized_returns_none() {
        assert!(parse_constraint_to_rule("random text").is_none());
    }
}
