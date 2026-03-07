//! Analysis tools: semantic_analyze, complexity, run_analyze, intent_check, detect_drift_with_baseline.

use std::path::PathBuf;

use sruja_graph::KnowledgeGraph;
use sruja_intent::{DriftDetector, IntentIntelligence, IntentModel};
use sruja_language::Parser;
use sruja_report::{
    build_recommendations, ComprehensiveReport, IntentSection as ReportIntentSection,
    RuntimeSection, SemanticSection as ReportSemanticSection,
    StructuralSection as ReportStructuralSection,
};
use sruja_semantic::{analyze as run_semantic_analyze, embedding::StubEmbeddingProvider};

use crate::tools::{SrujaTool, ToolResponse};

fn build_complexity_json(
    nodes: &[String],
    edges: &[(String, String)],
    include_treewidth: bool,
    include_scc: bool,
    include_centrality: bool,
    include_coupling: bool,
) -> serde_json::Value {
    let mut result = serde_json::json!({
        "total_nodes": nodes.len(),
        "total_edges": edges.len(),
    });

    if include_scc {
        let scc_analyzer = sruja_graph::SccAnalyzer::new();
        let scc_result = scc_analyzer.analyze(nodes, edges);
        result["scc"] = serde_json::json!({
            "total_sccs": scc_result.total_sccs,
            "cyclic_sccs": scc_result.cyclic_sccs,
            "largest_scc_size": scc_result.largest_scc_size,
            "components": scc_result.components.iter().take(10).map(|c| serde_json::json!({
                "id": c.id,
                "nodes": c.nodes,
                "is_cyclic": c.is_cyclic,
                "internal_density": c.internal_density,
                "suggested_boundary": c.suggested_boundary,
            })).collect::<Vec<_>>(),
        });
    }

    if include_treewidth {
        let tw_analyzer = sruja_graph::TreewidthAnalyzer::new();
        let tw_result = tw_analyzer.analyze(nodes, edges);
        result["treewidth"] = serde_json::json!({
            "treewidth": tw_result.treewidth,
            "rating": format!("{}", tw_result.complexity_rating),
            "hotspots": tw_result.hotspots.iter().take(5).map(|h| serde_json::json!({
                "nodes": h.nodes,
                "treewidth": h.treewidth,
                "suggestion": h.suggested_refactor.description,
            })).collect::<Vec<_>>(),
        });
    }

    if include_centrality {
        let c_analyzer = sruja_graph::CentralityAnalyzer::new();
        let c_result = c_analyzer.analyze(nodes, edges);
        result["centrality"] = serde_json::json!({
            "top_hubs": c_result.top_hubs.iter().take(5).map(|h| serde_json::json!({
                "node": h.node,
                "degree": h.degree_centrality,
                "dependents": h.dependents,
            })).collect::<Vec<_>>(),
            "top_bridges": c_result.top_bridges.iter().take(5).map(|b| serde_json::json!({
                "node": b.node,
                "betweenness": b.betweenness,
            })).collect::<Vec<_>>(),
        });
    }

    if include_coupling {
        let cp_analyzer = sruja_graph::CouplingAnalyzer::new();
        let cp_result = cp_analyzer.analyze(nodes, edges);
        result["coupling"] = serde_json::json!({
            "avg_instability": cp_result.summary.avg_instability,
            "avg_abstractness": cp_result.summary.avg_abstractness,
            "avg_distance": cp_result.summary.avg_distance,
            "pain_zone_count": cp_result.summary.pain_zone_count,
            "uselessness_zone_count": cp_result.summary.uselessness_zone_count,
            "violations": cp_result.violations.iter().take(5).map(|v| serde_json::json!({
                "module": v.module,
                "type": format!("{:?}", v.violation_type),
                "suggestion": v.suggestion,
            })).collect::<Vec<_>>(),
        });
    }

    result
}

