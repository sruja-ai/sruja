use colored::Colorize;
use std::path::Path;

use crate::commands::CliError;
use crate::context_detection::{
    build_repo_context, detect_architecture_style, detect_framework, detect_languages,
};
use crate::utils::architecture_path;
use sruja_scan::scan_repo;
use sruja_scan::scan_scope::resolve_scan_scope;

use super::drift::{should_fail_on_violations, truth_status_from_baseline_compare};
use super::output::{print_quickstart_summary, write_draft_baseline, QuickstartResult};

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

    eprintln!("{}", "─".repeat(70).truecolor(100, 100, 100));
    eprintln!(
        "{}",
        "🚀 Sruja Quickstart - Context Engineering".green().bold()
    );
    eprintln!("{}", "─".repeat(70).truecolor(100, 100, 100));
    eprintln!();

    eprintln!("📂 Scanning repository...");
    let graph = scan_repo(repo_path)?;
    eprintln!("   ✓ Found {} components", graph.nodes.len());
    let (_, scan_scope) = resolve_scan_scope(repo_path);

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

    // If a baseline exists, compute truth_status from scan vs DSL.
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let truth_status = if let Some(ref p) = baseline_path {
        match truth_status_from_baseline_compare(&graph, p) {
            Ok(s) => match s {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            }
            .to_string(),
            Err(_) => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    };

    if generate_baseline {
        match write_draft_baseline(repo_path, &graph, false)? {
            Some(p) => {
                eprintln!("📝 Wrote draft baseline: {}", p.to_string_lossy().cyan());
                eprintln!();
            }
            None => {
                eprintln!("📝 Baseline already exists (repo.sruja). Skipping write.");
                eprintln!();
            }
        }
    }

    match format {
        "json" => {
            let mut output =
                QuickstartResult::from_drift_report(&drift_report, &graph, repo_root, &scan_scope);
            output.truth_status = truth_status;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if baseline_path.is_some() {
                eprintln!("🧭 Truth (baseline present): {}", truth_status.cyan());
                eprintln!();
            }
            print_quickstart_summary(&drift_report, &graph, repo_root);
        }
    }

    if should_fail_on_violations(fail_on, &drift_report.violations) {
        return Err(CliError::FailOnViolations);
    }

    Ok(())
}

impl QuickstartResult {
    pub fn from_drift_report(
        report: &sruja_diff::DriftReport,
        graph: &sruja_scan::Graph,
        repo: &str,
        scan_scope: &sruja_scan::ScanScope,
    ) -> Self {
        use super::output::{generate_actionable_fixes_from_violations, Finding, InventorySummary};
        use sruja_scan::NodeKind;

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
            scan_scope: scan_scope.clone(),
            health_score: report.health_score,
            health_breakdown: None, // This was skipping in original scan.rs but available in report
            inventory: InventorySummary {
                modules: report.total_modules,
                services: report.total_services,
                databases: report.total_databases,
                external_apis,
                total_dependencies: report.total_dependencies,
            },
            top_findings,
            actionable_fixes,
            truth_status: "unknown".to_string(),
        }
    }
}
