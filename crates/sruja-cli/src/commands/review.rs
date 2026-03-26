//! Review command: refresh evidence, detect drift, propose updates or open questions.

use std::fs;
use std::path::Path;

use super::CliError;
use crate::utils::architecture_path;
use sruja_diff::ViolationKind;
use sruja_scan::{is_path_production_relevant as scan_prod_relevant, scan_repo};

fn is_production_relevant(v: &sruja_diff::Violation) -> bool {
    let paths: Vec<&str> = v
        .sources
        .iter()
        .filter_map(|s| s.file.as_deref())
        .chain(v.location.as_deref())
        .collect();
    if paths.is_empty() {
        return true;
    }
    let has_production = paths.iter().any(|p| scan_prod_relevant(p));
    has_production
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReviewOutput {
    pub truth_status: String,
    pub baseline: Option<String>,
    pub has_drift: bool,
    pub violations_count: usize,
    pub health_score: Option<u8>,
    pub new_components: Vec<String>,
    pub missing_components: Vec<String>,
    pub drifted_dependencies: Vec<String>,
    pub open_questions: Vec<String>,
    pub suggestions: Vec<String>,
}

fn categorize_violations(
    violations: &[sruja_diff::Violation],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut new_components = Vec::new();
    let mut missing_components = Vec::new();
    let mut drifted_dependencies = Vec::new();

    for v in violations {
        let location = v
            .location
            .as_deref()
            .or_else(|| v.sources.first().and_then(|s| s.detail.as_deref()))
            .unwrap_or("unknown");

        match v.kind {
            ViolationKind::OrphanComponent | ViolationKind::UndocumentedComponent => {
                new_components.push(format!("{} (potential new component)", location));
            }
            ViolationKind::LayerViolation => {
                drifted_dependencies.push(format!("{} - unexpected layer dependency", location));
            }
            ViolationKind::CircularDependency => {
                drifted_dependencies.push(format!("{} - circular dependency", location));
            }
            ViolationKind::GodModule => {
                drifted_dependencies.push(format!("{} - too many dependencies", location));
            }
            ViolationKind::MissingDependency => {
                missing_components.push(format!("{} - missing expected dependency", location));
            }
            ViolationKind::PatternMismatch => {
                drifted_dependencies.push(format!("{} - pattern mismatch", location));
            }
        }
    }

    (new_components, missing_components, drifted_dependencies)
}

fn generate_open_questions(violations: &[sruja_diff::Violation]) -> Vec<String> {
    let mut questions = Vec::new();

    let has_orphans = violations
        .iter()
        .any(|v| v.kind == ViolationKind::OrphanComponent);
    let has_cycles = violations
        .iter()
        .any(|v| v.kind == ViolationKind::CircularDependency);
    let has_layer = violations
        .iter()
        .any(|v| v.kind == ViolationKind::LayerViolation);
    let has_god = violations
        .iter()
        .any(|v| v.kind == ViolationKind::GodModule);

    if has_orphans {
        questions
            .push("Are there new components that should be documented in repo.sruja?".to_string());
    }
    if has_cycles {
        questions.push(
            "Should circular dependencies be broken by introducing an intermediary service?"
                .to_string(),
        );
    }
    if has_layer {
        questions.push(
            "Are there layer violations that indicate missing architectural boundaries?"
                .to_string(),
        );
    }
    if has_god {
        questions
            .push("Should any god modules be split into smaller, focused services?".to_string());
    }

    if questions.is_empty() && !violations.is_empty() {
        questions.push(
            "Review detected violations - are they intentional or should repo.sruja be updated?"
                .to_string(),
        );
    }

    questions
}

fn generate_suggestions(
    truth_status: &str,
    violations: &[sruja_diff::Violation],
    has_baseline: bool,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if !has_baseline {
        suggestions.push(
            "Run: Use sruja-architecture skill to generate repo.sruja from evidence".to_string(),
        );
    } else if truth_status == "drifted" {
        suggestions.push(
            "Review drift findings and update repo.sruja if architecture changed intentionally"
                .to_string(),
        );
        suggestions.push(
            "Or refactor code to match existing architecture if drift is unintentional".to_string(),
        );
    } else if truth_status == "reviewed" {
        suggestions.push("Architecture is in sync - no action needed".to_string());
    }

    let has_god = violations
        .iter()
        .any(|v| v.kind == ViolationKind::GodModule);
    if has_god {
        suggestions.push(
            "Consider splitting god modules into smaller services with clear responsibilities"
                .to_string(),
        );
    }

    suggestions
}

pub async fn review(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let has_baseline = baseline_path.is_some();

    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let (truth_status, violations, health_score) = if let Some(ref baseline) = baseline_path {
        let content = fs::read_to_string(baseline)?;
        let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: baseline.to_string_lossy().to_string(),
            message: diags
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            diagnostics: diags,
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

    let violations: Vec<_> = violations
        .into_iter()
        .filter(is_production_relevant)
        .collect();

    let has_drift = truth_status == "drifted" || (!has_baseline && !violations.is_empty());
    let (new_components, missing_components, drifted_dependencies) =
        categorize_violations(&violations);
    let open_questions = generate_open_questions(&violations);
    let suggestions = generate_suggestions(&truth_status, &violations, has_baseline);

    let output = ReviewOutput {
        truth_status: truth_status.clone(),
        baseline: baseline_path.and_then(|p| p.to_str().map(String::from)),
        has_drift,
        violations_count: violations.len(),
        health_score,
        new_components,
        missing_components,
        drifted_dependencies,
        open_questions,
        suggestions,
    };

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::Validation(e.to_string()))?
            );
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
                println!("Health score: {}/100", score);
            }
            println!();

            if !output.new_components.is_empty() {
                println!("New components (may need documentation):");
                for c in &output.new_components {
                    println!("  + {}", c);
                }
                println!();
            }

            if !output.missing_components.is_empty() {
                println!("Missing components (in baseline but not code):");
                for c in &output.missing_components {
                    println!("  - {}", c);
                }
                println!();
            }

            if !output.drifted_dependencies.is_empty() {
                println!("Drifted dependencies:");
                for d in &output.drifted_dependencies {
                    println!("  ~ {}", d);
                }
                println!();
            }

            if !output.open_questions.is_empty() {
                println!("Open questions:");
                for q in &output.open_questions {
                    println!("  ? {}", q);
                }
                println!();
            }

            if !output.suggestions.is_empty() {
                println!("Suggestions:");
                for s in &output.suggestions {
                    println!("  > {}", s);
                }
            }
        }
    }

    Ok(())
}
