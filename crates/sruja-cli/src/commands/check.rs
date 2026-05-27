//! Check command: CI-focused tool for non-blocking exit code 0 on violations.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use super::violation_shared::*;
use super::CliError;
use crate::utils::{architecture_path, colors};
use sruja_diff::Violation;
use sruja_scan::scan_repo;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckOutput {
    pub truth_status: String,
    pub baseline: Option<String>,
    pub violations_baseline: Option<String>,
    pub has_drift: bool,
    pub violations_count: usize,
    #[serde(default, skip_serializing_if = "is_usize_zero")]
    pub suppressed_count: usize,
    pub health_score: Option<u8>,
    pub violations: Vec<ViolationSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed_violations: Vec<ViolationSummary>,
    pub open_questions: Vec<String>,
    pub suggestions: Vec<String>,
}

fn is_usize_zero(v: &usize) -> bool {
    *v == 0
}

fn resolve_repo_relative(repo_root: &Path, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo_root.join(p)
    }
}

pub async fn baseline(repo_root: &str, output: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path).map_err(|e| CliError::scan(e.to_string()))?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);

    let filtered: Vec<Violation> = if let Some(ref baseline) = baseline_path {
        let content = fs::read_to_string(baseline)?;
        let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
        let program = parser
            .parse(&content)
            .map_err(|diags| CliError::parse_with_diagnostics(baseline.to_string_lossy(), diags))?;

        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);
        diff.violations
            .into_iter()
            .filter(is_production_relevant)
            .collect()
    } else {
        let drift = sruja_diff::detect_architectural_drift(&graph);
        drift
            .violations
            .into_iter()
            .filter(is_production_relevant)
            .collect()
    };

    let fingerprints: Vec<String> = filtered.iter().map(fingerprint_violation).collect();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let baseline = ViolationBaselineV2 {
        schema_version: 2,
        generated_at_unix: now,
        violations: fingerprints
            .into_iter()
            .map(|fp| ViolationBaselineEntry {
                fingerprint: fp,
                reason: "pre-existing violation (baseline)".to_string(),
                expires: None,
            })
            .collect(),
    };

    let out_path = resolve_repo_relative(repo_path, output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json =
        serde_json::to_string_pretty(&baseline).map_err(|e| CliError::validation(e.to_string()))?;
    fs::write(&out_path, json)?;
    println!("{}", out_path.to_string_lossy());

    Ok(())
}

