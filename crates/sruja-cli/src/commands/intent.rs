//! Intent commands: check, propose, adr-index.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use sruja_intent::{
    AdrParser, AdrStatus, DriftDetector, DriftKind, IntentIntelligence, IntentModel, IntentReport,
    ParsedAdr, Severity,
};
use sruja_scan::scan_repo;

use super::CliError;

/// JSON-serializable ADR index entry for timeline/export.
#[derive(serde::Serialize)]
struct AdrIndexEntry {
    path: String,
    number: Option<u32>,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    consequences: Option<String>,
}

fn adr_status_string(s: &AdrStatus) -> String {
    match s {
        AdrStatus::Proposed => "Proposed".to_string(),
        AdrStatus::Accepted => "Accepted".to_string(),
        AdrStatus::Deprecated => "Deprecated".to_string(),
        AdrStatus::Superseded { by } => {
            if let Some(n) = by {
                format!("Superseded by {}", n)
            } else {
                "Superseded".to_string()
            }
        }
        AdrStatus::Rejected => "Rejected".to_string(),
        AdrStatus::Draft => "Draft".to_string(),
    }
}

fn parsed_adr_to_entry(adr: &ParsedAdr, full: bool) -> AdrIndexEntry {
    AdrIndexEntry {
        path: adr.path.display().to_string(),
        number: adr.number,
        title: adr.title.clone(),
        status: adr_status_string(&adr.status),
        date: adr.date.map(|d| d.to_rfc3339()),
        tags: adr.tags.clone(),
        context: if full {
            Some(adr.context.clone())
        } else {
            None
        },
        decision: if full {
            Some(adr.decision.clone())
        } else {
            None
        },
        consequences: if full {
            Some(adr.consequences.clone())
        } else {
            None
        },
    }
}

pub async fn intent_check(
    repo_root: &str,
    intent_path: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = std::path::Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path)?;

    let mut intelligence = IntentIntelligence::new();

    let intent_dir = if let Some(path) = intent_path {
        PathBuf::from(path)
    } else {
        repo_path.join("docs").join("architecture")
    };

    let models = intelligence
        .load_from_directory(&intent_dir)
        .unwrap_or_default();

    let mut merged_model = IntentModel::default();
    for model in models {
        merged_model.merge(model);
    }

    let detector = DriftDetector::new();
    let mut report = detector.detect(&merged_model, &graph);

    let policy_drifts = crate::compliance::evaluate_policy_violations(&merged_model, &graph);
    if !policy_drifts.is_empty() {
        report.drifts.extend(policy_drifts);
        report.recompute_summary_and_score();
    }

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

    let mut intelligence = IntentIntelligence::new();

    let intent_dir = if let Some(path) = intent_path {
        PathBuf::from(path)
    } else {
        repo_path.join("docs").join("architecture")
    };

    let models = intelligence
        .load_from_directory(&intent_dir)
        .unwrap_or_default();

    let mut merged_model = IntentModel::default();
    for model in models {
        merged_model.merge(model);
    }

    let detector = DriftDetector::new();
    let report = detector.detect(&merged_model, &graph);

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

/// Build list of intent dirs: if given, use them; else auto-detect docs/architecture, docs/adr, doc/adr.
fn resolve_intent_dirs(repo_path: &Path, intent_paths: &[String]) -> Vec<PathBuf> {
    if !intent_paths.is_empty() {
        return intent_paths.iter().map(PathBuf::from).collect();
    }
    let candidates = [
        repo_path.join("docs").join("architecture"),
        repo_path.join("docs").join("adr"),
        repo_path.join("doc").join("adr"),
    ];
    candidates.into_iter().filter(|p| p.exists()).collect()
}

/// Export ADR index JSON for timeline capture. Supports multiple intent dirs and auto-detect.
pub async fn adr_index(
    repo_root: &str,
    intent_paths: &[String],
    output_path: &str,
    full: bool,
    ref_name: Option<&str>,
    sha: Option<&str>,
    captured_at: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let dirs = resolve_intent_dirs(repo_path, intent_paths);
    let intent_dirs_tried: Vec<String> = dirs.iter().map(|p| p.display().to_string()).collect();

    let parser = AdrParser::new();
    let mut all_adrs: Vec<ParsedAdr> = Vec::new();
    for dir in &dirs {
        let adr_dir = dir.join("adr").join("decisions");
        if adr_dir.exists() {
            let adrs = parser.parse_dir(&adr_dir).unwrap_or_default();
            for adr in adrs {
                if !all_adrs.iter().any(|a| a.path == adr.path) {
                    all_adrs.push(adr);
                }
            }
        }
    }
    all_adrs.sort_by(|a, b| a.number.unwrap_or(0).cmp(&b.number.unwrap_or(0)));

    let adrs: Vec<AdrIndexEntry> = all_adrs
        .iter()
        .map(|a| parsed_adr_to_entry(a, full))
        .collect();

    let mut out = serde_json::json!({
        "intent_dirs_tried": intent_dirs_tried,
        "adrs": adrs
    });
    if let Some(r) = ref_name {
        out["ref"] = serde_json::Value::String(r.to_string());
    }
    if let Some(s) = sha {
        out["sha"] = serde_json::Value::String(s.to_string());
    }
    if let Some(c) = captured_at {
        out["captured_at"] = serde_json::Value::String(c.to_string());
    }

    std::fs::write(
        output_path,
        serde_json::to_string_pretty(&out).map_err(CliError::Json)?,
    )
    .map_err(CliError::Io)?;

    Ok(())
}
