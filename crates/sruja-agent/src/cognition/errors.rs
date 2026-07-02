//! Error types for the agent cognition module.

use crate::llm::LlmError;
use crate::tool::ToolError;
use thiserror::Error;

/// Errors that occur during plan parsing from LLM response JSON.
///
/// These are *recoverable* — the caller may issue a format-correction
/// re-prompt on the first failure before hard-failing.
#[derive(Debug, Error)]
pub enum PlanParseError {
    #[error("malformed JSON: {0}")]
    MalformedJson(String),
    #[error("missing required field `{field}` on subtask {subtask_index}")]
    MissingRequiredField { field: String, subtask_index: usize },
    #[error("plan contains no subtasks")]
    NoSubtasks,
    #[error("empty plan (JSON had no subtasks or risks)")]
    EmptyPlan,
}

/// Top-level errors from the agent loop.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),
    #[error("tool error: {0}")]
    Tool(#[from] ToolError),
    #[error("no LLM client configured")]
    NoLlm,
    #[error("max tool iterations ({0}) exceeded")]
    MaxIterations(usize),
    #[error("agent loop timed out after {0}s")]
    Timeout(u64),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("checkpoint error: {0}")]
    Checkpoint(String),
    #[error("MCP error: {0}")]
    Mcp(String),
    #[error("aborted by hook: {0}")]
    HookAborted(String),
    #[error("plan parse failed (unrecoverable after retry): {0}")]
    PlanParseFailed(#[source] PlanParseError),
    #[error("{0}")]
    Other(String),
}
