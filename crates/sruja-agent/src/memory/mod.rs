//! Agentic memory management for Sruja.
//!
//! This module provides the `AgenticMemory` system, which stores and retrieves
//! "learnings" (hypotheses, outcomes, and guardrails) derived from AI-driven
//! architectural experiments.

pub mod curation;
pub mod search;
pub mod storage;
pub mod types;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use types::{
    CurationReport, ExperimentOutcome, LearningEntry, LearningKind, LearningPatch, LowUtilityEntry,
    MemoryError, MergeSuggestion, StaleEntry,
};

/// Persistent store for architectural learnings and agentic guardrails.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenticMemory {
    /// The list of recorded learnings.
    pub learnings: Vec<LearningEntry>,
}

impl AgenticMemory {
    /// Loads agentic memory from the specified repository root.
    pub fn load(repo_root: &Path) -> Result<Self, MemoryError> {
        storage::load(repo_root)
    }

    /// Loads agentic memory from a specific path.
    pub fn load_from_path(path: &Path) -> Result<Self, MemoryError> {
        storage::load_from_path(path)
    }

    /// Saves the current memory to the specified repository root.
    pub fn save(&self, repo_root: &Path) -> Result<(), MemoryError> {
        storage::save(self, repo_root)
    }

    /// Saves the current memory to a specific path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        storage::save_to_path(self, path)
    }

    /// Clears the agentic memory for the specified repository.
    pub fn clear(repo_root: &Path) -> Result<(), MemoryError> {
        storage::clear(repo_root)
    }

    /// Checks if the agentic memory file exists for the given repository.
    pub fn exists(repo_root: &Path) -> bool {
        storage::exists(repo_root)
    }

    /// Returns the path to the agentic memory file.
    pub fn get_path(repo_root: &Path) -> PathBuf {
        storage::get_path(repo_root)
    }

    /// Adds a new learning entry, auto-generating tags and linking to related entries.
    pub fn add_learning(&mut self, mut learning: LearningEntry) {
        if learning.id.is_empty() {
            learning.id = types::generate_entry_id();
        }
        if learning.tags.is_empty() {
            learning.tags = search::extract_tags(&learning);
        }

        let new_id = learning.id.clone();
        let related = search::find_related_indices(self, &learning);

        for &idx in &related {
            let existing_id = self.learnings[idx].id.clone();
            if !learning.related_ids.contains(&existing_id) {
                learning.related_ids.push(existing_id);
            }
            if !self.learnings[idx].related_ids.contains(&new_id) {
                self.learnings[idx].related_ids.push(new_id.clone());
            }
        }

        self.learnings.push(learning);
    }

    /// Adds a learning entry without auto-linking (for deserialization or migration).
    pub fn add_learning_raw(&mut self, learning: LearningEntry) {
        self.learnings.push(learning);
    }

    /// Finds learning entries relevant to a specific architectural element ID.
    pub fn find_relevant(&self, element_id: &str) -> Vec<&LearningEntry> {
        search::find_relevant(self, element_id)
    }

    /// Finds all relevant learnings and increments `retrieval_count`.
    pub fn touch_relevant_learnings(&mut self, element_id: &str) -> Vec<String> {
        let ids: Vec<String> = self
            .find_relevant(element_id)
            .into_iter()
            .map(|e| e.id.clone())
            .collect();
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        self.record_retrievals(&refs);
        ids
    }

    /// Records task outcome for learnings retrieved earlier in the same task (caller saves).
    pub fn finish_task_learnings(&mut self, ids: &[String], success: bool) {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        self.record_task_outcomes(&refs, success);
    }

    /// Increments `retrieval_count` for each id (call when learnings are surfaced for a task).
    pub fn record_retrievals(&mut self, ids: &[&str]) {
        for id in ids {
            if let Some(entry) = self.learnings.iter_mut().find(|e| e.id == *id) {
                entry.retrieval_count = entry.retrieval_count.saturating_add(1);
            }
        }
    }

    /// Records task outcome for entries that were retrieved during that task.
    pub fn record_task_outcomes(&mut self, ids: &[&str], success: bool) {
        for id in ids {
            if let Some(entry) = self.learnings.iter_mut().find(|e| e.id == *id) {
                entry.task_total_after = entry.task_total_after.saturating_add(1);
                if success {
                    entry.task_success_after = entry.task_success_after.saturating_add(1);
                }
            }
        }
    }

    /// Updates an existing learning by id. Re-extracts tags when text fields change.
    pub fn update_learning(&mut self, id: &str, patch: LearningPatch) -> Result<(), MemoryError> {
        let idx = self
            .learnings
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| MemoryError::NotFound(id.to_string()))?;

        let entry = &mut self.learnings[idx];
        let mut reextract_tags = false;

        if let Some(kind) = patch.kind {
            entry.kind = Some(kind);
        }
        if let Some(ctx) = patch.context {
            entry.context = ctx;
            reextract_tags = true;
        }
        if let Some(h) = patch.hypothesis {
            entry.hypothesis = h;
            reextract_tags = true;
        }
        if let Some(outcome) = patch.outcome {
            entry.outcome = outcome;
        }
        if let Some(reason) = patch.reason {
            entry.reason = reason;
        }
        if let Some(guardrail) = patch.guardrail_advice {
            entry.guardrail_advice = guardrail;
            reextract_tags = true;
        }
        if let Some(elements) = patch.affected_elements {
            entry.affected_elements = elements;
        }
        if let Some(refs) = patch.evidence_refs {
            entry.evidence_refs = refs;
        }
        if let Some(conf) = patch.confidence {
            entry.confidence = conf;
        }
        if let Some(tags) = patch.tags {
            entry.tags = tags;
            reextract_tags = false;
        }
        if let Some(hitl) = patch.hitl_kind {
            entry.hitl_kind = hitl;
        }

        if reextract_tags {
            entry.tags = search::extract_tags(entry);
        }

        Ok(())
    }

    /// Removes a learning and scrubs `related_ids` references across the library.
    pub fn delete_learning(&mut self, id: &str) -> Result<LearningEntry, MemoryError> {
        let idx = self
            .learnings
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| MemoryError::NotFound(id.to_string()))?;
        let removed = self.learnings.remove(idx);
        for entry in &mut self.learnings {
            entry.related_ids.retain(|rid| rid != id);
        }
        Ok(removed)
    }

    /// Merges multiple entries into one, preserving links and utility counters.
    pub fn merge_learnings(
        &mut self,
        ids: &[String],
        merged: LearningEntry,
    ) -> Result<String, MemoryError> {
        curation::merge_learnings(self, ids, merged)
    }

    /// Entries with many retrievals but low post-retrieval success (deletion candidates).
    pub fn low_utility_entries(
        &self,
        min_retrievals: u32,
        max_utility_ratio: f64,
    ) -> Vec<&LearningEntry> {
        curation::low_utility_entries(self, min_retrievals, max_utility_ratio)
    }

    /// Builds a curation report for `sruja agent curate`.
    pub fn curation_report(&self) -> CurationReport {
        curation::curation_report(self)
    }

    /// Archives entries that have decayed below the staleness threshold.
    pub fn auto_archive_stale(
        &mut self,
        decay_threshold: f64,
        min_age_days: i64,
    ) -> Vec<LearningEntry> {
        curation::auto_archive_stale(self, decay_threshold, min_age_days)
    }

    /// Returns all entries in the same thematic cluster as the given entry ID.
    pub fn find_cluster(&self, entry_id: &str) -> Vec<&LearningEntry> {
        search::find_cluster(self, entry_id)
    }

    /// Returns all distinct thematic tags across all entries.
    pub fn all_tags(&self) -> Vec<String> {
        search::all_tags(self)
    }

    /// Returns all entries matching a given tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&LearningEntry> {
        search::find_by_tag(self, tag)
    }
}