pub async fn check(
    repo_root: &str,
    format: &str,
    violations_baseline: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path).map_err(|e| CliError::scan(e.to_string()))?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);

    let baseline_filter_set = if let Some(b) = violations_baseline {
        let p = resolve_repo_relative(repo_path, b);
        Some(load_violations_baseline(&p)?.fingerprints)
    } else {
        None
    };

    let (truth_status, filtered_violations, health_score) =
        if let Some(ref baseline) = baseline_path {
            let content = fs::read_to_string(baseline)?;
            let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
            let program = parser.parse(&content).map_err(|diags| {
                CliError::parse_with_diagnostics(baseline.to_string_lossy(), diags)
            })?;

            let proposed_graph = sruja_diff::program_to_graph(&program);
            let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);
            let truth_status = match diff.truth_status {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            };

            let filtered: Vec<_> = diff
                .violations
                .iter()
                .filter(|v| is_production_relevant(v))
                .cloned()
                .collect();
            let health_score = Some(diff.summary.health_score);

            (truth_status.to_string(), filtered, health_score)
        } else {
            let drift = sruja_diff::detect_architectural_drift(&graph);
            let truth_status = match drift.truth_status {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            };
            let filtered: Vec<_> = drift
                .violations
                .iter()
                .filter(|v| is_production_relevant(v))
                .cloned()
                .collect();
            let health_score = Some(drift.health_score);

            (truth_status.to_string(), filtered, health_score)
        };

    let filtered_violations: Vec<Violation> = filtered_violations
        .into_iter()
        .map(|mut v| {
            v.production_relevant = Some(true);
            if v.evidence_count.is_none() {
                v.evidence_count = Some(v.sources.len());
            }
            v
        })
        .collect();

    let (active_violations, suppressed_violations): (Vec<Violation>, Vec<Violation>) =
        if let Some(ref set) = baseline_filter_set {
            filtered_violations
                .into_iter()
                .map(|mut v| {
                    let suppressed = set.contains(&fingerprint_violation(&v));
                    v.suppressed = Some(suppressed);
                    v.baseline_delta =
                        Some(if suppressed { "baseline" } else { "new" }.to_string());
                    v
                })
                .partition(|v| v.suppressed != Some(true))
        } else {
            (filtered_violations, Vec::new())
        };

    let violations: Vec<ViolationSummary> =
        active_violations.iter().map(summarize_violation).collect();
    let suppressed_violations: Vec<ViolationSummary> = suppressed_violations
        .iter()
        .map(summarize_violation)
        .collect();
    let has_drift = !violations.is_empty();
    let open_questions = generate_open_questions(&active_violations);
    let suggestions = generate_suggestions(
        repo_root,
        baseline_path.as_deref(),
        &truth_status,
        &active_violations,
    );

    let output = CheckOutput {
        truth_status,
        baseline: baseline_path.and_then(|p| p.to_str().map(String::from)),
        violations_baseline: violations_baseline.map(|s| s.to_string()),
        has_drift,
        violations_count: violations.len(),
        suppressed_count: suppressed_violations.len(),
        health_score,
        violations,
        suppressed_violations,
        open_questions,
        suggestions,
    };

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::validation(e.to_string()))?
            );
        }
        "github-actions" => {
            let file = output.baseline.as_deref().unwrap_or(".sruja/context.json");
            let msg = format!(
                "Truth status: {}. Violations: {}.",
                output.truth_status, output.violations_count
            );
            println!(
                "::notice file={}::title=Sruja Check::{}",
                file,
                msg.replace('%', "%25")
                    .replace('\n', "%0A")
                    .replace('\r', "%0D")
            );

            for v_sum in &output.violations {
                // Find original violation from summarized one if needed, but we can just use v_sum
                let sev = match v_sum.severity.as_str() {
                    "error" => "error",
                    "warning" => "warning",
                    _ => "notice",
                };
                let title = format!("Sruja {}", v_sum.kind);
                let message = format!("{} ({})", v_sum.message, v_sum.fingerprint);

                let escaped_title = title
                    .replace('%', "%25")
                    .replace('\n', "%0A")
                    .replace('\r', "%0D");
                let escaped_message = message
                    .replace('%', "%25")
                    .replace('\n', "%0A")
                    .replace('\r', "%0D");

                // Use first source for annotation
                if let Some(src) = v_sum.sources.first() {
                    let src_file = src.file.as_deref().unwrap_or(file);
                    if let Some(line) = src.line {
                        println!(
                            "::{sev} file={src_file},line={line}::title={escaped_title}::{escaped_message}"
                        );
                    } else {
                        println!(
                            "::{sev} file={src_file}::title={escaped_title}::{escaped_message}"
                        );
                    }
                } else {
                    println!("::{sev} file={file}::title={escaped_title}::{escaped_message}");
                }
            }
        }
        _ => {
            if let Some(ref base) = output.baseline {
                println!("Baseline: {}", base);
            } else {
                println!("No baseline (repo.sruja not found)");
            }
            println!();
            println!(
                "Truth: {} ({} violation(s))",
                output.truth_status, output.violations_count
            );
            if let Some(score) = output.health_score {
                println!("Health:       {}", colors::health_bar(score, 15));
            }
            println!();

            if output.violations.is_empty() {
                println!("No violations found.");
            } else {
                println!("Violations found:");
                for v in &output.violations {
                    println!(
                        "  {} [{}:{}] {}",
                        colors::severity_icon(&v.severity),
                        v.severity,
                        v.kind,
                        v.message
                    );
                    let evidence: Vec<String> = v
                        .sources
                        .iter()
                        .filter(|s| s.file.is_some() || s.detail.is_some())
                        .map(|s| s.display_string())
                        .collect();
                    if !evidence.is_empty() {
                        println!("    Evidence: {}", evidence.join(", "));
                    }
                    println!("    Fingerprint: {}", v.fingerprint);
                }
                println!();
            }

            if !output.open_questions.is_empty() {
                println!("Open questions:");
                for q in &output.open_questions {
                    println!("  {} {}", colors::info("?"), q);
                }
                println!();
            }

            if !output.suggestions.is_empty() {
                println!("Suggestions:");
                for s in &output.suggestions {
                    println!("  {} {}", colors::success(">"), s);
                }
                println!();
            }

            let status_icon = if has_drift {
                colors::error("✗")
            } else {
                colors::success("✓")
            };
            let drift_text = if has_drift { "drifted" } else { "in sync" };

            println!("──────────────────────────────────────────────");
            println!(
                "{} {} │ {} elements │ health: {}/100",
                status_icon,
                drift_text,
                output.violations_count + output.suppressed_count,
                output.health_score.unwrap_or(0)
            );
        }
    }

    Ok(())
}
