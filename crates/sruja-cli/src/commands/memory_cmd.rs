//! `sruja memory` — indexed search over learnings, context events, and decision records.

use crate::commands::CliError;
use sruja_memory::{MemoryStore, SearchMemoryOptions, TimelineOptions};
use std::path::Path;

pub fn memory_reindex(repo: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {repo}"
        )));
    }
    let mut store = MemoryStore::open(repo_path).map_err(map_memory_err)?;
    store.reindex().map_err(map_memory_err)?;
    println!(r#"{{"ok":true,"schema_version":"memory_index/v1"}}"#);
    Ok(())
}

pub fn memory_search(
    repo: &str,
    query: &str,
    element_id: Option<&str>,
    decision_id: Option<&str>,
    hitl_kind: Option<&str>,
    limit: usize,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let store = MemoryStore::open(repo_path).map_err(map_memory_err)?;
    let hits = store
        .search(SearchMemoryOptions {
            query,
            element_id,
            decision_id,
            hitl_kind,
            source: None,
            trust: None,
            limit,
        })
        .map_err(map_memory_err)?;
    let out = serde_json::json!({
        "schema_version": "memory_search/v1",
        "query": query,
        "count": hits.len(),
        "hits": hits,
        "note": "Results are hypothesis or reviewed_truth; never auto-merge into repo.sruja."
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

pub fn memory_timeline(
    repo: &str,
    anchor_id: Option<&str>,
    anchor_timestamp: Option<&str>,
    before: usize,
    after: usize,
    decision_id: Option<&str>,
    element_id: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let store = MemoryStore::open(repo_path).map_err(map_memory_err)?;
    let tl = store
        .timeline(TimelineOptions {
            anchor_id,
            anchor_timestamp,
            before,
            after,
            decision_id,
            element_id,
        })
        .map_err(map_memory_err)?;
    println!("{}", serde_json::to_string_pretty(&tl)?);
    Ok(())
}

fn map_memory_err(e: sruja_memory::MemoryStoreError) -> CliError {
    match e {
        sruja_memory::MemoryStoreError::Validation(msg) => CliError::validation(msg),
        other => CliError::Io(std::io::Error::other(other.to_string())),
    }
}
