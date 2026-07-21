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
    let violation_arg = arguments.get("violation");

    let graph = get_or_scan_graph(graph_cache, repo).await?;
    let mut suggestions = Vec::new();

    if let Some(val) = violation_arg {
        if let (Some(kind), Some(message)) = (
            val.get("kind").and_then(|v| v.as_str()),
            val.get("message").and_then(|v| v.as_str()),
        ) {
            let loc = val.get("location").and_then(|v| v.as_str()).unwrap_or("");
            generate_fix_suggestions(kind, message, loc, &mut suggestions);
        }
    } else {
        let report = sruja_diff::detect_architectural_drift(&graph);
        let mut violations = report.violations;

        let baseline_path =
            crate::utils::architecture_path::resolve_architecture_path(
                std::path::Path::new(&repo),
            );
        if let Some(p) = baseline_path {
            if let Ok(status) =
                crate::commands::scan::drift::truth_status_from_baseline_compare(&graph, &p)
            {
                if matches!(status, sruja_diff::TruthStatus::Drifted) {
                    let content = std::fs::read_to_string(&p)?;
                    let parser =
                        sruja_language::Parser::new(p.to_string_lossy().to_string());
                    if let Ok(program) = parser.parse(&content) {
                        let proposed = sruja_diff::program_to_graph(&program);
                        let diff = sruja_diff::compare_graphs(&graph, &proposed);
                        for v in diff.violations {
                            if !violations.iter().any(|rv| {
                                rv.message == v.message && rv.location == v.location
                            }) {
                                violations.push(v);
                            }
                        }
                    }
                }
            }
        }

        for v in violations {
            let kind_str = format!("{:?}", v.kind);
            let loc = v.location.as_deref().unwrap_or("");
            generate_fix_suggestions(&kind_str, &v.message, loc, &mut suggestions);
        }
    }

    finish(Ok(serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": "suggest_fix/v1",
        "suggestions": suggestions
    }))?))
}

fn extract_quoted_strings(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in s.chars() {
        if c == '\'' || c == '"' {
            if in_quotes {
                result.push(current.clone());
                current.clear();
                in_quotes = false;
            } else {
                in_quotes = true;
            }
        } else if in_quotes {
            current.push(c);
        }
    }
    result
}

fn generate_fix_suggestions(
    kind: &str,
    message: &str,
    location: &str,
    suggestions: &mut Vec<serde_json::Value>,
) {
    let quotes = extract_quoted_strings(message);
    match kind {
        "BoundaryViolation" => {
            let boundary_name = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "Services".to_string());
            let target_component = quotes
                .get(1)
                .cloned()
                .unwrap_or_else(|| "MySystem.DB".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "allowed_connection",
                "file": "architecture.sruja",
                "description": format!("Add '{}' to allowed_connections for boundary '{}' in architecture.sruja if this dependency is intended.", target_component, boundary_name),
                "code_edit": format!("allowed_connections = [\n  {{\n    target_boundary = \"{}\",\n    via = \"ApiCall\"\n  }}\n]", target_component)
            }));

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": format!("Refactor the code to avoid direct coupling to '{}'. Services should communicate via APIs/events, not direct database access or direct imports.", target_component),
                "code_edit": null
            }));
        }
        "UndocumentedComponent" => {
            let component_name = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "UnknownComponent".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "add_element",
                "file": "architecture.sruja",
                "description": format!("Add the undocumented component '{}' to your architecture DSL file to register it.", component_name),
                "code_edit": format!("{} = component \"{}\" {{\n  description \"Automatically discovered component\"\n}}", component_name, component_name)
            }));
        }
        "UndocumentedRelationship" => {
            let src = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "Source".to_string());
            let tgt = quotes
                .get(1)
                .cloned()
                .unwrap_or_else(|| "Target".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "add_relationship",
                "file": "architecture.sruja",
                "description": format!("Document the relationship '{} -> {}' in your architecture DSL file.", src, tgt),
                "code_edit": format!("{} -> {} \"calls\"", src, tgt)
            }));
        }
        "PolicyViolation" => {
            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": "Refactor code to comply with architecture policies. Remove the forbidden dependency or connection.",
                "code_edit": null
            }));
        }
        "MissingComponent" => {
            let component_name = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "UnknownComponent".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": format!("Implement the missing component '{}' in your codebase.", component_name),
                "code_edit": null
            }));

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "remove_element",
                "file": "architecture.sruja",
                "description": format!("Remove the declared component '{}' from architecture docs if it is no longer planned.", component_name),
                "code_edit": null
            }));
        }
        _ => {
            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": "Refactor code to comply with the architecture rules.",
                "code_edit": null
            }));
        }
    }
}
