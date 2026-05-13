//! Sruja Agent: Agentic memory and autonomous optimization.

pub mod executor;
pub mod matts;
pub mod memory;

pub use executor::TrajectoryExecutor;
pub use matts::{ContrastResult, TrajectoryOutcome, TrajectoryRunner, TrajectoryStatus};
pub use memory::{AgenticMemory, ExperimentOutcome, LearningEntry, LearningKind};
