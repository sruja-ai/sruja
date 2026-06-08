//! Shared violation infrastructure for CLI commands.
//! Unifies fingerprinting, categorization, and reporting across review, check, sync, and watch.

use sruja_diff::{SourceRef, Violation, ViolationKind};
use sruja_scan::is_path_production_relevant as scan_prod_relevant;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationBaselineV1 {
    pub schema_version: u32,
    pub generated_at_unix: u64,
    pub fingerprints: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationBaselineEntry {
    pub fingerprint: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationBaselineV2 {
    pub schema_version: u32,
    pub generated_at_unix: u64,
    pub violations: Vec<ViolationBaselineEntry>,
}

#[derive(Debug, Clone)]
pub struct LoadedViolationBaseline {
    /// Fingerprints that should be treated as suppressed.
    pub fingerprints: std::collections::HashSet<String>,
}

pub fn load_violations_baseline(
    baseline_path: &Path,
) -> Result<LoadedViolationBaseline, crate::commands::CliError> {
    let content = std::fs::read_to_string(baseline_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| crate::commands::CliError::validation(e.to_string()))?;
    let schema = json
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    // v1: { schema_version: 1, generated_at_unix, fingerprints: [] }
    if schema <= 1 {
        let baseline: ViolationBaselineV1 = serde_json::from_value(json)
            .map_err(|e| crate::commands::CliError::validation(e.to_string()))?;
        return Ok(LoadedViolationBaseline {
            fingerprints: baseline.fingerprints.into_iter().collect(),
        });
    }

    // v2: { schema_version: 2, generated_at_unix, violations: [{fingerprint, reason, expires?}] }
    let baseline: ViolationBaselineV2 = serde_json::from_str(&content)
        .map_err(|e| crate::commands::CliError::validation(e.to_string()))?;
    Ok(LoadedViolationBaseline {
        fingerprints: baseline
            .violations
            .into_iter()
            .map(|v| v.fingerprint)
            .collect(),
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationSummary {
    pub kind: String,
    pub severity: String,
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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

pub fn kind_slug(kind: ViolationKind) -> &'static str {
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

pub fn severity_slug(v: &Violation) -> &'static str {
    match v.severity {
        sruja_diff::Severity::Error => "error",
        sruja_diff::Severity::Warning => "warning",
        sruja_diff::Severity::Info => "info",
    }
}

pub fn fingerprint_violation(v: &Violation) -> String {
    let location = v.location.clone().unwrap_or_default();
    format!("{}|{}|{}", kind_slug(v.kind), location, v.message)
}

pub fn is_production_relevant(v: &Violation) -> bool {
    let paths: Vec<&str> = v
        .sources
        .iter()
        .filter_map(|s| s.file.as_deref())
        .chain(v.location.as_deref())
        .collect();
    if paths.is_empty() {
        return true;
    }
    paths.iter().any(|p| scan_prod_relevant(p))
}

pub fn categorize_violations(violations: &[Violation]) -> (Vec<String>, Vec<String>, Vec<String>) {
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

pub fn generate_open_questions(violations: &[Violation]) -> Vec<String> {
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

pub fn generate_suggestions(
    repo_root: &str,
    baseline_path: Option<&Path>,
    truth_status: &str,
    violations: &[Violation],
) -> Vec<String> {
    fn uniq_limit(items: Vec<String>, limit: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for s in items {
            if seen.insert(s.clone()) {
                out.push(s);
            }
            if out.len() >= limit {
                break;
            }
        }
        out
    }

    let mut suggestions: Vec<String> = Vec::new();

    let repo_path = Path::new(repo_root);
    if repo_path.join("justfile").exists() {
        suggestions.push("just check".to_string());
    } else if repo_path.join("Makefile").exists() {
        suggestions.push("make check".to_string());
    }

    if baseline_path.is_none() {
        suggestions.push(format!("sruja start -r {} --prompt", repo_root));
        suggestions.push("sruja lint repo.sruja".to_string());
        suggestions.push(format!("sruja daily -r {}", repo_root));
        return uniq_limit(suggestions, 3);
    }

    let baseline_hint = baseline_path
        .and_then(|p| p.to_str())
        .unwrap_or("repo.sruja");

    match truth_status {
        "drifted" => {
            suggestions.push(format!(
                "sruja drift -r {} -a {} --violations-only",
                repo_root, baseline_hint
            ));
            suggestions.push(format!("sruja lint {}", baseline_hint));
            suggestions.push(format!("sruja drift-pr -r {} -f text", repo_root));
        }
        "reviewed" => {
            suggestions.push(format!("sruja daily -r {}", repo_root));
            suggestions.push(format!("sruja watch -r {}", repo_root));
            suggestions.push(format!("sruja drift-pr -r {} -f text", repo_root));
        }
        _ => {
            suggestions.push(format!(
                "sruja drift -r {} -a {} --violations-only",
                repo_root, baseline_hint
            ));
            suggestions.push(format!("sruja daily -r {}", repo_root));
            suggestions.push(format!("sruja drift-pr -r {} -f text", repo_root));
        }
    }

    let _ = violations;

    uniq_limit(suggestions, 3)
}

pub fn summarize_violation(v: &Violation) -> ViolationSummary {
    ViolationSummary {
        kind: kind_slug(v.kind).to_string(),
        severity: severity_slug(v).to_string(),
        fingerprint: fingerprint_violation(v),
        location: v.location.clone(),
        message: v.message.clone(),
        confidence: v.confidence,
        evidence_count: v.evidence_count.or(Some(v.sources.len())),
        production_relevant: v.production_relevant.or(Some(true)),
        baseline_delta: v.baseline_delta.clone(),
        suppressed: v.suppressed,
        sources: v.sources.clone(),
    }
}

/// Apply baseline filter to violations, marking them as suppressed or new.
/// Returns (active_violations, suppressed_violations).
pub fn apply_baseline_filter(
    violations: Vec<Violation>,
    baseline_set: &Option<std::collections::HashSet<String>>,
) -> (Vec<Violation>, Vec<Violation>) {
    if let Some(ref set) = baseline_set {
        violations
            .into_iter()
            .map(|mut v| {
                let suppressed = set.contains(&fingerprint_violation(&v));
                v.suppressed = Some(suppressed);
                v.baseline_delta = Some(if suppressed { "baseline" } else { "new" }.to_string());
                v
            })
            .partition(|v| v.suppressed != Some(true))
    } else {
        (violations, Vec::new())
    }
}

/// Validate that a repository path exists.
pub fn validate_repo_exists(repo_root: &str) -> Result<&Path, crate::commands::CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(crate::commands::CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }
    Ok(repo_path)
}

/// Resolve a potentially relative path against a repository root.
pub fn resolve_repo_relative(repo_root: &Path, path: &str) -> std::path::PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo_root.join(p)
    }
}
