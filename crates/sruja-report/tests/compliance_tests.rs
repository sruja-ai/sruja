use sruja_report::{ComplianceReport, ComplianceStatus, DriftEntry, PolicyViolationEntry};

#[test]
fn test_compliant_report() {
    // Test that a report with no violations is compliant
    let report = ComplianceReport::from_parts(
        vec![], // structural_violations
        vec![], // drift_entries
        vec![], // policy_violations
        0,      // boundary_violations_count
        100,    // health_score
    );

    assert_eq!(report.status, ComplianceStatus::Compliant);
    assert_eq!(report.health_score, 100);
    assert!(report.structural_violations.is_empty());
    assert!(report.drift_entries.is_empty());
    assert!(report.policy_violations.is_empty());
    assert_eq!(report.boundary_violations_count, 0);
    assert!(report.remediation_checklist.is_empty());
}

#[test]
fn test_non_compliant_structural_violations() {
    // Test that structural violations make report non-compliant
    let violation = sruja_diff::Violation {
        kind: sruja_diff::ViolationKind::CircularDependency,
        message: "Circular dependency detected".to_string(),
        severity: sruja_diff::Severity::Error,
        location: Some("ComponentA -> ComponentB".to_string()),
        suggestion: None,
        sources: vec![],
    };

    let report = ComplianceReport::from_parts(
        vec![violation], // structural_violations
        vec![],          // drift_entries
        vec![],          // policy_violations
        0,               // boundary_violations_count
        80,              // health_score
    );

    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert_eq!(report.health_score, 80);
    assert_eq!(report.structural_violations.len(), 1);
    assert!(report.drift_entries.is_empty());
    assert!(report.policy_violations.is_empty());
    assert_eq!(report.boundary_violations_count, 0);
    assert!(!report.remediation_checklist.is_empty());
    assert!(report.remediation_checklist[0].contains("Fix 1 structural violation"));
}

#[test]
fn test_non_compliant_policy_violations() {
    // Test that policy violations make report non-compliant
    let report = ComplianceReport::from_parts(
        vec![], // structural_violations
        vec![], // drift_entries
        vec![PolicyViolationEntry {
            policy_name: "NoHardcodedSecrets".to_string(),
            message: "Hardcoded secret found".to_string(),
            source: "service.js".to_string(),
            target: "config".to_string(),
        }], // policy_violations
        0,      // boundary_violations_count
        70,     // health_score
    );

    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert_eq!(report.health_score, 70);
    assert!(report.structural_violations.is_empty());
    assert!(report.drift_entries.is_empty());
    assert_eq!(report.policy_violations.len(), 1);
    assert_eq!(report.boundary_violations_count, 0);
    assert!(!report.remediation_checklist.is_empty());
    assert!(report.remediation_checklist[0].contains("Resolve 1 policy violation"));
}

#[test]
fn test_non_compliant_boundary_violations() {
    // Test that boundary violations make report non-compliant
    let report = ComplianceReport::from_parts(
        vec![], // structural_violations
        vec![], // drift_entries
        vec![], // policy_violations
        2,      // boundary_violations_count
        90,     // health_score
    );

    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert_eq!(report.health_score, 90);
    assert!(report.structural_violations.is_empty());
    assert!(report.drift_entries.is_empty());
    assert!(report.policy_violations.is_empty());
    assert_eq!(report.boundary_violations_count, 2);
    assert!(!report.remediation_checklist.is_empty());
    println!("Remediation checklist: {:?}", report.remediation_checklist);
    assert!(report.remediation_checklist[0].contains("Fix 2 boundary violation"));
}

#[test]
fn test_drift_entries_remediation() {
    // Test that drift entries generate remediation only when no other violations
    let report = ComplianceReport::from_parts(
        vec![], // structural_violations
        vec![
            DriftEntry {
                kind: "UndocumentedComponent".to_string(),
                severity: "warning".to_string(),
                description: "Component UserService is undocumented".to_string(),
                suggestion: Some("Add ADR for UserService".to_string()),
            },
            DriftEntry {
                kind: "MissingRelationship".to_string(),
                severity: "info".to_string(),
                description: "Relationship between UserService and Database is missing".to_string(),
                suggestion: None,
            },
        ], // drift_entries
        vec![], // policy_violations
        0,      // boundary_violations_count
        85,     // health_score
    );

    assert_eq!(report.status, ComplianceStatus::Compliant); // Still compliant as drift alone doesn't make non-compliant
    assert_eq!(report.health_score, 85);
    assert!(report.structural_violations.is_empty());
    assert_eq!(report.drift_entries.len(), 2);
    assert!(report.policy_violations.is_empty());
    assert_eq!(report.boundary_violations_count, 0);
    assert!(!report.remediation_checklist.is_empty());
    assert!(report.remediation_checklist[0].contains("Address 2 intent drift(s)"));
}

