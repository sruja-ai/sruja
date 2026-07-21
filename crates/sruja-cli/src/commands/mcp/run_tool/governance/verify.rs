use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::super::helpers::*;
use super::super::finish;
use crate::commands::CliError;

pub(crate) async fn handle(
    arguments: &Value,
    repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<Option<String>, CliError> {
    let files_arg = arguments.get("files").and_then(|v| v.as_array());
    let file_list: Option<Vec<String>> = files_arg.map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });

    let baseline_path =
        crate::utils::architecture_path::resolve_architecture_path(std::path::Path::new(&repo));
    let mut lint_ok = true;
    let mut lint_errors = Vec::new();
    let mut program_opt = None;

    if let Some(ref path) = baseline_path {
        if let Ok(content) = std::fs::read_to_string(path) {
            let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
            match parser.parse(&content) {
                Ok(program) => {
                    let validator = sruja_engine::Validator::with_default_rules();
                    let diagnostics = validator.validate_sync(&program);
                    for d in diagnostics {
                        if d.severity == sruja_diagnostics::Severity::Error {
                            lint_ok = false;
                        }
                        lint_errors.push(serde_json::json!({
                            "severity": format!("{:?}", d.severity),
                            "code": d.code.clone(),
                            "message": d.message.clone(),
                            "line": d.location.line,
                            "column": d.location.column,
                        }));
                    }
                    program_opt = Some(program);
                }
                Err(diagnostics) => {
                    lint_ok = false;
                    for d in diagnostics {
                        lint_errors.push(serde_json::json!({
                            "severity": "Error",
                            "code": d.code.clone(),
                            "message": d.message.clone(),
                            "line": d.location.line,
                            "column": d.location.column,
                        }));
                    }
                }
            }
        }
    }

    let graph = get_or_scan_graph(graph_cache, repo).await?;
    let mut drift_violations = Vec::new();
    let report = sruja_diff::detect_architectural_drift(&graph);
    let mut all_violations = report.violations;

    if let Some(ref p) = baseline_path {
        if let Ok(status) =
            crate::commands::scan::drift::truth_status_from_baseline_compare(&graph, p)
        {
            if matches!(status, sruja_diff::TruthStatus::Drifted) {
                if let Some(ref program) = program_opt {
                    let proposed = sruja_diff::program_to_graph(program);
                    let diff = sruja_diff::compare_graphs(&graph, &proposed);
                    for v in diff.violations {
                        if !all_violations
                            .iter()
                            .any(|rv| rv.message == v.message && rv.location == v.location)
                        {
                            all_violations.push(v);
                        }
                    }
                }
            }
        }
    }

    if let Some(ref fl) = file_list {
        let mut impacted_ids = std::collections::HashSet::new();
        for f in fl {
            for node in &graph.nodes {
                if node
                    .path
                    .as_ref()
                    .is_some_and(|p| p.contains(f) || f.contains(p))
                {
                    impacted_ids.insert(node.id.clone());
                }
            }
        }
        all_violations.retain(|v| {
            v.location.as_ref().is_some_and(|l| {
                fl.iter().any(|f| l.contains(f)) || impacted_ids.contains(l)
            })
        });
    }

    for v in all_violations {
        drift_violations.push(serde_json::json!({
            "severity": format!("{:?}", v.severity),
            "kind": format!("{:?}", v.kind),
            "message": v.message.clone(),
            "location": v.location.clone(),
            "suggestion": v.suggestion.clone()
        }));
    }

    let mut critique_violations = Vec::new();
    if let Some(ref fl) = file_list {
        let engine = sruja_intent::CritiqueEngine::new(graph.clone(), program_opt);
        let report = engine.critique(&sruja_intent::CritiqueRequest {
            changed_files: fl.clone(),
            description: None,
            proposal_id: None,
            base_ref: None,
            head_ref: None,
        });
        for finding in report.violations {
            let location = finding
                .evidence
                .first()
                .and_then(|e| e.location.clone())
                .unwrap_or_default();
            critique_violations.push(serde_json::json!({
                "severity": format!("{:?}", finding.severity),
                "message": finding.detail.clone(),
                "location": location,
                "rule_id": finding.rule_id.clone().unwrap_or_else(|| format!("{:?}", finding.category))
            }));
        }
    }

    let ok = lint_ok && drift_violations.is_empty() && critique_violations.is_empty();

    let out = serde_json::json!({
        "schema_version": "verify_architecture/v1",
        "ok": ok,
        "lint": {
            "ok": lint_ok,
            "errors": lint_errors
        },
        "drift": {
            "ok": drift_violations.is_empty(),
            "violations": drift_violations
        },
        "critique": {
            "ok": critique_violations.is_empty(),
            "violations": critique_violations
        }
    });

    finish(Ok(serde_json::to_string_pretty(&out)?))
}
