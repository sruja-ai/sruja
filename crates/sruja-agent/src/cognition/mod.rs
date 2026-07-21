//! Agent cognition — the Principal Engineer loop.

pub mod changelog;
pub mod chat;
pub mod decision;
pub mod errors;
pub mod hook;
pub mod loop_event;
pub mod runbook;
pub mod subagent;
pub mod tool_tracing;

pub mod config;
pub mod complexity;
pub mod types;
pub mod checkpoint;
pub mod agent;
pub mod builder;

pub use crate::llm::{TaskTier, Usage};
pub use crate::LearningEntry;
pub use decision::{DecisionRecord, DecisionStatus};
pub use errors::{AgentError, PlanParseError};
pub use hook::{Hook, HookAction, HookRegistry, Hooks, LoggingHook};
pub use loop_event::{LoopEvent, LoopPhase, PlanBrief};
pub use runbook::{Runbook, RunbookSeverity};
pub use tool_tracing::ToolCallTracer;

pub use config::{AgentConfig, CritiqueMode, ModelMapping};
pub use complexity::{classify_task_complexity, TaskComplexity};
pub use types::{
    classify_error, content_has_quality, step_has_quality, AgentRunResult, Comprehension,
    CriterionStatus, CriterionVerdict, Critique, CritiquePersona, ErrorClass, FailureTracker,
    LoopConfig, LoopIteration, LoopResult, LoopTermination, PersonaResult, Plan, ScopeDrift,
    StepResult, StepStatus, Subtask, SubtaskKind, VerifierConfig,
};
pub use checkpoint::RunCheckpoint;
pub use agent::{extract_element_ids, truncate, Agent};
pub use builder::AgentBuilder;

mod prompts;

mod parsing;
pub use parsing::parse_plan_from_response;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_loop_event;
