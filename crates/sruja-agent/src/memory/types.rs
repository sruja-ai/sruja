//! Type definitions for agentic memory.
//!
//! Re-exports the canonical learning types from `sruja-graph` and defines
//! agent-specific curation types.

use serde::{Deserialize, Serialize};

// Re-export shared types from sruja-graph (single source of truth).
pub use sruja_graph::learning::{
    ExperimentOutcome, LearningEntry, LearningKind, LearningPatch, MemoryError,
};

pub use sruja_graph::learning::generate_entry_id;

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
