//! MCP `prompts/list` and `prompts/get` — deterministic architecture task templates.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use crate::commands::context::logic::{
    build_cache_friendly_task_export, build_task_context, TaskSelectors,
};
use crate::commands::context::{build_architecture_context, format_invariant_markdown};
use crate::commands::{list_decisions, scan_repo_cached, CliError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptDescriptor {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<McpPromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

pub fn list_prompts() -> Vec<McpPromptDescriptor> {
    vec![
        McpPromptDescriptor {
            name: "sruja_new_service_scaffold".to_string(),
            description: "Scaffold a new service/container using repo policies, layers, and naming conventions."
                .to_string(),
            arguments: Some(vec![McpPromptArgument {
                name: "service_name".to_string(),
                description: "Human-readable service name (e.g. Payment API)".to_string(),
                required: true,
            }]),
        },
        McpPromptDescriptor {
            name: "sruja_review_change".to_string(),
            description: "Review a code change against declared architecture, drift, and decision records."
                .to_string(),
            arguments: Some(vec![
                McpPromptArgument {
                    name: "files".to_string(),
                    description: "Comma-separated changed file paths".to_string(),
                    required: true,
                },
                McpPromptArgument {
                    name: "description".to_string(),
                    description: "Short summary of the change".to_string(),
                    required: false,
                },
            ]),
        },
        McpPromptDescriptor {
            name: "sruja_focus_task".to_string(),
            description: "Orient on an architecture element or file before editing (use MCP ladder + focus)."
                .to_string(),
            arguments: Some(vec![
                McpPromptArgument {
                    name: "element_id".to_string(),
                    description: "Architecture element id (e.g. MySystem.Api)".to_string(),
                    required: false,
                },
                McpPromptArgument {
                    name: "file".to_string(),
                    description: "Repository-relative file path".to_string(),
                    required: false,
                },
            ]),
        },
        McpPromptDescriptor {
            name: "sruja_mcp_guide".to_string(),
            description: "Canonical MCP tool order for token-efficient architecture context.".to_string(),
            arguments: None,
        },
    ]
}

pub fn prompts_list_result() -> Value {
    json!({ "prompts": list_prompts() })
}

pub async fn prompts_get_result(
    repo: &str,
    name: &str,
    arguments: &Value,
) -> Result<Value, CliError> {
    let messages = match name {
        "sruja_new_service_scaffold" => messages_new_service_scaffold(repo, arguments).await?,
        "sruja_review_change" => messages_review_change(repo, arguments).await?,
        "sruja_focus_task" => messages_focus_task(repo, arguments).await?,
        "sruja_mcp_guide" => messages_mcp_guide(),
        other => {
            return Err(CliError::validation(format!(
                "Unknown prompt: {other}. Use prompts/list."
            )));
        }
    };

    Ok(json!({
        "description": name,
        "messages": messages
    }))
}

fn text_message(role: &str, text: String) -> Value {
    json!({
        "role": role,
        "content": { "type": "text", "text": text }
    })
}

async fn messages_new_service_scaffold(repo: &str, args: &Value) -> Result<Vec<Value>, CliError> {
    let service_name = args
        .get("service_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::validation("Missing argument: service_name"))?;

    let graph = scan_repo_cached(Path::new(repo))?;
    let arch = build_architecture_context(&graph, repo, None, None, 1, 10_000)?;
    let invariant = format!("{}\n\n", format_invariant_for_prompt(&arch));

    let body = format!(
        r#"You are adding a new deployable service to this repository.

Service name: {service_name}

Steps:
1. Call MCP `sruja_list_architecture_index` then `sruja_get_topology` on related systems.
2. Propose valid `.sruja` elements nested under the correct system/container hierarchy.
3. Add relationships and run `sruja lint` on the architecture file.
4. Respect layer boundaries and forbidden patterns below.

Do not invent architecture that violates declared policies."#
    );

    Ok(vec![text_message("user", format!("{invariant}{body}"))])
}

async fn messages_review_change(repo: &str, args: &Value) -> Result<Vec<Value>, CliError> {
    let files = args
        .get("files")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::validation("Missing argument: files (comma-separated paths)"))?;
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("(no description provided)");

    let graph = scan_repo_cached(Path::new(repo))?;
    let arch = build_architecture_context(&graph, repo, None, None, 1, 8_000)?;
    let mut context = format!("{}\n\n", format_invariant_for_prompt(&arch));

    context.push_str(&format!(
        "## Change under review\n\nFiles: {files}\n\nSummary: {description}\n\n"
    ));
    context.push_str("Checklist:\n");
    context.push_str("1. `sruja_check_drift` or `sruja_validate_change` for these files\n");
    context.push_str("2. `sruja_get_decisions` for linked Decision Records\n");
    context.push_str("3. Report boundary violations and missing architecture updates\n");

    if let Ok(decisions) = list_decisions(Path::new(repo)) {
        if !decisions.is_empty() {
            context.push_str("\n## Decision records (index)\n\n");
            for d in decisions.iter().take(10) {
                context.push_str(&format!("- {} [{}] {}\n", d.id, d.status, d.title));
            }
        }
    }

    Ok(vec![text_message("user", context)])
}

async fn messages_focus_task(repo: &str, args: &Value) -> Result<Vec<Value>, CliError> {
    let element_id = args.get("element_id").and_then(|v| v.as_str());
    let file = args.get("file").and_then(|v| v.as_str());
    if element_id.is_none() && file.is_none() {
        return Err(CliError::validation(
            "Provide at least one of: element_id, file",
        ));
    }

    let graph = scan_repo_cached(Path::new(repo))?;
    let selectors = TaskSelectors {
        element_id,
        file,
        query: None,
        base_ref: None,
        head_ref: None,
        depth: Some(2),
    };
    let task = build_task_context(&graph, repo, selectors, 8_000)?;
    let arch = build_architecture_context(&graph, repo, file, None, 2, 8_000)?;
    let export = build_cache_friendly_task_export(repo, &arch, task);
    let json =
        serde_json::to_string_pretty(&export).map_err(|e| CliError::validation(e.to_string()))?;

    let preamble = "Use MCP ladder tools first for orientation; below is cache-friendly task context JSON.\n\n";
    Ok(vec![text_message("user", format!("{preamble}{json}"))])
}

fn messages_mcp_guide() -> Vec<Value> {
    let text = r#"Sruja MCP context workflow (token-efficient):

1. `sruja_list_architecture_index` — discover ids, cycles, policy samples
2. `sruja_get_topology` — upstream/downstream for one id
3. `sruja_get_elements` — detail for selected ids
4. `sruja_get_task_context` — hydrated task context (`cache_friendly: true` when supported)
5. `sruja_get_focus_briefing` — blast radius + decisions when you have a file/element
6. `sruja_check_drift` — before merging architecture or code changes

Resources (optional): `sruja://context/invariant.md`, `sruja://architecture/main`

If diagnostics are truncated, use `sruja_get_diagnostic_full` with the vfs URI."#;
    vec![text_message("user", text.to_string())]
}

fn format_invariant_for_prompt(
    arch: &crate::commands::context::types::ArchitectureContext,
) -> String {
    format_invariant_markdown(arch)
}
