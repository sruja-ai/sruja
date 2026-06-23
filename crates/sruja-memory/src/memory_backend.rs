//! `Memory` trait implementation backed by `MemoryStore` (FTS5 + BM25).
//!
//! This module wires the SQLite-backed `MemoryStore` to the agent's
//! `Memory` trait so the live agent loop retrieves learnings via ranked
//! full-text search instead of substring `contains()`.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sruja_agent::memory::{AgenticMemory, Memory};
use sruja_agent::{LearningEntry, MemoryError};

use crate::store::{MemoryStore, SearchMemoryOptions};

/// FTS5+BM25 ranked memory backend.
///
/// Writes go through to `agent_memory.json` (the system of record).
/// Reads come from the SQLite FTS5 index, which is lazily refreshed via
/// `ensure_indexed` (source-fingerprint check).
///
/// Falls back to in-memory substring search if the SQLite DB cannot be opened.
pub struct IndexedMemory {
    /// SQLite FTS5 index. `None` when the DB could not be opened (fallback mode).
    store: Mutex<Option<MemoryStore>>,
    /// In-memory JSON learnings (the write path + fallback search).
    mem: Mutex<AgenticMemory>,
    /// Repository root (for resolving `.sruja/` paths).
    repo: PathBuf,
}

impl IndexedMemory {
    /// Open the indexed memory backend for a repository.
    ///
    /// If the SQLite DB cannot be opened, search gracefully degrades to
    /// substring matching over the in-memory JSON.
    pub fn open(repo: impl Into<PathBuf>) -> Self {
        let repo = repo.into();
        let mem = AgenticMemory::load(&repo).unwrap_or_default();
        let store = MemoryStore::open(&repo).ok();
        Self {
            store: Mutex::new(store),
            mem: Mutex::new(mem),
            repo,
        }
    }
}

impl Memory for IndexedMemory {
    fn search(&self, query: &str, limit: usize) -> Vec<LearningEntry> {
        // Try FTS5 search first; fall back to substring if unavailable.
        let indexed_results = {
            let guard = self.store.lock().unwrap();
            guard.as_ref().and_then(|s| {
                s.search(SearchMemoryOptions {
                    query,
                    limit,
                    ..Default::default()
                })
                .ok()
            })
        };

        match indexed_results {
            Some(hits) if !hits.is_empty() => {
                // Hydrate FTS5 hits into full LearningEntry objects (KD2).
                let mem = self.mem.lock().unwrap();
                let by_id: std::collections::HashMap<&str, &LearningEntry> =
                    mem.learnings.iter().map(|e| (e.id.as_str(), e)).collect();
                let mut results: Vec<LearningEntry> = hits
                    .iter()
                    .filter_map(|hit| by_id.get(hit.id.as_str()).map(|e| (*e).clone()))
                    .collect();
                // Re-rank by utility as a tiebreaker.
                results.sort_by(|a, b| {
                    let sa = a.decay_score() * a.utility_ratio().unwrap_or(0.5);
                    let sb = b.decay_score() * b.utility_ratio().unwrap_or(0.5);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                });
                results.truncate(limit);
                results
            }
            _ => {
                // Fallback: substring search (KD3).
                let mem = self.mem.lock().unwrap();
                let q = query.to_lowercase();
                let mut results: Vec<LearningEntry> = mem
                    .learnings
                    .iter()
                    .filter(|e| {
                        e.context.to_lowercase().contains(&q)
                            || e.hypothesis.to_lowercase().contains(&q)
                            || e.guardrail_advice.to_lowercase().contains(&q)
                            || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
                            || e.affected_elements.iter().any(|el| el.contains(query))
                    })
                    .cloned()
                    .collect();
                results.sort_by(|a, b| {
                    let sa = a.decay_score() * a.utility_ratio().unwrap_or(0.5);
                    let sb = b.decay_score() * b.utility_ratio().unwrap_or(0.5);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                });
                results.truncate(limit);
                results
            }
        }
    }

