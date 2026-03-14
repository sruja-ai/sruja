//! Sync command: refresh evidence (discover → .sruja/context.json) and run drift.

use std::fs;
use std::path::Path;

use super::discover::discover_context_json_from_graph;
use super::scan::{print_diff_text, print_drift_text};
use super::CliError;
use crate::utils::architecture_path;
use sruja_scan::scan_repo;

fn iso8601_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Refresh evidence and drift: write .sruja/context.json (with timestamp), then run drift.
pub async fn sync(repo_root: &str) -> Result<(), CliError> {
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
    value["updated_at"] = serde_json::Value::String(iso8601_now());
    let path = dot_sruja.join("context.json");
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
    eprintln!("Wrote {}", path.display());

    if let Some(baseline_path) = architecture_path::resolve_architecture_path(repo_path) {
        let content = fs::read_to_string(&baseline_path)?;
        let parser = sruja_language::Parser::new(baseline_path.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| CliError::Parse {
            file: baseline_path.to_string_lossy().to_string(),
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
        eprintln!("Truth: {} ({} violation(s))", truth, diff.violations.len());
        print_diff_text(&diff, false);
        return Ok(());
    }

    let drift = sruja_diff::detect_architectural_drift(&graph);
    eprintln!("Truth: unknown ({} violation(s))", drift.violations.len());
    eprintln!("Health score: {}/100", drift.health_score);
    print_drift_text(&drift, false);
    Ok(())
}