pub(super) fn execute_analyze(
    tool: &SrujaTool,
    _graph: &KnowledgeGraph,
    validate_path: fn(&str) -> Result<PathBuf, String>,
) -> Option<ToolResponse> {
    match tool {
        SrujaTool::SemanticAnalyze { repo_path } => match validate_path(repo_path) {
            Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                Ok(scan_graph) => {
                    let components: Vec<(String, String)> = scan_graph
                        .nodes
                        .iter()
                        .map(|n| {
                            let text = format!(
                                "{} {} {}",
                                n.label,
                                n.technology.as_deref().unwrap_or(""),
                                n.path.as_deref().unwrap_or("")
                            );
                            (n.id.clone(), text)
                        })
                        .collect();
                    let structural_edges: Vec<(String, String)> = scan_graph
                        .edges
                        .iter()
                        .map(|e| (e.source.clone(), e.target.clone()))
                        .collect();
                    let provider = StubEmbeddingProvider::new();
                    let fut = run_semantic_analyze(&components, &structural_edges, &provider, None);
                    let report = match tokio::runtime::Handle::try_current() {
                        Ok(handle) => handle.block_on(fut),
                        Err(_) => {
                            let rt = tokio::runtime::Runtime::new()
                                .map_err(|e| e.to_string())
                                .unwrap();
                            rt.block_on(fut)
                        }
                    };
                    match report {
                        Ok(r) => {
                            let result = serde_json::json!({
                                "component_count": r.summary.component_count,
                                "context_count": r.summary.context_count,
                                "hidden_coupling_count": r.summary.hidden_coupling_count,
                                "vocabulary_leak_count": r.summary.vocabulary_leak_count,
                                "health_score": r.summary.health_score,
                                "contexts": r.contexts.iter().map(|c| serde_json::json!({
                                    "name": c.name,
                                    "components": c.components,
                                })).collect::<Vec<_>>(),
                                "hidden_couplings": r.coupling.hidden_couplings.iter().take(20).map(|h| serde_json::json!({
                                    "source": h.source,
                                    "target": h.target,
                                    "similarity": h.similarity,
                                    "shared_concepts": h.shared_concepts,
                                })).collect::<Vec<_>>(),
                                "recommendations": r.coupling.recommendations,
                            });
                            Some(ToolResponse::success("semantic_analyze", result))
                        }
                        Err(e) => Some(ToolResponse::error("semantic_analyze", e.to_string())),
                    }
                }
                Err(e) => Some(ToolResponse::error("semantic_analyze", e.to_string())),
            },
            Err(e) => Some(ToolResponse::error("semantic_analyze", e)),
        },

        SrujaTool::Complexity {
            repo_path,
            treewidth,
            scc,
            centrality,
            coupling,
        } => match validate_path(repo_path) {
            Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                Ok(scan_graph) => {
                    let nodes: Vec<String> =
                        scan_graph.nodes.iter().map(|n| n.id.clone()).collect();
                    let edges: Vec<(String, String)> = scan_graph
                        .edges
                        .iter()
                        .map(|e| (e.source.clone(), e.target.clone()))
                        .collect();
                    let all = !treewidth && !scc && !centrality && !coupling;
                    let result = build_complexity_json(
                        &nodes,
                        &edges,
                        all || *treewidth,
                        all || *scc,
                        all || *centrality,
                        all || *coupling,
                    );
                    Some(ToolResponse::success("complexity", result))
                }
                Err(e) => Some(ToolResponse::error("complexity", e.to_string())),
            },
            Err(e) => Some(ToolResponse::error("complexity", e)),
        },

        SrujaTool::RunAnalyze {
            repo_path,
            traces_path: _,
            intent_path,
        } => {
            let validated_path = match validate_path(repo_path) {
                Ok(p) => p,
                Err(e) => return Some(ToolResponse::error("run_analyze", e)),
            };
            let scan_graph = match sruja_scan::scan_repo(&validated_path) {
                Ok(g) => g,
                Err(e) => return Some(ToolResponse::error("run_analyze", e.to_string())),
            };

            let structural_report = sruja_diff::detect_architectural_drift(&scan_graph);
            let components: Vec<(String, String)> = scan_graph
                .nodes
                .iter()
                .map(|n| {
                    let text = format!(
                        "{} {} {}",
                        n.label,
                        n.technology.as_deref().unwrap_or(""),
                        n.path.as_deref().unwrap_or("")
                    );
                    (n.id.clone(), text)
                })
                .collect();
            let structural_edges: Vec<(String, String)> = scan_graph
                .edges
                .iter()
                .map(|e| (e.source.clone(), e.target.clone()))
                .collect();
            let provider = StubEmbeddingProvider::new();
            let fut = run_semantic_analyze(&components, &structural_edges, &provider, None);
            let semantic_report = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle.block_on(fut),
                Err(_) => {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| e.to_string())
                        .unwrap();
                    rt.block_on(fut)
                }
            };
            let semantic_report = match semantic_report {
                Ok(r) => r,
                Err(e) => {
                    return Some(ToolResponse::error(
                        "run_analyze",
                        format!("Semantic analysis failed: {}", e),
                    ))
                }
            };

            let intent_dir = intent_path
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| validated_path.join("docs").join("architecture"));
            let intent_report = {
                let mut intelligence = IntentIntelligence::new();
                let models = intelligence
                    .load_from_directory(&intent_dir)
                    .unwrap_or_default();
                if models.is_empty() {
                    None
                } else {
                    let mut merged = IntentModel::default();
                    for m in models {
                        merged.merge(m);
                    }
                    let detector = DriftDetector::new();
                    let drift_report = detector.detect(&merged, &scan_graph);
                    Some(sruja_intent::IntentReport::from_drift_report(&drift_report))
                }
            };

            let runtime_report: Option<RuntimeSection> = None;

            let mut scores: Vec<u8> = vec![
                structural_report.health_score,
                semantic_report.summary.health_score,
            ];
            if let Some(ref ir) = intent_report {
                scores.push(100u8.saturating_sub(ir.drift_score));
            }
            let overall_health = if scores.is_empty() {
                0u8
            } else {
                (scores.iter().copied().map(u32::from).sum::<u32>() / scores.len() as u32) as u8
            };

            let recommendations = build_recommendations(
                &structural_report,
                &semantic_report.coupling.recommendations,
                intent_report
                    .as_ref()
                    .map(|ir| ir.suggestions.as_slice())
                    .unwrap_or(&[]),
                10,
            );

            let report = ComprehensiveReport {
                structural: ReportStructuralSection {
                    modules: structural_report.total_modules,
                    services: structural_report.total_services,
                    databases: structural_report.total_databases,
                    dependencies: structural_report.total_dependencies,
                    health_score: structural_report.health_score,
                    violations_count: structural_report.violations.len(),
                },
                semantic: ReportSemanticSection {
                    component_count: semantic_report.summary.component_count,
                    context_count: semantic_report.summary.context_count,
                    hidden_coupling_count: semantic_report.summary.hidden_coupling_count,
                    vocabulary_leak_count: semantic_report.summary.vocabulary_leak_count,
                    health_score: semantic_report.summary.health_score,
                },
                intent: intent_report.as_ref().map(|ir| ReportIntentSection {
                    drift_score: ir.drift_score,
                    health: ir.health.clone(),
                    components_declared: ir.summary.components_declared,
                    components_discovered: ir.summary.components_discovered,
                    undocumented_count: ir.summary.undocumented_count,
                    missing_count: ir.summary.missing_count,
                }),
                runtime: runtime_report,
                overall_health,
                recommendations,
            };

            match serde_json::to_value(&report) {
                Ok(value) => Some(ToolResponse::success("run_analyze", value)),
                Err(e) => Some(ToolResponse::error("run_analyze", e.to_string())),
            }
        }

        SrujaTool::IntentCheck {
            repo_path,
            intent_path,
        } => match validate_path(repo_path) {
            Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                Ok(scan_graph) => {
                    let intent_dir = intent_path
                        .as_ref()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| validated_path.join("docs").join("architecture"));
                    let mut intelligence = IntentIntelligence::new();
                    let models = intelligence
                        .load_from_directory(&intent_dir)
                        .unwrap_or_default();
                    if models.is_empty() {
                        Some(ToolResponse::success(
                            "intent_check",
                            serde_json::json!({
                                "error": "No intent models found",
                                "intent_dir": format!("{}", intent_dir.display()),
                            }),
                        ))
                    } else {
                        let mut merged = IntentModel::default();
                        for m in models {
                            merged.merge(m);
                        }
                        let detector = DriftDetector::new();
                        let drift_report = detector.detect(&merged, &scan_graph);
                        let intent_report =
                            sruja_intent::IntentReport::from_drift_report(&drift_report);
                        match serde_json::to_value(&intent_report) {
                            Ok(value) => Some(ToolResponse::success("intent_check", value)),
                            Err(e) => Some(ToolResponse::error("intent_check", e.to_string())),
                        }
                    }
                }
                Err(e) => Some(ToolResponse::error("intent_check", e.to_string())),
            },
            Err(e) => Some(ToolResponse::error("intent_check", e)),
        },

        SrujaTool::DetectDriftWithBaseline {
            repo_path,
            architecture_path,
        } => {
            let validated_repo = match validate_path(repo_path) {
                Ok(p) => p,
                Err(e) => return Some(ToolResponse::error("detect_drift_with_baseline", e)),
            };
            let arch_path = match validate_path(architecture_path) {
                Ok(p) => p,
                Err(e) => return Some(ToolResponse::error("detect_drift_with_baseline", e)),
            };
            let actual_graph = match sruja_scan::scan_repo(&validated_repo) {
                Ok(g) => g,
                Err(e) => {
                    return Some(ToolResponse::error(
                        "detect_drift_with_baseline",
                        e.to_string(),
                    ))
                }
            };
            let content = match std::fs::read_to_string(&arch_path) {
                Ok(c) => c,
                Err(e) => {
                    return Some(ToolResponse::error(
                        "detect_drift_with_baseline",
                        e.to_string(),
                    ))
                }
            };
            let parser = Parser::new(arch_path.to_string_lossy().into_owned());
            let program = match parser.parse(&content) {
                Ok(p) => p,
                Err(diags) => {
                    let msg = diags
                        .iter()
                        .map(|d| d.message.as_str())
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Some(ToolResponse::error("detect_drift_with_baseline", msg));
                }
            };
            let proposed_graph = sruja_diff::program_to_graph(&program);
            let diff_result = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);
            match serde_json::to_value(&diff_result) {
                Ok(value) => Some(ToolResponse::success("detect_drift_with_baseline", value)),
                Err(e) => Some(ToolResponse::error(
                    "detect_drift_with_baseline",
                    e.to_string(),
                )),
            }
        }

        _ => None,
    }
}
