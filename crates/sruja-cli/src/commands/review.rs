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
    pub context_score: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
}

fn is_usize_zero(v: &usize) -> bool {
    *v == 0
}

pub async fn review(repo_root: &str, format: &str, verbose: bool, include_critique: bool) -> Result<(), CliError> {
    let start_time = Instant::now();
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    if include_critique {
        // Run critique for modified (but unstaged) + staged changes
        let mut files = Vec::new();
        let output = std::process::Command::new("git")
            .args(["diff", "HEAD", "--name-only"])
            .current_dir(repo_path)
            .output()
            .map_err(CliError::Io)?;
        
        let git_files = String::from_utf8_lossy(&output.stdout);
        for f in git_files.lines() {
            if !f.is_empty() {
                files.push(f.to_string());
            }
        }
        
        if !files.is_empty() {
            // We just print it to stdout for now as part of the dashboard
            super::critique::critique(repo_root, files, None, None, None, None, false, format, None).await?;
            println!();
        }
    }

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);

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
        .filter(is_production_relevant)
        .collect();

    // Sort by severity (error first)
    filtered_violations.sort_by(|a, b| {
        use sruja_diff::Severity;
        let a_sev = match a.severity { Severity::Error => 0, Severity::Warning => 1, Severity::Info => 2 };
        let b_sev = match b.severity { Severity::Error => 0, Severity::Warning => 1, Severity::Info => 2 };
        a_sev.cmp(&b_sev)
    });

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

    let has_drift = truth_status == "drifted" || (baseline_path.is_none() && !active_violations.is_empty());
    let (new_components, missing_components, drifted_dependencies) =
        categorize_violations(&active_violations);
    let open_questions = generate_open_questions(&active_violations);
    let suggestions = generate_suggestions(
        repo_root,
        baseline_path.as_deref(),
        &truth_status,
        &active_violations,
    );

    let context_score = (|| {
        let kg = crate::graph_store::load_or_build_graph(repo_path).ok()?;
        let age_hours = crate::utils::context::context_age_hours(repo_path);
        Some(sruja_graph::compute_context_score(&kg, graph.nodes.len(), repo_path, age_hours).score)
    })();

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
        context_score,
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
            use crate::utils::table_formatter::TableFormatter;
            let formatter = TableFormatter::auto();
            let mut blocks = Vec::new();

            // 1. Health Summary
            let mut health_info = String::new();
            if let Some(score) = output.health_score {
                health_info.push_str(&format!("Health:  {}\n", colors::health_bar(score, 20)));
            }
            if let Some(score) = output.context_score {
                health_info.push_str(&format!("Context: {}\n", colors::health_bar(score, 20)));
            }
            let status_color = match output.truth_status.as_str() {
                "reviewed" => colors::success(&output.truth_status),
                "drifted" => colors::error(&output.truth_status),
                _ => colors::warning(&output.truth_status),
            };
            health_info.push_str(&format!("Status: {}\n", status_color));
            health_info.push_str(&format!("Issues: {} active, {} suppressed\n", 
                colors::style(output.violations_count).bold(), 
                colors::dim(output.suppressed_count)
            ));
            blocks.push(("Architecture Review".to_string(), health_info));

            // 2. Priority Fix (DX highlight)
            if !output.violations.is_empty() {
                let priority = &output.violations[0];
                let mut fix_info = String::new();
                fix_info.push_str(&format!("{} {}\n", colors::severity_icon(&priority.severity), colors::style(&priority.message).bold()));
                if let Some(ref loc) = priority.location {
                    fix_info.push_str(&format!("{} {}\n", colors::dim("Loc:"), loc));
                }
                blocks.push((colors::error("Priority Fix").to_string(), fix_info));
            }

            // 3. Structural Changes
            let mut changes_info = String::new();
            if !output.new_components.is_empty() {
                changes_info.push_str(&format!("{} new components detected\n", colors::success(output.new_components.len())));
            }
            if !output.missing_components.is_empty() {
                changes_info.push_str(&format!("{} components missing from code\n", colors::error(output.missing_components.len())));
            }
            if !output.drifted_dependencies.is_empty() {
                changes_info.push_str(&format!("{} drifted dependencies\n", colors::warning(output.drifted_dependencies.len())));
            }
            if changes_info.is_empty() {
                changes_info.push_str("No structural changes detected.\n");
            }
            blocks.push(("Structural Read".to_string(), changes_info));

            println!("{}", formatter.format_dashboard("DAILY ARCHITECTURE REVIEW", blocks));

            // Detailed Violations
            if !output.violations.is_empty() {
                println!("{}", colors::style("Detailed Findings:").bold());
                let limit = if verbose { output.violations.len() } else { 5 };
                for v in output.violations.iter().take(limit) {
                    println!(
                        "  {} {}: {} {}",
                        colors::severity_icon(&v.severity),
                        colors::style(&v.kind).bold(),
                        v.message,
                        colors::dim(v.location.as_deref().unwrap_or(""))
                    );
                }
                
                if output.violations.len() > limit {
                    println!("  {} ... and {} more issues. Run with {} to see all.", 
                        colors::dim("•"), 
                        output.violations.len() - limit,
                        colors::info("--verbose")
                    );
                }
                println!();
            }

            // Suggestions
            if !output.suggestions.is_empty() {
                println!("{}", colors::style("Recommended Actions:").bold());
                for s in &output.suggestions {
                    println!("  {} {}", colors::success(">"), s);
                }
            }
            
            println!();
            println!("{}", colors::dim(format!("Done in {}", colors::elapsed_display(elapsed))));
        }
    }

    Ok(())
}
