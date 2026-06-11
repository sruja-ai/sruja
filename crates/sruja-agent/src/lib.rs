//! Sruja Agent: Agentic memory and autonomous optimization.

pub mod executor;
pub mod matts;
pub mod memory;

pub use executor::TrajectoryExecutor;
pub use matts::{ContrastResult, TrajectoryOutcome, TrajectoryRunner, TrajectoryStatus};
pub use memory::{AgenticMemory, CurationReport, LowUtilityEntry, MergeSuggestion, StaleEntry};

// Re-export shared learning types from sruja-graph for backward compatibility.
pub use sruja_graph::learning::{
    ExperimentOutcome, LearningEntry, LearningKind, LearningPatch, MemoryError,
};
