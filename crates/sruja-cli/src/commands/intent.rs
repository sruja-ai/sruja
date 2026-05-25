//! Intent commands: check, propose, adr-index.

use std::path::PathBuf;

use sruja_intent::{DriftDetector, DriftKind, IntentContext, IntentModel, IntentReport, Severity};
use sruja_scan::scan_repo;

use super::CliError;

pub async fn intent_check(
    repo_root: &str,
    intent_path: Option<&str>,
    format: &str,
    strict: bool,
) -> Result<(), CliError> {
    let repo_path = std::path::Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;

    let mut context = IntentContext::new();

    let intent_dir = if let Some(path) = intent_path {
        PathBuf::from(path)
    } else {
        repo_path.join("docs").join("architecture")
    };

    let models = context.load_from_directory(&intent_dir).unwrap_or_default();

    let mut merged_model = IntentModel::default();
    for model in models {
        merged_model.merge(model);
    }

    let detector = DriftDetector::new();
    let mut report = detector.detect(&merged_model, &graph, context.schema());

    if strict {
        let graph_json = {
            let new_path = repo_path.join(crate::commands::SCAN_CACHE_PATH);
            if new_path.exists() {
                new_path
            } else {
                repo_path.join(".sruja/graph.json")
            }
        };
        if graph_json.exists() {
            let previous_graph: sruja_scan::Graph =
                serde_json::from_str(&std::fs::read_to_string(graph_json)?)?;
            // 2. Load proposals
            let proposals = sruja_diff::Proposal::load_all(repo_path).unwrap_or_default();
            // 3. Detect unproposed changes
            let unproposed =
                sruja_diff::detect_unproposed_changes(&previous_graph, &graph, &proposals);
            report.drifts.extend(unproposed);
            report.recompute_summary_and_score();
        }
    }

    let policy_drifts = crate::compliance::evaluate_policy_violations(&merged_model, &graph);
    if !policy_drifts.is_empty() {
        report.drifts.extend(policy_drifts);
        report.recompute_summary_and_score();
    }

    crate::commands::context_events::record_intent_check(repo_path, &report, strict);

    if format == "json" {
        let intent_report = IntentReport::from_drift_report(&report);
        println!("{}", serde_json::to_string_pretty(&intent_report)?);
        return Ok(());
    }

    if format == "markdown" {
        let intent_report = IntentReport::from_drift_report(&report);
        println!("{}", intent_report.to_markdown());
        return Ok(());
    }

    println!("{}", "═".repeat(70));
    println!("📋 Intent vs Reality Comparison");
    println!("{}", "═".repeat(70));
    println!();

    println!("Intent Source: {}", report.intent_source);
    println!("Reality Source: {}", report.reality_source);
    println!();

    println!("{}", "─".repeat(70));
    println!(
        "📊 Drift Score: {}/100 ({})",
        report.drift_score, report.health
    );
    println!("{}", "─".repeat(70));
    println!();
    println!(
        "  Components declared: {}",
        report.summary.total_components_declared
    );
    println!(
        "  Components discovered: {}",
        report.summary.total_components_discovered
    );
    println!("  Undocumented: {}", report.summary.undocumented_components);
    println!("  Missing: {}", report.summary.missing_components);
    println!(
        "  Undocumented relationships: {}",
        report.summary.undocumented_relationships
    );
    println!(
        "  Boundary violations: {}",
        report.summary.boundary_violations
    );
    println!("  Policy violations: {}", report.summary.policy_violations);
    println!();

    if !report.drifts.is_empty() {
        let critical: Vec<_> = report
            .drifts
            .iter()
            .filter(|d| matches!(d.severity, Severity::Critical))
            .collect();
        let high: Vec<_> = report
            .drifts
            .iter()
            .filter(|d| matches!(d.severity, Severity::High))
            .collect();
        let medium: Vec<_> = report
            .drifts
            .iter()
            .filter(|d| matches!(d.severity, Severity::Medium))
            .collect();

        if !critical.is_empty() {
            println!("🚨 Critical ({})", critical.len());
            println!("{}", "-".repeat(40));
            for d in &critical {
                println!("  ✗ {}", d.description);
                if let Some(ref suggestion) = d.suggestion {
                    println!("    → {}", suggestion);
                }
            }
            println!();
        }

        if !high.is_empty() {
            println!("⚠️  High ({})", high.len());
            println!("{}", "-".repeat(40));
            for d in &high {
                println!("  ⚠ {}", d.description);
                if let Some(ref suggestion) = d.suggestion {
                    println!("    → {}", suggestion);
                }
            }
            println!();
        }

        if !medium.is_empty() {
            println!("ℹ️  Medium ({})", medium.len());
            println!("{}", "-".repeat(40));
            for d in medium.iter().take(5) {
                println!("  • {}", d.description);
            }
            if medium.len() > 5 {
                println!("  ... and {} more", medium.len() - 5);
            }
            println!();
        }
    }

    println!("{}", "═".repeat(70));
    Ok(())
}

pub async fn intent_propose(repo_root: &str, intent_path: Option<&str>) -> Result<(), CliError> {
    let repo_path = std::path::Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;

    let mut context = IntentContext::new();

    let intent_dir = if let Some(path) = intent_path {
        PathBuf::from(path)
    } else {
        repo_path.join("docs").join("architecture")
    };

    let models = context.load_from_directory(&intent_dir).unwrap_or_default();

    let mut merged_model = IntentModel::default();
    for model in models {
        merged_model.merge(model);
    }

    let detector = DriftDetector::new();
    let report = detector.detect(&merged_model, &graph, context.schema());

    let undocumented: Vec<_> = report
        .drifts
        .iter()
        .filter(|d| matches!(d.kind, DriftKind::UndocumentedComponent))
        .collect();

    println!("# ADR-XXXX: Document Current Reality");
    println!();
    println!("## Status");
    println!("Proposed");
    println!();
    println!("## Context");
    println!();

    if undocumented.is_empty() {
        println!("No undocumented components detected.");
    } else {
        println!("The following components exist in the codebase but are not documented:");
        println!();
        for d in &undocumented {
            println!("- {}", d.description);
        }
    }

    if report.summary.boundary_violations > 0 {
        println!();
        println!(
            "Additionally, {} boundary violations were detected.",
            report.summary.boundary_violations
        );
    }

    println!();
    println!("## Decision");
    println!();
    println!("<!-- To be filled -->");
    println!();
    println!("## Consequences");
    println!();
    println!("<!-- To be filled -->");

    Ok(())
}
