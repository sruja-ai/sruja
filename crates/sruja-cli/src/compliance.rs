//! Compliance: policy evaluation from intent and compliance report building.
//!
//! Evaluates DeclaredPolicy rules against the scan graph using the intent drift engine.

use sruja_intent::{Drift, DriftDetector, DriftKind, IntentModel};
use sruja_language::DomainSchema;
use sruja_scan::Graph;

/// Returns drifts for each policy violation found.
pub fn evaluate_policy_violations(intent: &IntentModel, scan_graph: &Graph) -> Vec<Drift> {
    let schema = DomainSchema::architecture();
    let report = DriftDetector::new().detect(intent, scan_graph, &schema);
    report
        .drifts
        .into_iter()
        .filter(|d| d.kind == DriftKind::PolicyViolation)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_policy_rule_produces_violation() {
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

        let mut scan_graph = sruja_scan::Graph::new();
        scan_graph.nodes.push(sruja_scan::Node {
            id: "ext".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::EXTERNAL_API),
            label: "External API".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.nodes.push(sruja_scan::Node {
            id: "db".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::DATABASE),
            label: "DB".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.edges.push(sruja_scan::Edge {
            source: "ext".to_string(),
            target: "db".to_string(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::CALLS),
            evidence: vec![],
            confidence: Default::default(),
        });

        let drifts = evaluate_policy_violations(&model, &scan_graph);
        assert!(!drifts.is_empty(), "should report policy violation");
    }

    #[test]
    fn test_structured_policy_rule_allows_exception_edge() {
        use sruja_intent::IntentModel;
        use std::path::PathBuf;

        let sruja = r#"
Security = policy "No external API to database" {
  category "security"
  rule deny edge from { kind "external_api" } to { kind "database" }
    except from { id "ext" } to { id "db" }
}
"#;
        let path = PathBuf::from("test.sruja");
        let model = IntentModel::from_sruja_content(sruja, &path).expect("parse");

        let mut scan_graph = sruja_scan::Graph::new();
        scan_graph.nodes.push(sruja_scan::Node {
            id: "ext".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::EXTERNAL_API),
            label: "External API".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.nodes.push(sruja_scan::Node {
            id: "db".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::DATABASE),
            label: "DB".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.edges.push(sruja_scan::Edge {
            source: "ext".to_string(),
            target: "db".to_string(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::CALLS),
            evidence: vec![],
            confidence: Default::default(),
        });

        let drifts = evaluate_policy_violations(&model, &scan_graph);
        assert!(drifts.is_empty(), "exception edge should be allowed");
    }

    #[test]
    fn test_policy_violation_detected_when_no_exception() {
        use sruja_intent::IntentModel;
        use std::path::PathBuf;

        let sruja = r#"
Security = policy "No external API to database" {
  category "security"
  rule deny edge from { kind "external_api" } to { kind "database" }
    except from { id "ext" } to { id "db" }
}
"#;
        let path = PathBuf::from("test.sruja");
        let model = IntentModel::from_sruja_content(sruja, &path).expect("parse");

        let mut scan_graph = sruja_scan::Graph::new();
        scan_graph.nodes.push(sruja_scan::Node {
            id: "ext2".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::EXTERNAL_API),
            label: "External API 2".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.nodes.push(sruja_scan::Node {
            id: "db".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::DATABASE),
            label: "DB".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.edges.push(sruja_scan::Edge {
            source: "ext2".to_string(),
            target: "db".to_string(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::CALLS),
            evidence: vec![],
            confidence: Default::default(),
        });

        let drifts = evaluate_policy_violations(&model, &scan_graph);
        assert!(
            !drifts.is_empty(),
            "non-excepted external_api to database should violate"
        );
    }

    #[test]
    fn test_policy_rule_with_id_selector() {
        use sruja_intent::IntentModel;
        use std::path::PathBuf;

        let sruja = r#"
Security = policy "Restrict Auth.Service" {
  category "security"
  rule deny edge from { id "auth-service" } to { id "legacy-auth" }
}
"#;
        let path = PathBuf::from("test.sruja");
        let model = IntentModel::from_sruja_content(sruja, &path).expect("parse");

        let mut scan_graph = sruja_scan::Graph::new();
        scan_graph.nodes.push(sruja_scan::Node {
            id: "auth-service".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::COMPONENT),
            label: "Auth.Service".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.nodes.push(sruja_scan::Node {
            id: "legacy-auth".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::COMPONENT),
            label: "Legacy Auth".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.edges.push(sruja_scan::Edge {
            source: "auth-service".to_string(),
            target: "legacy-auth".to_string(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::CALLS),
            evidence: vec![],
            confidence: Default::default(),
        });

        let drifts = evaluate_policy_violations(&model, &scan_graph);
        assert!(!drifts.is_empty(), "id-matched deny rule should fire");
    }

    #[test]
    fn test_multiple_rules_in_policy() {
        use sruja_intent::IntentModel;
        use std::path::PathBuf;

        let sruja = r#"
Security = policy "Multi-rule security" {
  category "security"
  rule deny edge from { kind "external_api" } to { kind "database" }
  rule deny edge from { kind "external_api" } to { kind "queue" }
}
"#;
        let path = PathBuf::from("test.sruja");
        let model = IntentModel::from_sruja_content(sruja, &path).expect("parse");

        let mut scan_graph = sruja_scan::Graph::new();
        scan_graph.nodes.push(sruja_scan::Node {
            id: "ext".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::EXTERNAL_API),
            label: "External API".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.nodes.push(sruja_scan::Node {
            id: "q".to_string(),
            kind: sruja_scan::NodeKind::new(sruja_scan::NodeKind::QUEUE),
            label: "Queue".to_string(),
            ..sruja_scan::Node::default()
        });
        scan_graph.edges.push(sruja_scan::Edge {
            source: "ext".to_string(),
            target: "q".to_string(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::PUBLISHES_TO),
            evidence: vec![],
            confidence: Default::default(),
        });

        let drifts = evaluate_policy_violations(&model, &scan_graph);
        assert_eq!(
            drifts.len(),
            1,
            "only the queue rule should fire (not the db rule)"
        );
    }
}
