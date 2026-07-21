//! Architectural drift detection: cycles, orphans, layer violations, god modules.

mod cycles;
mod god_modules;
mod helpers;
mod layers;
mod orphans;

pub use cycles::find_circular_dependencies;
pub use helpers::is_likely_entry_point;
pub use orphans::{find_orphan_modules, find_orphan_modules_with_config};
pub use layers::find_layer_violations_advanced;
pub use god_modules::find_god_modules_with_config;

use crate::health::calculate_health_score_with_breakdown;
use crate::source_ref::{collect_cycle_sources, collect_edge_sources, collect_node_path_source};
use crate::types::HealthScorePenalties;
use crate::types::{
    DriftConfig, DriftReport, HealthScoreBreakdown, Severity, TruthStatus, Violation, ViolationKind,
};
use helpers::{is_likely_doc_or_tool_path, top_targets_for_module};
use sruja_scan::{Graph, NodeKind};

/// Detect architectural drift in a codebase by analyzing the scanned graph
/// for common architectural issues like circular dependencies, god modules,
/// layer violations, and orphan components.
pub fn detect_architectural_drift(graph: &Graph) -> DriftReport {
    detect_architectural_drift_with_config(graph, &DriftConfig::default())
}

/// Detect architectural drift with custom configuration.
pub fn detect_architectural_drift_with_config(graph: &Graph, config: &DriftConfig) -> DriftReport {
    let mut violations = Vec::new();
    let mut suggestions = Vec::new();

    let circular = find_circular_dependencies(graph);
    for cycle in &circular {
        let sources = collect_cycle_sources(graph, cycle);
        violations.push(Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: format!("Circular dependency detected: {}", cycle.join(" -> ")),
            location: Some(cycle.first().cloned().unwrap_or_default()),
            suggestion: Some(
                "Consider introducing an interface or event-based communication to break the cycle"
                    .to_string(),
            ),
            sources: sources.clone(),
            confidence: None,
            evidence_count: Some(sources.len()),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        });
    }

    let orphans = find_orphan_modules_with_config(graph, config.exclude_barrel_files);
    for orphan in &orphans {
        let sources = collect_node_path_source(graph, orphan);
        violations.push(Violation {
            kind: ViolationKind::OrphanComponent,
            severity: Severity::Info,
            message: format!(
                "Module '{}' has no incoming or outgoing dependencies",
                orphan
            ),
            location: Some(orphan.clone()),
            suggestion: Some(format!(
                "Module '{}' has no import/export connections. \
                 If it's consumed via framework DI or reflection, this is expected. \
                 Otherwise, consider removing or connecting it.",
                orphan
            )),
            sources: sources.clone(),
            confidence: None,
            evidence_count: Some(sources.len()),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        });
    }

    let layer_violations = find_layer_violations_advanced(graph);
    for violation in &layer_violations {
        let sources = collect_edge_sources(graph, &violation.source, &violation.target);
        violations.push(Violation {
            kind: ViolationKind::LayerViolation,
            severity: Severity::Warning,
            message: format!(
                "Layer violation: '{}' directly accesses '{}'",
                violation.source, violation.target
            ),
            location: Some(format!("{} -> {}", violation.source, violation.target)),
            suggestion: Some(
                "Consider adding a service layer to abstract this dependency".to_string(),
            ),
            sources: sources.clone(),
            confidence: None,
            evidence_count: Some(sources.len()),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        });
    }

    let god_modules = find_god_modules_with_config(
        graph,
        config.god_module_threshold,
        config.exclude_barrel_files,
    );
    for module in &god_modules {
        let sources = collect_node_path_source(graph, &module.name);
        violations.push(Violation {
            kind: ViolationKind::GodModule,
            severity: Severity::Warning,
            message: format!(
                "Bottleneck Detected: Module '{}' acts as a 'God Module' with {} dependencies (threshold: {})",
                module.name, module.dependency_count, config.god_module_threshold
            ),
            location: Some(module.name.clone()),
            suggestion: Some(format!(
                "Bottleneck detected: this module has {} outgoing dependencies. Top targets: {}. \
                 Consider extracting related functionality into a separate module to reduce regression risk.",
                module.dependency_count,
                top_targets_for_module(graph, &module.name, 3).join(", ")
            )),
            sources: sources.clone(),
            confidence: None,
            evidence_count: Some(sources.len()),
            production_relevant: None,
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        });
    }

    for v in &mut violations {
        let mut prod_rel = true;
        for s in &v.sources {
            if let Some(ref file) = s.file {
                if is_likely_doc_or_tool_path(file, &v.location.clone().unwrap_or_default()) {
                    prod_rel = false;
                    break;
                }
            }
        }
        v.production_relevant = Some(prod_rel);
        crate::types::annotate_violation_metadata(v);
    }

    if !circular.is_empty() {
        suggestions.push("Fix circular dependencies to improve maintainability".to_string());
    }
    if !orphans.is_empty() {
        suggestions
            .push("Review orphan modules - they may be dead code or need integration".to_string());
    }
    if !layer_violations.is_empty() {
        suggestions.push("Introduce proper layering to reduce coupling".to_string());
    }
    if !god_modules.is_empty() {
        suggestions.push("Refactor god modules into smaller components".to_string());
    }

    let penalties = HealthScorePenalties::default();
    let breakdown = calculate_health_score_with_breakdown(&violations, penalties);
    let health_breakdown = Some(HealthScoreBreakdown {
        cycle_penalty: breakdown.cycle_penalty,
        layer_penalty: breakdown.zone_of_pain_penalty,
        god_module_penalty: breakdown.god_module_penalty,
        orphan_penalty: breakdown.orphan_penalty,
        other_penalty: breakdown.other_penalty,
    });

    DriftReport {
        scan_scope: sruja_scan::scan_scope::ScanScope::default(),
        truth_status: TruthStatus::Unknown,
        total_modules: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::MODULE)
            .count(),
        total_services: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::SERVICE)
            .count(),
        total_databases: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::DATABASE)
            .count(),
        total_dependencies: graph.edges.len(),
        circular_dependencies: circular.len(),
        orphan_modules: orphans.len(),
        layer_violations: layer_violations.len(),
        violations,
        suggestions,
        health_score: breakdown.score,
        health_breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_architectural_drift_with_config, find_circular_dependencies,
        find_god_modules_with_config, find_layer_violations_advanced,
        find_orphan_modules_with_config,
    };
    use crate::types::{DriftConfig, ViolationKind};
    use sruja_scan::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};

    fn node(id: &str, kind: NodeKind, path: Option<&str>) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            kind,
            path: path.map(String::from),
            ..Node::default()
        }
    }

    fn edge(source: &str, target: &str) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![],
            confidence: Default::default(),
        }
    }

    #[test]
    fn find_circular_dependencies_detects_simple_cycle() {
        let mut g = Graph::default();
        g.nodes
            .push(node("a", NodeKind::new(NodeKind::MODULE), None));
        g.nodes
            .push(node("b", NodeKind::new(NodeKind::MODULE), None));
        g.edges.push(edge("a", "b"));
        g.edges.push(edge("b", "a"));

        let cycles = find_circular_dependencies(&g);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 2);
    }

    #[test]
    fn find_circular_dependencies_canonicalizes_cycle_rotation() {
        let mut g = Graph::default();
        g.nodes
            .push(node("b", NodeKind::new(NodeKind::MODULE), None));
        g.nodes
            .push(node("c", NodeKind::new(NodeKind::MODULE), None));
        g.nodes
            .push(node("a", NodeKind::new(NodeKind::MODULE), None));
        g.edges.push(edge("b", "c"));
        g.edges.push(edge("c", "a"));
        g.edges.push(edge("a", "b"));

        let cycles = find_circular_dependencies(&g);
        assert_eq!(cycles.len(), 1);
        assert_eq!(
            cycles[0],
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn find_circular_dependencies_no_cycle_returns_empty() {
        let mut g = Graph::default();
        g.nodes
            .push(node("a", NodeKind::new(NodeKind::MODULE), None));
        g.nodes
            .push(node("b", NodeKind::new(NodeKind::MODULE), None));
        g.edges.push(edge("a", "b"));

        let cycles = find_circular_dependencies(&g);
        assert!(cycles.is_empty());
    }

    #[test]
    fn find_orphan_modules_detects_isolated_node() {
        let mut g = Graph::default();
        g.nodes
            .push(node("a", NodeKind::new(NodeKind::MODULE), Some("src/a.rs")));
        g.nodes
            .push(node("b", NodeKind::new(NodeKind::MODULE), Some("src/b.rs")));
        g.edges.push(edge("a", "b"));

        let orphans = find_orphan_modules_with_config(&g, true);
        assert!(orphans.is_empty(), "a and b are connected");

        g.nodes.push(node(
            "orphan",
            NodeKind::new(NodeKind::MODULE),
            Some("src/orphan.rs"),
        ));
        let orphans = find_orphan_modules_with_config(&g, true);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0], "orphan");
    }

    #[test]
    fn find_orphan_modules_excludes_barrel_files_when_configured() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "helper_mod",
            NodeKind::new(NodeKind::MODULE),
            Some("src/utils/helper_mod.rs"),
        ));
        g.nodes.push(node(
            "helper_init",
            NodeKind::new(NodeKind::MODULE),
            Some("src/utils/helper_init.py"),
        ));
        g.nodes.push(node(
            "helper_index",
            NodeKind::new(NodeKind::MODULE),
            Some("src/utils/helper_index.ts"),
        ));
        g.nodes.push(node(
            "real_orphan",
            NodeKind::new(NodeKind::MODULE),
            Some("src/real_orphan.rs"),
        ));

        let orphans = find_orphan_modules_with_config(&g, false);
        assert_eq!(orphans.len(), 2, "entry point filter excludes some files");
        assert!(orphans.contains(&"helper_init".to_string()));
        assert!(orphans.contains(&"real_orphan".to_string()));

        let orphans = find_orphan_modules_with_config(&g, true);
        assert_eq!(orphans.len(), 2, "non-barrel files should not be excluded");
    }

    #[test]
    fn find_god_modules_excludes_barrel_files_when_configured() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "barrel",
            NodeKind::new(NodeKind::MODULE),
            Some("tests/utils/helpers/mod.rs"),
        ));
        for i in 0..15 {
            let dep = format!("dep_{}", i);
            g.nodes.push(node(
                &dep,
                NodeKind::new(NodeKind::MODULE),
                Some(&format!("src/{}.rs", dep)),
            ));
            g.edges.push(edge("barrel", &dep));
        }
        g.nodes.push(node(
            "god",
            NodeKind::new(NodeKind::MODULE),
            Some("src/god.rs"),
        ));
        for i in 0..15 {
            let dep = format!("god_dep_{}", i);
            g.nodes.push(node(
                &dep,
                NodeKind::new(NodeKind::MODULE),
                Some(&format!("src/{}.rs", dep)),
            ));
            g.edges.push(edge("god", &dep));
        }

        let gods = find_god_modules_with_config(&g, 10, false);
        assert_eq!(
            gods.len(),
            1,
            "entry point filter already excludes barrel files"
        );
        assert_eq!(gods[0].name, "god");

        let gods = find_god_modules_with_config(&g, 10, true);
        assert_eq!(
            gods.len(),
            1,
            "barrel exclusion confirms entry point filter"
        );
        assert_eq!(gods[0].name, "god");
    }

    #[test]
    fn find_orphan_modules_excludes_mod_rs_and_init_py() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "mod_node",
            NodeKind::new(NodeKind::MODULE),
            Some("tests/helpers/mod.rs"),
        ));
        g.nodes.push(node(
            "init_node",
            NodeKind::new(NodeKind::MODULE),
            Some("tests/helpers/__init__.py"),
        ));
        g.nodes.push(node(
            "index_node",
            NodeKind::new(NodeKind::MODULE),
            Some("tests/helpers/index.ts"),
        ));
        g.nodes.push(node(
            "real_orphan",
            NodeKind::new(NodeKind::MODULE),
            Some("tests/real_orphan.rs"),
        ));

        let orphans = find_orphan_modules_with_config(&g, false);
        assert_eq!(
            orphans.len(),
            1,
            "entry point filter already excludes barrel files"
        );
        assert_eq!(orphans[0], "real_orphan");

        let orphans = find_orphan_modules_with_config(&g, true);
        assert_eq!(
            orphans.len(),
            1,
            "barrel exclusion confirms entry point filter"
        );
        assert_eq!(orphans[0], "real_orphan");
    }

    #[test]
    fn find_layer_violations_advanced_detects_frontend_to_db_access() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "web_frontend",
            NodeKind::new(NodeKind::MODULE),
            Some("src/web.rs"),
        ));
        g.nodes.push(node(
            "db",
            NodeKind::new(NodeKind::DATABASE),
            Some("src/db.rs"),
        ));
        g.nodes.push(node(
            "service",
            NodeKind::new(NodeKind::MODULE),
            Some("src/service.rs"),
        ));
        g.edges.push(edge("web_frontend", "db"));
        g.edges.push(edge("service", "db"));

        let violations = find_layer_violations_advanced(&g);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].source, "web_frontend");
        assert_eq!(violations[0].target, "db");
    }

    #[test]
    fn find_god_modules_respects_threshold_and_excludes_doc_paths() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "god",
            NodeKind::new(NodeKind::MODULE),
            Some("src/god.rs"),
        ));
        g.nodes.push(node(
            "docs_mod",
            NodeKind::new(NodeKind::MODULE),
            Some("src/doc/readme.rs"),
        ));
        for i in 0..3 {
            let dep = format!("dep_{i}");
            g.nodes.push(node(
                &dep,
                NodeKind::new(NodeKind::MODULE),
                Some(&format!("src/{dep}.rs")),
            ));
            g.edges.push(edge("god", &dep));
            g.edges.push(edge("docs_mod", &dep));
        }

        let god_modules = find_god_modules_with_config(&g, 3, true);
        assert_eq!(god_modules.len(), 1);
        assert_eq!(god_modules[0].name, "god");
        assert_eq!(god_modules[0].dependency_count, 3);
    }

    #[test]
    fn detect_architectural_drift_includes_source_refs_for_layer_and_god_module_violations() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "web",
            NodeKind::new(NodeKind::MODULE),
            Some("src/web.rs"),
        ));
        g.nodes.push(node(
            "db",
            NodeKind::new(NodeKind::DATABASE),
            Some("src/db.rs"),
        ));
        g.nodes.push(node(
            "god",
            NodeKind::new(NodeKind::MODULE),
            Some("src/god.rs"),
        ));

        g.edges.push(Edge {
            source: "web".to_string(),
            target: "db".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: "scan".to_string(),
                file: Some("src/web.rs".to_string()),
                line: Some(12),
                detail: Some("db.query".to_string()),
            }],
            confidence: Default::default(),
        });

        for i in 0..2 {
            let dep = format!("dep_{i}");
            g.nodes.push(node(
                &dep,
                NodeKind::new(NodeKind::MODULE),
                Some(&format!("src/{dep}.rs")),
            ));
            g.edges.push(edge("god", &dep));
        }

        let config = DriftConfig {
            god_module_threshold: 2,
            exclude_barrel_files: true,
        };
        let report = detect_architectural_drift_with_config(&g, &config);

        let layer = report
            .violations
            .iter()
            .find(|v| v.kind == ViolationKind::LayerViolation)
            .expect("layer violation");
        assert!(
            layer
                .sources
                .iter()
                .any(|s| { s.file.as_deref() == Some("src/web.rs") && s.line == Some(12) }),
            "layer violation should include evidence-derived sources"
        );

        let god = report
            .violations
            .iter()
            .find(|v| v.kind == ViolationKind::GodModule)
            .expect("god module violation");
        assert!(
            god.sources
                .iter()
                .any(|s| s.file.as_deref() == Some("src/god.rs")),
            "god module violation should include node path source"
        );
    }

    #[test]
    fn detect_architectural_drift_ignores_non_production_violations_for_health() {
        let mut g = Graph::default();
        g.nodes.push(node(
            "web",
            NodeKind::new(NodeKind::MODULE),
            Some("src/test/web.rs"),
        ));
        g.nodes.push(node(
            "db",
            NodeKind::new(NodeKind::DATABASE),
            Some("src/db.rs"),
        ));

        g.edges.push(Edge {
            source: "web".to_string(),
            target: "db".to_string(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: "scan".to_string(),
                file: Some("src/test/web.rs".to_string()),
                line: Some(12),
                detail: Some("db.query".to_string()),
            }],
            confidence: Default::default(),
        });

        let config = DriftConfig::default();
        let report = detect_architectural_drift_with_config(&g, &config);

        let layer = report
            .violations
            .iter()
            .find(|v| v.kind == ViolationKind::LayerViolation)
            .expect("layer violation");
        assert_eq!(layer.production_relevant, Some(false));
        assert_eq!(
            report.health_score, 100,
            "non-production violation shouldn't penalize health score"
        );
    }
}
