//! MCP JSON-RPC server for architecture context tools.

mod config;
mod definitions;
mod helpers;
mod ladder;
mod server;
mod trace;
mod transport;

mod run_tool;

#[cfg(test)]
mod tests;

pub use transport::mcp;

#[cfg(test)]
pub(crate) use config::{
    is_mutating_mcp_tool, mcp_tools_for_list_with_readonly, ENV_MCP_WATCH_DRIFT,
    MCP_MUTATING_TOOLS, MCP_PROTOCOL_VERSION,
};
#[cfg(test)]
pub(crate) use run_tool::run_tool;
#[cfg(test)]
pub(crate) use server::McpServer;
