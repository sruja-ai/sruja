//! Compliance command: single report combining structural drift, intent drift, and policy violations.

use std::path::{Path, PathBuf};

use sruja_diff::{compare_graphs, detect_architectural_drift, program_to_graph};
use sruja_intent::{
    compare::{DriftKind, DriftReport as IntentDriftReport},
    IntentIntelligence, IntentModel,
};
use sruja_language::Parser;
use sruja_report::{ComplianceReport, ComplianceStatus, DriftEntry, PolicyViolationEntry};
use sruja_scan::scan_repo;

use super::CliError;
use crate::compliance::evaluate_policy_violations;

fn resolve_intent_dir(repo_path: &Path, intent_opt: Option<&str>) -> PathBuf {
    if let Some(p) = intent_opt {
        return PathBuf::from(p);
    }
    let candidates = [
        repo_path.join("docs").join("architecture"),
        repo_path.join("docs").join("adr"),
        repo_path.join("doc").join("adr"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    repo_path.join("docs").join("architecture")
}

pub async fn compliance(
    repo_root: &str,
    architecture_path: Option<&str>,
    intent_path: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let scan_graph = scan_repo(repo_path)?;

    let (structural_violations, health_score) = if let Some(arch_path) = architecture_path {
        let arch_path = Path::new(arch_path);
        if !arch_path.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Architecture file not found: {}", arch_path.display()),
            )));
        }
        let content = std::fs::read_to_string(arch_path)?;
        let parser = Parser::new(arch_path.to_string_lossy().to_string());
        let program = parser.parse(&content).map_err(|e| CliError::Parse {
            file: arch_path.to_string_lossy().to_string(),
            message: format!("{:?}", e),
        })?;
        let proposed = program_to_graph(&program);
        let diff_result = compare_graphs(&scan_graph, &proposed);
        (diff_result.violations, diff_result.summary.health_score)
    } else {
        let drift_report = detect_architectural_drift(&scan_graph);
        (drift_report.violations, drift_report.health_score)
    };

    let intent_dir = resolve_intent_dir(repo_path, intent_path);
    let mut merged_model = IntentModel::default();
    if intent_dir.exists() {
        let mut intelligence = IntentIntelligence::new();
        if let Ok(models) = intelligence.load_from_directory(&intent_dir) {
            for model in models {
                merged_model.merge(model);
            }
        }
    }

    let detector = sruja_intent::DriftDetector::new();
    let mut intent_report: IntentDriftReport = detector.detect(&merged_model, &scan_graph);
    let policy_drifts = evaluate_policy_violations(&merged_model, &scan_graph);
    if !policy_drifts.is_empty() {
        intent_report.drifts.extend(policy_drifts);
        intent_report.recompute_summary_and_score();
    }

    let drift_entries: Vec<DriftEntry> = intent_report
        .drifts
        .iter()
        .map(|d| DriftEntry {
            kind: format!("{:?}", d.kind),
            severity: format!("{:?}", d.severity),
            description: d.description.clone(),
            suggestion: d.suggestion.clone(),
        })
        .collect();

    let policy_entries: Vec<PolicyViolationEntry> = intent_report
        .drifts
        .iter()
        .filter(|d| d.kind == DriftKind::PolicyViolation)
        .map(|d| {
            let (source, target) = d
                .evidence
                .first()
                .and_then(|e| {
                    e.location.as_ref().and_then(|loc| {
                        let parts: Vec<&str> = loc.split(" -> ").collect();
                        if parts.len() == 2 {
                            Some((parts[0].to_string(), parts[1].to_string()))
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or_else(|| ("?".to_string(), "?".to_string()));
            PolicyViolationEntry {
                policy_name: d.intent_ref.clone().unwrap_or_default(),
                message: d.description.clone(),
                source,
                target,
            }
        })
        .collect();

    let boundary_count = intent_report
        .drifts
        .iter()
        .filter(|d| d.kind == DriftKind::BoundaryViolation)
        .count() as u32;

    let report = ComplianceReport::from_parts(
        structural_violations,
        drift_entries,
        policy_entries,
        boundary_count,
        health_score,
    );

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_compliance_text(&report);
    }

    if report.status == ComplianceStatus::NonCompliant {
        std::process::exit(1);
    }
    Ok(())
}

fn print_compliance_text(report: &ComplianceReport) {
    use colored::Colorize;
    println!("{}", "═".repeat(70));
    println!("{}", "Compliance Report".bold());
    println!("{}", "═".repeat(70));
    println!();
    let status_str = match report.status {
        ComplianceStatus::Compliant => "Compliant".green().bold(),
        ComplianceStatus::NonCompliant => "Non-compliant".red().bold(),
    };
    println!("  Status: {}", status_str);
    println!("  Structural health score: {}", report.health_score);
    println!();
    println!(
        "  Structural violations: {}",
        report.structural_violations.len()
    );
    println!("  Intent/policy drifts: {}", report.drift_entries.len());
    println!("  Policy violations: {}", report.policy_violations.len());
    println!(
        "  Boundary violations: {}",
        report.boundary_violations_count
    );
    println!();
    if !report.remediation_checklist.is_empty() {
        println!("  {}:", "Remediation".yellow());
        for item in &report.remediation_checklist {
            println!("    • {}", item);
        }
    }
    println!("{}", "═".repeat(70));
}
