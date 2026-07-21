use super::types::*;
use super::format::node_matches_selector;
use crate::model::{
    BoundaryRule, BoundaryRuleType, ConnectionType, DeclaredBoundary, DeclaredComponent,
    DeclaredPolicy, DeclaredRelationship, IntentModel, IntentSourceInfo, IntentSourceType,
    PolicyEdgeException, PolicyMetaSelector, PolicyRule, PolicyRuleContent, PolicySelector,
    SourceReference,
};
use sruja_scan::{Edge, EdgeKind, Graph as ScanGraph, Node, NodeKind};
use std::collections::HashMap;
use std::path::PathBuf;

fn source_ref(element: &str) -> SourceReference {
    SourceReference {
        file: "test.sruja".to_string(),
        line: Some(1),
        element: Some(element.to_string()),
    }
}

fn scan_node(id: &str, label: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::new(NodeKind::MODULE),
        label: label.to_string(),
        path: Some(path.to_string()),
        ..Default::default()
    }
}

fn scan_node_with(
    id: &str,
    kind: NodeKind,
    technology: Option<&str>,
    metadata: HashMap<&str, &str>,
) -> Node {
    Node {
        id: id.to_string(),
        kind,
        label: id.to_string(),
        path: None,
        technology: technology.map(|t| t.to_string()),
        metadata: metadata
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}

fn create_test_graph() -> ScanGraph {
    ScanGraph {
        metadata: HashMap::new(),
        nodes: vec![
            scan_node("api", "API", "src/api"),
            scan_node("db", "Database", "src/db"),
        ],
        edges: vec![Edge {
            source: "api".to_string(),
            target: "db".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![],
            confidence: Default::default(),
        }],
        confidence: None,
        incidents: vec![],
        ..Default::default()
    }
}

fn create_test_intent() -> IntentModel {
    let mut model = IntentModel::new(IntentSourceInfo {
        source_type: IntentSourceType::Manual,
        path: PathBuf::from("test.sruja"),
        name: "test".to_string(),
    });
    model.components.push(DeclaredComponent {
        id: "api".to_string(),
        kind: "service".to_string(),
        label: "API Service".to_string(),
        description: None,
        technology: None,
        source_ref: source_ref("api"),
    });
    model
}

#[test]
fn test_detect_undocumented_component() {
    let detector = DriftDetector::new();
    let intent = create_test_intent();
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::UndocumentedComponent));
}

#[test]
fn test_detect_missing_component() {
    let detector = DriftDetector::new();
    let intent = create_test_intent();
    let reality = ScanGraph::new();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::MissingComponent));
}

#[test]
fn test_healthy_alignment() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.components.push(DeclaredComponent {
        id: "db".to_string(),
        kind: "service".to_string(),
        label: "Database".to_string(),
        description: None,
        technology: None,
        source_ref: source_ref("db"),
    });
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report.drift_score < 50);
}

#[test]
fn test_detect_undocumented_relationship() {
    let detector = DriftDetector::new();
    let intent = create_test_intent();
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::UndocumentedRelationship));
}

#[test]
fn test_detect_missing_relationship() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.relationships.push(DeclaredRelationship {
        source: "api".to_string(),
        target: "db".to_string(),
        kind: "calls".to_string(),
        label: None,
        source_ref: source_ref("api -> db"),
    });

    let mut reality = ScanGraph::new();
    reality.nodes.push(scan_node("api", "API", "src/api"));
    reality.nodes.push(scan_node("db", "Database", "src/db"));
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::MissingRelationship));
}

#[test]
fn test_detect_boundary_violation_no_direct_database_access() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.boundaries.push(DeclaredBoundary {
        name: "Core".to_string(),
        inside: vec!["api".to_string()],
        allowed_connections: Vec::new(),
        rules: vec![BoundaryRule {
            rule_type: BoundaryRuleType::NoDirectDatabaseAccess,
            description: "Use a repository/service boundary instead of direct DB access."
                .to_string(),
        }],
        source_ref: source_ref("Core"),
        max_depth: None,
    });
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::BoundaryViolation));
}

#[test]
fn test_detect_transitive_boundary_violation() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();

    intent.boundaries.push(DeclaredBoundary {
        name: "Frontend".to_string(),
        inside: vec!["frontend".to_string()],
        allowed_connections: Vec::new(),
        rules: vec![BoundaryRule {
            rule_type: BoundaryRuleType::NoDirectDatabaseAccess,
            description: "Frontend must not touch database, even transitively.".to_string(),
        }],
        max_depth: Some(3),
        source_ref: source_ref("Frontend"),
    });

    let mut reality = ScanGraph::new();
    reality
        .nodes
        .push(scan_node("frontend", "Frontend", "src/fe"));
    reality
        .nodes
        .push(scan_node("backend", "Backend", "src/be"));
    reality
        .nodes
        .push(scan_node("users_db", "Users DB", "src/db"));

    reality.edges.push(Edge {
        source: "frontend".to_string(),
        target: "backend".to_string(),
        kind: EdgeKind::new(EdgeKind::CALLS),
        evidence: Vec::new(),
        confidence: Default::default(),
    });
    reality.edges.push(Edge {
        source: "backend".to_string(),
        target: "users_db".to_string(),
        kind: EdgeKind::new(EdgeKind::WRITES_TO),
        evidence: Vec::new(),
        confidence: Default::default(),
    });
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    let boundary_violations: Vec<_> = report
        .drifts
        .iter()
        .filter(|d| d.kind == DriftKind::BoundaryViolation)
        .collect();

    assert!(
        !boundary_violations.is_empty(),
        "Should have detected transitive boundary violation"
    );
    assert!(boundary_violations[0].description.contains("transitively"));
    assert!(boundary_violations[0].description.contains("depth 2"));
}

