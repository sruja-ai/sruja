//! MCP `tools/call` dispatch (split by domain).

mod governance;
mod graph;
mod memory;
mod read;
mod utility;

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::config::{is_mutating_mcp_tool, mcp_readonly_enabled, ENV_MCP_READONLY};
use crate::commands::CliError;

pub(super) fn finish(result: Result<String, CliError>) -> Result<Option<String>, CliError> {
    Ok(Some(result?))
}

pub(crate) async fn run_tool(
    name: &str,
    arguments: &Value,
    default_repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<String, CliError> {
    let repo = arguments
        .get("path")
        .or_else(|| arguments.get("repo"))
        .and_then(|v| v.as_str())
        .unwrap_or(default_repo)
        .to_string();

    if mcp_readonly_enabled() && is_mutating_mcp_tool(name) {
        return Err(CliError::validation(format!(
            "MCP tool {name:?} is disabled when {} is set (read-only MCP profile)",
            ENV_MCP_READONLY
        )));
    }

    if let Some(out) = read::try_run(name, arguments, &repo, graph_cache).await? {
        return Ok(out);
    }
    if let Some(out) = governance::try_run(name, arguments, &repo, graph_cache).await? {
        return Ok(out);
    }
    if let Some(out) = graph::try_run(name, arguments, &repo, graph_cache).await? {
        return Ok(out);
    }
    if let Some(out) = memory::try_run(name, arguments, &repo, graph_cache).await? {
        return Ok(out);
    }
    if let Some(out) = utility::try_run(name, arguments, &repo, graph_cache).await? {
        return Ok(out);
    }
    Err(CliError::validation(format!("Unknown tool: {name}")))
}
