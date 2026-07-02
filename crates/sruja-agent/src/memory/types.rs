//! Type definitions for agentic memory.
//!
//! Re-exports the canonical learning types from `sruja-graph` and defines
//! agent-specific curation types.

use crate::cognition::ErrorClass;
use serde::{Deserialize, Serialize};

// Re-export shared types from sruja-graph (single source of truth).
pub use sruja_graph::learning::{
    BlastRadius, ExperimentOutcome, LearningCategory, LearningConstraints, LearningEntry,
    LearningKind, LearningPatch, MemoryError, SignalPattern,
};

pub use sruja_graph::learning::generate_entry_id;

/// Persisted error frequency counts for cross-run learning.
///
/// Keyed by `(repo_path, error_class)` — tracks how often each error class
/// occurs in a specific repository. This enables injection like:
/// "In this repo, 45% of failures are type errors — check type annotations first."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFrequency {
    /// Repository path (absolute path on disk).
    pub repo_path: String,
    /// Error class being tracked.
    pub error_class: ErrorClass,
    /// Number of occurrences (non-zero).
    pub count: usize,
    /// Timestamp of last update (ISO 8601).
    pub last_updated: String,
}

/// Suggested merge of clustered learnings (curator output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSuggestion {
    pub entry_ids: Vec<String>,
    pub shared_tags: Vec<String>,
    pub cluster_size: usize,
}

/// Curation report: low-utility entries, merge candidates, and stale entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurationReport {
    pub total_entries: usize,
    pub low_utility: Vec<LowUtilityEntry>,
    pub merge_suggestions: Vec<MergeSuggestion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stale_entries: Vec<StaleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowUtilityEntry {
    pub id: String,
    pub retrieval_count: u32,
    pub task_total_after: u32,
    pub utility_ratio: Option<f64>,
    pub context: String,
}

/// An entry that has decayed below the staleness threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleEntry {
    pub id: String,
    pub age_days: i64,
    pub decay_score: f64,
    pub retrieval_count: u32,
    pub context: String,
}
