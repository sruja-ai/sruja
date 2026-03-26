//! Review command: refresh evidence, detect drift, propose updates or open questions.

use std::fs;
use std::path::Path;

use super::{scan_repo_cached, CliError};
use crate::utils::architecture_path;
use sruja_diff::{SourceRef, Violation, ViolationKind};
use sruja_scan::is_path_production_relevant as scan_prod_relevant;

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

fn is_usize_zero(v: &usize) -> bool {
    *v == 0
}

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
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ViolationSummary {
    pub kind: String,
    pub severity: String,
    pub fingerprint: String,
    pub location: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_relevant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppressed: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceRef>,
}

fn kind_slug(kind: ViolationKind) -> &'static str {
    match kind {
        ViolationKind::OrphanComponent => "orphan-component",
        ViolationKind::UndocumentedComponent => "undocumented-component",
        ViolationKind::LayerViolation => "layer-violation",
        ViolationKind::CircularDependency => "circular-dependency",
        ViolationKind::GodModule => "god-module",
        ViolationKind::MissingDependency => "missing-dependency",
        ViolationKind::PatternMismatch => "pattern-mismatch",
    }
}

fn severity_slug(v: &Violation) -> &'static str {
    match v.severity {
        sruja_diff::Severity::Error => "error",
        sruja_diff::Severity::Warning => "warning",
        sruja_diff::Severity::Info => "info",
    }
}

fn fingerprint_violation(v: &Violation) -> String {
    let location = v.location.clone().unwrap_or_default();
    format!("{}|{}|{}", kind_slug(v.kind), location, v.message)
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

fn baseline_path_hint(baseline_path: Option<&Path>) -> String {
    baseline_path
        .and_then(|path| path.to_str())
        .unwrap_or("repo.sruja")
        .to_string()
}

fn generate_suggestions(
    repo_root: &str,
    baseline_path: Option<&Path>,
    truth_status: &str,
    violations: &[sruja_diff::Violation],
) -> Vec<String> {
    let mut suggestions = Vec::new();
    let baseline_hint = baseline_path_hint(baseline_path);

    if baseline_path.is_none() {
        suggestions.push(format!("Start here: sruja start -r {} --prompt", repo_root));
        suggestions.push(format!(
            "Optional first look: sruja overview -r {}",
            repo_root
        ));
        suggestions.push(
            "Use the generated prompt with the sruja-architecture skill, save the result as repo.sruja, then run: sruja lint repo.sruja".to_string(),
        );
        suggestions.push(format!(
            "Once a baseline exists, make this your daily check: sruja daily -r {}",
            repo_root
        ));
        return suggestions;
    }

    if truth_status == "drifted" {
        suggestions.push(format!(
            "Review drift in detail: sruja drift -r {} -a {}",
            repo_root, baseline_hint
        ));
        suggestions.push(format!(
            "If the change is intentional, update the baseline and validate it with: sruja lint {}",
            baseline_hint
        ));
    } else if truth_status == "reviewed" {
        suggestions.push(format!(
            "Architecture is in sync. Keep this in your normal loop: sruja daily -r {}",
            repo_root
        ));
    } else {
        suggestions.push(format!(
            "Refresh evidence and review changes with: sruja daily -r {}",
            repo_root
        ));
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

    suggestions.push(format!(
        "While coding, keep architecture feedback live with: sruja watch -r {}",
        repo_root
    ));

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

    // Review is the day-to-day workflow, so refresh cached evidence first.
    super::sync_cmd::sync(repo_root, "quiet").await?;
    let graph = scan_repo_cached(repo_path)?;

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

    let mut violations: Vec<_> = violations
        .into_iter()
        .filter(is_production_relevant)
        .collect();

    // Mark production_relevant/evidence_count and split by baseline suppression if baseline fingerprints exist.
    for v in &mut violations {
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
                serde_json::from_str(&content).map_err(|e| CliError::Validation(e.to_string()))?;
            Some(baseline.fingerprints.into_iter().collect())
        } else {
            None
        };

    let (active_violations, suppressed_violations): (Vec<Violation>, Vec<Violation>) =
        if let Some(ref set) = baseline_set {
            violations
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
            (violations, Vec::new())
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

    let summarize = |vs: &[Violation]| -> Vec<ViolationSummary> {
        vs.iter()
            .map(|v| ViolationSummary {
                kind: kind_slug(v.kind).to_string(),
                severity: severity_slug(v).to_string(),
                fingerprint: fingerprint_violation(v),
                location: v.location.clone(),
                message: v.message.clone(),
                confidence: v.confidence,
                evidence_count: v.evidence_count,
                production_relevant: v.production_relevant,
                baseline_delta: v.baseline_delta.clone(),
                suppressed: v.suppressed,
                sources: v.sources.clone(),
            })
            .collect()
    };

    let output = ReviewOutput {
        truth_status: truth_status.clone(),
        baseline: baseline_path.and_then(|p| p.to_str().map(String::from)),
        has_drift,
        violations_count: active_violations.len(),
        health_score,
        suppressed_count: suppressed_violations.len(),
        violations: summarize(&active_violations),
        suppressed_violations: summarize(&suppressed_violations),
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
                println!("Structural health score: {}/100", score);
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
                println!("Next steps:");
                for s in &output.suggestions {
                    println!("  > {}", s);
                }
            }
        }
    }

    Ok(())
}