#[test]
fn test_drift_entries_no_remediation_when_other_violations() {
    // Test that drift entries don't generate remediation when other violations exist
    let violation = sruja_diff::Violation {
        kind: sruja_diff::ViolationKind::OrphanComponent,
        message: "Orphan component detected".to_string(),
        severity: sruja_diff::Severity::Warning,
        location: Some("OldService".to_string()),
        suggestion: None,
        sources: vec![],
    };

    let report = ComplianceReport::from_parts(
        vec![violation], // structural_violations
        vec![DriftEntry {
            kind: "UndocumentedComponent".to_string(),
            severity: "warning".to_string(),
            description: "Component UserService is undocumented".to_string(),
            suggestion: Some("Add ADR for UserService".to_string()),
        }], // drift_entries
        vec![],          // policy_violations
        0,               // boundary_violations_count
        85,              // health_score
    );

    assert_eq!(report.status, ComplianceStatus::NonCompliant);
    assert_eq!(report.health_score, 85);
    assert_eq!(report.structural_violations.len(), 1);
    assert_eq!(report.drift_entries.len(), 1);
    assert!(report.policy_violations.is_empty());
    assert_eq!(report.boundary_violations_count, 0);
    // Should NOT contain drift remediation since there are structural violations
    assert!(!report.remediation_checklist.is_empty());
    assert!(!report.remediation_checklist[0].contains("Address 1 intent drift"));
    assert!(report.remediation_checklist[0].contains("Fix 1 structural violation"));
}

#[test]
fn test_serialization_deserialization() {
    // Test that the structs can be serialized and deserialized correctly
    let original_report = ComplianceReport::from_parts(
        vec![sruja_diff::Violation {
            kind: sruja_diff::ViolationKind::LayerViolation,
            message: "Layer violation".to_string(),
            severity: sruja_diff::Severity::Error,
            location: Some("Web -> DB".to_string()),
            suggestion: None,
            sources: vec![],
        }],
        vec![DriftEntry {
            kind: "UndocumentedComponent".to_string(),
            severity: "warning".to_string(),
            description: "Undocumented service".to_string(),
            suggestion: None,
        }],
        vec![PolicyViolationEntry {
            policy_name: "DataEncryption".to_string(),
            message: "Data not encrypted".to_string(),
            source: "api.ts".to_string(),
            target: "database".to_string(),
        }],
        1,  // boundary_violations_count
        75, // health_score
    );

    // Serialize to JSON
    let json = serde_json::to_string(&original_report).expect("Failed to serialize");

    // Deserialize from JSON
    let deserialized: ComplianceReport =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Check that values match
    assert_eq!(deserialized.status, ComplianceStatus::NonCompliant);
    assert_eq!(deserialized.health_score, 75);
    assert_eq!(deserialized.structural_violations.len(), 1);
    assert_eq!(deserialized.drift_entries.len(), 1);
    assert_eq!(deserialized.policy_violations.len(), 1);
    assert_eq!(deserialized.boundary_violations_count, 1);
    assert!(!deserialized.remediation_checklist.is_empty());
}

#[test]
fn test_enum_serialization() {
    // Test ComplianceStatus serialization
    let compliant = ComplianceStatus::Compliant;
    let non_compliant = ComplianceStatus::NonCompliant;

    let compliant_json = serde_json::to_string(&compliant).expect("Failed to serialize");
    let non_compliant_json = serde_json::to_string(&non_compliant).expect("Failed to serialize");

    assert_eq!(compliant_json, "\"compliant\"");
    assert_eq!(non_compliant_json, "\"non_compliant\"");

    let deserialized_compliant: ComplianceStatus =
        serde_json::from_str(&compliant_json).expect("Failed to deserialize");
    let deserialized_non_compliant: ComplianceStatus =
        serde_json::from_str(&non_compliant_json).expect("Failed to deserialize");

    assert_eq!(deserialized_compliant, ComplianceStatus::Compliant);
    assert_eq!(deserialized_non_compliant, ComplianceStatus::NonCompliant);
}
