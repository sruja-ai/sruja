//! Shared violation infrastructure for CLI commands.
//! Unifies fingerprinting, categorization, and reporting across review, check, sync, and watch.

use std::path::Path;
use sruja_diff::{SourceRef, Violation, ViolationKind};
use sruja_scan::is_path_production_relevant as scan_prod_relevant;

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

pub fn categorize_violations(
    violations: &[Violation],
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
    let mut suggestions = Vec::new();

    if baseline_path.is_none() {
        suggestions.push(format!("Start here: sruja start -r {} --prompt", repo_root));
        suggestions.push(
            "Use the generated prompt with the sruja-architecture skill, save as repo.sruja, then: sruja lint repo.sruja".to_string(),
        );
        return suggestions;
    }

    let baseline_hint = baseline_path.and_then(|p| p.to_str()).unwrap_or("repo.sruja");
    if truth_status == "drifted" {
        suggestions.push(format!(
            "Review drift: sruja drift -r {} -a {}",
            repo_root, baseline_hint
        ));
        suggestions.push(format!(
            "If the change is intentional, update {} and run 'sruja lint'",
            baseline_hint
        ));
    } else if truth_status == "reviewed" {
        suggestions.push("Architecture is in sync. Keep running 'sruja daily'.".to_string());
    }

    let has_god = violations.iter().any(|v| v.kind == ViolationKind::GodModule);
    if has_god {
        suggestions.push("Consider splitting god modules into smaller services.".to_string());
    }

    suggestions.push(format!("Keep live feedback while coding with: sruja watch -r {}", repo_root));

    suggestions
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
