//! Agentic memory management for Sruja.
//!
//! This module provides the `AgenticMemory` system, which stores and retrieves
//! "learnings" (hypotheses, outcomes, and guardrails) derived from AI-driven
//! architectural experiments.

pub mod curation;
pub mod search;
pub mod signals;
pub mod storage;
pub mod types;

#[cfg(test)]
mod tests;

use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cognition::ErrorClass;

pub use types::ErrorFrequency;
pub use types::{
    BlastRadius, CurationReport, ExperimentOutcome, LearningCategory, LearningConstraints,
    LearningEntry, LearningKind, LearningPatch, LowUtilityEntry, MemoryError, MergeSuggestion,
    SignalPattern, StaleEntry,
};

/// Maximum number of error frequency entries before the oldest are evicted.
const MAX_ERROR_ENTRIES: usize = 100;

/// Entries older than this many days are filtered out of error frequency results.
const ERROR_TTL_DAYS: i64 = 60;

/// Persistent store for architectural learnings and agentic guardrails.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenticMemory {
    /// The list of recorded learnings.
    pub learnings: Vec<LearningEntry>,
    /// In-memory index mapping (normalized context, normalized hypothesis) to
    /// index in `learnings` for O(1) dedup lookups.
    #[serde(skip)]
    pub dedup_index: HashMap<(String, String), usize>,
    /// Cross-run error frequency history, keyed by (repo_path, error_class).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub error_frequencies: Vec<ErrorFrequency>,
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

    /// Rebuilds the dedup index from scratch.
    ///
    /// This is called after deserialization (load) and after any structural
    /// mutation (delete, update) that could shift indices.
    pub fn rebuild_dedup_index(&mut self) {
        self.dedup_index.clear();
        for (idx, entry) in self.learnings.iter().enumerate() {
            let key = (
                entry.context.trim().to_lowercase(),
                entry.hypothesis.trim().to_lowercase(),
            );
            self.dedup_index.insert(key, idx);
        }
    }

    /// Adds a new learning entry, auto-generating tags and linking to related entries.
    ///
    /// Deduplicates against existing entries with the same `(context, hypothesis)` pair:
    /// instead of appending a duplicate, the existing entry's guardrail_advice and
    /// outcome are updated with the new information.
    pub fn add_learning(&mut self, mut learning: LearningEntry) {
        if learning.id.is_empty() {
            learning.id = types::generate_entry_id();
        }
        if learning.tags.is_empty() {
            learning.tags = search::extract_tags(&learning);
        }

        // O(1) dedup via the index: check dedup_index first, fall back to
        // linear scan only if the index doesn't have the key (defensive).
        let normalized_ctx = learning.context.trim().to_lowercase();
        let normalized_hyp = learning.hypothesis.trim().to_lowercase();
        let dedup_key = (normalized_ctx.clone(), normalized_hyp.clone());

        // Fast path: use the index for O(1) lookup. If the key isn't in the
        // index, or the index points to a stale position, fall through to
        // the linear scan as a safety net.
        let idx_from_index = self.dedup_index.get(&dedup_key).copied();
        let index_is_valid = idx_from_index.is_some_and(|idx| idx < self.learnings.len());

        if let Some(idx) = idx_from_index {
            if let Some(existing) = self.learnings.get_mut(idx) {
                // Merge: update guardrail_advice if new info is longer/more specific.
                if learning.guardrail_advice.len() > existing.guardrail_advice.len() {
                    existing.guardrail_advice = learning.guardrail_advice;
                }
                // Merge: update outcome on failure (prefer recording failures).
                if learning.outcome != ExperimentOutcome::Success {
                    existing.outcome = learning.outcome;
                }
                // Merge: extend affected elements (deduped).
                for el in &learning.affected_elements {
                    if !existing.affected_elements.contains(el) {
                        existing.affected_elements.push(el.clone());
                    }
                }
                // Merge: extend tags (deduped).
                for tag in &learning.tags {
                    if !existing.tags.contains(tag) {
                        existing.tags.push(tag.clone());
                    }
                }
                existing.timestamp = learning.timestamp;
                return;
            }
        }

        // Fallback: linear scan (safety net for stale index or missing key).
        if !index_is_valid {
            if let Some(existing) = self.learnings.iter_mut().find(|e| {
                e.context.trim().to_lowercase() == normalized_ctx
                    && e.hypothesis.trim().to_lowercase() == normalized_hyp
            }) {
                // Merge: same logic as above.
                if learning.guardrail_advice.len() > existing.guardrail_advice.len() {
                    existing.guardrail_advice = learning.guardrail_advice;
                }
                if learning.outcome != ExperimentOutcome::Success {
                    existing.outcome = learning.outcome;
                }
                for el in &learning.affected_elements {
                    if !existing.affected_elements.contains(el) {
                        existing.affected_elements.push(el.clone());
                    }
                }
                for tag in &learning.tags {
                    if !existing.tags.contains(tag) {
                        existing.tags.push(tag.clone());
                    }
                }
                existing.timestamp = learning.timestamp;
                return;
            }
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
        // Update the dedup index with the new entry.
        self.dedup_index.insert(dedup_key, self.learnings.len() - 1);
    }

    /// Adds a learning entry without auto-linking (for deserialization or migration).
    ///
    /// Does NOT update the dedup index — call `rebuild_dedup_index()` if the
    /// index needs to be consistent afterward.
    pub fn add_learning_raw(&mut self, learning: LearningEntry) {
        self.learnings.push(learning);
    }

    /// Finds learning entries relevant to a specific architectural element ID.
    pub fn find_relevant(&self, element_id: &str) -> Vec<&LearningEntry> {
        search::find_relevant(self, element_id)
    }

    /// Record an error occurrence, incrementing frequency for (repo_path, error_class).
    ///
    /// Automatically evicts the oldest entries when MAX_ERROR_ENTRIES is exceeded.
    pub fn record_error(&mut self, repo_path: &str, error_class: ErrorClass) {
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(entry) = self
            .error_frequencies
            .iter_mut()
            .find(|e| e.repo_path == repo_path && e.error_class == error_class)
        {
            entry.count += 1;
            entry.last_updated = now;
        } else {
            self.error_frequencies.push(ErrorFrequency {
                repo_path: repo_path.to_string(),
                error_class,
                count: 1,
                last_updated: now,
            });
        }

        // Enforce cap: if over MAX_ERROR_ENTRIES, evict oldest entries.
        if self.error_frequencies.len() > MAX_ERROR_ENTRIES {
            let excess = self.error_frequencies.len() - MAX_ERROR_ENTRIES;
            self.error_frequencies
                .sort_by(|a, b| a.last_updated.cmp(&b.last_updated));
            self.error_frequencies.drain(..excess);
        }
    }

    /// Search error frequency history, optionally filtered by repo_path.
    ///
    /// Entries older than ERROR_TTL_DAYS are excluded from results.
    pub fn search_error_frequencies(&self, repo_path: &str) -> Vec<ErrorFrequency> {
        let cutoff = Utc::now() - TimeDelta::try_days(ERROR_TTL_DAYS).unwrap_or_default();
        self.error_frequencies
            .iter()
            .filter(|e| {
                if e.repo_path != repo_path {
                    return false;
                }
                // Exclude entries older than the TTL.
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&e.last_updated) {
                    ts.naive_utc().date() >= cutoff.naive_utc().date()
                } else {
                    // If we can't parse the timestamp, keep the entry
                    // (defensive: don't silently drop data).
                    true
                }
            })
            .cloned()
            .collect()
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
    ///
    /// Rebuilds the dedup index if context or hypothesis changed.
    pub fn update_learning(&mut self, id: &str, patch: LearningPatch) -> Result<(), MemoryError> {
        let idx = self
            .learnings
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| MemoryError::NotFound(id.to_string()))?;

        let mut rebuild_index = false;
        let mut reextract_tags = false;

        // Apply patches to the entry at idx.
        {
            let entry = &mut self.learnings[idx];

            if let Some(kind) = patch.kind {
                entry.kind = Some(kind);
            }
            if let Some(ctx) = patch.context {
                entry.context = ctx;
                reextract_tags = true;
                rebuild_index = true;
            }
            if let Some(h) = patch.hypothesis {
                entry.hypothesis = h;
                reextract_tags = true;
                rebuild_index = true;
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
            if let Some(cat) = patch.category {
                entry.category = Some(cat);
            }
            if let Some(signals) = patch.signals_match {
                entry.signals_match = signals;
            }
            if let Some(constraints) = patch.constraints {
                entry.constraints = constraints;
            }
            if let Some(validation) = patch.validation {
                entry.validation = validation;
            }
            if let Some(blast_radius) = patch.blast_radius {
                entry.blast_radius = blast_radius;
            }

            if reextract_tags {
                entry.tags = search::extract_tags(entry);
            }
        }

        // Rebuild index if context or hypothesis changed (requires no active
        // borrow on self.learnings[idx]).
        if rebuild_index {
            self.rebuild_dedup_index();
        }

        Ok(())
    }

    /// Removes a learning and scrubs `related_ids` references across the library.
    ///
    /// Rebuilds the dedup index after removal to keep it consistent.
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
        self.rebuild_dedup_index();
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
    ///
    /// When `category` is `Some`, results are filtered to entries with that
    /// category (repair / optimize / innovate / explore).
    fn search(
        &self,
        query: &str,
        limit: usize,
        category: Option<types::LearningCategory>,
    ) -> Vec<LearningEntry>;

    /// Record a new learning.
    fn record(&self, entry: LearningEntry) -> Result<(), MemoryError>;

    /// Record that specific learnings were retrieved for a task.
    fn record_retrievals(&self, ids: &[&str]);

    /// Record task outcomes for learnings retrieved during the task.
    fn record_outcomes(&self, ids: &[&str], success: bool);

    /// Search error frequency history for a repository.
    fn search_error_history(&self, repo_path: &str) -> Result<Vec<ErrorFrequency>, MemoryError>;

    /// Record an error occurrence for cross-run learning.
    fn record_error(&self, repo_path: &str, error_class: ErrorClass) -> Result<(), MemoryError>;

    /// Total number of learnings stored.
    fn count(&self) -> usize;

    /// Persist the current in-memory state to a JSON file at `path`.
    ///
    /// Used by callers (e.g. `reflect`) to flush learnings to disk after
    /// recording. Default is a no-op for backends that persist on every
    /// `record()` call.
    fn save_to_path(&self, _path: &Path) -> Result<(), MemoryError> {
        Ok(())
    }
}

