//! Agent execution tree types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Root of an agent execution tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionTree {
    pub root: ExecutionNode,
    pub agent_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub tokens_used: Option<TokenUsage>,
}

/// A node in the execution tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNode {
    pub id: String,
    pub kind: ExecutionNodeKind,
    pub children: Vec<ExecutionNode>,
    pub duration: Duration,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecutionNodeKind {
    Reasoning,
    ToolCall { tool_name: String },
    LlmGeneration,
    Branch,
    Loop,
    ErrorHandler,
    ExternalCall { service: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExecutionStatus {
    Success,
    Failed { error: String },
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt: u64,
    pub completion: u64,
    pub total: u64,
}
