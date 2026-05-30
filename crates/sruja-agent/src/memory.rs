//! Agentic memory management for Sruja.
//!
//! This module provides the `AgenticMemory` system, which stores and retrieves
//! "learnings" (hypotheses, outcomes, and guardrails) derived from AI-driven
//! architectural experiments.

use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during memory operations.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// An I/O error occurred while reading or writing the memory file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// No learning entry exists for the given id.
    #[error("learning not found: {0}")]
    NotFound(String),
    /// Merge or update requested with invalid or duplicate ids.
    #[error("invalid learning ids: {0}")]
    InvalidIds(String),
}

/// The outcome of an architectural experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentOutcome {
    /// The experiment was successful (e.g., the change improved the architecture).
    #[serde(rename = "success")]
    Success,
    /// The experiment failed (e.g., the change caused a regression or drift).
    #[serde(rename = "failed")]
    Failed,
}

/// A single entry in the agentic memory representing a learned architectural lesson.
///
/// Inspired by the Zettelkasten method: each entry is an atomic note with
/// auto-generated tags and bidirectional links to related entries, enabling
/// thematic clustering and associative retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEntry {
    /// Stable identifier for cross-referencing between entries.
    #[serde(default = "generate_entry_id")]
    pub id: String,
    /// Optional classification for the type of learning.
    ///
    /// This is intentionally optional for backward compatibility with existing
    /// `.sruja/agent_memory.json` files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<LearningKind>,
    /// When this learning was recorded.
    pub timestamp: DateTime<Utc>,
    /// Optional run ID that produced this learning (for traceability).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Optional repository identifier/path (useful for federated multi-repo memory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Optional selector string that led to this learning (file/element_id/query/diff).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// The context in which the experiment was performed (e.g., "Refactoring API layer").
    pub context: String,
    /// The hypothesis being tested (e.g., "Moving logic to sruja-engine will reduce CLI bloat").
    pub hypothesis: String,
    /// Whether the experiment succeeded or failed.
    pub outcome: ExperimentOutcome,
    /// The reason for the outcome (optional).
    pub reason: Option<String>,
    /// Actionable advice for future agents to avoid repeating mistakes or to replicate success.
    pub guardrail_advice: String,
    /// IDs of architectural elements affected by this experiment.
    pub affected_elements: Vec<String>,
    /// Optional evidence references (files, ADR IDs, commands, etc.) that ground this learning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Optional confidence label (high/medium/low) aligned with Sruja task confidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    /// Auto-generated thematic tags extracted from context, hypothesis, and guardrail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Human-in-the-loop classification: precedent, exception, correction, guardrail (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<String>,
    /// IDs of related learning entries (bidirectional Zettelkasten links).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_ids: Vec<String>,
    /// How often this entry was retrieved for a task (focus, agent run, MCP).
    #[serde(default)]
    pub retrieval_count: u32,
    /// Tasks that succeeded after this entry was retrieved.
    #[serde(default)]
    pub task_success_after: u32,
    /// Total tasks where this entry was retrieved (denominator for utility).
    #[serde(default)]
    pub task_total_after: u32,
}

/// Partial update for an existing [`LearningEntry`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<LearningKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hypothesis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<ExperimentOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrail_advice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_elements: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<Option<String>>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningKind {
    /// A guardrail: “what not to try again”.
    Guardrail,
    /// A playbook: “what worked, do this again”.
    Playbook,
    /// An invariant: “must always hold”.
    Invariant,
}

fn generate_entry_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("learn_{}_{}", chrono::Utc::now().timestamp_millis(), seq)
}

impl LearningEntry {
    /// Success rate after retrieval (`None` if never retrieved for a completed task).
    pub fn utility_ratio(&self) -> Option<f64> {
        if self.task_total_after == 0 {
            None
        } else {
            Some(self.task_success_after as f64 / self.task_total_after as f64)
        }
    }

    /// Decay score in [0.0, 1.0] based on age and retrieval frequency.
    ///
    /// Entries that are old and rarely retrieved decay toward 0.0.
    /// Recent or frequently-retrieved entries stay near 1.0.
    /// Uses a half-life of 90 days, boosted by retrieval count.
    pub fn decay_score(&self) -> f64 {
        let age = Utc::now().signed_duration_since(self.timestamp);
        let age_days = age.num_days().max(0) as f64;
        let half_life = 90.0_f64;
        // Retrieval frequency extends effective freshness.
        let retrieval_boost = (self.retrieval_count as f64).ln_1p() * 15.0;
        let effective_age = (age_days - retrieval_boost).max(0.0);
        2.0_f64.powf(-effective_age / half_life)
    }

