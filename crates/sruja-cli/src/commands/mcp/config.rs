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
            "minimal" => ToolProfile::Minimal,
            "coding" => ToolProfile::Coding,
            "arch" => ToolProfile::Arch,
            "full" => ToolProfile::Full,
            _ => ToolProfile::Default,
        };
    }

    // Default to coding profile
    ToolProfile::Default
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolProfile {
    /// Minimal: 5 core tools for basic architecture queries
    Minimal,
    /// Coding: 12 tools for active development workflows
    Coding,
    /// Arch: 18 tools for architecture review and governance
    Arch,
    /// Full: all tools (legacy compatibility)
    Full,
    /// Default alias (maps to Coding)
    Default,
}

/// Tools that write under `.sruja`, mutate git state, run user-supplied gate commands, or may apply repo changes.
pub(crate) const MCP_MUTATING_TOOLS: &[&str] = &[
    "sruja_add_element",
    "sruja_add_relationship",
    "sruja_propose_change",
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
    "sruja_classify",
    "sruja_sync_ide_rules",
];

pub(crate) fn is_mutating_mcp_tool(name: &str) -> bool {
    MCP_MUTATING_TOOLS.contains(&name)
}

/// Minimal profile: 5 core tools for basic architecture queries
const MINIMAL_TOOLS: &[&str] = &[
    "sruja_list_architecture_index",
    "sruja_get_topology",
    "sruja_get_elements",
    "sruja_check_drift",
    "sruja_check_violations",
];

/// Coding profile: 12 tools for active development workflows
const CODING_TOOLS: &[&str] = &[
    "sruja_list_architecture_index",
    "sruja_get_topology",
    "sruja_get_elements",
    "sruja_check_drift",
    "sruja_check_violations",
    "sruja_get_boundaries",
    "sruja_suggest_fix",
    "sruja_verify_task",
    "sruja_get_task_context",
    "sruja_get_repomap",
    "sruja_classify",
    "sruja_sync_ide_rules",
];

/// Arch profile: 18 tools for architecture review and governance
const ARCH_TOOLS: &[&str] = &[
    "sruja_list_architecture_index",
    "sruja_get_topology",
    "sruja_get_elements",
    "sruja_check_drift",
    "sruja_check_violations",
    "sruja_get_boundaries",
    "sruja_suggest_fix",
    "sruja_verify_task",
    "sruja_get_task_context",
    "sruja_get_repomap",
    "sruja_classify",
    "sruja_sync_ide_rules",
    "sruja_get_decisions",
    "sruja_critique",
    "sruja_get_focus_briefing",
    "sruja_get_context_score",
    "sruja_preflight_check",
    "sruja_verify_architecture",
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
    match profile {
        ToolProfile::Minimal => {
            filtered_defs.retain(|t| {
                let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
                MINIMAL_TOOLS.contains(&tool_name)
            });
        }
        ToolProfile::Coding | ToolProfile::Default => {
            filtered_defs.retain(|t| {
                let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
                CODING_TOOLS.contains(&tool_name)
            });
        }
        ToolProfile::Arch => {
            filtered_defs.retain(|t| {
                let tool_name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
                ARCH_TOOLS.contains(&tool_name)
            });
        }
        ToolProfile::Full => {
            // No filtering - return all tools
        }
    }

    filtered_defs
}
