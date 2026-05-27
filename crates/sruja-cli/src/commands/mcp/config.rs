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
/// Tool profile to use: minimal, coding, arch, full. Can be set via SRUJA_MCP_TOOL_PROFILE env var or initializationOptions.
pub(crate) const ENV_MCP_TOOL_PROFILE: &str = "SRUJA_MCP_TOOL_PROFILE";

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

// pub(crate) fn mcp_watch_drift_enabled() -> bool {
//     mcp_env_truthy(ENV_MCP_WATCH_DRIFT)
// }

pub(crate) fn get_mcp_tool_profile() -> ToolProfile {
    // Check environment variable first
    if let Ok(profile) = std::env::var(ENV_MCP_TOOL_PROFILE) {
        return match profile.as_str() {
            "legacy" => ToolProfile::Legacy,
            "full" => ToolProfile::Legacy,
            "arch" => ToolProfile::Legacy,
            "coding" => ToolProfile::Legacy,
            "minimal" => ToolProfile::Legacy,
            _ => ToolProfile::Default,
        };
    }

    // Default to new default profile
    ToolProfile::Default
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolProfile {
    Default,
    Legacy,
}

/// Tools that write under `.sruja`, mutate git state, run user-supplied gate commands, or may apply repo changes.
pub(crate) const MCP_MUTATING_TOOLS: &[&str] = &[
    "sruja_propose_topology_change",
    "sruja_evaluate_mutation",
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

// Default Agent-Native profile (11 focused tools)
pub(crate) const DEFAULT_TOOLS: &[&str] = &[
    "sruja_get_boundaries",
    "sruja_suggest_fix",
    "sruja_check_violations",
    "sruja_verify_task",
    "sruja_get_decisions",
    "sruja_get_topology",
    "sruja_get_elements",
    "sruja_list_architecture_index",
    "sruja_get_task_context",
    "sruja_get_repomap",
    "sruja_check_drift",
];

pub(crate) fn mcp_tools_for_list_with_readonly(readonly: bool, profile: ToolProfile) -> Vec<Value> {
    let defs = tool_definitions();

    // First filter by readonly if needed
    let mut filtered_defs: Vec<Value> = if !readonly {
        defs
    } else {
        defs.into_iter()
            .filter(|t| {
                let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
                !is_mutating_mcp_tool(tool_name)
            })
            .collect()
    };

    // Then filter by profile
    if profile == ToolProfile::Default {
        filtered_defs.retain(|t| {
            let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
            DEFAULT_TOOLS.contains(&tool_name)
        });
    }

    filtered_defs
}
