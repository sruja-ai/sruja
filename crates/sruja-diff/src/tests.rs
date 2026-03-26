//! Tests for graph comparison and drift detection.

#[cfg(test)]
mod cases {
    use crate::types::{
        DiffResult, DiffSummary, EdgeDiff, HealthScorePenalties, NodeDiff, Severity, SourceRef,
        TruthStatus, Violation, ViolationKind,
    };
    use crate::{
        calculate_health_score_from_violations, compare_graphs, detect_architectural_drift,
        program_to_graph,
    };
    use sruja_language::Parser;
    use sruja_scan::{Edge, EdgeKind, Graph, Node, NodeKind};
    use std::collections::HashMap;

    fn make_node(id: &str, kind: NodeKind, label: &str) -> Node {
        Node {
            id: id.to_string(),
            kind,
            label: label.to_string(),
            technology: None,
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
        }
    }

    fn make_edge(source: &str, target: &str, kind: EdgeKind) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind,
            evidence: Vec::new(),
        }
    }

    fn make_edge_with_evidence(
        source: &str,
        target: &str,
        kind: EdgeKind,
        file: Option<&str>,
        line: Option<u32>,
    ) -> Edge {
        use sruja_scan::EdgeEvidence;
        let evidence = if file.is_some() || line.is_some() {
            vec![EdgeEvidence {
                rule: "test".to_string(),
                file: file.map(String::from),
                line,
                detail: None,
            }]
        } else {
            Vec::new()
        };
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind,
            evidence,
        }
    }

    #[test]
    fn test_compare_empty_graphs() {
        let actual = Graph::new();
        let proposed = Graph::new();
        let result = compare_graphs(&actual, &proposed);
        assert!(result.is_empty());
    }

    #[test]
    fn test_detect_added_node() {
        let actual = Graph::new();
        let mut proposed = Graph::new();
        proposed
            .nodes
            .push(make_node("api", NodeKind::Service, "API"));

        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.added.len(), 1);
        assert_eq!(result.node_diff.added[0].id, "api");
    }

    #[test]
    fn test_detect_removed_node() {
        let mut actual = Graph::new();
        actual
            .nodes
            .push(make_node("old", NodeKind::Service, "Old Service"));
        let proposed = Graph::new();

        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.removed.len(), 1);
    }

    #[test]
    fn test_detect_layer_violation() {
        let actual = Graph::new();
        let mut proposed = Graph::new();

        proposed
            .nodes
            .push(make_node("frontend", NodeKind::Module, "Frontend"));
        proposed
            .nodes
            .push(make_node("db", NodeKind::Database, "Database"));
        proposed
            .edges
            .push(make_edge("frontend", "db", EdgeKind::ReadsFrom));

        let result = compare_graphs(&actual, &proposed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::LayerViolation));
    }

    #[test]
    fn test_detect_architectural_drift_cycle_and_orphan() {
        let mut graph = Graph::new();
        graph.nodes.push(make_node("a", NodeKind::Module, "A"));
        graph.nodes.push(make_node("b", NodeKind::Module, "B"));
        graph.nodes.push(make_node("c", NodeKind::Module, "C"));
        graph
            .nodes
            .push(make_node("orphan", NodeKind::Module, "Orphan"));
        graph.edges.push(make_edge("a", "b", EdgeKind::Calls));
        graph.edges.push(make_edge("b", "c", EdgeKind::Calls));
        graph.edges.push(make_edge("c", "a", EdgeKind::Calls));

        let report = detect_architectural_drift(&graph);

        assert!(report
            .violations
            .iter()
            .any(|v| { v.kind == ViolationKind::CircularDependency }));
        assert!(report
            .violations
            .iter()
            .any(|v| { v.kind == ViolationKind::OrphanComponent }));
        assert!(report.health_score <= 100);
        assert_eq!(report.total_modules, 4);
        assert!(!report.suggestions.is_empty());
    }

    #[test]
    fn test_drift_cycle_violation_includes_source_refs_when_edges_have_evidence() {
        let mut graph = Graph::new();
        graph.nodes.push(make_node("x", NodeKind::Module, "X"));
        graph.nodes.push(make_node("y", NodeKind::Module, "Y"));
        graph.edges.push(make_edge_with_evidence(
            "x",
            "y",
            EdgeKind::Calls,
            Some("src/x.rs"),
            Some(10),
        ));
        graph.edges.push(make_edge_with_evidence(
            "y",
            "x",
            EdgeKind::Calls,
            Some("src/y.rs"),
            Some(20),
        ));

        let report = detect_architectural_drift(&graph);

        let cycle_violation = report
            .violations
            .iter()
            .find(|v| v.kind == ViolationKind::CircularDependency)
            .expect("cycle violation");
        assert!(
            !cycle_violation.sources.is_empty(),
            "source_ref: cycle violation should have sources when edges have evidence"
        );
    }

    #[test]
    fn test_health_score_no_overflow_with_many_orphans() {
        let actual = Graph::new();
        let mut proposed = Graph::new();

        for i in 0..200 {
            let id = format!("orphan_{}", i);
            proposed
                .nodes
                .push(make_node(&id, NodeKind::Module, &format!("Orphan {}", i)));
        }

        let result = std::panic::catch_unwind(|| compare_graphs(&actual, &proposed));

        assert!(
            result.is_ok(),
            "Health score calculation should not panic with many orphans"
        );
        let diff_result = result.unwrap();
        assert!(diff_result.summary.health_score <= 100);
    }

    #[test]
    fn test_calculate_health_score_empty_violations_returns_100() {
        let violations: Vec<Violation> = vec![];
        let penalties = HealthScorePenalties::default();
        let score = calculate_health_score_from_violations(&violations, penalties);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_calculate_health_score_with_cycle_deduction() {
        let violations = vec![Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: "Cycle A -> B -> A".to_string(),
            location: None,
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: Some(0),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        }];
        let penalties = HealthScorePenalties::default();
        let score = calculate_health_score_from_violations(&violations, penalties);
        assert!(score < 100, "cycle should reduce health score");
        assert!(score >= 30, "score should not go below minimum floor");
    }

    #[test]
    fn test_source_ref_display_string() {
        assert_eq!(
            SourceRef {
                file: Some("src/main.rs".to_string()),
                line: Some(42),
                detail: None,
            }
            .display_string(),
            "src/main.rs:42"
        );
        assert_eq!(
            SourceRef {
                file: Some("lib.ts".to_string()),
                line: None,
                detail: None,
            }
            .display_string(),
            "lib.ts"
        );
        assert_eq!(
            SourceRef {
                file: None,
                line: None,
                detail: Some("custom".to_string()),
            }
            .display_string(),
            "custom"
        );
    }

    #[test]
    fn test_calculate_health_score_with_multiple_violation_kinds() {
        let violations = vec![
            Violation {
                kind: ViolationKind::OrphanComponent,
                severity: Severity::Warning,
                message: "Orphan".to_string(),
                location: Some("mod_x".to_string()),
                suggestion: None,
                sources: vec![],
                confidence: None,
                evidence_count: Some(0),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            },
            Violation {
                kind: ViolationKind::LayerViolation,
                severity: Severity::Warning,
                message: "Layer violation".to_string(),
                location: None,
                suggestion: None,
                sources: vec![],
                confidence: None,
                evidence_count: Some(0),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            },
        ];
        let penalties = HealthScorePenalties::default();
        let score = calculate_health_score_from_violations(&violations, penalties);
        assert!(score < 100);
    }

    // --- program_to_graph (DSL -> scan graph) ---

    fn parse_dsl(input: &str) -> sruja_language::Program {
        Parser::new("test.sruja".to_string())
            .parse(input)
            .expect("parse should succeed")
    }

    #[test]
    fn test_program_to_graph_empty_produces_empty_graph() {
        let program = parse_dsl("");
        let graph = program_to_graph(&program);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_program_to_graph_systems_become_nodes() {
        let program = parse_dsl(
            r#"
A = system "System A"
B = system "System B"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.nodes.len(), 2);
        let ids: Vec<_> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"A"));
        assert!(ids.contains(&"B"));
    }

    #[test]
    fn test_program_to_graph_relation_becomes_edge() {
        let program = parse_dsl(
            r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, "A");
        assert_eq!(graph.edges[0].target, "B");
        assert_eq!(graph.edges[0].kind, EdgeKind::Calls);
    }

    #[test]
    fn test_program_to_graph_reads_label_maps_to_reads_from() {
        let program = parse_dsl(
            r#"
A = system "Service A"
DB = database "Database"
A -> DB "reads from"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::ReadsFrom);
    }

    #[test]
    fn test_program_to_graph_database_kind() {
        let program = parse_dsl(
            r#"
DB = database "Primary DB"
"#,
        );
        let graph = program_to_graph(&program);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].kind, NodeKind::Database);
    }

    #[test]
    fn test_diff_result_is_empty() {
        let empty = DiffResult {
            proposal_title: "Proposal".to_string(),
            node_diff: NodeDiff {
                added: vec![],
                removed: vec![],
                matched: vec![],
            },
            edge_diff: EdgeDiff {
                added: vec![],
                removed: vec![],
            },
            violations: vec![],
            suggestions: vec![],
            summary: DiffSummary {
                proposed_components: 0,
                existing_components: 0,
                new_components: 0,
                missing_components: 0,
                new_dependencies: 0,
                removed_dependencies: 0,
                health_score: 100,
            },
            truth_status: TruthStatus::Reviewed,
        };
        assert!(empty.is_empty());

        let with_added = DiffResult {
            node_diff: NodeDiff {
                added: vec![crate::types::DiffNode {
                    id: "new".to_string(),
                    kind: NodeKind::Module,
                    label: "New".to_string(),
                    technology: None,
                    description: None,
                }],
                removed: vec![],
                matched: vec![],
            },
            ..empty.clone()
        };
        assert!(!with_added.is_empty());
    }

    #[test]
    fn test_diff_result_has_issues() {
        let no_issues = DiffResult {
            proposal_title: "P".to_string(),
            node_diff: NodeDiff {
                added: vec![],
                removed: vec![],
                matched: vec![],
            },
            edge_diff: EdgeDiff {
                added: vec![],
                removed: vec![],
            },
            violations: vec![],
            suggestions: vec![],
            summary: DiffSummary {
                proposed_components: 0,
                existing_components: 0,
                new_components: 0,
                missing_components: 0,
                new_dependencies: 0,
                removed_dependencies: 0,
                health_score: 100,
            },
            truth_status: TruthStatus::Reviewed,
        };
        assert!(!no_issues.has_issues());

        let with_violation = DiffResult {
            violations: vec![Violation {
                kind: ViolationKind::OrphanComponent,
                severity: Severity::Warning,
                message: "Orphan".to_string(),
                location: None,
                suggestion: None,
                sources: vec![],
                confidence: None,
                evidence_count: Some(0),
                production_relevant: None,
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            }],
            truth_status: TruthStatus::Drifted,
            ..no_issues
        };
        assert!(with_violation.has_issues());
    }
}
