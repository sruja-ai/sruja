use serde_json::Value;

use super::definitions::tool_definitions;

pub(crate) const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// When set to `1`, `true`, `yes`, or `on` (case-insensitive), MCP lists only read/query tools and rejects mutating tool calls.
pub(crate) const ENV_MCP_READONLY: &str = "SRUJA_MCP_READONLY";
/// When set to `1`, `true`, `yes`, or `on`, emit one JSON line per `tools/call` on stderr for observability.
const ENV_MCP_LOG: &str = "SRUJA_MCP_LOG";
/// When set to `1`, `true`, `yes`, or `on`, append a `context_event/v2` row per `tools/call`.
const ENV_MCP_TRACE_EVENTS: &str = "SRUJA_MCP_TRACE_EVENTS";
/// When set, emit `notifications/drift_state` after MCP initialize (same as `initializationOptions.watch_drift`).
pub(crate) const ENV_MCP_WATCH_DRIFT: &str = "SRUJA_MCP_WATCH_DRIFT";

pub(crate) fn mcp_env_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let v = v.trim();
            v.eq_ignore_ascii_case("1")
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("yes")
                || v.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

pub(crate) fn mcp_readonly_enabled() -> bool {
    mcp_env_truthy(ENV_MCP_READONLY)
}

pub(crate) fn mcp_log_enabled() -> bool {
    mcp_env_truthy(ENV_MCP_LOG)
}

pub(crate) fn mcp_trace_events_enabled() -> bool {
    mcp_env_truthy(ENV_MCP_TRACE_EVENTS)
}

/// Tools that write under `.sruja`, mutate git state, run user-supplied gate commands, or may apply repo changes.
pub(crate) const MCP_MUTATING_TOOLS: &[&str] = &[
    "sruja_propose_topology_change",
    "sruja_commit_evolution",
    "sruja_add_element",
    "sruja_add_relationship",
    "sruja_propose_change",
    "sruja_ai_scratchpad",
    "sruja_sandbox",
    "sruja_evaluate_proposal",
    "sruja_record_learning",
    "sruja_record_learn_feedback",
    "sruja_agent_run",
    "sruja_record_context_event",
    "sruja_record_decision_event",
    "sruja_create_decision_record",
    "sruja_link_decision_to_element",
    "sruja_reindex_memory",
];

pub(crate) fn is_mutating_mcp_tool(name: &str) -> bool {
    MCP_MUTATING_TOOLS.contains(&name)
}

pub(crate) fn mcp_tools_for_list() -> Vec<Value> {
    mcp_tools_for_list_with_readonly(mcp_readonly_enabled())
}

pub(crate) fn mcp_tools_for_list_with_readonly(readonly: bool) -> Vec<Value> {
    let defs = tool_definitions();
    if !readonly {
        return defs;
    }
    defs.into_iter()
        .filter(|t| {
            let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
            !is_mutating_mcp_tool(tool_name)
        })
        .collect()
}
