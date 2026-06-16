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
//! # use sruja_agent::{Agent, llm::OpenAiClient, tool::{ToolRegistry, tools}};
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
//! let answer = agent.comprehend("How does the graph module work?").await?;
//! # Ok(())
//! # }
//! ```

pub mod cognition;
pub mod dlc;
pub mod executor;
pub mod llm;
pub mod matts;
pub mod memory;
pub mod multi;
pub mod pair;
pub mod program;
pub mod tool;
pub mod verify;

pub use cognition::{Agent, AgentBuilder, AgentConfig, Comprehension};
pub use dlc::DlcPipeline;
pub use executor::TrajectoryExecutor;
pub use matts::{ContrastResult, TrajectoryOutcome, TrajectoryRunner, TrajectoryStatus};
pub use memory::{
    AgenticMemory, CurationReport, LowUtilityEntry, MergeSuggestion, StaleEntry,
};
pub use multi::BrainstormSession;
pub use pair::PairSession;

// Re-export shared learning types from sruja-graph for backward compatibility.
pub use sruja_graph::learning::{
    generate_entry_id, parse_hitl_kind, ExperimentOutcome, LearningEntry, LearningKind,
    LearningPatch, MemoryError,
};
