//! Repo scan and drift tools: scan_repo, detect_drift, quickstart.

use sruja_graph::KnowledgeGraph;
use std::path::PathBuf;

use crate::tools::{SrujaTool, ToolResponse};

pub(super) fn execute_scan(
    tool: &SrujaTool,
    _graph: &KnowledgeGraph,
    validate_path: fn(&str) -> Result<PathBuf, String>,
) -> Option<ToolResponse> {
    match tool {
        SrujaTool::ScanRepo { path } => match validate_path(path) {
            Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                Ok(scan_graph) => Some(ToolResponse::success(
                    "scan_repo",
                    serde_json::to_value(scan_graph).unwrap_or_default(),
                )),
                Err(e) => Some(ToolResponse::error("scan_repo", e.to_string())),
            },
            Err(e) => Some(ToolResponse::error("scan_repo", e)),
        },

        SrujaTool::DetectDrift { repo_path } => match validate_path(repo_path) {
            Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                Ok(scan_graph) => {
                    let drift_report = sruja_diff::detect_architectural_drift(&scan_graph);
                    Some(ToolResponse::success(
                        "detect_drift",
                        serde_json::to_value(drift_report).unwrap_or_default(),
                    ))
                }
                Err(e) => Some(ToolResponse::error("detect_drift", e.to_string())),
            },
            Err(e) => Some(ToolResponse::error("detect_drift", e)),
        },

        SrujaTool::Quickstart { repo_path } => match validate_path(repo_path) {
            Ok(validated_path) => match sruja_scan::scan_repo(&validated_path) {
                Ok(scan_graph) => {
                    let drift_report = sruja_diff::detect_architectural_drift(&scan_graph);

                    let external_apis = scan_graph
                        .nodes
                        .iter()
                        .filter(|n| n.kind == sruja_scan::NodeKind::ExternalApi)
                        .count();

                    let result = serde_json::json!({
                        "repo": repo_path,
                        "health_score": drift_report.health_score,
                        "inventory": {
                            "modules": drift_report.total_modules,
                            "services": drift_report.total_services,
                            "databases": drift_report.total_databases,
                            "external_apis": external_apis,
                            "total_dependencies": drift_report.total_dependencies,
                        },
                        "violations_count": drift_report.violations.len(),
                        "suggestions_count": drift_report.suggestions.len(),
                    });

                    Some(ToolResponse::success("quickstart", result))
                }
                Err(e) => Some(ToolResponse::error("quickstart", e.to_string())),
            },
            Err(e) => Some(ToolResponse::error("quickstart", e)),
        },

        _ => None,
    }
}
