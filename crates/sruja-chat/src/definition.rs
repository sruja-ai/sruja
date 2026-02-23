//! Agent definitions — admin-configured, reusable expert templates.
//!
//! Admins create agent definitions with role, knowledge context, and model.
//! These are added to sessions to participate in architecture discussions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Admin-configured agent template. Reusable across sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub id: String,
    pub name: String,
    /// Role in the discussion (e.g. "Subsystem Expert", "Architecture Reviewer")
    pub role: String,
    /// Core instructions and persona
    pub system_prompt: String,
    /// Domain knowledge: docs, architecture context, subsystem details
    #[serde(default)]
    pub knowledge_context: Option<String>,
    /// LLM model (required; e.g. "openai/gpt-4o", "anthropic/claude-3-haiku")
    pub model: String,
    /// Max messages to include in context; None = full history
    #[serde(default)]
    pub memory_limit_messages: Option<usize>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating an agent definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentDefinition {
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    #[serde(default)]
    pub knowledge_context: Option<String>,
    pub model: String,
    #[serde(default)]
    pub memory_limit_messages: Option<usize>,
}
