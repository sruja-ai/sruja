//! Scan and drift commands: scan, why, drift, quickstart.

use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use sruja_graph::{merge_scan_into_graph, KnowledgeGraph};
use sruja_scan::{scan_repo, Graph, NodeKind};

use super::CliError;
use crate::context_detection::{
    build_repo_context, detect_architecture_style, detect_framework, detect_languages,
};

fn should_fail_on_violations(fail_on: Option<&str>, violations: &[sruja_diff::Violation]) -> bool {
    if let Some(criteria) = fail_on {
        let criteria_lower = criteria.to_lowercase();
        let criteria_list: Vec<&str> = criteria_lower.split(',').map(|s| s.trim()).collect();

        for criterion in criteria_list {
            match criterion {
                "all" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.severity, sruja_diff::Severity::Error))
                    {
                        return true;
                    }
                }
                "cycles" | "circular" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::CircularDependency))
                    {
                        return true;
                    }
                }
                "layer-violations" | "layer" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::LayerViolation))
                    {
                        return true;
                    }
                }
                "god-modules" | "god" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::GodModule))
                    {
                        return true;
                    }
                }
                "orphans" => {
                    if violations
                        .iter()
                        .any(|v| matches!(v.kind, sruja_diff::ViolationKind::OrphanComponent))
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn collect_file_evidence_from_scan(scan_graph: &Graph) -> Vec<String> {
    let mut files: std::collections::HashSet<String> = std::collections::HashSet::new();
    for edge in &scan_graph.edges {
        for ev in &edge.evidence {
            if let Some(ref f) = ev.file {
                files.insert(f.clone());
            }
        }
    }
    let mut v: Vec<_> = files.into_iter().collect();
    v.sort();
    v
}

pub async fn scan(repo_root: &str, output: &str) -> Result<(), CliError> {
    let graph =
        sruja_scan::scan_repo(Path::new(repo_root)).map_err(|e| CliError::Scan(e.to_string()))?;

    let json = serde_json::to_string_pretty(&graph)?;

    if output == "-" {
        println!("{}", json);
        return Ok(());
    }

    fs::write(output, json)?;
    println!("Wrote {}", output);
    Ok(())
}

pub async fn why(question: &str, repo: &str, graph_file: Option<&str>) -> Result<(), CliError> {
    let mut kg = KnowledgeGraph::new();
    let scan_graph: Graph = if let Some(path) = graph_file {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        scan_repo(Path::new(repo))?
    };

    let repo_path = graph_file.unwrap_or(repo);
    merge_scan_into_graph(&mut kg, &scan_graph, repo_path);

    match kg.query(question) {
        Ok(result) => {
            println!("{}\n", result.answer);
            println!("Confidence: {}%", (result.confidence * 100.0) as i32);
            if !result.evidence.is_empty() {
                println!("\nEvidence (from graph):");
                for ev in &result.evidence {
                    if ev.reference.is_empty() {
                        println!("  - {}", ev.excerpt);
                    } else {
                        println!("  - [{}] {}", ev.reference, ev.excerpt);
                    }
                }
            }
            let file_refs = collect_file_evidence_from_scan(&scan_graph);
            if !file_refs.is_empty() {
                println!("\nFile references (from scan):");
                for f in file_refs.iter().take(10) {
                    println!("  - {}", f);
                }
                if file_refs.len() > 10 {
                    println!("  ... and {} more", file_refs.len() - 10);
                }
            }
        }
        Err(e) => {
            return Err(CliError::Validation(format!("No answer found: {}", e)));
        }
    }

    Ok(())
}

pub async fn drift(
    repo_root: &str,
    architecture_path: Option<&str>,
    format: &str,
    _enrich: bool,
    violations_only: bool,
    fail_on: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let actual_graph = scan_repo(repo_path)?;

    if let Some(arch_path) = architecture_path {
        let arch_file = Path::new(arch_path);
        if !arch_file.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path),
            )));
        }
        let content = fs::read_to_string(arch_file)?;
        let parser = sruja_language::Parser::new(arch_path);
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: arch_path.to_string(),
            message: diags
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff_result = sruja_diff::compare_graphs(&actual_graph, &proposed_graph);

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&diff_result)?);
            }
            _ => {
                print_diff_text(&diff_result, violations_only);
            }
        }

        if should_fail_on_violations(fail_on, &diff_result.violations) {
            std::process::exit(1);
        }
    } else {
        let drift_result = sruja_diff::detect_architectural_drift(&actual_graph);

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&drift_result)?);
            }
            _ => {
                print_drift_text(&drift_result, violations_only);
            }
        }

        if should_fail_on_violations(fail_on, &drift_result.violations) {
            std::process::exit(1);
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuickstartResult {
    pub repo: String,
    /// Structural health only (cycles, layers, god modules, orphans).
    pub health_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_breakdown: Option<sruja_diff::HealthScoreBreakdown>,
    pub inventory: InventorySummary,
    pub top_findings: Vec<Finding>,
    pub actionable_fixes: Vec<ActionableFix>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InventorySummary {
    pub modules: usize,
    pub services: usize,
    pub databases: usize,
    pub external_apis: usize,
    pub total_dependencies: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionableFix {
    pub priority: String,
    pub description: String,
    pub impact: String,
    pub affected_components: Vec<String>,
}

impl QuickstartResult {
    fn from_drift_report(report: &sruja_diff::DriftReport, graph: &Graph, repo: &str) -> Self {
        let external_apis = graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();

        let mut all_violations: Vec<_> = report.violations.iter().collect();
        all_violations.sort_by(|a, b| {
            let severity_order = |s: &sruja_diff::Severity| match s {
                sruja_diff::Severity::Error => 0,
                sruja_diff::Severity::Warning => 1,
                sruja_diff::Severity::Info => 2,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        let top_findings: Vec<Finding> = all_violations
            .iter()
            .take(3)
            .map(|v| {
                let mut evidence: Vec<String> = v
                    .location
                    .as_ref()
                    .map(|s| vec![s.clone()])
                    .unwrap_or_default();
                for s in &v.sources {
                    evidence.push(sruja_diff::SourceRef::display_string(s));
                }
                Finding {
                    severity: match v.severity {
                        sruja_diff::Severity::Error => "error".to_string(),
                        sruja_diff::Severity::Warning => "warning".to_string(),
                        sruja_diff::Severity::Info => "info".to_string(),
                    },
                    kind: format!("{:?}", v.kind),
                    message: v.message.clone(),
                    evidence,
                }
            })
            .collect();

        let actionable_fixes = generate_actionable_fixes_from_violations(&report.violations);

        QuickstartResult {
            repo: repo.to_string(),
            health_score: report.health_score,
            health_breakdown: report.health_breakdown,
            inventory: InventorySummary {
                modules: report.total_modules,
                services: report.total_services,
                databases: report.total_databases,
                external_apis,
                total_dependencies: report.total_dependencies,
            },
            top_findings,
            actionable_fixes,
        }
    }
}

fn generate_actionable_fixes_from_violations(
    violations: &[sruja_diff::Violation],
) -> Vec<ActionableFix> {
    use sruja_diff::ViolationKind;
    let mut fixes = Vec::new();

    let circular: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::CircularDependency))
        .collect();
    if !circular.is_empty() {
        let affected: Vec<String> = circular.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "high".to_string(),
            description:
                "Break strong circular boundaries (Spaghetti Coupling) using Dependency Inversion or Event buses"
                    .to_string(),
            impact: "Cycles prevent modularity, cause cascading failures, and break independent testability/deployments.".to_string(),
            affected_components: affected,
        });
    }

    let layer: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::LayerViolation))
        .collect();
    if !layer.is_empty() {
        let affected: Vec<String> = layer.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "medium".to_string(),
            description: "Introduce proper service layers to abstract direct database access"
                .to_string(),
            impact: "Improves separation of concerns and testability".to_string(),
            affected_components: affected,
        });
    }

    let god: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::GodModule))
        .collect();
    if !god.is_empty() {
        let affected: Vec<String> = god.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "high".to_string(),
            description: "Decouple Bottlenecks (God Modules) to reduce fan-in/fan-out gravity".to_string(),
            impact: "High regression risk; modifying these components affects many distinct areas of the system, slowing down delivery.".to_string(),
            affected_components: affected,
        });
    }

    let orphans: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::OrphanComponent))
        .collect();
    if !orphans.is_empty() {
        let affected: Vec<String> = orphans.iter().filter_map(|v| v.location.clone()).collect();
        fixes.push(ActionableFix {
            priority: "low".to_string(),
            description: "Review orphan modules - integrate or remove unused code".to_string(),
            impact: "Reduces dead code and technical debt".to_string(),
            affected_components: affected,
        });
    }

    fixes.truncate(3);
    fixes
}

