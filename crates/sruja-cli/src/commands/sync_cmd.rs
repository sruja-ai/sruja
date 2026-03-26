//! Sync command: refresh evidence (discover → .sruja/context.json) and run drift.

use std::fs;
use std::path::Path;

use super::discover::{discover_context_json_from_graph, discover_explanation_json};
use super::CliError;
use crate::utils::architecture_path;
use sruja_diff::{Severity, Violation, ViolationKind};
use sruja_scan::scan_repo;
use std::collections::HashSet;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SyncOutput {
    truth_status: String,
    baseline: Option<String>,
    violations_count: usize,
    health_score: Option<u8>,
    context_path: String,
}

fn iso8601_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Read git HEAD commit (short) if repo is a git work tree; otherwise None.
fn git_commit_short(repo_path: &Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Context.json schema version for machine consumers.
const CONTEXT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, serde::Serialize)]
struct ViolationSummary {
    kind: String,
    severity: String,
    fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    production_relevant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppressed: Option<bool>,
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
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn fingerprint_violation(v: &Violation) -> String {
    let location = v.location.clone().unwrap_or_default();
    format!("{}|{}|{}", kind_slug(v.kind), location, v.message)
}

/// Refresh evidence and drift: write .sruja/context.json (with timestamp, git_commit, baseline_path, truth_status), then run drift.
pub async fn sync(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let dot_sruja = repo_path.join(".sruja");
    if !dot_sruja.exists() {
        fs::create_dir_all(&dot_sruja).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create {}: {}", dot_sruja.display(), e),
            ))
        })?;
    }

    // Scan once and reuse for context + drift/baseline-compare to avoid redundant work.
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;
    let mut value = match discover_explanation_json(repo_root) {
        Ok(json) => serde_json::from_str::<serde_json::Value>(&json)
            .map_err(|e| CliError::Validation(e.to_string()))?,
        Err(_) => {
            let ctx = discover_context_json_from_graph(repo_root, repo_path, &graph)?;
            serde_json::to_value(&ctx).map_err(|e| CliError::Validation(e.to_string()))?
        }
    };

    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let baseline = baseline_path
        .as_ref()
        .and_then(|p| p.to_str().map(String::from));

    let (truth_status, violations, health_score) = if let Some(ref baseline_file) = baseline_path {
        let content = fs::read_to_string(baseline_file)?;
        let parser = sruja_language::Parser::new(baseline_file.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: baseline_file.to_string_lossy().to_string(),
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
        (
            "unknown".to_string(),
            drift.violations,
            Some(drift.health_score),
        )
    };

    // Write versioned context.json with evidence + truth state (plan: updated_at, git_commit, baseline_path, truth_status).
    value["updated_at"] = serde_json::Value::String(iso8601_now());
    value["schema_version"] = serde_json::Value::Number(CONTEXT_SCHEMA_VERSION.into());
    value["truth_status"] = serde_json::Value::String(truth_status.clone());
    value["baseline_path"] = baseline
        .as_ref()
        .map(|s| serde_json::Value::String(s.clone()))
        .unwrap_or(serde_json::Value::Null);
    value["git_commit"] = git_commit_short(repo_path)
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);

    // Add normalized violations with shared metadata, split by baseline suppression if baseline file exists.
    let violations: Vec<Violation> = violations
        .into_iter()
        .map(|mut v| {
            v.production_relevant = Some(true);
            if v.evidence_count.is_none() {
                v.evidence_count = Some(v.sources.len());
            }
            v
        })
        .collect();
    let baseline_fp_path = repo_path.join(".sruja").join("violations.baseline.json");
    let baseline_set: Option<HashSet<String>> = if baseline_fp_path.exists() {
        let txt = fs::read_to_string(&baseline_fp_path)?;
        let base: super::check::ViolationBaseline =
            serde_json::from_str(&txt).map_err(|e| CliError::Validation(e.to_string()))?;
        Some(base.fingerprints.into_iter().collect())
    } else {
        None
    };
    let (active, suppressed): (Vec<Violation>, Vec<Violation>) = if let Some(ref set) = baseline_set {
        violations
            .into_iter()
            .map(|mut v| {
                let sup = set.contains(&fingerprint_violation(&v));
                v.suppressed = Some(sup);
                v.baseline_delta = Some(if sup { "baseline" } else { "new" }.to_string());
                v
            })
            .partition(|v| v.suppressed != Some(true))
    } else {
        (violations, Vec::new())
    };
    let map_summary = |v: &Violation| ViolationSummary {
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
    };
    let active_summ: Vec<ViolationSummary> = active.iter().map(map_summary).collect();
    let suppressed_summ: Vec<ViolationSummary> = suppressed.iter().map(map_summary).collect();

    value["violations"] =
        serde_json::to_value(&active_summ).map_err(|e| CliError::Validation(e.to_string()))?;
    value["suppressed_violations"] =
        serde_json::to_value(&suppressed_summ).map_err(|e| CliError::Validation(e.to_string()))?;
    value["suppressed_count"] =
        serde_json::Value::Number((suppressed_summ.len() as u64).into());

    let path = dot_sruja.join("context.json");
    let context_path = path.display().to_string();
    fs::write(
        &path,
        serde_json::to_string_pretty(&value).map_err(|e| CliError::Validation(e.to_string()))?,
    )
    .map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write {}: {}", path.display(), e),
        ))
    })?;

    let graph_path = dot_sruja.join("graph.json");
    fs::write(
        &graph_path,
        serde_json::to_string_pretty(&graph).map_err(|e| CliError::Validation(e.to_string()))?,
    )
    .map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write {}: {}", graph_path.display(), e),
        ))
    })?;

    let output = SyncOutput {
        truth_status: truth_status.clone(),
        baseline: baseline.clone(),
        violations_count: active_summ.len(),
        health_score,
        context_path: context_path.clone(),
    };

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|e| CliError::Validation(e.to_string()))?
            );
        }
        "quiet" => {}
        _ => {
            eprintln!("Wrote {}", context_path);
            eprintln!("Wrote {}", graph_path.display());
            if let Some(ref base) = baseline {
                eprintln!("Baseline: {}", base);
            } else {
                eprintln!("No baseline (repo.sruja not found)");
            }
            eprintln!(
                "Truth: {} ({} violation(s))",
                truth_status,
                active.len()
            );
            if let Some(score) = health_score {
                eprintln!("Health score: {}/100", score);
            }

            if !active.is_empty() {
                eprintln!();
                eprintln!("Violations:");
                for v in &active {
                    eprintln!("  - {:?}", v);
                }
            }
        }
    }

    Ok(())
}
