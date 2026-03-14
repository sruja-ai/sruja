//! Integration tests for sruja-report: ComprehensiveReport, build_recommendations,
//! ComplianceReport, and compliance status.

use sruja_diff::{DriftReport, Severity, Violation, ViolationKind};
use sruja_report::{
    build_recommendations, ComplianceReport, ComplianceStatus, ComprehensiveReport, Layer,
    PolicyViolationEntry, Priority, RecommendationCategory, SemanticSection, StructuralSection,
};

fn minimal_drift_report(violations: Vec<Violation>, suggestions: Vec<String>) -> DriftReport {
    DriftReport {
        total_modules: 10,
        total_services: 2,
        total_databases: 1,
        total_dependencies: 5,
        circular_dependencies: 0,
        orphan_modules: 0,
        layer_violations: 0,
        violations,
        suggestions,
        health_score: 85,
        health_breakdown: None,
    }
}

fn violation(kind: ViolationKind, severity: Severity, message: &str) -> Violation {
    Violation {
        kind,
        severity,
        message: message.to_string(),
        location: Some("mod_a".to_string()),
        suggestion: None,
        sources: vec![],
    }
}

#[test]
fn build_recommendations_empty_report_produces_empty_list() {
    let report = minimal_drift_report(vec![], vec![]);
    let recs = build_recommendations(&report, &[], &[], 100);
    assert!(recs.is_empty());
}

#[test]
fn build_recommendations_from_structural_violations() {
    let report = minimal_drift_report(
        vec![
            violation(
                ViolationKind::CircularDependency,
                Severity::Error,
                "Cycle detected: A -> B -> A",
            ),
            violation(
                ViolationKind::OrphanComponent,
                Severity::Warning,
                "Orphan module: util",
            ),
        ],
        vec![],
    );
    let recs = build_recommendations(&report, &[], &[], 100);
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].category, RecommendationCategory::Cycle);
    assert_eq!(recs[0].priority, Priority::Critical);
    assert_eq!(recs[0].source_layer, Layer::Structural);
    assert_eq!(recs[1].category, RecommendationCategory::Orphan);
    assert_eq!(recs[1].priority, Priority::Medium);
}

#[test]
fn build_recommendations_dedupes_by_description() {
    let report = minimal_drift_report(
        vec![
            violation(
                ViolationKind::LayerViolation,
                Severity::Warning,
                "Frontend accesses database",
            ),
            violation(
                ViolationKind::LayerViolation,
                Severity::Warning,
                "Frontend accesses database",
            ),
        ],
        vec![],
    );
    let recs = build_recommendations(&report, &[], &[], 100);
    assert_eq!(recs.len(), 1);
}

#[test]
fn build_recommendations_respects_limit() {
    let report = minimal_drift_report(
        (0..10)
            .map(|i| {
                violation(
                    ViolationKind::OrphanComponent,
                    Severity::Info,
                    &format!("Orphan {}", i),
                )
            })
            .collect(),
        vec![],
    );
    let recs = build_recommendations(&report, &[], &[], 3);
    assert_eq!(recs.len(), 3);
}

#[test]
fn build_recommendations_includes_semantic_and_intent() {
    let report = minimal_drift_report(vec![], vec![]);
    let semantic = vec!["Hidden coupling: A and B".to_string()];
    let intent = vec!["Undocumented component: C".to_string()];
    let recs = build_recommendations(&report, &semantic, &intent, 100);
    assert_eq!(recs.len(), 2);
    assert!(recs.iter().any(|r| r.source_layer == Layer::Semantic));
    assert!(recs.iter().any(|r| r.source_layer == Layer::Intent));
}

#[test]
fn build_recommendations_sorts_by_priority() {
    let report = minimal_drift_report(
        vec![
            violation(ViolationKind::OrphanComponent, Severity::Info, "Orphan"),
            violation(
                ViolationKind::CircularDependency,
                Severity::Error,
                "Cycle",
            ),
        ],
        vec![],
    );
    let recs = build_recommendations(&report, &[], &[], 100);
    assert_eq!(recs[0].priority, Priority::Critical);
    assert_eq!(recs[1].priority, Priority::Low);
}

#[test]
fn comprehensive_report_serde_schema_version() {
    let report = ComprehensiveReport {
        schema_version: 1,
        structural: StructuralSection {
            modules: 5,
            services: 1,
            databases: 1,
            dependencies: 3,
            health_score: 90,
            violations_count: 0,
        },
        semantic: SemanticSection {
            component_count: 5,
            context_count: 2,
            hidden_coupling_count: 0,
            vocabulary_leak_count: 0,
            health_score: 95,
        },
        intent: None,
        runtime: None,
        overall_health: 92,
        recommendations: vec![],
    };
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"schema_version\":1"));
    let parsed: ComprehensiveReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema_version, 1);
}

#[test]
fn compliance_report_compliant_when_no_violations() {
    let report = ComplianceReport::from_parts(
        vec![],
        vec![],
        vec![],
        0,
        100,
    );
    assert_eq!(report.status, ComplianceStatus::Compliant);
    assert!(report.remediation_checklist.is_empty());
}

#[test]
fn compliance_report_non_compliant_with_structural_violations() {
    let v = violation(
        ViolationKind::CircularDependency,
        Severity::Error,
        "Cycle",
    );
    let report = ComplianceReport::from_parts(
        vec![v],
        vec![],
        vec![],
        0,
        70,
    );
    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert!(!report.remediation_checklist.is_empty());
    assert!(report.remediation_checklist[0].contains("structural"));
}

#[test]
fn compliance_report_non_compliant_with_policy_violations() {
    let report = ComplianceReport::from_parts(
        vec![],
        vec![],
        vec![PolicyViolationEntry {
            policy_name: "No direct DB".to_string(),
            message: "Frontend must not call DB".to_string(),
            source: "web".to_string(),
            target: "db".to_string(),
        }],
        0,
        80,
    );
    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert!(report.remediation_checklist.iter().any(|s| s.contains("policy")));
}

#[test]
fn compliance_report_non_compliant_with_boundary_violations() {
    let report = ComplianceReport::from_parts(
        vec![],
        vec![],
        vec![],
        2,
        85,
    );
    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert!(report.remediation_checklist.iter().any(|s| s.contains("boundary")));
}

#[test]
fn compliance_report_serde_status() {
    let report = ComplianceReport::from_parts(vec![], vec![], vec![], 0, 100);
    let json = serde_json::to_string(&report.status).unwrap();
    assert_eq!(json, "\"compliant\"");
    let parsed: ComplianceStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, ComplianceStatus::Compliant);
}
