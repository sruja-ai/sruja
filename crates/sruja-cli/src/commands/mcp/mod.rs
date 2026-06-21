//! MCP JSON-RPC server for architecture context tools.
//!
//! Layout:
//! - `transport` — CLI entry (`sruja mcp`) and stdio framing
//! - `server` — JSON-RPC dispatch (initialize, tools/list, tools/call, …)
//! - `definitions` — tool JSON schemas for `tools/list`
//! - `run_tool` — tool handlers (`read`, `governance`, `graph`, `memory`)
//! - `ladder` / `helpers` — progressive disclosure builders and shared utilities

mod config;
mod definitions;
mod helpers;
mod ladder;
mod mcp_v2;
mod server;
mod trace;
mod transport;

mod run_tool;

#[cfg(test)]
mod tests;

pub use mcp_v2::mcp_v2;
pub use transport::mcp;

#[cfg(test)]
pub(crate) use config::{
    is_mutating_mcp_tool, mcp_tools_for_list_with_readonly, ToolProfile, ENV_MCP_TOOL_PROFILE,
    ENV_MCP_WATCH_DRIFT, MCP_MUTATING_TOOLS, MCP_PROTOCOL_VERSION,
};
#[cfg(test)]
pub(crate) use run_tool::run_tool;
#[cfg(test)]
pub(crate) use server::McpServer;
