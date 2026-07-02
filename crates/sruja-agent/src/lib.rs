// Sruja Agent: Programmable agent framework with a deterministic cognition loop.
//! @element Sruja.Agent
//! @layer Core Engine
//! @boundary The agent framework must not depend on sruja-cli; CLI provides
//!           concrete tool/provider implementations via traits.
//!
//! Sruja Agent: a programmable agent framework with a deterministic spine.
//!
//! ## Architecture
//!
//! The framework is built on three pluggable traits:
//!
//! - [`llm::LlmClient`] — the brain (any LLM provider: OpenAI, Anthropic, local).
//! - [`tool::Tool`] — the hands (file ops, shell, sruja deterministic tools, custom).
//! - [`memory::Memory`] — long-term institutional memory (learnings, facts, decisions).
//!
//! The [`Agent`] ties them together and drives a cognition loop:
//! **comprehend → plan → execute → critique → reflect**.
//!
//! ## Quick start
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use sruja_agent::{Agent, goal::GoalSpec, llm::OpenAiClient, tool::{ToolRegistry, tools}};
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let llm = OpenAiClient::from_env()?;
//! let mut tools = ToolRegistry::new();
//! tools.register(Box::new(tools::FileRead::new()));
//!
//! let agent = Agent::builder()
//!     .llm(Arc::new(llm))
//!     .tools(tools)
//!     .build()?;
//!
//! let answer = agent.comprehend(&GoalSpec::new("How does the graph module work?")).await?;
//! # Ok(())
//! # }
//! ```

pub mod calibration;
pub mod cognition;
pub mod goal;
pub mod llm;
pub mod manifest;
pub mod memory;
pub mod tool;
pub mod verify;

pub use calibration::{
    decide, infer_reversibility, proceed_decision_record, AskInput, AskPlan, Reversibility,
    TargetHints, Thresholds, Verdict,
};
pub use cognition::changelog::AgentChangelog;
pub use cognition::chat::{TurnEvent, TurnResult};
pub use cognition::{
    classify_task_complexity, Agent, AgentBuilder, AgentConfig, AgentError, AgentRunResult,
    Comprehension, Critique, LoopConfig, LoopEvent, LoopIteration, LoopPhase,
    LoopResult, LoopTermination, ModelMapping, Plan, PlanBrief, TaskComplexity, ToolCallTracer,
    VerifierConfig,
};
pub use goal::GoalSpec;
pub use llm::{DEFAULT_MODEL, PREMIUM_MODEL};
pub use manifest::{LoopManifest, PipelineConfig, RecoveryStrategy, StageKind};
pub use memory::{AgenticMemory, CurationReport, LowUtilityEntry, MergeSuggestion, StaleEntry};

// Re-export shared learning types from sruja-graph for backward compatibility.
pub use sruja_graph::learning::{
    generate_entry_id, parse_hitl_kind, ExperimentOutcome, LearningEntry, LearningKind,
    LearningPatch, MemoryError,
};
