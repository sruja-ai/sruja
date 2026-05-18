//! MCP `prompts/list` and `prompts/get` — deterministic architecture task templates.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use crate::commands::context::logic::{
    build_cache_friendly_task_export, build_task_context, TaskSelectors,
};
use crate::commands::context::{build_architecture_context, format_invariant_brief};
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
            description: "Scaffold a service/container under repo policies.".to_string(),
            arguments: Some(vec![McpPromptArgument {
                name: "service_name".to_string(),
                description: "Service name (e.g. Payment API)".to_string(),
                required: true,
            }]),
        },
        McpPromptDescriptor {
            name: "sruja_review_change".to_string(),
            description: "Review a change against architecture and decisions.".to_string(),
            arguments: Some(vec![
                McpPromptArgument {
                    name: "files".to_string(),
                    description: "Comma-separated file paths".to_string(),
                    required: true,
                },
                McpPromptArgument {
                    name: "description".to_string(),
                    description: "Change summary".to_string(),
                    required: false,
                },
            ]),
        },
        McpPromptDescriptor {
            name: "sruja_focus_task".to_string(),
            description: "Cache-friendly task JSON for an element or file.".to_string(),
            arguments: Some(vec![
                McpPromptArgument {
                    name: "element_id".to_string(),
                    description: "Element id (e.g. MySystem.Api)".to_string(),
                    required: false,
                },
                McpPromptArgument {
                    name: "file".to_string(),
                    description: "Repo-relative path".to_string(),
                    required: false,
                },
            ]),
        },
        McpPromptDescriptor {
            name: "sruja_mcp_guide".to_string(),
            description: "Token-efficient MCP tool order.".to_string(),
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

    Ok(json!({ "description": name, "messages": messages }))
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
    let arch = build_architecture_context(&graph, repo, None, None, 1, 1_500)?;
    let brief = format_invariant_brief(&arch);

    let body = format!(
        "Add service **{service_name}**.\n\
         1. MCP index → topology on related systems\n\
         2. Propose nested `.sruja` elements + relationships; `sruja lint`\n\
         3. Respect boundaries below; no invented policy violations\n\n\
         {brief}"
    );

    Ok(vec![text_message("user", body)])
}

async fn messages_review_change(repo: &str, args: &Value) -> Result<Vec<Value>, CliError> {
    let files = args
        .get("files")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::validation("Missing argument: files"))?;
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("(none)");

    let graph = scan_repo_cached(Path::new(repo))?;
    let arch = build_architecture_context(&graph, repo, None, None, 1, 1_500)?;
    let mut context = format_invariant_brief(&arch);
    context.push_str(&format!(
        "\n## Review\nFiles: {files}\nSummary: {description}\n\n\
         Run `sruja_check_drift` / `sruja_validate_change`; check decisions; report boundary gaps.\n"
    ));

    if let Ok(decisions) = list_decisions(Path::new(repo)) {
        for d in decisions.iter().take(5) {
            context.push_str(&format!("- {} [{}] {}\n", d.id, d.status, d.title));
        }
    }

    Ok(vec![text_message("user", context)])
}

async fn messages_focus_task(repo: &str, args: &Value) -> Result<Vec<Value>, CliError> {
    let element_id = args.get("element_id").and_then(|v| v.as_str());
    let file = args.get("file").and_then(|v| v.as_str());
    if element_id.is_none() && file.is_none() {
        return Err(CliError::validation("Provide element_id and/or file"));
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
    let task = build_task_context(&graph, repo, selectors, 3_500)?;
    let arch = build_architecture_context(&graph, repo, file, None, 2, 3_500)?;
    let export = build_cache_friendly_task_export(repo, &arch, task);
    let json = serde_json::to_string(&export).map_err(|e| CliError::validation(e.to_string()))?;

    Ok(vec![text_message(
        "user",
        format!("Ladder first (index → topology → elements). Task context JSON:\n{json}"),
    )])
}

fn messages_mcp_guide() -> Vec<Value> {
    vec![text_message(
        "user",
        "Sruja MCP (token-efficient): \
         `sruja_list_architecture_index` → `sruja_get_topology` → `sruja_get_elements` → \
         `sruja_get_task_context` (`cache_friendly: true`) → `sruja_get_focus_briefing` → \
         `sruja_check_drift`. Resources: `sruja://context/invariant.md`. \
         Truncated diagnostics: `sruja_get_diagnostic_full` + vfs URI."
            .to_string(),
    )]
}