    fn record(&self, entry: LearningEntry) -> Result<(), MemoryError> {
        let mut mem = self.mem.lock().unwrap();
        mem.add_learning(entry);
        // Save to JSON (system of record); FTS5 index refreshes lazily.
        let mem_clone = mem.clone();
        drop(mem);
        mem_clone.save(&self.repo)
    }

    fn record_retrievals(&self, ids: &[&str]) {
        let mut mem = self.mem.lock().unwrap();
        mem.record_retrievals(ids);
    }

    fn record_outcomes(&self, ids: &[&str], success: bool) {
        let mut mem = self.mem.lock().unwrap();
        mem.record_task_outcomes(ids, success);
    }

    fn count(&self) -> usize {
        self.mem.lock().unwrap().learnings.len()
    }

    fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        let mem = self.mem.lock().unwrap();
        mem.save_to_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use tempfile::tempdir;

    fn make_entry(context: &str, hypothesis: &str, elements: Vec<&str>) -> LearningEntry {
        LearningEntry {
            id: sruja_agent::generate_entry_id(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: context.to_string(),
            hypothesis: hypothesis.to_string(),
            outcome: sruja_agent::ExperimentOutcome::Failed,
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

    fn make_entry_with_utility(
        context: &str,
        hypothesis: &str,
        elements: Vec<&str>,
        retrieval_count: u32,
        success: u32,
        total: u32,
        age_days: i64,
    ) -> LearningEntry {
        let mut entry = make_entry(context, hypothesis, elements);
        entry.retrieval_count = retrieval_count;
        entry.task_success_after = success;
        entry.task_total_after = total;
        entry.timestamp = Utc::now() - Duration::days(age_days);
        entry
    }

    #[test]
    fn indexed_memory_record_persists_to_json() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        let mem = IndexedMemory::open(repo);

        let entry = make_entry("test context", "test hypothesis", vec!["API"]);
        let id = entry.id.clone();
        mem.record(entry).unwrap();

        // Verify count.
        assert_eq!(mem.count(), 1);

        // Verify JSON file exists on disk.
        let json_path = repo.join(".sruja/agent_memory.json");
        assert!(json_path.exists(), "JSON file should be created on record");

        // Verify the entry is readable from disk.
        let loaded = AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
        assert_eq!(loaded.learnings[0].id, id);
        assert_eq!(loaded.learnings[0].context, "test context");
    }

    #[test]
    fn indexed_memory_fts5_search_returns_ranked_results() {
        let dir = tempdir().unwrap();
        let repo = dir.path();

        // Pre-populate JSON so FTS5 index has something to search.
        let mut mem = AgenticMemory::default();
        mem.add_learning(make_entry(
            "database connection pooling",
            "connection pooling reduces overhead",
            vec!["DB"],
        ));
        mem.add_learning(make_entry(
            "API rate limiting",
            "rate limiting prevents abuse",
            vec!["API"],
        ));
        mem.add_learning(make_entry(
            "database migration safety",
            "migrations should be reversible",
            vec!["DB"],
        ));
        mem.save(repo).unwrap();

        let indexed = IndexedMemory::open(repo);
        assert_eq!(indexed.count(), 3);

        // Search for "database" — should match both DB entries via FTS5.
        let results = indexed.search("database", 10);
        assert!(
            results.len() >= 2,
            "FTS5 should find both database entries, got {}",
            results.len()
        );

        // Both results should contain "database" in context or hypothesis.
        for r in &results {
            let text = format!("{} {}", r.context, r.hypothesis).to_lowercase();
            assert!(
                text.contains("database"),
                "Result should mention database: {}",
                r.context
            );
        }
    }

    #[test]
    fn indexed_memory_fts5_search_limits_results() {
        let dir = tempdir().unwrap();
        let repo = dir.path();

        let mut mem = AgenticMemory::default();
        for i in 0..5 {
            mem.add_learning(make_entry(
                &format!("context {i} with database topic"),
                &format!("hypothesis {i}"),
                vec!["DB"],
            ));
        }
        mem.save(repo).unwrap();

        let indexed = IndexedMemory::open(repo);
        let results = indexed.search("database", 3);
        assert_eq!(results.len(), 3, "Should respect limit parameter");
    }

    #[test]
    fn indexed_memory_fallback_when_no_fts5_match() {
        let dir = tempdir().unwrap();
        let repo = dir.path();

        let mut mem = AgenticMemory::default();
        mem.add_learning(make_entry(
            "rust borrow checker",
            "borrow checker prevents data races",
            vec!["Core"],
        ));
        mem.save(repo).unwrap();

        let indexed = IndexedMemory::open(repo);

        // Search for something not indexed — FTS5 returns empty, fallback kicks in.
        let results = indexed.search("borrow", 10);
        // Substring fallback should still find it.
        assert_eq!(results.len(), 1);
        assert!(results[0].context.contains("borrow"));
    }

    #[test]
    fn indexed_memory_record_retrievals_updates_counters() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        let mem = IndexedMemory::open(repo);

        let entry = make_entry("ctx", "hyp", vec![]);
        let id = entry.id.clone();
        mem.record(entry).unwrap();

        mem.record_retrievals(&[id.as_str()]);
        mem.record_retrievals(&[id.as_str()]);

        // Persist in-memory state to disk (mirrors what reflect() does).
        let json_path = repo.join(".sruja/agent_memory.json");
        mem.save_to_path(&json_path).unwrap();

        let loaded = AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings[0].retrieval_count, 2);
    }

