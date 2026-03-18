//! Check command: CI-focused tool for non-blocking exit code 0 on violations.
//! Outputs GitHub Actions annotation format for PR checks.
//!
//! Non-blocking: always exits with code 0.
//! Filters out violations whose evidence is only from non-production paths (book/, evaluation/, docs/, etc.)
//! so PR signals are production-relevant.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashSet, path::PathBuf};

use super::CliError;
use crate::utils::architecture_path;
use sruja_diff::{Violation, ViolationKind};
use sruja_scan::scan_repo;

/// Path segments that indicate non-production content (docs, evaluation, book, build artifacts).
const NON_PRODUCTION_SEGMENTS: &[&str] = &[
    "book",
    "books",
    "evaluation",
    "docs",
    "documentation",
    "benchmark",
    "bench",
    "perf",
    "target",
    "node_modules",
    "dist",
    "build",
    ".next",
    "out",
    "vendor",
    "third_party",
    "fixtures",
    "__mocks__",
    "test_data",
];

fn path_looks_non_production(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let lower = normalized.to_lowercase();
    for seg in NON_PRODUCTION_SEGMENTS {
        if lower.contains(&format!("/{}/", seg))
            || lower.starts_with(&format!("{}/", seg))
            || lower.ends_with(&format!("/{}", seg))
        {
            return true;
        }
    }
    if lower.ends_with(".md") || lower.ends_with(".rst") {
        return true;
    }
    false
}