#[test]
fn test_boundary_violation_not_reported_when_allowed_connection_matches_target() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.boundaries.push(DeclaredBoundary {
        name: "Core".to_string(),
        inside: vec!["api".to_string()],
        allowed_connections: vec![crate::model::AllowedConnection {
            target_boundary: "db".to_string(),
            via: ConnectionType::Database,
        }],
        rules: vec![BoundaryRule {
            rule_type: BoundaryRuleType::NoDirectDatabaseAccess,
            description: "Use a repository/service boundary instead of direct DB access."
                .to_string(),
        }],
        source_ref: source_ref("Core"),
        max_depth: None,
    });
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(!report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::BoundaryViolation));
}

#[test]
fn test_policy_deny_edge_detects_violation_and_respects_exceptions() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.policies.push(DeclaredPolicy {
        name: "NoDbCalls".to_string(),
        description: "Services must not call the DB directly.".to_string(),
        category: String::new(),
        enforcement: String::new(),
        scope: Vec::new(),
        rules: vec![PolicyRule {
            description: "No direct api->db".to_string(),
            constraint: "deny".to_string(),
            content: Some(PolicyRuleContent::DenyEdge {
                from: PolicySelector {
                    id: Some("api".to_string()),
                    ..Default::default()
                },
                to: PolicySelector {
                    id: Some("db".to_string()),
                    ..Default::default()
                },
                except: vec![PolicyEdgeException {
                    from: PolicySelector {
                        id: Some("api".to_string()),
                        ..Default::default()
                    },
                    to: PolicySelector {
                        id: Some("db".to_string()),
                        ..Default::default()
                    },
                }],
                message: None,
                suggestions: vec!["Introduce an API boundary layer.".to_string()],
            }),
        }],
        source_ref: source_ref("NoDbCalls"),
    });
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(!report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::PolicyViolation));

    intent.policies[0].rules[0].content = Some(PolicyRuleContent::DenyEdge {
        from: PolicySelector {
            id: Some("api".to_string()),
            ..Default::default()
        },
        to: PolicySelector {
            id: Some("db".to_string()),
            ..Default::default()
        },
        except: Vec::new(),
        message: None,
        suggestions: vec![],
    });

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());
    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::PolicyViolation && d.severity == Severity::High));
}

#[test]
fn test_policy_require_tags_and_metadata_and_slo() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.policies.push(DeclaredPolicy {
        name: "MetadataRules".to_string(),
        description: "Ensure required tags/metadata exist.".to_string(),
        category: String::new(),
        enforcement: String::new(),
        scope: Vec::new(),
        rules: vec![
            PolicyRule {
                description: "API must be tagged".to_string(),
                constraint: "tags".to_string(),
                content: Some(PolicyRuleContent::RequireTags {
                    selector: PolicySelector {
                        id: Some("api".to_string()),
                        ..Default::default()
                    },
                    tags: vec!["public".to_string()],
                    except: Vec::new(),
                    message: None,
                    suggestions: Vec::new(),
                }),
            },
            PolicyRule {
                description: "API must have tier metadata".to_string(),
                constraint: "meta".to_string(),
                content: Some(PolicyRuleContent::RequireMetadata {
                    selector: PolicySelector {
                        id: Some("api".to_string()),
                        ..Default::default()
                    },
                    key: "tier".to_string(),
                    value: Some("1".to_string()),
                    except: Vec::new(),
                    message: None,
                    suggestions: Vec::new(),
                }),
            },
            PolicyRule {
                description: "API must define SLO".to_string(),
                constraint: "slo".to_string(),
                content: Some(PolicyRuleContent::RequireSlo {
                    selector: PolicySelector {
                        id: Some("api".to_string()),
                        ..Default::default()
                    },
                    except: Vec::new(),
                    message: None,
                    suggestions: Vec::new(),
                }),
            },
        ],
        source_ref: source_ref("MetadataRules"),
    });

    let mut reality = ScanGraph::new();
    reality.nodes.push(scan_node_with(
        "api",
        NodeKind::new(NodeKind::SERVICE),
        Some("Rust"),
        HashMap::new(),
    ));
    reality.nodes.push(scan_node_with(
        "db",
        NodeKind::new(NodeKind::DATABASE),
        Some("PostgreSQL"),
        HashMap::new(),
    ));
    reality.edges.push(Edge {
        source: "api".to_string(),
        target: "db".to_string(),
        kind: EdgeKind::new(EdgeKind::CALLS),
        evidence: vec![],
        confidence: Default::default(),
    });
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());
    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::PolicyViolation));

    let mut meta = HashMap::new();
    meta.insert("tags", "public,internal");
    meta.insert("tier", "1");
    meta.insert("availability_slo", "99.9");
    reality.nodes[0] =
        scan_node_with("api", NodeKind::new(NodeKind::SERVICE), Some("Rust"), meta);

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());
    assert!(!report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::PolicyViolation));
}