    #[test]
    fn indexed_memory_record_outcomes_updates_counters() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        let mem = IndexedMemory::open(repo);

        let entry = make_entry("ctx", "hyp", vec![]);
        let id = entry.id.clone();
        mem.record(entry).unwrap();

        mem.record_outcomes(&[id.as_str()], true);
        mem.record_outcomes(&[id.as_str()], false);

        // Persist in-memory state to disk (mirrors what reflect() does).
        let json_path = repo.join(".sruja/agent_memory.json");
        mem.save_to_path(&json_path).unwrap();

        let loaded = AgenticMemory::load(repo).unwrap();
        assert_eq!(loaded.learnings[0].task_total_after, 2);
        assert_eq!(loaded.learnings[0].task_success_after, 1);
    }

    #[test]
    fn indexed_memory_save_to_path_persists() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        let mem = IndexedMemory::open(repo);

        let entry = make_entry("ctx", "hyp", vec![]);
        mem.record(entry).unwrap();

        // Save to a custom path.
        let custom_path = dir.path().join("custom_memory.json");
        mem.save_to_path(&custom_path).unwrap();
        assert!(custom_path.exists());

        let loaded = AgenticMemory::load_from_path(&custom_path).unwrap();
        assert_eq!(loaded.learnings.len(), 1);
    }

    #[test]
    fn indexed_memory_fts5_utility_tiebreaker() {
        let dir = tempdir().unwrap();
        let repo = dir.path();

        let mut mem = AgenticMemory::default();
        // Two entries with same text but different utility.
        let mut high_util = make_entry("database pooling", "high utility", vec![]);
        high_util.retrieval_count = 10;
        high_util.task_success_after = 9;
        high_util.task_total_after = 10;
        let mut low_util = make_entry("database pooling", "low utility", vec![]);
        low_util.retrieval_count = 10;
        low_util.task_success_after = 1;
        low_util.task_total_after = 10;
        mem.add_learning(high_util);
        mem.add_learning(low_util);
        mem.save(repo).unwrap();

        let indexed = IndexedMemory::open(repo);
        let results = indexed.search("database pooling", 10);

        // Both should be found; higher utility should rank first.
        assert_eq!(results.len(), 2);
        let first_utility = results[0].utility_ratio().unwrap_or(0.5);
        let second_utility = results[1].utility_ratio().unwrap_or(0.5);
        assert!(
            first_utility >= second_utility,
            "Higher utility entry should rank first"
        );
    }
}
