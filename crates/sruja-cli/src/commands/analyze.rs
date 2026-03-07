//! Analysis commands: complexity, semantic, comprehensive analyze.

use std::collections::HashMap;
use std::path::Path;

use sruja_scan::{scan_repo, Graph, NodeKind};
use sruja_semantic::{analyze as run_semantic_analyze, embedding::StubEmbeddingProvider};

use super::CliError;
use crate::config::SrujaConfig;
use crate::views::{print_view_report, ViewContext};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActionableRecommendation {
    pub title: String,
    pub category: RecommendationCategory,
    pub affected_components: Vec<ComponentReference>,
    pub impact_metrics: ImpactMetrics,
    pub effort_estimate: EffortEstimate,
    pub priority: Priority,
    pub rationale: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComponentReference {
    pub name: String,
    pub file_path: String,
    pub line_numbers: Option<Vec<usize>>,
    pub centrality_score: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpactMetrics {
    pub services_affected: usize,
    pub modules_impacted: usize,
    pub databases_consolidated: Option<usize>,
    pub coupling_reduction: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EffortEstimate {
    pub weeks: f64,
    pub team_size: usize,
    pub complexity: Complexity,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RecommendationCategory {
    DataConsolidation,
    Decoupling,
    Performance,
    Maintainability,
    Security,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Complexity {
    Low,
    Medium,
    High,
}

pub async fn complexity(
    repo_root: &str,
    format: &str,
    include_treewidth: bool,
    include_scc: bool,
    include_centrality: bool,
    include_coupling: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;

    let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let edges: Vec<(String, String)> = graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    let all = !include_treewidth && !include_scc && !include_centrality && !include_coupling;

    if format == "json" {
        let output = build_complexity_json(
            &nodes,
            &edges,
            all || include_treewidth,
            all || include_scc,
            all || include_centrality,
            all || include_coupling,
        );
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("{}", "═".repeat(70));
    println!("📊 Structural Complexity Analysis");
    println!("{}", "═".repeat(70));
    println!();

    if all || include_treewidth {
        print_treewidth_section(&nodes, &edges);
    }

    if all || include_scc {
        print_scc_section(&nodes, &edges);
    }

    if all || include_centrality {
        print_centrality_section(&nodes, &edges);
    }

    if all || include_coupling {
        print_coupling_section(&nodes, &edges);
    }

    println!("{}", "═".repeat(70));
    Ok(())
}

pub async fn semantic_analyze(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;
    let components: Vec<(String, String)> = graph
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
    let structural_edges: Vec<(String, String)> = graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    let provider = StubEmbeddingProvider::new();
    let report = run_semantic_analyze(&components, &structural_edges, &provider, None)
        .await
        .map_err(|e| CliError::Validation(format!("Semantic analysis failed: {}", e)))?;

    match format {
        "json" => {
            #[derive(serde::Serialize)]
            struct SemanticOutput {
                component_count: usize,
                context_count: usize,
                hidden_coupling_count: usize,
                vocabulary_leak_count: usize,
                health_score: u8,
                contexts: Vec<ContextOut>,
                hidden_couplings: Vec<HiddenCouplingOut>,
                recommendations: Vec<String>,
            }
            #[derive(serde::Serialize)]
            struct ContextOut {
                name: String,
                components: Vec<String>,
            }
            #[derive(serde::Serialize)]
            struct HiddenCouplingOut {
                source: String,
                target: String,
                similarity: f32,
                shared_concepts: Vec<String>,
            }
            let out = SemanticOutput {
                component_count: report.summary.component_count,
                context_count: report.summary.context_count,
                hidden_coupling_count: report.summary.hidden_coupling_count,
                vocabulary_leak_count: report.summary.vocabulary_leak_count,
                health_score: report.summary.health_score,
                contexts: report
                    .contexts
                    .iter()
                    .map(|c| ContextOut {
                        name: c.name.clone(),
                        components: c.components.clone(),
                    })
                    .collect(),
                hidden_couplings: report
                    .coupling
                    .hidden_couplings
                    .iter()
                    .take(20)
                    .map(|c| HiddenCouplingOut {
                        source: c.source.clone(),
                        target: c.target.clone(),
                        similarity: c.similarity,
                        shared_concepts: c.shared_concepts.clone(),
                    })
                    .collect(),
                recommendations: report.coupling.recommendations.clone(),
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            eprintln!("{}", "═".repeat(70));
            eprintln!("🔍 Sruja Semantic Analysis");
            eprintln!("{}", "═".repeat(70));
            eprintln!();
            eprintln!("📊 Summary");
            eprintln!("   Components: {}", report.summary.component_count);
            eprintln!("   Bounded contexts: {}", report.summary.context_count);
            eprintln!(
                "   Hidden couplings: {}",
                report.summary.hidden_coupling_count
            );
            eprintln!(
                "   Vocabulary leaks: {}",
                report.summary.vocabulary_leak_count
            );
            eprintln!("   Health score: {}/100", report.summary.health_score);
            eprintln!();
            if !report.contexts.is_empty() {
                eprintln!("📦 Bounded Contexts");
                for ctx in report.contexts.iter().take(5) {
                    eprintln!("   {}: {} components", ctx.name, ctx.components.len());
                }
                if report.contexts.len() > 5 {
                    eprintln!("   ... and {} more", report.contexts.len() - 5);
                }
                eprintln!();
            }
            if !report.coupling.recommendations.is_empty() {
                eprintln!("💡 Recommendations");
                for r in report.coupling.recommendations.iter().take(5) {
                    eprintln!("   - {}", r);
                }
            }
        }
    }

    Ok(())
}

pub async fn analyze(
    repo_root: &str,
    view_name: &str,
    _intent_path: Option<&str>,
    format: &str,
    enable_llm: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let mut config = SrujaConfig::load(repo_path)
        .map_err(|e| CliError::Validation(format!("Failed to load config: {}", e)))?;

    config.defaults.enable_llm = enable_llm || config.defaults.enable_llm;

    let graph = scan_repo(repo_path)?;

    let view_context = ViewContext::new(view_name, graph.clone(), repo_path, config)
        .map_err(CliError::Validation)?;

    let view_report = view_context.analyze().await.map_err(CliError::Validation)?;

    print_view_report(&view_report, format);

    Ok(())
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

fn print_scc_section(nodes: &[String], edges: &[(String, String)]) {
    println!("{}", "─".repeat(70));
    println!("🔄 SCC (Strongly Connected Components)");
    println!("{}", "─".repeat(70));

    let analyzer = sruja_graph::SccAnalyzer::new();
    let result = analyzer.analyze(nodes, edges);

    println!();
    println!("  Total SCCs: {}", result.total_sccs);
    println!("  Cyclic SCCs: {}", result.cyclic_sccs);
    println!("  Largest SCC size: {}", result.largest_scc_size);
    println!();

    if !result.components.is_empty() {
        let cyclic: Vec<_> = result.components.iter().filter(|c| c.is_cyclic).collect();
        if !cyclic.is_empty() {
            println!("  🔗 Cyclic Components:");
            for scc in cyclic.iter().take(5) {
                let nodes_str = scc
                    .nodes
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" → ");
                let suffix = if scc.nodes.len() > 3 {
                    format!(" +{} more", scc.nodes.len() - 3)
                } else {
                    String::new()
                };
                println!("    • [{}] {}{}", scc.id, nodes_str, suffix);
                if let Some(ref b) = scc.suggested_boundary {
                    println!("      → {}", b);
                }
            }
            if cyclic.len() > 5 {
                println!("    ... and {} more cyclic SCCs", cyclic.len() - 5);
            }
            println!();
        }
    }

    #[allow(dead_code)]
    pub fn generate_specific_recommendations(
        graph: &Graph,
        violations: &[sruja_diff::Violation],
    ) -> Vec<ActionableRecommendation> {
        let mut recommendations = Vec::new();

        if let Some(rec) = analyze_database_consolidation(graph) {
            recommendations.push(rec);
        }

        for violation in violations
            .iter()
            .filter(|v| v.kind == sruja_diff::ViolationKind::GodModule)
        {
            if let Some(rec) = create_decoupling_recommendation(graph, violation) {
                recommendations.push(rec);
            }
        }

        if let Some(rec) = analyze_coupling_hotspots(graph) {
            recommendations.push(rec);
        }

        recommendations
    }

    #[allow(dead_code)]
    fn analyze_database_consolidation(graph: &Graph) -> Option<ActionableRecommendation> {
        let db_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .collect();

        if db_nodes.len() < 5 {
            return None;
        }

        let mut by_tech: HashMap<String, Vec<&sruja_scan::Node>> = HashMap::new();
        for db in &db_nodes {
            let tech = db.technology.as_deref().unwrap_or("Unknown");
            by_tech.entry(tech.to_string()).or_default().push(db);
        }

        let mut consolidation_targets = Vec::new();
        for dbs in by_tech.values() {
            if dbs.len() > 3 {
                consolidation_targets.extend(dbs.iter().take(3));
            }
        }

        if consolidation_targets.is_empty() {
            return None;
        }

        Some(ActionableRecommendation {
        title: format!("Consolidate {} database instances", consolidation_targets.len()),
        category: RecommendationCategory::DataConsolidation,
        affected_components: consolidation_targets.iter().map(|db: &&sruja_scan::Node| ComponentReference {
            name: db.label.clone(),
            file_path: db.path.clone().unwrap_or_default(),
            line_numbers: None,
            centrality_score: 0.0,
        }).collect(),
        impact_metrics: ImpactMetrics {
            services_affected: count_dependent_services(graph, &consolidation_targets),
            modules_impacted: count_dependent_modules(graph, &consolidation_targets),
            databases_consolidated: Some(consolidation_targets.len()),
            coupling_reduction: Some(0.2),
        },
        effort_estimate: EffortEstimate {
            weeks: 2.5,
            team_size: 3,
            complexity: Complexity::Medium,
        },
        priority: Priority::Medium,
        rationale: format!(
            "Multiple {} instances suggest data silos. Consolidation will improve data consistency and reduce operational overhead.",
            by_tech.keys().next().unwrap_or(&"database".to_string())
        ),
    })
    }

    #[allow(dead_code)]
    fn create_decoupling_recommendation(
        graph: &Graph,
        violation: &sruja_diff::Violation,
    ) -> Option<ActionableRecommendation> {
        let location = violation.location.as_ref()?;
        let god_module = graph
            .nodes
            .iter()
            .find(|n| &n.id == location || &n.label == location)?;

        let dependents: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.target == god_module.id)
            .collect();

        Some(ActionableRecommendation {
            title: format!("Decouple god module: {}", god_module.label),
            category: RecommendationCategory::Decoupling,
            affected_components: vec![ComponentReference {
                name: god_module.label.clone(),
                file_path: god_module.path.clone().unwrap_or_default(),
                line_numbers: None,
                centrality_score: calculate_centrality(graph, &god_module.id),
            }],
            impact_metrics: ImpactMetrics {
                services_affected: count_services_in_path(&god_module.path),
                modules_impacted: dependents.len(),
                databases_consolidated: None,
                coupling_reduction: Some(0.3),
            },
            effort_estimate: EffortEstimate {
                weeks: 2.0,
                team_size: 2,
                complexity: Complexity::High,
            },
            priority: Priority::High,
            rationale: format!(
                "Module has {} dependencies (threshold: 10). Splitting reduces regression risk.",
                dependents.len()
            ),
        })
    }

    #[allow(dead_code)]
    fn analyze_coupling_hotspots(graph: &Graph) -> Option<ActionableRecommendation> {
        let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let edges: Vec<(String, String)> = graph
            .edges
            .iter()
            .map(|e| (e.source.clone(), e.target.clone()))
            .collect();

        let analyzer = sruja_graph::CentralityAnalyzer::new();
        let result = analyzer.analyze(&nodes, &edges);

        let high_centrality_count = result
            .top_hubs
            .iter()
            .filter(|h| h.degree_centrality > 0.1)
            .count();

        if high_centrality_count < 3 {
            return None;
        }

        let hotspots: Vec<_> = result
            .top_hubs
            .iter()
            .filter(|h| h.degree_centrality > 0.1)
            .take(5)
            .collect();

        Some(ActionableRecommendation {
        title: "Reduce coupling in high-centrality modules".to_string(),
        category: RecommendationCategory::Maintainability,
        affected_components: hotspots.iter().map(|h| ComponentReference {
            name: h.node.clone(),
            file_path: h.node.clone(),
            line_numbers: None,
            centrality_score: h.degree_centrality,
        }).collect(),
        impact_metrics: ImpactMetrics {
            services_affected: hotspots.len(),
            modules_impacted: hotspots.iter().map(|h| h.dependents).sum(),
            databases_consolidated: None,
            coupling_reduction: Some(0.25),
        },
        effort_estimate: EffortEstimate {
            weeks: 3.0,
            team_size: 3,
            complexity: Complexity::High,
        },
        priority: Priority::High,
        rationale: format!(
            "{} modules have high centrality (>0.1). Reducing coupling improves maintainability and testability.",
            high_centrality_count
        ),
    })
    }

    #[allow(dead_code)]
    fn count_dependent_services(graph: &Graph, targets: &[&sruja_scan::Node]) -> usize {
        let target_ids: Vec<_> = targets.iter().map(|t| t.id.clone()).collect();
        graph
            .edges
            .iter()
            .filter(|e| target_ids.contains(&e.target))
            .map(|e| e.source.clone())
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    #[allow(dead_code)]
    fn count_dependent_modules(graph: &Graph, targets: &[&sruja_scan::Node]) -> usize {
        let target_ids: Vec<_> = targets.iter().map(|t| t.id.clone()).collect();
        graph
            .edges
            .iter()
            .filter(|e| target_ids.contains(&e.target))
            .count()
    }

    #[allow(dead_code)]
    fn calculate_centrality(graph: &Graph, node_id: &str) -> f64 {
        let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let edges: Vec<(String, String)> = graph
            .edges
            .iter()
            .map(|e| (e.source.clone(), e.target.clone()))
            .collect();

        let analyzer = sruja_graph::CentralityAnalyzer::new();
        let result = analyzer.analyze(&nodes, &edges);

        result
            .top_hubs
            .iter()
            .find(|h| h.node == node_id)
            .map(|h| h.degree_centrality)
            .unwrap_or(0.0)
    }

    #[allow(dead_code)]
    fn count_services_in_path(path: &Option<String>) -> usize {
        path.as_ref()
            .map(|p| p.matches('/').count() + 1)
            .unwrap_or(1)
    }

    #[allow(dead_code)]
    fn format_recommendation_for_llm(rec: &ActionableRecommendation) -> String {
        let mut output = String::new();

        output.push_str(&format!("## {}\n\n", rec.title));

        output.push_str("📍 **Affected Components:**\n");
        for comp in &rec.affected_components {
            output.push_str(&format!("  • {}\n", comp.file_path));
            if comp.centrality_score > 0.05 {
                output.push_str(&format!(
                    "    Centrality: {:.2} (high)\n",
                    comp.centrality_score
                ));
            }
        }
        output.push('\n');

        output.push_str("📊 **Impact:**\n");
        output.push_str(&format!(
            "  • Services affected: {}\n",
            rec.impact_metrics.services_affected
        ));
        output.push_str(&format!(
            "  • Modules impacted: {}\n",
            rec.impact_metrics.modules_impacted
        ));
        if let Some(dbs) = rec.impact_metrics.databases_consolidated {
            output.push_str(&format!("  • Databases consolidated: {}\n", dbs));
        }
        if let Some(coupling) = rec.impact_metrics.coupling_reduction {
            output.push_str(&format!(
                "  • Coupling reduction: {:.0}%\n",
                coupling * 100.0
            ));
        }
        output.push('\n');

        output.push_str("⏱️ **Effort:**\n");
        output.push_str(&format!(
            "  {} weeks ({} developers, {} complexity)\n\n",
            rec.effort_estimate.weeks,
            rec.effort_estimate.team_size,
            format_complexity(&rec.effort_estimate.complexity)
        ));

        output.push_str(&format!(
            "🎯 **Priority:** {}\n\n",
            format_priority(&rec.priority)
        ));

        output.push_str(&format!("💡 **Rationale:** {}\n", rec.rationale));

        output
    }

    #[allow(dead_code)]
    fn format_complexity(complexity: &Complexity) -> &'static str {
        match complexity {
            Complexity::Low => "low",
            Complexity::Medium => "medium",
            Complexity::High => "high",
        }
    }

    #[allow(dead_code)]
    fn format_priority(priority: &Priority) -> &'static str {
        match priority {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

fn print_treewidth_section(nodes: &[String], edges: &[(String, String)]) {
    println!("{}", "─".repeat(70));
    println!("🌲 Treewidth Analysis");
    println!("{}", "─".repeat(70));

    let analyzer = sruja_graph::TreewidthAnalyzer::new();
    let result = analyzer.analyze(nodes, edges);

    println!();
    println!(
        "  Treewidth: {} ({})",
        result.treewidth, result.complexity_rating
    );
    println!();

    if !result.hotspots.is_empty() {
        println!("  🔥 Complexity Hotspots:");
        for hotspot in result.hotspots.iter().take(5) {
            let nodes_str = hotspot
                .nodes
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = if hotspot.nodes.len() > 3 {
                format!(" +{} more", hotspot.nodes.len() - 3)
            } else {
                String::new()
            };
            println!();
            println!("    • [tw={}] {}{}", hotspot.treewidth, nodes_str, suffix);
            println!("      → {}", hotspot.suggested_refactor.description);
        }
        println!();
    }
}

fn print_centrality_section(nodes: &[String], edges: &[(String, String)]) {
    println!("{}", "─".repeat(70));
    println!("🎯 Centrality Metrics");
    println!("{}", "─".repeat(70));

    let analyzer = sruja_graph::CentralityAnalyzer::new();
    let result = analyzer.analyze(nodes, edges);

    println!();

    if !result.top_hubs.is_empty() {
        println!("  📌 Top Hub Nodes (high degree):");
        for hub in result.top_hubs.iter().take(5) {
            println!(
                "    • {} (degree: {:.2}, dependents: {})",
                hub.node, hub.degree_centrality, hub.dependents
            );
        }
        println!();
    }

    if !result.top_bridges.is_empty() {
        println!("  🌉 Top Bridge Nodes (high betweenness):");
        for bridge in result.top_bridges.iter().take(5) {
            println!(
                "    • {} (betweenness: {:.3})",
                bridge.node, bridge.betweenness
            );
        }
        println!();
    }

    let hotspot_count = result
        .hotspots
        .iter()
        .filter(|h| h.combined_score > 0.5)
        .count();
    if hotspot_count > 0 {
        println!(
            "  ⚠️  {} high-combined-score hotspots detected",
            hotspot_count
        );
        println!();
    }
}

fn print_coupling_section(nodes: &[String], edges: &[(String, String)]) {
    println!("{}", "─".repeat(70));
    println!("🔗 Coupling Metrics");
    println!("{}", "─".repeat(70));

    let analyzer = sruja_graph::CouplingAnalyzer::new();
    let result = analyzer.analyze(nodes, edges);

    println!();
    println!(
        "  Average Instability:    {:.2}",
        result.summary.avg_instability
    );
    println!(
        "  Average Abstractness:   {:.2}",
        result.summary.avg_abstractness
    );
    println!(
        "  Avg Distance (ideal=0): {:.2}",
        result.summary.avg_distance
    );
    println!();

    if result.summary.pain_zone_count > 0 {
        println!(
            "  🚨 Zone of Pain: {} modules (concrete + stable)",
            result.summary.pain_zone_count
        );
    }
    if result.summary.uselessness_zone_count > 0 {
        println!(
            "  ⚠️  Zone of Uselessness: {} modules (abstract + unstable)",
            result.summary.uselessness_zone_count
        );
    }

    if !result.violations.is_empty() {
        println!();
        println!("  📋 Coupling Violations:");
        for violation in result.violations.iter().take(5) {
            println!();
            println!(
                "    • {} [{:?}]",
                violation.module, violation.violation_type
            );
            println!("      → {}", violation.suggestion);
        }
        println!();
    }
}
