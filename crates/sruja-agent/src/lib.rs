//! Sruja Agent: Agentic memory and autonomous optimization.

pub mod matts;
pub mod memory;

pub use matts::{ContrastResult, TrajectoryOutcome, TrajectoryRunner, TrajectoryStatus};
pub use memory::{AgenticMemory, ExperimentOutcome, LearningEntry, LearningKind};
