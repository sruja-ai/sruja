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
        mem.save(path)
    }
}
