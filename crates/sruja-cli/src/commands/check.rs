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
use sruja_diff::{SourceRef, Violation, ViolationKind};
use sruja_scan::{is_path_production_relevant as scan_prod_relevant, scan_repo};

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
    let has_production = paths.iter().any(|p| scan_prod_relevant(p));
    has_production
}

fn is_usize_zero(v: &usize) -> bool {
    *v == 0
}

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

fn severity_slug(v: &Violation) -> &'static str {
    match v.severity {
        sruja_diff::Severity::Error => "error",
        sruja_diff::Severity::Warning => "warning",
        sruja_diff::Severity::Info => "info",
    }
}

fn primary_source_for_annotations(v: &Violation) -> Option<SourceRef> {
    v.sources
        .iter()
        .find(|s| s.file.as_deref().is_some_and(scan_prod_relevant))
        .or_else(|| v.sources.first())
        .cloned()
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
            let fingerprint = fingerprint_violation(v);
            ViolationSummary {
                kind: kind_slug(v.kind).to_string(),
                severity: severity_slug(v).to_string(),
                fingerprint,
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
                confidence: v.confidence,
                evidence_count: v.evidence_count,
                production_relevant: v.production_relevant,
                baseline_delta: v.baseline_delta.clone(),
                suppressed: v.suppressed,
                sources: v.sources.clone(),
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

    let violations = categorize_violations(&active_violations);
    let suppressed_violations = categorize_violations(&suppressed_violations);
    let has_drift = !violations.is_empty();
    let open_questions = generate_open_questions(&violations);
    let suggestions = generate_suggestions(&violations, baseline_path.as_deref(), &truth_status);

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

            for v in &active_violations {
                let sev = match v.severity {
                    sruja_diff::Severity::Error => "error",
                    sruja_diff::Severity::Warning => "warning",
                    sruja_diff::Severity::Info => "notice",
                };
                let title = format!("Sruja {}", kind_slug(v.kind));
                let message = format!("{} ({})", v.message, fingerprint_violation(v));

                let escaped_title = title
                    .replace('%', "%25")
                    .replace('\n', "%0A")
                    .replace('\r', "%0D");
                let escaped_message = message
                    .replace('%', "%25")
                    .replace('\n', "%0A")
                    .replace('\r', "%0D");

                if let Some(src) = primary_source_for_annotations(v) {
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
                println!("Health score: {}/100", score);
            }
            println!();

            if output.violations.is_empty() {
                println!("No violations found.");
            } else {
                println!("Violations found:");
                for v in &output.violations {
                    println!("  - [{}:{}] {}", v.severity, v.kind, v.message);
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