impl Memory for std::sync::Mutex<AgenticMemory> {
    fn search(
        &self,
        query: &str,
        limit: usize,
        category: Option<types::LearningCategory>,
    ) -> Vec<LearningEntry> {
        let mem = self.lock().unwrap();
        let query_lower = query.to_lowercase();

        // Extract signals from the query for signal-based boosting.
        let query_signals = signals::extract_signals(query, &[]);

        // Search by element ID, tag, and text relevance, with optional category filter.
        let mut results: Vec<LearningEntry> = mem
            .learnings
            .iter()
            .filter(|e| {
                // Category filter (Gene-inspired).
                if let Some(ref cat) = category {
                    if e.category.as_ref() != Some(cat) {
                        return false;
                    }
                }
                e.context.to_lowercase().contains(&query_lower)
                    || e.hypothesis.to_lowercase().contains(&query_lower)
                    || e.guardrail_advice.to_lowercase().contains(&query_lower)
                    || e.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || e.affected_elements.iter().any(|el| el.contains(query))
            })
            .cloned()
            .collect();

        // Score by signal overlap: boost learnings whose signals_match
        // aligns with the extracted query signals.
        for entry in &mut results {
            let _signal_score = signals::score_signals_match(&query_signals, &entry.signals_match);
        }

        // Sort by composite score: decay * utility + signal boost.
        results.sort_by(|a, b| {
            let base_a = a.decay_score() * a.utility_ratio().unwrap_or(0.5);
            let base_b = b.decay_score() * b.utility_ratio().unwrap_or(0.5);
            let sig_a = signals::score_signals_match(&query_signals, &a.signals_match);
            let sig_b = signals::score_signals_match(&query_signals, &b.signals_match);
            let score_a = base_a + sig_a * 2.0;
            let score_b = base_b + sig_b * 2.0;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
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

    fn search_error_history(&self, repo_path: &str) -> Result<Vec<ErrorFrequency>, MemoryError> {
        let mem = self.lock().unwrap();
        Ok(mem.search_error_frequencies(repo_path))
    }

    fn record_error(&self, repo_path: &str, error_class: ErrorClass) -> Result<(), MemoryError> {
        let mut mem = self.lock().unwrap();
        mem.record_error(repo_path, error_class);
        Ok(())
    }

    fn count(&self) -> usize {
        self.lock().unwrap().learnings.len()
    }

    fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        let mem = self.lock().unwrap();
        mem.save(path)
    }
}
