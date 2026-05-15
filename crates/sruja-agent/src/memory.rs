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
        }
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
}
