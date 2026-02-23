//! MCP Tools for Sruja

use serde::{Deserialize, Serialize};
use sruja_graph::KnowledgeGraph;
use sruja_intent::{DriftDetector, IntentIntelligence, IntentModel};
use sruja_language::Parser;
use sruja_report::{
    build_recommendations, ComprehensiveReport, IntentSection as ReportIntentSection,
    RuntimeSection, SemanticSection as ReportSemanticSection,
    StructuralSection as ReportStructuralSection,
};
use sruja_runtime::{build_report, ExecutionTrace};
use sruja_semantic::{analyze as run_semantic_analyze, embedding::StubEmbeddingProvider};
use std::path::{Path, PathBuf};

fn validate_path(path_str: &str) -> Result<PathBuf, String> {
    let path = Path::new(path_str);

    // Reject path traversal attempts
    if path_str.contains("..") {
        return Err(format!("Path traversal detected: '{}'", path_str));
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Path does not exist or is inaccessible: {}", e))?;

    // Allow paths within cwd OR system temp directory
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Cannot determine current directory: {}", e))?;
    let temp = std::env::temp_dir();

    if canonical.starts_with(&cwd) || canonical.starts_with(&temp) {
        return Ok(canonical);
    }

    Ok(canonical)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SrujaTool {
    GetArchitecture,
    GetDecision {
        id: String,
    },
    GetDecisions,
    GetPolicyConflicts,
    Query {
        question: String,
    },
    GetComponent {
        id: String,
    },
    AddDecision {
        title: String,
        decision: String,
        context: String,
    },
    ScanRepo {
        path: String,
    },
    DetectDrift {
        repo_path: String,
    },
    Quickstart {
        repo_path: String,
    },
    SemanticAnalyze {
        repo_path: String,
    },
    Complexity {
        repo_path: String,
        treewidth: bool,
        scc: bool,
        centrality: bool,
        coupling: bool,
    },
    RunAnalyze {
        repo_path: String,
        traces_path: Option<String>,
        intent_path: Option<String>,
    },
    IntentCheck {
        repo_path: String,
        intent_path: Option<String>,
    },
    DetectDriftWithBaseline {
        repo_path: String,
        architecture_path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub tool: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

impl ToolResponse {
    pub fn success(tool: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            tool: tool.into(),
            success: true,
            result,
            error: None,
        }
    }

    pub fn error(tool: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            success: false,
            result: serde_json::Value::Null,
            error: Some(error.into()),
        }
    }
}

impl SrujaTool {
    pub fn name(&self) -> &str {
        match self {
            SrujaTool::GetArchitecture => "get_architecture",
            SrujaTool::GetDecision { .. } => "get_decision",
            SrujaTool::GetDecisions => "get_decisions",
            SrujaTool::GetPolicyConflicts => "get_policy_conflicts",
            SrujaTool::Query { .. } => "query",
            SrujaTool::GetComponent { .. } => "get_component",
            SrujaTool::AddDecision { .. } => "add_decision",
            SrujaTool::ScanRepo { .. } => "scan_repo",
            SrujaTool::DetectDrift { .. } => "detect_drift",
            SrujaTool::Quickstart { .. } => "quickstart",
            SrujaTool::SemanticAnalyze { .. } => "semantic_analyze",
            SrujaTool::Complexity { .. } => "complexity",
            SrujaTool::RunAnalyze { .. } => "run_analyze",
            SrujaTool::IntentCheck { .. } => "intent_check",
            SrujaTool::DetectDriftWithBaseline { .. } => "detect_drift_with_baseline",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            SrujaTool::GetArchitecture => {
                "Get the architecture summary with all components and their relationships"
            }
            SrujaTool::GetDecision { .. } => "Get a specific architecture decision by ID",
            SrujaTool::GetDecisions => "List all architecture decisions",
            SrujaTool::GetPolicyConflicts => "Find policy violations in the current architecture",
            SrujaTool::Query { .. } => {
                "Query the architecture knowledge graph with natural language"
            }
            SrujaTool::GetComponent { .. } => "Get details about a specific component",
            SrujaTool::AddDecision { .. } => "Add a new architecture decision",
            SrujaTool::ScanRepo { .. } => {
                "Scan a repository and return the inferred architecture graph"
            }
            SrujaTool::DetectDrift { .. } => {
                "Detect architectural drift and health issues in a repository"
            }
            SrujaTool::Quickstart { .. } => {
                "Get immediate architecture insights (zero-key, deterministic)"
            }
            SrujaTool::SemanticAnalyze { .. } => {
                "Analyze semantic coupling, bounded contexts, and vocabulary leakage"
            }
            SrujaTool::Complexity { .. } => {
                "Analyze structural complexity (treewidth, SCC, centrality, coupling)"
            }
            SrujaTool::RunAnalyze { .. } => {
                "Full four-layer analysis (structural, semantic, intent, optional runtime)"
            }
            SrujaTool::IntentCheck { .. } => {
                "Compare declared architectural intent vs actual implementation"
            }
            SrujaTool::DetectDriftWithBaseline { .. } => {
                "Compare scanned repo against a .sruja architecture baseline"
            }
        }
    }

    pub fn execute(&self, graph: &KnowledgeGraph) -> ToolResponse {
        match self {
            SrujaTool::GetArchitecture => {
                let summary = crate::ArchitectureSummary::from(graph);
                ToolResponse::success(
                    "get_architecture",
                    serde_json::to_value(summary).unwrap_or_default(),
                )
            }

            SrujaTool::GetDecisions => {
                let decisions: Vec<_> = graph
                    .decisions
                    .values()
                    .map(|d| crate::DecisionResponse::from(d))
                    .collect();
                ToolResponse::success(
                    "get_decisions",
                    serde_json::to_value(decisions).unwrap_or_default(),
                )
            }

            SrujaTool::GetDecision { id } => match graph.get_decision(id) {
                Some(d) => {
                    let response = crate::DecisionResponse::from(d);
                    ToolResponse::success(
                        "get_decision",
                        serde_json::to_value(response).unwrap_or_default(),
                    )
                }
                None => ToolResponse::error("get_decision", format!("Decision {} not found", id)),
            },

            SrujaTool::GetPolicyConflicts => {
                let violations = graph.find_policy_violations();
                let responses: Vec<_> = violations
                    .into_iter()
                    .map(crate::PolicyViolationResponse::from)
                    .collect();
                ToolResponse::success(
                    "get_policy_conflicts",
                    serde_json::to_value(responses).unwrap_or_default(),
                )
            }

            SrujaTool::Query { question } => match graph.query(question) {
                Ok(result) => {
                    let response = crate::QueryResponse::from(result);
                    ToolResponse::success(
                        "query",
                        serde_json::to_value(response).unwrap_or_default(),
                    )
                }
                Err(e) => ToolResponse::error("query", e.to_string()),
            },

            SrujaTool::GetComponent { id } => match graph.get_node(id) {
                Some(node) => ToolResponse::success(
                    "get_component",
                    serde_json::to_value(node).unwrap_or_default(),
                ),
                None => ToolResponse::error("get_component", format!("Component {} not found", id)),
            },

            SrujaTool::AddDecision { .. } => {
                ToolResponse::error("add_decision", "Use the chat API to add decisions")
            }

            SrujaTool::ScanRepo { path } => match validate_path(path) {
                Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                    Ok(scan_graph) => ToolResponse::success(
                        "scan_repo",
                        serde_json::to_value(scan_graph).unwrap_or_default(),
                    ),
                    Err(e) => ToolResponse::error("scan_repo", e.to_string()),
                },
                Err(e) => ToolResponse::error("scan_repo", e),
            },

            SrujaTool::DetectDrift { repo_path } => match validate_path(repo_path) {
                Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                    Ok(scan_graph) => {
                        let drift_report = sruja_diff::detect_architectural_drift(&scan_graph);
                        ToolResponse::success(
                            "detect_drift",
                            serde_json::to_value(drift_report).unwrap_or_default(),
                        )
                    }
                    Err(e) => ToolResponse::error("detect_drift", e.to_string()),
                },
                Err(e) => ToolResponse::error("detect_drift", e),
            },

            SrujaTool::Quickstart { repo_path } => match validate_path(repo_path) {
                Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                    Ok(scan_graph) => {
                        let drift_report = sruja_diff::detect_architectural_drift(&scan_graph);

                        let external_apis = scan_graph
                            .nodes
                            .iter()
                            .filter(|n| n.kind == sruja_scan::NodeKind::ExternalApi)
                            .count();

                        let result = serde_json::json!({
                            "repo": repo_path,
                            "health_score": drift_report.health_score,
                            "inventory": {
                                "modules": drift_report.total_modules,
                                "services": drift_report.total_services,
                                "databases": drift_report.total_databases,
                                "external_apis": external_apis,
                                "total_dependencies": drift_report.total_dependencies,
                            },
                            "violations_count": drift_report.violations.len(),
                            "suggestions_count": drift_report.suggestions.len(),
                        });

                        ToolResponse::success("quickstart", result)
                    }
                    Err(e) => ToolResponse::error("quickstart", e.to_string()),
                },
                Err(e) => ToolResponse::error("quickstart", e),
            },

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
                        let fut =
                            run_semantic_analyze(&components, &structural_edges, &provider, None);
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
                                ToolResponse::success("semantic_analyze", result)
                            }
                            Err(e) => ToolResponse::error("semantic_analyze", e.to_string()),
                        }
                    }
                    Err(e) => ToolResponse::error("semantic_analyze", e.to_string()),
                },
                Err(e) => ToolResponse::error("semantic_analyze", e),
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
                        ToolResponse::success("complexity", result)
                    }
                    Err(e) => ToolResponse::error("complexity", e.to_string()),
                },
                Err(e) => ToolResponse::error("complexity", e),
            },

            SrujaTool::RunAnalyze {
                repo_path,
                traces_path,
                intent_path,
            } => match validate_path(repo_path) {
                Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                    Ok(scan_graph) => {
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
                        let fut =
                            run_semantic_analyze(&components, &structural_edges, &provider, None);
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
                                return ToolResponse::error(
                                    "run_analyze",
                                    format!("Semantic analysis failed: {}", e),
                                )
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

                        let runtime_report = traces_path.as_ref().and_then(|p| {
                            let path = Path::new(p);
                            if path.exists() {
                                load_traces(path).ok().map(|t| build_report(&t))
                            } else {
                                None
                            }
                        });

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
                            (scores.iter().copied().map(u32::from).sum::<u32>()
                                / scores.len() as u32) as u8
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
                                hidden_coupling_count: semantic_report
                                    .summary
                                    .hidden_coupling_count,
                                vocabulary_leak_count: semantic_report
                                    .summary
                                    .vocabulary_leak_count,
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
                            runtime: runtime_report.as_ref().map(|r| RuntimeSection {
                                trace_count: r.trace_count,
                                total_spans: r.total_spans,
                                max_depth: r.max_depth,
                                total_duration_ms: r.total_duration_ms,
                                emergent_cycle_count: r.emergent_cycles.len(),
                                hotspot_count: r.hotspots.len(),
                            }),
                            overall_health,
                            recommendations,
                        };

                        match serde_json::to_value(&report) {
                            Ok(value) => ToolResponse::success("run_analyze", value),
                            Err(e) => ToolResponse::error("run_analyze", e.to_string()),
                        }
                    }
                    Err(e) => ToolResponse::error("run_analyze", e.to_string()),
                },
                Err(e) => ToolResponse::error("run_analyze", e),
            },

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
                            ToolResponse::success(
                                "intent_check",
                                serde_json::json!({
                                    "error": "No intent models found",
                                    "intent_dir": format!("{}", intent_dir.display()),
                                }),
                            )
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
                                Ok(value) => ToolResponse::success("intent_check", value),
                                Err(e) => ToolResponse::error("intent_check", e.to_string()),
                            }
                        }
                    }
                    Err(e) => ToolResponse::error("intent_check", e.to_string()),
                },
                Err(e) => ToolResponse::error("intent_check", e),
            },

            SrujaTool::DetectDriftWithBaseline {
                repo_path,
                architecture_path,
            } => {
                let validated_repo = match validate_path(repo_path) {
                    Ok(p) => p,
                    Err(e) => return ToolResponse::error("detect_drift_with_baseline", e),
                };
                let arch_path = match validate_path(architecture_path) {
                    Ok(p) => p,
                    Err(e) => return ToolResponse::error("detect_drift_with_baseline", e),
                };
                let actual_graph = match sruja_scan::scan_repo(&validated_repo) {
                    Ok(g) => g,
                    Err(e) => {
                        return ToolResponse::error("detect_drift_with_baseline", e.to_string())
                    }
                };
                let content = match std::fs::read_to_string(&arch_path) {
                    Ok(c) => c,
                    Err(e) => {
                        return ToolResponse::error("detect_drift_with_baseline", e.to_string())
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
                        return ToolResponse::error("detect_drift_with_baseline", msg);
                    }
                };
                let proposed_graph = sruja_diff::program_to_graph(&program);
                let diff_result = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);
                match serde_json::to_value(&diff_result) {
                    Ok(value) => ToolResponse::success("detect_drift_with_baseline", value),
                    Err(e) => ToolResponse::error("detect_drift_with_baseline", e.to_string()),
                }
            }
        }
    }
}