fn print_quickstart_summary(report: &sruja_diff::DriftReport, graph: &Graph, repo: &str) {
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "📊 Architecture Inventory".cyan().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("  Repository: {}", repo.green());
    println!();
    println!("  Components detected:");
    println!(
        "    • {} modules",
        report.total_modules.to_string().yellow()
    );
    println!(
        "    • {} services",
        report.total_services.to_string().yellow()
    );
    println!(
        "    • {} databases",
        report.total_databases.to_string().yellow()
    );
    let external_apis = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::ExternalApi)
        .count();
    println!("    • {} external APIs", external_apis.to_string().yellow());
    println!(
        "    • {} total dependencies",
        report.total_dependencies.to_string().yellow()
    );
    println!();

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    let score_str = format!("{}/100", report.health_score);
    let colored_score = match report.health_score {
        80..=100 => score_str.green().bold(),
        60..=79 => score_str.yellow().bold(),
        _ => score_str.red().bold(),
    };
    println!("💚 Architecture Health Score (structural only): {}", colored_score);
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));

    let score_bar = match report.health_score {
        80..=100 => "████████████████████ ✓ Good".green(),
        60..=79 => "██████████████░░░░░░ ⚠ Fair".yellow(),
        40..=59 => "██████████░░░░░░░░░░ ⚠ Needs Work".truecolor(255, 140, 0),
        _ => "████░░░░░░░░░░░░░░░░ ✗ Critical".red(),
    };
    println!("  {}", score_bar);
    println!();

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "🔍 Top 3 Critical Findings".red().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));

    let mut sorted: Vec<_> = report.violations.iter().collect();
    sorted.sort_by(|a, b| {
        let severity_order = |s: &sruja_diff::Severity| match s {
            sruja_diff::Severity::Error => 0,
            sruja_diff::Severity::Warning => 1,
            sruja_diff::Severity::Info => 2,
        };
        severity_order(&a.severity).cmp(&severity_order(&b.severity))
    });

    for (i, v) in sorted.iter().take(3).enumerate() {
        let (icon, msg) = match v.severity {
            sruja_diff::Severity::Error => ("🚨", v.message.red().bold()),
            sruja_diff::Severity::Warning => ("⚠️", v.message.yellow().bold()),
            sruja_diff::Severity::Info => ("ℹ️", v.message.cyan().bold()),
        };
        println!();
        println!("  {}. {} {}", i + 1, icon, msg);
        if let Some(ref loc) = v.location {
            // Find actual path or sanitize ID to look like a path
            let display_loc = graph
                .nodes
                .iter()
                .find(|n| &n.id == loc)
                .map(|n| n.path.as_deref().unwrap_or(loc))
                .unwrap_or(loc)
                .replace("_", "/");
            println!(
                "     📍 Component: {}",
                display_loc.truecolor(180, 180, 180)
            );
        }
        if let Some(ref s) = v.suggestion {
            println!("     💡 Suggestion: {}", s.italic());
        }
    }

    if sorted.is_empty() {
        println!();
        println!("  ✓ No critical issues found!");
    }
    println!();

    let fixes = generate_actionable_fixes_from_violations(&report.violations);

    if !fixes.is_empty() {
        println!("{}", "─".repeat(70).truecolor(100, 100, 100));
        println!("{}", "🎯 Top Actionable Fixes".green().bold());
        println!("{}", "─".repeat(70).truecolor(100, 100, 100));

        for (i, fix) in fixes.iter().enumerate() {
            let (priority_icon, priority_color) = match fix.priority.as_str() {
                "high" => ("🔴", fix.priority.to_uppercase().red()),
                "medium" => ("🟡", fix.priority.to_uppercase().yellow()),
                _ => ("🟢", fix.priority.to_uppercase().cyan()),
            };

            println!();
            println!(
                "  {}. {} [{}] {}",
                i + 1,
                priority_icon,
                priority_color,
                fix.description.bold()
            );
            println!("     Impact: {}", fix.impact.italic());
            if !fix.affected_components.is_empty() {
                let display_affected: Vec<_> = fix
                    .affected_components
                    .iter()
                    .map(|c| c.replace("_", "/"))
                    .collect();
                println!(
                    "     Affected: {}",
                    display_affected.join(", ").truecolor(180, 180, 180)
                );
            }
        }
        println!();
    }

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "🗺️  High-Level Domain Map".magenta().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));

    let mut domains: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for node in &graph.nodes {
        let path_str = node.path.as_deref().unwrap_or(&node.id);
        let normalized = path_str.replace(['\\', '_'], "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect();
        if parts.is_empty() {
            continue;
        }

        let mut domain_name = parts[0].to_string();
        if (parts[0] == "crates"
            || parts[0] == "packages"
            || parts[0] == "src"
            || parts[0] == "internal")
            && parts.len() > 1
        {
            domain_name = format!("{}/{}", parts[0], parts[1]);
        }
        *domains.entry(domain_name).or_insert(0) += 1;
    }

    let mut sorted_domains: Vec<_> = domains.into_iter().collect();
    sorted_domains.sort_by(|a, b| b.1.cmp(&a.1));

    if sorted_domains.is_empty() {
        println!("\n  No clear domains identified.");
    } else {
        println!();
        let max_items = 10;
        let total = sorted_domains.len();
        for (i, (domain, count)) in sorted_domains.iter().take(max_items).enumerate() {
            let is_last = i == max_items - 1 || i == sorted_domains.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            println!(
                "  {} 📂 {} ({} components)",
                prefix.truecolor(100, 100, 100),
                domain.cyan().bold(),
                count.to_string().yellow()
            );
        }
        if total > max_items {
            println!(
                "  {} ... and {} more",
                "└──".truecolor(100, 100, 100),
                (total - max_items).to_string().truecolor(100, 100, 100)
            );
        }
    }

    println!();

    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!("{}", "🚀 Next Steps".blue().bold());
    println!("{}", "─".repeat(70).truecolor(100, 100, 100));
    println!();
    println!(
        "  1. {}",
        "Review the findings above and prioritize fixes".white()
    );
    println!(
        "  2. {}",
        "Run 'sruja drift -r . --format json' for detailed analysis".white()
    );
    println!(
        "  3. {}",
        "Run 'sruja scan -r . -o architecture.json' to save the graph".white()
    );
    println!(
        "  4. {}",
        "Run 'sruja why \"your question\" -r .' to explore architecture decisions".white()
    );
    println!();
    println!("{}", "═".repeat(70).truecolor(100, 100, 100));
}