    /// Age of this entry in days.
    pub fn age_days(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.timestamp)
            .num_days()
            .max(0)
    }

    /// Checks if this learning entry is relevant to a specific architectural element.
    pub fn is_relevant_to(&self, element_id: &str) -> bool {
        let element_id_lower = element_id.to_lowercase();

        // 1. Check affected elements (direct match or hierarchy match)
        let element_match = self.affected_elements.iter().any(|e| {
            e == element_id
                || element_id.starts_with(&format!("{}.", e))
                || e.starts_with(&format!("{}.", element_id))
        });

        if element_match {
            return true;
        }

        // 2. Check context for keywords
        self.context.to_lowercase().contains(&element_id_lower)
    }
}

/// Persistent store for architectural learnings and agentic guardrails.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenticMemory {
    /// The list of recorded learnings.
    pub learnings: Vec<LearningEntry>,
}

impl AgenticMemory {
    /// Loads agentic memory from the specified repository root.
    ///
    /// Memory is stored in `.sruja/agent_memory.json`. If the file does not exist,
    /// an empty memory is returned.
    pub fn load(repo_root: &Path) -> Result<Self, MemoryError> {
        let path = Self::get_path(repo_root);
        Self::load_from_path(&path)
    }

    /// Loads agentic memory from a specific path.
    pub fn load_from_path(path: &Path) -> Result<Self, MemoryError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = File::open(path)?;
        file.lock_shared()?;
        let mut content = String::new();
        let mut reader = std::io::BufReader::new(&file);
        reader.read_to_string(&mut content)?;
        file.unlock()?;
        let memory = serde_json::from_str(&content)?;
        Ok(memory)
    }

    /// Saves the current memory to the specified repository root.
    ///
    /// This will create the `.sruja` directory if it doesn't exist.
    pub fn save(&self, repo_root: &Path) -> Result<(), MemoryError> {
        let path = Self::get_path(repo_root);
        self.save_to_path(&path)
    }

    /// Saves the current memory to a specific path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        file.lock_exclusive()?;
        let content = serde_json::to_string_pretty(self)?;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        {
            let mut writer = std::io::BufWriter::new(&mut file);
            writer.write_all(content.as_bytes())?;
            writer.flush()?;
        }
        file.unlock()?;
        Ok(())
    }

    /// Clears the agentic memory for the specified repository.
    pub fn clear(repo_root: &Path) -> Result<(), MemoryError> {
        let path = Self::get_path(repo_root);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Checks if the agentic memory file exists for the given repository.
    pub fn exists(repo_root: &Path) -> bool {
        Self::get_path(repo_root).exists()
    }

    /// Adds a new learning entry, auto-generating tags and linking to related entries.
    ///
    /// Tags are extracted from the entry's context, hypothesis, and guardrail text.
    /// Bidirectional links are forged to existing entries that share affected elements,
    /// tags, or contextual keywords -- mirroring Zettelkasten's associative linking.
    pub fn add_learning(&mut self, mut learning: LearningEntry) {
        if learning.id.is_empty() {
            learning.id = generate_entry_id();
        }
        if learning.tags.is_empty() {
            learning.tags = Self::extract_tags(&learning);
        }

        let new_id = learning.id.clone();
        let related = self.find_related_indices(&learning);

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
    ///
    /// Relevancy is determined by:
    /// 1. Direct match in `affected_elements`.
    /// 2. Parent match (element_id starts with an affected element).
    /// 3. String match in the `context` field.
    pub fn find_relevant(&self, element_id: &str) -> Vec<&LearningEntry> {
        self.learnings
            .iter()
            .filter(|l| l.is_relevant_to(element_id))
            .collect()
    }

    /// Finds all relevant learnings and increments `retrieval_count`.
    ///
    /// Prefer surfacing via focus (`surface_agent_learnings` / briefing `surfaced_learning_ids`)
    /// so counters match what was actually injected into context.
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
            entry.tags = Self::extract_tags(entry);
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
        mut merged: LearningEntry,
    ) -> Result<String, MemoryError> {
        if ids.len() < 2 {
            return Err(MemoryError::InvalidIds(
                "merge requires at least two entry ids".to_string(),
            ));
        }

        let unique: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
        if unique.len() != ids.len() {
            return Err(MemoryError::InvalidIds(
                "duplicate ids in merge request".to_string(),
            ));
        }

        let mut to_remove: Vec<usize> = Vec::new();
        let mut tags = std::collections::HashSet::new();
        let mut elements = std::collections::HashSet::new();
        let mut evidence = std::collections::HashSet::new();
        let mut related = std::collections::HashSet::new();
        let mut retrieval_count: u32 = 0;

        for (idx, entry) in self.learnings.iter().enumerate() {
            if ids.contains(&entry.id) {
                to_remove.push(idx);
                tags.extend(entry.tags.iter().cloned());
                elements.extend(entry.affected_elements.iter().cloned());
                evidence.extend(entry.evidence_refs.iter().cloned());
                related.extend(entry.related_ids.iter().cloned());
                retrieval_count = retrieval_count.saturating_add(entry.retrieval_count);
                // Task outcomes reset on merge — merged text is a new editorial artifact.
            }
        }

        if to_remove.len() != ids.len() {
            let missing: Vec<_> = ids
                .iter()
                .filter(|id| !self.learnings.iter().any(|e| e.id == **id))
                .cloned()
                .collect();
            return Err(MemoryError::NotFound(missing.join(", ")));
        }

        for id in ids {
            related.remove(id.as_str());
        }

        if merged.id.is_empty() {
            merged.id = generate_entry_id();
        }
        let new_id = merged.id.clone();

        merged.tags = if merged.tags.is_empty() {
            tags.into_iter().collect()
        } else {
            let mut t: std::collections::HashSet<_> = merged.tags.into_iter().collect();
            t.extend(tags);
            t.into_iter().collect()
        };
        if merged.affected_elements.is_empty() {
            merged.affected_elements = elements.into_iter().collect();
        } else {
            let mut e: std::collections::HashSet<_> =
                merged.affected_elements.into_iter().collect();
            e.extend(elements);
            merged.affected_elements = e.into_iter().collect();
        }
        if merged.evidence_refs.is_empty() {
            merged.evidence_refs = evidence.into_iter().collect();
        } else {
            let mut e: std::collections::HashSet<_> = merged.evidence_refs.into_iter().collect();
            e.extend(evidence);
            merged.evidence_refs = e.into_iter().collect();
        }
        merged.related_ids = related.into_iter().collect();
        merged.retrieval_count = retrieval_count;
        merged.task_success_after = 0;
        merged.task_total_after = 0;

        for idx in to_remove.into_iter().rev() {
            self.learnings.remove(idx);
        }

        for entry in &mut self.learnings {
            for old_id in ids {
                if let Some(pos) = entry.related_ids.iter().position(|r| r == old_id) {
                    entry.related_ids[pos] = new_id.clone();
                }
            }
            entry
                .related_ids
                .retain(|rid| rid != &new_id || entry.id == new_id);
        }

        self.add_learning(merged);
        Ok(new_id)
    }

    /// Entries with many retrievals but low post-retrieval success (deletion candidates).
    pub fn low_utility_entries(
        &self,
        min_retrievals: u32,
        max_utility_ratio: f64,
    ) -> Vec<&LearningEntry> {
        self.learnings
            .iter()
            .filter(|e| {
                e.retrieval_count >= min_retrievals
                    && e.task_total_after > 0
                    && e.utility_ratio().is_some_and(|r| r < max_utility_ratio)
            })
            .collect()
    }

    /// Builds a curation report for `sruja agent curate`.
    pub fn curation_report(&self) -> CurationReport {
        let low_utility = self
            .low_utility_entries(2, 0.4)
            .into_iter()
            .map(|e| LowUtilityEntry {
                id: e.id.clone(),
                retrieval_count: e.retrieval_count,
                task_total_after: e.task_total_after,
                utility_ratio: e.utility_ratio(),
                context: e.context.clone(),
            })
            .collect();

        let stale_threshold = 0.15_f64;
        let stale_entries: Vec<StaleEntry> = self
            .learnings
            .iter()
            .filter(|e| {
                let score = e.decay_score();
                score < stale_threshold && e.age_days() > 30
            })
            .map(|e| StaleEntry {
                id: e.id.clone(),
                age_days: e.age_days(),
                decay_score: e.decay_score(),
                retrieval_count: e.retrieval_count,
                context: e.context.clone(),
            })
            .collect();

        let mut merge_suggestions = Vec::new();
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

        for entry in &self.learnings {
            if visited.contains(&entry.id) {
                continue;
            }
            let cluster = self.find_cluster(&entry.id);
            if cluster.len() < 2 {
                continue;
            }
            for e in &cluster {
                visited.insert(e.id.clone());
            }
            let ids: Vec<String> = cluster.iter().map(|e| e.id.clone()).collect();
            let mut tag_counts: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for e in &cluster {
                for t in &e.tags {
                    *tag_counts.entry(t.as_str()).or_default() += 1;
                }
            }
            let shared_tags: Vec<String> = tag_counts
                .into_iter()
                .filter(|(_, c)| *c >= 2)
                .map(|(t, _)| t.to_string())
                .collect();
            merge_suggestions.push(MergeSuggestion {
                entry_ids: ids,
                shared_tags,
                cluster_size: cluster.len(),
            });
        }

        CurationReport {
            total_entries: self.learnings.len(),
            low_utility,
            merge_suggestions,
            stale_entries,
        }
    }

    /// Archives entries that have decayed below the staleness threshold.
    ///
    /// Returns the archived entries. Invariant entries are never archived.
    pub fn auto_archive_stale(
        &mut self,
        decay_threshold: f64,
        min_age_days: i64,
    ) -> Vec<LearningEntry> {
        let to_archive: Vec<String> = self
            .learnings
            .iter()
            .filter(|e| {
                e.decay_score() < decay_threshold
                    && e.age_days() > min_age_days
                    && e.kind != Some(LearningKind::Invariant)
            })
            .map(|e| e.id.clone())
            .collect();

        let mut archived = Vec::new();
        for id in &to_archive {
            if let Ok(entry) = self.delete_learning(id) {
                archived.push(entry);
            }
        }
        archived
    }

    pub fn get_path(repo_root: &Path) -> PathBuf {
        repo_root.join(".sruja").join("agent_memory.json")
    }

    /// Returns all entries in the same thematic cluster as the given entry ID.
    ///
    /// Performs a transitive walk through `related_ids` links, returning
    /// the full connected component -- analogous to opening a Zettelkasten "box."
    pub fn find_cluster(&self, entry_id: &str) -> Vec<&LearningEntry> {
        let index_map: std::collections::HashMap<&str, usize> = self
            .learnings
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.as_str(), i))
            .collect();

        let Some(&start) = index_map.get(entry_id) else {
            return Vec::new();
        };

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(idx) = queue.pop_front() {
            for related_id in &self.learnings[idx].related_ids {
                if let Some(&ri) = index_map.get(related_id.as_str()) {
                    if visited.insert(ri) {
                        queue.push_back(ri);
                    }
                }
            }
        }

        visited.iter().map(|&i| &self.learnings[i]).collect()
    }

    /// Returns all distinct thematic tags across all entries.
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();
        for entry in &self.learnings {
            for tag in &entry.tags {
                tags.insert(tag.clone());
            }
        }
        let mut sorted: Vec<String> = tags.into_iter().collect();
        sorted.sort();
        sorted
    }

    /// Returns all entries matching a given tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&LearningEntry> {
        let tag_lower = tag.to_lowercase();
        self.learnings
            .iter()
            .filter(|e| e.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    /// Extracts thematic tags from an entry's textual fields.
    ///
    /// Tags are normalized, deduplicated keywords drawn from the context,
    /// hypothesis, and guardrail text. Short common words are filtered out.
    fn extract_tags(entry: &LearningEntry) -> Vec<String> {
        let combined = format!(
            "{} {} {}",
            entry.context, entry.hypothesis, entry.guardrail_advice
        );
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "could", "should", "may", "might",
            "shall", "can", "need", "must", "to", "of", "in", "for", "on", "with", "at", "by",
            "from", "as", "into", "through", "during", "before", "after", "above", "below",
            "between", "out", "off", "over", "under", "again", "further", "then", "once", "and",
            "but", "or", "nor", "not", "no", "so", "if", "it", "its", "this", "that", "these",
            "those", "all", "each", "every", "both", "few", "more", "most", "other", "some",
            "such", "only", "same", "than", "too", "very", "just", "don", "now", "also", "use",
            "using", "used", "via",
        ]
        .into_iter()
        .collect();

        let mut seen = std::collections::HashSet::new();
        let mut tags = Vec::new();

        for word in combined.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_') {
            let w = word.to_lowercase();
            if w.len() >= 4 && !stop_words.contains(w.as_str()) && seen.insert(w.clone()) {
                tags.push(w);
            }
        }

        tags.truncate(12);
        tags
    }

    /// Finds indices of existing entries related to a new entry.
    ///
    /// Relatedness is determined by shared affected elements, overlapping tags,
    /// or matching context keywords -- implementing Zettelkasten's association logic.
    fn find_related_indices(&self, new_entry: &LearningEntry) -> Vec<usize> {
        let new_tags: std::collections::HashSet<&str> =
            new_entry.tags.iter().map(|s| s.as_str()).collect();
        let new_elements: std::collections::HashSet<&str> = new_entry
            .affected_elements
            .iter()
            .map(|s| s.as_str())
            .collect();

        let mut scored: Vec<(usize, u32)> = Vec::new();

        for (idx, existing) in self.learnings.iter().enumerate() {
            let mut score: u32 = 0;

            let shared_elements = existing
                .affected_elements
                .iter()
                .filter(|e| {
                    new_elements.contains(e.as_str())
                        || new_elements.iter().any(|ne| {
                            ne.starts_with(&format!("{}.", e)) || e.starts_with(&format!("{}.", ne))
                        })
                })
                .count();
            score += (shared_elements as u32) * 3;

            let shared_tags = existing
                .tags
                .iter()
                .filter(|t| new_tags.contains(t.as_str()))
                .count();
            score += (shared_tags as u32) * 2;

            let ctx_lower = existing.context.to_lowercase();
            let new_ctx_lower = new_entry.context.to_lowercase();
            let ctx_words: Vec<&str> = new_ctx_lower
                .split_whitespace()
                .filter(|w| w.len() >= 4)
                .collect();
            let ctx_overlap = ctx_words.iter().filter(|w| ctx_lower.contains(*w)).count();
            score += ctx_overlap as u32;

            if score >= 2 {
                scored.push((idx, score));
            }
        }

        scored.sort_by_key(|item| std::cmp::Reverse(item.1));
        scored.truncate(5);
        scored.into_iter().map(|(idx, _)| idx).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_entry(context: &str, hypothesis: &str, elements: Vec<&str>) -> LearningEntry {
        LearningEntry {
            id: generate_entry_id(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: context.to_string(),
            hypothesis: hypothesis.to_string(),
            outcome: ExperimentOutcome::Failed,
            reason: None,
            guardrail_advice: String::new(),
            affected_elements: elements.into_iter().map(String::from).collect(),
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        }
    }

    #[test]
    fn test_utility_tracking() {
        let mut memory = AgenticMemory::default();
        let entry = make_entry("ctx", "hyp", vec!["API"]);
        let id = entry.id.clone();
        memory.add_learning(entry);

        memory.record_retrievals(&[id.as_str()]);
        memory.record_task_outcomes(&[id.as_str()], true);
        memory.record_task_outcomes(&[id.as_str()], false);

        let e = &memory.learnings[0];
        assert_eq!(e.retrieval_count, 1);
        assert_eq!(e.task_total_after, 2);
        assert_eq!(e.task_success_after, 1);
        assert_eq!(e.utility_ratio(), Some(0.5));
    }

    #[test]
    fn test_update_learning() {
        let mut memory = AgenticMemory::default();
        let entry = make_entry("old context", "hyp", vec![]);
        let id = entry.id.clone();
        memory.add_learning(entry);

        memory
            .update_learning(
                &id,
                LearningPatch {
                    context: Some("new context".to_string()),
                    guardrail_advice: Some("updated advice".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(memory.learnings[0].context, "new context");
        assert_eq!(memory.learnings[0].guardrail_advice, "updated advice");
        assert!(!memory.learnings[0].tags.is_empty());
    }

    #[test]
    fn test_delete_learning() {
        let mut memory = AgenticMemory::default();
        let e1 = make_entry("a", "h1", vec![]);
        let id1 = e1.id.clone();
        let mut e2 = make_entry("b", "h2", vec![]);
        e2.related_ids = vec![id1.clone()];
        memory.add_learning(e1);
        memory.add_learning(e2);

        memory.delete_learning(&id1).unwrap();
        assert_eq!(memory.learnings.len(), 1);
        assert!(memory.learnings[0].related_ids.is_empty());
    }

    #[test]
    fn test_merge_learnings() {
        let mut memory = AgenticMemory::default();
        let mut e1 = make_entry("shared topic A", "h1", vec!["API.Routes"]);
        e1.retrieval_count = 3;
        e1.task_success_after = 1;
        e1.task_total_after = 2;
        let id1 = e1.id.clone();
        let mut e2 = make_entry("shared topic B", "h2", vec!["API.Service"]);
        e2.retrieval_count = 2;
        let id2 = e2.id.clone();
        memory.add_learning(e1);
        memory.add_learning(e2);

        let merged_id = memory
            .merge_learnings(
                &[id1.clone(), id2.clone()],
                LearningEntry {
                    context: "Merged boundary guidance".to_string(),
                    hypothesis: "Combined".to_string(),
                    outcome: ExperimentOutcome::Success,
                    guardrail_advice: "Use service layer".to_string(),
                    ..make_entry("", "", vec![])
                },
            )
            .unwrap();

        assert_eq!(memory.learnings.len(), 1);
        assert_eq!(memory.learnings[0].id, merged_id);
        assert_eq!(memory.learnings[0].retrieval_count, 5);
        assert_eq!(memory.learnings[0].task_total_after, 0);
        assert_eq!(memory.learnings[0].task_success_after, 0);
        assert!(memory.learnings[0]
            .affected_elements
            .iter()
            .any(|e| e == "API.Routes"));
    }

    #[test]
    fn test_curation_report() {
        let mut memory = AgenticMemory::default();
        let mut e = make_entry("low utility", "h", vec![]);
        e.retrieval_count = 5;
        e.task_success_after = 0;
        e.task_total_after = 4;
        memory.add_learning(e);

        let report = memory.curation_report();
        assert_eq!(report.total_entries, 1);
        assert_eq!(report.low_utility.len(), 1);
    }

    #[test]
    fn test_add_and_find_relevant() {
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            id: "test-1".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Refactoring API".to_string(),
            hypothesis: "Test hypothesis".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "Keep doing this".to_string(),
            affected_elements: vec!["Sruja.API".to_string(), "Sruja.CLI".to_string()],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        };

        memory.add_learning(entry.clone());

        assert_eq!(memory.find_relevant("Sruja.API").len(), 1);
        assert_eq!(memory.find_relevant("Sruja.API.V1").len(), 1);
        assert_eq!(memory.find_relevant("api").len(), 1);
        assert_eq!(memory.find_relevant("Other").len(), 0);
    }

    #[test]
    fn test_auto_tag_extraction() {
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            id: "tag-test".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Boundary violation in service layer".to_string(),
            hypothesis: "Direct database access from routes".to_string(),
            outcome: ExperimentOutcome::Failed,
            reason: None,
            guardrail_advice: "Always use service layer for database queries".to_string(),
            affected_elements: vec!["API.Routes".to_string()],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        };
        memory.add_learning(entry);

        let tags = &memory.learnings[0].tags;
        assert!(!tags.is_empty(), "Tags should be auto-generated");
        assert!(
            tags.iter()
                .any(|t| t.contains("boundary") || t.contains("violation")),
            "Should extract domain-relevant tags: {:?}",
            tags
        );
    }

    #[test]
    fn test_bidirectional_linking() {
        let mut memory = AgenticMemory::default();

        let e1 = make_entry(
            "Boundary violation refactoring",
            "Move DB calls to service layer",
            vec!["API.Routes", "API.Service"],
        );
        let e2 = make_entry(
            "Another boundary violation fix",
            "Extract repository pattern",
            vec!["API.Routes", "API.Repository"],
        );

        memory.add_learning(e1);
        memory.add_learning(e2);

        let first = &memory.learnings[0];
        let second = &memory.learnings[1];

        assert!(
            !first.related_ids.is_empty() || !second.related_ids.is_empty(),
            "Entries sharing affected elements should be linked"
        );
        if !second.related_ids.is_empty() {
            assert!(second.related_ids.contains(&first.id));
        }
        if !first.related_ids.is_empty() {
            assert!(first.related_ids.contains(&second.id));
        }
    }

    #[test]
    fn test_find_cluster() {
        let mut memory = AgenticMemory::default();

        let e1 = make_entry("Boundary violation A", "hypothesis A", vec!["API.Routes"]);
        let e2 = make_entry("Boundary violation B", "hypothesis B", vec!["API.Routes"]);
        let e3 = make_entry("Unrelated topic", "hypothesis C", vec!["Database.Schema"]);

        memory.add_learning(e1);
        let id1 = memory.learnings[0].id.clone();
        memory.add_learning(e2);
        memory.add_learning(e3);

        let cluster = memory.find_cluster(&id1);
        assert!(
            cluster.len() >= 2,
            "Cluster should include linked entries, got {}",
            cluster.len()
        );

        let cluster_ids: Vec<&str> = cluster.iter().map(|e| e.id.as_str()).collect();
        assert!(cluster_ids.contains(&id1.as_str()));
    }

    #[test]
    fn test_find_by_tag() {
        let mut memory = AgenticMemory::default();
        let mut entry = make_entry("boundary violation test", "hypothesis", vec!["API"]);
        entry.tags = vec!["boundary".to_string(), "violation".to_string()];
        memory.add_learning_raw(entry);

        let results = memory.find_by_tag("boundary");
        assert_eq!(results.len(), 1);

        let results = memory.find_by_tag("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_all_tags() {
        let mut memory = AgenticMemory::default();
        let mut e1 = make_entry("ctx", "hyp", vec![]);
        e1.tags = vec!["alpha".to_string(), "beta".to_string()];
        let mut e2 = make_entry("ctx", "hyp", vec![]);
        e2.tags = vec!["beta".to_string(), "gamma".to_string()];
        memory.add_learning_raw(e1);
        memory.add_learning_raw(e2);

        let tags = memory.all_tags();
        assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            id: "save-test".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Test".to_string(),
            hypothesis: "Hypo".to_string(),
            outcome: ExperimentOutcome::Failed,
            reason: Some("Error".to_string()),
            guardrail_advice: "Don't".to_string(),
            affected_elements: vec!["ID1".to_string()],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        };

        memory.add_learning(entry);
        memory.save(repo_root).unwrap();

        let loaded = AgenticMemory::load(repo_root).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "Test");
        assert!(!loaded.learnings[0].id.is_empty(), "ID should persist");
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        let loaded = AgenticMemory::load(repo_root).unwrap();
        assert_eq!(loaded.learnings.len(), 0);
    }

    #[test]
    fn test_clear_and_exists() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();

        assert!(!AgenticMemory::exists(repo_root));

        let memory = AgenticMemory::default();
        memory.save(repo_root).unwrap();

        assert!(AgenticMemory::exists(repo_root));

        AgenticMemory::clear(repo_root).unwrap();

        assert!(!AgenticMemory::exists(repo_root));
    }

    #[test]
    fn test_load_invalid_json() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let path = AgenticMemory::get_path(repo_root);

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "invalid json").unwrap();

        let result = AgenticMemory::load(repo_root);
        assert!(matches!(result, Err(MemoryError::Serialization(_))));
    }

    #[test]
    fn test_find_relevant_edge_cases() {
        let mut memory = AgenticMemory::default();
        let entry = LearningEntry {
            id: "edge-case".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Some Context".to_string(),
            hypothesis: "".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "".to_string(),
            affected_elements: vec!["Sruja.API.V1".to_string()],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        };
        memory.add_learning(entry);

        assert_eq!(memory.find_relevant("Sruja.API").len(), 1);
        assert_eq!(memory.find_relevant("Sruja.API.V1.Endpoint").len(), 1);
        assert_eq!(memory.find_relevant("Sruja.API.V1").len(), 1);
        assert_eq!(memory.find_relevant("Sruja.API.V2").len(), 0);
    }

    #[test]
    fn test_load_save_custom_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("custom_memory.json");
        let mut memory = AgenticMemory::default();
        memory.add_learning(LearningEntry {
            id: "custom-path".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Custom".to_string(),
            hypothesis: "H".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "G".to_string(),
            affected_elements: vec![],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        });

        memory.save_to_path(&path).unwrap();
        assert!(path.exists());

        let loaded = AgenticMemory::load_from_path(&path).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "Custom");
    }

    #[test]
    fn test_learning_entry_relevance() {
        let entry = LearningEntry {
            id: "rel-test".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "API Refactoring".to_string(),
            hypothesis: "".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "".to_string(),
            affected_elements: vec!["System.Core".to_string()],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        };

        assert!(entry.is_relevant_to("System.Core"));
        assert!(entry.is_relevant_to("System.Core.UI"));
        assert!(entry.is_relevant_to("System"));
        assert!(entry.is_relevant_to("api"));
        assert!(!entry.is_relevant_to("Database"));
    }

    #[test]
    fn test_save_to_path_replaces_longer_existing_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");
        let mut memory = AgenticMemory::default();
        memory.add_learning(LearningEntry {
            id: "replace-test".to_string(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Short".to_string(),
            hypothesis: "H".to_string(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "G".to_string(),
            affected_elements: vec![],
            evidence_refs: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            hitl_kind: None,
            related_ids: Vec::new(),
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        });

        std::fs::write(&path, "{".repeat(4096)).unwrap();

        memory.save_to_path(&path).unwrap();

        let loaded = AgenticMemory::load_from_path(&path).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].context, "Short");
    }

    #[test]
    fn test_backward_compatible_deserialization() {
        let legacy_json = r#"{
            "learnings": [{
                "timestamp": "2026-01-01T00:00:00Z",
                "context": "Legacy entry",
                "hypothesis": "Old format",
                "outcome": "failed",
                "reason": null,
                "guardrail_advice": "Upgrade",
                "affected_elements": ["X"]
            }]
        }"#;
        let memory: AgenticMemory = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(memory.learnings.len(), 1);
        assert!(memory.learnings[0].tags.is_empty());
        assert!(memory.learnings[0].related_ids.is_empty());
        assert!(memory.learnings[0].kind.is_none());
        assert!(memory.learnings[0].run_id.is_none());
    }

    #[test]
    fn test_decay_score_recent_entry() {
        let entry = make_entry("ctx", "hyp", vec![]);
        let score = entry.decay_score();
        assert!(
            score > 0.9,
            "Recent entry should have high decay score, got {}",
            score
        );
    }

    #[test]
    fn test_decay_score_old_entry() {
        let mut entry = make_entry("ctx", "hyp", vec![]);
        entry.timestamp = Utc::now() - chrono::Duration::days(250);
        let score = entry.decay_score();
        assert!(
            score < 0.15,
            "Old entry should have low decay score, got {}",
            score
        );
    }

    #[test]
    fn test_decay_score_with_retrievals() {
        let mut entry = make_entry("ctx", "hyp", vec![]);
        entry.timestamp = Utc::now() - chrono::Duration::days(120);
        let score_no_retrievals = entry.decay_score();
        entry.retrieval_count = 10;
        let score_with_retrievals = entry.decay_score();
        assert!(
            score_with_retrievals > score_no_retrievals,
            "Retrievals should boost decay score: {} vs {}",
            score_with_retrievals,
            score_no_retrievals
        );
    }

    #[test]
    fn test_curation_report_includes_stale() {
        let mut memory = AgenticMemory::default();
        let mut old = make_entry("old context", "hyp", vec![]);
        old.timestamp = Utc::now() - chrono::Duration::days(250);
        old.retrieval_count = 0;
        memory.add_learning(old);

        let report = memory.curation_report();
        assert_eq!(report.stale_entries.len(), 1, "Old entry should be stale");
        assert!(report.stale_entries[0].decay_score < 0.15);
    }

    #[test]
    fn test_auto_archive_stale() {
        let mut memory = AgenticMemory::default();
        let mut old = make_entry("old", "hyp", vec![]);
        old.timestamp = Utc::now() - chrono::Duration::days(250);
        memory.add_learning(old);

        let recent = make_entry("recent", "hyp", vec![]);
        memory.add_learning(recent);

        let archived = memory.auto_archive_stale(0.15, 30);
        assert_eq!(archived.len(), 1, "Should archive old entry");
        assert_eq!(memory.learnings.len(), 1, "Recent entry should remain");
    }

    #[test]
    fn test_auto_archive_preserves_invariants() {
        let mut memory = AgenticMemory::default();
        let mut old_invariant = make_entry("invariant", "hyp", vec![]);
        old_invariant.timestamp = Utc::now() - chrono::Duration::days(250);
        old_invariant.kind = Some(LearningKind::Invariant);
        memory.add_learning(old_invariant);

        let archived = memory.auto_archive_stale(0.15, 30);
        assert_eq!(archived.len(), 0, "Should not archive invariants");
        assert_eq!(memory.learnings.len(), 1);
    }
}
