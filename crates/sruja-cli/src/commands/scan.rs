//! Scan and drift commands: scan, why, drift, quickstart.

use std::fs;
use std::path::Path;

use sruja_graph::{merge_scan_into_graph, KnowledgeGraph};
use sruja_scan::{scan_repo, Graph, NodeKind};

use super::CliError;

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
    let graph = sruja_scan::scan_repo(Path::new(repo_root)).map_err(|e| CliError::Scan(e.to_string()))?;

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
            message: diags.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("; "),
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

        if diff_result
            .violations
            .iter()
            .any(|v| matches!(v.severity, sruja_diff::Severity::Error))
        {
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

        if drift_result
            .violations
            .iter()
            .any(|v| matches!(v.severity, sruja_diff::Severity::Error))
        {
            std::process::exit(1);
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuickstartResult {
    pub repo: String,
    pub health_score: u8,
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
                "Break circular dependencies by introducing interfaces or event-based communication"
                    .to_string(),
            impact: "Improves maintainability and reduces coupling".to_string(),
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
            priority: "medium".to_string(),
            description: "Refactor god modules into smaller, focused components".to_string(),
            impact: "Improves code maintainability and reduces cognitive load".to_string(),
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
    println!("{}", "─".repeat(70));
    println!("📊 Architecture Inventory");
    println!("{}", "─".repeat(70));
    println!("  Repository: {}", repo);
    println!();
    println!("  Components detected:");
    println!("    • {} modules", report.total_modules);
    println!("    • {} services", report.total_services);
    println!("    • {} databases", report.total_databases);
    let external_apis = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::ExternalApi)
        .count();
    println!("    • {} external APIs", external_apis);
    println!("    • {} total dependencies", report.total_dependencies);
    println!();

    println!("{}", "─".repeat(70));
    println!("💚 Architecture Health Score: {}/100", report.health_score);
    println!("{}", "─".repeat(70));

    let score_bar = match report.health_score {
        80..=100 => "████████████████████ ✓ Good",
        60..=79 => "██████████████░░░░░░ ⚠ Fair",
        40..=59 => "██████████░░░░░░░░░░ ⚠ Needs Work",
        _ => "████░░░░░░░░░░░░░░░░ ✗ Critical",
    };
    println!("  {}", score_bar);
    println!();

    println!("{}", "─".repeat(70));
    println!("🔍 Top 3 Critical Findings");
    println!("{}", "─".repeat(70));

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
        let icon = match v.severity {
            sruja_diff::Severity::Error => "🚨",
            sruja_diff::Severity::Warning => "⚠️",
            sruja_diff::Severity::Info => "ℹ️",
        };
        println!();
        println!("  {}. {} {}", i + 1, icon, v.message);
        if let Some(ref loc) = v.location {
            println!("     📍 Component: {}", loc);
        }
        if let Some(ref s) = v.suggestion {
            println!("     💡 Suggestion: {}", s);
        }
    }

    if sorted.is_empty() {
        println!();
        println!("  ✓ No critical issues found!");
    }
    println!();

    let fixes = generate_actionable_fixes_from_violations(&report.violations);

    if !fixes.is_empty() {
        println!("{}", "─".repeat(70));
        println!("🎯 Top 3 Actionable Fixes");
        println!("{}", "─".repeat(70));

        for (i, fix) in fixes.iter().enumerate() {
            let priority_icon = match fix.priority.as_str() {
                "high" => "🔴",
                "medium" => "🟡",
                _ => "🟢",
            };

            println!();
            println!(
                "  {}. {} [{}] {}",
                i + 1,
                priority_icon,
                fix.priority.to_uppercase(),
                fix.description
            );
            println!("     Impact: {}", fix.impact);
            if !fix.affected_components.is_empty() {
                println!("     Affected: {}", fix.affected_components.join(", "));
            }
        }
        println!();
    }

    println!("{}", "─".repeat(70));
    println!("📎 Evidence References");
    println!("{}", "─".repeat(70));

    let sample_nodes: Vec<_> = graph.nodes.iter().take(5).collect();
    if !sample_nodes.is_empty() {
        println!();
        println!("  Sample components detected:");
        for node in &sample_nodes {
            println!(
                "    • {} ({:?}) - {}",
                node.id,
                node.kind,
                node.path.as_deref().unwrap_or("unknown")
            );
        }
    }
    println!();

    println!("{}", "─".repeat(70));
    println!("🚀 Next Steps");
    println!("{}", "─".repeat(70));
    println!();
    println!("  1. Review the findings above and prioritize fixes");
    println!("  2. Run 'sruja drift -r . --format json' for detailed analysis");
    println!("  3. Run 'sruja scan -r . -o architecture.json' to save the graph");
    println!("  4. Run 'sruja why \"your question\" -r .' to explore architecture decisions");
    println!();
    println!("{}", "═".repeat(70));
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
        println!("  Health Score: {}/100", result.health_score);
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
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);

    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    eprintln!("{}", "═".repeat(70));
    eprintln!("🚀 Sruja Quickstart - Architecture Intelligence");
    eprintln!("{}", "═".repeat(70));
    eprintln!();

    eprintln!("📂 Scanning repository...");
    let graph = scan_repo(repo_path)?;
    eprintln!("   ✓ Found {} components", graph.nodes.len());
    eprintln!();

    eprintln!("🔍 Analyzing architecture health...");
    let drift_report = sruja_diff::detect_architectural_drift(&graph);
    eprintln!("   ✓ Analysis complete");
    eprintln!();

    if generate_baseline {
        eprintln!("📝 Generating architecture baseline...");
        let baseline = generate_baseline_from_graph(&graph);
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

    Ok(())
}