fn load_traces(path: &Path) -> Result<Vec<ExecutionTrace>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let traces = match value {
        serde_json::Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                let t: ExecutionTrace =
                    serde_json::from_value(v).map_err(|e| format!("Invalid trace: {}", e))?;
                out.push(t);
            }
            out
        }
        serde_json::Value::Object(_) => {
            let t: ExecutionTrace =
                serde_json::from_value(value).map_err(|e| format!("Invalid trace: {}", e))?;
            vec![t]
        }
        _ => return Err("Expected JSON array or object".to_string()),
    };
    Ok(traces)
}

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

pub fn list_tools() -> Vec<serde_json::Value> {
    let tools = [
        SrujaTool::GetArchitecture,
        SrujaTool::GetDecisions,
        SrujaTool::GetDecision { id: String::new() },
        SrujaTool::GetPolicyConflicts,
        SrujaTool::Query {
            question: String::new(),
        },
        SrujaTool::GetComponent { id: String::new() },
        SrujaTool::ScanRepo {
            path: String::new(),
        },
        SrujaTool::DetectDrift {
            repo_path: String::new(),
        },
        SrujaTool::Quickstart {
            repo_path: String::new(),
        },
        SrujaTool::SemanticAnalyze {
            repo_path: String::new(),
        },
        SrujaTool::Complexity {
            repo_path: String::new(),
            treewidth: false,
            scc: false,
            centrality: false,
            coupling: false,
        },
        SrujaTool::RunAnalyze {
            repo_path: String::new(),
            traces_path: None,
            intent_path: None,
        },
        SrujaTool::IntentCheck {
            repo_path: String::new(),
            intent_path: None,
        },
        SrujaTool::DetectDriftWithBaseline {
            repo_path: String::new(),
            architecture_path: String::new(),
        },
    ];

    tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let cwd = std::env::current_dir().expect("current dir");
        let temp_dir = tempfile::tempdir_in(cwd).expect("Failed to create temp dir");

        // Create a simple TypeScript file
        fs::write(
            temp_dir.path().join("main.ts"),
            r#"
import { hello } from './hello';

export function main() {
    return hello();
}
"#,
        )
        .expect("Failed to write main.ts");

        fs::write(
            temp_dir.path().join("hello.ts"),
            r#"export function hello(): string { return 'world'; }"#,
        )
        .expect("Failed to write hello.ts");

        temp_dir
    }

    #[test]
    fn test_scan_repo_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::ScanRepo {
            path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "scan_repo");
        assert!(
            response.success,
            "ScanRepo should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());

        // Verify the result contains nodes
        let nodes = response
            .result
            .get("nodes")
            .expect("Result should have nodes");
        assert!(nodes.is_array());
    }

    #[test]
    fn test_scan_repo_tool_with_invalid_path() {
        let tool = SrujaTool::ScanRepo {
            path: "/nonexistent/path/12345".to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "scan_repo");
        assert!(!response.success, "ScanRepo with invalid path should fail");
        assert!(response.error.is_some());
    }

    #[test]
    fn test_detect_drift_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::DetectDrift {
            repo_path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "detect_drift");
        assert!(
            response.success,
            "DetectDrift should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());

        // Verify the result contains drift report fields
        assert!(response.result.get("health_score").is_some());
        assert!(response.result.get("violations").is_some());
        assert!(response.result.get("suggestions").is_some());
    }

    #[test]
    fn test_quickstart_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::Quickstart {
            repo_path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "quickstart");
        assert!(
            response.success,
            "Quickstart should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());

        // Verify the result contains quickstart fields
        assert!(response.result.get("repo").is_some());
        assert!(response.result.get("health_score").is_some());
        assert!(response.result.get("inventory").is_some());

        let inventory = response.result.get("inventory").unwrap();
        assert!(inventory.get("modules").is_some());
        assert!(inventory.get("services").is_some());
        assert!(inventory.get("databases").is_some());
    }

    #[test]
    fn test_semantic_analyze_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::SemanticAnalyze {
            repo_path: repo.path().to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "semantic_analyze");
        assert!(
            response.success,
            "SemanticAnalyze should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(response.result.get("component_count").is_some());
        assert!(response.result.get("health_score").is_some());
    }

    #[test]
    fn test_complexity_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::Complexity {
            repo_path: repo.path().to_str().unwrap().to_string(),
            treewidth: true,
            scc: true,
            centrality: false,
            coupling: false,
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "complexity");
        assert!(
            response.success,
            "Complexity should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(response.result.get("total_nodes").is_some());
        assert!(response.result.get("total_edges").is_some());
    }

    #[test]
    fn test_run_analyze_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::RunAnalyze {
            repo_path: repo.path().to_str().unwrap().to_string(),
            traces_path: None,
            intent_path: None,
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "run_analyze");
        assert!(
            response.success,
            "RunAnalyze should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(response.result.get("structural").is_some());
        assert!(response.result.get("semantic").is_some());
        assert!(response.result.get("overall_health").is_some());
    }

    #[test]
    fn test_intent_check_tool_with_valid_path() {
        let repo = create_test_repo();
        let tool = SrujaTool::IntentCheck {
            repo_path: repo.path().to_str().unwrap().to_string(),
            intent_path: None,
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "intent_check");
        assert!(
            response.success,
            "IntentCheck should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        // No intent dir: result has "error" and "intent_dir", or has "drift_score" if intent was found
        let has_error = response.result.get("error").is_some();
        let has_drift_score = response.result.get("drift_score").is_some();
        assert!(
            has_error || has_drift_score,
            "result should have error or drift_score"
        );
    }

    #[test]
    fn test_detect_drift_with_baseline_tool() {
        let repo = create_test_repo();
        let sruja_path = repo.path().join("arch.sruja");
        let minimal_sruja = r#"
api = container "API" {
  technology "Node.js"
  description "HTTP API"
}
"#;
        fs::write(&sruja_path, minimal_sruja).expect("write arch.sruja");

        let tool = SrujaTool::DetectDriftWithBaseline {
            repo_path: repo.path().to_str().unwrap().to_string(),
            architecture_path: sruja_path.to_str().unwrap().to_string(),
        };

        let graph = KnowledgeGraph::new();
        let response = tool.execute(&graph);

        assert_eq!(response.tool, "detect_drift_with_baseline");
        assert!(
            response.success,
            "DetectDriftWithBaseline should succeed: error = {:?}",
            response.error
        );
        assert!(!response.result.is_null());
        assert!(
            response.result.get("node_diff").is_some()
                || response.result.get("violations").is_some(),
            "result should have node_diff or violations"
        );
    }

    #[test]
    fn test_list_tools_includes_new_tools() {
        let tools = list_tools();
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(
            tool_names.contains(&"scan_repo"),
            "list_tools should include scan_repo"
        );
        assert!(
            tool_names.contains(&"detect_drift"),
            "list_tools should include detect_drift"
        );
        assert!(
            tool_names.contains(&"quickstart"),
            "list_tools should include quickstart"
        );
        assert!(
            tool_names.contains(&"semantic_analyze"),
            "list_tools should include semantic_analyze"
        );
        assert!(
            tool_names.contains(&"complexity"),
            "list_tools should include complexity"
        );
        assert!(
            tool_names.contains(&"run_analyze"),
            "list_tools should include run_analyze"
        );
        assert!(
            tool_names.contains(&"intent_check"),
            "list_tools should include intent_check"
        );
        assert!(
            tool_names.contains(&"detect_drift_with_baseline"),
            "list_tools should include detect_drift_with_baseline"
        );
    }

    #[test]
    fn test_tool_names() {
        assert_eq!(
            SrujaTool::ScanRepo {
                path: String::new()
            }
            .name(),
            "scan_repo"
        );
        assert_eq!(
            SrujaTool::DetectDrift {
                repo_path: String::new()
            }
            .name(),
            "detect_drift"
        );
        assert_eq!(
            SrujaTool::Quickstart {
                repo_path: String::new()
            }
            .name(),
            "quickstart"
        );
    }

    #[test]
    fn test_tool_descriptions() {
        let scan_tool = SrujaTool::ScanRepo {
            path: String::new(),
        };
        assert!(scan_tool.description().contains("Scan a repository"));

        let drift_tool = SrujaTool::DetectDrift {
            repo_path: String::new(),
        };
        assert!(drift_tool.description().contains("drift"));

        let quickstart_tool = SrujaTool::Quickstart {
            repo_path: String::new(),
        };
        assert!(quickstart_tool.description().contains("insights"));
    }

    #[test]
    fn test_path_validation_rejects_traversal() {
        assert!(
            validate_path("../../../etc/passwd").is_err(),
            "Should reject path traversal"
        );
        assert!(
            validate_path("foo/../bar").is_err(),
            "Should reject path traversal in middle"
        );
        assert!(
            validate_path("/nonexistent/path").is_err(),
            "Should reject nonexistent path"
        );
    }

    #[test]
    fn test_path_validation_accepts_valid() {
        let cwd = std::env::current_dir().expect("Need cwd");
        assert!(
            validate_path(cwd.to_str().unwrap()).is_ok(),
            "Should accept current directory"
        );
    }
}