fn print_diff_text(result: &sruja_diff::DiffResult, violations_only: bool) {
    println!("{}", "═".repeat(60));
    println!("Baseline Drift: Scan vs DSL");
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        println!("📊 Summary");
        println!("{}", "-".repeat(40));
        let s = &result.summary;
        println!(
            "  Proposed: {} | Actual (scan): {}",
            s.proposed_components, s.existing_components
        );
        println!(
            "  New: {} | Missing: {} | Edges +{} -{}",
            s.new_components, s.missing_components, s.new_dependencies, s.removed_dependencies
        );
        println!("  Health Score: {}/100", s.health_score);
        println!();
    }

    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Warning))
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Info))
        .collect();

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            println!("  ✗ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            println!("  ⚠ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            println!("  ℹ {}", v.message);
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !result.suggestions.is_empty() {
        println!("💡 Suggestions");
        println!("{}", "-".repeat(40));
        for (i, s) in result.suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
        println!();
    }

    println!("{}", "═".repeat(60));
}

fn print_violation_sources(v: &sruja_diff::Violation) {
    if !v.sources.is_empty() {
        let refs: Vec<String> = v
            .sources
            .iter()
            .map(sruja_diff::SourceRef::display_string)
            .collect();
        println!("    📎 Sources: {}", refs.join(", "));
    }
}

