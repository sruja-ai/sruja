use serde::{Deserialize, Serialize};

use crate::cognition::types::{
    Comprehension, FailureTracker, LoopIteration, LoopTermination, Plan, StepResult, Critique,
};
use crate::llm::Usage;

/// Persisted state for resuming a long-running agent loop after timeout or crash.
///
/// Written to `.sruja/runs/<run_id>/checkpoint.json` after each iteration.
/// On resume, the agent loads this file and continues from the next iteration.
/// Cleaned up on successful convergence.
/// Checkpoint for saving and resuming agent loop state.
///
/// Captures the full state of a running agent loop so it can be resumed
/// after interruption (crash, timeout, user cancel). Includes the goal,
/// comprehension, plan, step results, and all tracking state.
///
/// Checkpoints are written to `.sruja/runs/<run_id>/checkpoint.json`
/// after each iteration and cleaned up on successful convergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCheckpoint {
    /// The goal statement (for display and verification on resume).
    pub goal: String,
    /// The comprehension from the initial run (carried forward).
    pub comprehension: Comprehension,
    /// Iterations completed so far.
    pub iterations: Vec<LoopIteration>,
    /// The last plan produced (may be rejected, but needed for replanning).
    pub last_plan: Option<Plan>,
    /// Step results from the last iteration.
    pub last_steps: Vec<StepResult>,
    /// Critique from the last iteration.
    pub last_critique: Option<Critique>,
    /// Failure tracker state (what approaches failed and why).
    pub failure_tracker: FailureTracker,
    /// Total token usage accumulated so far.
    pub total_usage: Usage,
    /// Whether the loop converged.
    pub converged: bool,
    /// Termination reason.
    pub termination: LoopTermination,
    /// Issue signatures seen so far (for oscillation detection).
    pub seen_signatures: Vec<String>,
    /// Checkpoint timestamp (ISO 8601).
    pub timestamp: String,
}

impl RunCheckpoint {
    /// Save checkpoint to disk.
    pub fn write(&self, run_dir: &std::path::Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(run_dir)?;
        let path = run_dir.join("checkpoint.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json)?;
        tracing::debug!(path = %path.display(), "checkpoint: saved");
        Ok(())
    }

    /// Load checkpoint from disk.
    pub fn load(run_dir: &std::path::Path) -> Result<Self, std::io::Error> {
        let path = run_dir.join("checkpoint.json");
        let json = std::fs::read_to_string(&path)?;
        let checkpoint: Self = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(checkpoint)
    }

    /// Delete checkpoint file (called on successful convergence).
    pub fn cleanup(run_dir: &std::path::Path) -> Result<(), std::io::Error> {
        let path = run_dir.join("checkpoint.json");
        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::debug!("checkpoint: cleaned up");
        }
        Ok(())
    }

    /// Check if a checkpoint exists for a run directory.
    pub fn exists(run_dir: &std::path::Path) -> bool {
        run_dir.join("checkpoint.json").exists()
    }
}