#[test]
fn test_policy_phrase_compatibility_shim() {
    let detector = DriftDetector::new();
    let mut intent = create_test_intent();
    intent.policies.push(DeclaredPolicy {
        name: "NoApiDbPhrase".to_string(),
        description: "Legacy phrase rule".to_string(),
        category: String::new(),
        enforcement: String::new(),
        scope: Vec::new(),
        rules: vec![PolicyRule {
            description: "Legacy phrase".to_string(),
            constraint: "api must not call db".to_string(),
            content: Some(PolicyRuleContent::Phrase(
                "api must not call db".to_string(),
            )),
        }],
        source_ref: source_ref("NoApiDbPhrase"),
    });
    let reality = create_test_graph();
    use sruja_language::DomainSchema;

    let report = detector.detect(&intent, &reality, &DomainSchema::architecture());

    assert!(report
        .drifts
        .iter()
        .any(|d| d.kind == DriftKind::PolicyViolation));
}

#[test]
fn test_node_matches_selector_filters_kind_id_technology_tags_and_meta() {
    let mut metadata = HashMap::new();
    metadata.insert("tags", "public, pci");
    metadata.insert("tier", "1");
    metadata.insert("pci", "");
    let node = scan_node_with(
        "payments.api",
        NodeKind::new(NodeKind::SERVICE),
        Some("Rust"),
        metadata,
    );

    let selector = PolicySelector {
        kind: Some("service".to_string()),
        id: Some("api".to_string()),
        technology: Some("rust".to_string()),
        tags: vec!["public".to_string(), "pci".to_string()],
        meta: vec![
            PolicyMetaSelector {
                key: "tier".to_string(),
                value: Some("1".to_string()),
            },
            PolicyMetaSelector {
                key: "pci".to_string(),
                value: None,
            },
        ],
    };

    assert!(node_matches_selector(&node, &selector));

    let selector = PolicySelector {
        technology: Some("Go".to_string()),
        ..selector
    };
    assert!(!node_matches_selector(&node, &selector));
}

#[test]
fn test_compute_drift_score_zero_when_no_declared_components() {
    let summary = DriftSummary {
        total_components_declared: 0,
        total_components_discovered: 0,
        undocumented_components: 1,
        missing_components: 1,
        undocumented_relationships: 1,
        boundary_violations: 1,
        policy_violations: 1,
        schema_violations: 0,
        taxonomy_mismatches: 0,
        unproposed_changes: 0,
    };
    let drifts = vec![Drift {
        kind: DriftKind::PolicyViolation,
        severity: Severity::Critical,
        description: "x".to_string(),
        evidence: vec![],
        intent_ref: None,
        suggestion: None,
    }];

    assert_eq!(DriftDetector::compute_drift_score(&summary, &drifts), 0);
}

#[test]
fn test_drift_health_classification() {
    let _detector = DriftDetector::new();

    assert_eq!(DriftDetector::classify_health(10), DriftHealth::Healthy);
    assert_eq!(DriftDetector::classify_health(35), DriftHealth::MinorDrift);
    assert_eq!(
        DriftDetector::classify_health(60),
        DriftHealth::SignificantDrift
    );
    assert_eq!(
        DriftDetector::classify_health(80),
        DriftHealth::CriticalDrift
    );
}

#[test]
fn test_detect_with_loaded_intent_from_fixture_dir() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let sruja_path = temp_dir.path().join("arch.sruja");
    let minimal_sruja = r#"
api = container "API" {
  technology "Node.js"
  description "HTTP API"
}
"#;
    std::fs::write(&sruja_path, minimal_sruja).expect("write sruja");

    let mut intelligence = crate::IntentContext::new();
    let models = intelligence
        .load_from_directory(temp_dir.path())
        .expect("load from directory");

    assert!(
        !models.is_empty(),
        "should load at least one model from .sruja"
    );

    let mut merged = crate::IntentModel::default();
    for m in models {
        merged.merge(m);
    }

    let detector = DriftDetector::new();
    let reality = create_test_graph();
    use sruja_language::DomainSchema;
    let report = detector.detect(&merged, &reality, &DomainSchema::architecture());

    assert!(report.drift_score <= 100);
    assert!(!report.intent_source.is_empty());
    assert!(!report.reality_source.is_empty());
}