fn generate_baseline_from_graph(graph: &Graph) -> String {
    let mut dsl = String::new();
    dsl.push_str("// Auto-generated architecture baseline from Sruja quickstart\n");
    dsl.push_str("// Edit this file to match your intended architecture\n\n");
    
    let services: Vec<_> = graph.nodes.iter().filter(|n| n.kind == NodeKind::Service).collect();
    let databases: Vec<_> = graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).collect();
    let modules: Vec<_> = graph.nodes.iter().filter(|n| n.kind == NodeKind::Module).collect();
    
    dsl.push_str("// Component kinds\n");
    dsl.push_str("person = kind \"Person\"\n");
    dsl.push_str("system = kind \"System\"\n");
    dsl.push_str("container = kind \"Container\"\n");
    dsl.push_str("database = kind \"Database\"\n\n");
    
    dsl.push_str("// External actors\n");
    dsl.push_str("user = person \"User\" {\n");
    dsl.push_str("  description \"End user of the application\"\n");
    dsl.push_str("}\n\n");
    
    if !services.is_empty() || !modules.is_empty() {
        dsl.push_str("// System\n");
        dsl.push_str("app = system \"Application\" {\n");
        
        for service in &services {
            let name = sanitize_id(&service.label);
            dsl.push_str(&format!("  {} = container \"{}\" {{\n", name, service.label));
            if let Some(ref tech) = service.technology {
                dsl.push_str(&format!("    technology \"{}\"\n", tech));
            }
            dsl.push_str("  }\n");
        }
        
        dsl.push_str("}\n\n");
    }
    
    if !databases.is_empty() {
        dsl.push_str("// Databases\n");
        for db in &databases {
            let name = sanitize_id(&db.label);
            dsl.push_str(&format!("{} = database \"{}\" {{\n", name, db.label));
            if let Some(ref tech) = db.technology {
                dsl.push_str(&format!("  technology \"{}\"\n", tech));
            }
            dsl.push_str("}\n");
        }
        dsl.push_str("\n");
    }
    
    if !graph.edges.is_empty() {
        dsl.push_str("// Key relationships (sample)\n");
        for edge in graph.edges.iter().take(10) {
            let source = sanitize_id(&edge.source);
            let target = sanitize_id(&edge.target);
            dsl.push_str(&format!("{} -> {} \"uses\"\n", source, target));
        }
        dsl.push_str("\n");
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
