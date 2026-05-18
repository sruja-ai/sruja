//! Structured drift payload for host context injection (Phase 3).

use serde_json::{json, Value};
use sruja_scan::Graph;
use std::path::Path;

use crate::commands::context_events::policy_fingerprint;
use crate::commands::CliError;

const TOP_VIOLATIONS: usize = 10;

/// Compact, schema-stable drift block for MCP / extension injectors (`drift_state/v1`).
pub fn build_drift_state_payload(repo: &str, graph: &Graph) -> Value {
    let repo_path = Path::new(repo);
    let report = sruja_diff::detect_architectural_drift(graph);
    let violations: Vec<Value> = report
        .violations
        .iter()
        .take(TOP_VIOLATIONS)
        .map(|v| {
            json!({
                "message": v.message,
                "location": v.location,
                "severity": format!("{:?}", v.severity),
            })
        })
        .collect();

    json!({
        "schema_version": "drift_state/v1",
        "repo": repo,
        "truth_status": format!("{:?}", report.truth_status),
        "health_score": report.health_score,
        "violation_count": report.violations.len(),
        "violations": violations,
        "truncated": report.violations.len() > TOP_VIOLATIONS,
        "policy_fingerprint": policy_fingerprint(repo_path),
        "refresh": {
            "mcp_tool": "sruja_check_drift",
            "cli": "sruja drift -r ."
        }
    })
}

pub fn build_drift_state_json(repo: &str, graph: &Graph) -> Result<String, CliError> {
    Ok(serde_json::to_string_pretty(&build_drift_state_payload(
        repo, graph,
    ))?)
}

/// Print `drift_state/v1` JSON to stdout (`sruja drift-state -r .`).
pub fn drift_state_print(repo: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {repo}"),
        )));
    }
    let graph = crate::commands::scan_repo_cached(repo_path)?;
    println!("{}", build_drift_state_json(repo, &graph)?);
    Ok(())
}