// ---------------------------------------------------------------------------
// Memory trait — pluggable backend abstraction
// ---------------------------------------------------------------------------

/// Trait abstracting long-term memory for the agent.
///
/// Implemented by [`AgenticMemory`] (JSON file) and optionally by SQLite/FTS5
/// backends. The cognition loop uses this trait — not the concrete type.
pub trait Memory: Send + Sync {
    /// Search for learnings relevant to a query, returning at most `limit` results.
    fn search(&self, query: &str, limit: usize) -> Vec<LearningEntry>;

    /// Record a new learning.
    fn record(&self, entry: LearningEntry) -> Result<(), MemoryError>;

    /// Record that specific learnings were retrieved for a task.
    fn record_retrievals(&self, ids: &[&str]);

    /// Record task outcomes for learnings retrieved during the task.
    fn record_outcomes(&self, ids: &[&str], success: bool);

    /// Total number of learnings stored.
    fn count(&self) -> usize;
}

impl Memory for std::sync::Mutex<AgenticMemory> {
    fn search(&self, query: &str, limit: usize) -> Vec<LearningEntry> {
        let mem = self.lock().unwrap();
        // Search by element ID, tag, and text relevance.
        let mut results: Vec<LearningEntry> = mem
            .learnings
            .iter()
            .filter(|e| {
                e.context.to_lowercase().contains(&query.to_lowercase())
                    || e.hypothesis.to_lowercase().contains(&query.to_lowercase())
                    || e.guardrail_advice
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                    || e.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query.to_lowercase()))
                    || e.affected_elements.iter().any(|el| el.contains(query))
            })
            .cloned()
            .collect();

        // Sort by utility (decay_score * utility_ratio) — best first.
        results.sort_by(|a, b| {
            let sa = a.decay_score() * a.utility_ratio().unwrap_or(0.5);
            let sb = b.decay_score() * b.utility_ratio().unwrap_or(0.5);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(limit);
        results
    }

    fn record(&self, entry: LearningEntry) -> Result<(), MemoryError> {
        let mut mem = self.lock().unwrap();
        mem.add_learning(entry);
        Ok(())
    }

    fn record_retrievals(&self, ids: &[&str]) {
        let mut mem = self.lock().unwrap();
        mem.record_retrievals(ids);
    }

    fn record_outcomes(&self, ids: &[&str], success: bool) {
        let mut mem = self.lock().unwrap();
        mem.record_task_outcomes(ids, success);
    }

    fn count(&self) -> usize {
        self.lock().unwrap().learnings.len()
    }
}
