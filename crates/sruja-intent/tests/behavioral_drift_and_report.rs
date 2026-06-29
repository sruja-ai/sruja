use sruja_intent::{behavioral_drift::check_behavioral_drift, critique::CritiqueEvidence};
use sruja_intent::{
    format_critique_json, format_critique_text, CritiqueCategory, CritiqueFinding, CritiqueReport,
    CritiqueSeverity, RiskLevel,
};
use sruja_language::Program;
use sruja_scan::{Graph, Node, NodeKind, ResolvedContract, ResolvedStateMachine};

fn mk_graph_with_behavioral_contracts(node_path: Option<&str>) -> Graph {
    let mut graph = Graph::default();
    graph.nodes.push(Node {
        id: "Svc".to_string(),
        kind: NodeKind::new(NodeKind::COMPONENT),
        label: "Service".to_string(),
        path: node_path.map(|p| p.to_string()),
        state_machines: vec![ResolvedStateMachine {
            name: "Lifecycle".to_string(),
            states: vec!["Created".to_string(), "Done".to_string()],
            initial_state: "Created".to_string(),
            terminal_states: vec!["Done".to_string()],
            transitions: vec![],
        }],
        contracts: vec![ResolvedContract {
            name: "GetUser".to_string(),
            description: Some("Fetch user".to_string()),
            inputs: vec![],
            outputs: vec![],
            errors: vec![],
            constraints: vec![],
        }],
        ..Default::default()
    });
    graph
}

#[test]
fn behavioral_drift_returns_findings_when_node_file_changed() {
    let graph = mk_graph_with_behavioral_contracts(Some("src/svc.rs"));
    let program = Program::new();

    let findings = check_behavioral_drift(
        &graph,
        &program,
        &["src/svc.rs".to_string()],
        &["Svc".to_string()],
    );

    assert_eq!(findings.len(), 2, "expected one finding per contract type");
    assert!(findings
        .iter()
        .all(|f| f.category == CritiqueCategory::BehavioralContractDrift));
    assert!(findings
        .iter()
        .all(|f| f.severity == CritiqueSeverity::High));
    assert!(findings
        .iter()
        .any(|f| f.title.contains("State Machine Impact")));
    assert!(findings
        .iter()
        .any(|f| f.title.contains("API Contract Impact")));
}

#[test]
fn behavioral_drift_is_empty_when_node_has_no_path() {
    let graph = mk_graph_with_behavioral_contracts(None);
    let program = Program::new();

    let findings = check_behavioral_drift(
        &graph,
        &program,
        &["src/svc.rs".to_string()],
        &["Svc".to_string()],
    );

    assert!(findings.is_empty());
}

#[test]
fn critique_report_formatters_are_stable_and_include_expected_fields() {
    let report = CritiqueReport {
        violations: vec![
            CritiqueFinding {
                category: CritiqueCategory::ConstraintBreach,
                severity: CritiqueSeverity::Medium,
                title: "Constraint: Svc".to_string(),
                detail: "Must remain idempotent".to_string(),
                evidence: vec![CritiqueEvidence {
                    source: "sruja".to_string(),
                    location: Some("src/svc.rs".to_string()),
                    detail: "declared constraint".to_string(),
                }],
                suggestion: Some("Add regression test".to_string()),
                confidence: 1.0,
                rule_id: Some("SRUJA-INTENT-POLICY-001".to_string()),
            },
            CritiqueFinding {
                category: CritiqueCategory::BehavioralContractDrift,
                severity: CritiqueSeverity::High,
                title: "API Contract Impact: GetUser".to_string(),
                detail: "Verify input/output shapes".to_string(),
                evidence: vec![],
                suggestion: None,
                confidence: 0.8,
                rule_id: Some("SRUJA-INTENT-POLICY-002".to_string()),
            },
        ],
        context: vec![],
        risk_level: RiskLevel::Warning,
        summary: "2 issues found".to_string(),
        affected_elements: vec!["Svc".to_string()],
        blast_radius: sruja_intent::critique::BlastRadiusSummary {
            total_affected_elements: 1,
            downstream_consumers: 0,
            max_depth: 0,
        },
        baseline_present: true,
    };

    let text = format_critique_text(&report);
    assert!(text.contains("ARCHITECTURAL CRITIQUE"));
    assert!(text.contains("Summary: 2 issues found"));
    assert!(text.contains("Constraint: Svc"));
    assert!(text.contains("API Contract Impact: GetUser"));

    let json = format_critique_json(&report);
    assert!(json.contains("\"risk_level\""));
    assert!(json.contains("\"summary\""));
    assert!(json.contains("2 issues found"));
}
