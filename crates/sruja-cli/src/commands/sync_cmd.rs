//! Sync command: refresh evidence (discover → .sruja/context.json) and run drift.

use std::fs;
use std::path::Path;

use super::discover::discover_context_json_from_graph;
use super::CliError;
use crate::utils::architecture_path;
use sruja_scan::scan_repo;

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
    let ctx = discover_context_json_from_graph(repo_root, repo_path, &graph)?;
    let mut value = serde_json::to_value(&ctx).map_err(|e| CliError::Validation(e.to_string()))?;

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

    let output = SyncOutput {
        truth_status: truth_status.clone(),
        baseline: baseline.clone(),
        violations_count: violations.len(),
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
        _ => {
            eprintln!("Wrote {}", context_path);
            if let Some(ref base) = baseline {
                eprintln!("Baseline: {}", base);
            } else {
                eprintln!("No baseline (repo.sruja not found)");
            }
            eprintln!(
                "Truth: {} ({} violation(s))",
                truth_status,
                violations.len()
            );
            if let Some(score) = health_score {
                eprintln!("Health score: {}/100", score);
            }

            if !violations.is_empty() {
                eprintln!();
                eprintln!("Violations:");
                for v in &violations {
                    eprintln!("  - {:?}", v);
                }
            }
        }
    }

    Ok(())
}
