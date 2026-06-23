//! Tool-call tracing trait for context event attribution (U5).
//!
//! The agent emits `tool_call`/`tool_result` events via this trait,
//! which the CLI layer implements using its `context_events` module.
//! This keeps the agent crate free of CLI dependencies.

use std::path::Path;

/// Emitter for tool-call tracing events into `context_events.jsonl`.
///
/// Implemented by the CLI layer (`sruja-cli`) and injected into the agent
/// via [`crate::AgentBuilder::tool_call_tracer`]. The agent calls these
/// methods before and after every tool dispatch when tracing is enabled.
pub trait ToolCallTracer: Send + Sync {
    /// Emit a `tool_call` event before tool dispatch.
    fn on_tool_call(
        &self,
        repo: &Path,
        run_id: &str,
        trace_id: &str,
        tool_name: &str,
        args_keys: &[String],
    );

    /// Emit a `tool_result` event after tool dispatch.
    #[allow(clippy::too_many_arguments)]
    fn on_tool_result(
        &self,
        repo: &Path,
        run_id: &str,
        trace_id: &str,
        tool_name: &str,
        ok: bool,
        empty: bool,
        elapsed_ms: u64,
    );
}