/// True if this violation should be reported in check (PR signal). Excludes violations
/// whose only evidence is under non-production paths (book/, evaluation/, docs/, etc.).
fn is_production_relevant(v: &Violation) -> bool {
    let paths: Vec<&str> = v
        .sources
        .iter()
        .filter_map(|s| s.file.as_deref())
        .chain(v.location.as_deref())
        .collect();
    if paths.is_empty() {
        return true;
    }
    let has_production = paths.iter().any(|p| !path_looks_non_production(p));
    has_production
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckOutput {
    pub truth_status: String,
    pub baseline: Option<String>,
    pub violations_baseline: Option<String>,
    pub has_drift: bool,
    pub violations_count: usize,
    pub health_score: Option<u8>,
    pub violations: Vec<ViolationSummary>,
    pub open_questions: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ViolationSummary {
    pub kind: String,
    pub location: Option<String>,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ViolationBaseline {
    pub schema_version: u32,
    pub generated_at_unix: u64,
    pub fingerprints: Vec<String>,
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

fn fingerprint_violation(v: &Violation) -> String {
    let location = v.location.clone().unwrap_or_default();
    format!("{}|{}|{}", kind_slug(v.kind), location, v.message)
}

fn resolve_repo_relative(repo_root: &Path, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo_root.join(p)
    }
}

fn load_violation_baseline(baseline_path: &Path) -> Result<HashSet<String>, CliError> {
    let content = fs::read_to_string(baseline_path)?;
    let baseline: ViolationBaseline =
        serde_json::from_str(&content).map_err(|e| CliError::Validation(e.to_string()))?;
    Ok(baseline.fingerprints.into_iter().collect())
}

fn categorize_violations(violations: &[sruja_diff::Violation]) -> Vec<ViolationSummary> {
    violations
        .iter()
        .map(|v| {
            let location = v
                .location
                .as_deref()
                .or_else(|| v.sources.first().and_then(|s| s.detail.as_deref()))
                .unwrap_or("unknown");
            ViolationSummary {
                kind: kind_slug(v.kind).to_string(),
                location: v.location.clone(),
                message: match v.kind {
                    ViolationKind::OrphanComponent | ViolationKind::UndocumentedComponent => {
                        format!("{} (potential new component)", location)
                    }
                    ViolationKind::LayerViolation => {
                        format!("{} - unexpected layer dependency", location)
                    }
                    ViolationKind::CircularDependency => {
                        format!("{} - circular dependency", location)
                    }
                    ViolationKind::GodModule => {
                        format!("{} - too many dependencies", location)
                    }
                    ViolationKind::MissingDependency => {
                        format!("{} - missing expected dependency", location)
                    }
                    ViolationKind::PatternMismatch => {
                        format!("{} - pattern mismatch", location)
                    }
                },
            }
        })
        .collect()
}

fn generate_open_questions(violations: &[ViolationSummary]) -> Vec<String> {
    let mut questions = Vec::new();

    if violations
        .iter()
        .any(|v| v.kind == "orphan-component" || v.kind == "undocumented-component")
    {
        questions
            .push("Are there new components that should be documented in repo.sruja?".to_string());
    }

    if violations.iter().any(|v| v.kind == "circular-dependency") {
        questions.push(
            "Should circular dependencies be broken by introducing an intermediary service?"
                .to_string(),
        );
    }

    if violations.iter().any(|v| v.kind == "layer-violation") {
        questions.push(
            "Are there layer violations that indicate missing architectural boundaries?"
                .to_string(),
        );
    }

    if violations.iter().any(|v| v.kind == "god-module") {
        questions.push(
            "Should any god modules be split into smaller services with clear responsibilities?"
                .to_string(),
        );
    }

    questions
}

fn generate_suggestions(
    _violations: &[ViolationSummary],
    baseline_path: Option<&Path>,
    truth_status: &str,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if baseline_path.is_none() {
        suggestions.push(
            "Run: Use sruja-architecture skill to generate repo.sruja from evidence. Ask targeted questions if scope is unclear."
                .to_string(),
        );
    }

    match truth_status {
        "reviewed" => {
            suggestions.push("Architecture is in sync - no action needed.".to_string());
        }
        "drifted" => {
            suggestions.push(
                "Review drift findings and update repo.sruja if architecture changed intentionally"
                    .to_string(),
            );
            suggestions.push(
                "Or refactor code to match existing architecture if drift is unintentional"
                    .to_string(),
            );
        }
        _ => {}
    }

    suggestions
}

pub async fn baseline(repo_root: &str, output: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;
    let drift = sruja_diff::detect_architectural_drift(&graph);
    let filtered: Vec<_> = drift
        .violations
        .iter()
        .filter(|v| is_production_relevant(v))
        .cloned()
        .collect();

    let fingerprints: Vec<String> = filtered.iter().map(fingerprint_violation).collect();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let baseline = ViolationBaseline {
        schema_version: 1,
        generated_at_unix: now,
        fingerprints,
    };

    let out_path = resolve_repo_relative(repo_path, output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json =
        serde_json::to_string_pretty(&baseline).map_err(|e| CliError::Validation(e.to_string()))?;
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

    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);

    let baseline_filter_set = if let Some(b) = violations_baseline {
        let p = resolve_repo_relative(repo_path, b);
        Some(load_violation_baseline(&p)?)
    } else {
        None
    };

    let (truth_status, filtered_violations, health_score) =
        if let Some(ref baseline) = baseline_path {
            let content = fs::read_to_string(baseline)?;
            let parser = sruja_language::Parser::new(baseline.to_string_lossy().as_ref());
            let program = parser.parse(&content).map_err(|diags| {
                let message = diags
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");
                CliError::Parse {
                    file: baseline.to_string_lossy().to_string(),
                    message,
                }
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

    let filtered_violations: Vec<Violation> = if let Some(ref set) = baseline_filter_set {
        filtered_violations
            .into_iter()
            .filter(|v| !set.contains(&fingerprint_violation(v)))
            .collect()
    } else {
        filtered_violations
    };

    let violations = categorize_violations(&filtered_violations);
    let has_drift = !violations.is_empty();
    let open_questions = generate_open_questions(&violations);
    let suggestions = generate_suggestions(&violations, baseline_path.as_deref(), &truth_status);

    let output = CheckOutput {
        truth_status,
        baseline: baseline_path.and_then(|p| p.to_str().map(String::from)),
        violations_baseline: violations_baseline.map(|s| s.to_string()),
        has_drift,
        violations_count: violations.len(),
        health_score,
        violations,
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
            if output.has_drift {
                let drift_msg = format!(
                    "Drift detected ({} violation(s)). Review and update repo.sruja if intentional.",
                    output.violations_count
                );
                let escaped = drift_msg
                    .replace('%', "%25")
                    .replace('\n', "%0A")
                    .replace('\r', "%0D");
                println!("::warning file={}::title=Sruja Drift::{}", file, escaped);
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
                println!("Health score: {}/100", score);
            }
            println!();

            if output.violations.is_empty() {
                println!("No violations found.");
            } else {
                println!("Violations found:");
                for v in &output.violations {
                    println!("  - [{}] {}", v.kind, v.message);
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
