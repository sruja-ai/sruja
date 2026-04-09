//! Review command: refresh evidence, detect drift, propose updates or open questions.

use std::fs;
use std::path::Path;
use std::time::Instant;

use super::{scan_repo_cached, CliError};
use super::violation_shared::*;
use crate::utils::{architecture_path, colors};
use sruja_diff::Violation;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReviewOutput {
    pub truth_status: String,
    pub baseline: Option<String>,
    pub has_drift: bool,
    pub violations_count: usize,
    pub health_score: Option<u8>,
    #[serde(default, skip_serializing_if = "is_usize_zero")]
    pub suppressed_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<ViolationSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed_violations: Vec<ViolationSummary>,
    pub new_components: Vec<String>,
    pub missing_components: Vec<String>,
    pub drifted_dependencies: Vec<String>,
    pub open_questions: Vec<String>,
    pub suggestions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
}

fn is_usize_zero(v: &usize) -> bool {
    *v == 0
}

pub async fn review(repo_root: &str, format: &str) -> Result<(), CliError> {
    let start_time = Instant::now();
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let has_baseline = baseline_path.is_some();

    // Review is the day-to-day workflow, so refresh cached evidence first.
    super::sync_cmd::sync(repo_root, "quiet").await?;
    let graph = scan_repo_cached(repo_path)?;

    let (truth_status, violations, health_score) = if let Some(ref baseline) = baseline_path {
        let content = fs::read_to_string(baseline)?;
        let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(baseline.to_string_lossy().to_string(), diags)
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);

        let truth = match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };

        (
            truth.to_string(),
            diff.violations,
            Some(diff.summary.health_score),
        )
    } else {
        let drift = sruja_diff::detect_architectural_drift(&graph);
        let truth = match drift.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        };

        (
            truth.to_string(),
            drift.violations,
            Some(drift.health_score),
        )
    };

    let mut filtered_violations: Vec<_> = violations
        .into_iter()
        .filter(|v| is_production_relevant(v))
        .collect();

    // Mark production_relevant/evidence_count and split by baseline suppression if baseline fingerprints exist.
    for v in &mut filtered_violations {
        v.production_relevant = Some(true);
        if v.evidence_count.is_none() {
            v.evidence_count = Some(v.sources.len());
        }
    }

    let violations_baseline_path = repo_path.join(".sruja").join("violations.baseline.json");
    let baseline_set: Option<std::collections::HashSet<String>> =
        if violations_baseline_path.exists() {
            let content = fs::read_to_string(&violations_baseline_path)?;
            let baseline: super::check::ViolationBaseline =
                serde_json::from_str(&content).map_err(|e| CliError::validation(e.to_string()))?;
            Some(baseline.fingerprints.into_iter().collect())
        } else {
            None
        };

    let (active_violations, suppressed_violations): (Vec<Violation>, Vec<Violation>) =
        if let Some(ref set) = baseline_set {
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

    let has_drift = truth_status == "drifted" || (!has_baseline && !active_violations.is_empty());
    let (new_components, missing_components, drifted_dependencies) =
        categorize_violations(&active_violations);
    let open_questions = generate_open_questions(&active_violations);
    let suggestions = generate_suggestions(
        repo_root,
        baseline_path.as_deref(),
        &truth_status,
        &active_violations,
    );

    let elapsed = start_time.elapsed();
    let output = ReviewOutput {
        truth_status: truth_status.clone(),
        baseline: baseline_path.and_then(|p| p.to_str().map(String::from)),
        has_drift,
        violations_count: active_violations.len(),
        health_score,
        suppressed_count: suppressed_violations.len(),
        violations: active_violations.iter().map(summarize_violation).collect(),
        suppressed_violations: suppressed_violations.iter().map(summarize_violation).collect(),
        new_components,
        missing_components,
        drifted_dependencies,
        open_questions,
        suggestions,
        elapsed_ms: Some(elapsed.as_millis()),
    };

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::validation(e.to_string()))?
            );
        }
        _ => {
            colors::print_header("📅 Sruja Daily Review");
            if let Some(ref base) = output.baseline {
                println!("  {} {}", colors::dim("Baseline:"), base);
            } else {
                println!("  {} {}", colors::dim("Baseline:"), colors::warning("none (repo.sruja not found)"));
            }
            
            let time_str = colors::elapsed_display(elapsed);
            println!("  {} {}", colors::dim("Elapsed:"), colors::info(time_str));
            println!();

            let status_color = match output.truth_status.as_str() {
                "reviewed" => colors::success(&output.truth_status),
                "drifted" => colors::error(&output.truth_status),
                _ => colors::warning(&output.truth_status),
            };

            let errors = output.violations.iter().filter(|v| v.severity == "error").count();
            let warnings = output.violations.iter().filter(|v| v.severity == "warning").count();
            
            println!(
                "  {} {} ({} errors, {} warnings)",
                colors::dim("Truth Status:"),
                status_color,
                colors::error(errors),
                colors::warning(warnings)
            );

            if let Some(score) = output.health_score {
                println!(
                    "  {} {}",
                    colors::dim("Health Score: "),
                    colors::health_bar(score, 20)
                );
            }
            println!();

            if !output.new_components.is_empty() {
                println!("{}", colors::style("New components (may need documentation):").bold());
                for c in &output.new_components {
                    println!("  {} {}", colors::success("+"), c);
                }
                println!();
            }

            if !output.missing_components.is_empty() {
                println!("{}", colors::style("Missing components (in baseline but not code):").bold());
                for c in &output.missing_components {
                    println!("  {} {}", colors::error("-"), c);
                }
                println!();
            }

            if !output.drifted_dependencies.is_empty() {
                println!("{}", colors::style("Drifted dependencies:").bold());
                for d in &output.drifted_dependencies {
                    println!("  {} {}", colors::warning("~"), d);
                }
                println!();
            }

            if !output.violations.is_empty() {
                println!("{}", colors::style("Active Violations:").bold());
                for v in &output.violations {
                    println!(
                        "  {} {}: {} {}",
                        colors::severity_icon(&v.severity),
                        colors::style(&v.kind).bold(),
                        v.message,
                        colors::dim(v.location.as_deref().unwrap_or(""))
                    );
                }
                println!();
            }

            if !output.open_questions.is_empty() {
                println!("{}", colors::style("Open questions:").bold());
                for q in &output.open_questions {
                    println!("  {} {}", colors::info("?"), q);
                }
                println!();
            }

            if !output.suggestions.is_empty() {
                println!("{}", colors::style("Next steps:").bold());
                for s in &output.suggestions {
                    println!("  {} {}", colors::success(">"), s);
                }
            }
        }
    }

    Ok(())
}
