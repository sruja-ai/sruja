//! Runtime trace analysis for AI-heavy systems.
//!
//! Provides execution trees, tool invocation graphs, and emergent cycle detection.
//! OTLP integration and full analysis pipeline in later phases.

pub mod agent;
pub mod analysis;
pub mod report;
pub mod trace;

pub use report::{build_report, RuntimeReport};

pub use agent::{AgentExecutionTree, ExecutionNode, ExecutionNodeKind, ExecutionStatus};
pub use analysis::{
    CycleSeverity, EmergentCycle, EmergentCycleDetector, HotspotDetector, RuntimeHotspot,
};
pub use trace::{ExecutionEdge, ExecutionGraph, ExecutionGraphProcessor, ExecutionTrace};
