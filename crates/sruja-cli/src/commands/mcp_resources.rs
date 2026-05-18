//! MCP `resources/list` and `resources/read` for `sruja://` URIs.

use std::path::Path;

use crate::commands::context::{
    build_architecture_context, format_invariant_markdown, format_llms_architecture,
    types::ArchitectureContext,
};
use crate::commands::{list_decisions, parse_sruja_file, scan_repo_cached, CliError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sruja_export::mermaid::exporter::{MermaidConfig, MermaidExporter};

pub const RESOURCE_SCHEME: &str = "sruja://";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceDescriptor {
    pub uri: String,
    #[serde(rename = "name")]
    pub name: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceContents {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: String,
}

fn load_arch_context(
    repo: &str,
    depth: usize,
    max_tokens: usize,
) -> Result<ArchitectureContext, CliError> {
    let graph = scan_repo_cached(Path::new(repo))?;
    build_architecture_context(&graph, repo, None, None, depth, max_tokens)
}

/// Static resource catalog for a repository (does not load file bodies).
pub fn list_resources(repo: &str) -> Result<Vec<McpResourceDescriptor>, CliError> {
    let repo_path = Path::new(repo);
    let mut out = vec![
        McpResourceDescriptor {
            uri: "sruja://architecture/main".to_string(),
            name: "Architecture DSL".to_string(),
            description: "Declared architecture file (repo.sruja)".to_string(),
            mime_type: "text/plain".to_string(),
        },
        McpResourceDescriptor {
            uri: "sruja://context/invariant.md".to_string(),
            name: "Invariant context".to_string(),
            description: "Policies, layers, boundaries (prompt-cache prefix)".to_string(),
            mime_type: "text/markdown".to_string(),
        },
        McpResourceDescriptor {
            uri: "sruja://context/llms-architecture.txt".to_string(),
            name: "LLM architecture brief".to_string(),
            description: "Compact counts, ladder, top boundaries".to_string(),
            mime_type: "text/plain".to_string(),
        },
    ];

    if crate::utils::architecture_path::resolve_architecture_path(repo_path).is_some() {
        out.push(McpResourceDescriptor {
            uri: "sruja://diagrams/current.mmd".to_string(),
            name: "Mermaid diagram".to_string(),
            description: "System-view Mermaid export".to_string(),
            mime_type: "text/plain".to_string(),
        });
    }

    if repo_path.join(".sruja").join("decisions").is_dir() {
        out.push(McpResourceDescriptor {
            uri: "sruja://decisions/index".to_string(),
            name: "Decision records index".to_string(),
            description: "Index of `.sruja/decisions/`".to_string(),
            mime_type: "application/json".to_string(),
        });
    }

    Ok(out)
}

pub fn resources_list_result(repo: &str) -> Result<Value, CliError> {
    Ok(json!({ "resources": list_resources(repo)? }))
}

pub async fn read_resource(repo: &str, uri: &str) -> Result<McpResourceContents, CliError> {
    let repo_path = Path::new(repo);
    let normalized = uri.strip_prefix(RESOURCE_SCHEME).unwrap_or(uri);

    match normalized {
        "architecture/main" => read_architecture_main(repo_path).await,
        "context/invariant.md" => read_invariant_markdown(repo).await,
        "context/llms-architecture.txt" => read_llms_architecture(repo).await,
        "diagrams/current.mmd" => read_mermaid_diagram(repo_path).await,
        "decisions/index" => read_decisions_index(repo_path),
        other => Err(CliError::validation(format!(
            "Unknown resource URI: {uri} (path {other:?}). Use resources/list."
        ))),
    }
}

pub async fn resources_read_result(repo: &str, uri: &str) -> Result<Value, CliError> {
    Ok(json!({ "contents": [read_resource(repo, uri).await?] }))
}

async fn read_architecture_main(repo_path: &Path) -> Result<McpResourceContents, CliError> {
    let path = crate::utils::architecture_path::resolve_architecture_path(repo_path)
        .ok_or_else(|| CliError::validation("No architecture file found (repo.sruja)"))?;
    let text = std::fs::read_to_string(&path).map_err(CliError::Io)?;
    Ok(McpResourceContents {
        uri: format!("{RESOURCE_SCHEME}architecture/main"),
        mime_type: "text/plain".to_string(),
        text,
    })
}

async fn read_invariant_markdown(repo: &str) -> Result<McpResourceContents, CliError> {
    let arch = load_arch_context(repo, 1, 3_000)?;
    Ok(McpResourceContents {
        uri: format!("{RESOURCE_SCHEME}context/invariant.md"),
        mime_type: "text/markdown".to_string(),
        text: format_invariant_markdown(&arch),
    })
}

async fn read_llms_architecture(repo: &str) -> Result<McpResourceContents, CliError> {
    let arch = load_arch_context(repo, 1, 1_200)?;
    Ok(McpResourceContents {
        uri: format!("{RESOURCE_SCHEME}context/llms-architecture.txt"),
        mime_type: "text/plain".to_string(),
        text: format_llms_architecture(&arch),
    })
}

async fn read_mermaid_diagram(repo_path: &Path) -> Result<McpResourceContents, CliError> {
    let path = crate::utils::architecture_path::resolve_architecture_path(repo_path)
        .ok_or_else(|| CliError::validation("No architecture file for diagram export"))?;
    let (_, program) = parse_sruja_file(&path)?;
    let text = MermaidExporter::new(MermaidConfig {
        direction: "LR".to_string(),
        view_level: 1,
        target_id: None,
    })
    .export(&program);
    Ok(McpResourceContents {
        uri: format!("{RESOURCE_SCHEME}diagrams/current.mmd"),
        mime_type: "text/plain".to_string(),
        text,
    })
}

fn read_decisions_index(repo_path: &Path) -> Result<McpResourceContents, CliError> {
    let items = list_decisions(repo_path)?;
    let text = serde_json::to_string(&items).map_err(|e| CliError::validation(e.to_string()))?;
    Ok(McpResourceContents {
        uri: format!("{RESOURCE_SCHEME}decisions/index"),
        mime_type: "application/json".to_string(),
        text,
    })
}