fn print_drift_text(result: &sruja_diff::DriftReport, violations_only: bool) {
    println!("{}", "═".repeat(60));
    println!("Architecture Drift Detection");
    println!("{}", "═".repeat(60));
    println!();

    if !violations_only {
        println!("📊 Summary");
        println!("{}", "-".repeat(40));
        println!(
            "  Modules: {} | Services: {} | Databases: {}",
            result.total_modules, result.total_services, result.total_databases
        );
        println!("  Dependencies: {}", result.total_dependencies);
        println!("  Health Score (structural only): {}/100", result.health_score);
        println!();
    }

    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Error))
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Warning))
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| matches!(v.severity, sruja_diff::Severity::Info))
        .collect();

    if !errors.is_empty() {
        println!("🚨 Errors ({})", errors.len());
        println!("{}", "-".repeat(40));
        for v in &errors {
            println!("  ✗ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("⚠️  Warnings ({})", warnings.len());
        println!("{}", "-".repeat(40));
        for v in &warnings {
            println!("  ⚠ {}", v.message);
            if let Some(ref suggestion) = v.suggestion {
                println!("    → {}", suggestion);
            }
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !info.is_empty() {
        println!("ℹ️  Info ({})", info.len());
        println!("{}", "-".repeat(40));
        for v in &info {
            println!("  ℹ {}", v.message);
            print_violation_sources(v);
        }
        println!();
    }

    if !violations_only && !result.suggestions.is_empty() {
        println!("💡 Suggestions");
        println!("{}", "-".repeat(40));
        for (i, s) in result.suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
        println!();
    }

    println!("{}", "═".repeat(60));
}

pub async fn quickstart(
    repo_root: &str,
    format: &str,
    generate_baseline: bool,
    fail_on: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    eprintln!("{}", "═".repeat(70).truecolor(100, 100, 100));
    eprintln!(
        "{}",
        "🚀 Sruja Quickstart - Architecture Intelligence"
            .green()
            .bold()
    );
    eprintln!("{}", "═".repeat(70).truecolor(100, 100, 100));
    eprintln!();

    eprintln!("📂 Scanning repository...");
    let graph = scan_repo(repo_path)?;
    eprintln!("   ✓ Found {} components", graph.nodes.len());

    let languages = detect_languages(repo_path);
    let primary_language = languages
        .first()
        .map(|(l, _)| l.as_str())
        .unwrap_or("Unknown");
    let framework = detect_framework(repo_path, primary_language);
    let (is_monolith, is_microservices) = detect_architecture_style(&graph);
    let context = build_repo_context(repo_path, &graph);

    eprintln!();
    eprintln!("📊 Repository Context");
    eprintln!("   • Primary Language: {}", primary_language.cyan());
    if let Some(ref fw) = framework {
        eprintln!("   • Framework: {}", fw.cyan());
    }
    if is_microservices {
        eprintln!("   • Architecture: {}", "Microservices".cyan());
    } else if is_monolith {
        eprintln!("   • Architecture: {}", "Monolith".cyan());
    }
    if let Some(ref domain) = context.domain {
        eprintln!("   • Domain: {}", domain.cyan());
    }
    eprintln!();

    eprintln!("🔍 Analyzing architecture health...");
    let drift_report = sruja_diff::detect_architectural_drift(&graph);
    eprintln!("   ✓ Analysis complete");
    eprintln!();

    if generate_baseline {
        eprintln!("📝 Generating architecture baseline...");
        let baseline = generate_baseline_from_graph(&graph, repo_path).await;
        let baseline_path = repo_path.join("architecture.sruja");
        fs::write(&baseline_path, &baseline)?;
        eprintln!("   ✓ Baseline written to {}", baseline_path.display());
        eprintln!("   💡 Edit this file to match your intended architecture");
        eprintln!();
    }

    match format {
        "json" => {
            let output = QuickstartResult::from_drift_report(&drift_report, &graph, repo_root);
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            print_quickstart_summary(&drift_report, &graph, repo_root);
        }
    }

    if should_fail_on_violations(fail_on, &drift_report.violations) {
        std::process::exit(1);
    }

    Ok(())
}

pub async fn smart_coverage(
    repo_root: &str,
    format: &str,
    target_ratio: Option<f64>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;
    let ratio = target_ratio.unwrap_or(0.15);

    let total = graph.nodes.len();
    let target_count = ((total as f64) * ratio).ceil() as usize;

    let mut nodes_with_deps: Vec<_> = graph
        .nodes
        .iter()
        .map(|n| {
            let incoming = graph.edges.iter().filter(|e| e.target == n.id).count();
            let outgoing = graph.edges.iter().filter(|e| e.source == n.id).count();
            (n, incoming + outgoing)
        })
        .collect();
    nodes_with_deps.sort_by(|a, b| b.1.cmp(&a.1));

    let selected: Vec<_> = nodes_with_deps.into_iter().take(target_count).collect();

    match format {
        "json" => {
            let result = serde_json::json!({
                "total_components": total,
                "target_ratio": ratio,
                "selected_count": selected.len(),
                "components": selected.iter().map(|(n, deps)| serde_json::json!({
                    "id": n.id,
                    "label": n.label,
                    "path": n.path,
                    "dependencies": deps,
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!("{}", "═".repeat(60));
            println!("🎯 Smart Coverage Selection");
            println!("{}", "═".repeat(60));
            println!();
            println!("  Total components: {}", total);
            println!("  Target ratio: {:.0}%", ratio * 100.0);
            println!("  Selected: {} components", selected.len());
            println!();
            println!("Top selected components by architectural importance:");
            for (i, (n, deps)) in selected.iter().enumerate().take(20) {
                println!(
                    "  {}. {} ({} deps)",
                    i + 1,
                    n.label.cyan(),
                    deps.to_string().yellow()
                );
            }
            if selected.len() > 20 {
                println!("  ... and {} more", selected.len() - 20);
            }
            println!();
            println!("{}", "═".repeat(60));
        }
    }

    Ok(())
}

async fn generate_baseline_from_graph(graph: &Graph, repo_path: &Path) -> String {
    let mut domains: std::collections::HashMap<String, (usize, String, Vec<String>)> =
        std::collections::HashMap::new();
    let mut db_nodes = Vec::new();
    let mut api_nodes = Vec::new();

    // Canonicalize repo_path for consistent stripping (important for macOS /var vs /private/var)
    let repo_canon = repo_path
        .canonicalize()
        .unwrap_or_else(|_| repo_path.to_path_buf());

    for node in &graph.nodes {
        if node.kind == NodeKind::Database {
            db_nodes.push(node);
            continue;
        }
        if node.kind == NodeKind::ExternalApi {
            api_nodes.push(node);
            continue;
        }

        let mut path_str = node.path.clone().unwrap_or_else(|| node.id.clone());

        // Strip out the repo_root prefix if it exists to clean up domain grouping
        if let Ok(p) = Path::new(&path_str).canonicalize() {
            if let Ok(stripped) = p.strip_prefix(&repo_canon) {
                path_str = stripped.to_string_lossy().to_string();
            }
        } else if let Ok(stripped) = Path::new(&path_str).strip_prefix(&repo_canon) {
            path_str = stripped.to_string_lossy().to_string();
        } else if let Ok(stripped) = Path::new(&path_str).strip_prefix(repo_path) {
            path_str = stripped.to_string_lossy().to_string();
        }

        let normalized = path_str.replace(['\\', '_'], "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect();
        if parts.is_empty() {
            continue;
        }

        let src_idx = parts.iter().position(|&x| {
            x == "src" || x == "lib" || x == "internal" || x == "packages" || x == "crates"
        });

        let domain_name = if let Some(idx) = src_idx {
            if parts.len() >= idx + 3 {
                // E.g. ["axum", "src", "routing", "mod.rs"] -> idx=1, idx+3=4, len=4 -> groups as "axum/src/routing"
                parts[0..=idx + 1].join("/")
            } else {
                // E.g. ["axum", "src", "lib.rs"] -> groups as "axum/src"
                parts[0..=idx].join("/")
            }
        } else if parts.len() > 2 {
            parts[0..2].join("/")
        } else {
            parts[0].to_string()
        };

        let entry = domains
            .entry(domain_name)
            .or_insert((0, String::new(), Vec::new()));
        entry.0 += 1;
        if entry.1.is_empty() {
            if let Some(tech) = &node.technology {
                entry.1 = tech.clone();
            }
        }

        if entry.2.len() < 5 {
            let name = std::path::Path::new(&node.label)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| node.label.clone());
            if name != "mod" && name != "lib" && name != "main" && !entry.2.contains(&name) {
                entry.2.push(name);
            }
        }
    }

    // Deterministic baseline (domain aggregated)
    let mut dsl = String::new();
    dsl.push_str("// Auto-generated high-level architecture baseline from Sruja quickstart\n");
    dsl.push_str("// Edit this file to match your intended architecture\n\n");

    dsl.push_str("person = kind \"Person\"\n");
    dsl.push_str("system = kind \"System\"\n");
    dsl.push_str("container = kind \"Container\"\n");
    dsl.push_str("database = kind \"Database\"\n");
    dsl.push_str("external_api = kind \"External_Api\"\n\n");

    dsl.push_str("user = person \"User\" {\n");
    dsl.push_str("  description \"End user of the application\"\n");
    dsl.push_str("}\n\n");

    if !domains.is_empty() {
        dsl.push_str("app = system \"Application\" {\n");
        let mut sorted_domains: Vec<_> = domains.iter().collect();
        sorted_domains.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        for (domain, (count, tech, _)) in sorted_domains.iter().take(40) {
            let id = sanitize_id(domain);
            dsl.push_str(&format!("  {} = container \"{}\" {{\n", id, domain));
            if !tech.is_empty() {
                dsl.push_str(&format!("    technology \"{}\"\n", tech));
            }
            dsl.push_str(&format!(
                "    description \"Contains {} components\"\n",
                count
            ));
            dsl.push_str("  }\n");
        }
        dsl.push_str("}\n\n");
    }

    if !db_nodes.is_empty() {
        dsl.push_str("// Databases\n");
        for db in db_nodes.iter().take(5) {
            let id = sanitize_id(&db.id);
            let display_name = db.path.as_deref().unwrap_or(&db.label);
            dsl.push_str(&format!("{} = database \"{}\" {{\n", id, display_name));
            if let Some(ref tech) = db.technology {
                dsl.push_str(&format!("  technology \"{}\"\n", tech));
            }
            dsl.push_str("}\n");
        }
        dsl.push('\n');
    }

    if !api_nodes.is_empty() {
        dsl.push_str("// External APIs\n");
        for api in api_nodes.iter().take(5) {
            let id = sanitize_id(&api.id);
            let display_name = api.path.as_deref().unwrap_or(&api.label);
            dsl.push_str(&format!("{} = external_api \"{}\" {{\n", id, display_name));
            if let Some(ref tech) = api.technology {
                dsl.push_str(&format!("  technology \"{}\"\n", tech));
            }
            dsl.push_str("}\n");
        }
        dsl.push('\n');
    }

    dsl.push_str("// Run 'sruja lint architecture.sruja' to validate\n");
    dsl.push_str("// Run 'sruja drift -r . -a architecture.sruja' to compare code vs baseline\n");

    dsl
}

fn sanitize_id(s: &str) -> String {
    s.replace("-", "_")
        .replace(" ", "_")
        .replace(".", "_")
        .replace("/", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .trim_start_matches(|c: char| c.is_numeric())
        .to_string()
}

/// PR-scoped drift: compare base and head refs to find NEW violations
pub async fn drift_pr(
    repo_root: &str,
    base_ref: Option<&str>,
    head_ref: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let base = base_ref.unwrap_or("origin/main");
    let head = head_ref.unwrap_or("HEAD");

    eprintln!("🔍 PR-Scoped Drift Detection");
    eprintln!("   Base: {} | Head: {}", base, head);
    eprintln!();

    // Check if git is available
    let git_check = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_path)
        .output();

    if git_check.is_err() || !git_check.unwrap().status.success() {
        return Err(CliError::Validation(
            "Not a git repository. PR-scoped drift requires git.".to_string(),
        ));
    }

    // Get list of changed files between base and head
    let changed_files_output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{}...{}", base, head)])
        .current_dir(repo_path)
        .output()
        .map_err(|e| {
            CliError::Io(std::io::Error::other(format!(
                "Failed to get changed files: {}",
                e
            )))
        })?;

    let changed_files: Vec<String> = String::from_utf8_lossy(&changed_files_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

    if changed_files.is_empty() {
        eprintln!("✅ No changed files detected between {} in {}", base, head);
        return Ok(());
    }

    eprintln!("📝 Changed files: {}", changed_files.len());

    // Head graph: use cache by commit SHA if present (incremental / CI retry)
    let head_sha_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok();
    let head_sha = head_sha_output
        .and_then(|o| {
            o.status
                .success()
                .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default();
    let cache_dir = repo_path.join(".sruja").join("cache");
    let _ = fs::create_dir_all(&cache_dir);
    let head_cache_path = if !head_sha.is_empty() {
        cache_dir.join(format!("head_{}.json", head_sha))
    } else {
        PathBuf::new() // sentinel: no cache
    };
    let head_graph = if !head_cache_path.as_os_str().is_empty() && head_cache_path.exists() {
        eprintln!(
            "📂 Using cached head graph ({})",
            &head_sha[..head_sha.len().min(8)]
        );
        let content = fs::read_to_string(&head_cache_path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        let g = scan_repo(repo_path)?;
        if !head_cache_path.as_os_str().is_empty() {
            if let Ok(json) = serde_json::to_string_pretty(&g) {
                let _ = fs::write(&head_cache_path, json);
            }
        }
        g
    };
    let head_drift = sruja_diff::detect_architectural_drift(&head_graph);

    // Base graph: from cache, or checkout base in a temp worktree and scan (so base != head)
    let cache_filename = base.replace(['/', '.'], "_");
    let cache_path = cache_dir.join(format!("{}.json", cache_filename));
    let base_graph = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        // No cache: checkout base ref in a temp worktree and scan there
        let worktree_dir =
            std::env::temp_dir().join(format!("sruja-drift-base-{}", std::process::id()));
        if worktree_dir.exists() {
            let _ = fs::remove_dir_all(&worktree_dir);
        }
        let status = std::process::Command::new("git")
            .args([
                "worktree",
                "add",
                "--detach",
                worktree_dir.to_str().unwrap(),
                base,
            ])
            .current_dir(repo_path)
            .status()
            .map_err(|e| {
                CliError::Io(std::io::Error::other(format!(
                    "git worktree add failed (is '{}' a valid ref?): {}",
                    base, e
                )))
            })?;
        if !status.success() {
            return Err(CliError::Validation(format!(
                "Could not checkout base ref '{}'. Run a full scan on base and save to .sruja/cache/{}.json, or ensure the ref exists.",
                base, cache_filename
            )));
        }
        let base_graph = scan_repo(&worktree_dir).map_err(|e| {
            let _ = std::process::Command::new("git")
                .args([
                    "worktree",
                    "remove",
                    "--force",
                    worktree_dir.to_str().unwrap(),
                ])
                .current_dir(repo_path)
                .status();
            CliError::Scan(e.to_string())
        })?;
        let _ = std::process::Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_dir.to_str().unwrap(),
            ])
            .current_dir(repo_path)
            .status();
        base_graph
    };

    // Compare: what violations exist in head that don't exist in base
    let base_drift = sruja_diff::detect_architectural_drift(&base_graph);

    let new_violations: Vec<_> = head_drift
        .violations
        .iter()
        .filter(|hv| {
            !base_drift.violations.iter().any(|bv| {
                bv.kind == hv.kind && bv.message == hv.message && bv.location == hv.location
            })
        })
        .collect();

    let result = PrDriftResult {
        base_ref: base_ref.unwrap_or("origin/main").to_string(),
        head_ref: head_ref.unwrap_or("HEAD").to_string(),
        changed_files,
        base_health: base_drift.health_score,
        head_health: head_drift.health_score,
        new_violations: new_violations
            .iter()
            .map(|v| PrViolation {
                severity: format!("{:?}", v.severity),
                kind: format!("{:?}", v.kind),
                message: v.message.clone(),
                location: v.location.clone(),
                suggestion: v.suggestion.clone(),
            })
            .collect(),
        base_violations_count: base_drift.violations.len(),
        head_violations_count: head_drift.violations.len(),
    };

    match format {
        "json" => {
            let output = serde_json::to_string_pretty(&result)?;
            println!("{}", output);
        }
        "github-actions" => {
            print_github_actions_output(&result);
        }
        _ => {
            print_pr_drift_text(&result);
        }
    }

    // Exit with non-zero so CI fails when this PR introduces new violations
    if !result.new_violations.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
fn print_pr_drift_text(result: &PrDriftResult) {
    println!("{}", "═".repeat(70));
    println!("🔍 PR-Scoped Drift Detection");
    println!("{}", "═".repeat(70));
    println!();
    println!("Base: {} → Head: {}", result.base_ref, result.head_ref);
    println!("Changed files: {}", result.changed_files.len());
    println!();

    println!("📊 Health Score Change");
    println!("{}", "-".repeat(40));
    if result.head_health < result.base_health {
        println!(
            "  {} → {} (⚠️ -{})",
            result.base_health,
            result.head_health,
            result.base_health - result.head_health
        );
    } else if result.head_health > result.base_health {
        println!(
            "  {} → {} (✓ +{})",
            result.base_health,
            result.head_health,
            result.head_health - result.base_health
        );
    } else {
        println!(
            "  {} → {} (no change)",
            result.base_health, result.head_health
        );
    }
    println!();

    if result.new_violations.is_empty() {
        println!("{}", "-".repeat(40));
        println!("✅ No NEW architectural violations introduced in this PR!");
        println!("{}", "-".repeat(40));
        println!();
        println!(
            "Existing violations: {} (base) → {} (head)",
            result.base_violations_count, result.head_violations_count
        );
    } else {
        println!(
            "🚨 NEW Violations Introduced in This PR ({})",
            result.new_violations.len()
        );
        println!("{}", "-".repeat(40));

        for v in &result.new_violations {
            let icon = match v.severity.as_str() {
                "Error" => "❌",
                "Warning" => "⚠️",
                _ => "ℹ️",
            };
            println!();
            println!("  {} [{}] {}", icon, v.severity.to_uppercase(), v.message);
            if let Some(ref loc) = v.location {
                println!("     📍 {}", loc);
            }
            if let Some(ref s) = v.suggestion {
                println!("     💡 {}", s);
            }
        }

        if result.new_violations.len() > 3 {
            println!();
            println!("     ... and {} more", result.new_violations.len() - 3);
        }

        println!();
        println!(
            "⚠️  This PR introduces {} new violation(s). Consider fixing before merge.",
            result.new_violations.len()
        );
        println!();
    }

    println!("{}", "═".repeat(70));
}
fn print_github_actions_output(result: &PrDriftResult) {
    for v in &result.new_violations {
        let level = match v.severity.as_str() {
            "Error" => "error",
            "Warning" => "warning",
            _ => "notice",
        };
        if let Some(ref loc) = v.location {
            println!(
                "::{} file={},title=Sruja {}::{}",
                level, loc, v.kind, v.message
            );
        } else {
            println!("::{} title=Sruja {}::{}", level, v.kind, v.message);
        }
    }

    if result.new_violations.is_empty() {
        println!(
            "::notice title=Sruja::✅ No new architectural violations. Health: {} → {}",
            result.base_health, result.head_health
        );
    } else {
        println!(
            "::error title=Sruja::🚨 {} new violation(s) introduced. Health: {} → {}",
            result.new_violations.len(),
            result.base_health,
            result.head_health
        );
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PrDriftResult {
    base_ref: String,
    head_ref: String,
    changed_files: Vec<String>,
    base_health: u8,
    head_health: u8,
    new_violations: Vec<PrViolation>,
    base_violations_count: usize,
    head_violations_count: usize,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PrViolation {
    severity: String,
    kind: String,
    message: String,
    location: Option<String>,
    suggestion: Option<String>,
}
